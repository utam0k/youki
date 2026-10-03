use std::collections::HashMap;
use std::io::{ErrorKind, Write};
use std::os::unix::prelude::CommandExt;
use std::path::Path;
use std::{process, thread, time};

use nix::sys::signal;
use nix::unistd::Pid;
use oci_spec::runtime::{Hook, State as OciState};

use crate::container::{State, StateConversionError};
use crate::utils;

#[derive(Debug, thiserror::Error)]
pub enum HookError {
    #[error("failed to execute hook command")]
    CommandExecute(#[source] std::io::Error),
    #[error("failed to encode container state")]
    EncodeContainerState(#[source] serde_json::Error),
    #[error(
        "hook command exited with non-zero exit code: {exit_code}, stdout: {stdout}, stderr: {stderr}"
    )]
    NonZeroExitCode {
        exit_code: i32,
        stdout: String,
        stderr: String,
    },
    #[error("hook command was killed by a signal, stdout: {stdout}, stderr: {stderr}")]
    Killed { stdout: String, stderr: String },
    #[error("failed to execute hook command due to a timeout")]
    Timeout,
    #[error("container state is required to run hook")]
    MissingContainerState,
    #[error("failed to write container state to stdin")]
    WriteContainerState(#[source] std::io::Error),
    #[error("failed to convert state to OCI format")]
    StateConversion(#[from] StateConversionError),
    #[error("error running {kind} hook #{index}: {source}")]
    Run {
        kind: HookKind,
        index: usize,
        source: Box<HookError>,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum HookKind {
    Prestart,
    CreateRuntime,
    CreateContainer,
    StartContainer,
    Poststart,
    Poststop,
}

impl std::fmt::Display for HookKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Match runc's hook names, which follow the OCI config.json keys.
        let name = match *self {
            Self::Prestart => "prestart",
            Self::CreateRuntime => "createRuntime",
            Self::CreateContainer => "createContainer",
            Self::StartContainer => "startContainer",
            Self::Poststart => "poststart",
            Self::Poststop => "poststop",
        };

        write!(f, "{name}")
    }
}

type Result<T> = std::result::Result<T, HookError>;

fn wait_for_hook_output(mut hook: process::Child, state: Vec<u8>) -> Result<process::Output> {
    let mut stdin = hook.stdin.take().expect("hook stdin must be piped");
    // A hook may fill stdout/stderr before reading its state. Write stdin in
    // parallel with wait_with_output, which drains both output pipes.
    let state_writer = thread::spawn(move || stdin.write_all(&state));
    let output = hook.wait_with_output().map_err(HookError::CommandExecute);
    let written = state_writer.join().map_err(|_| {
        HookError::WriteContainerState(std::io::Error::other("hook state writer thread panicked"))
    })?;
    if let Err(err) = written {
        // Hooks may exit without consuming their state; preserve that behavior.
        if err.kind() != ErrorKind::BrokenPipe {
            return Err(HookError::WriteContainerState(err));
        }
    }
    output
}

pub fn run_hooks(
    hooks: Option<&Vec<Hook>>,
    kind: HookKind,
    state: Option<&State>,
    // TODO: Remove the following parameters. To comply with the OCI State, hooks should only depend on structures defined in oci-spec-rs. Cleaning these up ensures proper functional isolation.
    cwd: Option<&Path>,
    pid: Option<Pid>,
    default_env: Option<&HashMap<String, String>>,
) -> Result<()> {
    let base_state = state.ok_or(HookError::MissingContainerState)?;

    // High-level container runtimes use OCI state to pass the container state to the hooks.
    // So we need to convert the container state to OCI state.
    // Ref: https://github.com/containerd/containerd/blob/v2.2.1/cmd/containerd/command/oci-hook.go#L82
    let mut oci_state = OciState::try_from(base_state)?;

    // The `pid` parameter allows overriding the PID in the state. This is needed because
    // high-level container runtimes like containerd set the PID separately for certain hooks.
    // Ref: https://github.com/containerd/containerd/blob/main/cmd/containerd/command/oci-hook.go#L90
    if let Some(override_pid) = pid {
        oci_state.set_pid(Some(override_pid.as_raw()));
    }

    if let Some(hooks) = hooks {
        for (index, hook) in hooks.iter().enumerate() {
            let result = || -> Result<()> {
                let mut hook_command = process::Command::new(hook.path());

                if let Some(cwd) = cwd {
                    hook_command.current_dir(cwd);
                }

                // Based on OCI spec, the first argument of the args vector is the
                // arg0, which can be different from the path.  For example, path
                // may be "/usr/bin/true" and arg0 is set to "true". However, rust
                // command differentiates arg0 from args, where rust command arg
                // doesn't include arg0. So we have to make the split arg0 from the
                // rest of args.
                if let Some((arg0, args)) = hook.args().as_ref().and_then(|a| a.split_first()) {
                    tracing::debug!("run_hooks arg0: {:?}, args: {:?}", arg0, args);
                    hook_command.arg0(arg0).args(args)
                } else {
                    hook_command.arg0(hook.path().display().to_string())
                };

                let envs: HashMap<String, String> = if let Some(env) = hook.env() {
                    utils::parse_env(env)
                } else if let Some(default) = default_env {
                    default.clone()
                } else {
                    HashMap::new()
                };
                tracing::debug!("run_hooks envs: {:?}", envs);

                let encoded_state =
                    serde_json::to_vec(&oci_state).map_err(HookError::EncodeContainerState)?;
                let hook_process = hook_command
                    .env_clear()
                    .envs(envs)
                    .stdin(process::Stdio::piped())
                    .stdout(process::Stdio::piped())
                    .stderr(process::Stdio::piped())
                    .spawn()
                    .map_err(HookError::CommandExecute)?;
                let hook_process_pid = Pid::from_raw(hook_process.id() as i32);

                let output = if let Some(timeout_sec) = hook.timeout() {
                    // Include state input and output collection in the timeout.
                    let (s, r) = std::sync::mpsc::channel();
                    thread::spawn(move || {
                        let res = wait_for_hook_output(hook_process, encoded_state);
                        let _ = s.send(res);
                    });
                    match r.recv_timeout(time::Duration::from_secs(timeout_sec as u64)) {
                        Ok(res) => res?,
                        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                            let _ = signal::kill(hook_process_pid, signal::Signal::SIGKILL);
                            // Reap the hook and finish its I/O before returning.
                            let _ = r.recv();
                            return Err(HookError::Timeout);
                        }
                        Err(_) => {
                            unreachable!();
                        }
                    }
                } else {
                    wait_for_hook_output(hook_process, encoded_state)?
                };

                if !output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
                    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
                    return Err(match output.status.code() {
                        Some(exit_code) => HookError::NonZeroExitCode {
                            exit_code,
                            stdout,
                            stderr,
                        },
                        None => HookError::Killed { stdout, stderr },
                    });
                }

                Ok(())
            }();
            result.map_err(|source| HookError::Run {
                kind,
                index,
                source: Box::new(source),
            })?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use std::io::{Read, Seek, SeekFrom};
    use std::{env, fs};

    use anyhow::{Context, Result, bail};
    use oci_spec::runtime::HookBuilder;
    use serial_test::serial;

    use super::*;
    use crate::container::Container;

    fn is_command_in_path(program: &str) -> bool {
        if let Ok(path) = env::var("PATH") {
            for p in path.split(':') {
                let p_str = format!("{p}/{program}");
                if fs::metadata(p_str).is_ok() {
                    return true;
                }
            }
        }
        false
    }

    // Note: the run_hook will require the use of pipe to write the container
    // state into stdin of the hook command. When cargo test runs these tests in
    // parallel with other tests, the pipe becomes flaky and often we will get
    // broken pipe or bad file descriptors. There is not much we can do and we
    // decide not to retry in the test. The most sensible way to test this is
    // ask cargo test to run these tests in serial.

    #[test]
    #[serial]
    fn test_run_hook() -> Result<()> {
        {
            let default_container: Container = Default::default();
            run_hooks(
                None,
                HookKind::CreateRuntime,
                Some(&default_container.state),
                None,
                None,
                None,
            )
            .context("Failed simple test")?;
        }

        {
            assert!(is_command_in_path("true"), "The true was not found.");
            let default_container: Container = Default::default();

            let hook = HookBuilder::default().path("true").build()?;
            let hooks = Some(vec![hook]);
            run_hooks(
                hooks.as_ref(),
                HookKind::CreateRuntime,
                Some(&default_container.state),
                None,
                None,
                None,
            )
            .context("Failed true")?;
        }

        {
            assert!(
                is_command_in_path("printenv"),
                "The printenv was not found."
            );
            // Use `printenv` to make sure the environment is set correctly.
            let default_container: Container = Default::default();
            let hook = HookBuilder::default()
                .path("bash")
                .args(vec![
                    String::from("bash"),
                    String::from("-c"),
                    String::from("printenv key > /dev/null"),
                ])
                .env(vec![String::from("key=value")])
                .build()?;
            let hooks = Some(vec![hook]);
            run_hooks(
                hooks.as_ref(),
                HookKind::CreateRuntime,
                Some(&default_container.state),
                None,
                None,
                None,
            )
            .context("Failed printenv test")?;
        }

        {
            assert!(is_command_in_path("pwd"), "The pwd was not found.");

            let tmp = tempfile::tempdir()?;

            let default_container: Container = Default::default();
            let hook = HookBuilder::default()
                .path("bash")
                .args(vec![
                    String::from("bash"),
                    String::from("-c"),
                    format!("test $(pwd) = {:?}", tmp.path()),
                ])
                .build()?;
            let hooks = Some(vec![hook]);
            run_hooks(
                hooks.as_ref(),
                HookKind::CreateRuntime,
                Some(&default_container.state),
                Some(tmp.path()),
                None,
                None,
            )
            .context("Failed pwd test")?;
        }

        {
            let default_container: Container = Default::default();
            let expected_pid = Pid::from_raw(1000);

            let hook = HookBuilder::default()
                .path("bash")
                .args(vec![
                    String::from("bash"),
                    String::from("-c"),
                    format!("cat | grep '\"pid\":{}'", expected_pid),
                ])
                .build()?;
            let hooks = Some(vec![hook]);
            run_hooks(
                hooks.as_ref(),
                HookKind::CreateRuntime,
                Some(&default_container.state),
                None,
                Some(expected_pid),
                None,
            )
            .context("Failed pid test")?;
        }

        Ok(())
    }

    #[test]
    #[serial]
    // This will test executing hook with a timeout. Since the timeout is set in
    // secs, minimally, the test will run for 1 second to trigger the timeout.
    fn test_run_hook_timeout() -> Result<()> {
        let default_container: Container = Default::default();
        // We use `tail -f /dev/null` here to simulate a hook command that hangs.
        let hook = HookBuilder::default()
            .path("tail")
            .args(vec![
                String::from("tail"),
                String::from("-f"),
                String::from("/dev/null"),
            ])
            .timeout(1)
            .build()?;
        let hooks = Some(vec![hook]);
        match run_hooks(
            hooks.as_ref(),
            HookKind::CreateRuntime,
            Some(&default_container.state),
            None,
            None,
            None,
        ) {
            Ok(_) => {
                bail!(
                    "The test expects the hook to error out with timeout. Should not execute cleanly"
                );
            }
            Err(HookError::Run { source, .. }) => match *source {
                HookError::Timeout => {}
                _ => {
                    bail!(
                        "The test expects the hook to error out with timeout. Got error: {}",
                        source
                    );
                }
            },
            Err(err) => {
                bail!(
                    "The test expects the hook to error out with timeout. Got error: {}",
                    err
                );
            }
        };

        Ok(())
    }

    #[test]
    #[serial]
    fn test_run_hook_failure_output() -> Result<()> {
        for timeout in [None, Some(2)] {
            let container = Container::default();
            let mut builder = HookBuilder::default().path("/bin/sh").args(vec![
                "sh".to_owned(),
                "-c".to_owned(),
                "printf 'hook stdout\\377'; printf 'hook stderr' >&2; exit 7".to_owned(),
            ]);
            if let Some(timeout) = timeout {
                builder = builder.timeout(timeout);
            }
            let hook = builder.build()?;
            let err = run_hooks(
                Some(&vec![hook]),
                HookKind::CreateRuntime,
                Some(&container.state),
                None,
                None,
                None,
            )
            .expect_err("hook should exit with code 7");
            let message = err.to_string();
            assert!(message.contains('7'), "missing exit code: {message}");
            assert!(message.contains("hook stdout"), "missing stdout: {message}");
            assert!(message.contains("hook stderr"), "missing stderr: {message}");
        }
        Ok(())
    }

    // Mirrors runc's `[hook fails]` integration tests in hooks.bats.
    #[test]
    #[serial]
    fn test_run_hook_failure_names_hook() -> Result<()> {
        for timeout in [None, Some(2)] {
            let container = Container::default();
            let mut hooks = Vec::new();
            for path in ["/bin/true", "/bin/false"] {
                let mut builder = HookBuilder::default().path(path);
                if let Some(timeout) = timeout {
                    builder = builder.timeout(timeout);
                }
                hooks.push(builder.build()?);
            }
            let err = run_hooks(
                Some(&hooks),
                HookKind::CreateRuntime,
                Some(&container.state),
                None,
                None,
                None,
            )
            .expect_err("second hook should fail");
            let message = err.to_string();
            assert!(
                message.contains("error running createRuntime hook #1:"),
                "missing hook kind and index: {message}"
            );
        }
        Ok(())
    }

    #[test]
    #[serial]
    fn test_run_hook_signal_output() -> Result<()> {
        let container = Container::default();
        let hook = HookBuilder::default()
            .path("/bin/sh")
            .args(vec![
                "sh".to_owned(),
                "-c".to_owned(),
                "printf 'hook stdout'; printf 'hook stderr' >&2; kill -TERM $$".to_owned(),
            ])
            .timeout(2)
            .build()?;
        let err = run_hooks(
            Some(&vec![hook]),
            HookKind::CreateRuntime,
            Some(&container.state),
            None,
            None,
            None,
        )
        .expect_err("hook should be killed by SIGTERM");
        let message = err.to_string();
        assert!(
            message.contains("signal"),
            "missing signal failure: {message}"
        );
        assert!(message.contains("hook stdout"), "missing stdout: {message}");
        assert!(message.contains("hook stderr"), "missing stderr: {message}");
        Ok(())
    }

    // Re-exec this test to give run_hooks private stdio, without changing the
    // descriptors of the multithreaded test harness.
    #[test]
    #[serial]
    fn test_hook_output_child() -> Result<()> {
        let Ok(mode) = env::var("YOUKI_TEST_HOOK_OUTPUT_CHILD") else {
            return Ok(());
        };
        let mut container = Container::default();
        container.set_annotations(Some(HashMap::from([(
            "large-state".to_owned(),
            "x".repeat(256 * 1024),
        )])));
        let finish = if mode == "timeout" {
            "exec tail -f /dev/null"
        } else {
            "test \"$(wc -c)\" -gt 262144"
        };
        let hook = HookBuilder::default()
            .path("/bin/sh")
            .args(vec![
                "sh".to_owned(),
                "-c".to_owned(),
                // Write more than a pipe buffer to both outputs before reading
                // stdin, which also contains more than a pipe buffer of data.
                format!(
                    "printf HOOK_STDOUT; head -c 131072 /dev/zero; \
                 printf HOOK_STDERR >&2; head -c 131072 /dev/zero >&2; \
                 {finish}"
                ),
            ])
            .timeout(2)
            .build()?;
        let result = run_hooks(
            Some(&vec![hook]),
            HookKind::CreateRuntime,
            Some(&container.state),
            None,
            None,
            None,
        );
        if mode == "timeout" {
            assert!(
                matches!(&result, Err(HookError::Run { source, .. })
                    if matches!(**source, HookError::Timeout)),
                "{result:?}"
            );
        } else {
            result?;
        }
        Ok(())
    }

    #[test]
    #[serial]
    fn test_run_hook_large_output_is_private() -> Result<()> {
        run_hook_output_subprocess("success")
    }

    #[test]
    #[serial]
    fn test_run_hook_timeout_with_large_state() -> Result<()> {
        run_hook_output_subprocess("timeout")
    }

    fn run_hook_output_subprocess(mode: &str) -> Result<()> {
        let mut stdout = tempfile::tempfile()?;
        let mut stderr = tempfile::tempfile()?;
        let mut child = process::Command::new(env::current_exe()?)
            .args([
                "--exact",
                "hooks::test::test_hook_output_child",
                "--nocapture",
            ])
            .env("YOUKI_TEST_HOOK_OUTPUT_CHILD", mode)
            .stdin(process::Stdio::null())
            .stdout(stdout.try_clone()?)
            .stderr(stderr.try_clone()?)
            .process_group(0)
            .spawn()?;

        // Bound even a regression that blocks while writing state, before the
        // hook's timeout handling starts. Kill the whole private process group.
        let deadline = time::Instant::now() + time::Duration::from_secs(10);
        let status = loop {
            if let Some(status) = child.try_wait()? {
                break status;
            }
            if time::Instant::now() >= deadline {
                let _ = signal::killpg(Pid::from_raw(child.id() as i32), signal::SIGKILL);
                child.wait()?;
                bail!("hook input/output deadlocked");
            }
            thread::sleep(time::Duration::from_millis(10));
        };
        stdout.seek(SeekFrom::Start(0))?;
        stderr.seek(SeekFrom::Start(0))?;
        let mut captured_stdout = Vec::new();
        let mut captured_stderr = Vec::new();
        stdout.read_to_end(&mut captured_stdout)?;
        stderr.read_to_end(&mut captured_stderr)?;
        assert!(status.success(), "hook subprocess failed: {status}");
        // The subprocess's test harness writes its summary to stdout. Hook
        // output must not be mixed into that stream or inherited stderr.
        assert!(!captured_stdout.contains(&0), "hook stdout leaked");
        assert!(
            captured_stderr.is_empty(),
            "hook stderr leaked {} bytes",
            captured_stderr.len()
        );
        Ok(())
    }

    #[test]
    #[serial]
    fn test_run_hook_default_env() -> Result<()> {
        // Test: hook without explicit env uses default_env
        {
            let default_container: Container = Default::default();
            let hook = HookBuilder::default()
                .path("sh")
                .args(vec![
                    String::from("sh"),
                    String::from("-c"),
                    String::from("test \"$TEST_ENV\" = 'default_value'"),
                ])
                .build()?;
            let hooks = Some(vec![hook]);
            let mut default_env = HashMap::new();
            default_env.insert("TEST_ENV".to_string(), "default_value".to_string());
            run_hooks(
                hooks.as_ref(),
                HookKind::CreateRuntime,
                Some(&default_container.state),
                None,
                None,
                Some(&default_env),
            )
            .context("Failed: hook without explicit env should use default_env")?;
        }

        // Test: hook with explicit env takes priority over default_env
        {
            let default_container: Container = Default::default();
            let hook = HookBuilder::default()
                .path("sh")
                .args(vec![
                    String::from("sh"),
                    String::from("-c"),
                    String::from("test \"$TEST_ENV\" = 'explicit_value'"),
                ])
                .env(vec![String::from("TEST_ENV=explicit_value")])
                .build()?;
            let hooks = Some(vec![hook]);
            let mut default_env = HashMap::new();
            default_env.insert("TEST_ENV".to_string(), "default_value".to_string());
            run_hooks(
                hooks.as_ref(),
                HookKind::CreateRuntime,
                Some(&default_container.state),
                None,
                None,
                Some(&default_env),
            )
            .context("Failed: hook with explicit env should ignore default_env")?;
        }

        // Test: hook without explicit env and no default_env gets empty environment
        {
            let default_container: Container = Default::default();
            let hook = HookBuilder::default()
                .path("sh")
                .args(vec![
                    String::from("sh"),
                    String::from("-c"),
                    // Verify that the environment is empty (no TEST_ENV, etc.)
                    String::from("test -z \"$TEST_ENV\""),
                ])
                .build()?;
            let hooks = Some(vec![hook]);
            run_hooks(
                hooks.as_ref(),
                HookKind::CreateRuntime,
                Some(&default_container.state),
                None,
                None,
                None,
            )
            .context("Failed: hook without env and without default_env should have empty env")?;
        }

        Ok(())
    }
}

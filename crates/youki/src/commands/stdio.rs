use std::fs::File;
use std::os::fd::{AsFd, AsRawFd, OwnedFd};
use std::sync::Arc;
use std::{io, thread};

use libcontainer::container::builder::ContainerBuilder;
use nix::fcntl::OFlag;
use nix::sys::eventfd::{EfdFlags, EventFd};
use nix::sys::signal::{SigSet, SigmaskHow};
use nix::unistd::{self, pipe2};

use super::foreground::relay_output;

// Pipe endpoint ownership (arrows show the intended data flow):
//
//             HostStdio                     ContainerStdio
// stdin:      write end  -----------------> read end
// stdout:     read end   <----------------- write end
// stderr:     read end   <----------------- write end
pub(crate) fn create_stdio_pipes() -> nix::Result<(HostStdio, ContainerStdio)> {
    let (stdin_read, stdin_write) = pipe2(OFlag::O_CLOEXEC)?;
    let (stdout_read, stdout_write) = pipe2(OFlag::O_CLOEXEC)?;
    let (stderr_read, stderr_write) = pipe2(OFlag::O_CLOEXEC)?;

    Ok((
        HostStdio {
            stdin: stdin_write,
            stdout: stdout_read,
            stderr: stderr_read,
        },
        ContainerStdio {
            stdin: stdin_read,
            stdout: stdout_write,
            stderr: stderr_write,
        },
    ))
}

fn start_stdin_relay(stdin: OwnedFd) -> thread::JoinHandle<io::Result<u64>> {
    thread::spawn(move || {
        let stdin_fd = io::stdin().as_fd().try_clone_to_owned()?;
        let mut reader = File::from(stdin_fd);
        let mut writer = File::from(stdin);
        io::copy(&mut reader, &mut writer)
    })
}

fn start_stdout_relay(stdout: OwnedFd, stop: Arc<EventFd>) -> thread::JoinHandle<io::Result<()>> {
    thread::spawn(move || {
        let mut reader = File::from(stdout);
        let stdout_fd = io::stdout().as_fd().try_clone_to_owned()?;
        let mut writer = File::from(stdout_fd);
        relay_output(&mut reader, &mut writer, &stop)
    })
}

fn start_stderr_relay(stderr: OwnedFd, stop: Arc<EventFd>) -> thread::JoinHandle<io::Result<()>> {
    thread::spawn(move || {
        let mut reader = File::from(stderr);
        let stderr_fd = io::stderr().as_fd().try_clone_to_owned()?;
        let mut writer = File::from(stderr_fd);
        relay_output(&mut reader, &mut writer, &stop)
    })
}

pub(crate) struct HostStdio {
    pub stdin: OwnedFd,
    pub stdout: OwnedFd,
    pub stderr: OwnedFd,
}

// Keep this guard alive while the container runs. Dropping it wakes both output
// threads, drains the available output, and waits for the threads to finish.
pub(crate) struct StdioRelay {
    stop: Arc<EventFd>,
    stdout: Option<thread::JoinHandle<io::Result<()>>>,
    stderr: Option<thread::JoinHandle<io::Result<()>>>,
}

impl Drop for StdioRelay {
    fn drop(&mut self) {
        // Neither reader consumes this event, so both threads observe the stop request.
        let _ = self.stop.arm();
        for (stream, handle) in [("stdout", &mut self.stdout), ("stderr", &mut self.stderr)] {
            if let Some(handle) = handle.take() {
                match handle.join() {
                    Ok(Ok(())) => {}
                    Ok(Err(err)) => tracing::warn!(stream, ?err, "stdio relay failed"),
                    Err(_) => tracing::warn!(stream, "stdio relay thread panicked"),
                }
            }
        }
    }
}

impl HostStdio {
    pub(crate) fn start(self) -> io::Result<StdioRelay> {
        let stop = Arc::new(EventFd::from_flags(EfdFlags::EFD_CLOEXEC)?);
        let _signal_mask = SignalMaskGuard::block()?;
        drop(start_stdin_relay(self.stdin));
        Ok(StdioRelay {
            stdout: Some(start_stdout_relay(self.stdout, Arc::clone(&stop))),
            stderr: Some(start_stderr_relay(self.stderr, Arc::clone(&stop))),
            stop,
        })
    }
}

pub(crate) struct ContainerStdio {
    pub stdin: OwnedFd,
    pub stdout: OwnedFd,
    pub stderr: OwnedFd,
}

impl ContainerStdio {
    pub(crate) fn apply_to(self, builder: ContainerBuilder) -> ContainerBuilder {
        builder
            .with_stdin(self.stdin)
            .with_stdout(self.stdout)
            .with_stderr(self.stderr)
    }

    pub(crate) fn set_owner(&self, uid: unistd::Uid, gid: unistd::Gid) -> nix::Result<()> {
        unistd::fchown(self.stdin.as_raw_fd(), Some(uid), Some(gid))?;
        unistd::fchown(self.stdout.as_raw_fd(), Some(uid), Some(gid))?;
        unistd::fchown(self.stderr.as_raw_fd(), Some(uid), Some(gid))?;
        Ok(())
    }
}

struct SignalMaskGuard {
    previous: SigSet,
}

impl SignalMaskGuard {
    fn block() -> nix::Result<Self> {
        let previous = SigSet::all().thread_swap_mask(SigmaskHow::SIG_BLOCK)?;
        Ok(Self { previous })
    }
}

impl Drop for SignalMaskGuard {
    fn drop(&mut self) {
        if let Err(err) = self.previous.thread_swap_mask(SigmaskHow::SIG_SETMASK) {
            tracing::warn!(?err, "failed to restore signal mask");
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Seek, SeekFrom, Write};
    use std::sync::mpsc::{self, Receiver};
    use std::time::Duration;

    use anyhow::{Context, Result};
    use nix::errno::Errno;
    use nix::unistd::write;

    use super::*;

    struct TestOutput {
        handle: thread::JoinHandle<io::Result<()>>,
        writer: File,
        capture: File,
        finished: Receiver<()>,
    }

    // Use a private pipe and file instead of changing the test process's stdio.
    fn output_pipe(stop: Arc<EventFd>) -> Result<TestOutput> {
        let (reader, writer) = pipe2(OFlag::O_CLOEXEC)?;
        let capture = tempfile::tempfile()?;
        let mut output = capture.try_clone()?;
        let (finished_tx, finished) = mpsc::channel();
        let handle = thread::spawn(move || {
            let result = relay_output(&mut File::from(reader), &mut output, &stop);
            let _ = finished_tx.send(());
            result
        });
        Ok(TestOutput {
            handle,
            writer: File::from(writer),
            capture,
            finished,
        })
    }

    fn check_drop_with_open_writers(stdout_data: &[u8], stderr_data: &[u8]) -> Result<()> {
        let stop = Arc::new(EventFd::from_flags(EfdFlags::EFD_CLOEXEC)?);
        let mut stdout = output_pipe(Arc::clone(&stop))?;
        let mut stderr = output_pipe(Arc::clone(&stop))?;
        stdout.writer.write_all(stdout_data)?;
        stderr.writer.write_all(stderr_data)?;
        let relay = StdioRelay {
            stop,
            stdout: Some(stdout.handle),
            stderr: Some(stderr.handle),
        };

        let (dropped_tx, dropped_rx) = mpsc::channel();
        let drop_thread = thread::spawn(move || {
            drop(relay);
            let _ = dropped_tx.send(());
        });
        // Keeping both writers open models descendants holding stdio after init exits.
        let stopped = dropped_rx.recv_timeout(Duration::from_secs(5)).is_ok();
        let stdout_closed = write(&stdout.writer, b"unexpected output");
        let stderr_closed = write(&stderr.writer, b"unexpected output");
        // Release EOF on failure too, so a broken stop notification does not strand workers.
        drop(stdout.writer);
        drop(stderr.writer);
        drop_thread.join().expect("drop thread panicked");

        assert!(stopped, "output relay waited for EOF despite being dropped");
        assert_eq!(
            stdout_closed,
            Err(Errno::EPIPE),
            "stdout reader is still open"
        );
        assert_eq!(
            stderr_closed,
            Err(Errno::EPIPE),
            "stderr reader is still open"
        );
        let mut actual_stdout = Vec::new();
        let mut actual_stderr = Vec::new();
        stdout.capture.seek(SeekFrom::Start(0))?;
        stderr.capture.seek(SeekFrom::Start(0))?;
        stdout.capture.read_to_end(&mut actual_stdout)?;
        stderr.capture.read_to_end(&mut actual_stderr)?;
        assert_eq!(actual_stdout, stdout_data);
        assert_eq!(actual_stderr, stderr_data);
        Ok(())
    }

    #[test]
    fn drop_stops_idle_output_without_eof() -> Result<()> {
        check_drop_with_open_writers(b"", b"")
    }

    #[test]
    fn drop_drains_both_outputs_without_eof() -> Result<()> {
        check_drop_with_open_writers(b"last stdout bytes\0without newline", b"last stderr bytes")
    }

    #[test]
    fn stop_request_drains_more_than_one_buffer() -> Result<()> {
        // A file keeps all data ready before the stop request without depending
        // on the kernel's pipe capacity or the output thread's scheduling.
        let expected = vec![b'x'; 128 * 1024];
        let mut input = tempfile::tempfile()?;
        input.write_all(&expected)?;
        input.seek(SeekFrom::Start(0))?;
        let mut output = tempfile::tempfile()?;
        let stop = EventFd::from_flags(EfdFlags::EFD_CLOEXEC)?;
        stop.arm()?;

        relay_output(&mut input, &mut output, &stop)?;

        let mut actual = Vec::new();
        output.seek(SeekFrom::Start(0))?;
        output.read_to_end(&mut actual)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn output_relay_copies_until_eof() -> Result<()> {
        let stop = Arc::new(EventFd::from_flags(EfdFlags::EFD_CLOEXEC)?);
        let mut output = output_pipe(stop)?;
        let expected: Vec<u8> = (0..256).cycle().take(128 * 1024).map(|n| n as u8).collect();
        output.writer.write_all(&expected)?;
        drop(output.writer);
        output
            .finished
            .recv_timeout(Duration::from_secs(5))
            .context("output relay did not finish at EOF")?;
        output.handle.join().expect("output thread panicked")?;

        let mut actual = Vec::new();
        output.capture.seek(SeekFrom::Start(0))?;
        output.capture.read_to_end(&mut actual)?;
        assert_eq!(actual, expected);
        Ok(())
    }
}

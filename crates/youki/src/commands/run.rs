use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use libcontainer::container::builder::ContainerBuilder;
use libcontainer::namespaces::Namespaces;
use libcontainer::process::user_namespaces::read_userns_map;
use libcontainer::syscall::syscall::SyscallType;
use liboci_cli::Run;
use nix::unistd;
use oci_spec::runtime::{LinuxIdMapping, LinuxIdMappingBuilder, LinuxNamespace, Spec};

use crate::commands::{foreground, stdio};
use crate::workload::executor::default_executor;

pub fn run(args: Run, root_path: PathBuf, systemd_cgroup: bool) -> Result<i32> {
    let mut builder = ContainerBuilder::new(args.container_id.clone(), SyscallType::default())
        .with_executor(default_executor())
        .with_pid_file(args.pid_file.as_ref())?
        .with_console_socket(args.console_socket.as_ref())
        .with_root_path(root_path)?
        .with_preserved_fds(args.preserve_fds)
        .validate_id()?;

    let spec = Spec::load(args.bundle.join("config.json"))?;
    let process = spec.process().as_ref().context("missing process")?;
    let terminal = process.terminal().unwrap_or(false);

    let host_stdio = if !args.detach && !terminal {
        let (host_stdio, container_stdio) = stdio::create_stdio_pipes()?;

        // Set the pipe owner so the container user can reopen its stdio.
        // Preserve the existing GID to match runc.
        let uid = process.user().uid();
        let linux = spec.linux().as_ref().context("missing linux")?;
        let namespaces = Namespaces::try_from(linux.namespaces().as_ref())?;
        let user_ns = namespaces.get(oci_spec::runtime::LinuxNamespaceType::User)?;
        let creates_user_ns = user_ns.is_some_and(|ns| ns.path().is_none());
        if user_ns.is_some() && creates_user_ns {
            let mappings = linux
                .uid_mappings()
                .as_deref()
                .context("missing UID mappings")?;
            let gid_mappings = linux
                .gid_mappings()
                .as_deref()
                .context("missing GID mappings")?;

            let uid_in_host =
                map_to_host_id(uid, mappings).context("container UID is not maped")?;
            // Use the container root's GID to match runc.
            let root_gid_in_host =
                map_to_host_id(0, gid_mappings).context("container root GID is not mapped")?;

            container_stdio.set_owner(
                unistd::Uid::from_raw(uid_in_host),
                unistd::Gid::from_raw(root_gid_in_host),
            )?;
        } else if let Some(ns) = user_ns {
            let (uid_mappings, gid_mappings) =
                read_id_mappings(&ns).context("failed to read ID mappings")?;
            let uid_in_host =
                map_to_host_id(uid, &uid_mappings).context("container UID is not mapped")?;
            // Use the container root's GID to match runc.
            let root_gid_in_host =
                map_to_host_id(0, &gid_mappings).context("container root GID is not mapped")?;
            container_stdio.set_owner(
                unistd::Uid::from_raw(uid_in_host),
                unistd::Gid::from_raw(root_gid_in_host),
            )?;
        } else {
            container_stdio.set_owner(unistd::Uid::from_raw(uid), unistd::Gid::from_raw(0))?;
        }

        builder = container_stdio.apply_to(builder);
        Some(host_stdio)
    } else {
        None
    };

    let (mut container, foreground_pty_fd) = builder
        .as_init(&args.bundle)
        .with_systemd(systemd_cgroup)
        .with_detach(args.detach)
        .with_no_pivot(args.no_pivot)
        .build()?;

    container
        .start()
        .with_context(|| format!("failed to start container {}", args.container_id))?;

    if args.detach {
        return Ok(0);
    }

    // Using `debug_assert` here rather than returning an error because this is
    // a invariant. The design when the code path arrives to this point, is that
    // the container state must have recorded the container init pid.
    debug_assert!(
        container.pid().is_some(),
        "expects a container init pid in the container state"
    );
    let foreground_result =
        foreground::handle_foreground(container.pid().unwrap(), foreground_pty_fd, host_stdio);
    // execute the destruction action after the container finishes running
    container.delete(true)?;
    // return result
    foreground_result
}

fn map_to_host_id(id_in_container: u32, mappings: &[LinuxIdMapping]) -> Option<u32> {
    for m in mappings {
        let Some(offset) = id_in_container.checked_sub(m.container_id()) else {
            continue;
        };
        if offset < m.size() {
            return m.host_id().checked_add(offset);
        }
    }
    None
}

fn read_id_mappings(ns: &LinuxNamespace) -> Result<(Vec<LinuxIdMapping>, Vec<LinuxIdMapping>)> {
    let ns_path = ns
        .path()
        .as_deref()
        .context("missing user namespace path")?
        .to_str();

    let (uid_map_text, gid_map_text) = if let Some(pid) = ns_path.and_then(|ns_path| {
        ns_path
            .strip_prefix("/proc/")
            .and_then(|ns_path| ns_path.strip_suffix("/ns/user"))
            .map(|pid_text| pid_text.parse::<u32>().ok())
            .flatten()
            .filter(|pid| *pid > 0)
    }) {
        (
            fs::read_to_string(format!("/proc/{pid}/uid_map")).context("failed to read uid_map")?,
            fs::read_to_string(format!("/proc/{pid}/gid_map")).context("failed to read gid_map")?,
        )
    } else {
        (
            read_userns_map(ns, "uid_map")?,
            read_userns_map(ns, "gid_map")?,
        )
    };

    let parse = |text: &str| -> Result<Vec<LinuxIdMapping>> {
        let mut mappings = Vec::new();
        for line in text.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            anyhow::ensure!(parts.len() == 3, "invalid ID mapping: {line}");

            let container_id = parts[0].parse::<u32>().context("invalid container ID")?;
            let host_id = parts[1].parse::<u32>().context("invalid host ID")?;
            let size = parts[2].parse::<u32>().context("invalid mapping size")?;
            mappings.push(
                LinuxIdMappingBuilder::default()
                    .container_id(container_id)
                    .host_id(host_id)
                    .size(size)
                    .build()?,
            );
        }

        Ok(mappings)
    };

    Ok((parse(&uid_map_text)?, parse(&gid_map_text)?))
}

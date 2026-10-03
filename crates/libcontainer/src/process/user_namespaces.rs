use std::{
    fs::File,
    io::{self, Read},
    path::PathBuf,
};

use oci_spec::runtime::{LinuxNamespace, LinuxNamespaceType};

use nix::{errno::Errno, fcntl::OFlag, sys::wait, unistd::pipe2};

use crate::{
    namespaces::{NamespaceError, Namespaces},
    process::fork,
};

#[derive(Debug, thiserror::Error)]
pub enum UserNamespaceReadError {
    #[error(transparent)]
    Nix(#[from] nix::Error),
    #[error(transparent)]
    Clone(#[from] fork::CloneError),
    #[error(transparent)]
    IO(#[from] io::Error),
    #[error(transparent)]
    Namespace(#[from] NamespaceError),
    #[error("user namespace helper failed with {status:?}: {message}")]
    ChildFailed {
        status: wait::WaitStatus,
        message: String,
    },
    #[error("invalid namespace type: expected {expected:?}, got {actual:?}")]
    InvalidNamespaceType {
        expected: LinuxNamespaceType,
        actual: LinuxNamespaceType,
    },
    #[error("invalid map file: expected 'uid_map' or 'gid_map', got {0}")]
    InvalidMapFile(String),
    #[error("missing user namespace path")]
    MissingPath,
}

pub fn read_userns_map(
    user_ns: &LinuxNamespace,
    map_file: &str,
) -> Result<String, UserNamespaceReadError> {
    if user_ns.typ() != LinuxNamespaceType::User {
        return Err(UserNamespaceReadError::InvalidNamespaceType {
            expected: LinuxNamespaceType::User,
            actual: user_ns.typ(),
        });
    }
    if map_file != "uid_map" && map_file != "gid_map" {
        return Err(UserNamespaceReadError::InvalidMapFile(map_file.to_string()));
    }

    let (data_read, data_write) = pipe2(OFlag::O_CLOEXEC)?;
    let (error_read, error_write) = pipe2(OFlag::O_CLOEXEC)?;
    let mut data_read = Some(data_read);
    let mut error_read = Some(error_read);

    user_ns
        .path()
        .as_ref()
        .ok_or(UserNamespaceReadError::MissingPath)?;
    let ns = Namespaces::try_from(Some(vec![user_ns.clone()].as_ref()))?;
    let mut dw = Some(File::from(data_write));
    let mut ew = File::from(error_write);
    let child_pid = super::fork::container_clone(Box::new(|| {
        drop(data_read.take());
        drop(error_read.take());
        let mut dw = dw.take().expect("data write pipe should be open");
        match ns.unshare_or_setns(user_ns) {
            Ok(_) => {
                let id_map = PathBuf::from("/proc/self").join(map_file);
                let mut file = match File::open(id_map) {
                    Err(err) => {
                        drop(dw);
                        let _ = io::copy(&mut err.to_string().as_bytes(), &mut ew);
                        return 1;
                    }
                    Ok(file) => file,
                };
                if let Err(err) = io::copy(&mut file, &mut dw) {
                    drop(dw);
                    let _ = io::copy(&mut err.to_string().as_bytes(), &mut ew);
                    return 1;
                }

                return 0;
            }
            Err(err) => {
                drop(dw);
                let _ = io::copy(&mut err.to_string().as_bytes(), &mut ew);
                return 1;
            }
        }
    }))?;

    drop(dw);
    drop(ew);
    let data_read = data_read.take().expect("data read pipe should be open");
    let error_read = error_read.take().expect("error read pipe should be open");

    let (mut buf, mut err_buf) = (String::new(), String::new());
    let result = File::from(data_read).read_to_string(&mut buf);
    let err_result = File::from(error_read).read_to_string(&mut err_buf);

    loop {
        match wait::waitpid(child_pid, None) {
            Ok(status @ wait::WaitStatus::Exited(_, code)) => {
                if code != 0 {
                    return Err(UserNamespaceReadError::ChildFailed {
                        status,
                        message: err_buf,
                    });
                }
                break;
            }
            Ok(status) => {
                return Err(UserNamespaceReadError::ChildFailed {
                    status,
                    message: "unexpected wait status".to_owned(),
                });
            }
            Err(Errno::EINTR) => continue,
            Err(err) => {
                return Err(UserNamespaceReadError::Nix(err));
            }
        }
    }
    result?;
    err_result?;

    Ok(buf)
}

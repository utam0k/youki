use std::path::PathBuf;
use std::process::Command;

use nix::sys::stat::stat;
use nix::sys::stat::SFlag;

fn test_file_read_access(path: &str) -> Result<(), std::io::Error> {
    let _ = std::fs::OpenOptions::new()
        .create(false)
        .read(true)
        .open(path)?;
    Ok(())
}

fn test_dir_read_access(path: &str) -> Result<(), std::io::Error> {
    let _ = std::fs::read_dir(path)?;
    Ok(())
}

fn is_file_like(mode: u32) -> bool {
    // for this please refer
    // https://stackoverflow.com/questions/40163270/what-is-s-isreg-and-what-does-it-do
    // https://linux.die.net/man/2/stat
    mode & SFlag::S_IFREG.bits() != 0 || mode & SFlag::S_IFCHR.bits() != 0
}

fn is_dir(mode: u32) -> bool {
    mode & SFlag::S_IFDIR.bits() != 0
}

pub fn test_read_access(path: &str) -> Result<(), std::io::Error> {
    let fstat = stat(path)?;
    let mode = fstat.st_mode;
    if is_file_like(mode) {
        // we have a file or a char/block device
        return test_file_read_access(path);
    } else if is_dir(mode) {
        return test_dir_read_access(path);
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        format!("cannot test read access for {path:?}, has mode {mode:x}"),
    ))
}

fn test_file_write_access(path: &str) -> Result<(), std::io::Error> {
    let _ = std::fs::OpenOptions::new().write(true).open(path)?;
    Ok(())
}

fn test_dir_write_access(path: &str) -> Result<(), std::io::Error> {
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(PathBuf::from(path).join("test.txt"))?;
    Ok(())
}

pub fn test_write_access(path: &str) -> Result<(), std::io::Error> {
    let fstat = stat(path)?;
    let mode = fstat.st_mode;
    if is_file_like(mode) {
        // we have a file or a char/block device
        return test_file_write_access(path);
    } else if is_dir(mode) {
        return test_dir_write_access(path);
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        format!("cannot test write access for {path:?}, has mode {mode:x}"),
    ))
}

pub fn test_file_executable(path: &str) -> Result<(), std::io::Error> {
    let fstat = stat(path)?;
    let mode = fstat.st_mode;
    if is_file_like(mode) {
        Command::new(path).output()?;
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        format!("{path:?} is directory, so cannot execute"),
    ))
}

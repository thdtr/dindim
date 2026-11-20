use std::fs::{self, File, TryLockError};
use std::path::Path;

/// Held for the whole run so a second copy of the app can't write to the
/// same files. The operating system releases the lock if the process dies.
#[derive(Debug)]
pub struct InstanceLock {
    _file: File,
}

#[derive(Debug)]
pub enum LockError {
    AlreadyRunning,
    Io(std::io::Error),
}

impl InstanceLock {
    pub fn acquire(path: &Path) -> Result<Self, LockError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(LockError::Io)?;
        }
        let file = File::options()
            .create(true)
            .truncate(false)
            .write(true)
            .open(path)
            .map_err(LockError::Io)?;
        match file.try_lock() {
            Ok(()) => Ok(InstanceLock { _file: file }),
            Err(TryLockError::WouldBlock) => Err(LockError::AlreadyRunning),
            Err(TryLockError::Error(e)) => Err(LockError::Io(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_second_lock_is_refused_until_the_first_is_dropped() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("app.lock");
        let first = InstanceLock::acquire(&path).unwrap();
        assert!(matches!(
            InstanceLock::acquire(&path),
            Err(LockError::AlreadyRunning)
        ));
        drop(first);
        assert!(InstanceLock::acquire(&path).is_ok());
    }
}

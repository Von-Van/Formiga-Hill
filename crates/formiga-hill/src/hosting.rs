//! One colony at a time. While a Hill window is open it holds a lock in Hill's data folder; a
//! trip that arrives meanwhile is refused as busy, so Desktop knows to try again later rather
//! than wait, and two windows never write the same memories. The lock is the operating system's,
//! so it goes with the window however that closes.

use std::fs::{File, OpenOptions, TryLockError};
use std::path::Path;

const LOCK: &str = "hosting.lock";

/// Held for as long as this Hill is hosting a visit.
pub struct Hosting {
    /// Nothing, if there was nowhere to keep a lock.
    _lock: Option<File>,
}

/// Takes the lock in `data`, or says another Hill has it. Anything else that goes wrong (a data
/// folder that cannot be made, a filesystem without locks) is not another Hill, and nothing is
/// refused for it.
pub fn take(data: &Path) -> Result<Hosting, Busy> {
    let opened = std::fs::create_dir_all(data).and_then(|()| {
        OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(data.join(LOCK))
    });
    let Ok(file) = opened else {
        return Ok(Hosting { _lock: None });
    };
    match file.try_lock() {
        Err(TryLockError::WouldBlock) => Err(Busy),
        Ok(()) | Err(TryLockError::Error(_)) => Ok(Hosting { _lock: Some(file) }),
    }
}

/// Another Hill is hosting a colony already.
#[derive(Debug, PartialEq, Eq)]
pub struct Busy;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_second_hill_finds_the_first_hosting_until_it_closes() {
        let data = std::env::temp_dir().join(format!("hill-hosting-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&data);
        let first = take(&data).expect("nobody else is hosting");
        assert_eq!(take(&data).err(), Some(Busy));
        drop(first);
        assert!(take(&data).is_ok(), "the lock outlived its window");
    }
}

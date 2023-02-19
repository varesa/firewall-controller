use anyhow::Error;
use nix::sched::{setns, CloneFlags};
use std::fs::File;
use std::os::fd::AsRawFd;

pub struct NetworkNamespace {
    file: File,
}

impl NetworkNamespace {
    pub fn from_file(file: File) -> Self {
        Self { file }
    }

    pub fn run<F, T>(&self, func: F) -> Result<T, Error>
    where
        F: FnOnce() -> T,
        F: Send + 'static,
        T: Send + 'static,
    {
        let fd = self.file.as_raw_fd();
        let thread = std::thread::spawn(move || {
            setns(fd, CloneFlags::CLONE_NEWNET).expect("Failed to switch netns");
            func()
        });
        let result = thread.join().expect("Failed to join thread");
        Ok(result)
    }
}

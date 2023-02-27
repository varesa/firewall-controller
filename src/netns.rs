use nix::sched::{setns, CloneFlags};
use std::fs::File;
use std::os::fd::{AsRawFd, RawFd};

pub struct NetworkNamespace {
    file: File,
}

impl NetworkNamespace {
    pub fn from_file(file: File) -> Self {
        Self { file }
    }

    pub fn run<F, T>(&self, func: F) -> T
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
        thread.join().expect("Failed to join thread")
    }

    pub fn raw_fd(&self) -> RawFd {
        self.file.as_raw_fd()
    }
}

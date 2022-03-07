use std::fs::File;
use std::os::unix::io::AsRawFd;
#[cfg(target_os = "macos")]
use std::path::Path;

pub fn set_lock(rootfs: &str) -> File {
    let lock_path = format!("{}/.carrier.lock", rootfs);
    let file = File::create(lock_path).expect("Couldn't create lock file");

    let ret = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
    if ret < 0 {
        println!("Couldn't acquire lock file. Is another instance of this VM already running?");
        std::process::exit(-1);
    }

    file
}
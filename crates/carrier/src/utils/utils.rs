use std::collections::HashMap;
use std::path::Path;
use crate::configs::VmConfig;
use std::ffi::CString;
#[cfg(target_os = "linux")]
use std::io::{Error, ErrorKind};
#[cfg(target_os = "macos")]
use std::path::Path;

pub fn parse_mapped_ports(port_matches: Vec<&str>) -> HashMap<String, String> {
    let mut mapped_ports = HashMap::new();
    for port in port_matches.iter() {
        let vtuple: Vec<&str> = port.split(':').collect();
        if vtuple.len() != 2 {
            println!("Invalid value for \"port\"");
            std::process::exit(-1);
        }
        let host_port: u16 = match vtuple[0].parse() {
            Ok(p) => p,
            Err(_) => {
                println!("Invalid host port");
                std::process::exit(-1);
            }
        };
        let guest_port: u16 = match vtuple[1].parse() {
            Ok(p) => p,
            Err(_) => {
                println!("Invalid guest port");
                std::process::exit(-1);
            }
        };

        mapped_ports.insert(host_port.to_string(), guest_port.to_string());
    }

    mapped_ports
}

pub fn parse_mapped_volumes(volume_matches: Vec<&str>) -> HashMap<String, String> {
    let mut mapped_volumes = HashMap::new();
    for volume in volume_matches.iter() {
        let vtuple: Vec<&str> = volume.split(':').collect();
        if vtuple.len() != 2 {
            println!("Invalid value for \"volume\"");
            std::process::exit(-1);
        }
        let host_path = Path::new(vtuple[0]);
        if !host_path.is_absolute() {
            println!("Invalid volume, host_path is not an absolute path");
            std::process::exit(-1);
        }
        if !host_path.exists() {
            println!("Invalid volume, host_path does not exists");
            std::process::exit(-1);
        }
        let guest_path = Path::new(vtuple[1]);
        if !guest_path.is_absolute() {
            println!("Invalid volume, guest_path is not an absolute path");
            std::process::exit(-1);
        }
        if guest_path.components().count() != 2 {
            println!(
                "Invalid volume, only single direct root children are supported as guest_path"
            );
            std::process::exit(-1);
        }
        mapped_volumes.insert(
            host_path.to_str().unwrap().to_string(),
            guest_path.to_str().unwrap().to_string(),
        );
    }

    mapped_volumes
}


#[cfg(target_os = "linux")]
pub fn map_volumes(_ctx: u32, vmcfg: &VmConfig, rootfs: &str) {
    for (host_path, guest_path) in vmcfg.mapped_volumes.iter() {
        let host_dir = CString::new(host_path.to_string()).unwrap();
        let guest_dir = CString::new(format!("{}{}", rootfs, guest_path)).unwrap();

        let ret = unsafe { libc::mkdir(guest_dir.as_ptr(), 0o755) };
        if ret < 0 && Error::last_os_error().kind() != ErrorKind::AlreadyExists {
            println!("Error creating directory {:?}", guest_dir);
            std::process::exit(-1);
        }
        unsafe { libc::umount(guest_dir.as_ptr()) };
        let ret = unsafe {
            libc::mount(
                host_dir.as_ptr(),
                guest_dir.as_ptr(),
                std::ptr::null(),
                libc::MS_BIND | libc::MS_REC,
                std::ptr::null(),
            )
        };
        if ret < 0 {
            println!("Error mounting volume {}", guest_path);
            std::process::exit(-1);
        }
    }
}

#[cfg(target_os = "macos")]
pub fn map_volumes(ctx: u32, vmcfg: &VmConfig, rootfs: &str) {
    let mut volumes = Vec::new();
    for (host_path, guest_path) in vmcfg.mapped_volumes.iter() {
        let full_guest = format!("{}{}", &rootfs, guest_path);
        let full_guest_path = Path::new(&full_guest);
        if !full_guest_path.exists() {
            std::fs::create_dir(full_guest_path)
                .expect("Couldn't create guest_path for mapped volume");
        }
        let map = format!("{}:{}", host_path, guest_path);
        volumes.push(CString::new(map).unwrap());
    }
    let mut vols: Vec<*const i8> = Vec::new();
    for vol in volumes.iter() {
        vols.push(vol.as_ptr());
    }
    vols.push(std::ptr::null());
    let ret = unsafe { krun_set_mapped_volumes(ctx, vols.as_ptr()) };
    if ret < 0 {
        println!("Error setting VM mapped volumes");
        std::process::exit(-1);
    }
}

pub fn printvm(vm: &VmConfig) {
    println!("{}", vm.name);
    println!(" CPUs: {}", vm.cpus);
    println!(" RAM (MiB): {}", vm.mem);
    println!(" DNS server: {}", vm.dns);
    println!(" Buildah container: {}", vm.container);
    println!(" Workdir: {}", vm.workdir);
    println!(" Mapped volumes: {:?}", vm.mapped_volumes);
    println!(" Mapped ports: {:?}", vm.mapped_ports);
}
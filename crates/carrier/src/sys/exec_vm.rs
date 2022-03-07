use std::ffi::CString;
#[cfg(target_os = "macos")]
use std::path::Path;

use lib::*;
use crate::configs::*;

use crate::utils::utils::*;


pub unsafe fn exec_vm(vmcfg: &VmConfig, rootfs: &str, cmd: &str, args: Vec<CString>) {

    let ctx = carrier_create_ctx() as u32;

    let ret = carrier_set_vm_config(ctx, vmcfg.cpus as u32, vmcfg.mem);
    if ret < 0 {
        println!("Error setting VM config");
        std::process::exit(-1);
    }

    let c_rootfs = CString::new(rootfs).unwrap();
    let ret = carrier_set_root(ctx, c_rootfs.as_ptr() as *const i8);
    if ret < 0 {
        println!("Error setting VM rootfs");
        std::process::exit(-1);
    }

    map_volumes(ctx, &vmcfg, rootfs);

    let mut ports = Vec::new();
    for (host_port, guest_port) in vmcfg.mapped_ports.iter() {
        let map = format!("{}:{}", host_port, guest_port);
        ports.push(CString::new(map).unwrap());
    }
    let mut ps: Vec<*const i8> = Vec::new();
    for port in ports.iter() {
        ps.push(port.as_ptr() as *const i8);
    }
    ps.push(std::ptr::null());
    let ret = carrier_set_port_map(ctx, ps.as_ptr());
    if ret < 0 {
        println!("Error setting VM port map");
        std::process::exit(-1);
    }

    let c_workdir = CString::new(vmcfg.workdir.clone()).unwrap();
    let ret = carrier_set_workdir(ctx, c_workdir.as_ptr() as *const i8);
    if ret < 0 {
        println!("Error setting VM workdir");
        std::process::exit(-1);
    }

    let mut argv: Vec<*const i8> = Vec::new();
    for a in args.iter() {
        argv.push(a.as_ptr() as *const i8);
    }
    argv.push(std::ptr::null());

    let hostname = CString::new(format!("HOSTNAME={}", vmcfg.name)).unwrap();
    let home = CString::new("HOME=/root").unwrap();
    let path = CString::new("PATH=/bin:/sbin:/usr/bin:/usr/sbin:/usr/local/bin").unwrap();
    let env: [*const i8; 4] = [
        hostname.as_ptr() as *const i8,
        home.as_ptr() as *const i8,
        path.as_ptr() as *const i8,
        std::ptr::null(),
    ];

    let c_cmd = CString::new(cmd).unwrap();
    let ret = carrier_set_exec(
        ctx,
        c_cmd.as_ptr() as *const i8,
        argv.as_ptr() as *const *const i8,
        env.as_ptr() as *const *const i8,
    );
    if ret < 0 {
        println!("Error setting VM config");
        std::process::exit(-1);
    }

    let ret = carrier_start_enter(ctx);
    if ret < 0 {
        println!("Error starting VM");
        std::process::exit(-1);
    }
}

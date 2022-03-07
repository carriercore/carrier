use std::ffi::CString;

#[cfg(target_os = "macos")]
use std::path::Path;

use crate::{ArgMatches};
use crate::configs::*;
use crate::container::mount_container::mount_container;
use crate::container::umount_container::umount_container;
use crate::sys::exec_vm::exec_vm;
use crate::sys::set_lock::set_lock;
use crate::sys::set_rlimits::set_rlimits;


pub fn start(cfg: &CarrierConfig, matches: &ArgMatches) {
    let cmd = matches.value_of("COMMAND").unwrap();
    let name = matches.value_of("NAME").unwrap();

    let vmcfg = match cfg.vmconfig_map.get(name) {
        None => {
            println!("No VM found with name {}", name);
            std::process::exit(-1);
        }
        Some(vmcfg) => vmcfg,
    };

    umount_container(&cfg, vmcfg).expect("Error unmounting container");
    let rootfs = mount_container(&cfg, vmcfg).expect("Error mounting container");

    let args: Vec<CString> = match matches.values_of("ARGS") {
        Some(a) => a.map(|val| CString::new(val).unwrap()).collect(),
        None => Vec::new(),
    };

    set_rlimits();

    let _file = set_lock(&rootfs);

    unsafe { exec_vm(vmcfg, &rootfs, cmd, args) };

    umount_container(&cfg, vmcfg).expect("Error unmounting container");
}

use std::process::Command;

use crate::{ArgMatches, APP_NAME};
use crate::configs::*;
use crate::container::mount_container::*;
use crate::container::umount_container::umount_container;
use crate::sys::fix_resolv_conf::fix_resolv_conf;
use crate::utils::utils::*;


pub fn create(cfg: &mut CarrierConfig, matches: &ArgMatches) {
    let cpus = match matches.value_of("cpus") {
        Some(c) => match c.parse::<u32>() {
            Err(_) => {
                println!("Invalid value for \"cpus\"");
                std::process::exit(-1);
            }
            Ok(cpus) => cpus,
        },
        None => cfg.default_cpus,
    };
    let mem = match matches.value_of("mem") {
        Some(m) => match m.parse::<u32>() {
            Err(_) => {
                println!("Invalid value for \"mem\"");
                std::process::exit(-1);
            }
            Ok(mem) => mem,
        },
        None => cfg.default_mem,
    };
    let dns = match matches.value_of("dns") {
        Some(d) => d,
        None => &cfg.default_dns,
    };

    let workdir = matches.value_of("workdir").unwrap();

    let volume_matches = if matches.is_present("volume") {
        matches.values_of("volume").unwrap().collect()
    } else {
        vec![]
    };
    let mapped_volumes = parse_mapped_volumes(volume_matches);

    let port_matches = if matches.is_present("port") {
        matches.values_of("port").unwrap().collect()
    } else {
        vec![]
    };
    let mapped_ports = parse_mapped_ports(port_matches);

    let image = matches.value_of("IMAGE").unwrap();

    let name = matches.value_of("name");
    if let Some(name) = name {
        if cfg.vmconfig_map.contains_key(name) {
            println!("A VM with this name already exists");
            std::process::exit(-1);
        }
    }

    #[cfg(target_os = "linux")]
    let mut args = vec!["from"];
    #[cfg(target_os = "macos")]
    let storage_root = format!("{}/root", cfg.storage_volume);
    #[cfg(target_os = "macos")]
    let storage_runroot = format!("{}/runroot", cfg.storage_volume);
    #[cfg(target_os = "macos")]
    let mut args = vec![
        "--root",
        &storage_root,
        "--runroot",
        &storage_runroot,
        "from",
        "--os",
        "linux",
    ];

    args.push(image);

    let output = match Command::new("buildah")
        .args(&args)
        .stderr(std::process::Stdio::inherit())
        .output()
    {
        Ok(output) => output,
        Err(err) => {
            if err.kind() == std::io::ErrorKind::NotFound {
                println!("{} requires buildah to manage the OCI images, and it wasn't found on this system.", APP_NAME);
            } else {
                println!("Error executing buildah: {}", err.to_string());
            }
            std::process::exit(-1);
        }
    };

    let exit_code = output.status.code().unwrap_or(-1);
    if exit_code != 0 {
        println!(
            "buildah returned an error: {}",
            std::str::from_utf8(&output.stdout).unwrap()
        );
        std::process::exit(-1);
    }

    let container = std::str::from_utf8(&output.stdout).unwrap().trim();
    let name = if let Some(name) = name {
        name.to_string()
    } else {
        container.to_string()
    };
    let vmcfg = VmConfig {
        name: name.clone(),
        cpus,
        mem,
        dns: dns.to_string(),
        container: container.to_string(),
        workdir: workdir.to_string(),
        mapped_volumes,
        mapped_ports,
    };

    let rootfs = mount_container(&cfg, &vmcfg).unwrap();
    fix_resolv_conf(&rootfs, &dns).unwrap();
    umount_container(&cfg, &vmcfg).unwrap();

    cfg.vmconfig_map.insert(name.clone(), vmcfg);
    confy::store(APP_NAME, cfg).unwrap();

    println!("Lightweight VM created with name: {}", name);
}

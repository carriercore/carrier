use std::process::Command;

use crate::{APP_NAME, CarrierConfig};
use crate::configs::VmConfig;

#[allow(unused_variables)]
pub fn umount_container(cfg: &CarrierConfig, vmcfg: &VmConfig) -> Result<(), std::io::Error> {
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
        "umount",
    ];
    #[cfg(target_os = "linux")]
        let mut args = vec!["umount"];

    args.push(&vmcfg.container);

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

    Ok(())
}

use crate::{ArgMatches, APP_NAME};

use crate::configs::*;
use crate::container::remove_container::remove_container;
use crate::container::umount_container::umount_container;

pub fn delete(cfg: &mut CarrierConfig, matches: &ArgMatches) {
    let name = matches.value_of("NAME").unwrap();

    let vmcfg = match cfg.vmconfig_map.remove(name) {
        None => {
            println!("No VM found with that name");
            std::process::exit(-1);
        }
        Some(vmcfg) => vmcfg,
    };

    umount_container(&cfg, &vmcfg).unwrap();
    remove_container(&cfg, &vmcfg).unwrap();

    confy::store(APP_NAME, &cfg).unwrap();
}

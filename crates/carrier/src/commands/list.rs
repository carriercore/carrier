use crate::{ArgMatches};
use crate::configs::*;
use crate::utils::utils::printvm;


pub fn list(cfg: &CarrierConfig, _matches: &ArgMatches) {
    if cfg.vmconfig_map.is_empty() {
        println!("No lightweight VMs found");
    } else {
        for (_name, vm) in cfg.vmconfig_map.iter() {
            println!();
            printvm(vm);
        }
        println!();
    }
}
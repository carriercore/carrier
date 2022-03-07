use std::collections::HashMap;
use serde_derive::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct VmConfig {
    pub(crate) name: String,
    pub(crate) cpus: u32,
    pub(crate) mem: u32,
    pub(crate) container: String,
    pub(crate) workdir: String,
    pub(crate) dns: String,
    pub(crate) mapped_volumes: HashMap<String, String>,
    pub(crate) mapped_ports: HashMap<String, String>,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct CarrierConfig {
    pub(crate) version: u8,
    pub(crate) default_cpus: u32,
    pub(crate) default_mem: u32,
    pub(crate) default_dns: String,
    pub(crate) storage_volume: String,
    pub(crate) vmconfig_map: HashMap<String, VmConfig>,
}

impl Default for CarrierConfig {
    fn default() -> CarrierConfig {
        CarrierConfig {
            version: 1,
            default_cpus: 2,
            default_mem: 1024,
            default_dns: "1.1.1.1".to_string(),
            storage_volume: String::new(),
            vmconfig_map: HashMap::new(),
        }
    }
}
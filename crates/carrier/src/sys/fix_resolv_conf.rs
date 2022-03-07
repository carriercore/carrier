use std::fs;
use std::io::Write;

pub fn fix_resolv_conf(rootfs: &str, dns: &str) -> Result<(), std::io::Error> {
    let resolvconf_dir = format!("{}/etc/", rootfs);
    fs::create_dir_all(resolvconf_dir)?;
    let resolvconf = format!("{}/etc/resolv.conf", rootfs);
    let mut file = fs::File::create(resolvconf)?;
    file.write_all(b"options use-vc\nnameserver ")?;
    file.write_all(dns.as_bytes())?;
    file.write_all(b"\n")?;
    Ok(())
}
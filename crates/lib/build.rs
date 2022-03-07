fn main() {
    #[cfg(target_os = "macos")]
    println!("cargo:rustc-link-lib=framework=Hypervisor");
    #[cfg(target_os = "macos")]
    println!("cargo:rustc-link-search=/opt/homebrew/lib");
    println!("cargo:rustc-link-lib=carriercore");
    #[cfg(target_arch = "aarch64")]
    println!("cargo:rustc-link-lib=fdt");
}

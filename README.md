# Carrier

`Carrier` is a lightweight kernel-based virtual machine container.

## Features

- Minimal attack surface
- No Daemons
- Minimal footprint
- Fast boot time
- Zero disk image maintenance
- Zero network configuration
- Support for mapping host volumes into the guest
- Support for exposing guest ports to the host
- Support running in TEE

## Roadmap

- [x] Linux/KVM on x86_64
- [x] Linux/KVM on AArch64
- [x] macOS/Hypervisor.framework on ARM64
- [ ] Support enclave runtime
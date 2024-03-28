#![no_std]
pub mod perfect_hash {
    pub use phf::Map;
}
pub type PhfMap = perfect_hash::Map<&'static str, &'static str>;
pub mod arch_os;
pub mod debian_arch;
pub mod linux_oci_platform;
pub mod tmm_arch_v0;

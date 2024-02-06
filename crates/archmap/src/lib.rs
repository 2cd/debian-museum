#![no_std]
pub mod perfect_hash {
    pub use phf::Map;
}
pub mod linux_oci_platform;
pub type PhfMap = perfect_hash::Map<&'static str, &'static str>;

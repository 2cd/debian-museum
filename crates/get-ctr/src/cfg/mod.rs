pub(crate) mod debootstrap;
pub(crate) mod digest;
pub(crate) mod disk;
pub(crate) mod mirror;

pub(crate) mod components {
    /// debian 2.1 ~ debian 11
    // pub(crate) const OLD_DEBIAN: [&str; 3] = ["main", "contrib", "non-free"];
    pub(crate) const OLD_DEBIAN: &str = "main contrib non-free";
    pub(crate) const DEBIAN_BOOTSTRAP: &str = "main,contrib,non-free";

    pub(crate) const UBUNTU: &str = "main restricted universe multiverse";
    pub(crate) const UBUNTU_BOOTSTRAP: &str = "main,restricted,universe,multiverse";

    // /// debian 12 +
    // pub(crate) const DEBIAN: &str = "main contrib non-free non-free-firmware";
}

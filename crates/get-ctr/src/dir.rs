use std::{
    env,
    path::{Path, PathBuf},
    sync::OnceLock,
};

static WORKDIR: OnceLock<PathBuf> = OnceLock::new();

pub(crate) fn set_static_workdir() -> &'static Path {
    WORKDIR.get_or_init(|| {
        let dft = |e| {
            log::error!("Error: {e}");
            PathBuf::from(".")
        };
        #[allow(unused_variables)]
        let pwd = || env::current_dir().unwrap_or_else(dft);

        #[cfg(debug_assertions)]
        let tmp = Path::new(env!("CARGO_MANIFEST_DIR")).join("tmp");

        #[cfg(not(debug_assertions))]
        let tmp = pwd().join("tmp");

        std::fs::create_dir_all(&tmp).expect("Failed to create tmp dir");
        env::set_current_dir(&tmp).expect("Failed to set current dir to tmp");
        log::info!("working dir: {tmp:?}");
        tmp
    })
}

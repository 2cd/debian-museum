use log::debug;
use std::{fs, io, path::PathBuf};

use crate::{docker::DOCKER_FILE_CONTENT, task::old_old_debian::TarFile};

pub(crate) fn create_docker_file(tar: &TarFile) -> io::Result<PathBuf> {
    let TarFile {
        tar_fname,
        tar_path,
    } = tar;

    let docker_file = tar_path
        .parent()
        .expect("Invalid Tar Path")
        .join("Dockerfile");
    debug!("docker_file: {:?}", docker_file);

    debug!("creating the Dockerfile");
    fs::write(
        &docker_file,
        DOCKER_FILE_CONTENT.replace("base.tar", tar_fname),
    )?;

    Ok(docker_file)
}

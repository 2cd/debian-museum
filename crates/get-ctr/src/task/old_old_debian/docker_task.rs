use crate::{
    command,
    docker::{
        repo::Repository,
        repo_map::{MainRepo, RepoMap},
        DOCKER_FILE_FOR_NEW_DISTROS, DOCKER_FILE_OLD_CONTENT, DOCKER_IGNORE_CONTENT,
    },
    task::{
        docker::{run_docker_build, run_docker_push},
        old_old_debian::{
            self, deser_ron, digest_cfg::DISTROS_THAT_REQUIRE_XTERM, TarFile,
        },
        pool::wait_process,
    },
};
use ahash::{HashMapExt, HashSetExt};
use anyhow::bail;
use log_l10n::level::color::OwoColorize;
use std::{
    collections::BTreeSet,
    fs, io,
    path::Path,
    process::{Command, Stdio},
};
use tinyvec::TinyVec;

pub(crate) type MainRepoDigests = TinyVec<[String; 4]>;
pub(crate) type MainRepoDigestMap = ahash::HashMap<String, MainRepoDigests>;

pub(crate) fn docker_push<'a, I>(repos: I) -> anyhow::Result<()>
where
    I: IntoIterator<Item = &'a Repository<'a>>,
{
    let (map, ..) = get_repo_map_from_ron(repos.into_iter().next())?;
    let mut repo_set = ahash::HashSet::with_capacity(4);

    let mut arr = [""; 2];
    for k in map.keys() {
        #[cfg(debug_assertions)]
        if k.is_ghcr() {
            continue;
        }

        use MainRepo::*;
        let repo = match k {
            Reg(s) => s,
            Ghcr(s) => s,
        };
        repo_set.insert(rsplit_colon(repo, &mut arr));
    }

    for i in repo_set {
        run_docker_push(i)
    }

    Ok(())
}

fn get_repo_map_from_ron(
    opt_repo: Option<&Repository<'_>>,
) -> Result<(RepoMap, String), anyhow::Error> {
    let docker_ron = match opt_repo
        .map(|r| r.docker_ron_filename())
    {
        Some(v) if Path::new(&v).exists() => v,
        _ => bail!("The docker(tags).ron file does not exist and you may need to rebuild it using `--build`."),
    };
    let map = old_old_debian::deser_ron::<RepoMap, _>(&docker_ron)?;
    Ok((map, docker_ron))
}

/// Splits `xx/yy:latest` and returns xx/yy.
///
/// `url:port/xx:latest` must be split from right to left.
fn rsplit_colon<'a>(s: &'a str, arr: &mut [&'a str; 2]) -> &'a str {
    s.rsplitn(2, ':')
        .enumerate()
        .for_each(|(i, x)| unsafe { *arr.get_unchecked_mut(i) = x });
    arr.reverse();

    if arr[0].is_empty() {
        if arr[1].is_empty() {
            panic!("Invalid repo: {s}")
        }
        return arr[1];
    }

    arr[0]
}

///
/// ```sh
/// docker manifest create --amend reg.tmoe.me:2096/debian/potato:base \
///     reg.tmoe.me:2096/debian/potato:alpha-base \
///     reg.tmoe.me:2096/debian/potato:armv3-base \
///     reg.tmoe.me:2096/debian/potato:ppc-base \
///     reg.tmoe.me:2096/debian/potato:x86-base \
///     reg.tmoe.me:2096/debian/potato:m68k-base \
///     reg.tmoe.me:2096/debian/potato:sparc-base
/// ```
pub(crate) fn create_manifest<'a, I>(repos: I) -> anyhow::Result<()>
where
    I: IntoIterator<Item = &'a Repository<'a>>,
{
    let (map, ron_filename) = get_repo_map_from_ron(repos.into_iter().next())?;

    // let mut repo_digests = MainRepoDigests::new();
    let mut repo_digest_map = ahash::HashMap::with_capacity(4);

    for (k, v) in map.iter() {
        log::info!("creating the docker manifest: {k:?}");
        #[cfg(debug_assertions)]
        if k.is_ghcr() {
            continue;
        }

        let mut args =
            TinyVec::<[&str; 24]>::from_iter(["manifest", "create", "--amend"]);

        let (repo, digest_map_key) = match k {
            MainRepo::Reg(s) => (s, "reg"),
            MainRepo::Ghcr(s) => (s, "ghcr"),
        };

        args.push(repo);

        for i in v {
            args.push(i)
        }

        log::debug!("cmd: {}, args: {:#?}", "docker".green(), args.cyan());
        command::run("docker", &args, true);
        // -----------
        let digest = push_docker_manifest(repo)?;
        update_repo_digest_map(&mut repo_digest_map, digest_map_key, digest)
    }

    fs::write(
        repo_digests_filename(&ron_filename),
        ron::to_string(&repo_digest_map)?,
    )?;

    Ok(())
}

fn update_repo_digest_map<'a>(
    repo_digest_map: &mut ahash::HashMap<&'a str, MainRepoDigests>,
    key: &'a str,
    digest_element: String,
) {
    repo_digest_map
        .entry(key)
        .and_modify(|v| v.push(digest_element.clone()))
        .or_insert_with(|| {
            let mut v = MainRepoDigests::new();
            v.push(digest_element);
            v
        });
}

pub(crate) fn repo_digests_filename(ron_filename: &str) -> String {
    ron_filename.replace(".ron", ".repo-digests")
}

fn push_docker_manifest(org_repo: &str) -> Result<String, anyhow::Error> {
    log::info!(
        "{} {} {} {} {}",
        "docker".green(),
        "manifest".yellow(),
        "push".magenta(),
        "--purge".cyan(),
        org_repo.blue()
    );

    let cmd = Command::new("docker")
        .args(["manifest", "push", "--purge", org_repo])
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .output()?;

    let out = String::from_utf8_lossy(&cmd.stdout);
    let mut arr = [""; 2];
    let repo = rsplit_colon(org_repo, &mut arr);
    let repo_digest = format!("{repo}@{}", out.trim());
    // println!("{repo_digest}");
    Ok(repo_digest)
}

// docker_build
pub(crate) fn docker_build<'a, I>(repos: I) -> anyhow::Result<()>
where
    I: IntoIterator<Item = &'a Repository<'a>>,
{
    let mut children = Vec::with_capacity(32);
    let mut tag_map = RepoMap::default();

    let mut docker_ron_name = String::with_capacity(64);
    let mut treeset = BTreeSet::new();
    let mut init = false;

    for r in repos {
        if !init {
            init = true;
            docker_ron_name = r.docker_ron_filename();
        }

        let TarFile {
            ref tar_fname,
            ref docker_dir,
            ..
        } = r.base_tar_name()?;

        let is_new = !matches!(r.get_series().as_str(), s if DISTROS_THAT_REQUIRE_XTERM.contains(&s));

        create_docker_file(docker_dir, tar_fname, is_new)?;

        run_docker_build(r, &mut children, docker_dir, &mut tag_map)?;
        treeset.insert(r.oci_platform());
    }

    // tag_map => docker-ron
    // tree_set => docker-r
    {
        fs::write(&docker_ron_name, ron::to_string(&tag_map)?)?;
        fs::write(
            platforms_ron_name(&docker_ron_name),
            ron::to_string(&treeset)?,
        )?;
    }

    log::debug!("map: {tag_map:?}");
    wait_process(children);

    Ok(())
}

/// Replaces "base.tar" in the default DOCKER_FILE_CONTENT with tar_fname(e.g., 2.2_potato_x86_base_2001-06-14.tar), and finally write.
fn create_docker_file(
    docker_dir: &Path,
    tar_fname: &str,
    new: bool,
) -> Result<(), io::Error> {
    let docker_file = docker_dir.join("Dockerfile");
    log::debug!("docker_file: {:?}", docker_file);

    log::debug!("creating the Dockerfile");

    let docker_file_content =
        if new { DOCKER_FILE_FOR_NEW_DISTROS } else { DOCKER_FILE_OLD_CONTENT };

    fs::write(
        &docker_file,
        docker_file_content.replace("base.tar", tar_fname),
    )?;

    let docker_ignore = docker_dir.join(".dockerignore");
    log::debug!("creating the .dockerignore");
    fs::write(docker_ignore, DOCKER_IGNORE_CONTENT)?;

    Ok(())
}

pub(crate) fn platforms_ron_name(docker_ron_name: &str) -> &str {
    docker_ron_name.trim_end_matches("on")
}

pub(crate) fn pull_image_and_create_repo_digests<'a, I>(
    repos: I,
) -> anyhow::Result<()>
where
    I: IntoIterator<Item = &'a Repository<'a>>,
{
    for r in repos {
        let TarFile { docker_dir, .. } = r.base_tar_name()?;

        for fname in ["ghcr.ron", "reg.ron"] {
            #[cfg(debug_assertions)]
            if fname == "ghcr.ron" {
                continue;
            }

            let cfg = deser_ron::<MainRepoDigests, _>(docker_dir.join(fname))?;
            let repo = match cfg.first() {
                Some(x) => x,
                _ => continue,
            };

            log::info!("{} {} {}", "docker".green(), "pull".yellow(), repo.blue());
            command::run("docker", &["pull", repo], true);

            let args = ["inspect", "--format", r##"{{json .RepoDigests}}"##, repo];
            log::info!("cmd: {}, args: {:#?}", "docker".green(), args.blue());
            let cmd = Command::new("docker")
                .args(args)
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .output()?;

            let json_arr = String::from_utf8_lossy(&cmd.stdout);
            log::debug!("cmd.output: {json_arr}");
            let new_fname = repo_digests_filename(fname);
            log::info!("writing to: {new_fname}");

            let cfg = serde_yaml::from_str::<MainRepoDigests>(json_arr.trim())?;
            fs::write(docker_dir.join(new_fname), ron::to_string(&cfg)?)?
        }
    }
    Ok(())
}

use crate::command::spawn_cmd;
use log::{debug, info, trace};
use std::{borrow::Cow, path::Path, process::Child};
use tinyvec::TinyVec;
use typed_builder::TypedBuilder;

pub(crate) const DOCKER_FILE: &str = "assets/ci/base/Dockerfile";

pub(crate) fn get_oci_platform(arch: &str) -> &str {
    archmap::linux_oci_platform::map()
        .get(arch)
        // .copied()
        .expect("linux/amd64")
}
/// `docker build --tag $tag0 --tag $tag1 ...`
pub(crate) fn spawn_docker_build<'a, T: IntoIterator<Item = &'a str>>(
    tags: T,
    platform: &str,
    context: &Path,
) -> Child {
    debug!("building the docker container ...");
    let mut args = Vec::with_capacity(16);
    args.push("build");

    for tag in tags {
        info!("tag:\t {}", tag);
        args.extend(["--tag", tag])
    }

    let context_path_str = context.to_string_lossy();
    args.extend(["--platform", platform, "--pull", &context_path_str]);

    spawn_cmd("docker", &args)
}

#[derive(Debug, TypedBuilder)]
pub(crate) struct Repository<'r> {
    #[builder(default = "2cd")]
    pub(crate) owner: &'r str,
    #[builder(default = "debian")]
    pub(crate) project: &'r str,
    pub(crate) codename: &'r str,
    pub(crate) version: &'r str,
    pub(crate) arch: &'r str,

    #[builder(default = Some("latest"))]
    pub(crate) tag: Option<&'r str>,
}

impl<'r> Default for Repository<'r> {
    fn default() -> Self {
        let dft = Default::default();

        Self::builder()
            .codename(dft)
            .version(dft)
            .arch(dft)
            .build()
    }
}

type NormalRepos = TinyVec<[String; 2]>;
type MainRepos = TinyVec<[MainRepo; 2]>;

impl<'r> Repository<'r> {
    pub(crate) const REG_URI: &'static str = "reg.tmoe.me:2096";
    pub(crate) const GHCR_URI: &'static str = "ghcr.io";

    pub(crate) fn opt_tag_suffix(&self) -> Cow<str> {
        match self.tag {
            Some(t) => Cow::from(format!("-{t}")),
            _ => Cow::from(""),
        }
    }

    /// -> `[ghcr.io/xx/yy, ghcr.io/xx/zz]`
    /// > xx/yy/zz from Self.
    pub(crate) fn ghcr_repos(&self) -> NormalRepos {
        let suffix = self.opt_tag_suffix();
        let uri = Self::GHCR_URI;
        let Self {
            owner,
            project,
            codename,
            arch,
            version,
            ..
        } = self;

        // let mut v =
        [
            format!(
                // "{}-{}{}",
                // ghcr.io/2cd/debian:potato-x86-base OR 2cd/debain:bo-x86
                "{}/{}/{}:{}-{}{}",
                uri, owner, project, codename, arch, suffix
            ),
            format!(
                // ghcr: "{}-{}{}",
                // ghcr.io/2cd/debian:potato-x86-base OR ghcr.io/2cd/debain:bo-x86
                "{}/{}/{}:{}-{}{}",
                uri, owner, project, version, arch, suffix
            ),
        ]
        .into()
    }

    /// -> `[reg.tmoe.me/xx/yy, reg.tmoe.me/xx/zz]`
    /// > xx/yy/zz from Self.
    pub(crate) fn reg_repos(&self) -> NormalRepos {
        let suffix = self.opt_tag_suffix();
        let uri = Self::REG_URI;
        let Self {
            project,
            codename,
            version,
            arch,
            ..
        } = self;

        [
            format!(
                // REG_URI/debian/potato:x86-base OR REG_URI/debain/bo-x86
                "{}/{}/{}:{}{}",
                uri, project, codename, arch, suffix
            ),
            format!(
                // REG_URI/debian/2.2:x86-base OR REG_URI/debain/1.3:x86
                "{}/{}/{}:{}{}",
                uri, project, version, arch, suffix
            ),
        ]
        .into()
    }

    /// e.g. -> `[MainRepo::Reg(REG_URI/debain/bo:latest), MainRepo::Reg(REG_URI/debain/1.3:latest)]`
    pub(crate) fn reg_main_repos(&self) -> MainRepos {
        let uri = Self::REG_URI;
        let tag = self.tag.unwrap_or("latest");
        let Self {
            project,
            codename,
            version,
            ..
        } = self;

        [
            // REG_URI/debian/potato:base OR REG_URI/debain/bo:latest
            format!("{}/{}/{}:{}", uri, project, codename, tag),
            // REG_URI/debian/2.2:base OR REG_URI/debain/1.3:latest
            format!("{}/{}/{}:{}", uri, project, version, tag),
        ]
        .map(MainRepo::Reg)
        .into()
    }

    pub(crate) fn ghcr_main_repos(&self) -> MainRepos {
        let suffix = self.opt_tag_suffix();
        let uri = Self::GHCR_URI;
        let Self {
            owner,
            project,
            codename,
            version,
            ..
        } = self;

        [
            // ghcr.io/2cd/debian:potato-base OR ghcr.io/2cd/debian:bo
            format!("{}/{}/{}:{}{}", uri, owner, project, codename, suffix),
            // ghcr.io/2cd/debian:2.2-base OR ghcr.io/2cd/debian:1.3
            format!("{}/{}/{}:{}{}", uri, owner, project, version, suffix),
        ]
        .map(MainRepo::Ghcr)
        .into()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum MainRepo {
    Reg(String),
    Ghcr(String),
}

impl Default for MainRepo {
    fn default() -> Self {
        Self::Ghcr(Default::default())
    }
}

pub(crate) type Repos = TinyVec<[String; 16]>;

#[derive(Debug, Default, derive_more::Deref)]
pub(crate) struct RepoMap(ahash::HashMap<MainRepo, Repos>);

impl RepoMap {
    /// Instead of resetting to a new value, this function pushes a new element to the value(&mut TinyVec) corresponding to the key.
    ///
    /// Note: If the corresponding key does not exist in the map, the Key and Value are created.
    ///
    /// # Example
    ///
    /// ```no_run
    /// let mut map = RepoMap::default();
    ///
    /// let key = MainRepo::Ghcr("ghcr.io/xx/yy:latest".into());
    ///
    /// map.push_to_value(key.to_owned(), "ghcr.io/xx/yy:x64".into());
    /// map.push_to_value(key.to_owned(), "ghcr.io/xx/yy:rv64gc".into());
    ///
    /// let value = map
    ///     .get(&key)
    ///     .expect("Failed to unwrap map");
    ///
    /// assert_eq!(value[0], "ghcr.io/xx/yy:x64");
    /// assert_eq!(value[1], "ghcr.io/xx/yy:rv64gc");
    /// ```
    pub(crate) fn push_to_value(&mut self, key: MainRepo, element: String) {
        self.0
            .entry(key)
            .and_modify(|v| {
                debug!("RepoMap.value\t is_heap: {}", v.is_heap());
                trace!("capacity: {}, len: {}", v.capacity(), v.len(),);
                v.push(element.clone())
            })
            .or_insert_with(|| {
                trace!("init RepoMap.value");
                let mut v = Repos::new();
                v.push(element);
                v
            });
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn tag_map() {
        let mut map = docker::RepoMap::default();

        let key = docker::MainRepo::Ghcr("ghcr.io/xx/yy:latest".into());

        map.push_to_value(key.clone(), "ghcr.io/xx/yy:x64".into());
        map.push_to_value(key.clone(), "ghcr.io/xx/yy:rv64gc".into());

        let value = map
            .get(&key)
            .expect("Failed to unwrap map");

        assert_eq!(value[0], "ghcr.io/xx/yy:x64");
        assert_eq!(value[1], "ghcr.io/xx/yy:rv64gc");
    }
}

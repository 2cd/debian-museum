use derive_more::{Deref, From};
use log::trace;
use log_l10n::{
    get_pkg_name,
    level::color::OwoColorize,
    logger::{before_init, env_logger},
};
use std::{borrow::Cow, env, sync::OnceLock};

pub(crate) fn today() -> &'static str {
    static D: OnceLock<String> = OnceLock::new();
    D.get_or_init(|| {
        time::OffsetDateTime::now_utc()
            .date()
            .to_string()
    })
}

pub(crate) fn init() {
    let pkg = get_pkg_name!();
    let env_name = EnvName::new(pkg);

    before_init(pkg, &env_name);
    env_logger::init(&env_name, Some("info"));
    trace!("LOG_EnvName: {}", (*env_name).yellow().bold());
}

#[derive(Deref, From, Debug, Default)]
#[from(forward)]
struct EnvName<'n>(Cow<'n, str>);

impl<'n> EnvName<'n> {
    /// `xyz` => `XYZ_LOG`
    ///
    /// `a-b` => `A_B_LOG`
    fn new(s: &str) -> Self {
        let s = s.trim();
        if s.is_empty() {
            return "RUST_LOG".into();
        }

        let mut env_name = String::with_capacity(s.len() + 4);
        env_name.push_str(s);
        env_name.make_ascii_uppercase();

        // Warning: The unsafe function is used here!
        for i in unsafe { env_name.as_bytes_mut() } {
            // Replace all '-' with '_'
            if *i == b'-' {
                *i = b'_';
            }
        }
        env_name.push_str("_LOG");

        env_name.into()
    }
}

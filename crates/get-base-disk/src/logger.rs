use log_l10n::{
    get_pkg_name,
    level::color::OwoColorize,
    logger::{before_init, env_logger},
};
use std::env;

pub(crate) fn init() {
    let pkg = get_pkg_name!();
    let env_name = to_env_log_name(pkg);

    before_init(pkg, &env_name);
    env_logger::init(&env_name, Some("info"));
    log::trace!("LOG_ENV_NAME: {}", env_name.yellow().bold());
}

fn to_env_log_name(s: &str) -> String {
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
    env_name
}

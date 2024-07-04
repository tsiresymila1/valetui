use std::env;
use std::path::Path;
use std::process::Command;

use dirs;

pub const VALET_SERVER_PATH: &str = "/path/to/valet/server";
pub const ISOLATED_PHP_VERSION: &str = "7.4"; // Example PHP version
pub const VALET_STATIC_PREFIX: &str = "/static";

pub const SUPPORTED_PHP_VERSIONS: [&str; 2] = ["8.2", "8.3"];
pub const ISOLATION_SUPPORTED_PHP_VERSIONS: [&str; 9] = [
    "7.0", "7.1", "7.2", "7.3", "7.4", "8.0", "8.1", "8.2", "8.3"
];
pub const COMMON_EXTENSIONS: [&str; 10] = [
    "cli", "mysql", "gd", "zip", "xml", "curl", "mbstring", "pgsql", "intl", "posix"
];
pub const FPM_CONFIG_FILE_NAME: &str = "valet.conf";

pub const NGINX_CONF: &str = "/etc/nginx/nginx.conf";
pub const SITES_AVAILABLE_CONF: &str = "/etc/nginx/sites-available/valet.conf";
pub const SITES_ENABLED_CONF: &str = "/etc/nginx/sites-enabled/valet.conf";

pub fn user() -> String {
    env::var("SUDO_USER").unwrap_or_else(|_| {
        env::var("USER").unwrap_or_else(|_| "".to_string())
    })
}
pub fn group() -> Option<String> {
    let sudo_user = env::var("SUDO_USER").ok();
    let user = env::var("USER").ok();

    let username = sudo_user.or(user)?;

    let output = Command::new("id")
        .arg("-gn")
        .arg(&username)
        .output()
        .expect("Failed to execute command");

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

pub struct Valet;

impl Valet {
    pub fn root_path() -> String {
        let out_dir = env!("OUT_DIR");
        Path::new(out_dir).to_string_lossy().to_string()
    }
    pub fn home_path() -> String {
        let dir = dirs::home_dir().unwrap();
        let config_path = dir.join(".config/valetui");
        config_path.to_string_lossy().into_owned()
    }
}
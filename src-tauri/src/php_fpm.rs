use regex::Regex;
use serde_json::json;

use crate::configuration::Configuration;
use crate::constants::{COMMON_EXTENSIONS, FPM_CONFIG_FILE_NAME, group, ISOLATION_SUPPORTED_PHP_VERSIONS, SUPPORTED_PHP_VERSIONS, user, Valet};
use crate::devtools::DevTools;
use crate::manager::apt::Apt;
use crate::manager::command::ValetCommandLine;
use crate::manager::file_system::ValetFilesystem;
use crate::manager::interface::{CommandLine, Filesystem, PackageManager, ServiceManager};
use crate::manager::service_manager::ValetServiceManager;
use crate::nginx::Nginx;

pub struct PhpFpm {
    config: Configuration,
    pm: Apt,
    sm: ValetServiceManager,
    cli: ValetCommandLine,
    files: ValetFilesystem,
    nginx: Nginx,
}

impl PhpFpm {
    pub fn new(
        config: Configuration,
        pm: Apt,
        sm: ValetServiceManager,
        cli: ValetCommandLine,
        files: ValetFilesystem,
        nginx: Nginx,
    ) -> Self {
        Self {
            config,
            pm,
            sm,
            cli,
            files,
            nginx,
        }
    }


    pub fn restart(&self, version: Option<&str>) {
        self.sm.restart(vec![self.service_name(version).as_str()]);
    }

    pub fn stop(&self, version: Option<&str>) {
        self.sm.stop(vec![self.service_name(version).as_str()]);
    }

    pub fn status(&self, version: Option<&str>) {
        self.sm.print_status(&self.service_name(version));
    }

    pub fn socket_file_name(&self, v: Option<&str>) -> String {
        let current_version = self.get_current_version();
        let version = current_version.as_str();
        let version = v.unwrap_or(version);
        let version = version.replace(|c: char| !c.is_digit(10), "");
        format!("valet{}.sock", version)
    }
    pub fn normalize_php_version(&self, version: &str) -> String {
        let re = Regex::new(r"^(?:php[@-]?)?(?P<MAJOR_VERSION>\d{1}).?(?P<MINOR_VERSION>\d{1})$").unwrap();
        if let Some(caps) = re.captures(version) {
            return format!("{}.{}", &caps["MAJOR_VERSION"], &caps["MINOR_VERSION"]);
        }
        String::new()
    }
    pub fn get_current_version(&self) -> String {
        let default = json!(self.get_default_version());
        match self.config.get("php_version") {
            Some(value) => {
                value.as_str().unwrap_or(default.as_str().unwrap()).to_string()
            }
            None => {
                default.to_string()
            }
        }
    }
    pub fn get_php_executable_path(&self, version: Option<&str>) {
        let bin = match version {
            Some(v) => format!("php{}", self.normalize_php_version(v)),
            None => "php".to_string(),
        };
        DevTools::get_bin(&bin, &["/usr/local/bin"])
    }

    pub fn fpm_socket_file(&self, version: &str) -> String {
        format!("{}/{}", Valet::home_path(), self.socket_file_name(Some(version)))
    }
    fn service_name(&self, version: Option<&str>) -> String {
        let current_version = self.get_current_version();
        let v = current_version.as_str();
        let version = version.unwrap_or(v);
        self.pm.get_php_fpm_name(version)
    }
    pub fn validate_version(&self, version: &str) -> bool {
        SUPPORTED_PHP_VERSIONS.contains(&version)
    }
    pub fn stop_if_unused(&self, version: &str) {
        let version = self.normalize_php_version(version);
        if !self.utilized_php_versions().contains(&version) {
            self.stop(Some(&version));
        }
    }

    pub fn install(&mut self, version: Option<&str>, install_ext: bool) {
        let current_version = self.get_current_version();
        let v = current_version.as_str();
        let version = version.unwrap_or(v);
        let version = self.normalize_php_version(version);
        if version.clone().is_empty() {
            return;
        }
        let package_name = self.pm.get_php_fpm_name(&version.clone());
        if !self.pm.installed(&package_name) {
            self.pm.ensure_installed(&package_name);
            if install_ext {
                self.install_extensions(&version.clone());
            }
            self.sm.enable(&self.service_name(Some(&version.clone())));
        }
        self.files.ensure_dir_exists("/var/log", &user(), 0o775).unwrap();
        self.install_configuration(&version.clone());
        self.restart(Some(&version.clone()));
    }
    pub fn uninstall(&mut self, version: Option<&str>) {
        let current_version = self.get_current_version();
        let v = current_version.as_str();
        let version = version.unwrap_or(v);
        let version = self.normalize_php_version(version);
        if version.is_empty() {
            return;
        }
        let fpm_conf_path = format!("{}/{}", self.fpm_config_path(Some(&version)), FPM_CONFIG_FILE_NAME);
        if self.files.exists(&fpm_conf_path) {
            self.files.unlink(&fpm_conf_path).unwrap();
            self.stop(Some(&version));
        }
    }
    pub fn switch_version(&mut self, version: &str, update_cli: bool, ignore_ext: bool) {
        let current_version = self.get_current_version();
        let version = self.normalize_php_version(version);
        println!("Changing PHP version...");
        self.install(Some(&version.clone()), !ignore_ext);

        if self.sm.disabled(&self.service_name(Some(&version.clone()))) {
            self.sm.enable(&self.service_name(Some(&version.clone())));
        }
        self.config.set("php_version", json!(version));

        self.stop_if_unused(&current_version);
        self.update_nginx_config_files(&version);
        self.nginx.restart();
        self.status(Some(&version));
        if update_cli {
            self.cli.run(&format!("update-alternatives --set php /usr/bin/php{}", version)).unwrap();
        }
    }

    pub fn update_home_path(&self, old_home_path: &str, new_home_path: &str) {
        for version in ISOLATION_SUPPORTED_PHP_VERSIONS.iter() {
            let conf_path = format!("{}/{}", self.fpm_config_path(Some(version)), FPM_CONFIG_FILE_NAME);
            if self.files.exists(&conf_path) {
                let valet_conf = self.files.get(&conf_path).unwrap();
                let updated_conf = valet_conf.replace(old_home_path, new_home_path);
                self.files.put(&conf_path, &updated_conf).unwrap();
            }
        }
    }

    fn update_nginx_config_files(&self, version: &str) {
        // Action 1: Update all separate secured versions
        for file in self.nginx.configured_sites() {
            let path = format!("{}/Nginx/{}", Valet::home_path(), file);
            let mut content = self.files.get(&path).unwrap();
            if content.contains(&format!("# {}", "ISOLATED_PHP_VERSION")) {
                continue;
            }
            if let Some(_) = Regex::new(r"unix:(.*?.sock)").unwrap().captures(&content) {
                content = content.replace(
                    "unix:(.*?.sock)",
                    &format!("unix:{}", self.socket_file_name(Some(version))),
                );
                self.files.put(&path, &content).unwrap();
            }
        }
        // Action 2: Update NGINX valet.conf for php socket version
        let s_file_name = self.socket_file_name(Some(version));
        self.nginx.install_server(Some(s_file_name.as_str()));
    }

    fn install_extensions(&self, version: &str) {
        let extension_prefix = self.pm.get_php_extension_prefix(version);
        let extensions: Vec<String> = COMMON_EXTENSIONS.iter()
            .map(|&ext| format!("{}{}", extension_prefix, ext))
            .collect();
        self.pm.ensure_installed(&extensions.join(" "));
    }

    fn install_configuration(&self, version: &str) {
        let contents = self.files.get(&format!("{}/stubs/fpm.conf", Valet::root_path())).unwrap();
        let contents = contents.replace("VALET_USER", &user())
            .replace("VALET_GROUP", group().unwrap().as_str())
            .replace("VALET_FPM_SOCKET_FILE", &self.fpm_socket_file(version));

        self.files.put(&format!("{}/{}", self.fpm_config_path(Some(version)), FPM_CONFIG_FILE_NAME), &contents).unwrap();
    }

    fn utilized_php_versions(&self) -> Vec<String> {
        let fpm_sock_files: Vec<String> = ISOLATION_SUPPORTED_PHP_VERSIONS.iter()
            .map(|&version| self.socket_file_name(Some(&self.normalize_php_version(version))))
            .collect();

        let mut versions: Vec<String> = self.nginx.configured_sites().iter()
            .filter_map(|file| {
                let path = format!("{}/Nginx/{}", Valet::home_path(), file);
                let content = self.files.get(&path).unwrap();
                fpm_sock_files.iter()
                    .find(|&sock| content.contains(sock))
                    .map(|sock| self.normalize_php_version(&sock.replace("valet", "").replace(".sock", "")))
            })
            .collect();
        let current_version = self.get_current_version();
        if !versions.contains(&current_version) {
            versions.push(current_version);
        }
        versions
    }

    fn fpm_config_path(&self, version: Option<&str>) -> String {
        let current_version = self.get_current_version();
        let v = current_version.as_str();
        let version = version.unwrap_or(v);
        let version_without_dot = version.replace(".", "");
        let conf_dirs = vec![
            format!("/etc/php/{}/fpm/pool.d", version), // Ubuntu
            format!("/etc/php{}/fpm/pool.d", version), // Ubuntu
            format!("/etc/php{}/php-fpm.d", version), // Manjaro
            format!("/etc/php{}/php-fpm.d", version_without_dot), // ArchLinux
            "/etc/php7/fpm/php-fpm.d".to_string(), // openSUSE PHP7
            "/etc/php8/fpm/php-fpm.d".to_string(), // openSUSE PHP8
            "/etc/php-fpm.d".to_string(), // Fedora
            "/etc/php/php-fpm.d".to_string(), // Arch
        ];
        for path in conf_dirs {
            if self.files.is_dir(&path) {
                return path;
            }
        }
        panic!("Unable to determine PHP-FPM configuration folder.");
    }

    fn validate_isolation_version(&self, version: &str) {
        if !ISOLATION_SUPPORTED_PHP_VERSIONS.contains(&version) {
            panic!(
                "Invalid version [{}] used. Supported versions are: {}",
                version,
                ISOLATION_SUPPORTED_PHP_VERSIONS.join(", ")
            );
        }
    }
    fn get_default_version(&self) -> String {
        self.normalize_php_version(&format!("{}.{}", std::env::var("PHP_MAJOR_VERSION").unwrap(), std::env::var("PHP_MINOR_VERSION").unwrap()))
    }
}

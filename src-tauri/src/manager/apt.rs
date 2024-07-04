use std::collections::HashMap;
use std::str;

use crate::manager::interface::{CommandLine, PackageManager, ServiceManager};

pub struct Apt {
    cli: Box<dyn CommandLine>,
    service_manager: Box<dyn ServiceManager>,
    php_fpm_pattern_by_version: HashMap<String, String>,
}

impl Apt {
    const PACKAGES: &'static [(&'static str, &'static str)] = &[
        ("redis", "redis-server"),
        ("mysql", "mysql-server"),
        ("mariadb", "mariadb-server"),
    ];
    pub fn new(cli: Box<dyn CommandLine>, service_manager: Box<dyn ServiceManager>) -> Self {
        Self { cli, service_manager, php_fpm_pattern_by_version: HashMap::new() }
    }

    fn packages(&self, package: &str) -> Vec<String> {
        let query = format!("dpkg -l {} | grep '^ii' | sed 's/\\s\\+/ /g' | cut -d' ' -f2", package);
        match self.cli.run(&query) {
            Ok(output) => output.lines().map(|s| s.to_string()).collect(),
            Err(_) => vec![],
        }
    }
}

impl PackageManager for Apt {
    fn installed(&self, package: &str) -> bool {
        self.packages(package).contains(&package.to_string())
    }

    fn ensure_installed(&self, package: &str) {
        if !self.installed(package) {
            self.install_or_fail(package);
        }
    }

    fn install_or_fail(&self, package: &str) {
        println!("{}: Installing", package);
        match self.cli.run(&format!("apt-get install -y {}", package)) {
            Ok(_) => (),
            Err(error) => {
                eprintln!("{}", error);
                panic!("Apt was unable to install [{}].", package);
            }
        }
    }

    fn setup(&self) {
        // Nothing to do
    }

    fn is_available(&self) -> bool {
        match self.cli.run("which apt-get") {
            Ok(output) => !output.is_empty(),
            Err(_) => false,
        }
    }

    fn get_php_fpm_name(&self, version: &str) -> String {
        if let Some(pattern) = self.php_fpm_pattern_by_version.get(version) {
            pattern.replace("{VERSION}", version)
        } else {
            format!("php{}-fpm", version)
        }
    }

    fn get_php_extension_prefix(&self, version: &str) -> String {
        format!("php{}-", version)
    }

    fn restart_network_manager(&self) {
        self.service_manager.restart(vec!["NetworkManager"]);
    }

    fn package_name(&self, name: &str) -> String {
        for &(key, value) in Self::PACKAGES {
            if key == name {
                return value.to_string();
            }
        }
        panic!("Package not found by {}", name);
    }
}
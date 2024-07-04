use std::collections::HashMap;
use std::env;

use crate::configuration::Configuration;
use crate::constants::user;
use crate::manager::command::ValetCommandLine;
use crate::manager::file_system::ValetFilesystem;
use crate::manager::interface::Filesystem;
use crate::paths::{Paths, PathTrait};
use crate::php_fpm::PhpFpm;

pub struct Site {
    config: Configuration,
    cli: ValetCommandLine,
    files: ValetFilesystem,
    fpm: PhpFpm,
}

impl Site {
    pub fn new(config: Configuration, cli: ValetCommandLine, files: ValetFilesystem, fpm: PhpFpm) -> Self {
        Site { config, cli, files, fpm }
    }
    pub fn prune_links(&self) {
        self.files.ensure_dir_exists(Paths::sites_path(None).as_str(), &user(), 0o775).unwrap();
        self.files.remove_broken_links_at(Paths::sites_path(None).as_str()).unwrap();
    }
    pub fn get_site_url(&self, directory: &str) -> Result<String, String> {
        let tld = self.config.get("domain").ok_or("Domain not found")?;
        let directory = if directory == "." || directory == "./" {
            env::current_dir().unwrap().file_name().unwrap().to_str().unwrap().to_string()
        } else {
            directory.replace(&format!(".{}", tld), "")
        };
        let served_sites = self.served_sites();
        if !served_sites.contains_key(&directory) {
            return Err(format!("The [{}] site could not be found in Valet's site list.", directory));
        }

        Ok(format!("{}.{}", directory, tld))
    }
    pub fn php_rc_version(&self, site: &str) -> Option<String> {
        let served_sites = self.served_sites();
        if let Some(site_path) = served_sites.get(site) {
            let path = format!("{}/.valetphprc", site_path);
            if self.files.exists(&path) {
                return Some(self.fpm.normalize_php_version(&self.files.get(&path).unwrap().trim().to_string()));
            }
        }
        None
    }

    fn served_sites(&self) -> HashMap<String, String> {
        let mut parked_sites = HashMap::new();
        let values = self.config.get("paths").unwrap();
        let parked_paths: Vec<String> = values.to_string().split(",").map(String::from).collect();
        for path in parked_paths {
            if path == Paths::sites_path(None) {
                continue;
            }
            for site in self.files.scandir(&path).unwrap() {
                if self.files.is_dir(&format!("{}/{}", path, site)) {
                    parked_sites.insert(site.clone(), format!("{}/{}", path, site));
                }
            }
        }
        for linked_site in self.files.scandir(&Paths::sites_path(None)).unwrap() {
            parked_sites.insert(linked_site.clone(), self.files.realpath(&Paths::sites_path(Some(linked_site.as_str()))));
        }
        parked_sites
    }
}

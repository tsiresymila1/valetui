use std::error::Error as SdtError;
use std::{format, fs};
use std::io::Error;
use crate::manager::command::ValetCommandLine;

pub trait PackageManager {
    fn installed(&self, package: &str) -> bool;
    fn ensure_installed(&self, package: &str);
    fn install_or_fail(&self, package: &str);
    fn setup(&self);
    fn is_available(&self) -> bool;
    fn get_php_fpm_name(&self, version: &str) -> String;
    fn get_php_extension_prefix(&self, version: &str) -> String;
    fn restart_network_manager(&self);
    fn package_name(&self, name: &str) -> String;
}

pub trait ServiceManager {
    fn start(&self, services: Vec<&str>);
    fn stop(&self, services: Vec<&str>);
    fn restart(&self, services: Vec<&str>);
    fn print_status(&self, service: &str);
    fn disabled(&self, service: &str) -> bool;
    fn disable(&self, service: &str);
    fn enable(&self, service: &str);
    fn is_available(&self) -> bool;
    fn is_systemd(&self) -> bool;
    fn remove_valet_dns(&self);
}

pub trait CommandLine {
    fn quietly(&self, command: &str) -> Result<(), Box<dyn SdtError>>;
    fn quietly_as_user(&self, command: &str) -> Result<(), Box<dyn SdtError>>;
    fn passthru(&self, command: &str) -> Result<(), Box<dyn SdtError>>;
    fn run(&self, command: &str) -> Result<String, Box<dyn SdtError>>;
    fn run_as_user(&self, command: &str) -> Result<String, Box<dyn SdtError>>;
}

pub trait Filesystem {
    fn remove(&self, files: &[&str]) -> Result<(), Box<dyn std::error::Error>>;
    fn is_dir(&self, path: &str) -> bool;
    fn mkdir(&self, path: &str, mode: u32) -> Result<(), Error>;
    fn ensure_dir_exists(&self, path: &str, owner: &str, mode: u32) -> Result<(), Error>;
    fn touch(&self, path: &str) -> Result<(), Error>;
    fn exists(&self, path: &str) -> bool;
    fn get(&self, path: &str) -> Result<String, Error>;
    fn put(&self, path: &str, contents: &str) -> Result<(), Error>;
    fn append(&self, path: &str, contents: &str) -> Result<(), Error>;
    fn copy_directory(&self, from: &str, to: &str) -> Result<(), Error>;
    fn copy(&self, from: &str, to: &str) -> Result<(), Error>;
    fn backup(&self, file: &str) -> Result<(), Error>;
    fn restore(&self, file: &str) -> Result<(), Error>;
    fn symlink(&self, target: &str, link: &str) -> Result<(), Error>;
    fn unlink(&self, path: &str) -> Result<(), Error>;
    fn chmod(&self, path: &str, mode: u32) -> Result<(), Error>;
    fn is_link(&self, path: &str) -> bool;
    fn read_link(&self, path: &str) -> Result<String, Error>;
    fn remove_broken_links_at(&self, path: &str) -> Result<(), Error>;
    fn is_broken_link(&self, path: &str) -> bool;
    fn scandir(&self, path: &str) -> Result<Vec<String>, Error>;
    fn uncomment_line(&self, line: &str, path: &str);
    fn comment_line(&self, line: &str, file: &str);
    fn realpath(&self,path: &str) -> String;
}
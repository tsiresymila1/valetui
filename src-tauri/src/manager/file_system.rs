use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Error;
use std::io::prelude::*;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use crate::manager::command::ValetCommandLine;
use crate::manager::interface::{CommandLine, Filesystem};

#[derive(Clone, Copy)]
pub struct ValetFilesystem;

impl Filesystem for ValetFilesystem {
    fn remove(&self, files: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
        let mut files: Vec<&str> = files.iter().map(|f| *f).collect();
        files.reverse();

        for file in files {
            if !Path::new(file).exists() {
                continue;
            }

            if Path::new(file).is_dir() && !fs::symlink_metadata(file)?.file_type().is_symlink() {
                self.remove(&[file])?;
            } else {
                fs::remove_file(file)?;
            }
        }

        Ok(())
    }

    fn is_dir(&self, path: &str) -> bool {
        Path::new(path).is_dir()
    }

    fn mkdir(&self, path: &str, mode: u32) -> Result<(), Error> {
        fs::create_dir_all(path)?;
        self.chmod(path, mode)?;
        Ok(())
    }

    fn ensure_dir_exists(&self, path: &str, _owner: &str, mode: u32) -> Result<(), Error> {
        if !self.is_dir(path) {
            self.mkdir(path, mode)?;
        }

        Ok(())
    }

    fn touch(&self, path: &str) -> Result<(), Error> {
        OpenOptions::new()
            .create(true)
            .write(true)
            .open(path)?;

        Ok(())
    }

    fn exists(&self, path: &str) -> bool {
        Path::new(path).exists()
    }

    fn get(&self, path: &str) -> Result<String, Error> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(contents)
    }

    fn put(&self, path: &str, contents: &str) -> Result<(), Error> {
        let mut file = File::create(path)?;
        file.write_all(contents.as_bytes())?;

        Ok(())
    }

    fn append(&self, path: &str, contents: &str) -> Result<(), Error> {
        let mut file = OpenOptions::new().append(true).open(path)?;
        file.write_all(contents.as_bytes())?;

        Ok(())
    }

    fn copy_directory(&self, from: &str, to: &str) -> Result<(), Error> {
        fs::create_dir_all(to)?;

        let entries = fs::read_dir(from)?;

        for entry in entries {
            let entry = entry?;
            let from_path = entry.path();
            let to_path = Path::new(to).join(from_path.file_name().unwrap());

            if entry.file_type()?.is_dir() {
                self.copy_directory(from_path.to_str().unwrap(), to_path.to_str().unwrap())?;
            } else {
                fs::copy(&from_path, &to_path)?;
            }
        }

        Ok(())
    }

    fn copy(&self, from: &str, to: &str) -> Result<(), Error> {
        fs::copy(from, to)?;
        self.chmod(to, 0o755)?;

        Ok(())
    }

    fn backup(&self, file: &str) -> Result<(), Error> {
        let backup_file = format!("{}.bak", file);

        if self.exists(file) && !self.exists(&backup_file) {
            fs::copy(file, &backup_file)?;
        }

        Ok(())
    }

    fn restore(&self, file: &str) -> Result<(), Error> {
        let backup_file = format!("{}.bak", file);

        if self.exists(&backup_file) {
            fs::rename(&backup_file, file)?;
        }

        Ok(())
    }

    fn symlink(&self, target: &str, link: &str) -> Result<(), Error> {
        if self.exists(link) {
            self.unlink(link)?;
        }

        #[cfg(not(windows))]
        {
            std::os::unix::fs::symlink(target, link)?;
        }

        Ok(())
    }

    fn unlink(&self, path: &str) -> Result<(), Error> {
        if self.exists(path) {
            fs::remove_file(path)?;
        }

        Ok(())
    }

    fn chmod(&self, path: &str, mode: u32) -> Result<(), Error> {
        let mut perms = fs::metadata(path)?.permissions();
        perms.set_mode(mode);
        fs::set_permissions(path, perms)?;

        Ok(())
    }

    fn is_link(&self, path: &str) -> bool {
        fs::symlink_metadata(path)
            .map(|meta| meta.file_type().is_symlink())
            .unwrap_or(false)
    }

    fn read_link(&self, path: &str) -> Result<String, Error> {
        let link_path = fs::read_link(path)?;
        Ok(link_path.to_string_lossy().to_string())
    }

    fn remove_broken_links_at(&self, path: &str) -> Result<(), Error> {
        let entries = fs::read_dir(path)?;
        for entry in entries {
            let entry = entry?;
            let entry_path = entry.path();
            if self.is_broken_link(&entry_path.to_string_lossy()) {
                self.unlink(&entry_path.to_string_lossy())?;
            }
        }
        Ok(())
    }

    fn is_broken_link(&self, path: &str) -> bool {
        self.is_link(path) && !self.exists(path)
    }

    fn scandir(&self, path: &str) -> Result<Vec<String>, Error> {
        let mut entries: Vec<String> = Vec::new();

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let file_name = entry.file_name().into_string().unwrap();
            if file_name != "." && file_name != ".." && file_name != ".keep" {
                entries.push(file_name);
            }
        }
        Ok(entries)
    }

    fn uncomment_line(&self, line: &str, path: &str) {
        if self.exists(path) {
            let command = format!("sed -i '/{}/ s/# *//' {}", line, path);
            ValetCommandLine {}.run(command.as_str()).unwrap();
        }
    }
    fn comment_line(&self, line: &str, file: &str) {
        if self.exists(file) {
            let command = format!("sed -i '/{}/ s/^/# /' {}", line, file);
            ValetCommandLine {}.run(&command).unwrap();
        }
    }
    fn realpath(&self, path: &str) -> String {
        let canonical_path = fs::canonicalize(path).unwrap();
        canonical_path.to_string_lossy().into_owned()
    }
}
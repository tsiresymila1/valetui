use std::process::Command;
use crate::constants::Valet;
use crate::manager::command::ValetCommandLine;

pub struct Requirements {
    cli: ValetCommandLine,
    ignore_selinux: bool,
}

impl Requirements {
    pub fn new(cli: ValetCommandLine, ignore_selinux: bool) -> Requirements {
        Self {
            cli,
            ignore_selinux,
        }
    }

    pub fn set_ignore_selinux(&mut self, ignore: bool) -> &mut Self {
        self.ignore_selinux = ignore;
        self
    }

    pub fn check(&self) {
        self.home_path_is_inside_root();
        self.selinux_is_enabled();
    }

    fn home_path_is_inside_root(&self) {
        if Valet::home_path().starts_with("/root/") {
            panic!("Valet home directory is inside /root");
        }
    }

    fn selinux_is_enabled(&self) {
        if self.ignore_selinux {
            return;
        }
        let output = Command::new("sestatus").output().expect("Failed to execute command");
        let output_str = String::from_utf8_lossy(&output.stdout);
        if output_str.contains("SELinux status: enabled")
            && output_str.contains("Current mode: enforcing")
        {
            panic!("SELinux is in enforcing mode");
        }
    }
}
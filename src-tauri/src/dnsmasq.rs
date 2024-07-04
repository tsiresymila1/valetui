use std::fs::File;
use std::io::Write;
use crate::constants::{user, Valet};
use crate::manager::apt::Apt;
use crate::manager::command::ValetCommandLine;
use crate::manager::file_system::ValetFilesystem;
use crate::manager::interface::{CommandLine, Filesystem, PackageManager, ServiceManager};
use crate::manager::service_manager::ValetServiceManager;

pub struct DnsMasq {
    pm: Apt,
    sm: ValetServiceManager,
    cli: ValetCommandLine,
    files: ValetFilesystem,
    rclocal: String,
    resolvconf: String,
    dnsmasqconf: String,
    dnsmasq_opts: String,
    resolved_config_path: String,
    config_path: String,
    nm_config_path: String,
}

impl DnsMasq {
    pub fn new(pm: Apt, sm: ValetServiceManager, cli: ValetCommandLine, files: ValetFilesystem) -> Self {
        DnsMasq {
            pm,
            sm,
            cli,
            files,
            rclocal: "/etc/rc.local".to_string(),
            resolvconf: "/etc/resolv.conf".to_string(),
            dnsmasqconf: "/etc/dnsmasq.conf".to_string(),
            dnsmasq_opts: "/etc/dnsmasq.d/options".to_string(),
            resolved_config_path: "/etc/systemd/resolved.conf".to_string(),
            config_path: "/etc/dnsmasq.d/valet".to_string(),
            nm_config_path: "/etc/NetworkManager/conf.d/valet.conf".to_string(),
        }
    }

    pub fn install(&self, domain: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.dnsmasq_setup().unwrap();
        self.stop_resolved();
        self.create_custom_config_file(domain).unwrap();
        self.sm.restart(vec!["dnsmasq"]);
        Ok(())
    }

    pub fn stop(&self) {
        self.sm.stop(vec!["dnsmasq"])
    }

    pub fn restart(&self) {
        self.sm.restart(vec!["dnsmasq"])
    }

    pub fn update_domain(&self, new_domain: &str) {
        self.create_custom_config_file(new_domain).unwrap();
        self.sm.restart(vec!["dnsmasq"])
    }

    pub fn uninstall(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.sm.remove_valet_dns();
        self.cli.passthru("rm -rf /opt/valet-linux").unwrap();
        self.files.unlink(&self.config_path).unwrap();
        self.files.unlink(&self.dnsmasq_opts).unwrap();
        self.files.unlink(&self.nm_config_path).unwrap();
        self.files.restore(&self.resolved_config_path).unwrap();

        self.lock_resolv_conf().unwrap();
        self.files.restore(&self.rclocal).unwrap();

        self.cli.passthru("rm -f /etc/resolv.conf").unwrap();
        self.sm.stop(vec!["systemd-resolved"]);
        self.sm.start(vec!["systemd-resolved"]);
        self.files.symlink("/run/systemd/resolve/resolv.conf", &self.resolvconf).unwrap();

        self.files.restore(&self.dnsmasqconf).unwrap();
        self.files.comment_line("IGNORE_RESOLVCONF", "/etc/default/dnsmasq");

        self.pm.restart_network_manager();
        self.sm.restart(vec!["dnsmasq"]);
        println!("Valet DNS changes have been rolled back");
        Ok(())
    }

    fn lock_resolv_conf(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.files.is_link(&self.resolvconf) {
            self.cli.run(
                &format!("chattr -i {}", &self.resolvconf),
            ).unwrap();
        }
        Ok(())
    }

    fn merge_dns(&self) -> Result<(), Box<dyn std::error::Error>> {
        let opt_dir = "/opt/valet-linux";
        self.files.remove(&[opt_dir]).unwrap();
        self.files.ensure_dir_exists(opt_dir, &user(), 0o775).unwrap();
        self.sm.remove_valet_dns();
        if self.files.exists(&self.rclocal) {
            self.files.restore(&self.rclocal).unwrap();
        }

        Ok(())
    }

    fn create_custom_config_file(&self, domain: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = File::create(&self.config_path).unwrap();
        file.write_all(format!("address=/{}.127.0.0.1\nserver=1.1.1.1\nserver=8.8.8.8\n", domain).as_bytes()).unwrap();
        Ok(())
    }

    fn stop_resolved(&self) {
        if !self.sm.disabled("systemd-resolved") {
            self.sm.disable("systemd-resolved");
        }
        self.sm.stop(vec!["systemd-resolved"]);
    }

    fn dnsmasq_setup(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.pm.ensure_installed("dnsmasq");
        self.sm.enable("dnsmasq");
        self.files.ensure_dir_exists("/etc/NetworkManager/conf.d", &user(), 0o775).unwrap();
        self.files.ensure_dir_exists("/etc/dnsmasq.d", &user(), 0o775).unwrap();

        self.files.uncomment_line("IGNORE_RESOLVCONF", "/etc/default/dnsmasq");

        self.lock_resolv_conf().unwrap();
        self.merge_dns().unwrap();

        self.files.unlink("/etc/dnsmasq.d/network-manager").unwrap();
        self.files.backup(&self.dnsmasqconf).unwrap();

        let dnsmasq_conf_stub = format!("{}/stubs/dnsmasq.conf.stub", Valet::root_path());
        let dnsmasq_opts_stub = format!("{}/stubs/dnsmasq_options.stub", Valet::root_path());
        let nm_config_stub = format!("{}/stubs/networkmanager.conf.stub", Valet::root_path());

        self.files.put(&self.dnsmasqconf, dnsmasq_conf_stub.as_str()).unwrap();
        self.files.put(&self.dnsmasq_opts, dnsmasq_opts_stub.as_str()).unwrap();
        self.files.put(&self.nm_config_path, nm_config_stub.as_str()).unwrap();
        Ok(())
    }
}
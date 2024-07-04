use crate::configuration::Configuration;
use crate::constants::{group, NGINX_CONF, SITES_AVAILABLE_CONF, SITES_ENABLED_CONF, user, Valet, VALET_SERVER_PATH, VALET_STATIC_PREFIX};
use crate::manager::apt::Apt;
use crate::manager::command::ValetCommandLine;
use crate::manager::file_system::ValetFilesystem;
use crate::manager::interface::{CommandLine, Filesystem, PackageManager, ServiceManager};
use crate::manager::service_manager::ValetServiceManager;
use crate::site_secure::SiteSecure;

pub struct Nginx {
    pm: Apt,
    sm: Box<dyn ServiceManager>,
    cli: ValetCommandLine,
    files: ValetFilesystem,
    configuration: Configuration,
    site_secure: SiteSecure,
}

impl Nginx {
    pub fn new(
        pm: Apt,
        sm: Box<dyn ServiceManager>,
        cli: ValetCommandLine,
        files: ValetFilesystem,
        configuration: Configuration,
        site_secure: SiteSecure,
    ) -> Self {
        Self {
            pm,
            sm,
            cli,
            files,
            configuration,
            site_secure,
        }
    }

    pub fn install(&self) {
        self.pm.ensure_installed("nginx");
        self.sm.enable("nginx");
        self.handle_apache_service();
        self.files.ensure_dir_exists("/etc/nginx/sites-available", &user(), 0o775).unwrap();
        self.files.ensure_dir_exists("/etc/nginx/sites-enabled", &user(), 0o775).unwrap();
        self.stop();
        self.install_configuration();
        self.install_server(None);
        self.install_nginx_directory();
    }

    pub fn restart(&self) {
        self.sm.restart(vec!["nginx"])
    }

    pub fn stop(&self) {
        self.sm.stop(vec!["nginx"])
    }
    pub fn status(&self) {
        self.sm.print_status("nginx");
    }
    fn handle_apache_service(&self) {
        if self.pm.installed("apache2") {
            return;
        }
        if !self.sm.disabled("apache2") {
            self.sm.disable("apache2");
        }
        self.sm.stop(vec!["apache2"])
    }
    pub fn configured_sites(&self) -> Vec<String> {
        let entries = self.files.scandir(&format!("{}/Nginx", Valet::home_path())).unwrap();
        let filtered_entries: Vec<String> = entries
            .into_iter()
            .filter(|file| !file.starts_with('.'))
            .collect();
        filtered_entries
    }

    fn install_configuration(&self) {
        let contents = self.files.get(format!("{}/stubs/nginx.conf", Valet::root_path()).as_str()).unwrap();
        let mut pid_path = "pid /run/nginx.pid";
        let has_pid_option = self.cli.run("cat /lib/systemd/system/nginx.service").unwrap().contains("pid /");
        if has_pid_option {
            pid_path = "# pid /run/nginx.pid";
        }
        self.files.backup(NGINX_CONF).unwrap();
        let content_cloned = contents.as_str();
        let binding = content_cloned
            .replace("VALET_USER", user().as_str())
            .replace("VALET_GROUP", &group().unwrap())
            .replace("VALET_HOME_PATH", Valet::home_path().as_str())
            .replace("VALET_PID", pid_path);
        let replaced_contents = binding.as_str();
        self.files.put(
            NGINX_CONF,
            replaced_contents,
        ).unwrap()
    }

    pub fn install_nginx_directory(&self) {
        let nginx_dir = format!("{}/Nginx", Valet::home_path());
        if !self.files.is_dir(nginx_dir.as_str()) {
            self.files.mkdir(nginx_dir.as_str(), 0o775).unwrap()
        }
        self.files.put(format!("{}/.keep", nginx_dir).as_str(), "\n").unwrap();
        self.rewrite_secure_nginx_files()
    }

    fn rewrite_secure_nginx_files(&self) {
        let domain = self.configuration.get("domain").unwrap().to_string();
        self.site_secure.re_secure_for_new_domain(domain.clone().as_str(), domain.clone().as_str());
    }

    pub fn install_server(&self, socket_file_name: Option<&str>) {
        let default_valet_conf = self.files.get(format!("{}/stubs/valet.conf", Valet::home_path()).as_str()).unwrap();
        let valet_conf = default_valet_conf
            .replace("VALET_HOME_PATH", Valet::home_path().as_str())
            .replace("VALET_FPM_SOCKET_FILE", format!("{}/{}", Valet::home_path(), socket_file_name.unwrap_or("")).as_str())
            .replace("VALET_SERVER_PATH", VALET_SERVER_PATH)
            .replace("VALET_STATIC_PREFIX", VALET_STATIC_PREFIX)
            .replace("VALET_PORT", self.configuration.get("port").unwrap().to_string().as_str());

        self.files.put(SITES_AVAILABLE_CONF, valet_conf.as_str()).unwrap();
        if self.files.exists("/etc/nginx/sites-enabled/default") {
            self.files.unlink("/etc/nginx/sites-enabled/default").unwrap()
        }
        self.cli.run(format!("ln -snf {}{}", SITES_AVAILABLE_CONF, SITES_ENABLED_CONF).as_str()).unwrap();
        self.files.backup("/etc/nginx/fastcgi_params").unwrap();
        self.files.put("/etc/nginx/fastcgi_params", self.files.get(format!("{}/stubs/fastcgi_params", Valet::root_path()).as_str()).unwrap().as_str()).unwrap();
    }
}
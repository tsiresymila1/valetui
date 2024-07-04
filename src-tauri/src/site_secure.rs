use std::collections::HashSet;
use std::fs::File;
use std::io::{Read, Write};

use chrono::{Duration, Utc};
use regex::Regex;
use serde_json::Value;

use crate::configuration::Configuration;
use crate::constants::{user, Valet, VALET_SERVER_PATH, VALET_STATIC_PREFIX};
use crate::manager::command::ValetCommandLine;
use crate::manager::file_system::ValetFilesystem;
use crate::manager::interface::{CommandLine, Filesystem};
use crate::paths::{Paths, PathTrait};
use crate::php_fpm::PhpFpm;

pub struct SiteSecure {
    files: ValetFilesystem,
    cli: ValetCommandLine,
    config: Configuration,
    ca_certificate_path: String,
    ca_certificate_pem: String,
    ca_certificate_key: String,
    ca_certificate_srl: String,
    ca_certificate_organization: String,
    ca_certificate_common_name: String,
    certificate_dummy_email: String,
}

impl SiteSecure {
    pub fn new(filesystem: ValetFilesystem, cli: ValetCommandLine, config: Configuration) -> Self {
        Self {
            files: filesystem,
            cli,
            config,
            ca_certificate_path: "/usr/local/share/ca-certificates/".to_string(),
            ca_certificate_pem: "ValetLinuxCASelfSigned.pem".to_string(),
            ca_certificate_key: "ValetLinuxCASelfSigned.key".to_string(),
            ca_certificate_srl: "ValetLinuxCASelfSigned.srl".to_string(),
            ca_certificate_organization: "Valet Linux CA Self Signed Organization".to_string(),
            ca_certificate_common_name: "Valet Linux CA Self Signed CN".to_string(),
            certificate_dummy_email: "certificate@valet.linux".to_string(),
        }
    }

    pub fn secure(&self, url: &str, stub: Option<&str>) {
        let binding = self.prepare_conf(url, true);
        let stub = stub.unwrap_or(binding.as_deref().unwrap());
        self.files.ensure_dir_exists(self.ca_path(None).as_str(), &user(), 0o775).unwrap();
        self.files.ensure_dir_exists(self.certificates_path(None).as_str(), &user(), 0o775).unwrap();
        let ca_expire_in_days = self.calculate_expiry_days(20 * 365);
        self.create_ca(ca_expire_in_days);
        let cert_expire_in_days = self.calculate_expiry_days(1 * 365);
        self.create_certificate(url, cert_expire_in_days);
        self.files.put(&self.nginx_path(Some(url)), self.build_secure_nginx_server(url, stub).as_str()).unwrap();
    }

    pub fn unsecure(&self, url: &str, preserve_unsecure_config: bool) {
        let mut stub = None;
        if self.files.exists(&self.certificates_path(Some(&(url.to_owned() + ".crt")))) {
            if preserve_unsecure_config {
                stub = Some(self.prepare_conf(url, false));
            }
            self.files.unlink(&self.nginx_path(Some(url))).unwrap();
            self.files.unlink(&self.certificates_path(Some(&(url.to_owned() + ".conf")))).unwrap();
            self.files.unlink(&self.certificates_path(Some(&(url.to_owned() + ".key")))).unwrap();
            self.files.unlink(&self.certificates_path(Some(&(url.to_owned() + ".csr")))).unwrap();
            self.files.unlink(&self.certificates_path(Some(&(url.to_owned() + ".crt")))).unwrap();
        }

        if let Some(stub) = stub {
            let stub = self.build_unsecure_nginx_server(url, &stub.unwrap());
            self.files.put(&self.nginx_path(Some(url)), &stub).unwrap();
        }
    }

    pub fn secured(&self) -> HashSet<String> {
        let entries = self.files.scandir(&self.certificates_path(None)).unwrap();
        let mut secured_sites = HashSet::new();
        for entry in entries {
            let site = entry.replace(".key", "").replace(".csr", "").replace(".crt", "").replace(".conf", "");
            secured_sites.insert(site);
        }
        secured_sites
    }

    pub fn regenerate_secured_sites_config(&self) {
        for url in self.secured() {
            self.files.put(&self.nginx_path(Some(&url)), &self.build_secure_nginx_server(&url, "")).unwrap();
        }
    }

    pub fn re_secure_for_new_domain(&self, old_domain: &str, domain: &str) {
        if !self.files.exists(&self.certificates_path(None)) {
            return;
        }
        let secured = self.secured();
        for old_url in secured {
            let new_url = old_url.replace(&format!(".{}", old_domain), &format!(".{}", domain));
            let has_conf = self.files.exists(&self.nginx_path(Some(&old_url)));
            let mut nginx_conf = None;
            if has_conf {
                nginx_conf = Some(self.files.get(&self.nginx_path(Some(&old_url))).unwrap());
                if let Some(ref mut conf) = nginx_conf {
                    *conf = conf.replace(&old_url, &new_url);
                }
            }
            self.unsecure(&old_url, false);
            self.secure(&new_url, nginx_conf.as_deref());
        }
    }

    fn create_ca(&self, ca_expire_in_days: i64) {
        let ca_pem_path = self.ca_path(Some(&self.ca_certificate_pem));
        let ca_key_path = self.ca_path(Some(&self.ca_certificate_key));

        if self.files.exists(&ca_key_path) && self.files.exists(&ca_pem_path) {
            self.trust_ca(&ca_pem_path);
            return;
        }

        self.files.unlink(&ca_key_path).unwrap();
        self.files.unlink(&ca_pem_path).unwrap();
        self.un_trust_ca();

        let subject = format!(
            "/C=/ST=/O={}/localityName=/commonName={}/organizationalUnitName=Developers/emailAddress={}/",
            self.ca_certificate_organization,
            self.ca_certificate_common_name,
            self.certificate_dummy_email,
        );

        self.cli.run_as_user(&format!(
            "openssl req -new -newkey rsa:2048 -days {} -nodes -x509 -subj \"{}\" -keyout \"{}\" -out \"{}\"",
            ca_expire_in_days,
            subject,
            ca_key_path,
            ca_pem_path
        )).unwrap();

        self.trust_ca(&ca_pem_path);
    }

    fn un_trust_ca(&self) {
        self.files.remove(&[format!("{}{}.crt", self.ca_certificate_path, self.ca_certificate_pem).as_str()]).unwrap();
        self.cli.run("sudo update-ca-certificates").unwrap();
    }

    fn trust_ca(&self, ca_pem_path: &str) {
        self.files.copy(ca_pem_path, &format!("{}{}.crt", self.ca_certificate_path, self.ca_certificate_pem)).unwrap();
        self.cli.run("sudo update-ca-certificates").unwrap();
        self.cli.run_as_user(&format!(
            "certutil -d sql:$HOME/.pki/nssdb -A -t TC -n \"{}\" -i \"{}\"",
            self.ca_certificate_organization, ca_pem_path
        )).unwrap();
        self.cli.run_as_user(&format!(
            "certutil -d $HOME/.mozilla/firefox/*.default -A -t TC -n \"{}\" -i \"{}\"",
            self.ca_certificate_organization, ca_pem_path
        )).unwrap();
        self.cli.run_as_user(&format!(
            "certutil -d $HOME/snap/firefox/common/.mozilla/firefox/*.default -A -t TC -n \"{}\" -i \"{}\"",
            self.ca_certificate_organization, ca_pem_path
        )).unwrap();
    }

    fn create_certificate(&self, url: &str, certificate_expire_in_days: i64) {
        let ca_pem_path = self.ca_path(Some(&self.ca_certificate_pem));
        let ca_key_path = self.ca_path(Some(&self.ca_certificate_key));
        let ca_srl_path = self.ca_path(Some(&self.ca_certificate_srl));

        let key_path = format!("{}/{}.key", self.certificates_path(None), url);
        let csr_path = format!("{}/{}.csr", self.certificates_path(None), url);
        let crt_path = format!("{}/{}.crt", self.certificates_path(None), url);
        let conf_path = format!("{}/{}.conf", self.certificates_path(None), url);

        self.files.unlink(&key_path).unwrap();
        self.files.unlink(&csr_path).unwrap();
        self.files.unlink(&crt_path).unwrap();

        let mut conf = File::create(&conf_path).unwrap();
        writeln!(conf, "[dn]").unwrap();
        writeln!(conf, "CN={}", url).unwrap();
        writeln!(conf, "[req]").unwrap();
        writeln!(conf, "distinguished_name = dn").unwrap();
        writeln!(conf, "[EXT]").unwrap();
        writeln!(conf, "subjectAltName=DNS:{}", url).unwrap();
        writeln!(conf, "keyUsage=digitalSignature").unwrap();
        writeln!(conf, "extendedKeyUsage=serverAuth").unwrap();
        writeln!(conf, "[x509_ext]").unwrap();
        writeln!(conf, "subjectAltName=DNS:{}", url).unwrap();

        self.cli.run_as_user(&format!(
            "openssl req -new -newkey rsa:2048 -sha256 -nodes -keyout \"{}\" -subj \"/CN={}\" -out \"{}\" -config \"{}\"",
            key_path, url, csr_path, conf_path
        )).unwrap();
        self.cli.run_as_user(&format!(
            "openssl x509 -req -sha256 -in \"{}\" -CA \"{}\" -CAkey \"{}\" -CAcreateserial -CAserial \"{}\" -out \"{}\" -days {} -extfile \"{}\" -extensions x509_ext",
            csr_path, ca_pem_path, ca_key_path, ca_srl_path, crt_path, certificate_expire_in_days, conf_path
        )).unwrap();
    }

    fn build_unsecure_nginx_server(&self, url: &str, stub: &str) -> String {
        let unsecure_port = self.config.get("port").unwrap_or(Value::String("80".to_string()));
        let secure_port = self.config.get("https_port").unwrap_or(Value::String("433".to_string()));
        stub
            .replace("VALET_HOME_PATH", Valet::home_path().as_str())
            .replace("VALET_SERVER_PATH", VALET_SERVER_PATH)
            .replace("VALET_STATIC_PREFIX", VALET_STATIC_PREFIX)
            .replace("VALET_SITE", url)
            .replace("VALET_HTTP_PORT", unsecure_port.clone().as_str().unwrap())
            .replace("VALET_HTTPS_PORT", secure_port.clone().as_str().unwrap())
    }

    fn build_secure_nginx_server(&self, url: &str, stub: &str) -> String {
        let secure_port = self.config.get("https_port").unwrap_or(Value::String("433".to_string()));
        let unsecure_port = self.config.get("port").unwrap_or(Value::String("80".to_string()));
        let mut file = File::open(format!("{}/stubs/secure.valet.conf", Valet::root_path())).unwrap();
        let mut contents = String::new();
        let path = self.certificates_path(None);
        file.read_to_string(&mut contents).unwrap();
        stub
            .replace("VALET_HOME_PATH", Valet::home_path().as_str())
            .replace("VALET_SERVER_PATH", VALET_SERVER_PATH)
            .replace("VALET_STATIC_PREFIX", VALET_STATIC_PREFIX)
            .replace("VALET_SITE", url)
            .replace("VALET_CERT", format!("{}?{}.cert", path, url).as_str())
            .replace("VALET_KEY", format!("{}?{}.key", path, url).as_str())
            .replace("VALET_HTTP_PORT", unsecure_port.as_str().unwrap())
            .replace("VALET_HTTPS_PORT", secure_port.as_str().unwrap())
            .replace("VALET_REDIRECT_PORT", self.https_suffix().as_str())
        // todo!()
            // .replace("VALET_FPM_SOCKET_FILE", self.fpm.socket_file_name(None).as_str())
    }

    fn calculate_expiry_days(&self, days: i64) -> i64 {
        let now = Utc::now();
        let expiry_date = now + Duration::days(days);
        (expiry_date - now).num_days()
    }

    fn nginx_path(&self, url: Option<&str>) -> String {
        Paths::nginx_path(url)
    }

    fn certificates_path(&self, file: Option<&str>) -> String {
        Paths::certificates_path(file)
    }

    fn ca_path(&self, file: Option<&str>) -> String {
        Paths::ca_path(file)
    }

    fn prepare_conf(&self, url: &str, secure: bool) -> Option<String> {
        if !self.files.exists(&self.nginx_path(Some(url))) {
            return None;
        }
        let existing_conf = self.files.get(&self.nginx_path(Some(url))).unwrap();
        let re = Regex::new(r"# valet stub: (?<tls>secure)?\.?(?<stub>.*?).valet.conf").unwrap();
        let stub_detail = re.captures(&existing_conf)?;
        if stub_detail.name("stub").is_none() {
            return None;
        }

        let stub = stub_detail.name("stub").unwrap().as_str();
        if stub == "proxy" {
            let proxy_pass = self.get_proxy_pass(url, Some(&existing_conf)).unwrap();
            let stub_path = if secure {
                format!("{}/stubs/secure.proxy.valet.conf", Valet::root_path())
            } else {
                format!("{}/stubs/proxy.valet.conf", Valet::root_path())
            };
            let mut stub = self.files.get(&stub_path).unwrap();
            stub = stub.replace("VALET_PROXY_HOST", &proxy_pass);
            return Some(stub);
        }
        if stub == "isolated" {
            let php_version = self.isolated_php_version(&existing_conf);
            let stub_path = if secure {
                format!("{}/stubs/secure.isolated.valet.conf", Valet::root_path())
            } else {
                format!("{}/stubs/isolated.valet.conf", Valet::root_path())
            };
            let mut stub = self.files.get(&stub_path).unwrap();
            // stub = stub.replace("VALET_FPM_SOCKET_FILE", self.fpm.socket_file_name(Some(php_version.as_str())).as_str());
            stub = stub.replace("VALET_ISOLATED_PHP_VERSION", &php_version);
            return Some(stub);
            todo!();
        }
        None
    }

    fn get_proxy_pass(&self, url: &str, site_conf: Option<&str>) -> Option<String> {
        let binding = self.files.get(&self.nginx_path(Some(url))).unwrap();
        let site_conf = site_conf.unwrap_or(&binding);
        let re = Regex::new(r"proxy_pass (?<host>.*?);").unwrap();
        let matches = re.captures(site_conf)?;
        Some(matches.name("host")?.as_str().to_string())
    }

    fn generate_certificate_conf(&self, path: &str, url: &str) {
        let config = self.files.get(&format!("{}/stubs/openssl.conf", Valet::root_path())).unwrap();
        let config = config.replace("VALET_DOMAIN", url);
        self.files.put(path, &config).unwrap();
    }

    fn https_suffix(&self) -> String {
        let port = self.config.get("https_port").and_then(|arg0: Value| Value::as_u64(&arg0)).unwrap_or(443);
        if port == 443 {
            String::new()
        } else {
            format!(":{}", port)
        }
    }

    fn isolated_php_version(&self, site_conf: &str) -> String {
        let re = Regex::new(r"^# ISOLATED_PHP_VERSION=(.*?)\n").unwrap();
        if site_conf.contains("# ISOLATED_PHP_VERSION") {
            if let Some(captures) = re.captures(site_conf) {
                return captures.get(1).unwrap().as_str().to_string();
            }
        }
        String::new()
    }
}


use serde_json::Value;

use crate::constants::{user, Valet};
use crate::manager::file_system::ValetFilesystem;
use crate::manager::interface::Filesystem;

#[derive(Clone, Copy)]
pub struct Configuration {
    files: ValetFilesystem,
}

impl Configuration {
    // Create a new Valet configuration instance
    pub fn new(files: ValetFilesystem) -> Configuration {
        Configuration { files }
    }

    pub fn install(&self) {
        self.create_configuration_directory();
        self.create_drivers_directory();
        self.create_sites_directory();
        self.create_extensions_directory();
        self.create_log_directory();
        self.create_certificates_directory();
        self.write_base_configuration();
        // self.files.chown(&self.path(), &user());
    }

    // Uninstall the Valet configuration folder
    fn uninstall(&self) {
        if self.files.is_dir(Valet::home_path().as_str()) {
            self.files.remove(&[Valet::home_path().as_str()]).expect("Error to remove file ");
        }
    }

    // Add the given path to the configuration
    fn add_path(&self, path: &str, prepend: bool) {
        let mut config = self.read();
        let paths = config["paths"].as_array_mut().unwrap();
        if prepend {
            paths.insert(0, Value::String(path.to_string()));
        } else {
            paths.push(Value::String(path.to_string()));
        }
        *paths = paths.clone().into_iter().collect();
        self.write(&config);
    }

    // Remove the given path from the configuration
    fn remove_path(&self, path: &str) {
        let mut config = self.read();
        let paths = config["paths"].as_array_mut().unwrap();
        *paths = paths
            .iter()
            .filter(|&p| p != path)
            .map(|p| p.clone())
            .collect();

        self.write(&config);
    }

    // Prune all non-existent paths from the configuration
    fn prune(&self) {
        if !self.files.exists(&self.path()) {
            return;
        }

        let mut config = self.read();
        let paths = config["paths"].as_array_mut().unwrap();

        *paths = paths
            .iter()
            .filter(|&p| self.files.is_dir(p.as_str().unwrap()))
            .map(|p| p.clone())
            .collect();
        self.write(&config);
    }

    // Get a configuration value
    pub(crate) fn get(&self, key: &str) -> Option<Value> {
        let cgf = self.read();
        cgf.get(key).cloned()
    }
    // Set a configuration value
    pub(crate) fn set(&self, key: &str, value: Value) {
        let mut config = self.read();
        config[key] = value;
        self.write(&config);
    }

    // Parse domain based on configuration
    fn parse_domain(&self, site_name: &str) -> String {
        let domain = self.get("domain")
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_else(|| "test".to_string());
        if !site_name.ends_with(&format!(".{}", domain)) {
            format!("{}.{}", site_name, domain)
        } else {
            site_name.to_string()
        }
    }

    // Update a specific key in the configuration file
    fn update_key(&self, key: &str, value: Value) {
        let mut config = self.read();
        config[key] = value;
        self.write(&config);
    }

    // Write the given configuration to disk
    fn write(&self, config: &Value) {
        self.files
            .put(&self.path(), &serde_json::to_string_pretty(config).unwrap()).unwrap();
    }

    // Read the configuration file as JSON
    fn read(&self) -> serde_json::Value {
        let content = self.files.get(&self.path()).unwrap();
        serde_json::from_str(&content).unwrap_or_default()
    }

    // Create the Valet configuration directory
    fn create_configuration_directory(&self) {
        let valet_path = Valet::home_path();
        self.files.ensure_dir_exists(valet_path.as_str(), &user(), 0o755).unwrap();
    }

    // Create the Valet drivers directory
    fn create_drivers_directory(&self) {
        let drivers_directory = format!("{}/Drivers", Valet::home_path());
        if !self.files.is_dir(&drivers_directory) {
            self.files.mkdir(&drivers_directory, 0o775).unwrap();
            let sample_driver_content =
                self.files.get(&format!("{}/stubs/SampleValetDriver.php", Valet::root_path())).unwrap();
            self.files.put(
                &format!("{}/SampleValetDriver.php", drivers_directory),
                &sample_driver_content,
            ).unwrap();
        }
    }

    // Create the Valet sites directory
    fn create_sites_directory(&self) {
        let sites_directory = format!("{}/Sites", Valet::home_path());
        self.files.ensure_dir_exists(&sites_directory, &user(), 0o755).expect("create_sites_directory: panic message");
    }

    // Create the directory for Valet extensions
    fn create_extensions_directory(&self) {
        let extensions_directory = format!("{}/Extensions", Valet::home_path());
        self.files.ensure_dir_exists(&extensions_directory, &user(), 0o755).expect("create_extensions_directory: panic message");
    }

    // Create the directory for Nginx logs
    fn create_log_directory(&self) {
        let log_directory = format!("{}/Log", Valet::home_path());
        self.files.ensure_dir_exists(&log_directory, &user(), 0o755).expect("create_log_directory: panic message");
        self.files.touch(&format!("{}/nginx-error.log", log_directory)).expect("create_log_directory: panic message 2");
    }

    // Create the directory for SSL certificates
    fn create_certificates_directory(&self) {
        let certificates_directory = format!("{}/Certificates", Valet::home_path());
        self.files.ensure_dir_exists(&certificates_directory, &user(), 0o755).expect("TODO: panic message");
    }

    // Write the base, initial configuration for Valet
    fn write_base_configuration(&self) {
        if !self.files.exists(&self.path()) {
            let base_config = serde_json::json!({
                "domain": "test",
                "paths": [],
                "port": "80"
            });
            self.write(&base_config);
        }
    }

    // Get the configuration file path
    fn path(&self) -> String {
        format!("{}/config.json", Valet::home_path())
    }
}
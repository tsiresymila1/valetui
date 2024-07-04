use crate::manager::command::ValetCommandLine;
use crate::manager::file_system::ValetFilesystem;
use crate::manager::interface::{CommandLine, Filesystem, ServiceManager};

#[derive(Clone)]
pub struct ValetServiceManager {
    cli: ValetCommandLine,
    files: ValetFilesystem,
}

impl ValetServiceManager {
    pub fn new(cli: ValetCommandLine, files: ValetFilesystem) -> Self {
        Self { cli, files }
    }

    fn get_real_service(&self, service: &str) -> Result<String, String> {
        let status = self.cli.run(&format!("service {} status", service));
        match status {
            Ok(output) => {
                if output.contains("not-found") {
                    Err("Unable to determine service name.".into())
                } else {
                    Ok(service.to_string())
                }
            }
            Err(e) => Err(e.to_string()),
        }
    }

    fn handle_service(&self, services: Vec<&str>, action: &str) {
        for &service in &services {
            let real_service = self.get_real_service(service).expect("Unable to determine service name.");
            self.cli.quietly(&format!("sudo service {} {}", real_service, action)).unwrap();
        }
    }
}

impl ServiceManager for ValetServiceManager {
    fn start(&self, services: Vec<&str>) {
        self.handle_service(services, "start");
    }

    fn stop(&self, services: Vec<&str>) {
        self.handle_service(services, "stop");
    }

    fn restart(&self, services: Vec<&str>) {
        self.handle_service(services, "restart");
    }

    fn print_status(&self, service: &str) {
        let real_service = self.get_real_service(service).expect("Unable to determine service name.");
        let status = self.cli.run(&format!("service {} status", real_service));
        match status {
            Ok(output) => {
                if output.contains("running") {
                    println!("{} is running...", service);
                } else {
                    println!("{} is stopped...", service);
                }
            }
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    fn disabled(&self, service: &str) -> bool {
        let real_service = self.get_real_service(service).expect(format!("Unable to determine service with name {}.", service).as_str());
        match self.cli.run(&format!("systemctl is-enabled {}", real_service)) {
            Ok(output) => !output.trim().contains("enabled"),
            Err(_) => true,
        }
    }

    fn disable(&self, service: &str) {
        let real_service = self.get_real_service(service).expect("Unable to determine service name.");
        self.cli.quietly(&format!("sudo chmod -x /etc/init.d/{}", real_service)).unwrap();
        self.cli.quietly(&format!("sudo update-rc.d {} defaults", real_service)).unwrap();
        println!("{} disabled", service);
    }

    fn enable(&self, service: &str) {
        let real_service = self.get_real_service(service).expect(format!("Unable to determine service {}.",service).as_str());
        self.cli.quietly(&format!("sudo update-rc.d {} defaults", real_service)).unwrap();
        println!("{} enabled", service);
    }

    fn is_available(&self) -> bool {
        match self.cli.run("which service") {
            Ok(output) => !output.trim().is_empty(),
            Err(_) => false,
        }
    }

    fn is_systemd(&self) -> bool {
        false
    }

    fn remove_valet_dns(&self) {
        let service_path = "/etc/init.d/valet-dns";
        if self.files.exists(service_path) {
            println!("Removing Valet DNS service...");
            self.disable("valet-dns");
            self.stop(vec!["valet-dns"]);
            self.files.remove(&[service_path]).unwrap();
        }
    }
}
use crate::manager::apt::Apt;
use crate::manager::command::ValetCommandLine;
use crate::manager::file_system::ValetFilesystem;
use crate::manager::interface::{CommandLine, Filesystem, ServiceManager};
use crate::manager::service_manager::ValetServiceManager;

struct Mailpit {
    pm: Apt,
    sm: ValetServiceManager,
    cli: ValetCommandLine,
    files: ValetFilesystem,
}

impl Mailpit {
    const SERVICE_NAME: &'static str = "mailpit";
    pub fn new(
        pm: Apt, sm: ValetServiceManager, cli: ValetCommandLine, files: ValetFilesystem) -> Self {
        Self {
            pm,
            sm,
            cli,
            files,
        }
    }
    // Install method
    fn install(&self) {
        self.ensure_installed();
        self.create_service();
        self.sm.start(vec![Self::SERVICE_NAME]);

        // Handle mailhog removal and unsecuring if necessary
        if !self.sm.disabled("mailhog") {
            self.sm.disable("mailhog");
            if self.files.exists("/opt/valet-linux/mailhog") {
                self.files.remove(&["/opt/valet-linux/mailhog"]).unwrap();
            }
            let domain = "example.com"; // Replace with actual domain fetch
            if self.files.exists(&format!("/path/to/Nginx/mailhog.{}", domain)) {
                // SiteSecureFacade::unsecure("mailhog.$domain");
                println!("Unsecuring mailhog.{}", domain);
            }
        }
    }

    // Start method
    fn start(&self) {
        self.sm.start(vec![Self::SERVICE_NAME]);
    }

    fn restart(&self) {
        self.sm.restart(vec![Self::SERVICE_NAME]);
    }

    // Stop method
    fn stop(&self) {
        self.sm.stop(vec![Self::SERVICE_NAME]);
    }

    // Status method
    fn status(&self) {
        self.sm.print_status(Self::SERVICE_NAME);
    }

    // Uninstall method
    fn uninstall(&self) {
        self.stop();
    }

    // Ensure Mailpit is installed method
    fn ensure_installed(&self) {
        if !self.is_available() {
            self.cli.run_as_user("curl -sL https://raw.githubusercontent.com/axllent/mailpit/develop/install.sh | bash").unwrap();
        }
    }

    // Create Mailpit service method
    fn create_service(&self) {
        let service_path;
        let service_file;
        if self.sm.is_systemd() {
            service_path = "/etc/systemd/system/mailpit.service";
            service_file = "/path/to/cli/stubs/init/mailpit";
        } else {
            service_path = "/etc/init.d/mailpit";
            service_file = "/path/to/cli/stubs/init/mailpit.sh";
        }

        let service_content = self.files.get(service_file).unwrap();
        self.files.put(service_path, &service_content).unwrap();

        if !self.sm.is_systemd() {
            // self.cli.run("chmod +x $servicePath");
            println!("chmod +x {}", service_path);
        }

        self.sm.enable(Self::SERVICE_NAME);

        self.update_domain();
    }

    // Update domain method
    fn update_domain(&self) {
        let domain = "example.com"; // Replace with actual domain fetch
        println!("Updating domain for HTTP access: mails.{}", domain);
        todo!()
    }
    fn is_available(&self) -> bool {
        let output = self.cli.run_as_user("which mailpit").unwrap();
        !output.trim().is_empty()
    }
}
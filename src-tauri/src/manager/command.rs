use std::error::Error;
use std::io;
use std::process::Command;
use crate::manager::interface::CommandLine;

#[derive(Clone, Copy)]
pub struct ValetCommandLine;

impl CommandLine for ValetCommandLine {
    fn quietly(&self, command: &str) -> Result<(), Box<dyn Error>> {
        let _ = Command::new("sh")
            .arg("-c")
            .arg(format!("{} > /dev/null 2>&1", command))
            .output()?;
        Ok(())
    }
    fn quietly_as_user(&self, command: &str) -> Result<(), Box<dyn Error>> {
        let _ = Command::new("sh")
            .arg("-c")
            .arg(format!("sudo -u $(whoami) {} > /dev/null 2>&1", command))
            .output()?;
        Ok(())
    }

    fn passthru(&self, command: &str) -> Result<(), Box<dyn Error>> {
        let status = Command::new("sh")
            .arg("-c")
            .arg(command)
            .status()?;

        if !status.success() {
            return Err(Box::new(io::Error::new(io::ErrorKind::Other, format!("Command failed with status {:?}", status))));
        }

        Ok(())
    }

    fn run(&self, command: &str) -> Result<String, Box<dyn Error>> {
        let output = Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()?;

        if !output.status.success() {
            return Err(Box::new(io::Error::new(io::ErrorKind::Other, String::from_utf8_lossy(&output.stderr).to_string())));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    fn run_as_user(&self, command: &str) -> Result<String, Box<dyn Error>> {
        let user = whoami::username();
        let output = Command::new("sh")
            .arg("-c")
            .arg(format!("sudo -u {} {}", user, command))
            .output()?;

        if !output.status.success() {
            return Err(Box::new(io::Error::new(io::ErrorKind::Other, String::from_utf8_lossy(&output.stderr).to_string())));
        }
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

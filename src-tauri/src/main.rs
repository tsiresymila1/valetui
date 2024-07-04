// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod manager;
mod constants;
mod configuration;
mod requirements;
mod site_secure;
mod paths;
mod site;
mod nginx;
mod devtools;
mod php_fpm;
mod dnsmasq;
mod mailpit;

use tauri::{Manager, SystemTray, SystemTrayEvent};
use tauri::{CustomMenuItem, SystemTrayMenu};
use crate::configuration::Configuration;
use crate::manager::apt::Apt;
use crate::manager::command::ValetCommandLine;
use crate::manager::file_system::ValetFilesystem;
use crate::manager::service_manager::ValetServiceManager;
use crate::manager::systemd::ValetSystemDManager;
use crate::nginx::Nginx;
use crate::requirements::Requirements;
use crate::site_secure::SiteSecure;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn main() {

    //
    // let cli = ValetCommandLine{};
    // let req = Requirements::new(cli, true);
    // req.check();
    // let files = ValetFilesystem{};
    // let conf = Configuration::new(files.clone());
    // conf.install();
    // let sm  = ValetSystemDManager::new(cli, files);
    // let pm = Apt::new(Box::new(cli), Box::new(sm.clone()));
    // let site_secure = SiteSecure::new(files.clone(),cli,conf);
    // let nginx = Nginx::new(pm,Box::new(sm.clone()),cli,files.clone(),conf,site_secure);
    // nginx.install();

    let hide = CustomMenuItem::new("hide".to_string(), "Hide");
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let open = CustomMenuItem::new("open".to_string(), "Open");
    let tray_menu = SystemTrayMenu::new()
        .add_item(open)
        .add_item(hide)
        .add_item(quit);
    let tray = SystemTray::new().with_menu(tray_menu);
    tauri::Builder::default()
        .system_tray(tray)
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::MenuItemClick { id, .. } => {
                let _item_handle = app.tray_handle().get_item(&id);
                match id.as_str() {
                    "open" => {
                        let window = app.get_window("main").unwrap();
                        window.center().unwrap();
                        window.show().unwrap();
                    }
                    "hide" => {
                        let window = app.get_window("main").unwrap();
                        window.hide().unwrap();
                    }
                    "quit" => {
                        let window = app.get_window("main").unwrap();
                        window.close().unwrap();
                    }
                    _ => {}
                }
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

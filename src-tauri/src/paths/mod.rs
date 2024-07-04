use crate::constants::Valet;

pub struct Paths;

pub trait PathTrait {
    fn sites_path(file: Option<&str>) -> String;
    fn certificates_path(file: Option<&str>) -> String;
    fn ca_path(file: Option<&str>) -> String;
    fn nginx_path(file: Option<&str>) -> String;
}

impl PathTrait for  Paths {
    fn sites_path(file: Option<&str>) -> String {
        let file_path = file.map_or("".to_string(), |f| format!("/{}", f));
        format!("{}/Sites{}", Valet::home_path(), file_path)
    }

    fn certificates_path(file: Option<&str>) -> String {
        let file_path = file.map_or("".to_string(), |f| format!("/{}", f));
        format!("{}/Certificates{}", Valet::home_path(), file_path)
    }
    fn ca_path(file: Option<&str>) -> String {
        let file_path = file.map_or("".to_string(), |f| format!("/{}", f));
        format!("{}/CA{}", Valet::home_path(), file_path)
    }
    fn nginx_path(file: Option<&str>) -> String {
        let file_path = file.map_or("".to_string(), |f| format!("/{}", f));
        format!("{}/Nginx{}", Valet::home_path(), file_path)
    }
}
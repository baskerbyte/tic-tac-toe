use std::str::FromStr;

use crate::entities::system::Environment;

pub mod web;

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct AppSettings {
    pub web: web::WebSettings
}

pub trait Protocol {
    fn get_host(&self) -> &String;

    fn get_port(&self) -> &u16;

    fn socket(&self) -> String {
        format!("{}:{}", self.get_host(), self.get_port())
    }
}

impl AppSettings {
    pub fn new(path: &str) -> Self {
        let path = std::path::Path::new(path);

        if path.exists() {
            log::info!("Config file found. Reading configurations from file: '{}'", path.display());

            return Self::read(path)
        }

        Self::handle_environment(path)
    }

    fn read(path: &std::path::Path) -> Self {
        toml::from_str(&std::fs::read_to_string(path).unwrap())
            .expect("Error reading file")
    }

    fn handle_environment(path: &std::path::Path) -> Self {
        let environment = std::env::var("ENVIRONMENT")
            .map(|env| Environment::from_str(&env).unwrap())
            .unwrap_or(Environment::Production);

        match environment {
            Environment::Production | Environment::Stage => {
                log::info!("Reading configurations from ENV or using default.");

                // Read configs from env or use default
                Self::default()
            },
            Environment::Development => {
                Self::write(path);
                log::error!("Default configurations written to '{path:?}'. Please edit this file to continue.");

                std::process::exit(1)
            }
        }
    }

    fn write(path: &std::path::Path) {
        std::fs::write(path, toml::to_string(&Self::default()).unwrap())
            .expect("Error serializing to TOML");
    }
}
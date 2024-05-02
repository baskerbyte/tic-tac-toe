use std::str::FromStr;

use crate::entities::system::Environment;

pub mod cert;
pub mod web;

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct AppSettings {
    pub web: web::WebSettings,
    pub certificates: cert::Certificates,
    #[serde(skip)]
    pub environment: Environment,
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
        let environment = std::env::var("ENVIRONMENT")
            .map(|env| Environment::from_str(&env).unwrap())
            .unwrap_or(Environment::Production);

        if path.exists() {
            log::info!(
                "Config file found. Reading configurations from file: '{}'",
                path.display()
            );

            return Self::read(path, environment);
        }

        Self::handle_environment(path, environment)
    }

    fn read(path: &std::path::Path, env: Environment) -> Self {
        let partial: AppSettings =
            toml::from_str(&std::fs::read_to_string(path).unwrap()).expect("Error reading file");

        Self {
            web: partial.web,
            certificates: partial.certificates,
            environment: env
        }
    }

    fn handle_environment(path: &std::path::Path, env: Environment) -> Self {
        match env {
            Environment::Production | Environment::Stage => {
                log::info!("Reading configurations from ENV or using default.");

                // Read configs from env or use default
                Self {
                    web: Default::default(),
                    certificates: Default::default(),
                    environment: env,
                }
            }
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

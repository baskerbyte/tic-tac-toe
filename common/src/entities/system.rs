#[derive(Debug, Default)]
pub enum Environment {
    #[default]
    Production,
    Stage,
    Development,
}

impl std::str::FromStr for Environment {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "PRODUCTION" => Ok(Self::Production),
            "STAGE" => Ok(Self::Stage),
            "DEVELOPMENT" => Ok(Self::Development),
            _ => Err(Self::Err::new(
                std::io::ErrorKind::NotFound,
                "environment not recognized",
            )),
        }
    }
}

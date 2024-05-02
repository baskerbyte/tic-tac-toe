#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct Certificates {
    pub websocket: Certificate,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Certificate {
    pub path: String,
}

impl Default for Certificate {
    fn default() -> Self {
        Self {
            path: "".to_string()
        }
    }
}

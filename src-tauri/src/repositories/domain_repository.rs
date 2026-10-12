use std::{fs, path::PathBuf};

use crate::models::Domain;

pub struct DomainRepository {
    file_path: PathBuf,
}

impl DomainRepository {
    pub fn new(app_data_dir: PathBuf) -> Self {
        Self {
            file_path: app_data_dir.join("domains.json"),
        }
    }

    pub fn load(&self) -> Result<Vec<Domain>, String> {
        if !self.file_path.exists() {
            return Ok(Self::default_domains());
        }

        let content = fs::read_to_string(&self.file_path)
            .map_err(|error| format!("Failed to read domains file: {}", error))?;

        serde_json::from_str(&content)
            .map_err(|error| format!("Failed to parse domains file: {}", error))
    }

    pub fn save(&self, domains: &[Domain]) -> Result<(), String> {
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("Failed to create data directory: {}", error))?;
        }

        let content = serde_json::to_string_pretty(domains)
            .map_err(|error| format!("Failed to serialize domains: {}", error))?;

        fs::write(&self.file_path, content)
            .map_err(|error| format!("Failed to write domains file: {}", error))
    }

    fn default_domains() -> Vec<Domain> {
        Vec::new()
    }
}

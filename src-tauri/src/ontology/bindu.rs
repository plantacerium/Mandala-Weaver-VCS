use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bindu {
    pub project_name: String,
    pub timestamp: u64,
}

impl Bindu {
    pub fn genesis(project_name: &str) -> Self {
        Self {
            project_name: project_name.to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}

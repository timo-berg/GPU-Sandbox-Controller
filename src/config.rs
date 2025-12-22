use serde::Deserialize;
use tokio::fs;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub queue_length: usize,
}

impl Config {
    pub async fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let contents: String = fs::read_to_string(path).await?;
        let config = toml::from_str(&contents)?;
        Ok(config)
    }
}

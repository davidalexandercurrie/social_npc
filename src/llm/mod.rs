pub mod ollama;

use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;

#[async_trait]
pub trait LlmClient: Send + Sync {
    async fn query(&self, prompt: String, working_dir: &Path) -> Result<String>;
}

pub use ollama::OllamaClient;
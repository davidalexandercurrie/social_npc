use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::Path;

use super::LlmClient;

pub struct OllamaClient {
    model: String,
    base_url: String,
}

impl OllamaClient {
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            base_url: "http://localhost:11434".to_string(),
        }
    }

    pub fn with_url(model: impl Into<String>, base_url: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            base_url: base_url.into(),
        }
    }
}

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
    format: String,
    options: OllamaOptions,
}

#[derive(Serialize)]
struct OllamaOptions {
    temperature: f32,
    top_p: f32,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

#[async_trait]
impl LlmClient for OllamaClient {
    async fn query(&self, prompt: String, _working_dir: &Path) -> Result<String> {
        log::debug!("Ollama query to model: {}", self.model);
        log::debug!("Prompt length: {} chars", prompt.len());

        let client = reqwest::Client::new();
        
        let request = OllamaRequest {
            model: self.model.clone(),
            prompt,
            stream: false,
            format: "json".to_string(),
            options: OllamaOptions {
                temperature: 0.7,
                top_p: 0.9,
            },
        };

        let response = tokio::time::timeout(
            std::time::Duration::from_secs(60),
            client
                .post(format!("{}/api/generate", self.base_url))
                .json(&request)
                .send()
        )
        .await
        .map_err(|_| anyhow!("Ollama query timed out after 60 seconds"))??;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Ollama request failed: {}", error_text));
        }

        let ollama_response: OllamaResponse = response.json().await
            .map_err(|e| anyhow!("Failed to parse Ollama response: {}", e))?;

        Ok(ollama_response.response)
    }
}

/// Check if Ollama is running and accessible
pub async fn check_ollama_status(base_url: &str) -> Result<()> {
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/api/tags", base_url))
        .send()
        .await
        .map_err(|e| anyhow!("Failed to connect to Ollama: {}", e))?;

    if !response.status().is_success() {
        return Err(anyhow!("Ollama is not responding correctly"));
    }

    Ok(())
}

/// List available models
pub async fn list_ollama_models(base_url: &str) -> Result<Vec<String>> {
    #[derive(Deserialize)]
    struct ModelsResponse {
        models: Vec<Model>,
    }
    
    #[derive(Deserialize)]
    struct Model {
        name: String,
    }

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/api/tags", base_url))
        .send()
        .await?;

    let models_response: ModelsResponse = response.json().await?;
    Ok(models_response.models.into_iter().map(|m| m.name).collect())
}
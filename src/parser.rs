use anyhow::{anyhow, Context, Result};
use serde::de::DeserializeOwned;
use serde_json::Value;

/// Extract and parse JSON from LLM responses, handling common formatting issues
pub fn extract_json<T: DeserializeOwned>(response: &str) -> Result<T> {
    // First check if the response is wrapped in code blocks
    let cleaned = if response.contains("```json") {
        // Extract content between ```json and ```
        let start = response.find("```json").unwrap() + 7; // 7 = len("```json")
        let end = response.rfind("```").unwrap();
        response[start..end].trim()
    } else if response.contains("```") {
        // Extract content between ``` and ```
        let start = response.find("```").unwrap() + 3;
        let end = response.rfind("```").unwrap();
        response[start..end].trim()
    } else {
        response
    };
    
    // Find JSON boundaries in the cleaned content
    let json_start = cleaned
        .find('{')
        .ok_or_else(|| anyhow!("No JSON found in LLM response: {}", response))?;
    
    let json_end = cleaned
        .rfind('}')
        .ok_or_else(|| anyhow!("No closing brace found in LLM response: {}", response))?;
    
    let json_str = &cleaned[json_start..=json_end];
    
    // First validate it's valid JSON
    let json_value: Value = serde_json::from_str(json_str)
        .with_context(|| format!("LLM returned invalid JSON. Response: {}", response))?;
    
    // Log the parsed JSON for debugging
    log::debug!("Parsed JSON from LLM: {}", serde_json::to_string_pretty(&json_value)?);
    
    // Then try to deserialize to the target type
    serde_json::from_value::<T>(json_value.clone())
        .with_context(|| {
            format!(
                "LLM JSON has incorrect format for type {}. Expected fields may be missing or have wrong types.\nReceived: {}",
                std::any::type_name::<T>(),
                serde_json::to_string_pretty(&json_value).unwrap_or_else(|_| json_str.to_string())
            )
        })
}
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::fs;

use super::templates::{NPC_BASE_DEFAULT, GM_BASE_DEFAULT};

/// Loads prompt templates from the filesystem with fallback to defaults
pub struct PromptLoader {
    prompts_dir: PathBuf,
}

impl PromptLoader {
    pub fn new(data_path: impl AsRef<Path>) -> Self {
        Self {
            prompts_dir: data_path.as_ref().join("prompts"),
        }
    }

    /// Load the NPC base prompt, using default if file doesn't exist
    pub fn load_npc_base(&self) -> Result<String> {
        // Try multiple possible locations
        let possible_paths = vec![
            self.prompts_dir.join("core").join("npc_base.md"),
            self.prompts_dir.join("npc_base.md"),
        ];

        for path in possible_paths {
            if path.exists() {
                log::debug!("Loading NPC base prompt from: {:?}", path);
                return fs::read_to_string(path)
                    .map_err(|e| anyhow::anyhow!("Failed to read NPC base prompt: {}", e));
            }
        }

        log::debug!("Using default NPC base prompt");
        Ok(NPC_BASE_DEFAULT.to_string())
    }

    /// Load the GM base prompt, using default if file doesn't exist
    pub fn load_gm_base(&self) -> Result<String> {
        // Try multiple possible locations
        let possible_paths = vec![
            self.prompts_dir.join("gm").join("gm_base.md"),
            self.prompts_dir.join("gm_base.md"),
        ];

        for path in possible_paths {
            if path.exists() {
                log::debug!("Loading GM base prompt from: {:?}", path);
                return fs::read_to_string(path)
                    .map_err(|e| anyhow::anyhow!("Failed to read GM base prompt: {}", e));
            }
        }

        log::debug!("Using default GM base prompt");
        Ok(GM_BASE_DEFAULT.to_string())
    }

    /// Load a custom prompt template
    pub fn load_custom(&self, name: &str) -> Result<String> {
        let path = self.prompts_dir.join(format!("{}.md", name));
        fs::read_to_string(&path)
            .map_err(|e| anyhow::anyhow!("Failed to read prompt '{}': {}", name, e))
    }
}
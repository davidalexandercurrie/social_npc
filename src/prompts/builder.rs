use anyhow::Result;
use std::path::Path;
use std::fs;
use serde_json;

use crate::types::{GameState, Npc};
use crate::memory::MemorySystem;
use super::loader::PromptLoader;

/// Builds prompts for NPCs and the GM
pub struct PromptBuilder {
    loader: PromptLoader,
    data_path: std::path::PathBuf,
}

impl PromptBuilder {
    pub fn new(data_path: impl AsRef<Path>) -> Self {
        let data_path = data_path.as_ref().to_path_buf();
        let loader = PromptLoader::new(&data_path);
        Self { loader, data_path }
    }

    /// Build a prompt for an NPC to decide their next action
    pub fn build_npc_intent_prompt(
        &self,
        npc: &Npc,
        game_state: &GameState,
    ) -> Result<String> {
        let mut sections = vec![];

        // 1. Base NPC instructions (response format, etc.)
        sections.push(self.loader.load_npc_base()?);
        
        // 2. Personality
        if let Ok(personality) = self.load_personality(&npc.name) {
            sections.push(personality);
        }
        
        // 3. Current memories
        if let Ok(memories) = self.load_memories(&npc.name) {
            sections.push(format!("## Your Current Memories\n\n```json\n{}\n```", 
                serde_json::to_string_pretty(&memories)?));
        }
        
        // 4. Current state
        sections.push(self.format_current_state(npc, game_state));
        
        // 5. Contract context if in one
        if let Some(contract_id) = &npc.active_contract {
            if let Ok(transcript) = self.read_contract_transcript(contract_id) {
                sections.push(format!("## Current Interaction\n\n{}", transcript));
            }
        }
        
        // 6. GM's specific prompt or generic "What do you do next?"
        let prompt = npc.next_prompt.as_ref()
            .map(|p| p.clone())
            .unwrap_or_else(|| "What do you do next?".to_string());
        sections.push(prompt);

        Ok(sections.join("\n\n---\n\n"))
    }

    /// Build a prompt for the GM to resolve intents
    pub fn build_gm_prompt(&self, input_json: &str) -> Result<String> {
        let mut sections = vec![];
        
        // GM base instructions
        sections.push(self.loader.load_gm_base()?);
        
        // Current game state and intents
        sections.push(format!("## Current Input\n\n```json\n{}\n```", input_json));
        
        Ok(sections.join("\n\n---\n\n"))
    }

    /// Build a prompt for updating an NPC's memories
    pub fn build_memory_update_prompt(
        &self,
        npc_name: &str,
        intent_json: &str,
        reality: &str,
        other_npcs: &[String],
    ) -> Result<String> {
        let mut sections = vec![];
        
        // Load memory update instructions
        let memory_prompt = self.loader.load_custom("memory_update")
            .unwrap_or_else(|_| MEMORY_UPDATE_DEFAULT.to_string());
        sections.push(memory_prompt);
        
        // Load current memories
        if let Ok(memories) = self.load_memories(npc_name) {
            sections.push(format!("## Current Memories\n\n```json\n{}\n```", 
                serde_json::to_string_pretty(&memories)?));
        }
        
        // Add context
        sections.push(format!("## Your Intent\n\n```json\n{}\n```", intent_json));
        sections.push(format!("## What Actually Happened\n\n{}", reality));
        
        if !other_npcs.is_empty() {
            sections.push(format!("## NPCs Present\n\n{}", other_npcs.join(", ")));
        }
        
        Ok(sections.join("\n\n---\n\n"))
    }

    fn format_current_state(&self, npc: &Npc, game_state: &GameState) -> String {
        let mut state = String::from("## Current Situation\n\n");
        
        // NPC's own state
        state.push_str(&format!("- You are at: {}\n", npc.location));
        state.push_str(&format!("- You are: {}\n", npc.activity));
        
        // Others at same location
        let others_here: Vec<_> = game_state.npcs
            .iter()
            .filter(|(name, other_npc)| {
                name.as_str() != npc.name.as_str() && other_npc.location == npc.location
            })
            .collect();
            
        if !others_here.is_empty() {
            state.push_str("\nAlso here:\n");
            for (name, other_npc) in others_here {
                state.push_str(&format!("- {} is {}\n", name, other_npc.activity));
            }
        }
        
        // Active contracts
        let active_contracts = game_state.contracts
            .values()
            .filter(|c| c.participants.contains(&npc.name))
            .count();
            
        if active_contracts > 0 {
            state.push_str(&format!("\nYou are currently engaged in {} interaction(s)\n", active_contracts));
        }
        
        state
    }

    fn load_personality(&self, npc_name: &str) -> Result<String> {
        let path = self.data_path
            .join("npcs")
            .join(npc_name)
            .join("personality.md");
        fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Failed to load personality: {}", e))
    }

    fn load_memories(&self, npc_name: &str) -> Result<MemorySystem> {
        let path = self.data_path
            .join("npcs")
            .join(npc_name)
            .join("memories.json");
        
        if !path.exists() {
            // Try initial_memories.json as fallback
            let initial_path = self.data_path
                .join("npcs")
                .join(npc_name)
                .join("initial_memories.json");
            
            if initial_path.exists() {
                let content = fs::read_to_string(initial_path)?;
                return serde_json::from_str(&content)
                    .map_err(|e| anyhow::anyhow!("Failed to parse memories: {}", e));
            }
            
            // Return empty memory system if no files exist
            return Ok(MemorySystem::new());
        }
        
        let content = fs::read_to_string(path)?;
        serde_json::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Failed to parse memories: {}", e))
    }

    fn read_contract_transcript(&self, contract_id: &str) -> Result<String> {
        let path = self.data_path
            .join("contracts")
            .join(format!("{}.json", contract_id));
        fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Failed to read contract: {}", e))
    }
}

// Default memory update prompt if not provided
const MEMORY_UPDATE_DEFAULT: &str = r#"# Memory Update

You are updating your memories based on what just happened.

## Response Format

Respond with JSON in this format:

```json
{
  "immediate_self_context": "Your updated understanding of your current situation",
  "new_self_memory": "A significant memory to add (or null if nothing significant)",
  "relationship_updates": {
    "other_npc_name": {
      "immediate_context": "Your current feeling about this NPC",
      "new_memory": {
        "event": "What happened with them",
        "timestamp": "2024-01-01T00:00:00Z",
        "emotional_impact": "how it made you feel",
        "importance": 0.5
      },
      "current_sentiment": 0.5,
      "long_term_summary_update": "Updated understanding of your relationship (or null)",
      "potential_core_memory": "Something fundamental about them (or null)"
    }
  }
}
```

Consider:
- How did reality differ from your intent?
- What did you learn about yourself or others?
- How do you feel about what happened?
"#;
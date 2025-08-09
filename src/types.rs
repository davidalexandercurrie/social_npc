use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a Non-Player Character with location and activity state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Npc {
    pub name: String,
    pub location: String,
    pub activity: String,
    pub folder_path: String,
    pub active_contract: Option<String>,
    pub next_prompt: Option<String>,
}

impl Npc {
    /// Creates a new NPC with the given name, location, and activity
    pub fn new(name: impl Into<String>, location: impl Into<String>, activity: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            folder_path: format!("data/npcs/{}", name.to_lowercase()),
            name,
            location: location.into(),
            activity: activity.into(),
            active_contract: None,
            next_prompt: None,
        }
    }
}

/// Represents an NPC's intended action with internal thoughts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intent {
    pub npc: String,
    pub thought: String,
    pub action: String,
    pub dialogue: Option<String>,
}

/// Represents an action taken by an NPC in the game world
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcAction {
    pub action: String,
    pub dialogue: Option<String>,
}

/// A contract between NPCs for extended interactions
#[derive(Debug, Clone, Serialize)]
pub struct Contract {
    pub id: String,
    pub participants: Vec<String>,
    pub transcript_file: String,
}

/// The current state of the game world
#[derive(Debug, Clone, Serialize)]
pub struct GameState {
    pub npcs: HashMap<String, Npc>,
    pub contracts: HashMap<String, Contract>,
}

/// Data sent to the GM for resolution
#[derive(Debug, Serialize)]
pub struct GmInput {
    pub current_state: CurrentState,
    pub intents: Vec<Intent>,
}

#[derive(Debug, Serialize)]
pub struct CurrentState {
    pub npcs: HashMap<String, Npc>,
    pub active_contracts: HashMap<String, Contract>,
}

/// Response from the GM about what actually happened
#[derive(Debug, Serialize, Deserialize)]
pub struct GmResponse {
    pub reality: String,
    pub state_changes: Vec<StateChange>,
    pub contracts: Vec<ContractUpdate>,
    pub next_prompts: HashMap<String, String>,
}

/// A change to an NPC's state
#[derive(Debug, Serialize, Deserialize)]
pub struct StateChange {
    pub npc: String,
    pub location: String,
    pub activity: String,
}

/// Updates to contracts
#[derive(Debug, Serialize, Deserialize)]
pub struct ContractUpdate {
    pub id: String,
    pub participants: Vec<String>,
    pub action: String,  // "create", "update", "end"
    pub transcript_entry: Option<TranscriptEntry>,
}

/// An entry in a contract's transcript
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptEntry {
    pub reality: String,
    pub details: HashMap<String, NpcAction>,
}

/// Input for updating an NPC's memories
#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryUpdateInput {
    pub npc_name: String,
    pub intent: Intent,
    pub reality: String,
    pub other_npcs_present: Vec<String>,
}
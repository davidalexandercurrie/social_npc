use anyhow::Result;
use crate::types::{Intent, Npc};
use crate::memory::{MemorySystem, MemoryUpdate};

/// Trait for implementing NPC behavior and decision-making
pub trait NpcBehavior: Send + Sync {
    /// Determines what action this NPC wants to take given the current context
    async fn decide_action(&self, npc: &Npc, context: &dyn Context) -> Result<Intent>;
    
    /// Processes the outcome of an action and updates internal state
    async fn process_outcome(&mut self, intent: &Intent, outcome: &str) -> Result<()>;
}

/// Provides context about the game world for NPCs to make decisions
pub trait Context: Send + Sync {
    /// Gets all NPCs visible to the current NPC
    fn get_visible_npcs(&self) -> Vec<&Npc>;
    
    /// Gets the current environment description
    fn get_environment(&self) -> &str;
    
    /// Gets the current location of an NPC
    fn get_npc_location(&self, npc_name: &str) -> Option<String>;
    
    /// Gets active contracts or interactions involving this NPC
    fn get_active_interactions(&self, npc_name: &str) -> Vec<String>;
}

/// Trait for managing NPC memories
pub trait MemoryManager: Send + Sync {
    /// Updates memories based on new events
    async fn update_memories(&mut self, update: MemoryUpdate) -> Result<()>;
    
    /// Retrieves the current memory system
    fn get_memories(&self) -> &MemorySystem;
    
    /// Processes memory decay over time
    async fn process_memory_decay(&mut self) -> Result<()>;
    
    /// Consolidates short-term memories into long-term
    async fn consolidate_memories(&mut self) -> Result<()>;
}

/// Trait for loading and saving NPC data
pub trait NpcStorage: Send + Sync {
    /// Loads an NPC from storage
    async fn load_npc(&self, name: &str) -> Result<Npc>;
    
    /// Saves an NPC to storage
    async fn save_npc(&self, npc: &Npc) -> Result<()>;
    
    /// Loads memory system for an NPC
    async fn load_memories(&self, npc_name: &str) -> Result<MemorySystem>;
    
    /// Saves memory system for an NPC
    async fn save_memories(&self, npc_name: &str, memories: &MemorySystem) -> Result<()>;
    
    /// Lists all available NPCs
    async fn list_npcs(&self) -> Result<Vec<String>>;
}

/// Trait for NPC perception and awareness
pub trait Perception: Send + Sync {
    /// Determines what this NPC can perceive in their environment
    fn perceive(&self, npc: &Npc, context: &dyn Context) -> PerceptionResult;
}

/// Result of perception check
pub struct PerceptionResult {
    pub visible_npcs: Vec<String>,
    pub audible_events: Vec<String>,
    pub environmental_details: Vec<String>,
}

impl PerceptionResult {
    pub fn new() -> Self {
        Self {
            visible_npcs: Vec::new(),
            audible_events: Vec::new(),
            environmental_details: Vec::new(),
        }
    }
}

/// Trait for social interactions between NPCs
pub trait SocialInteraction: Send + Sync {
    /// Initiates a social interaction with another NPC
    async fn initiate_interaction(&self, initiator: &Npc, target: &Npc, interaction_type: &str) -> Result<InteractionResult>;
    
    /// Responds to a social interaction from another NPC
    async fn respond_to_interaction(&self, responder: &Npc, initiator: &Npc, interaction: &str) -> Result<InteractionResult>;
}

/// Result of a social interaction
pub struct InteractionResult {
    pub success: bool,
    pub description: String,
    pub sentiment_change: f32,
    pub relationship_impact: f32,
}

impl InteractionResult {
    pub fn success(description: impl Into<String>) -> Self {
        Self {
            success: true,
            description: description.into(),
            sentiment_change: 0.0,
            relationship_impact: 0.0,
        }
    }
    
    pub fn failure(description: impl Into<String>) -> Self {
        Self {
            success: false,
            description: description.into(),
            sentiment_change: 0.0,
            relationship_impact: 0.0,
        }
    }
    
    pub fn with_sentiment(mut self, change: f32) -> Self {
        self.sentiment_change = change;
        self
    }
    
    pub fn with_relationship_impact(mut self, impact: f32) -> Self {
        self.relationship_impact = impact;
        self
    }
}
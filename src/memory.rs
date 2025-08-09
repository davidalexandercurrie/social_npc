use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Complete memory system for an NPC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySystem {
    pub self_memories: SelfMemories,
    pub relationships: HashMap<String, RelationshipMemory>,
}

impl MemorySystem {
    /// Creates a new empty memory system
    pub fn new() -> Self {
        Self {
            self_memories: SelfMemories::new(),
            relationships: HashMap::new(),
        }
    }

    /// Creates a memory system with initial context
    pub fn with_context(immediate_context: impl Into<String>) -> Self {
        Self {
            self_memories: SelfMemories::with_context(immediate_context),
            relationships: HashMap::new(),
        }
    }

    /// Gets or creates a relationship memory for the given NPC
    pub fn get_or_create_relationship(&mut self, npc_name: impl Into<String>) -> &mut RelationshipMemory {
        self.relationships
            .entry(npc_name.into())
            .or_insert_with(RelationshipMemory::new)
    }

    /// Updates immediate context for self
    pub fn update_self_context(&mut self, context: impl Into<String>) {
        self.self_memories.immediate_context = context.into();
    }

    /// Adds a recent event to self memories
    pub fn add_self_event(&mut self, event: impl Into<String>) {
        self.self_memories.add_recent_event(event);
    }
}

impl Default for MemorySystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Memories about the NPC's own state and experiences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfMemories {
    pub immediate_context: String,
    pub recent_events: Vec<String>,
    pub core_memories: Vec<String>,
}

impl SelfMemories {
    pub fn new() -> Self {
        Self {
            immediate_context: String::new(),
            recent_events: Vec::new(),
            core_memories: Vec::new(),
        }
    }

    pub fn with_context(immediate_context: impl Into<String>) -> Self {
        Self {
            immediate_context: immediate_context.into(),
            recent_events: Vec::new(),
            core_memories: Vec::new(),
        }
    }

    pub fn add_recent_event(&mut self, event: impl Into<String>) {
        self.recent_events.push(event.into());
        // Keep only the last 10 recent events
        if self.recent_events.len() > 10 {
            self.recent_events.remove(0);
        }
    }

    pub fn add_core_memory(&mut self, memory: impl Into<String>) {
        self.core_memories.push(memory.into());
    }
}

impl Default for SelfMemories {
    fn default() -> Self {
        Self::new()
    }
}

/// Memories about relationships with other NPCs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipMemory {
    pub immediate_context: String,
    pub recent_memories: Vec<Memory>,
    pub long_term_summary: String,
    pub core_memories: Vec<String>,
    pub current_sentiment: f32,  // -1.0 to 1.0
    pub overall_bond: f32,       // -1.0 to 1.0
}

impl RelationshipMemory {
    pub fn new() -> Self {
        Self {
            immediate_context: String::new(),
            recent_memories: Vec::new(),
            long_term_summary: String::new(),
            core_memories: Vec::new(),
            current_sentiment: 0.0,
            overall_bond: 0.0,
        }
    }

    pub fn add_memory(&mut self, memory: Memory) {
        self.recent_memories.push(memory);
        // Keep only the last 5 memories per relationship
        if self.recent_memories.len() > 5 {
            self.recent_memories.remove(0);
        }
    }

    pub fn update_sentiment(&mut self, sentiment: f32) {
        self.current_sentiment = sentiment.clamp(-1.0, 1.0);
    }

    pub fn update_bond(&mut self, bond: f32) {
        self.overall_bond = bond.clamp(-1.0, 1.0);
    }
}

impl Default for RelationshipMemory {
    fn default() -> Self {
        Self::new()
    }
}

/// A single memory event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub event: String,
    pub timestamp: DateTime<Utc>,
    pub emotional_impact: String,
    pub importance: f32,  // 0.0 to 1.0
}

impl Memory {
    pub fn new(event: impl Into<String>, emotional_impact: impl Into<String>, importance: f32) -> Self {
        Self {
            event: event.into(),
            timestamp: Utc::now(),
            emotional_impact: emotional_impact.into(),
            importance: importance.clamp(0.0, 1.0),
        }
    }

    pub fn with_timestamp(
        event: impl Into<String>,
        emotional_impact: impl Into<String>,
        importance: f32,
        timestamp: DateTime<Utc>,
    ) -> Self {
        Self {
            event: event.into(),
            timestamp,
            emotional_impact: emotional_impact.into(),
            importance: importance.clamp(0.0, 1.0),
        }
    }
}

// Input from LLM when updating memories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryUpdate {
    pub immediate_self_context: String,
    pub new_self_memory: Option<String>,
    pub relationship_updates: HashMap<String, RelationshipUpdate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipUpdate {
    pub immediate_context: String,
    pub new_memory: Option<Memory>,
    pub current_sentiment: f32,
    pub long_term_summary_update: Option<String>,
    pub potential_core_memory: Option<String>,
}

// Used when a memory needs to fade
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FadeDecision {
    pub memory_to_fade: Memory,
    pub impacts_long_term: bool,
    pub new_long_term_summary: Option<String>,
    pub forms_core_memory: bool,
}
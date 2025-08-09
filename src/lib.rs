//! # Social NPC Library
//!
//! A library for managing NPC social behaviors and memory systems in games.
//!
//! ## Features
//!
//! - **Memory System**: Hierarchical memory management with immediate context, short-term, and long-term memories
//! - **Relationship Tracking**: Track relationships between NPCs with sentiment and bond strength
//! - **Intent System**: NPCs can form intentions and take actions
//! - **Extensible Traits**: Implement custom behavior, storage, and perception systems
//!
//! ## Example
//!
//! ```rust
//! use social_npc::{Npc, MemorySystem, Intent};
//!
//! // Create an NPC
//! let npc = Npc::builder("Alice")
//!     .location("tavern")
//!     .activity("drinking")
//!     .build();
//!
//! // Create a memory system
//! let mut memories = MemorySystem::with_context("Just arrived at the tavern");
//!
//! // Add a recent event
//! memories.add_self_event("Ordered a drink from the bartender");
//!
//! // Create an intent
//! let intent = Intent::with_target(
//!     "Alice",
//!     "talk",
//!     "Bob",
//!     "Want to catch up with an old friend"
//! );
//! ```

pub mod memory;
pub mod traits;
pub mod types;

// Re-export main types for convenience
pub use memory::{
    FadeDecision, Memory, MemorySystem, MemoryUpdate, RelationshipMemory,
    RelationshipUpdate, SelfMemories,
};
pub use traits::{
    Context, InteractionResult, MemoryManager, NpcBehavior, NpcStorage, Perception,
    PerceptionResult, SocialInteraction,
};
pub use types::{Intent, Npc, NpcAction, NpcBuilder};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_npc_creation() {
        let npc = Npc::new("Test", "location", "idle");
        assert_eq!(npc.name, "Test");
        assert_eq!(npc.location, "location");
        assert_eq!(npc.activity, "idle");
    }

    #[test]
    fn test_npc_builder() {
        let npc = Npc::builder("Alice")
            .location("tavern")
            .activity("drinking")
            .build();
        
        assert_eq!(npc.name, "Alice");
        assert_eq!(npc.location, "tavern");
        assert_eq!(npc.activity, "drinking");
    }

    #[test]
    fn test_memory_system() {
        let mut memories = MemorySystem::new();
        memories.update_self_context("Testing");
        assert_eq!(memories.self_memories.immediate_context, "Testing");
        
        memories.add_self_event("Event 1");
        memories.add_self_event("Event 2");
        assert_eq!(memories.self_memories.recent_events.len(), 2);
    }

    #[test]
    fn test_intent_creation() {
        let intent = Intent::with_target("Alice", "talk", "Bob", "Greeting");
        assert_eq!(intent.npc, "Alice");
        assert_eq!(intent.action, "talk");
        assert_eq!(intent.target, Some("Bob".to_string()));
        assert_eq!(intent.reason, "Greeting");
    }
}
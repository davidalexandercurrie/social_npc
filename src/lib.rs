//! # Social NPC Library
//!
//! A complete NPC engine for managing social behaviors, memory systems, and turn-based gameplay.
//!
//! ## Features
//!
//! - **NPC Engine**: Complete turn-based NPC management system
//! - **Memory System**: Hierarchical memory management with immediate context, short-term, and long-term memories
//! - **Intent System**: NPCs form intentions based on their state and memories
//! - **GM Resolution**: Game Master system resolves intents into reality
//! - **Contract System**: Multi-turn interactions between NPCs
//! - **LLM Integration**: Built-in Ollama support for AI-driven behaviors
//!
//! ## Example
//!
//! ```rust,no_run
//! use social_npc::{NpcEngine, llm::OllamaClient};
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Create the engine with Ollama
//! let llm = OllamaClient::new("llama3.2:latest");
//! let mut engine = NpcEngine::new("./data", llm)?;
//!
//! // Execute a complete turn
//! let result = engine.execute_turn().await?;
//!
//! // Or run individual phases
//! let intents = engine.collect_intents().await?;
//! let reality = engine.resolve_intents(intents).await?;
//! # Ok(())
//! # }
//! ```

pub mod engine;
pub mod llm;
pub mod memory;
pub mod parser;
pub mod traits;
pub mod types;

// Re-export main types for convenience
pub use engine::NpcEngine;
pub use memory::{
    FadeDecision, Memory, MemorySystem, MemoryUpdate, RelationshipMemory,
    RelationshipUpdate, SelfMemories,
};
pub use traits::{
    Context, InteractionResult, MemoryManager, NpcBehavior, NpcStorage, Perception,
    PerceptionResult, SocialInteraction,
};
pub use types::{
    Contract, CurrentState, GameState, GmInput, GmResponse, Intent, MemoryUpdateInput,
    Npc, NpcAction, StateChange, TranscriptEntry,
};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::llm::LlmClient;
use crate::types::{Contract, GameState, GmResponse, Intent, Npc};

/// The main NPC engine that manages game state and orchestrates NPC behaviors
pub struct NpcEngine {
    /// Path to the data directory containing NPCs and prompts
    data_path: PathBuf,
    
    /// The LLM client for generating NPC behaviors
    llm_client: Arc<dyn LlmClient>,
    
    /// Current game state
    state: Arc<Mutex<GameState>>,
}

impl NpcEngine {
    /// Create a new NPC engine with the given data directory and LLM client
    pub fn new(data_path: impl AsRef<Path>, llm_client: impl LlmClient + 'static) -> Result<Self> {
        let data_path = data_path.as_ref().to_path_buf();
        
        // TODO: Load NPCs from data directory
        let npcs = HashMap::new();
        let contracts = HashMap::new();
        
        Ok(Self {
            data_path,
            llm_client: Arc::new(llm_client),
            state: Arc::new(Mutex::new(GameState { npcs, contracts })),
        })
    }
    
    /// Get the current game state
    pub fn get_state(&self) -> GameState {
        self.state.lock().unwrap().clone()
    }
    
    /// Update the game state
    pub fn update_state<F>(&self, updater: F) -> Result<()> 
    where
        F: FnOnce(&mut GameState) -> Result<()>
    {
        let mut state = self.state.lock().unwrap();
        updater(&mut state)
    }
    
    /// Collect intents from all NPCs
    pub async fn collect_intents(&self) -> Result<Vec<Intent>> {
        // TODO: Implement intent collection
        Ok(Vec::new())
    }
    
    /// Have the GM resolve intents into reality
    pub async fn resolve_intents(&self, intents: Vec<Intent>) -> Result<GmResponse> {
        // TODO: Implement GM resolution
        Ok(GmResponse {
            reality: String::new(),
            state_changes: Vec::new(),
            contracts: Vec::new(),
            next_prompts: HashMap::new(),
        })
    }
    
    /// Update NPC memories based on what happened
    pub async fn update_memories(&self, intents: &[Intent], reality: &GmResponse) -> Result<()> {
        // TODO: Implement memory updates
        Ok(())
    }
    
    /// Execute a complete turn (collect, resolve, update)
    pub async fn execute_turn(&self) -> Result<GmResponse> {
        log::info!("Starting turn execution");
        
        // Collect intents
        let intents = self.collect_intents().await?;
        log::info!("Collected {} intents", intents.len());
        
        // Resolve with GM
        let reality = self.resolve_intents(intents.clone()).await?;
        log::info!("GM resolved reality");
        
        // Update memories
        self.update_memories(&intents, &reality).await?;
        log::info!("Updated NPC memories");
        
        Ok(reality)
    }
    
    /// Initialize a new NPC with template files
    pub fn init_npc(&self, name: &str) -> Result<()> {
        // TODO: Create NPC directory and template files
        Ok(())
    }
    
    /// Load NPCs from the data directory
    pub fn load_npcs(&mut self) -> Result<()> {
        // TODO: Load NPCs from filesystem
        Ok(())
    }
}
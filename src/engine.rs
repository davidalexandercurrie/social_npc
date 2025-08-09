use anyhow::Result;
use futures::future::join_all;
use serde_json;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::llm::LlmClient;
use crate::parser;
use crate::prompts::PromptBuilder;
use crate::types::{Contract, GameState, GmInput, GmResponse, Intent, Npc, CurrentState, MemoryUpdateInput};
use crate::memory::MemoryUpdate;

/// The main NPC engine that manages game state and orchestrates NPC behaviors
pub struct NpcEngine {
    /// Path to the data directory containing NPCs and prompts
    data_path: PathBuf,
    
    /// The LLM client for generating NPC behaviors
    llm_client: Arc<dyn LlmClient>,
    
    /// Current game state
    state: Arc<Mutex<GameState>>,
    
    /// Prompt builder for constructing prompts
    prompt_builder: PromptBuilder,
}

impl NpcEngine {
    /// Create a new NPC engine with the given data directory and LLM client
    pub fn new(data_path: impl AsRef<Path>, llm_client: impl LlmClient + 'static) -> Result<Self> {
        let data_path = data_path.as_ref().to_path_buf();
        let prompt_builder = PromptBuilder::new(&data_path);
        
        // Start with empty state
        let npcs = HashMap::new();
        let contracts = HashMap::new();
        
        let mut engine = Self {
            data_path,
            llm_client: Arc::new(llm_client),
            state: Arc::new(Mutex::new(GameState { npcs, contracts })),
            prompt_builder,
        };
        
        // Load NPCs from data directory
        engine.load_npcs()?;
        
        Ok(engine)
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
        let npcs_to_process = self.get_state().npcs;
        
        if npcs_to_process.is_empty() {
            log::debug!("No NPCs to collect intents from");
            return Ok(Vec::new());
        }
        
        let total_npcs = npcs_to_process.len();
        log::debug!("Collecting intents from {} NPCs in parallel", total_npcs);
        
        let game_state = self.get_state();
        
        // Create futures for all NPCs
        let intent_futures: Vec<_> = npcs_to_process
            .iter()
            .map(|(name, npc)| {
                let name = name.clone();
                let npc = npc.clone();
                let game_state = game_state.clone();
                let llm_client = Arc::clone(&self.llm_client);
                let prompt_builder = &self.prompt_builder;
                
                async move {
                    Self::collect_single_intent(
                        name,
                        npc,
                        game_state,
                        llm_client,
                        prompt_builder,
                    ).await
                }
            })
            .collect();
        
        // Wait for all intents to be collected in parallel
        let results = join_all(intent_futures).await;
        
        // Filter out None values and collect successful intents
        let intents: Vec<Intent> = results.into_iter().flatten().collect();
        
        log::info!("Collected {} intents from {} NPCs", intents.len(), total_npcs);
        
        Ok(intents)
    }
    
    async fn collect_single_intent(
        name: String,
        npc: Npc,
        game_state: GameState,
        llm_client: Arc<dyn LlmClient>,
        prompt_builder: &PromptBuilder,
    ) -> Option<Intent> {
        log::debug!("Getting intent from {}", name);
        
        // Build prompt
        let prompt = match prompt_builder.build_npc_intent_prompt(&npc, &game_state) {
            Ok(p) => p,
            Err(e) => {
                log::error!("Failed to build prompt for {}: {}", name, e);
                return None;
            }
        };
        
        // Query LLM
        log::info!("🎭 Collecting intent from {}", name);
        let response = match llm_client.query(prompt, Path::new(".")).await {
            Ok(resp) => resp,
            Err(e) => {
                log::error!("Failed to get response from LLM for {}: {}", name, e);
                return None;
            }
        };
        
        // Parse response
        match parser::extract_json::<Intent>(&response) {
            Ok(intent) => {
                log::info!("  💭 {}: {}", name, intent.action);
                Some(intent)
            }
            Err(e) => {
                log::error!("Failed to parse intent from {}: {}", name, e);
                None
            }
        }
    }
    
    /// Have the GM resolve intents into reality
    pub async fn resolve_intents(&self, intents: Vec<Intent>) -> Result<GmResponse> {
        if intents.is_empty() {
            log::debug!("No intents to resolve");
            return Ok(GmResponse {
                reality: "Nothing happened.".to_string(),
                state_changes: Vec::new(),
                contracts: Vec::new(),
                next_prompts: HashMap::new(),
            });
        }
        
        log::info!("🎲 Resolving {} intents with GM", intents.len());
        
        // Get current game state
        let game_state = self.get_state();
        
        // Prepare input for GM
        let gm_input = GmInput {
            current_state: CurrentState {
                npcs: game_state.npcs.clone(),
                active_contracts: game_state.contracts.clone(),
            },
            intents,
        };
        
        let input_json = serde_json::to_string_pretty(&gm_input)?;
        log::debug!("Sending to GM: {}", input_json);
        
        // Build GM prompt
        let prompt = self.prompt_builder.build_gm_prompt(&input_json)?;
        
        // Query LLM
        let response = self.llm_client
            .query(prompt, Path::new("."))
            .await?;
        
        // Parse response
        let gm_response: GmResponse = parser::extract_json(&response)?;
        log::info!("🎭 Reality: {}", gm_response.reality);
        
        // Apply state changes
        self.update_state(|state| {
            for change in &gm_response.state_changes {
                if let Some(npc) = state.npcs.get_mut(&change.npc) {
                    npc.location = change.location.clone();
                    npc.activity = change.activity.clone();
                    log::info!("  📍 {}: {} - {}", change.npc, change.location, change.activity);
                }
            }
            
            // Handle contract updates
            for contract_update in &gm_response.contracts {
                match contract_update.action.as_str() {
                    "create" => {
                        let contract = Contract {
                            id: contract_update.id.clone(),
                            participants: contract_update.participants.clone(),
                            transcript_file: format!("data/contracts/{}.json", contract_update.id),
                        };
                        
                        // Update NPCs' active_contract field
                        for participant in &contract_update.participants {
                            if let Some(npc) = state.npcs.get_mut(participant) {
                                npc.active_contract = Some(contract.id.clone());
                            }
                        }
                        
                        // Add to game state
                        state.contracts.insert(contract.id.clone(), contract);
                        log::info!("  📜 Contract created: {}", contract_update.id);
                    }
                    "update" => {
                        // Contract continues, append to transcript if needed
                        log::info!("  📜 Contract updated: {}", contract_update.id);
                    }
                    "end" => {
                        // Remove contract and clear NPCs' active_contract
                        if let Some(contract) = state.contracts.remove(&contract_update.id) {
                            for participant in &contract.participants {
                                if let Some(npc) = state.npcs.get_mut(participant) {
                                    npc.active_contract = None;
                                }
                            }
                        }
                        log::info!("  📜 Contract ended: {}", contract_update.id);
                    }
                    _ => log::warn!("Unknown contract action: {}", contract_update.action),
                }
            }
            
            // Update next prompts for NPCs
            for (npc_name, prompt) in &gm_response.next_prompts {
                if let Some(npc) = state.npcs.get_mut(npc_name) {
                    npc.next_prompt = Some(prompt.clone());
                }
            }
            
            Ok(())
        })?;
        
        Ok(gm_response)
    }
    
    /// Update NPC memories based on what happened
    pub async fn update_memories(&self, intents: &[Intent], reality: &GmResponse) -> Result<()> {
        if intents.is_empty() {
            log::debug!("No intents to process for memory updates");
            return Ok(());
        }
        
        log::info!("🧠 Updating memories for {} NPCs", intents.len());
        
        // Build memory update inputs for each NPC that acted
        let memory_inputs: Vec<MemoryUpdateInput> = intents
            .iter()
            .map(|intent| {
                // Find which other NPCs were present
                let other_npcs: Vec<String> = self.get_state()
                    .npcs
                    .iter()
                    .filter(|(name, other_npc)| {
                        if let Some(my_npc) = self.get_state().npcs.get(&intent.npc) {
                            name.as_str() != intent.npc.as_str() && 
                            other_npc.location == my_npc.location
                        } else {
                            false
                        }
                    })
                    .map(|(name, _)| name.clone())
                    .collect();
                
                MemoryUpdateInput {
                    npc_name: intent.npc.clone(),
                    intent: intent.clone(),
                    reality: reality.reality.clone(),
                    other_npcs_present: other_npcs,
                }
            })
            .collect();
        
        // Update each NPC's memories
        for input in memory_inputs {
            if let Err(e) = self.update_single_npc_memory(input).await {
                log::error!("Failed to update memory: {}", e);
            }
        }
        
        Ok(())
    }
    
    async fn update_single_npc_memory(&self, input: MemoryUpdateInput) -> Result<()> {
        let npc_name = &input.npc_name;
        log::debug!("Updating memories for {}", npc_name);
        
        // Load current memories
        let mut current_memories = self.load_npc_memories(npc_name)?;
        
        // Build memory update prompt
        let intent_json = serde_json::to_string(&input.intent)?;
        let prompt = self.prompt_builder.build_memory_update_prompt(
            npc_name,
            &intent_json,
            &input.reality,
            &input.other_npcs_present,
        )?;
        
        // Query LLM
        let response = self.llm_client
            .query(prompt, Path::new("."))
            .await?;
        
        // Parse memory update
        let memory_update: MemoryUpdate = parser::extract_json(&response)?;
        
        // Apply the update to the memory system
        current_memories.self_memories.immediate_context = memory_update.immediate_self_context.clone();
        
        if let Some(new_memory) = memory_update.new_self_memory {
            current_memories.self_memories.add_recent_event(new_memory);
        }
        
        // Update relationship memories
        for (other_npc, rel_update) in memory_update.relationship_updates {
            let relationship = current_memories.get_or_create_relationship(&other_npc);
            
            relationship.immediate_context = rel_update.immediate_context;
            relationship.current_sentiment = rel_update.current_sentiment;
            
            if let Some(new_memory) = rel_update.new_memory {
                relationship.add_memory(new_memory);
            }
            
            if let Some(summary) = rel_update.long_term_summary_update {
                relationship.long_term_summary = summary;
            }
            
            if let Some(core_memory) = rel_update.potential_core_memory {
                relationship.core_memories.push(core_memory);
            }
        }
        
        // Save updated memories
        self.save_npc_memories(npc_name, &current_memories)?;
        
        log::info!("  💭 {}: {}", npc_name, memory_update.immediate_self_context);
        
        Ok(())
    }
    
    /// Load memories for an NPC
    fn load_npc_memories(&self, npc_name: &str) -> Result<crate::memory::MemorySystem> {
        let memory_path = self.data_path.join("npcs").join(npc_name).join("memories.json");
        
        if memory_path.exists() {
            let content = std::fs::read_to_string(&memory_path)?;
            let memories = serde_json::from_str(&content)?;
            Ok(memories)
        } else {
            // This shouldn't happen if ensure_memories_exist was called, but handle it anyway
            Ok(crate::memory::MemorySystem::new())
        }
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
    
    /// Set the location and activity for an NPC
    pub fn set_npc_state(&self, npc_name: &str, location: impl Into<String>, activity: impl Into<String>) -> Result<()> {
        self.update_state(|state| {
            if let Some(npc) = state.npcs.get_mut(npc_name) {
                npc.location = location.into();
                npc.activity = activity.into();
                Ok(())
            } else {
                Err(anyhow::anyhow!("NPC '{}' not found", npc_name))
            }
        })
    }
    
    /// Initialize a new NPC with template files
    pub fn init_npc(&self, name: &str) -> Result<()> {
        // TODO: Create NPC directory and template files
        Ok(())
    }
    
    /// Load NPCs from the data directory
    pub fn load_npcs(&mut self) -> Result<()> {
        let npcs_dir = self.data_path.join("npcs");
        
        if !npcs_dir.exists() {
            log::warn!("NPCs directory does not exist: {:?}", npcs_dir);
            return Ok(());
        }
        
        let mut npcs = HashMap::new();
        
        // Read all directories in the npcs folder
        for entry in std::fs::read_dir(&npcs_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                let npc_name = path.file_name()
                    .and_then(|n| n.to_str())
                    .ok_or_else(|| anyhow::anyhow!("Invalid NPC directory name"))?;
                
                log::info!("Loading NPC: {}", npc_name);
                
                // Check if personality.md exists
                let personality_path = path.join("personality.md");
                if !personality_path.exists() {
                    log::warn!("No personality.md found for NPC: {}", npc_name);
                    continue;
                }
                
                // Create NPC with default values
                let npc = Npc {
                    name: npc_name.to_string(),
                    location: "start".to_string(), // Default location
                    activity: "idle".to_string(),   // Default activity
                    folder_path: path.to_string_lossy().to_string(),
                    active_contract: None,
                    next_prompt: None,
                };
                
                // Ensure memories exist (create from initial_memories.json if needed)
                self.ensure_memories_exist(npc_name)?;
                
                npcs.insert(npc_name.to_string(), npc);
            }
        }
        
        // Update the game state with loaded NPCs
        self.update_state(|state| {
            state.npcs = npcs;
            Ok(())
        })?;
        
        log::info!("Loaded {} NPCs", self.get_state().npcs.len());
        Ok(())
    }
    
    /// Ensure memories.json exists for an NPC, creating from initial_memories.json if needed
    fn ensure_memories_exist(&self, npc_name: &str) -> Result<()> {
        let npc_dir = self.data_path.join("npcs").join(npc_name);
        let memory_path = npc_dir.join("memories.json");
        
        if !memory_path.exists() {
            let initial_path = npc_dir.join("initial_memories.json");
            if initial_path.exists() {
                log::info!("Creating memories.json from initial_memories.json for {}", npc_name);
                let content = std::fs::read_to_string(&initial_path)?;
                
                // Validate it's valid JSON
                let memories: crate::memory::MemorySystem = serde_json::from_str(&content)?;
                
                // Save as memories.json
                let json = serde_json::to_string_pretty(&memories)?;
                std::fs::write(&memory_path, json)?;
            } else {
                log::info!("Creating empty memories.json for {}", npc_name);
                // Create empty memory system
                let memories = crate::memory::MemorySystem::new();
                let json = serde_json::to_string_pretty(&memories)?;
                std::fs::write(&memory_path, json)?;
            }
        }
        
        Ok(())
    }
    
    /// Save memories for an NPC
    fn save_npc_memories(&self, npc_name: &str, memories: &crate::memory::MemorySystem) -> Result<()> {
        let npc_dir = self.data_path.join("npcs").join(npc_name);
        
        // Create directory if it doesn't exist
        std::fs::create_dir_all(&npc_dir)?;
        
        let memory_path = npc_dir.join("memories.json");
        let json = serde_json::to_string_pretty(memories)?;
        std::fs::write(memory_path, json)?;
        
        Ok(())
    }
}
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
        
        // TODO: Load NPCs from data directory
        let npcs = HashMap::new();
        let contracts = HashMap::new();
        
        Ok(Self {
            data_path,
            llm_client: Arc::new(llm_client),
            state: Arc::new(Mutex::new(GameState { npcs, contracts })),
            prompt_builder,
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
        
        // Apply the update (for now just log it)
        // In a real implementation, this would save to filesystem
        log::info!("  💭 {}: {}", npc_name, memory_update.immediate_self_context);
        
        // TODO: Save updated memories to filesystem
        
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
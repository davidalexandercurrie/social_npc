use serde::{Deserialize, Serialize};

/// Represents a Non-Player Character with location and activity state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Npc {
    pub name: String,
    pub location: String,
    pub activity: String,
}

impl Npc {
    /// Creates a new NPC with the given name, location, and activity
    pub fn new(name: impl Into<String>, location: impl Into<String>, activity: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            location: location.into(),
            activity: activity.into(),
        }
    }

    /// Builder pattern for creating NPCs
    pub fn builder(name: impl Into<String>) -> NpcBuilder {
        NpcBuilder::new(name)
    }
}

/// Builder for creating NPCs with a fluent interface
pub struct NpcBuilder {
    name: String,
    location: Option<String>,
    activity: Option<String>,
}

impl NpcBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            location: None,
            activity: None,
        }
    }

    pub fn location(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }

    pub fn activity(mut self, activity: impl Into<String>) -> Self {
        self.activity = Some(activity.into());
        self
    }

    pub fn build(self) -> Npc {
        Npc {
            name: self.name,
            location: self.location.unwrap_or_else(|| "unknown".to_string()),
            activity: self.activity.unwrap_or_else(|| "idle".to_string()),
        }
    }
}

/// Represents an NPC's intended action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intent {
    pub npc: String,
    pub action: String,
    pub target: Option<String>,
    pub reason: String,
}

impl Intent {
    /// Creates a new intent
    pub fn new(
        npc: impl Into<String>,
        action: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            npc: npc.into(),
            action: action.into(),
            target: None,
            reason: reason.into(),
        }
    }

    /// Creates an intent with a target
    pub fn with_target(
        npc: impl Into<String>,
        action: impl Into<String>,
        target: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            npc: npc.into(),
            action: action.into(),
            target: Some(target.into()),
            reason: reason.into(),
        }
    }
}

/// Represents an action taken by an NPC in the game world
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcAction {
    pub action: String,
    pub result: String,
}

impl NpcAction {
    pub fn new(action: impl Into<String>, result: impl Into<String>) -> Self {
        Self {
            action: action.into(),
            result: result.into(),
        }
    }
}
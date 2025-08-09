# Social NPC

A Rust library for managing NPC (Non-Player Character) social behaviors and memory systems in games.

## Features

- **Hierarchical Memory System**: Manage immediate context, short-term memories, and long-term core memories
- **Relationship Tracking**: Track relationships between NPCs with sentiment analysis and bond strength
- **Intent System**: NPCs can form intentions and execute actions based on their memories and relationships
- **Extensible Architecture**: Trait-based design allows custom implementations of behavior, storage, and perception systems

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
social_npc = { git = "https://github.com/yourusername/social_npc.git" }
```

Or for local development:

```toml
[dependencies]
social_npc = { path = "../social_npc" }
```

## Quick Start

```rust
use social_npc::{Npc, MemorySystem, Intent, Memory};

fn main() {
    // Create an NPC
    let mut alice = Npc::builder("Alice")
        .location("marketplace")
        .activity("shopping")
        .build();

    // Initialize memory system
    let mut memories = MemorySystem::with_context("At the busy marketplace");
    
    // Add memories about self
    memories.add_self_event("Found a good deal on apples");
    
    // Track relationship with another NPC
    let bob_memory = memories.get_or_create_relationship("Bob");
    bob_memory.add_memory(Memory::new(
        "Bob helped me carry heavy bags",
        "grateful",
        0.8
    ));
    bob_memory.update_sentiment(0.7);  // Positive sentiment
    
    // Create an intent based on memories
    let intent = Intent::with_target(
        "Alice",
        "thank",
        "Bob",
        "Wants to thank Bob for his help"
    );
}
```

## Core Components

### NPC

Represents a game character with:

- Name
- Current location
- Current activity

### Memory System

Hierarchical memory management:

- **Immediate Context**: Current situation awareness
- **Recent Events**: Short-term memory buffer
- **Core Memories**: Long-term significant memories
- **Relationship Memories**: Per-NPC relationship tracking

### Intent System

NPCs can form intentions with:

- Action to perform
- Optional target
- Reasoning behind the action

### Extensible Traits

- `NpcBehavior`: Implement custom decision-making logic
- `MemoryManager`: Custom memory processing
- `NpcStorage`: Persistence layer for NPCs and memories
- `Perception`: Environmental awareness system
- `SocialInteraction`: Inter-NPC interaction handling

## Examples

See the `examples/` directory for more detailed usage examples.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.


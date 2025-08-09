use social_npc::{Intent, Memory, MemorySystem, Npc};

fn main() {
    println!("=== Social NPC Example ===\n");

    // Create NPCs
    let alice = Npc::builder("Alice")
        .location("tavern")
        .activity("drinking ale")
        .build();

    let bob = Npc::builder("Bob")
        .location("tavern")
        .activity("playing cards")
        .build();

    println!("NPCs created:");
    println!("- {} is {} at the {}", alice.name, alice.activity, alice.location);
    println!("- {} is {} at the {}\n", bob.name, bob.activity, bob.location);

    // Create memory system for Alice
    let mut alice_memories = MemorySystem::with_context("Relaxing at the tavern after a long day");

    // Add some self memories
    alice_memories.add_self_event("Ordered my favorite ale");
    alice_memories.add_self_event("The bard is playing a familiar tune");
    alice_memories.self_memories.add_core_memory("I always feel at home in this tavern");

    println!("Alice's memories:");
    println!("Context: {}", alice_memories.self_memories.immediate_context);
    println!("Recent events: {:?}", alice_memories.self_memories.recent_events);
    println!("Core memories: {:?}\n", alice_memories.self_memories.core_memories);

    // Create relationship memory with Bob
    let bob_relationship = alice_memories.get_or_create_relationship("Bob");
    
    // Add memories about Bob
    bob_relationship.add_memory(Memory::new(
        "Bob bought me a drink",
        "happy",
        0.6,
    ));
    
    bob_relationship.add_memory(Memory::new(
        "We laughed about old times",
        "nostalgic",
        0.7,
    ));

    bob_relationship.update_sentiment(0.8);  // Very positive
    bob_relationship.update_bond(0.6);       // Good friends
    bob_relationship.long_term_summary = "Bob is an old friend from my adventuring days".to_string();

    println!("Alice's relationship with Bob:");
    println!("Current sentiment: {:.1} (range: -1 to 1)", bob_relationship.current_sentiment);
    println!("Overall bond: {:.1} (range: -1 to 1)", bob_relationship.overall_bond);
    println!("Long-term summary: {}", bob_relationship.long_term_summary);
    println!("Recent memories about Bob:");
    for memory in &bob_relationship.recent_memories {
        println!("  - {}: {} (importance: {:.1})", 
            memory.event, 
            memory.emotional_impact,
            memory.importance
        );
    }
    println!();

    // Create intents based on memories
    let intent1 = Intent::with_target(
        &alice.name,
        "join_game",
        &bob.name,
        "Wants to join Bob's card game for old time's sake",
    );

    let intent2 = Intent::new(
        &bob.name,
        "order_round",
        "Feeling generous after winning at cards",
    );

    println!("Intents formed:");
    println!("- {}: {} {} because: {}", 
        intent1.npc, 
        intent1.action,
        intent1.target.as_ref().unwrap_or(&"".to_string()),
        intent1.reason
    );
    println!("- {}: {} because: {}", 
        intent2.npc, 
        intent2.action,
        intent2.reason
    );
}
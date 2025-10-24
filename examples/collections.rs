//! Collections example for the Grok Rust SDK

use grok_rust_sdk::{chat::Model, collections::CollectionManager, session::SessionManager, Client};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load API key from environment
    let api_key =
        std::env::var("XAI_API_KEY").expect("XAI_API_KEY environment variable must be set");

    // Create client and managers
    let client = Client::new(api_key)?;
    let session_mgr = client.session_manager();
    let collection_mgr = client.collection_manager(session_mgr.clone());

    println!("Collection Manager created");

    // Create collections
    let coding_collection = collection_mgr
        .create_collection(
            "Coding Discussions",
            Some("Conversations about programming and development"),
            vec![
                "coding".to_string(),
                "programming".to_string(),
                "tech".to_string(),
            ],
        )
        .await;

    let ai_collection = collection_mgr
        .create_collection(
            "AI Conversations",
            Some("Discussions about artificial intelligence"),
            vec![
                "ai".to_string(),
                "machine-learning".to_string(),
                "tech".to_string(),
            ],
        )
        .await;

    println!("Created collections:");
    println!(
        "- {}: {}",
        coding_collection.id, coding_collection.metadata.name
    );
    println!("- {}: {}", ai_collection.id, ai_collection.metadata.name);

    // Create sessions and add to collections
    let rust_session = session_mgr
        .create_session(
            Model::Grok4FastReasoning,
            Some("Rust Programming Tips".to_string()),
        )
        .await;

    let python_session = session_mgr
        .create_session(Model::Grok4, Some("Python Best Practices".to_string()))
        .await;

    let ai_ethics_session = session_mgr
        .create_session(
            Model::Grok4FastReasoning,
            Some("AI Ethics Discussion".to_string()),
        )
        .await;

    // Add some content to sessions
    let _ = rust_session
        .chat("What are some advanced Rust features?")
        .await?;
    let _ = rust_session
        .chat("Tell me about async programming in Rust.")
        .await?;

    let _ = python_session.chat("How does Python's GIL work?").await?;
    let _ = python_session
        .chat("What's the difference between list and tuple?")
        .await?;

    let _ = ai_ethics_session
        .chat("What are the main ethical concerns with AI?")
        .await?;
    let _ = ai_ethics_session
        .chat("How can we ensure AI safety?")
        .await?;

    // Add sessions to collections
    coding_collection.add_session(rust_session).await?;
    coding_collection.add_session(python_session).await?;
    ai_collection.add_session(ai_ethics_session).await?;

    println!("\nAdded sessions to collections");

    // List all collections
    println!("\n--- All Collections ---");
    let collections = collection_mgr.list_collections().await;
    for collection in &collections {
        println!("Collection: {}", collection.metadata.name);
        println!("  Description: {:?}", collection.metadata.description);
        println!("  Tags: {:?}", collection.metadata.tags);
        println!("  Sessions: {}", collection.metadata.session_count);
        println!("  Total messages: {}", collection.metadata.total_messages);
        println!("  Total tokens: {}", collection.metadata.total_tokens);
        println!();
    }

    // Search collections
    println!("--- Search Results ---");
    let coding_results = collection_mgr.search_collections("coding").await;
    println!("Collections matching 'coding': {}", coding_results.len());

    let tech_results = collection_mgr.collections_by_tag("tech").await;
    println!("Collections with 'tech' tag: {}", tech_results.len());

    // Get collection statistics
    let stats = collection_mgr.stats().await;
    println!("\n--- Global Collection Stats ---");
    println!("Total collections: {}", stats.total_collections);
    println!("Total sessions: {}", stats.total_sessions);
    println!("Total messages: {}", stats.total_messages);
    println!("Total tokens: {}", stats.total_tokens);

    // Demonstrate session retrieval from collections
    println!("\n--- Sessions in Coding Collection ---");
    let coding_sessions = coding_collection.list_sessions().await;
    for session in coding_sessions {
        println!(
            "- {}: {:?} ({} messages)",
            session.id, session.metadata.title, session.metadata.message_count
        );
    }

    Ok(())
}

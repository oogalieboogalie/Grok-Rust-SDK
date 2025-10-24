//! Session management example for the Grok Rust SDK

use grok_rust_sdk::{chat::Model, session::SessionManager, Client};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load API key from environment
    let api_key =
        std::env::var("XAI_API_KEY").expect("XAI_API_KEY environment variable must be set");

    // Create client and session manager
    let client = Client::new(api_key)?;
    let session_mgr = client.session_manager();

    println!("Session Manager created");

    // Create a new session
    let session = session_mgr
        .create_session(
            Model::Grok4FastReasoning,
            Some("Multi-turn Conversation Example".to_string()),
        )
        .await;

    println!("Created session: {}", session.id);
    println!("Session title: {:?}", session.metadata.title);

    // First interaction
    println!("\n--- First Interaction ---");
    let response1 = session.chat("Hello! My name is Alice.").await?;
    println!("Assistant: {}", response1.message.content);

    // Second interaction (context preserved)
    println!("\n--- Second Interaction ---");
    let response2 = session.chat("What's my name?").await?;
    println!("Assistant: {}", response2.message.content);

    // Third interaction
    println!("\n--- Third Interaction ---");
    let response3 = session
        .chat("Can you remind me what we were talking about?")
        .await?;
    println!("Assistant: {}", response3.message.content);

    // Check session stats
    println!("\n--- Session Statistics ---");
    let message_count = session.message_count().await;
    let messages = session.messages().await;

    println!("Total messages: {}", message_count);
    println!("Session created: {}", session.metadata.created_at);
    println!("Last updated: {}", session.metadata.updated_at);

    println!("\n--- Conversation History ---");
    for (i, message) in messages.iter().enumerate() {
        println!("{}. {:?}: {}", i + 1, message.role, message.content);
    }

    // Create another session
    let session2 = session_mgr
        .create_session(Model::Grok4, Some("Different Conversation".to_string()))
        .await;

    let _ = session2.chat("This is a different conversation.").await?;

    // List all sessions
    println!("\n--- All Sessions ---");
    let all_sessions = session_mgr.list_sessions().await;
    println!("Total sessions: {}", all_sessions.len());

    for session in &all_sessions {
        println!(
            "- {}: {:?} ({} messages)",
            session.id, session.metadata.title, session.metadata.message_count
        );
    }

    // Get session stats
    let stats = session_mgr.stats().await;
    println!("\nGlobal stats:");
    println!("- Total sessions: {}", stats.total_sessions);
    println!("- Total messages: {}", stats.total_messages);
    println!("- Total tokens: {}", stats.total_tokens);

    Ok(())
}

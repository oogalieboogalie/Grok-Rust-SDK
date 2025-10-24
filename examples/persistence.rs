//! Example demonstrating persistent storage of sessions and collections

use grok_rust_sdk::{Client, chat::Message, persistence::SqliteStorage};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client
    let client = Client::new("your-api-key-here")?;
    let client = std::sync::Arc::new(client);

    // Create SQLite storage (in-memory for this example)
    let storage = SqliteStorage::in_memory()?;

    // Create a session
    let session_mgr = client.session_manager();
    let session = session_mgr.create_session(grok_rust_sdk::Model::Grok4FastReasoning, Some("Persistent Chat".to_string())).await?;

    // Add some messages
    session.append(Message::user("Hello!")).await?;
    session.append(Message::assistant("Hi there! How can I help you today?".to_string())).await?;

    // Save the session
    storage.save_session(&session).await?;
    println!("Session saved with ID: {}", session.id());

    // Create a collection
    let collection_mgr = client.collection_manager(session_mgr);
    let collection = collection_mgr.create_collection("My Conversations", Some("A collection of my chats".to_string())).await?;

    // Add the session to the collection
    collection.add_session(&session).await?;
    storage.save_collection(&collection).await?;
    println!("Collection saved with ID: {}", collection.id());

    // Later, load the session back
    if let Some(loaded_session) = storage.load_session(&session.id()).await? {
        println!("Loaded session with {} messages", loaded_session.message_count().await);
    }

    // List all sessions and collections
    let session_ids = storage.list_sessions().await?;
    let collection_ids = storage.list_collections().await?;

    println!("Stored sessions: {:?}", session_ids);
    println!("Stored collections: {:?}", collection_ids);

    Ok(())
}
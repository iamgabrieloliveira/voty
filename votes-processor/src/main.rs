use queue_client::{QueueClient, Subscriber};
use serde::Deserialize;
use sqlx::{postgres::PgConnectOptions, PgPool};
use std::sync::Arc;

#[derive(Deserialize, Debug)]
struct Config {
    db_name: String,
    db_host: String,
    db_port: u16,
    db_user: String,
    db_password: String,
    queue_host: String,
    queue_port: u16,
}

#[derive(Deserialize, Debug)]
struct VotePayload {
    voter_id: String,
    votee_id: String,
}

async fn subscribe_in_queue(config: &Config) -> Result<Subscriber, String> {
    let queue_addr = format!("{}:{}", config.queue_host, config.queue_port);

    QueueClient::subscribe(&queue_addr)
        .await
        .map_err(|e| format!("Failed to connect to queue: {}", e))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let config = envy::from_env::<Config>()?;

    let db_config = PgConnectOptions::new()
        .host(&config.db_host)
        .port(config.db_port)
        .username(&config.db_user)
        .password(&config.db_password)
        .database(&config.db_name);

    let db = Arc::new(PgPool::connect_with(db_config).await?);

    let mut subscriber = subscribe_in_queue(&config).await?;

    log::info!("Starting to listen for votes");

    while let Ok(message) = subscriber.listen().await {
        let db = Arc::clone(&db);

        let payload = serde_json::from_slice::<VotePayload>(&message).unwrap();

        log::debug!("Received vote: {:?}", payload);

        let uuid = sqlx::types::Uuid::new_v4();

        sqlx::query("INSERT INTO votes (id, voter_id, votee_id) VALUES ($1, $2, $3)")
            .bind(&uuid)
            .bind(&payload.voter_id)
            .bind(&payload.votee_id)
            .execute(&*db)
            .await?;
    }

    Ok(())
}

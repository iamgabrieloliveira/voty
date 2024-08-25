use queue_client::Publisher;
use queue_client::QueueClient;
use std::sync::Arc;
use std::sync::Mutex;

use actix_web::{post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct Config {
    api_host: String,
    api_port: u16,
    queue_host: String,
    queue_port: u16,
}

struct AppState {
    queue: SharedPublisher,
}

#[derive(Deserialize, Serialize)]
struct VotePayload {
    voter_id: String,
    votee_id: String,
}

fn internal_server_error() -> HttpResponse {
    HttpResponse::InternalServerError().body("An error occurred")
}

fn bad_request(message: &'static str) -> HttpResponse {
    HttpResponse::BadRequest().body(message)
}

fn accepted(message: &'static str) -> HttpResponse {
    HttpResponse::Accepted().body(message)
}

#[post("/vote")]
async fn vote(app: web::Data<AppState>, payload: web::Json<VotePayload>) -> impl Responder {
    match app.queue.lock() {
        Err(err) => {
            log::error!("Failed to lock queue: {:?}", err);
            internal_server_error()
        }
        Ok(mut queue) => match serde_json::to_vec(&payload) {
            Err(err) => {
                log::error!("Failed to serialize vote payload {:?}", err);
                bad_request("Failed to serialize vote payload")
            }
            Ok(payload) => match queue.publish(&payload).await {
                Ok(_) => accepted("Vote received successfully"),
                Err(err) => {
                    log::error!("Failed to publish vote: {:?}", err);
                    internal_server_error()
                }
            },
        },
    }
}

type SharedPublisher = Arc<Mutex<Publisher>>;

async fn queue_connect(addr: &str) -> Result<SharedPublisher, std::io::Error> {
    let retries = 5;
    let mut attempt = 0;

    while attempt < retries {
        match QueueClient::connect(addr).await {
            Ok(queue) => return Ok(Arc::new(Mutex::new(queue))),
            Err(err) => {
                log::error!("Failed to connect to queue at {}, error: {}", addr, err);
                attempt += 1;
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    log::error!(
        "Failed to connect to queue at {}, max retries of {} exceeded",
        addr,
        retries
    );

    Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        "Failed to connect to queue",
    ))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let config = envy::from_env::<Config>().expect("Missing configuration");

    log::info!("Starting server at {}:{}", config.api_host, config.api_port);

    let queue_addr = format!("{}:{}", config.queue_host, config.queue_port);

    HttpServer::new(move || {
        let queue_addr = queue_addr.clone();

        App::new()
            .data_factory(move || {
                let queue_addr = queue_addr.clone();

                async move {
                    let queue = queue_connect(&queue_addr).await?;

                    Ok::<AppState, std::io::Error>(AppState { queue })
                }
            })
            .service(vote)
    })
    .bind((config.api_host, config.api_port))?
    .run()
    .await
}

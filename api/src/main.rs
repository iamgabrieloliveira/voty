use queue_client::Queue;
use std::sync::Arc;
use std::sync::Mutex;

use actix_web::{post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};

type SharedQueue = Arc<Mutex<Queue>>;

struct AppState {
    queue: SharedQueue,
}

fn create_app_state() -> AppState {
    let queue = create_queue("127.0.0.1:8080");

    AppState { queue }
}

fn create_queue(addr: &str) -> SharedQueue {
    let queue = Queue::new(addr).unwrap();
    return Arc::new(Mutex::new(queue));
}

#[derive(Deserialize, Serialize)]
struct VotePayload {
    voter_id: String,
    votee_id: String,
}

#[post("/vote")]
async fn vote(app: web::Data<AppState>, payload: web::Json<VotePayload>) -> impl Responder {
    let mut queue = app.queue.lock().unwrap();

    let payload = serde_json::to_string(&payload).unwrap();

    queue.publish(payload.as_bytes());

    HttpResponse::Ok().body("Vote received")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(move || {
        let app = create_app_state();
        let app = web::Data::new(app);

        App::new().app_data(app).service(vote)
    })
    .bind(("127.0.0.1", 9000))?
    .run()
    .await
}

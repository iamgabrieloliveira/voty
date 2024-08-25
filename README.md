# ⚡ Voty

## 📑 Description
**Voty** is a simple voting application that simulates a basic voting system. It's inspired by a [tweet from Zan](https://x.com/zanfranceschi/status/1501583683685425159). The application is designed with performance in mind, leveraging modern Rust frameworks and tools to deliver a robust and efficient system.

## 🚀 What I Delivered

### API
I developed an API using [Actix Web](https://actix.rs/), a powerful Rust web framework. The API features a single endpoint where users can submit their votes. To ensure optimal performance, the votes are sent to [**CrabMQ**](https://github.com/CrabMQ/crab-mq) for processing and storage.

### Vote Processor
I implemented a Vote Processor using [Tokio](https://tokio.rs/), a runtime for writing reliable, asynchronous applications in Rust. The processor retrieves votes from the queue and stores them in a PostgreSQL database.

## ✨ Features
- [x] 🛠 **API to Receive Votes**  
  An endpoint to submit votes.
- [ ] 🔐 **One Vote per User**  
  Ensures that each user can only vote once.
- [x] 📨 **Queue Service**  
  Integrates with **CrabMQ** to queue votes for processing.
- [x] ⚙️ **Connection Handler**  
  Manages connections to the queue.
- [x] 📨 **Message Handler**  
  Processes messages retrieved from the queue.
- [x] 🗃 **Vote Processor**  
  Subscribes to the queue and processes incoming votes.
- [x] 🗄 **Store Votes in Database**  
  Uses PostgreSQL for storing votes.
- [ ] 🧪 **Load Test**  
  Includes a basic load test, which can be further improved.
- [ ] 🐳 **Dockerized Application**  
  The entire application is containerized using Docker for easy deployment.

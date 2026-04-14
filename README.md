# api-service Backend (Rust)

A high-performance systems-level backend designed to handle massive datasets with low latency. Built to demonstrate proficiency in Rust memory management, asynchronous I/O, and database optimization.

## Performance Benchmarks
I conducted load testing using `oha` to measure the stability and throughput of the system under heavy concurrency.

* **Throughput:** 1,014.69 Requests/sec
* **Average Latency:** 98.15 ms
* **Success Rate:** 100% (10,000/10,000 requests)
* **Dataset Size:** 5,000,000+ records (MariaDB)
* **Hardware:** 8-core CPU (WSL2 environment)

## Tech Stack
- **Language:** Rust
- **Runtime:** Tokio (Multi-threaded)
- **Database:** MariaDB / SQLx
- **Error Handling:** Anyhow
- **Benchmarking:** oha

## Documentation
To view the internal architectural documentation and function-level comments:
```bash
cargo doc --document-private-items --open
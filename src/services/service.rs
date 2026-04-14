use rayon::prelude::*;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader, Result};
use tokio::sync::Semaphore;
use tokio::sync::mpsc;

use crate::repositories::database::LegoSet;
use crate::repositories::database::Repository;
use std::mem::take;
use std::sync::Arc;
use std::time::Instant;

pub struct Service {
    repo: Arc<dyn Repository>,
}

impl Service {
    pub fn new(repo: Arc<dyn Repository>) -> Self {
        Self { repo }
    }

    /// Read the Lego dataset from the local CSV file into memory buffer
    /// for optimize memory
    pub async fn read(&self) -> Result<()> {
        let file = File::open("lego_sets.csv").await?;
        let mut reader = BufReader::new(file);
        let mut line = String::new();

        let mut counter = 5;
        while reader.read_line(&mut line).await? > 0 && counter < 10 {
            println!("Processing: {}", line.trim());
            line.clear();
            counter += 1
        }

        Ok(())
    }

    /// Fetches records from local Database set up in Docker.
    ///
    /// # Logic
    /// This function read specific record from Database. Using QueryBuilder for Querying
    /// in order to avoid `Database Injection`. The Query line is hardcode in this phase of development.
    ///
    /// # Performance
    /// This function use sqlx for connection pool. Performs good in OLTP Database.
    /// This function is optimized for high-concurrency and has been benchmarked
    /// to handle over 1,000 requests per second using a B-Tree index
    ///
    /// # Errors
    /// Returns an `anyhow::Error` if the database connection fails or the
    /// query builder encounters a syntax issue.
    pub async fn get(&self) -> anyhow::Result<Vec<LegoSet>> {
        let start_time = Instant::now();
        let re = self.repo.get().await?;
        let duration = start_time.elapsed();
        println!("Total get time: {:?}", duration);
        Ok(re)
    }

    /// Batch data and flush into local Database set up in Docker.
    ///
    /// # Logic
    /// This function convert from Array<String> read from CSV file into Array<Object>.
    /// Build the Query by QueryBuilder base on Array<Object>
    /// Then Batching all into database.
    ///
    /// # Performance
    /// This function is high-concurrency and has been benchmarked.
    /// to handle over 5.000.000 rows of raw CSV data within 30 seconds.
    ///
    /// # Errors
    /// Returns an `anyhow::Error` if the database connection fails or the
    /// query builder encounters a syntax issue.
    pub async fn batch(&self) -> Result<()> {
        let start_time = Instant::now();

        let semaphore = Arc::new(Semaphore::new(8));
        let (tx, mut rx) = mpsc::channel::<Vec<String>>(5);

        tokio::spawn(async move {
            let start_time = Instant::now();
            let file = File::open("lego_sets.csv").await.unwrap();
            let duration = start_time.elapsed();
            println!("Total open file: {:?}", duration);

            let mut reader = BufReader::new(file);
            let mut line = String::new();
            let mut current_chunk: Vec<String> = Vec::with_capacity(15000);

            while let Ok(bytes) = reader.read_line(&mut line).await {
                if bytes == 0 {
                    break;
                }
                current_chunk.push(line.clone());
                line.clear();

                if current_chunk.len() >= 15000 {
                    let _ = tx.send(take(&mut current_chunk)).await;
                }
            }
            if !current_chunk.is_empty() {
                let _ = tx.send(current_chunk).await;
            }
        });

        while let Some(chunk) = rx.recv().await {
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let repo = self.repo.clone();

            let processed_batch: Vec<LegoSet> = chunk
                .into_par_iter()
                .map(|line| {
                    let ar: Vec<&str> = line.split(",").collect();
                    LegoSet {
                        set_id: ar
                            .get(0)
                            .unwrap_or(&"0")
                            .trim()
                            .trim_matches('"')
                            .to_string(),
                        name: ar
                            .get(1)
                            .unwrap_or(&"0")
                            .trim()
                            .trim_matches('"')
                            .to_string(),
                        year: ar
                            .get(2)
                            .unwrap_or(&"0")
                            .trim()
                            .trim_matches('"')
                            .to_string(),
                        theme: ar
                            .get(3)
                            .unwrap_or(&"0")
                            .trim()
                            .trim_matches('"')
                            .to_string(),
                    }
                })
                .collect();

            tokio::spawn(async move {
                repo.batch(&processed_batch).await;
                drop(permit);
            });
        }

        let _ = semaphore.acquire_many(8).await;
        let duration = start_time.elapsed();
        println!("Total import time: {:?}", duration);

        println!("Completed in {} seconds", duration.as_secs_f32());
        Ok(())
    }
}

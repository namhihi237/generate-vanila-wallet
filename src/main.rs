mod config;
mod db;
mod wallet_generator;

use anyhow::Result;
use clap::Parser;
use log::{error, info, warn};
use solana_sdk::signature::Signer;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use tokio::sync::Mutex;

use crate::config::Config;
use crate::db::MongoDBClient;
use crate::wallet_generator::WalletGenerator;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Number of threads to use for wallet generation
    #[arg(short, long, env = "THREADS", default_value_t = num_cpus::get())]
    threads: usize,

    /// MongoDB connection string
    #[arg(short, long, env = "MONGODB_URI")]
    mongodb_uri: Option<String>,

    /// MongoDB database name
    #[arg(long, default_value = "vanity_wallets")]
    db_name: String,

    /// MongoDB collection name
    #[arg(long, default_value = "wallets")]
    collection_name: String,

    /// The suffix to search for in wallet addresses
    #[arg(short, long, default_value = "pump")]
    suffix: String,
}

/// The main wallet generation loop that runs in each thread
async fn wallet_generation_loop(
    thread_id: usize,
    wallet_generator: &WalletGenerator,
    counter: &Arc<AtomicUsize>,
    found_wallets: &Arc<AtomicUsize>,
    db_client: &Arc<Mutex<MongoDBClient>>,
) -> Result<()> {
    loop {
        // Generate a wallet
        let wallet = wallet_generator.generate_wallet();

        // Increment counter
        let count = counter.fetch_add(1, Ordering::SeqCst);

        // Print progress every 100000 wallets
        if count % 100000 == 0 {
            let total_found = found_wallets.load(Ordering::SeqCst);
            let wallets_per_second = 100000.0 / 10.0; // Approximate, assuming 10 seconds per 100000 wallets

            info!("=== PROGRESS UPDATE ====");
            info!("Thread: {}", thread_id);
            info!("Generated: {} wallets", count);
            info!("Found: {} vanity wallets", total_found);
            if total_found > 0 {
                info!("Success rate: 1 in {} wallets", count / total_found);
            }
            info!(
                "Performance: ~{:.2} wallets/second (~{:.2} million wallets/hour)",
                wallets_per_second,
                wallets_per_second * 3600.0 / 1_000_000.0
            );
            info!("=== CONTINUING SEARCH ====");
        }

        // Check if wallet address ends with the suffix
        if wallet_generator.is_vanity_wallet(&wallet) {
            let pubkey = wallet.pubkey().to_string();
            let private_key = WalletGenerator::get_private_key_string(&wallet);
            let total_found = found_wallets.fetch_add(1, Ordering::SeqCst) + 1;
            let total_generated = counter.load(Ordering::SeqCst);

            info!("=== VANITY WALLET FOUND! ====");
            info!("Thread: {}", thread_id);
            info!("Public Key: {}", pubkey);
            info!("Private Key: {}", private_key);
            info!("Wallet ends with 'pump' (lowercase)");
            info!("Total wallets generated: {}", total_generated);
            info!("Total vanity wallets found: {}", total_found);
            info!(
                "Success rate: 1 in {} wallets",
                total_generated / total_found
            );
            info!("=== SAVING TO DATABASE ====");

            // Save wallet to MongoDB with error handling
            let mut retry_count = 0;
            const MAX_RETRIES: usize = 3;

            while retry_count < MAX_RETRIES {
                match db_client.lock().await.save_wallet(&wallet).await {
                    Ok(_) => {
                        info!("Wallet successfully saved to MongoDB");
                        break; // Success, exit retry loop
                    }
                    Err(e) => {
                        retry_count += 1;
                        if retry_count >= MAX_RETRIES {
                            error!(
                                "Failed to save wallet to MongoDB after {} retries: {}",
                                MAX_RETRIES, e
                            );
                        } else {
                            warn!(
                                "MongoDB save attempt {} failed: {}. Retrying...",
                                retry_count, e
                            );
                            tokio::time::sleep(tokio::time::Duration::from_millis(
                                500 * retry_count as u64,
                            ))
                            .await;
                        }
                    }
                }
            }
        }

        // Yield to the scheduler occasionally to prevent thread starvation
        if counter.load(Ordering::SeqCst) % 1000 == 0 {
            tokio::task::yield_now().await;
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger
    env_logger::init();

    // Load environment variables from .env file if it exists
    dotenv::dotenv().ok();

    // Parse command line arguments
    let cli = Cli::parse();

    // Create configuration
    let config = Config {
        threads: cli.threads,
        mongodb_uri: cli.mongodb_uri.unwrap_or_else(|| {
            std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_string())
        }),
        db_name: cli.db_name,
        collection_name: cli.collection_name,
        suffix: cli.suffix,
    };

    info!("=== Starting Solana Vanity Wallet Generator ===");
    info!("Configuration:");
    info!("  - Looking for wallets ending with exactly 'pump' (lowercase only)");
    info!("  - Using {} threads", config.threads);
    info!("  - MongoDB URI: {}", config.mongodb_uri);
    info!("  - Database: {}", config.db_name);
    info!("  - Collection: {}", config.collection_name);
    info!("=== Initialization Complete ===");

    // Initialize MongoDB client
    let db_client = MongoDBClient::new(
        &config.mongodb_uri,
        &config.db_name,
        &config.collection_name,
    )
    .await?;

    // Create wallet generator
    let wallet_generator = WalletGenerator::new(&config.suffix);

    // Counter for generated wallets
    let counter = Arc::new(AtomicUsize::new(0));
    let found_wallets = Arc::new(AtomicUsize::new(0));

    // Create a shared MongoDB client
    let db_client = Arc::new(Mutex::new(db_client));

    // Create thread pool
    let handles = (0..config.threads)
        .map(|thread_id| {
            let wallet_generator = wallet_generator.clone();
            let counter = counter.clone();
            let found_wallets = found_wallets.clone();
            let db_client = db_client.clone();

            tokio::spawn(async move {
                info!("Starting thread {}", thread_id);

                // Main processing loop with error recovery
                loop {
                    // Try to run the wallet generation loop
                    // If it fails, log the error and restart the thread
                    if let Err(e) = wallet_generation_loop(
                        thread_id,
                        &wallet_generator,
                        &counter,
                        &found_wallets,
                        &db_client,
                    )
                    .await
                    {
                        error!(
                            "Thread {} encountered an error: {}. Restarting thread...",
                            thread_id, e
                        );
                        // Sleep briefly before restarting to prevent rapid restart loops
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                        warn!("Restarting thread {}", thread_id);
                    }
                }
            })
        })
        .collect::<Vec<_>>();

    // Create a thread monitoring task
    let active_threads = Arc::new(AtomicUsize::new(config.threads));
    let active_threads_clone = active_threads.clone();

    // Spawn a monitoring task
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            let current_active = active_threads_clone.load(Ordering::SeqCst);
            info!(
                "Thread monitor: {} of {} threads active",
                current_active, config.threads
            );

            if current_active < config.threads {
                warn!(
                    "Some threads have stopped! Only {} of {} threads are active",
                    current_active, config.threads
                );
            }
        }
    });

    // Wait for all threads to complete (they won't unless interrupted)
    for handle in handles {
        if let Err(e) = handle.await {
            error!("A thread has terminated with error: {}", e);
            active_threads.fetch_sub(1, Ordering::SeqCst);
        }
    }

    Ok(())
}

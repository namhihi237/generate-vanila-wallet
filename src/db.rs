use crate::wallet_generator::WalletGenerator;
use anyhow::Result;
use chrono::Utc;
use mongodb::bson::doc;
use mongodb::{options::ClientOptions, Client, Collection};
use serde::{Deserialize, Serialize};
use solana_sdk::signature::{Keypair, Signer};

#[derive(Debug, Serialize, Deserialize)]
pub struct WalletDocument {
    pub public_key: String,
    pub private_key: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub struct MongoDBClient {
    collection: Collection<WalletDocument>,
}

impl MongoDBClient {
    pub async fn new(uri: &str, db_name: &str, collection_name: &str) -> Result<Self> {
        // Parse a connection string into an options struct
        let client_options = ClientOptions::parse(uri).await?;

        // Get a handle to the deployment
        let client = Client::with_options(client_options)?;

        // Ping the server to see if you can connect to the cluster
        client
            .database("admin")
            .run_command(doc! {"ping": 1}, None)
            .await?;

        log::info!("Connected to MongoDB!");

        // Get a handle to the specified database and collection
        let db = client.database(db_name);
        let collection = db.collection::<WalletDocument>(collection_name);

        Ok(Self { collection })
    }

    pub async fn save_wallet(&self, keypair: &Keypair) -> Result<()> {
        let public_key = keypair.pubkey().to_string();
        let private_key = WalletGenerator::get_private_key_string(keypair);
        let created_at = chrono::Utc::now();

        log::debug!("Creating wallet document for public key: {}", public_key);
        let wallet_doc = WalletDocument {
            public_key: public_key.clone(),
            private_key,
            created_at,
        };

        log::debug!("Inserting wallet document into MongoDB");
        let result = self.collection.insert_one(wallet_doc, None).await?;
        log::info!("Wallet saved to MongoDB with ID: {}", result.inserted_id);

        Ok(())
    }

    pub async fn get_wallet_count(&self) -> Result<u64> {
        let count = self.collection.count_documents(None, None).await?;
        Ok(count)
    }
}

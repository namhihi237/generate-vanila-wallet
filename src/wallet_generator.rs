use solana_sdk::signature::{Keypair, Signer};

#[derive(Clone)]
pub struct WalletGenerator {}

impl WalletGenerator {
    pub fn new(_suffix: &str) -> Self {
        // We ignore the suffix parameter since we hardcode "pump" in is_vanity_wallet
        Self {}
    }

    /// Generate a new random Solana keypair
    pub fn generate_wallet(&self) -> Keypair {
        let keypair = Keypair::new();
        log::trace!(
            "Generated new keypair with public key: {}",
            keypair.pubkey()
        );
        keypair
    }

    /// Check if the wallet address ends with the specified suffix
    /// Only matches exact case (lowercase "pump")
    pub fn is_vanity_wallet(&self, keypair: &Keypair) -> bool {
        let pubkey = keypair.pubkey().to_string();
        // Check if the public key ends with exactly "pump" (no case conversion)
        let is_vanity = pubkey.ends_with("pump");

        if is_vanity {
            log::debug!("Found vanity wallet ending with 'pump': {}", pubkey);
        } else {
            log::trace!("Public key {} does not end with 'pump'", pubkey);
        }

        is_vanity
    }

    /// Get the public key as a string
    pub fn get_pubkey_string(keypair: &Keypair) -> String {
        keypair.pubkey().to_string()
    }

    /// Get the private key as a base58 string
    pub fn get_private_key_string(keypair: &Keypair) -> String {
        bs58::encode(keypair.to_bytes()).into_string()
    }
}

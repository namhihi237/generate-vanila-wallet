# Solana Vanity Wallet Generator

A high-performance Solana vanity wallet generator that finds wallets with addresses ending with exactly "pump" in lowercase. The application uses multithreading to speed up the wallet generation process and stores the results in MongoDB.

## Features

- Generate Solana wallets with addresses ending with exactly "pump" in lowercase
- Utilize multiple threads for high-performance wallet generation
- Store generated wallets in MongoDB for easy access and management
- Configurable via command-line arguments or environment variables

## Requirements

- Rust 1.57 or higher
- MongoDB (local or remote)

## Installation

1. Clone the repository:
```bash
git clone https://github.com/yourusername/solana-vanity-wallet.git
cd solana-vanity-wallet
```

2. Build the project:
```bash
cargo build --release
```

## Usage

Run the application with default settings:

```bash
./target/release/solana-vanity-wallet
```

### Command-line Options

- `-t, --threads <THREADS>`: Number of threads to use for wallet generation (default: number of CPU cores)
- `-m, --mongodb-uri <MONGODB_URI>`: MongoDB connection string (default: "mongodb://localhost:27017")
- `--db-name <DB_NAME>`: MongoDB database name (default: "vanity_wallets")
- `--collection-name <COLLECTION_NAME>`: MongoDB collection name (default: "wallets")

### Environment Variables

- `THREADS`: Number of threads to use for wallet generation (default: number of CPU cores)
- `MONGODB_URI`: MongoDB connection string (default: "mongodb://localhost:27017")
- `RUST_LOG`: Logging level (e.g., "info", "debug", "trace")

## Examples

### Using Command-line Arguments

Find wallets ending with "pump" using 8 threads and store them in a remote MongoDB database:

```bash
./target/release/solana-vanity-wallet --threads 8 --mongodb-uri "mongodb+srv://username:password@cluster.mongodb.net/vanity_wallets"
```

### Using Environment Variables

You can also use environment variables to configure the application:

```bash
# Set environment variables
export THREADS=16
export MONGODB_URI="mongodb+srv://username:password@cluster.mongodb.net/vanity_wallets"
export RUST_LOG=info

# Run the application
./target/release/solana-vanity-wallet
```

Or in a single line:

```bash
THREADS=16 MONGODB_URI="mongodb+srv://username:password@cluster.mongodb.net/vanity_wallets" RUST_LOG=info ./target/release/solana-vanity-wallet
```

## MongoDB Schema

The generated wallets are stored in MongoDB with the following schema:

```json
{
  "public_key": "String",
  "private_key": "String",
  "created_at": "DateTime"
}
```

## Performance

The application is designed to be highly performant, utilizing all available CPU cores by default. On a modern multi-core system, it can generate and check millions of wallets per hour.

## Security Note

The private keys of the generated wallets are stored in the database. Make sure to secure your MongoDB instance properly to prevent unauthorized access to these keys.

## License

MIT

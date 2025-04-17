#!/bin/bash

# Configuration
REMOTE_USER="root"
REMOTE_HOST="v2202503263036326822.hotsrv.de"  # Replace with your server IP
REMOTE_DIR="/root/vanila"  # Remote directory where code will be deployed
LOCAL_DIR="."  # Current directory containing the project

# Authentication options (defaults)
AUTH_METHOD="key"  # Options: "key" or "password"
SSH_KEY="./id_ed25519"  # ED25519 SSH key path (used if AUTH_METHOD="key")
SSH_PASSWORD=""  # SSH password (used if AUTH_METHOD="password")

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Function to display usage information
show_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo "Options:"
    echo "  -m, --method <key|password>   Authentication method (default: key)"
    echo "  -k, --key <path>              Path to SSH key (default: ./id_ed25519)"
    echo "  -p, --password <password>     SSH password (will prompt if not provided)"
    echo "  -h, --help                    Show this help message"
    exit 1
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -m|--method)
            AUTH_METHOD="$2"
            if [[ "$AUTH_METHOD" != "key" && "$AUTH_METHOD" != "password" ]]; then
                echo -e "${RED}Error: Authentication method must be 'key' or 'password'${NC}"
                show_usage
            fi
            shift 2
            ;;
        -k|--key)
            SSH_KEY="$2"
            shift 2
            ;;
        -p|--password)
            SSH_PASSWORD="$2"
            shift 2
            ;;
        -h|--help)
            show_usage
            ;;
        *)
            echo -e "${RED}Error: Unknown option $1${NC}"
            show_usage
            ;;
    esac
done

# Set SSH options based on authentication method
if [ "$AUTH_METHOD" = "key" ]; then
    SSH_OPTS="-i ${SSH_KEY} -o StrictHostKeyChecking=no"
    RSYNC_SSH_CMD="ssh ${SSH_OPTS}"
else
    # Check if sshpass is installed
    if ! command -v sshpass &> /dev/null; then
        echo -e "${RED}Error: sshpass is not installed. Please install it first.${NC}"
        echo -e "On Ubuntu/Debian: sudo apt-get install sshpass"
        echo -e "On macOS: brew install hudochenkov/sshpass/sshpass"
        exit 1
    fi

    # If password is empty, prompt for it
    if [ -z "$SSH_PASSWORD" ]; then
        read -sp "Enter SSH password for ${REMOTE_USER}@${REMOTE_HOST}: " SSH_PASSWORD
        echo ""
    fi

    SSH_OPTS="-o StrictHostKeyChecking=no"
    RSYNC_SSH_CMD="sshpass -p '${SSH_PASSWORD}' ssh ${SSH_OPTS}"
fi

# Start deployment
echo -e "${GREEN}Starting deployment to ${REMOTE_HOST}...${NC}"

# Ensure the script stops on first error
set -e

# Verify authentication method
if [ "$AUTH_METHOD" = "key" ]; then
    if [ ! -f "${SSH_KEY/#\~/$HOME}" ]; then
        echo -e "${RED}SSH key not found at ${SSH_KEY}${NC}"
        exit 1
    fi
fi


# Create remote directory if it doesn't exist and ensure proper permissions
echo -e "${GREEN}Creating remote directories and setting permissions...${NC}"
if [ "$AUTH_METHOD" = "key" ]; then
    ssh ${SSH_OPTS} ${REMOTE_USER}@${REMOTE_HOST} "mkdir -p ${REMOTE_DIR} ${REMOTE_DIR}/logs ${REMOTE_DIR}/target/release && chmod -R 755 ${REMOTE_DIR}"
else
    sshpass -p "${SSH_PASSWORD}" ssh ${SSH_OPTS} ${REMOTE_USER}@${REMOTE_HOST} "mkdir -p ${REMOTE_DIR} ${REMOTE_DIR}/logs ${REMOTE_DIR}/target/release && chmod -R 755 ${REMOTE_DIR}"
fi

# Sync the code to the remote server
# --delete: remove files on remote that don't exist locally
# --exclude: exclude unnecessary files/directories
echo -e "${GREEN}Syncing code to remote server...${NC}"

# Set rsync SSH command based on authentication method
if [ "$AUTH_METHOD" = "key" ]; then
    RSYNC_CMD="rsync -avz --delete -e \"ssh ${SSH_OPTS}\" \
        --exclude '.git' \
        --exclude 'target' \
        --exclude '.env' \
        --exclude '.gitignore' \
        --exclude 'deploy.sh' \
        --exclude '.DS_Store' \
        --exclude 'id_ed25519' \
        --exclude 'logs' \
        ${LOCAL_DIR}/ ${REMOTE_USER}@${REMOTE_HOST}:${REMOTE_DIR}/"
else
    RSYNC_CMD="rsync -avz --delete -e \"sshpass -p '${SSH_PASSWORD}' ssh ${SSH_OPTS}\" \
        --exclude '.git' \
        --exclude 'target' \
        --exclude '.env' \
        --exclude '.gitignore' \
        --exclude 'deploy.sh' \
        --exclude '.DS_Store' \
        --exclude 'id_ed25519' \
        --exclude 'logs' \
        ${LOCAL_DIR}/ ${REMOTE_USER}@${REMOTE_HOST}:${REMOTE_DIR}/"
fi

# Execute the rsync command
eval "$RSYNC_CMD"


# Stop any existing process
echo -e "${GREEN}Deployment completed successfully!${NC}"

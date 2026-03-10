#!/bin/bash
set -e

# ==============================================================================
# Oxide VM Setup Script
# Run this on a fresh Ubuntu 22.04 / 24.04 instance to prepare the environment
# ==============================================================================

echo "========================================"
echo "    Starting Oxide VM Initialization    "
echo "========================================"

# 1. Update system packages
echo ">>> Updating system packages..."
sudo apt-get update && sudo apt-get upgrade -y
sudo apt-get install -y curl wget git cmake pkg-config libssl-dev build-essential

# 2. Install Docker
echo ">>> Installing Docker..."
if ! command -v docker &> /dev/null; then
    curl -fsSL https://get.docker.com -o get-docker.sh
    sudo sh get-docker.sh
    sudo usermod -aG docker $USER
    rm get-docker.sh
    echo "Docker installed successfully."
else
    echo "Docker is already installed."
fi

# 3. Install Nix (Determinate Systems Installer)
echo ">>> Installing Nix..."
if ! command -v nix &> /dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install --no-confirm
    
    # Source Nix immediately for the script to use
    . /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh
    echo "Nix installed successfully."
else
    echo "Nix is already installed."
fi

# 4. Install Rust
echo ">>> Installing Rust..."
if ! command -v cargo &> /dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    echo "Rust installed successfully."
else
    echo "Rust is already installed."
fi

# 5. Install PostgreSQL
echo ">>> Installing PostgreSQL..."
if ! command -v psql &> /dev/null; then
    sudo apt-get install -y postgresql postgresql-contrib
    sudo systemctl enable postgresql
    sudo systemctl start postgresql

    # Setup oxide user and database
    echo ">>> Configuring PostgreSQL User and Database..."
    sudo -u postgres psql -c "CREATE USER postgres WITH PASSWORD 'postgres';" || true
    sudo -u postgres psql -c "ALTER USER postgres WITH SUPERUSER;" || true
    sudo -u postgres psql -c "CREATE DATABASE oxide;" || true
    echo "PostgreSQL configured successfully."
else
    echo "PostgreSQL is already installed."
fi

# 6. Prepare Oxide Environment Directories
echo ">>> Preparing Oxide Build & Runtime Directories..."
sudo mkdir -p /var/oxide/builds
sudo mkdir -p /var/oxide/runtime
sudo chown -R $USER:$USER /var/oxide

echo "========================================"
echo "  Oxide Environment Setup Complete!     "
echo "========================================"
echo ""
echo "Next Steps:"
echo "1. Run 'source ~/.bashrc' or log out and log back in (so Docker and Nix groups apply)."
echo "2. Clone your Oxide repository: git clone https://github.com/HrushikeshAnandSarangi/oxide"
echo "3. cd into oxide and run 'cargo build --release'"
echo "4. Set DATABASE_URL and run migrations via sqlx."

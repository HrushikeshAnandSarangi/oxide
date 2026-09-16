#!/bin/bash
set -e

# ==============================================================================
# Oxide Systemd Installation Script
# Run this after compiling the project on the VM to install the services.
# ==============================================================================

echo "========================================"
echo "    Installing Oxide Systemd Services   "
echo "========================================"

# 1. Copy the compiled binaries to a stable location
echo ">>> Setting up application directories..."
sudo mkdir -p /var/oxide/app

# Change directory to the root of the oxide repository (parent of the script)
cd "$(dirname "$0")/.."

sudo cp -r . /var/oxide/app/
sudo chown -R root:root /var/oxide/app

# 2. Copy the systemd service file
# NOTE: there is only one binary (`api`) — it spawns the Pingora proxy
# in-process (see api/src/main.rs), so there is no separate proxy service.
echo ">>> Copying systemd service file..."
sudo cp systemd/oxide-api.service /etc/systemd/system/

# 3. Reload systemd daemon
echo ">>> Reloading systemctl daemon..."
sudo systemctl daemon-reload

# 4. Enable service to start on boot
echo ">>> Enabling service..."
sudo systemctl enable oxide-api

# 5. Start service
echo ">>> Starting service..."
sudo systemctl start oxide-api

echo "========================================"
echo "  Oxide Services Installed & Running!   "
echo "========================================"
echo "Check status manually with: "
echo "  sudo systemctl status oxide-api"
echo "  sudo systemctl status oxide-proxy"
echo "View logs with: "
echo "  sudo journalctl -u oxide-api -f"
echo "  sudo journalctl -u oxide-proxy -f"

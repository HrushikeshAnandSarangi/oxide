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
# We assume this script is run from the root of the oxide repository
sudo cp -r . /var/oxide/app/
sudo chown -R root:root /var/oxide/app

# 2. Copy the systemd service files
echo ">>> Copying systemd service files..."
sudo cp systemd/oxide-api.service /etc/systemd/system/
sudo cp systemd/oxide-proxy.service /etc/systemd/system/

# 3. Reload systemd daemon
echo ">>> Reloading systemctl daemon..."
sudo systemctl daemon-reload

# 4. Enable services to start on boot
echo ">>> Enabling services..."
sudo systemctl enable oxide-api
sudo systemctl enable oxide-proxy

# 5. Start services
echo ">>> Starting services..."
sudo systemctl start oxide-api
sudo systemctl start oxide-proxy

echo "========================================"
echo "  Oxide Services Installed & Running!   "
echo "========================================"
echo "Check status manually with: "
echo "  sudo systemctl status oxide-api"
echo "  sudo systemctl status oxide-proxy"
echo "View logs with: "
echo "  sudo journalctl -u oxide-api -f"
echo "  sudo journalctl -u oxide-proxy -f"

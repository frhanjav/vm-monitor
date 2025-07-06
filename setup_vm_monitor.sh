#!/bin/bash

# VM Monitor Setup Script
# This script clones the vm-monitor repository and sets up all components

set -e  # Exit on any error

echo "🚀 Starting VM Monitor setup..."

# Update system packages
echo "📦 Updating system packages..."
sudo apt update && sudo apt upgrade -y

# Install essential build tools and dependencies
echo "🔧 Installing build essentials and dependencies..."
sudo apt install -y \
    build-essential \
    curl \
    wget \
    git \
    pkg-config \
    libssl-dev \
    libssl3 \
    openssl \
    ca-certificates \
    gnupg \
    lsb-release \
    python3 \
    python3-pip \
    python3-venv \
    python3-dev

# Clone the repository
echo "📁 Cloning vm-monitor repository..."
if [ -d "vm-monitor" ]; then
    echo "⚠️  vm-monitor directory already exists. Removing it..."
    rm -rf vm-monitor
fi
git clone https://github.com/frhanjav/vm-monitor.git
cd vm-monitor

# Install Node.js 20
echo "🟢 Installing Node.js 20..."
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt-get install -y nodejs

# Verify Node.js installation
echo "✅ Node.js version: $(node --version)"
echo "✅ npm version: $(npm --version)"

# Install Rust and Cargo
echo "🦀 Installing Rust and Cargo..."
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source ~/.cargo/env

# Add cargo to PATH for current session
export PATH="$HOME/.cargo/bin:$PATH"

# Verify Rust installation
echo "✅ Rust version: $(rustc --version)"
echo "✅ Cargo version: $(cargo --version)"

# Setup vm-monitor-frontend
echo "🌐 Setting up vm-monitor-frontend..."
cd vm-monitor-frontend
npm install
echo "✅ Frontend dependencies installed successfully"
cd ..

# Setup vm-monitor (Rust project)
echo "🦀 Setting up vm-monitor Rust project..."
cd vm-monitor

# Set environment variables for OpenSSL
export OPENSSL_DIR=/usr
export OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu
export OPENSSL_INCLUDE_DIR=/usr/include/openssl

# Build the Rust project
echo "🔨 Building Rust project..."
cargo build

if [ $? -eq 0 ]; then
    echo "✅ Rust project built successfully"
else
    echo "❌ Rust build failed"
    exit 1
fi
cd ..

# Setup vm_monitor_api (Python project)
echo "🐍 Setting up vm_monitor_api..."
cd vm_monitor_api

# Create virtual environment
echo "🔧 Creating Python virtual environment..."
python3 -m venv venv

# Activate virtual environment
echo "🔧 Activating virtual environment..."
source venv/bin/activate

# Install Python dependencies
echo "📦 Installing Python dependencies..."
pip install -r requirements.txt

if [ $? -eq 0 ]; then
    echo "✅ Python dependencies installed successfully"
else
    echo "❌ Python dependencies installation failed"
    deactivate
    exit 1
fi

# Deactivate virtual environment
deactivate
echo "✅ Python virtual environment setup complete"
cd ..

# Final setup completion
echo ""
echo "🎉 VM Monitor setup completed successfully!"
echo ""
echo "📋 Summary:"
echo "  ✅ Repository cloned"
echo "  ✅ Node.js 20 installed"
echo "  ✅ Rust/Cargo installed"
echo "  ✅ Python 3 with venv ready"
echo "  ✅ Frontend dependencies installed"
echo "  ✅ Rust project built"
echo "  ✅ Python API dependencies installed"
echo ""
echo "🚀 Next steps:"
echo "  • Frontend: cd vm-monitor/vm-monitor-frontend && npm start"
echo "  • Rust: cd vm-monitor/vm-monitor && cargo run"
echo "  • Python API: cd vm-monitor/vm_monitor_api && source venv/bin/activate && python3 app.py"
echo ""
echo "💡 Note: To use cargo in new terminals, run: source ~/.cargo/env"
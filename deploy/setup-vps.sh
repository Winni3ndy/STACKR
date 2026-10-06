#!/bin/bash
# Stackr VPS Initial Setup Script
# Run this on a fresh Ubuntu 22.04 VPS
# Usage: wget -O - https://raw.githubusercontent.com/yourusername/stackr/main/deploy/setup-vps.sh | bash

set -e

echo "🚀 Stackr VPS Setup Script"
echo "=========================="

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${YELLOW}Step 1: Updating system...${NC}"
apt update && apt upgrade -y

echo -e "${YELLOW}Step 2: Installing dependencies...${NC}"
apt install -y \
    build-essential \
    curl \
    git \
    pkg-config \
    libssl-dev \
    postgresql \
    postgresql-contrib \
    redis-server \
    nginx \
    certbot \
    python3-certbot-nginx \
    jq \
    htop

echo -e "${YELLOW}Step 3: Installing Rust...${NC}"
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env

echo -e "${YELLOW}Step 4: Creating stackr user...${NC}"
if ! id "stackr" &>/dev/null; then
    useradd -m -s /bin/bash stackr
    echo "Enter password for stackr user:"
    passwd stackr
    echo -e "${GREEN}✓ User created${NC}"
else
    echo -e "${GREEN}✓ User already exists${NC}"
fi

echo -e "${YELLOW}Step 5: Setting up PostgreSQL...${NC}"
sudo -u postgres psql -c "CREATE USER stackr WITH PASSWORD 'changeme';" || echo "User exists"
sudo -u postgres psql -c "CREATE DATABASE stackr OWNER stackr;" || echo "Database exists"
sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE stackr TO stackr;"

echo -e "${YELLOW}Step 6: Configuring Redis...${NC}"
systemctl enable redis-server
systemctl start redis-server

echo -e "${YELLOW}Step 7: Configuring firewall...${NC}"
ufw allow 22/tcp
ufw allow 80/tcp
ufw allow 443/tcp
ufw --force enable

echo -e "${YELLOW}Step 8: Creating app directory...${NC}"
mkdir -p /opt/stackr
chown stackr:stackr /opt/stackr

echo -e "${YELLOW}Step 9: Creating log directory...${NC}"
mkdir -p /var/log/stackr
chown stackr:stackr /var/log/stackr

echo ""
echo -e "${GREEN}=========================="
echo -e "✓ VPS setup complete!"
echo -e "==========================${NC}"
echo ""
echo "Next steps:"
echo "1. Switch to stackr user: su - stackr"
echo "2. Clone repository: cd /opt/stackr && git clone <your-repo> ."
echo "3. Configure .env: cp .env.example .env && nano .env"
echo "4. Build: cargo build --release"
echo "5. Setup service: sudo cp deploy/stackr.service /etc/systemd/system/"
echo "6. Start service: sudo systemctl start stackr"
echo ""
echo "Security reminders:"
echo "- Change PostgreSQL password: sudo -u postgres psql"
echo "- Configure SSH key authentication"
echo "- Install fail2ban: apt install fail2ban"
echo "- Review .env permissions: chmod 600 .env"

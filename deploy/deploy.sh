#!/bin/bash
# Stackr Production Deployment Script
# Usage: ./deploy/deploy.sh

set -e

echo "🚀 Stackr Production Deployment"
echo "================================"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
APP_DIR="/opt/stackr"
APP_USER="stackr"
SERVICE_NAME="stackr"

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo -e "${RED}✗ Please run as root (sudo ./deploy/deploy.sh)${NC}"
    exit 1
fi

echo -e "${YELLOW}Step 1: Stopping service...${NC}"
systemctl stop $SERVICE_NAME || echo "Service not running yet"

echo -e "${YELLOW}Step 2: Building release binary...${NC}"
cd /opt/stackr
sudo -u $APP_USER cargo build --release

echo -e "${YELLOW}Step 3: Running database migrations...${NC}"
sudo -u $APP_USER cargo run --release --bin migrate || echo "No migrations to run"

echo -e "${YELLOW}Step 4: Updating systemd service...${NC}"
cp deploy/stackr.service /etc/systemd/system/
systemctl daemon-reload

echo -e "${YELLOW}Step 5: Starting service...${NC}"
systemctl start $SERVICE_NAME
systemctl enable $SERVICE_NAME

echo -e "${YELLOW}Step 6: Checking service status...${NC}"
sleep 3
if systemctl is-active --quiet $SERVICE_NAME; then
    echo -e "${GREEN}✓ Service is running${NC}"
    systemctl status $SERVICE_NAME --no-pager
else
    echo -e "${RED}✗ Service failed to start${NC}"
    journalctl -u $SERVICE_NAME --no-pager -n 50
    exit 1
fi

echo -e "${YELLOW}Step 7: Testing health endpoint...${NC}"
sleep 2
if curl -f http://localhost:3003/health > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Health check passed${NC}"
    curl -s http://localhost:3003/health | jq .
else
    echo -e "${RED}✗ Health check failed${NC}"
    exit 1
fi

echo ""
echo -e "${GREEN}================================${NC}"
echo -e "${GREEN}✓ Deployment successful!${NC}"
echo -e "${GREEN}================================${NC}"
echo ""
echo "Service status: systemctl status $SERVICE_NAME"
echo "View logs: journalctl -u $SERVICE_NAME -f"
echo "Restart: systemctl restart $SERVICE_NAME"
echo "Stop: systemctl stop $SERVICE_NAME"

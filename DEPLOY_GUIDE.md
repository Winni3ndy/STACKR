# Production Deployment Guide

Complete guide to deploying Stackr to a VPS in production.

## Prerequisites

- VPS with **minimum 2GB RAM** (4GB recommended)
- Ubuntu 22.04 LTS or newer
- Domain name pointed to your VPS IP
- SSH access to your VPS

---

## Step 1: VPS Setup

### 1.1 Connect to your VPS

```bash
ssh root@your-vps-ip
```

### 1.2 Create application user

```bash
# Create user
useradd -m -s /bin/bash stackr
usermod -aG sudo stackr

# Set password
passwd stackr
```

### 1.3 Install dependencies

```bash
# Update system
apt update && apt upgrade -y

# Install build essentials
apt install -y build-essential curl git pkg-config libssl-dev

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env

# Install PostgreSQL
apt install -y postgresql postgresql-contrib

# Install Redis
apt install -y redis-server

# Install Nginx
apt install -y nginx

# Install certbot (for SSL)
apt install -y certbot python3-certbot-nginx

# Install jq (for JSON parsing)
apt install -y jq
```

---

## Step 2: Database Setup

### 2.1 Configure PostgreSQL

```bash
# Switch to postgres user
sudo -u postgres psql

# Inside psql:
CREATE USER stackr WITH PASSWORD 'your_strong_password';
CREATE DATABASE stackr OWNER stackr;
GRANT ALL PRIVILEGES ON DATABASE stackr TO stackr;
\q
```

### 2.2 Configure Redis

```bash
# Edit redis config
nano /etc/redis/redis.conf

# Change bind address (if using external Redis):
# bind 127.0.0.1 ::1

# Restart Redis
systemctl restart redis-server
systemctl enable redis-server
```

---

## Step 3: Clone and Build

### 3.1 Clone repository

```bash
# Switch to stackr user
su - stackr

# Create app directory
sudo mkdir -p /opt/stackr
sudo chown stackr:stackr /opt/stackr

# Clone repository
cd /opt/stackr
git clone https://github.com/yourusername/stackr.git .

# Or if using external database (Neon/Supabase), skip PostgreSQL setup
```

### 3.2 Configure environment

```bash
cd /opt/stackr
cp .env.example .env
nano .env
```

**Edit .env with your production values:**

```bash
# Server
HOST=0.0.0.0
PORT=3003
ENVIRONMENT=production

# Database (local or external)
DATABASE_URL=postgresql://stackr:your_password@localhost:5432/stackr
# OR for Neon:
# DATABASE_URL=postgresql://user:pass@ep-xxx.neon.tech/stackr?sslmode=require

# Redis (local or Upstash)
REDIS_URL=redis://127.0.0.1:6379
# OR for Upstash:
# REDIS_URL=rediss://default:xxx@global-xxx.upstash.io:6379

# Stellar (use testnet first!)
STELLAR_HORIZON_URL=https://horizon-testnet.stellar.org
STELLAR_NETWORK_PASSPHRASE=Test SDF Network ; September 2015

# Generate with: stellar keys generate fee-payer --network testnet
FEE_PAYER_SECRET=S...your_fee_payer_secret...

# USDC (testnet issuer)
USDC_ASSET_CODE=USDC
USDC_ISSUER=GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5

# Wallet security
# Generate with: openssl rand -base64 64
WALLET_MASTER_SEED=your_very_long_random_master_seed_64_chars_minimum
# Generate with: openssl rand -hex 32
WALLET_ENCRYPTION_KEY=64_hex_characters_here

# Africa's Talking
AT_API_KEY=your_production_api_key
AT_USERNAME=your_username
AT_USSD_SHORTCODE=*384*5678#
AT_SENDER_ID=STACKR

# Airbills
AIRBILLS_API_KEY=your_airbills_key
AIRBILLS_BASE_URL=https://api.airbills.co

# Local currency
FIAT_CURRENCY=RWF
FIAT_COUNTRY=RW

# Anchor
ANCHOR_DOMAIN=cowrie.exchange
PUBLIC_BASE_URL=https://api.yourdomain.com

# Security
TRUSTED_PROXY_IPS=
AT_ALLOWED_IPS=54.80.243.101,54.220.204.17
INTERNAL_API_KEY=generate_random_key_here
WEBHOOK_SECRET=another_random_key

# Rate limiting
RATE_LIMIT_REQUESTS=60
RATE_LIMIT_WINDOW_SECS=60

# KYC limits (USDC)
KYC_TIER0_DAILY_LIMIT=50.0
KYC_TIER1_DAILY_LIMIT=500.0
KYC_TIER2_DAILY_LIMIT=5000.0
```

### 3.3 Build application

```bash
cd /opt/stackr
cargo build --release

# This will take 5-10 minutes
# Binary will be at: /opt/stackr/target/release/stackr
```

### 3.4 Create log directory

```bash
sudo mkdir -p /var/log/stackr
sudo chown stackr:stackr /var/log/stackr
```

---

## Step 4: Configure Systemd Service

```bash
# Copy service file
sudo cp /opt/stackr/deploy/stackr.service /etc/systemd/system/

# Reload systemd
sudo systemctl daemon-reload

# Start service
sudo systemctl start stackr

# Enable on boot
sudo systemctl enable stackr

# Check status
sudo systemctl status stackr

# View logs
journalctl -u stackr -f
```

---

## Step 5: Configure Nginx

### 5.1 Update nginx config

```bash
# Copy config
sudo cp /opt/stackr/deploy/nginx.conf /etc/nginx/sites-available/stackr

# Update domain name
sudo nano /etc/nginx/sites-available/stackr
# Change: server_name api.yourdomain.com;

# Enable site
sudo ln -s /etc/nginx/sites-available/stackr /etc/nginx/sites-enabled/

# Remove default site
sudo rm /etc/nginx/sites-enabled/default

# Test config
sudo nginx -t

# Reload nginx
sudo systemctl reload nginx
```

### 5.2 Get SSL certificate

```bash
# Get Let's Encrypt certificate
sudo certbot --nginx -d api.yourdomain.com

# Follow prompts:
# - Enter email
# - Agree to terms
# - Redirect HTTP to HTTPS: Yes

# Test auto-renewal
sudo certbot renew --dry-run
```

---

## Step 6: Verify Deployment

### 6.1 Test health endpoint

```bash
curl https://api.yourdomain.com/health
```

Expected response:
```json
{
  "status": "healthy",
  "database": true,
  "redis": true,
  "stellar": true
}
```

### 6.2 Test API (with your API key)

```bash
# Create test API key first (see below)

# Test balance check
curl https://api.yourdomain.com/api/v1/balance/+250781234567 \
  -H "Authorization: Bearer sk_test_your_key"
```

---

## Step 7: Create First Operator & API Key

### 7.1 Connect to database

```bash
sudo -u postgres psql stackr
```

### 7.2 Create operator

```sql
-- Generate UUID (or use: SELECT gen_random_uuid())
INSERT INTO operators (
    id,
    slug,
    name,
    is_active,
    fiat_currency,
    fiat_country,
    wallet_master_seed,
    wallet_encryption_key,
    fee_payer_secret,
    usdc_asset_code,
    usdc_issuer,
    stellar_horizon_url,
    stellar_network_passphrase,
    anchor_domain,
    at_api_key,
    at_username,
    at_ussd_shortcode,
    at_sender_id,
    airbills_api_key,
    airbills_base_url,
    kyc_tier0_daily_limit,
    kyc_tier1_daily_limit,
    kyc_tier2_daily_limit
) VALUES (
    gen_random_uuid(),
    'default',
    'Default Operator',
    true,
    'RWF',
    'RW',
    'your_master_seed_from_env',
    '0000000000000000000000000000000000000000000000000000000000000000',
    'your_fee_payer_secret',
    'USDC',
    'GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5',
    'https://horizon-testnet.stellar.org',
    'Test SDF Network ; September 2015',
    'cowrie.exchange',
    'your_at_api_key',
    'your_at_username',
    '*384*5678#',
    'STACKR',
    'your_airbills_key',
    'https://api.airbills.co',
    50.0,
    500.0,
    5000.0
);
```

### 7.3 Generate API key

```bash
# Generate random API key
openssl rand -base64 32 | tr -d "=+/" | cut -c1-32

# Output: abc123def456...
# Full key: sk_live_abc123def456...
```

### 7.4 Store API key hash

```sql
-- Hash the key (SHA-256)
-- Use this SQL function:
INSERT INTO api_keys (
    id,
    operator_id,
    key_hash,
    key_prefix,
    label,
    is_active
) VALUES (
    gen_random_uuid(),
    (SELECT id FROM operators WHERE slug = 'default'),
    encode(sha256('sk_live_abc123def456...'::bytea), 'hex'),
    'sk_live_',
    'Production API Key',
    true
);

-- Save the full key somewhere safe!
-- You can't retrieve it later, only the hash is stored.
```

---

## Step 8: Monitoring & Logs

### 8.1 View application logs

```bash
# Real-time logs
journalctl -u stackr -f

# Last 100 lines
journalctl -u stackr -n 100

# Errors only
journalctl -u stackr -p err

# Today's logs
journalctl -u stackr --since today
```

### 8.2 View nginx logs

```bash
# Access log
tail -f /var/log/nginx/stackr_access.log

# Error log
tail -f /var/log/nginx/stackr_error.log
```

### 8.3 Monitor resources

```bash
# CPU and memory
htop

# Disk usage
df -h

# Database size
sudo -u postgres psql stackr -c "SELECT pg_size_pretty(pg_database_size('stackr'));"
```

---

## Step 9: Backups

### 9.1 Database backup script

```bash
# Create backup script
sudo nano /opt/stackr/backup.sh
```

```bash
#!/bin/bash
BACKUP_DIR="/opt/stackr/backups"
DATE=$(date +%Y%m%d_%H%M%S)
mkdir -p $BACKUP_DIR

# Backup database
sudo -u postgres pg_dump stackr | gzip > $BACKUP_DIR/stackr_$DATE.sql.gz

# Keep only last 7 days
find $BACKUP_DIR -name "stackr_*.sql.gz" -mtime +7 -delete

echo "Backup completed: stackr_$DATE.sql.gz"
```

```bash
# Make executable
chmod +x /opt/stackr/backup.sh

# Add to cron (daily at 2 AM)
sudo crontab -e
# Add line:
0 2 * * * /opt/stackr/backup.sh >> /var/log/stackr/backup.log 2>&1
```

### 9.2 .env backup

```bash
# Backup .env file (keep secure!)
sudo cp /opt/stackr/.env /opt/stackr/.env.backup
sudo chmod 600 /opt/stackr/.env.backup
```

---

## Step 10: Updates & Maintenance

### 10.1 Deploy updates

```bash
cd /opt/stackr

# Pull latest code
git pull origin main

# Build
cargo build --release

# Run migrations
cargo run --release

# Restart service
sudo systemctl restart stackr

# Check status
sudo systemctl status stackr
```

### 10.2 Or use deployment script

```bash
sudo /opt/stackr/deploy/deploy.sh
```

---

## Troubleshooting

### Service won't start

```bash
# Check logs
journalctl -u stackr -n 50

# Common issues:
# - Database connection failed → Check DATABASE_URL in .env
# - Redis connection failed → Check if Redis is running
# - Port already in use → Check if another process uses port 3003
```

### Health check fails

```bash
# Test from inside VPS
curl http://localhost:3003/health

# If works locally but not externally:
# - Check firewall: sudo ufw status
# - Check nginx: sudo nginx -t && sudo systemctl status nginx
# - Check SSL: sudo certbot certificates
```

### Database migration errors

```bash
# Check current migration state
sudo -u postgres psql stackr -c "SELECT * FROM _migrations;"

# Manually run migrations
cd /opt/stackr
cargo run --release
```

---

## Security Checklist

- [ ] Strong passwords for database
- [ ] `.env` file permissions: `chmod 600 .env`
- [ ] Firewall enabled: `sudo ufw enable`
- [ ] Only ports 22, 80, 443 open
- [ ] SSH key-only authentication
- [ ] Fail2ban installed
- [ ] Backups automated
- [ ] SSL certificate auto-renewal working
- [ ] API keys stored securely
- [ ] Secrets not in git repository

---

## Production Checklist

Before going live:

- [ ] Using mainnet Stellar network
- [ ] Using production USDC issuer
- [ ] Africa's Talking production account
- [ ] Real shortcode registered
- [ ] SSL certificate installed
- [ ] Monitoring set up
- [ ] Backups automated
- [ ] Health checks passing
- [ ] API keys generated
- [ ] Rate limiting tested
- [ ] Load testing completed
- [ ] Security audit passed

---

## Support

For issues:
- Check logs: `journalctl -u stackr -f`
- Review this guide
- Check README.md
- Test health endpoint

**Never share:**
- `.env` file contents
- API keys
- Database passwords
- Wallet master seed
- Fee payer secret key

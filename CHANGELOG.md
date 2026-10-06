# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Multi-tenant operator system
- REST API for external integrations
- USSD interface via Africa's Talking
- Stellar blockchain integration
- SEP-24 anchor support (deposit/withdrawal)
- Airtime and bill payment integration
- Token swap functionality
- Rate limiting and security middleware
- Database migrations
- API key authentication
- Comprehensive documentation
- Production deployment configurations
- Systemd service files
- Nginx reverse proxy config
- VPS setup scripts

### Security
- API key SHA-256 hashing
- PIN encryption with AES-256-GCM
- Wallet master seed HKDF derivation
- IP allowlist for webhooks
- Rate limiting on all endpoints

## [0.1.0] - Initial Release

### Added
- Basic USSD menu system
- Wallet creation and management
- P2P transfers
- Balance checking
- Transaction history
- Redis session management

[Unreleased]: https://github.com/yourusername/stackr/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/yourusername/stackr/releases/tag/v0.1.0

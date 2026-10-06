# Contributing to Stackr

Thank you for your interest in contributing to Stackr!

## Development Setup

1. **Clone the repository**
   ```bash
   git clone https://github.com/yourusername/stackr.git
   cd stackr
   ```

2. **Install dependencies**
   - Rust (latest stable)
   - PostgreSQL 12+
   - Redis 6+

3. **Setup environment**
   ```bash
   cp .env.example .env
   # Edit .env with your local config
   ```

4. **Run migrations**
   ```bash
   cargo run
   ```

5. **Run tests**
   ```bash
   cargo test
   cargo clippy --all-targets -- -D warnings
   cargo fmt --check
   ```

## Code Style

- Use `cargo fmt` before committing
- Run `cargo clippy` and fix all warnings
- Write tests for new features
- Update documentation for public APIs

## Pull Request Process

1. Create a feature branch: `git checkout -b feature/your-feature`
2. Make your changes
3. Run tests and linters
4. Commit with clear messages
5. Push and create a pull request
6. Wait for review

## Testing USSD Flows

Use the test script:
```bash
./scripts/test_ussd.sh
```

All 41 tests should pass.

## Security

If you find a security vulnerability, please email security@yourdomain.com instead of creating a public issue.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

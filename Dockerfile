# Multi-stage build for minimal production image

# Stage 1: Build
FROM rust:1.80-slim-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && echo "" > src/lib.rs
RUN cargo build --release 2>/dev/null || true
RUN rm -rf src

# Build actual source
COPY src/ src/
COPY migrations/ migrations/
RUN touch src/main.rs src/lib.rs
RUN cargo build --release

# Stage 2: Runtime (distroless for minimal attack surface)
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

RUN useradd -r -s /bin/false stackr

COPY --from=builder /app/target/release/stackr /usr/local/bin/stackr

USER stackr

EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
  CMD ["/usr/local/bin/stackr", "--health-check"] || exit 1

ENTRYPOINT ["/usr/local/bin/stackr"]

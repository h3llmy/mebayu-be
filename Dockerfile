FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this remains cached unless dependencies change
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
# Ensure we use offline mode for SQLx so we don't need a live DB during build
ENV SQLX_OFFLINE=true
RUN cargo build --release --bin mebayu_be

# Runtime stage
FROM debian:trixie-slim AS runtime
WORKDIR /app

# Install required packages (openssl, certs, curl for healthcheck)
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends \
        openssl \
        ca-certificates \
        curl \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/mebayu_be mebayu_be

ENV APP_ENVIRONMENT=production

ENTRYPOINT ["./mebayu_be"]

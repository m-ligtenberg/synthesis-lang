FROM rust:1.75-slim AS builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    curl \
    wget \
    git \
    && rm -rf /var/lib/apt/lists/*

# Set up Synthesis language sandbox
WORKDIR /app

# Copy necessary files
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build the sandbox runtime
RUN cargo build --release --bin synthesis-sandbox

# Final stage
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy built binary
COPY --from=builder /app/target/release/synthesis-sandbox /usr/local/bin/synthesis-sandbox

# Set up sandbox environment
RUN useradd -m sandboxuser
USER sandboxuser
WORKDIR /home/sandboxuser

# Expose sandbox API port
EXPOSE 8080

# Entrypoint
ENTRYPOINT ["/usr/local/bin/synthesis-sandbox"]
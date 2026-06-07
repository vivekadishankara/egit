# ── Stage 1: Build ────────────────────────────────────────────────────
FROM rust:1.82-bookworm AS builder

# Install cargo-leptos and wasm target
RUN rustup target add wasm32-unknown-unknown
RUN cargo install cargo-leptos --version 0.3.6 --locked

# Install Node for Tailwind
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
    && apt-get install -y nodejs

WORKDIR /app

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && echo 'fn main(){}' > src/main.rs && echo '' > src/lib.rs
RUN cargo fetch

# Copy source and build
COPY . .
RUN npm install
RUN cargo leptos build --release

# ── Stage 2: Runtime ──────────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates git && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/egit /app/egit
COPY --from=builder /app/target/site /app/site

# Repo storage volume
RUN mkdir -p /data/repos
VOLUME ["/data/repos"]

EXPOSE 3000

CMD ["/app/egit"]

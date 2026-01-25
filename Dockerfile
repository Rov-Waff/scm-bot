# Multi-stage Dockerfile for building `scm-bot` (Rust)

FROM rust:1.71-slim AS builder
WORKDIR /usr/src/app

# Install build dependencies (kept minimal)
RUN apt-get update \
	&& apt-get install -y --no-install-recommends \
	   pkg-config \
	   libssl-dev \
	   build-essential \
	   ca-certificates \
	   git \
	&& rm -rf /var/lib/apt/lists/*

# Copy manifest first to cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() { println!("placeholder"); }' > src/main.rs
RUN cargo build --release || true
RUN rm -rf src

# Copy the source and build the release binary
COPY . .
RUN cargo build --release


FROM debian:bullseye-slim AS runtime
RUN apt-get update \
	&& apt-get install -y --no-install-recommends ca-certificates \
	&& rm -rf /var/lib/apt/lists/* \
	&& groupadd -r app || true \
	&& useradd -r -g app app || true

# Copy the release binary from the builder stage
COPY --from=builder /usr/src/app/target/release/scm-bot /usr/local/bin/scm-bot
RUN chmod +x /usr/local/bin/scm-bot

USER app
ENV RUST_LOG=info
ENTRYPOINT ["/usr/local/bin/scm-bot"]


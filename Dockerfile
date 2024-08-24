# Stage 1: build the Rust application
FROM rust:slim-buster AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

# Stage 2: create the final image
FROM debian:buster-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends ffmpeg ca-certificates && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/media-vault .
RUN chmod +x media-vault

ENTRYPOINT ["./media-vault"]

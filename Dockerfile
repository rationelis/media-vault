# Step 1: Build the Rust project
FROM rust:latest AS builder
WORKDIR /app

COPY Cargo.toml .
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release

COPY src src
RUN touch src/main.rs
RUN cargo build --release

# Step 2: Install ffmpeg in a separate stage
FROM debian:stable-slim AS ffmpeg-installer
RUN apt-get update \
    && apt-get install -y --no-install-recommends ffmpeg \
    && rm -rf /var/lib/apt/lists/*

# Step 3: Copy the binary and ffmpeg to the final image
FROM gcr.io/distroless/cc-debian12
WORKDIR /app

COPY --from=builder /app/target/release/media-vault .

COPY --from=ffmpeg-installer /usr/bin/ffmpeg /usr/bin/ffmpeg
COPY --from=ffmpeg-installer /usr/lib/x86_64-linux-gnu /usr/lib/x86_64-linux-gnu
COPY --from=ffmpeg-installer /usr/lib/x86_64-linux-gnu/libblas.so.3 /usr/lib/x86_64-linux-gnu/libblas.so.3
COPY --from=ffmpeg-installer /usr/lib/x86_64-linux-gnu/liblapack.so.3 /usr/lib/x86_64-linux-gnu/liblapack.so.3

COPY config.yaml .

CMD ["./media-vault"]

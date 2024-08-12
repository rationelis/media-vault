# Step 1: Build the project
FROM rust:latest AS builder
WORKDIR /app

RUN mkdir -p in out

COPY Cargo.toml .
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release

COPY src src
RUN touch src/main.rs
RUN cargo build --release

# Step 2: Copy the binary to a new image
FROM gcr.io/distroless/cc-debian12
WORKDIR /app

COPY --from=builder /app/target/release/media-vault .

CMD ["./media-vault"]

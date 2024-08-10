# Step 1: Build the project
FROM rust:latest AS builder
WORKDIR /app

COPY Cargo.toml .
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release

COPY src src
RUN touch src/main.rs
RUN cargo build --release

#RUN strip target/release/media_vault

# Step 2: Run the project
FROM gcr.io/distroless/cc-debian12
WORKDIR /app

COPY --from=builder /app/target/release/media-vault .

CMD ["./media-vault"]

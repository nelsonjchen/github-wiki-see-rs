FROM rust:1.82.0 AS chef
# We only pay the installation cost once,
# it will be cached from the second build onwards
RUN cargo install cargo-chef
WORKDIR app

FROM chef AS planner
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --bin github-wiki-see

# We do not need the Rust toolchain to run the binary!
FROM gcr.io/distroless/cc-debian12 AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/github-wiki-see /usr/local/bin/github-wiki-see

ENV ROCKET_ADDRESS=0.0.0.0

CMD ["/usr/local/bin/github-wiki-see"]

# Build Stage
FROM rust:slim-bullseye AS builder
WORKDIR /builder

RUN cargo init
# Compile deps in a separate layer (for caching)
COPY Cargo.toml Cargo.lock ./
RUN apt-get update
RUN apt install -y pkg-config libssl-dev
RUN cargo build --release

# Compile for release
COPY ./src ./src
COPY ./migrations ./migrations
RUN rm ./target/release/deps/root*
RUN cargo build --release

# Release Stage
FROM debian:bullseye-slim AS release
COPY --from=builder /builder/target/release/root /usr/local/bin
CMD ["/usr/local/bin/root"]

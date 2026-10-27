# Railway deploys the public, non-signing policy API only.
# IMPORTANT: Do not copy wallet databases, seed material, or local key files.
FROM rust:1.95-bookworm AS builder

WORKDIR /usr/src/leaklens
RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev libsqlite3-dev protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates ./crates

# Build the API executable only: no signing or broadcasting executable is shipped.
ENV CARGO_BUILD_JOBS=2
RUN cargo build --locked --release --package leaklens-api --bin leaklens-api

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 libsqlite3-0 \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --system leaklens \
    && useradd --system --gid leaklens --home /nonexistent --no-create-home leaklens

COPY --from=builder /usr/src/leaklens/target/release/leaklens-api /usr/local/bin/leaklens-api

USER leaklens
ENV PORT=8080
ENV RUST_LOG=info
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/leaklens-api"]

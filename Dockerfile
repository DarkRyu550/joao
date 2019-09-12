FROM rustlang/rust:nightly AS builder

# Create dummy project with our dependencies
WORKDIR /milo-build
RUN USER=root cargo new milo
COPY Cargo.toml Cargo.lock /milo-build/milo/

# Build it - this enables docker to cache built dependencies
WORKDIR /milo-build/milo
RUN cargo build

# Overwrite with our source and do the final build
RUN rm -rf target/release/joao \
           target/release/deps/joao-* \
           target/release/.fingerprint/joao-* \
           target/release/incremental/joao-* \
           target/release/joao.d
COPY src src
RUN cargo build

FROM debian:stretch-slim

WORKDIR /milo

COPY --from=builder /milo-build/milo/target/release/joao /bin/joao

CMD ["joao"]

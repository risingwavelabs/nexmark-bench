FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /nexmark-server
COPY rust-toolchain rust-toolchain
RUN rustup show
RUN rustup default `rustup show active-toolchain | awk '{print $1}'`

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
RUN apt-get update && apt-get -y install cmake protobuf-compiler
COPY --from=planner /nexmark-server/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo install --path .

FROM ubuntu:22.04 as nexmark-bench
RUN apt-get update && DEBIAN_FRONTEND=noninteractive apt-get -y install ca-certificates && rm -rf /var/lib/{apt,dpkg,cache,log}/
RUN mkdir -p /nexmark-bench/bin
COPY --from=builder /usr/local/cargo/bin/nexmark-server /nexmark-bench/bin/nexmark-server

ENV KAFKA_HOST="127.0.0.1:9092"
ENV BASE_TOPIC="nexmark-events"
ENV AUCTION_TOPIC="nexmark-auction"
ENV BID_TOPIC="nexmark-bid"
ENV PERSON_TOPIC="nexmark-person"
ENV NUM_PARTITIONS=3
ENV SEPARATE_TOPICS=true
ENV RUST_LOG="nexmark_server=info"

ENTRYPOINT [ "/nexmark-bench/bin/nexmark-server" ]
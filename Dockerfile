FROM rust AS planner 
WORKDIR /nexmark-server
RUN cargo install cargo-chef 
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM rust AS cacher 
WORKDIR /nexmark-server
RUN cargo install cargo-chef
COPY --from=planner /nexmark-server/recipe.json recipe.json
RUN apt-get update && apt-get -y install cmake protobuf-compiler
RUN cargo chef cook --recipe-path recipe.json

FROM rust:latest AS builder
COPY . ./nexmark-server
WORKDIR /nexmark-server
COPY --from=cacher /nexmark-server/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
RUN apt-get update && apt-get -y install cmake protobuf-compiler
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
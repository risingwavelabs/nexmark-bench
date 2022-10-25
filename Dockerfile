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

FROM rust AS builder
COPY . ./nexmark-server
WORKDIR /nexmark-server
COPY --from=cacher /nexmark-server/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
RUN apt-get update && apt-get -y install cmake protobuf-compiler
RUN cargo install --path .

FROM debian:buster-slim
RUN apt-get update && apt install -y openssl && apt install -y libssl-dev && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/nexmark-server /usr/local/bin/nexmark-server


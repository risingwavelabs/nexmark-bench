PROJECT_DIR=$(shell pwd)

setup-local: install
	docker compose -f services.yml up kafka1 zoo1 kafka-ui

setup-docker:
	docker compose -f services.yml up

setup-docker-build: 
	docker compose -f services.yml up --build

rust_check_all: 
	$(MAKE) rust_fmt_check && $(MAKE) rust_clippy_check && $(MAKE) rust_cargo_sort_check

rust_fmt_check:
	cargo fmt --all -- --check

rust_clippy_check_locked:
	cargo clippy --all-targets --locked -- -D warnings

rust_clippy_check:
	cargo clippy --all-targets -- -D warnings

rust_cargo_sort_check:
	cargo sort -c -w

install:
	cargo install --path .

docker_build:
	docker build -f Dockerfile -t nexmark-bench:latest .

rust_build:
	cargo build

rust_build_release:
	cargo build --release 

rust_test:
	RUSTFLAGS=-Dwarnings cargo test
PROJECT_DIR=$(shell pwd)

setup-local: install
	docker compose -f services.yml up kafka1 zoo1 kafka-ui

setup-docker:
	docker compose -f services.yml up

setup-docker-build: 
	docker compose -f services.yml up --build

install:
	cargo install --path .

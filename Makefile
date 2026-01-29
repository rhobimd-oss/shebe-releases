MAKEFLAGS += --warn-undefined-variables
SHELL := bash
.SHELLFLAGS := -o errexit -o nounset -o pipefail -c
.DEFAULT_GOAL := build
.DELETE_ON_ERROR:
.SUFFIXES:

PROJECT_DIR ?= $(PWD)
COMPOSE := docker compose --file ${PROJECT_DIR}/deploy/docker-compose.yml run --rm
RUN_DEV := $(COMPOSE) zed-ext-dev

# Zed Extension Build Targets ------------------------------------------------

build:
	@echo "Building WASM extension in container..."
	$(RUN_DEV) bash -c \
		"rustup target add wasm32-wasip1 && cargo build --release --target wasm32-wasip1"

check:
	@echo "Running cargo check in container..."
	$(RUN_DEV) bash -c \
		"rustup target add wasm32-wasip1 && cargo check --target wasm32-wasip1"

fmt:
	@echo "Formatting code in container..."
	$(RUN_DEV) cargo fmt

fmt-check:
	@echo "Checking formatting in container..."
	$(RUN_DEV) cargo fmt -- --check --verbose

clippy:
	@echo "Running clippy in container..."
	$(RUN_DEV) bash -c \
		"rustup target add wasm32-wasip1 && cargo clippy --target wasm32-wasip1 --no-deps -- -D warnings"

ci: fmt-check clippy build
	@echo "CI checks complete"

shell:
	@echo "Starting interactive shell in container..."
	cd deploy && docker compose run --rm zed-ext-dev bash

clean:
	@echo "Cleaning Docker volumes..."
	docker volume rm deploy_cargo-registry deploy_cargo-git deploy_cargo-target 2>/dev/null || true
	@echo "Docker volumes cleaned"

# Help ------------------------------------------------------------------------

help:
	@echo "Shebe Releases - Zed Extension Makefile"
	@echo ""
	@echo "Build Targets:"
	@echo "  build      Build WASM extension (release)"
	@echo "  check      Run cargo check for wasm32-wasip1"
	@echo "  fmt        Format code"
	@echo "  fmt-check  Check code formatting"
	@echo "  clippy     Run clippy linter"
	@echo "  ci         Run all CI checks (fmt-check, clippy, build)"
	@echo "  shell      Open interactive shell in dev container"
	@echo "  clean      Clean Docker volumes"

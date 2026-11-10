.DEFAULT_GOAL := help
.PHONY: help check fmt build install clean

help: ## List the targets
	@grep -E '^[a-z]+:.*## ' $(MAKEFILE_LIST) | awk -F ':.*## ' '{ printf "  %-8s %s\n", $$1, $$2 }'

check: ## Format check, clippy and tests, as the CI runs them
	cargo fmt --check
	cargo clippy --all-targets -- -D warnings
	cargo test

fmt: ## Format the code
	cargo fmt

build: ## Build the release binary and package claude.alfredworkflow
	bash scripts/build-workflow.sh

install: ## Build the package and open it in Alfred
	bash scripts/build-workflow.sh --install

clean: ## Remove build outputs
	cargo clean
	rm -rf build claude.alfredworkflow

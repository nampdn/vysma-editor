# Vysma Development Commands
# Use `just <command>` to run these recipes

# Build commands
build: ## Build current crate
	cargo build

build-all: ## Build entire workspace
	cargo build --workspace

build-http: ## Build with HTTP assets feature
	cargo build --workspace --features "vysma-app/http_assets"

# Development server/client commands
serve: ## Run desktop server with auto-discovery
	cargo run -p vysma -- serve

serve-nogui: ## Run server without GUI
	cargo run -p vysma -- serve --gui false

serve-quick: ## Run server with timeout (for testing)
	timeout 10s cargo run -p vysma -- serve || true

client: ## Run desktop client with auto-discovery
	cargo run -p vysma -- client --gui

client-nogui: ## Run client without GUI
	cargo run -p vysma -- client --gui false

client-quick: ## Run client with timeout (for testing)
	timeout 8s cargo run -p vysma -- client --gui || true

host: ## Run host-client (server + client)
	cargo run -p vysma -- host

host-quick: ## Run host with timeout (for testing)
	timeout 15s cargo run -p vysma -- host || true

# Project scaffolding
new name: ## Create new basic project
	vysma new {{name}}

new-editor name: ## Create new editor-as-game project
	vysma new --template editor_game {{name}}

new-editor-f name: ## Force create editor project (overwrite existing)
	vysma new --template editor_game --overwrite {{name}}

# CLI management
install-vysma: ## Install vysma CLI globally
	cargo install --path crates/vysma

install-vysma-local: ## Install vysma CLI locally (for development)
	cargo install --path crates/vysma --force

uninstall-vysma: ## Uninstall vysma CLI
	cargo uninstall vysma

# Module publishing
module-publish-dry owner name version hcl assets: ## Dry-run module publish
	vysma module publish --owner {{owner}} --name {{name}} --version {{version}} --hcl {{hcl}} --assets {{assets}} --dry-run

# Generic CLI passthrough
vysma +args: ## Pass arguments to vysma CLI
	cargo run -p vysma -- {{args}}

# Preview and testing
preview: ## Build and run browser preview
	vysma preview --open

# Development utilities
clean: ## Clean build artifacts
	cargo clean

check: ## Check code without building
	cargo check --workspace

test: ## Run tests
	cargo test --workspace

fmt: ## Format code
	cargo fmt

clippy: ## Run clippy linter
	cargo clippy --workspace

# Quick development workflow
dev-setup: ## Setup new development environment
	just install-vysma-local
	just new demo
	cd demo
	just serve

dev-test: ## Test the development workflow
	just new test-project
	cd test-project
	just serve-quick
	just client-quick

# Help
help: ## Show this help
	@just --list
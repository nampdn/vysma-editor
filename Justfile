set shell := ["/bin/sh", "-cu"]

# Build current crate
build:
	cargo build

# Build entire workspace
build-all:
	cargo build --workspace

# Run desktop server
serve:
	cargo run -p vysma -- serve --gui

# Run desktop server with timeout (default 10s)
serve-quick secs="10":
	( cargo run -p vysma -- serve --gui & pid=$!; sleep {{secs}}; kill $pid >/dev/null 2>&1 || true )

# Run desktop client
client:
	cargo run -p vysma -- client --gui

# Run desktop client with timeout (default 10s)
client-quick secs="10":
	( cargo run -p vysma -- client --gui & pid=$!; sleep {{secs}}; kill $pid >/dev/null 2>&1 || true )

# Run desktop server (no GUI)
serve-nogui:
	cargo run -p vysma -- serve

# Run desktop client (no GUI)
client-nogui:
	cargo run -p vysma -- client

# Run client+server
host:
	( cargo run -p vysma -- serve --gui & ) ; sleep 2 ; cargo run -p vysma -- client --gui

# Run client+server with timeout (default 12s)
host-quick secs="12":
	( cargo run -p vysma -- serve --gui & sp=$!; sleep 2; cargo run -p vysma -- client --gui & cp=$!; sleep {{secs}}; kill $sp >/dev/null 2>&1 || true; kill $cp >/dev/null 2>&1 || true )

# Build with HTTP assets feature
build-http:
	cargo build --workspace --features http_assets

# CLI: generic passthrough to vysma
vysma +args:
	cargo run -p vysma -- {{args}}

# CLI: preview (browser stub)
preview:
	cargo run -p vysma -- preview --open

# CLI: dry-run publish manifest (assets: path or '-')
module-publish-dry owner name version hcl assets:
	if [ "{{assets}}" = "-" ]; then \
		cargo run -p vysma -- module publish --dry-run --owner {{owner}} --name {{name}} --version {{version}} --hcl {{hcl}} ; \
	else \
		cargo run -p vysma -- module publish --dry-run --owner {{owner}} --name {{name}} --version {{version}} --hcl {{hcl}} --assets {{assets}} ; \
	fi

# CLI: real publish (assets: path or '-')
module-publish owner name version hcl assets:
	if [ "{{assets}}" = "-" ]; then \
		cargo run -p vysma -- module publish --owner {{owner}} --name {{name}} --version {{version}} --hcl {{hcl}} ; \
	else \
		cargo run -p vysma -- module publish --owner {{owner}} --name {{name}} --version {{version}} --hcl {{hcl}} --assets {{assets}} ; \
	fi

# New project scaffold
new name:
	cargo run -p vysma -- new {{name}}

# New project scaffold with template (basic or editor_game)
new-editor name:
	cargo run -p vysma -- new --template editor_game {{name}}

# Force new editor project (overwrite)
new-editor-f name:
	cargo run -p vysma -- new --template editor_game --overwrite {{name}}

# Ensure schema (placeholder)
ensure-schema:
	cargo run -p vysma -- ensure-schema

# Verify (placeholder)
verify:
	cargo run -p vysma -- verify 

# Build only CLI crate
build-cli:
	cargo build -p vysma

# Tests
test:
	cargo test

test-all:
	cargo test --workspace

# Quick typecheck
check:
	cargo check --workspace

# Install vysma CLI into ~/.cargo/bin (global on PATH)
install-vysma:
	cargo install --path crates/vysma --force
	(which vysma && vysma --help >/dev/null 2>&1 && echo "Installed: $(which vysma)") || echo "Note: add \"~/.cargo/bin\" to your PATH"

# Install using local release build (alternative fast reinstall)
install-vysma-local:
	cargo build -p vysma --release
	mkdir -p $$HOME/.cargo/bin
	cp target/release/vysma $$HOME/.cargo/bin/vysma
	(which vysma && echo "Installed: $(which vysma)") || echo "Note: add \"~/.cargo/bin\" to your PATH"

# Uninstall vysma CLI
uninstall-vysma:
	cargo uninstall vysma || true
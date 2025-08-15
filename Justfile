set shell := ["/bin/sh", "-cu"]

# Build current crate
build:
	cargo build

# Build entire workspace
build-all:
	cargo build --workspace

# Run desktop server
serve:
	cargo sv

# Run desktop client
client:
	cargo cl

# Run client+server
host:
	cargo hc

# Build with HTTP assets feature
build-http:
	cargo build --workspace --features http_assets

# CLI: dry-run publish manifest
publish-dry module name version root:
	cargo run -p vysma -- module publish --dry-run --module {{module}} --version {{name}}-{{version}} --root {{root}}

# New project scaffold
new name:
	cargo run -p vysma -- new {{name}}

# Ensure schema (placeholder)
ensure-schema:
	cargo run -p vysma -- ensure-schema

# Verify (placeholder)
verify:
	cargo run -p vysma -- verify 
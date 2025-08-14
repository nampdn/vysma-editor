default:
	just --list

# Build workspace (default features)
build:
	cargo build --workspace

# Build with full features for desktop testing
build-all:
	cargo build --workspace --all-features

# Run local server (headless)
serve:
	cargo run -- server

# Run client with id=1
client:
	cargo run -- client -c 1

# Publish module (example; requires env)
publish owner name version hcl assets:
	cargo run -p vysma -- module publish --owner {{owner}} --name {{name}} --version {{version}} --hcl {{hcl}} --assets {{assets}}

# Dry-run publish to compute manifest only
publish-dry owner name version hcl assets:
	cargo run -p vysma -- module publish --dry-run --owner {{owner}} --name {{name}} --version {{version}} --hcl {{hcl}} --assets {{assets}}

# Ensure Appwrite schema (requires env)
ensure-schema:
	cargo run -p vysma -- ensure-schema

# Verify env presence
verify:
	cargo run -p vysma -- verify

# Bootstrap a new game repo in ./sandbox/<name>
new name:
	mkdir -p sandbox
	cargo run -p vysma -- new sandbox/{{name}}

# End-to-end local iteration: serve + client (two terminals recommended)
e2e:
	@echo "Run 'just serve' in one terminal and 'just client' in another" 
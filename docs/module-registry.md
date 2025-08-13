### Community Module Registry Spec

Goal: allow developers to publish reusable HCL modules and import them by `username::module_name[@version]` with a human-friendly workflow.

---

### Naming and resolution
- Import syntax: `modules = [{ name = "username::module", alias = "mod", version = "0.1.0"? }]`
- Version:
  - If omitted, resolve to module.latestVersion.
  - Semantic version strings recommended; no constraints enforced initially.
- Namespace:
  - `alias` is required; we use `alias::` prefix for all imported names and assets.

### Data model (Appwrite)
- Collections
  - Modules: { id, ownerUserId, ownerUsername, name, latestVersion, visibility: "public"|"private", description?, tags? }
  - ModuleVersions: { id, moduleId, version, sha256, hcl (string), manifest (json), createdAt (datetime) }
  - ModuleAssetsIndex: { id, moduleVersionId, path, storageFileId, sha256, size, original_path }
- Storage bucket: `module-assets`

### Fetch API (server‑side)
- `GET latest` by (ownerUsername, name) → Module + ModuleVersions.latest
- `GET version` by (ownerUsername, name, version) → ModuleVersion
- Manifest used to map asset `file` fields to URL paths

### Merge rules
- Prefabs: include those listed in module's `exports` (if omitted, include all in MVP) → rename to `alias::PrefabName`.
- Entities: rename `name` to `alias::Name` when present.
- Triggers: rename trigger `name` with alias prefix.
- Vars: rename to `alias::var`.
- Assets: rename logical asset names with `alias::` prefix; HCL keeps name references.

### Publishing workflow (CLI)
- Command: `module publish`
  - Args: `--owner <username> --name <module> --version <v> --hcl <file> [--assets <dir>] [--visibility public|private] [--desc <text>]`
  - Steps:
    1) Read HCL; compute SHA256.
    2) Create module if missing (id = `owner__name`); set `latestVersion` if requested.
    3) Create module version with HCL and sha (id = `owner__name__version`).
    4) Upload assets under deterministic keys `owner/name/<sha256>.ext` to `module-assets` bucket (idempotent; 409 → skip).
    5) Build a manifest array and persist into `ModuleVersions.manifest`.
    6) Optionally index assets in `ModuleAssetsIndex` for search.
    7) Update Modules.latestVersion when `--set-latest` is enabled (default true).
  - Output: IDs and URLs; print recommended import line and resolved assets summary.

CLI crate: `vysma`

Bootstrap and dev workflow:
- `vysma new mygame` → creates a repo with `assets/`, example HCL, and cargo workspace wired
- `vysma serve` → runs the server in watch mode with hot‑reload
- `vysma client` → runs a local client connected to the dev server
- `vysma publish` → publishes module or scene with deduped hashed assets and manifest

Example:
```
cargo run -p vysma -- module publish \
  --owner alice --name axe --version 0.1.0 \
  --hcl assets/moba_hcl/heroes/axe.hcl \
  --assets assets/mesh/heroes
```

### Server integration (runtime)
- During spawn/hot‑reload:
  - For each `ModuleImport` without `path` and `name` contains `::`: fetch ModuleVersion.hcl.
  - Parse to `SceneDoc` and call `merge_module_import` with alias.
  - Keep the `manifest` in memory for asset URL mapping.
  - Cache resolved docs by `username::module@version` (pending in‑memory cache).

### Security and visibility
- Only public modules are resolved by default.
- Private modules require project membership (future enhancement).

### Checklist
- [x] Resolve `username::module` imports via Appwrite
- [x] Namespacing applied to prefabs/entities/triggers/vars/assets
- [ ] Module cache with invalidation by sha
- [x] CLI: create module
- [x] CLI: publish version (assets upload + manifest)
- [ ] CLI: new/serve/client bootstrap commands
- [ ] Docs: authoring guidance and examples

### Authoring example
```hcl
modules = [
  { name = "alice::moba_core", alias = "core" },
  { name = "alice::axe", alias = "axe", version = "0.1.0" }
]

entity "root" {
  children = [
    { name = "Game", include = ["core::BaseGame"], components = { Transform = { t = [0,0,0] } } },
    { name = "HeroAxe", include = ["axe::Axe"], components = { Transform = { t = [2,0,0] } } }
  ]
}
``` 
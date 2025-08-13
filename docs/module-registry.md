### Community Module Registry Spec

Goal: allow developers to publish reusable HCL modules and import them by `username::module_name[@version]`.

---

### Naming and resolution
- Import syntax: `modules = [{ name = "username::module", alias = "mod", version = "0.1.0"? }]`
- Version:
  - If omitted, resolve to module.latestVersion.
  - Semantic version strings recommended; no constraints enforced initially.
- Namespace:
  - `alias` is required; we use `alias::` prefix for all imported names.
  - If `alias` omitted, default to `username::module` (longer, discouraged).

### Data model (Appwrite)
- Collections
  - Modules: { id, ownerUserId, ownerUsername, name, latestVersion, visibility: "public"|"private", description?, tags? }
  - ModuleVersions: { id, moduleId, version, sha256, hcl (string), createdAt (datetime) }
  - ModuleAssetsIndex: { id, moduleVersionId, path, storageFileId, sha256, size }
- Storage bucket: `module-assets`

### Fetch API (server‑side)
- `GET latest` by (ownerUsername, name) → Module + ModuleVersions.latest
- `GET version` by (ownerUsername, name, version) → ModuleVersion
- `LIST assets` by moduleVersionId → list of asset refs with URLs

Runtime implementation:
- Uses `unofficial_appwrite` Rust SDK with queries: `equal(ownerUsername, ..)`, `equal(name, ..)`, `orderDesc($createdAt)`, `limit(1)`.

### Merge rules
- Prefabs: include those listed in module's `exports` (if omitted, include all in MVP) → rename to `alias::PrefabName`.
- Entities: rename `name` to `alias::Name` when present.
- Triggers: rename trigger `name` with alias prefix.
- Vars: rename to `alias::var`.
- Assets: rename logical asset names with `alias::` prefix; keep URLs unchanged.

### Publishing workflow (CLI)
- Command: `module publish`
  - Args: `--owner <username> --name <module> --version <v> --hcl <file> [--assets <dir>] [--visibility public|private] [--desc <text>]`
  - Steps:
    1) Read HCL; compute SHA256.
    2) Create module if missing; else validate ownership.
    3) Create module version with HCL and sha.
    4) Optionally upload assets under deterministic keys `username/module/version/<relpath>` to `module-assets` bucket.
    5) Update Modules.latestVersion if desired.
  - Output: IDs and URLs; print next import line.

Status:
- First pass available via `scripts/populate_modules.rs` (creates required attributes, module, version). Assets upload deferred.

### Server integration (runtime)
- During spawn/hot‑reload:
  - For each `ModuleImport` without `path` and `name` contains `::`: fetch ModuleVersion.hcl.
  - Parse to `SceneDoc` and call `merge_module_import` with alias.
  - Cache resolved docs by `username::module@version` (pending in‑memory cache).

### Security and visibility
- Only public modules are resolved by default.
- Private modules require project membership (future enhancement).

### Checklist
- [x] Resolve `username::module` imports via Appwrite
- [x] Namespacing applied to prefabs/entities/triggers/vars/assets
- [ ] Module cache with invalidation by sha
- [x] CLI: create module
- [x] CLI: publish version (assets later)
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
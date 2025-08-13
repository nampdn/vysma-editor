### HTTP Asset IO Spec (MVP+)

Goal: load images and glTF scenes via http(s) URLs using Bevy's `AssetServer` on native and wasm.

---

### Approach
- Provide a custom `HttpAssetIo` that delegates:
  - If path starts with `http://` or `https://` → fetch bytes via `reqwest` (native) or `wasm-bindgen` fetch (wasm), return a virtual reader.
  - Otherwise delegate to default `FileAssetIo`.
- Integrate by setting `AssetPlugin` source with our chained IO on startup.
- In-memory cache keyed by URL; optional disk cache later.

### Storage keying (CLI publish)
- To avoid duplicates and enable content-addressable fetching, assets are uploaded under hashed keys:
  - `owner/name/<sha256>` with `fileId = owner__name__<sha256>`
  - `ModuleAssetsIndex` stores: { moduleVersionId, path: `owner/name/<sha256>`, original_path, storageFileId, sha256, size }
- This ensures deduplication across versions and stable URLs for remote fetch.

### HCL authoring
- Images and glTF assets can specify `url` instead of `file`:
```hcl
assets {
  image "tex" { url = "https://storage.example.com/owner/name/<sha256>.png" }
  gltf  "hero" { url = "https://storage.example.com/owner/name/<sha256>.glb", node = "Scene0" }
}
```

### Implementation notes
- Caching:
  - MVP: in‑memory per‑run cache keyed by URL (native). Disk cache later.
- Content types:
  - Bevy uses extension to pick loaders. Preserve the extension in virtual path mapping if needed; otherwise loaders using magic sniffing may still work for some formats. We prefer URL with original extension when available.
- Errors:
  - Propagate HTTP status; log and fall back to placeholder assets if needed.

### Integration
- Feature flag: `http_assets` (in `vysma-hcl`).
- In `new_gui_app` and headless server, when feature is enabled:
  - Register `HttpAssetIoPlugin` to override default reader.

### Checklist
- [x] Implement `HttpAssetIo` (native, in-memory cache)
- [x] Plug into app startup behind feature flag
- [ ] Validate image and glTF loads from Appwrite storage URLs
- [ ] Add simple disk cache to avoid repeated downloads across runs 
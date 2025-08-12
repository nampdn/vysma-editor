### HTTP Asset IO Spec (MVP+)

Goal: load images and glTF scenes via http(s) URLs using Bevy's `AssetServer` on native and wasm.

---

### Approach
- Provide a custom `HttpAssetIo` that delegates:
  - If path starts with `http://` or `https://` → fetch bytes via `reqwest` (native) or `wasm-bindgen` fetch (wasm), return a virtual reader.
  - Otherwise delegate to default `FileAssetIo`.
- Integrate by setting `AssetPlugin` source with our chained IO on startup.

### HCL authoring
- Images and glTF assets can specify `url` instead of `file`:
```hcl
assets {
  image "tex" { url = "https://cdn.example.com/tex.png" }
  gltf  "hero" { url = "https://cdn.example.com/hero.glb", node = "Scene0" }
}
```

### Implementation notes
- Caching:
  - MVP: in‑memory per‑run cache keyed by URL and ETag/Last‑Modified if available.
  - Optional: disk cache under `~/.vysma/cache/` on native.
- Content types:
  - Bevy uses extension to pick loaders. Preserve the extension in virtual path mapping, e.g., `virtual://http/hero.glb` so the glTF loader triggers.
- Errors:
  - Propagate HTTP status; log and fall back to placeholder assets if needed.

### Integration
- Feature flag: `http_assets`.
- In `new_gui_app` and headless server, when feature is enabled:
  - Wrap default `AssetIo` with `HttpAssetIo` and set on `AssetPlugin`.

### Checklist
- [ ] Implement `HttpAssetIo` (native + wasm)
- [ ] Plug into app startup behind feature flag
- [ ] Validate image and glTF loads from Appwrite storage URLs
- [ ] Add simple memory cache to avoid repeated downloads 
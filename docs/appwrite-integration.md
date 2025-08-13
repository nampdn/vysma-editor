### Appwrite Integration Spec (MVP)

We use Appwrite for identity, database, and storage to back projects, scenes, and module registry.

Crate: `unofficial_appwrite` (server and CLI). Clients (editor) do not use server keys.

---

### Data model

Collections (Databases):
- Projects: { id, ownerUserId, ownerUsername, name, createdAt }
- Scenes: { id, projectId, name, publishedVersionId?, createdAt }
- SceneVersions: { id, sceneId, sha256, hcl (string), authorUserId, createdAt (datetime) }
- Modules: { id, ownerUserId, ownerUsername, name, latestVersion, visibility: "public"|"private", description?, tags? }
- ModuleVersions: { id, moduleId, version, sha256, hcl (string), manifest (json), createdAt (datetime) }
- ModuleAssetsIndex: { id, moduleVersionId, path, storageFileId, sha256, size, original_path, content_type? }

Storage buckets:
- `module-assets` (public or signed URLs)

Notes
- `ModuleVersions.manifest` captures per-version assets: [{ original_path, sha256, size, url_path, content_type? }]
- `url_path` is a relative path like `owner/name/<sha256>.ext` (base URL resolved on client)

---

### API wrapper (`cloud::appwrite_client`)
- Config: `AppwriteConfig { endpoint, project_id, api_key }` (endpoint normalized to include `/v1`)
- Client: initializes SDK objects (Databases); uses query helpers (equal/orderDesc/limit)
- Module APIs (MVP):
  - `get_module_latest(username, name)` → ModuleVersion
  - `get_module_version(username, name, version)` → ModuleVersion
- Scene APIs (MVP+):
  - `get_published_scene(project_id)` → SceneVersion
  - `create_scene_version(scene_id, hcl, sha, author)` → SceneVersion
  - `publish_scene(scene_id, version_id)` → update Scenes.publishedVersionId
- Upload (CLI):
  - `upload_asset(bucket, local_path, dst_key, file_id)` → returns file id and size

All functions return `anyhow::Result<T>`.

---

### Server startup flow (optional MVP+)
1) Read env: `APPWRITE_ENDPOINT`, `APPWRITE_PROJECT_ID`, `APPWRITE_API_KEY`, `VYSMA_PROJECT_ID`, `VYSMA_SCENE_ID`.
2) Fetch `get_published_scene(project_id)`; if present, set `HclSource { content, sha, path = Some("project://<id>/active") }`.
3) Publish `HclSceneBlob` and trigger spawn.

### Editor update flow (MVP)
1) Editor sends `HclUpdateRequest { path?, content, sha256, jwt? }`.
2) Server validates JWT (if required) and project membership.
3) Parse → success → set `HclEntry` and `HclSource`.
4) (MVP+) Persist: create SceneVersion and set as published.
5) Republish `HclSceneBlob { content, sha }` to clients.

### Module resolve flow (MVP)
1) At spawn/hot‑reload, for each `ModuleImport` with `name` containing `::` and empty `path`:
   - If `version` missing → fetch latest; else fetch that version.
   - Parse ModuleVersion.hcl → `SceneDoc`.
   - Merge into working doc with alias namespacing.
2) If `manifest` present → keep it in memory for URL resolution of `file` fields to CDN paths.
3) Cache results keyed by `username::module@version`.

### Asset publish flow (CLI)
1) For each referenced local file path (from HCL):
   - Read bytes → compute sha256.
   - Derive `file_id` = first 32 hex chars (Appwrite limit) and `url_path` = `owner/name/<sha256>.<ext>`.
   - Upload to storage bucket if missing (`409` → skip).
   - Create `ModuleAssetsIndex` row with { moduleVersionId, path = url_path, storageFileId = file_id, sha256, size, original_path, content_type? } (optional).
2) Create `ModuleVersions` doc with embedded `manifest` array for this version.

Headers
- Requests include: `X-Appwrite-Project`, `X-Appwrite-Key`, `X-Appwrite-Response-Format: 1.7.0`

---

### Security
- Server uses API key; never shipped to clients.
- Editor JWT (optional in MVP) is verified on server via Appwrite Accounts/JWKS.
- Public modules resolved without auth; private modules require project membership (future).

---

### Checklist
- [x] Add `cloud::appwrite_client` (read‑only module fetch)
- [x] Env/config resource and initialization
- [x] Wire resolve into module loader/spawn
- [x] CLI: publish module (create module/version + manifest; hashed assets upload)
- [ ] (MVP+) Persist scene on update; load on startup
- [ ] (MVP+) Private module auth path

---

### Env/CLI
- Server env: `APPWRITE_ENDPOINT`, `APPWRITE_PROJECT_ID`, `APPWRITE_API_KEY`, `VYSMA_PROJECT_ID`, `VYSMA_SCENE_ID`
- CLI flags mirror env for local publishing. 
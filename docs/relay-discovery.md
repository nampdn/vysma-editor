### Relay and Discovery Spec (MVP)

Goal: Enable Expo‑style "connect by URL" across LAN and Internet so any client (desktop/mobile/wasm) mirrors live HCL from a development or SaaS server.

---

### Components
- Discovery (LAN): mDNS broadcast of project metadata
- Relay (Internet): Minimal WebSocket relay to traverse NAT/firewalls
- Tokens: Short‑lived connection tokens tied to a project code
- URL scheme: Human‑friendly project URLs

---

### LAN Discovery
- mDNS service name: `_vysma._udp.local`
- TXT records: `{ project=<uuid>, name=<string>, mode=dev|saas, port=<udp/quic>, ws_port=<ws> }`
- Client behavior: if `--connect lan`, scan for services and present list; auto‑select when only one

### Internet Relay
- Service: `relay.vysma.dev` (configurable via `VYSMA_RELAY_URL`)
- Transport: WebSocket (wss) that forwards Lightyear frames between server and clients
- Registration: server opens `wss://relay/.../register` with `project_code` and receives a signed `register_token`
- Connect: clients open `wss://relay/.../connect/<project_code>?token=<client_token>`
- Backpressure: drop oldest frames beyond N; heartbeat pings to measure RTT

### URL Scheme
- Short project URL: `vysma.dev/<project_code>`
  - Resolves to relay connect endpoint via HTTPS page that bootstraps the ws URL for wasm clients
- Direct ws URL: `wss://relay.vysma.dev/connect/<project_code>?token=<client_token>`

### Security
- Server obtains a `register_token` after authenticating CLI/session (Dev) or via SaaS console (Prod)
- Clients receive `client_token` from server or CLI (scoped, short TTL, single‑use recommended)
- No privileged commands over relay; editor Apply requires JWT verified on the authoritative server
- Rate limits per project_code to mitigate abuse

### Server API (conceptual)
- `POST /projects/:id/register` → { project_code, register_token, ws_url }
- `POST /projects/:id/tokens` → { client_token, expires_at }
- `WS /register` — upsert server presence for a project_code
- `WS /connect/:project_code` — client attaches; relay pairs with an active server

### Client/Server Behavior
- Server
  - On start: if relay configured, register and print URL
  - Publish `HclSceneBlob` diffs or full state when content changes
- Client
  - On connect: request latest snapshot then apply deltas
  - Reconnect with exponential backoff

### Acceptance Criteria
- With relay disabled: clients on LAN discover and connect via mDNS and receive live updates
- With relay enabled: a remote client connects via short URL and mirrors changes in <300ms on good links
- Security tests: tokens expire; unauthorized Apply requests rejected at server

### Configuration
- Server env: `VYSMA_RELAY_URL`, `VYSMA_RELAY_ENABLE=true`, `VYSMA_PROJECT_CODE?`
- CLI: `vysma serve --public` toggles relay registration; prints URL
- Client: `vysma client --connect <lan|url|ws(s)://...>`

### Future Enhancements
- NAT punching for QUIC (hole punching) to bypass relay when possible
- Server selection UI when multiple dev servers are discovered
- Metrics channel for bandwidth/latency reporting 
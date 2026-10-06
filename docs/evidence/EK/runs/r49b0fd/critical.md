### F-01 — Shared agent UID across tenants breaks workspace isolation
- Topics: BL-06, EN-01.4, ES-03.1
- Evidence: ADR-019 addendum acknowledges that per-project UIDs are unbuilt: "the agent uid is shared across tenants... so the contract separates agent from custodian and tenant runtime from host files, never one tenant's agent from another's (`docs/reference/adr/ADR-019-single-pod-multi-project-tenancy.md:694`, `docs/reference/adr/ADR-019-single-pod-multi-project-tenancy.md:705-708`)." BL-06 states: "the agents of both tenants run as the same system account, so a request on one can change the other's source; until each project gets its own account, the two tenants are one trust domain."
- Failure: When multiple tenants reside on the same pod, all agent executions run as UID 1004 (`enclosure/docker/Dockerfile.control-plane:180`). The control plane mounts every tenant workspace volume into the same container (`enclosure/docker/compose.pod.yaml:524-531`), with the writable engineering directories across all tenant workspaces owned by UID 1004 (`enclosure/docker/workspace-seed.sh:115-120`). A prompt executed on behalf of Tenant A can instruct the agent to inspect, modify, or plant malicious code in Tenant B's workspace (`/workspaces/<tenant-b>/...`), compromising Tenant B's code and preview environment.
- Confidence: high
- Marked by authors: yes (ADR-019 addendum; BL-06)

### F-02 — Project registration permits arbitrary cross-tenant credential references
- Topics: BL-06, CP-07.1, CP-08.1, CP-09.1
- Evidence: CP-08.1 notes that nothing binds a secret reference to the project claiming it: "registration admits any reference the vault holds (`control-plane/src/index.ts:3178-3179`) and the registry resolves it as given (`control-plane/src/project/registry.ts:316-317`)." The control plane resolves whatever reference is supplied into runtime handles and writes `<dir>/<id>/env` for the container (`control-plane/src/vault/inject.ts:50-53`).
- Failure: An administrator registering Tenant B can set `supabaseRef` or `tokenRef` to point to Tenant A's existing credential reference (e.g., `tenant-a` or `tenant-a-pat`). The control plane resolves Tenant A's database connection string or GitHub Personal Access Token from the secret broker and writes it into Tenant B's credential file (`/run/campaignbuilder-credentials/<tenant-b>/env`). Tenant B's runtime application and development server then run with full access to Tenant A's production database and source repository.
- Confidence: high
- Marked by authors: yes (Open in CP-08.1 and BL-06)

### F-03 — Flat `pod-net` bridge network allows direct lateral access and bypasses ingress security controls
- Topics: BL-06, EN-02.2, EN-03.1, ES-03.1
- Evidence: EN-02.2 shows all tenant containers, the database, and the control plane share the bridge network `pod-net` (`enclosure/docker/compose.pod.yaml:707`, `enclosure/docker/compose.pod.yaml:555`, `enclosure/docker/compose.pod.yaml:194`). ES-03.1 notes: "every container on the pod network can reach a tenant directly, past the ingress strip; what a tenant's entrypoint hands its application, the application can use... and tenants share one network and have unrestricted outbound access (`docs/reference/adr/ADR-019-single-pod-multi-project-tenancy.md:403-408`)."
- Failure: Ingress strips spoofed identity headers (`X-Auth-Request-*`, `X-Dev-User`, `X-Break-Glass`) on requests arriving from the public internet (`enclosure/docker/Caddyfile:67-78`). However, because tenant containers share `pod-net` with no inter-container network segmentation, code running in any tenant container (or an SSRF payload executed by `next dev`) can bypass Caddy completely and connect directly to another tenant on port 3000, to the pod database on port 5432, or to the control plane, sending forged forward-auth headers without encountering the ingress strip.
- Confidence: high
- Marked by authors: yes (Limitation in ADR-019 and ES-03.1)

### F-04 — Agent-edited code executes immediately in `next dev` with `BYPASSRLS` database credentials
- Topics: BL-01, EN-04.3, ES-03.1, WS-01.1
- Evidence: BL-01 notes: "the agent edits the project workspace... The workspace is the volume the tenant's development server serves, so an edit is visible in the authenticated preview without a rebuild (`enclosure/docker/compose.pod.yaml:182-184`)." In EN-04.3, the tenant runs `next dev` as `surface` (UID 1005). The database role `campaignbuilder_app` is explicitly granted `BYPASSRLS` on every boot (`enclosure/docker/web-surface-entrypoint.sh:342`) because the tenant schema's RLS policies deny all operations unless bypassed (`enclosure/docker/web-surface-entrypoint.sh:43-48`).
- Failure: As soon as the agent edits any file under `app/` or `components/` during an engineering request, `next dev` hot-reloads and executes that code immediately—prior to any test, check, complexity classification, or human approval. Because the running Node process holds `DATABASE_URL` with `BYPASSRLS`, malicious or malformed code generated by the agent can read, truncate, or exfiltrate all tenant database records directly from the preview environment before an operator ever sees the change.
- Confidence: high
- Marked by authors: yes (Threat model noted in ES-03.1)

### F-05 — Deploy capabilities are governed by an unversioned, unaudited environment variable
- Topics: BL-02, CP-10.4, CP-15.6
- Evidence: CP-10.4 notes: "grants are configuration with no route and no audit event of their own (`control-plane/src/policy/deploy.ts:16-20`, `docs/how-to/push-live.md:198-200`); the variable is parsed on every request (`control-plane/src/index.ts:2449`)." This directly contradicts BL-02's claim that deploy capabilities require configuration and a restart: "granting that permission is today a change to the pod's configuration and a restart, not a recorded act inside the product."
- Failure: Deploy permissions (`none`, `pr`, `live`) are resolved dynamically from `process.env.CAMPAIGNBUILDER_DEPLOY` on every single request (`control-plane/src/index.ts:2449`). Any process or operator with container environment access can alter this variable at runtime without restarting the container, without triggering an audit log entry, and without leaving a record of who granted deploy authority. An untracked privilege escalation immediately permits live production pushes.
- Confidence: high
- Marked by authors: yes (Drift noted in CP-10.4 and CP-15.6)

### F-06 — Pre-migration database restore points are excluded from backups, risking total data loss on migration failure
- Topics: BL-05, CP-18.5, EN-10.1
- Evidence: BL-05 acknowledges: "The backup copies the system's own memory and the default site's approved pages... never the copies of the client's database taken before a schema change." EN-10.1 confirms: "the state archive leaves out its top-level `restore-points/`, the migration lane's pre-migration dumps of a client's database, and the manifest says so (`enclosure/tools/backup/backup.mjs:68-73`, `enclosure/tools/backup/backup.mjs:134`, `enclosure/tools/backup/backup.mjs:626`)."
- Failure: When a schema migration is approved, a full `pg_dump` is written to `restore-points/` (`control-plane/src/state/dir.ts:60-62`, `control-plane/src/scr/migration-runner.ts:233`). If a migration fails midway or corrupts table structures, and a concurrent host crash, filesystem corruption, or container eviction occurs, the automated backup archive will contain the corrupted database state but zero pre-migration restore points. The database cannot be recovered to its pre-migration state, causing irreversible data loss.
- Confidence: high
- Marked by authors: no

### F-07 — Failed render rollbacks leave broken changes applied, which are subsequently published to production
- Topics: CP-12.4, CP-12.6, CP-15.7
- Evidence: In CP-12.4: "settleRefusal returns the proposal to `proposed` only when the page was written back; otherwise it stays `applied` with the refusal as its error (`control-plane/src/surface/gate.ts:267-269`)." In CP-15.7: "The substrate pushes the page's current row (`control-plane/src/scr/substrate.ts:113-125`), so both doors ask whether the most recent applied proposal on that page, by decision time, is live but unverified... a later apply that verifies makes the page pushable again (`control-plane/src/surface/gate.ts:483-487`)."
- Failure: Proposal A applies a change to a page, but the post-apply browser render check fails and the database rollback write also fails (e.g., due to a temporary DB error or constraint). Proposal A remains in the `applied` state on the database row. Later, Proposal B applies a minor copy change to the same page and its render check succeeds. Because Proposal B is now the newest applied proposal, `isLiveUnverified` clears. Pushing Proposal B to production writes the entire database row (`PageTestVariant.pageConfig`), publishing Proposal A's broken, unverified, and failed changes straight to production.
- Confidence: high
- Marked by authors: no

### F-08 — Non-default tenants cannot use conversational editing, the fast lane, or content publishing
- Topics: BL-06, CP-06.1, CP-07.1, CP-15.7
- Evidence: BL-06 admits: "Until it closes, a second tenant is served and can be edited through the engineering lane, but has no page chat, no fast content lane, and a content push to it is refused rather than risked." In CP-06.1: "chat answers for the default project or refuses: a project named in the body is a 400 naming it (`control-plane/src/index.ts:1009-1011`)." In CP-15.7: "The route refuses a target other than `SURFACE_SOURCE_PROJECT_ID` before the proposal is read... 409 (`control-plane/src/index.ts:1327-1333`)."
- Failure: Tenants added alongside the default tenant cannot use the product's primary advertised capability—editing sites by conversation. Any marketing turn submitted via `/api/chat` naming a non-default project is rejected with HTTP 400. Furthermore, fast-lane proposals and content `/pushlive` operations are hard-locked to `SURFACE_SOURCE_PROJECT_ID`, returning HTTP 409 for any other project. Multi-tenancy for content editing and publishing is completely inoperable.
- Confidence: high
- Marked by authors: yes (ADR-019 second-tenant backlog)

### F-09 — Ingress exempts `@tenant_intake` from session cookie stripping, enabling cross-site change submission
- Topics: EN-03.1, EN-03.2, CP-02.6, WS-01.3
- Evidence: EN-03.2 documents: "Every proxy to the tenant or the overlay imports `(tenant_session_strip)`... Only `@tenant_intake` omits it, so the tenant's intake route can forward the requester's own credential to the control plane (`enclosure/docker/Caddyfile:211-215`)." CP-02.6 and ES-03.1 confirm: "The browser still attaches it to same-origin calls made by tenant script...".
- Failure: Browsers automatically attach the `cb_session` cookie to all same-origin requests sent to the tenant domain. Because `@tenant_intake` intentionally skips session cookie stripping, any script running within the tenant preview (including third-party scripts or agent-injected code) can issue a cross-site request to `POST /api/ai/structural-change`. The tenant server validates the forwarded session cookie via `/auth/me` and submits the intake request to `POST /api/scr/intake`, minting an authenticated change request under the victim marketer's credentials without their knowledge.
- Confidence: high
- Marked by authors: no

### F-10 — Single shared engine subprocess causes head-of-line blocking and drops queued chat requests on timeout
- Topics: CP-05.1, CP-05.4, CP-05.8
- Evidence: CP-05.1 notes: "chat's engine is memoised, because a fresh instance per call would spawn a `pi` subprocess no route ever closes (`control-plane/src/engine/client.ts:118-120`)." CP-05.8 states: "Both routes now wait in one line for the shared engine... Chat and `POST /api/engine/prompt` race the one shared engine against `CAMPAIGNBUILDER_CHAT_TURN_TIMEOUT_MS`, ten minutes by default... past it the stalled process is closed... the turns waiting behind one that timed out are refused as busy rather than run on the engine being closed (`control-plane/src/index.ts:772`)."
- Failure: When multiple users interact with the chat bubble, all requests queue sequentially behind a single shared `pi` subprocess. If one user's prompt hangs or hits the 10-minute timeout, the process is killed. Instead of allowing subsequent waiting turns to run on a restarted process, every single request waiting in the queue is aborted and returned as HTTP 503 `AGENT_BUSY` (`control-plane/src/index.ts:696-700`), causing a total denial of service for all conversational users.
- Confidence: high
- Marked by authors: no

### F-11 — Engineering lane deadlocks permanently on failed restore or failed rollback
- Topics: BL-03, CP-13.2, CP-13.6
- Evidence: BL-03 defines: "a failed row whose restore failed holds the lane until an operator closes or retries it, because nobody can vouch for the tree (`control-plane/src/scr/bracket.ts:162-170`, `control-plane/src/scr/gate.ts:861-865`); the request clock does not free it, since such a row is held and a held row never expires (`control-plane/src/scr/transition.ts:225-226`, `control-plane/src/scr/transition.ts:239`, `control-plane/src/scr/transition.ts:262`)."
- Failure: If a git restore operation fails during a rollback (e.g., due to an untracked file conflict, permissions mismatch, or disk I/O error), the request is transitioned to `FAILED` with `active: true` held. Because held rows are explicitly exempted from TTL expiration (both the 24-hour park TTL and the 72-hour request TTL), the project's single engineering lane is locked indefinitely. All subsequent intake submissions for that project are rejected with lane-busy errors until an operator manually discovers and forces closure via Foreman.
- Confidence: high
- Marked by authors: no

### F-12 — Ingress routing requires manual out-of-band restarts, serving stale routes or 502 Bad Gateway
- Topics: CP-08.4, EN-03.1, FM-04.4
- Evidence: CP-08.4 states: "Stopping the site deletes the routing file... The front door itself reads those files only when the operator restarts it; the box never reconfigures it while running. So a new address goes live, and a stopped site's address stops answering 'bad gateway' to the public, only at that restart, and nothing in the box prompts for it." EN-03.1 adds: "generated host changes reach the public only after an ingress restart, because Caddy's admin API is disabled and the glob is read when the process starts (`enclosure/docker/Caddyfile:469-474`)."
- Failure: When an administrator provisions a new tenant or tears down an existing one in Foreman, Caddy's routing configuration is not reloaded. Newly created tenant domains cannot be resolved by users. When a tenant container is stopped, Caddy continues routing traffic to the dead upstream, serving HTTP 502 Bad Gateway errors to public visitors until an operator logs into the host VM shell and manually executes `docker compose restart ingress`.
- Confidence: high
- Marked by authors: no

### F-13 — Code promotion is a stub that terminates at pull request creation without production deployment
- Topics: BL-01, BL-02, BL-07, CP-10.4
- Evidence: BL-02 records the POC owner decision: "code promotion opens a pull request after the required human approval. The target GitHub repository owns merge and any CI/CD deployment... The scratch target has no deployment workflow; its existing VM preview already served the same tree, so merge did not trigger a rollout or external production deployment." CP-10.4 enforces: "agent-written code never deploys `live`. The deploy route's decision caps an allowed `live` at `pr` with `downgraded: true` (`control-plane/src/policy/deploy.ts:153-157`)."
- Failure: The product claims to allow marketers to edit live campaign sites by conversation. However, for any code change (Tier 1 or Tier 2), executing "Push live" only pushes a branch and opens a GitHub pull request. There is no automated build, container rollout, or deployment to any production environment. The entire path to production depends on an external manual merge and an unbuilt CI/CD pipeline.
- Confidence: high
- Marked by authors: yes (Decision recorded in BL-02 and BL-07)

### F-14 — Audit trail sidecar uses an in-memory queue that silently loses events on container restart or backlog
- Topics: BL-05, CP-03.1, CP-03.7
- Evidence: CP-03.1 acknowledges: "The copy is handed to an in-memory queue (`control-plane/src/audit/emit.ts:151`, `control-plane/src/audit/ship.ts:148`) for the audit sidecar... it never blocks the caller... and it is lost for whatever is still queued at a restart or evicted past the queue bound (`control-plane/src/audit/ship.ts:13-14`, `control-plane/src/audit/ship.ts:205-207`)." Furthermore, CP-03.7 notes that despite documentation calling the copy "off-box", compose places it on the exact same Docker daemon (`enclosure/docker/compose.pod.yaml:943`).
- Failure: If the audit ingest container experiences downtime, restarts, or slow disk I/O, events accumulate in the control plane's Node.js heap memory. When the backlog reaches 10,000 events, subsequent records silently evict the oldest pending events. If the control plane container crashes or restarts, all queued audit events are permanently lost with no mechanism to backfill or detect missing sequence numbers. The secondary audit chain is left corrupted with missing history.
- Confidence: high
- Marked by authors: yes (Tension noted in CP-03)

### F-15 — Foreman Configuration tab and rebuild plans are client-side simulations with no backend persistence
- Topics: CP-01.8, FM-06.1, FM-06.3
- Evidence: FM-06.1 documents: "On a live build there is nothing to apply. The control plane's one config route reads the TOML file and nothing under `foreman/src/` calls it; it has no write route, so a change has nowhere to go (`control-plane/src/index.ts:2579-2581`)." FM-06.3 adds: "The rebuild plan is the demo world's four 950 ms timers, each step marked simulated (`foreman/src/features/config/RebuildPlan.tsx:62`, `foreman/src/features/config/RebuildPlan.tsx:155`)."
- Failure: An operator attempting to reconfigure the platform or execute system adjustments via the console's Configuration tab is interacting with a client-side mockup. The control plane exposes no configuration write endpoint (`PUT /api/config` returns 404). Rebuild plans run mock JavaScript timers and display simulated completion cards. Operators cannot persist settings or trigger rebuilds through the console UI.
- Confidence: high
- Marked by authors: yes (ADR-004 addendum; FM-06)

---

## Not judgeable from this material

1. **Docker socket escape vulnerability in the provisioner container**: The provisioner container mounts the host's `/var/run/docker.sock` (`enclosure/docker/compose.pod.yaml:851`). Because it runs as root, any arbitrary command injection or path traversal in `provisioner.mjs` grants full root host compromise. The exact sanitization and isolation of Docker daemon commands cannot be fully verified without auditing the full source of `executor.mjs` and `command.mjs`.
2. **Behavior and safety of Postgres migration scratch instances under concurrency**: The scratch database runner spins up isolated Postgres databases for testing migrations (`control-plane/src/scr/migration-scratch.ts`). Whether concurrent schema validations cause connection pool starvation, temp disk exhaustion, or socket contention against the single shared PostgreSQL container cannot be judged from the text alone.
3. **SSRF resilience of the Playwright headless browser check**: The in-pod headless browser (`ws-render.mjs` / `verify-surface.mjs`) renders pages inside the container network. While an allowlist is referenced, whether Playwright can be redirected by a compromised page or malicious tenant HTML to probe internal cloud metadata services (e.g. `169.254.169.254`) or internal Docker network IPs cannot be verified without inspecting the Chromium network interception rules.
4. **Tenant application database isolation in real customer deployments**: While the local POC uses a single Postgres container with `campaignbuilder_app` bypassing RLS, real tenant deployments depend on external Supabase configurations. Whether external customer Supabase projects actually enforce tenant separation or rely on the same flawed bypass cannot be evaluated without customer infrastructure definitions.
5. **Memory and process stability under large diffs and long-running git operations**: Git diffs and restore points use in-memory buffers and child processes (`control-plane/src/scr/bracket-git.ts`, `control-plane/src/scr/complexity.ts`). Whether large binary assets, package lockfiles, or deep directory trees cause Node.js out-of-memory crashes or unhandled buffer truncation during complexity classification cannot be determined from the distillation.

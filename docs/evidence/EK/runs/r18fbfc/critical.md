### F-01 — Shared Agent UID Across Tenants Breaks Workspace Isolation
- Topics: BL-06, ES-03, CP-17, CP-15
- Evidence: ADR-019 addenda state that per-project UIDs were never built, so all tenant agents share UID 1004 (`docs/reference/adr/ADR-019-single-pod-multi-project-tenancy.md:694`, `docs/reference/adr/ADR-019-single-pod-multi-project-tenancy.md:705-708`). BL-06 admits that "the agents of both tenants run as the same system account, so a request on one can change the other's source; until each project gets its own account, the two tenants are one trust domain" (`BL-06: For the business`). CP-17 confirms UID 1004 owns the engineering grant directories across every tenant workspace mounted into the container (`enclosure/docker/workspace-seed.sh:115-120`), and post-turn scope checks only compare the target tenant's workspace against its restore point (`control-plane/src/scr/executor.ts:1181-1195`).
- Failure: An engineering change executed on behalf of Tenant A instructs the agent to write files into `/workspaces/<tenant-b>/app/`. Because UID 1004 owns those directories in Tenant B's mounted workspace and the scope validator only diffs Tenant A's workspace against Tenant A's restore point, Tenant B's source is modified or backdoored without triggering a scope denial or recording an audit event on Tenant B.
- Confidence: High
- Marked by authors: Yes (Open / Addendum in ADR-019)

### F-02 — Tenant Runtime Container Executes Unverified Agent Code with Bypass-RLS Database Credentials
- Topics: ES-03, EN-04, BL-06
- Evidence: ES-03 documents that source writes are immediately compiled and served by `next dev` inside the tenant container (`enclosure/docker/compose.pod.yaml:182-185`, `enclosure/docker/web-surface-entrypoint.sh:537`). The tenant runtime account (`surface`, UID 1005) holds `DATABASE_URL` with `campaignbuilder_app`, which is granted `BYPASSRLS` (`enclosure/docker/web-surface-entrypoint.sh:342`), granting unrestricted access to all rows across the tenant database. Furthermore, tenants share `pod-net` with outbound internet access.
- Failure: During an engineering attempt, the agent writes malicious code into an API route or server component. Before the attempt passes verification or is submitted for human review, `next dev` hot-reloads the code. The code executes in the runtime container using `BYPASSRLS` database authority, allowing it to extract all database rows and exfiltrate them to any host permitted by the egress firewall.
- Confidence: High
- Marked by authors: No

### F-03 — In-Memory Audit Shipper Drops Events Permanently on Restart or Backlog
- Topics: CP-03, BL-05, EN-01
- Evidence: ADR-006 and ADR-028 §2.2 promise an audit plane that is "never lost" and shipped off-box (`control-plane/src/audit/emit.ts:9-14`). However, CP-03 reveals that delivery to the audit sidecar relies on an in-memory queue capped at 10,000 events (`control-plane/src/audit/ship.ts:98`, `:148`). CP-03 explicitly acknowledges that queued copies are "lost for whatever is still queued at a restart or evicted past the queue bound (`control-plane/src/audit/ship.ts:13-14`, `:205-207`)" and "nothing re-sends them, and the copy service cannot tell that it is missing anything (`control-plane/src/audit/ship.ts:16-17`)." Moreover, compose places the sidecar on the same Docker daemon (`enclosure/docker/compose.pod.yaml:943`), not off-box.
- Failure: If the audit sidecar container crashes, restarts, or lags during high mutation throughput, the in-memory queue overflows or is cleared on control-plane restart. Thousands of audit records vanish from the secondary audit service, while the sidecar continues linking subsequent events into a disjointed hash chain that cannot detect missing records.
- Confidence: High
- Marked by authors: Yes (Tension / documented limitation)

### F-04 — Database Rollback on Migration Failure Overwrites Concurrent Production Data
- Topics: CP-18, CP-15, BL-05
- Evidence: CP-18 states that when an admin approves a schema migration, a full `pg_dump` of the database is taken prior to execution (`control-plane/src/scr/migration-runner.ts:233-234`). If migration execution times out past 15 minutes (`CAMPAIGNBUILDER_MIGRATION_TIMEOUT_MS`) or fails post-apply verification, `migration-runner.ts` restores the database using `pg_restore` (`control-plane/src/scr/migration-runner.ts:286-304`). CP-18 explicitly notes: "Putting the copy back also discards anything the site wrote in the meantime, and the result now says so."
- Failure: During migration execution against a live database, visitors submit form responses, leads, or account updates. If the migration stalls or fails post-apply checks, the runner rewinds the database to the pre-migration snapshot, permanently destroying all production records and user submissions created while the migration was in flight.
- Confidence: High
- Marked by authors: Yes (acknowledged in CP-18 prose)

### F-05 — Failed Workspace Restores Cause Unrecoverable Lane Deadlock
- Topics: BL-03, CP-13, CP-15
- Evidence: BL-03 and CP-13 state as an invariant: "a failed row whose restore failed holds the lane until an operator closes or retries it, because nobody can vouch for the tree (`control-plane/src/scr/bracket.ts:162-170`, `control-plane/src/scr/gate.ts:861-865`); the request clock does not free it, since such a row is held and a held row never expires (`control-plane/src/scr/transition.ts:225-226`, `control-plane/src/scr/transition.ts:239`, `control-plane/src/scr/transition.ts:262`)."
- Failure: If a git restore or bracket rollback encounters an I/O error or permission failure, the request transitions to `FAILED` while retaining `active: true`. Because held rows are explicitly exempt from the 24-hour and 72-hour TTL expiration sweeps, the project lane remains permanently blocked. All subsequent intake or execution requests for that project fail immediately with `SCR_LANE_BUSY` until an operator manually triggers a close route; if the close action also fails to restore the tree, the project is deadlocked indefinitely.
- Confidence: High
- Marked by authors: No

### F-06 — Single Corrupt Ledger or State JSON File Prevents Control Plane Boot for All Tenants
- Topics: CP-01, CP-04, CP-12, CP-13
- Evidence: CP-13 states: "Invariant: a corrupt ledger refuses the gate and the boot, names the file and leaves it untouched (`control-plane/src/scr/gate.ts:281-286`, `control-plane/src/index.ts:4332`)." CP-01 notes that `bootServer` loops through all projects sequentially and aborts boot if any gate cannot be constructed (`control-plane/src/index.ts:4365`). CP-04 states that an unparseable `world.json` halts boot (`control-plane/src/world/store.ts:363`), CP-12 states that a malformed `proposals.json` halts boot (`control-plane/src/surface/store.ts:463-492`), and CP-01 states that an unparseable `runtime-projects.json` halts boot (`control-plane/src/project/runtime-store.ts:107`).
- Failure: A sudden power cut or out-of-space event results in a truncated or malformed JSON write in `scr-ledger.<id>.json` for a single tenant. On restart, `bootServer` fails to parse that file and aborts process initialization before opening the HTTP listener. Every tenant hosted on the pod suffers a complete outage due to one corrupted project file.
- Confidence: High
- Marked by authors: No

### F-07 — Surface Fast Lane and Live Content Publishing Are Hardcoded to Default Project Only
- Topics: BL-06, BL-07, CP-06, CP-12, CP-15
- Evidence: ADR-019 claims single-pod multi-project tenancy. However, BL-06 admits that the content lane has no multi-tenant implementation: "The host's editing source is the default project's alone, so a tenant's validator has no page write to guard yet: a tenant has no fast lane and its content push is refused, named, before anything is read (`control-plane/src/project/registry.ts:654-655`, `control-plane/src/index.ts:1327-1333`)." CP-06 enforces that chat requests with any `projectId` other than `default` return HTTP 400 (`control-plane/src/index.ts:1009-1011`). CP-15 confirms that pushing content for any project other than `SURFACE_SOURCE_PROJECT_ID` is refused with HTTP 409 (`control-plane/src/index.ts:1327-1333`).
- Failure: An operator registers a second client tenant. When marketers access that tenant, conversational chat editing, fast-lane proposals, and `/pushlive` content publishing fail completely with HTTP 400 or 409 errors.
- Confidence: High
- Marked by authors: Yes (Backlog item in ADR-019)

### F-08 — Unrestricted Vault Token References Permit Cross-Tenant Credential Squatting
- Topics: BL-06, CP-07, CP-08, CP-09
- Evidence: CP-08.1 notes as an open vulnerability: "nothing binds a token reference to the project that names it: registration admits any reference the vault holds (`control-plane/src/index.ts:3178-3179`) and the registry resolves it as given (`control-plane/src/project/registry.ts:316-317`)." CP-09 confirms that `POST /api/projects` accepts arbitrary named references for database slots and PAT credentials (`control-plane/src/index.ts:3130`, `:3168`).
- Failure: An administrator registers Tenant B and supplies the vault reference string belonging to Tenant A's Supabase connection or GitHub PAT. The control plane binds and materializes Tenant A's database credentials into Tenant B's container environment at `/run/credentials/<tenant-b>/env`, giving Tenant B full access to Tenant A's database.
- Confidence: High
- Marked by authors: Yes (Open question in CP-08.1)

### F-09 — In-Pod Verification Harness Couples Hydration Proof to Default Tenant DOM Attribute
- Topics: CP-14, EN-05
- Evidence: CP-14 and EN-05 describe a repair where `ws-render` waits after HTTP 200 for `main[data-campaignbuilder-hydrated="true"]` (`enclosure/tools/ws-verify/ws-render.mjs:148-176`). If this attribute is absent within the navigation timeout, `ws-render` emits exit code 2 (`enclosure/tools/ws-verify/ws-render.mjs:178-186`). CP-14 confirms: "This contract does not prove every delayed effect, all pages or other tenant frameworks."
- Failure: A non-default tenant with standard Next.js, Remix, or Astro code that does not embed `data-campaignbuilder-hydrated="true"` on a `<main>` tag runs an engineering change. The verification check times out waiting for the specific DOM attribute and exits with code 2. The executor interprets exit code 2 as a harness error (`control-plane/src/scr/verify.ts:194-206`), aborting and rolling back every valid change.
- Confidence: High
- Marked by authors: No

### F-10 — Publishing Earlier Content Proposals Deploys Unverified Working Copy to Production
- Topics: CP-12, CP-15
- Evidence: CP-15 describes that `writePageConfig` pushes the current database row (`pageConfig` and `sectionDrafts`) directly to production (`control-plane/src/scr/substrate.ts:113-125`). CP-12 records that if a proposal's render check fails and its rollback also fails, the proposal remains in state `applied` and its broken modifications remain live in the editing database row (`control-plane/src/surface/gate.ts:267-269`). CP-15 admits: "publishing an earlier change to the same page sends the page as it stands, including that failed change."
- Failure: Proposal 2 introduces broken content, passes schema validation, but fails browser verification; its rollback fails, leaving its content in the database. A marketer subsequently selects older Proposal 1 on that page and executes `/pushlive`. Because the substrate pushes the entire row state, Proposal 2's unverified and broken content is published directly to the production website.
- Confidence: High
- Marked by authors: No

### F-11 — Dynamic Tenant Ingress Host Routing Requires Out-of-Band Host Docker Restarts
- Topics: CP-08, EN-03, FM-04
- Evidence: EN-03 states: "Invariant: generated host changes reach the public only after an ingress restart, because Caddy's admin API is disabled and the glob is read when the process starts (`enclosure/docker/Caddyfile:469-474`)." CP-08 and FM-04 note: "Every control-plane answer that writes or removes a block carries `ingressRecreateRequired`, the console shows the command from it, and nothing runs it (`control-plane/src/index.ts:2997`, `foreman/src/features/projects/ProjectsTab.tsx:32`)."
- Failure: An operator provisions a new tenant or tears down an existing one using Foreman. The control plane updates the `.caddy` configuration files in the volume. However, because Caddy has no active reload mechanism, the ingress never picks up the changes. New tenant hostnames fail with connection refused or SSL errors, and deleted tenants continue routing traffic to stopped containers until an engineer manually SSHes into the VM host and restarts the container via `docker restart ingress`.
- Confidence: High
- Marked by authors: Yes (Documented manual requirement)

### F-12 — Engine Turn Timeout Triggers Process Termination and Cascading Abortion of Queued Requests
- Topics: CP-05
- Evidence: CP-05 specifies that all turns share a single subprocess (`control-plane/src/engine/pi.ts:813-817`) bounded by `CAMPAIGNBUILDER_CHAT_TURN_TIMEOUT_MS` (default 10 minutes). When a turn exceeds this timeout, the process is killed (`control-plane/src/index.ts:773-776`). CP-05 explicitly documents that "the turns waiting behind one that timed out are refused as busy rather than run on the engine being closed (`control-plane/src/index.ts:772`)."
- Failure: A user submits a prompt that triggers a 10-minute model timeout. Several other marketers queue chat messages behind it. When the timeout expires, the engine process is killed, and every queued turn waiting in line is immediately aborted with 503 `AGENT_BUSY` instead of executing against the respawned engine.
- Confidence: High
- Marked by authors: No

### F-13 — First-Boot Tenant Provisioning Executes Unfenced `npm ci` as Runtime User
- Topics: EN-04, EN-08, ES-03
- Evidence: EN-04 states: "A provisioned tenant is the exception before that: on first boot its provision entrypoint runs `npm ci` as the runtime account before any fence exists (`enclosure/docker/tenant-provision-entrypoint.sh:219`, `docs/reference/adr/ADR-019-single-pod-multi-project-tenancy.md:808-810`)." EN-08 confirms that the egress firewall (`init-firewall.sh`) is not invoked until the subsequent `web-surface-entrypoint.sh` starts (`enclosure/docker/web-surface-entrypoint.sh:206-216`).
- Failure: A customer provisions a tenant whose cloned repository contains an npm dependency with a malicious `preinstall` or `postinstall` script. Because `npm ci` executes before `init-firewall.sh` applies iptables rules, the script runs with unrestricted outbound network access, allowing it to scan internal pod networks or exfiltrate environment data.
- Confidence: High
- Marked by authors: Yes (Documented exception in ADR-019)

### F-14 — Companion IDE Extension Cannot Authenticate to Control Plane as Shipped
- Topics: EN-07, CP-02
- Evidence: EN-07 documents: "The extension sends no credential. Its `fetch` runs in the extension host, not the editor's browser, so no session cookie travels with it (`enclosure/extension/src/controlPlane.ts:7-13`), and `POST /api/chat` refuses an anonymous caller (EN-07.2)... so as shipped it cannot hold a conversation unless something in front of it supplies an identity; the only thing that does today is the operator's emergency access route (`enclosure/access/access-proxy.Caddyfile:92-94`)."
- Failure: A developer attempts to use the Companion chat panel inside code-server in the sandbox container. Every request made to `POST /api/chat` is rejected by the control plane with HTTP 401 Unauthorized because the extension host has no session cookie or credentials to attach, rendering the feature completely inoperative.
- Confidence: High
- Marked by authors: Yes (Marked as known gap in EN-07)

### F-15 — Foreman Configuration Tab and Rebuild Plan Are Non-Functional Simulations
- Topics: FM-06, CP-01
- Evidence: FM-06 documents that on live builds, the Configuration tab is strictly read-only (`foreman/src/features/config/ConfigTab.tsx:37`), pending edits and rebuild plans do not mount, and editing works only in demo mode (`foreman/src/features/config/ConfigTab.tsx:220`, `:231`). CP-01 confirms that `/api/config` has no `PUT` route, returning 404 (`control-plane/src/index.ts:2591`, tested at `control-plane/src/index.test.ts:188`). FM-06 confirms the rebuild plan consists of four mock 950ms `setTimeout` timers (`foreman/src/features/config/RebuildPlan.tsx:62`).
- Failure: An operator attempting to adjust settings (e.g., spending limits, deploy capabilities, or timeouts) from the Foreman UI finds the configuration controls disabled. In demo mode, changes update local React state and run fake progress timers without modifying the host, while in production, changing settings requires manually editing host files and restarting services.
- Confidence: High
- Marked by authors: Yes (Documented in ADR-004 addendum and FM-06)

---

## Not judgeable from this material

1. **Target Repository CI/CD and Production Deployment Workflow**: Code promotion opens a GitHub Pull Request, but the material notes that the target scratch repository has no deployment pipeline and merges do not trigger automated deployments. The actual production rollout and rollback mechanism post-merge cannot be verified.
2. **Container Breakout and Docker Daemon Security in Provisioner**: The provisioner container bind-mounts `/var/run/docker.sock` to start and stop tenant containers. The host-level daemon configuration, user namespace remapping, and AppArmor/SELinux profiles cannot be evaluated from these files to determine if the provisioner can escape to root on the host VM.
3. **End-to-End Pod Disaster Recovery and Database Restores**: The backup utility (`backup.mjs`) writes encrypted `.age` archives and tests decryption and HMAC integrity in unit suites, but its restore command merely extracts a `.dump` file to disk without restoring the PostgreSQL database or validating container state on a live running pod.
4. **Model Provider Latency, Concurrency, and Token Throttling Under Load**: Chat and prompt turns are strictly serialized on a single shared engine process. Real-world concurrency behavior, upstream model rate-limiting, and cost behavior under burst traffic cannot be determined from static unit test fixtures.
5. **Ingress TLS Automated Certificate Renewal Under Disabled Admin API**: Caddy's dynamic administration API is disabled, and configuration is loaded via filesystem globs. Whether automated Let's Encrypt or ZeroSSL certificate renewals succeed across container restarts and dynamic tenant additions cannot be confirmed without live network and ACME telemetry.

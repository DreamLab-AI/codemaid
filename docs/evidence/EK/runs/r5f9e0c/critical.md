### F-01 — Shared agent UID destroys tenant workspace isolation
- Topics: BL-06, ES-03, CP-15, ADR-019
- Evidence: `BL-06` claims that "keeping each project's source tree, credentials, agent process, deploy target and trust dial separate is what makes one tenant unable to reach another's work." However, `BL-06` and `docs/reference/adr/ADR-019-single-pod-multi-project-tenancy.md:694, 705-708` concede that "the agent uid is shared across tenants, per-project uids being unbuilt, so the contract separates agent from custodian and tenant runtime from host files, never one tenant's agent from another's." In `enclosure/docker/compose.pod.yaml:524-531`, the control plane mounts every tenant workspace into the same container where the engine executes under UID 1004 (`enclosure/docker/Dockerfile.control-plane:180-186`).
- Failure: When an engineering request runs for Tenant A, the agent process (running as UID 1004) can traverse the shared filesystem into `/workspaces/tenant-b`, reading Tenant B's source code, altering its files, or planting malicious scripts that will be served to Tenant B's preview visitors or committed to Tenant B's repository.
- Confidence: high
- Marked by authors: yes (ADR-019 addendum; marked as open gap in BL-06)

### F-02 — Project registration permits arbitrary cross-tenant credential theft
- Topics: BL-06, CP-07, CP-08.1, CP-09
- Evidence: `BL-06` asserts that one tenant cannot reach another's credentials. Yet `CP-08.1` and `control-plane/src/index.ts:3156, 3179` state that `POST /api/projects` dry-resolves any existing credential reference passed in the request body (`supabaseConnRef`, `githubPatRef`, `modelKeyRef`) against the global vault via `control-plane/src/vault/store.ts`. `control-plane/src/project/registry.ts:316-317` resolves whatever reference is configured without verifying that the reference belongs to the registering project.
- Failure: An administrator registering or updating Tenant B can configure Tenant B's `supabaseConnRef` to reference Tenant A's database slot reference. When the control plane executes `materialiseSurfaceEnv` (`control-plane/src/vault/inject.ts:53`), it reads Tenant A's Supabase credentials from the vault and writes them directly into Tenant B's private credential mount (`<dir>/tenant-b/env`), granting Tenant B full database access to Tenant A's data.
- Confidence: high
- Marked by authors: yes (Open in CP-08.1)

### F-03 — Unreviewed agent code hot-reloads inside tenant runtime holding credentials and external egress
- Topics: ES-03.1, EN-04.3, CP-14, BL-01
- Evidence: `README.md:13-17` and `BL-01` promise that code changes are bounded by checkpoints and human governance before reaching live systems. However, `enclosure/docker/web-surface-entrypoint.sh:537` executes `next dev -H 0.0.0.0` inside the tenant container directly against the live workspace volume (`/workspace`) where the agent applies edits. `enclosure/docker/web-surface-entrypoint.sh:160, 374` exports the tenant's database connection (`DATABASE_URL`, bypassing RLS via `BYPASSRLS`) and model API keys into that runtime environment. In `enclosure/scripts/init-firewall.sh:206-216`, the tenant runtime maintains outbound network access to its database and external APIs.
- Failure: As soon as the agent writes an edit during an automated turn, `next dev` hot-reloads and executes that code on the preview server *before* tests run, *before* complexity analysis, and *before* human approval. An agent prompt injection or malicious patch can immediately execute server-side code that queries the tenant database and transmits sensitive customer records to allowed external APIs during the drafting phase.
- Confidence: high
- Marked by authors: no

### F-04 — Model provider egress firewall resolves to shared multi-tenant CDN IP pools
- Topics: EN-08.1, EN-08.2, EN-08.4, BL-05
- Evidence: `BL-05` and `docs/explanation/cloud-isolation.md:175-178` state that outbound traffic is strictly restricted: "its own outbound traffic beyond its own container may reach only the model provider and the pod services it is named, each at one address and one port and never a whole subnet." In `enclosure/scripts/init-firewall.sh:213, 380`, model provider domain names (`api.openai.com`, `openrouter.ai`, `api.anthropic.com`) are resolved via DNS at startup and loaded into an `ipset` matching `address,tcp:port`.
- Failure: Major AI providers host their API endpoints behind Cloudflare or AWS CloudFront IP ranges shared with millions of third-party domains. Any process running under the restricted UID (1004 or 1005) can connect to arbitrary third-party endpoints hosted on those same shared CDN IP addresses on port 443, bypassing outbound data exfiltration blocks. Additionally, when the provider's CDN rotates IP addresses, the static ipset entries become invalid, causing legitimate model calls to fail with connection timeouts until the container is restarted.
- Confidence: high
- Marked by authors: no

### F-05 — Failed schema migration rollback drops production database state
- Topics: CP-18.5, BL-02, CP-15.8
- Evidence: `BL-02` states that database schema migrations safely park for an administrator with a written brief and restore point. `control-plane/src/scr/migration-runner.ts:233, 284` creates a `pg_dump` backup before applying SQL. If verification fails after the SQL has executed (`control-plane/src/scr/migration-runner.ts:358-364`), or if `psql` fails with non-transactional errors, the runner executes `pg_restore --clean` to revert the database to the pre-apply dump (`migration-runner.ts:284`).
- Failure: When a migration fails verification after committing, `pg_restore --clean` completely overwrites the live production database with the snapshot taken prior to migration execution. Any user registrations, lead captures, form submissions, or orders created by live site visitors between the initial dump and the rollback are permanently destroyed. Furthermore, if `pg_restore --clean` fails halfway through, the database remains in an inconsistent, partially dropped state without automated remediation.
- Confidence: high
- Marked by authors: yes (Partially marked in CP-18.5 as writes being discarded)

### F-06 — Pre-migration database snapshots are omitted from backup archives
- Topics: EN-10.1, BL-05, CP-15.8
- Evidence: `BL-05` asserts that the backup mechanism protects system state and data integrity. In `enclosure/tools/backup/backup.mjs:134, 595, 681-682`, the backup utility explicitly excludes the `restore-points/` directory from the state archive (`payload.tar.age`). `control-plane/src/state/dir.ts:60-62` and `control-plane/src/scr/migration-runner.ts:233-234` define `restore-points/` as the sole directory containing the `pg_dump` archives taken before schema migrations.
- Failure: If a database migration corrupts state or a hardware/container failure occurs during a migration window, an administrator attempting disaster recovery from the system backup will discover that the backup archive contains no pre-migration database restore points. The database cannot be recovered to its pre-migration baseline from the backup archive.
- Confidence: high
- Marked by authors: no

### F-07 — Failed bracket restore permanently wedges project execution lane
- Topics: BL-03, CP-13.1, CP-13.2, CP-15.2
- Evidence: `CP-13` claims that "Nothing gets stuck, because a finished request frees the site for the next one and a request nobody answers for three days is closed." However, `control-plane/src/scr/bracket.ts:162-170`, `control-plane/src/scr/gate.ts:861-865`, and `control-plane/src/scr/transition.ts:225-226, 239` dictate that if a filesystem restore throws during a rollback, the row is marked `FAILED` with `active: true` (holding the lane). In `control-plane/src/scr/transition.ts:225-226`, `isEligibleForExpiry` explicitly returns `false` for any row where the lane is held.
- Failure: If a bracket restore fails (due to an unhandled git lock, filesystem permission error in `workspace-reassert`, or disk space exhaustion), the request enters `FAILED` while retaining the lane lock. Automated 24-hour and 72-hour TTL sweeps cannot expire or clear the request. All subsequent engineering requests for that project are rejected with `503 SCR_LEDGER_UNAVAILABLE` or lane-busy errors indefinitely, creating a permanent project deadlock that requires manual ledger modification to resolve.
- Confidence: high
- Marked by authors: no

### F-08 — Off-box audit log shipping drops events and diverges hash chains on crash
- Topics: BL-05, CP-03.1, CP-03.7
- Evidence: `ADR-028 §2.2` and `BL-05` promise an audit plane that is "never lost", "durable locally, shipped when reachable, tamper-evident by hash chain", and protected against local tampering by an off-box copy. However, `control-plane/src/audit/ship.ts:13-14, 148, 205-207` uses a volatile, in-memory queue to transmit records to the sidecar. Furthermore, in `control-plane/src/audit/ingest.ts:198`, the sidecar calculates its own hash chain using its own head rather than recording the hashes from the local spool.
- Failure: If the control plane crashes, restarts, or runs out of memory while the audit sidecar is temporarily unreachable or backlog-bound (over 10,000 events), all un-shipped events in memory are permanently dropped without replay from the local disk spool. Because the sidecar computes hashes independently based on arrival order, any dropped event causes the sidecar's cryptographic chain to permanently diverge from the local spool, invalidating tamper-evidence audits. Additionally, both containers run on the same Docker daemon (`enclosure/docker/compose.pod.yaml:943`), so host compromise defeats both logs simultaneously.
- Confidence: high
- Marked by authors: yes (Marked as a known limitation in CP-03)

### F-09 — Single-process chat queue timeout aborts and discards all concurrent user requests
- Topics: CP-05.4, CP-05.8, BL-01
- Evidence: `BL-01` presents the product as an interactive platform for marketing teams. In `control-plane/src/index.ts:739, 762, 772` and `control-plane/src/engine/turn-queue.ts:54-82`, all conversational turns and engine prompts across all users funnel through a single serialized `pi` RPC process. The active turn is bound by `CAMPAIGNBUILDER_CHAT_TURN_TIMEOUT_MS` (defaulting to 10 minutes). If a turn times out, `abortWaiting()` runs (`turn-queue.ts:78-82`), immediately terminating all waiting requests.
- Failure: If one user submits a prompt that causes the engine to stall or hang for 10 minutes, all other users with requests queued in the turn queue (up to `CAMPAIGNBUILDER_CHAT_QUEUE_MAX = 4`) are blocked for the full 10-minute duration. When the timeout expires, the stalled engine is killed, and every queued user request is rejected with `503 AGENT_BUSY` or 504 errors without automatic retry, allowing a single malformed or slow prompt to create a denial-of-service condition across the entire control plane.
- Confidence: high
- Marked by authors: no

### F-10 — Hardcoded default tenant dependencies disable multi-tenant core workflows
- Topics: BL-06, CP-06.1, CP-08.6, CP-15.7, ES-07
- Evidence: `BL-06` and `docs/reference/adr/ADR-019-single-pod-multi-project-tenancy.md` promote multi-tenancy where a second tenant is simply a configuration change. In reality, `control-plane/src/index.ts:1009-1011` explicitly rejects any chat request specifying a `projectId` other than the default project with `400`. `control-plane/src/index.ts:1327-1333` and `control-plane/src/project/registry.ts:654-655` reject data publishing (`/pushlive`) for any non-default project with `409`. `enclosure/docker/Caddyfile:469-474` and `control-plane/src/project/caddy-tenants.ts:89-92` require an out-of-band manual Docker restart of Caddy on the host whenever a tenant hostname is added or modified.
- Failure: A second provisioned tenant cannot use conversational chat, cannot use fast-lane editing, cannot publish content pages to production, and cannot have public domain routing activated automatically through the API. The system cannot operate as a multi-tenant platform without manual operator interventions on the host and code alterations in the control plane.
- Confidence: high
- Marked by authors: yes (Marked as Debt and Open in BL-06 and CP-08)

### F-11 — Unknown model pricing disables request spending limits and allows unbounded token consumption
- Topics: CP-05.7, CP-14.7, BL-01, BL-07
- Evidence: `BL-01` and `BL-07` promise that model spending is capped per request (defaulting to $2.00) so the system never spends money unchecked. However, `control-plane/src/engine/pi.ts:180, 665, 903` and `control-plane/src/scr/executor.ts:1313-1315` dictate that when an unknown or custom model ID is reported by `pi`, cost calculation yields `null` (`UNKNOWN`). The code specifies that "an unknown total never trips the budget (`pi.ts:666`)" and "a null spend never trips the ceiling (`executor.ts:1313-1315`)".
- Failure: When an operator configures a custom model identifier or a provider model not recognized by `pi`'s internal registry, the engine reports spend as `null`. Because a null spend cannot trigger `overBudget` or exceed the numeric ceiling, the spending cap check is completely disarmed. The agent will continue executing retry loops and generating tokens indefinitely up to the attempt limit, incurring unmetered, unbounded API charges.
- Confidence: high
- Marked by authors: no

### F-12 — Failed compensate-on-error actions leave unrecorded system mutations
- Topics: BL-01, BL-05, CP-03.6, CP-07.3, CP-10.3
- Evidence: `BL-01` and `CP-01` claim that the system strictly adheres to record-before-act semantics: "Every request that changes something is asked the same three questions... and can it be recorded. If the answer to the last one is no, nothing happens." Yet `control-plane/src/index.ts:3649, 3772` and `CP-03.6` document that tenant port and triple changes register in memory first, then record to audit. If the audit append fails, `putBackUnpersisted` is called to revert the in-memory registration (`index.ts:3605`).
- Failure: If the audit spool is unavailable when an administrator updates a project port or triple, the mutation has already occurred in the live registry. If the compensating `putBack` fails or if the process crashes before compensation finishes, the project configuration remains altered in live memory, but the audit trail contains no record of the change. Similarly, `control-plane/src/scr/bracket.ts:196` restores filesystem state before appending `SCR_ROLLED_BACK`; an audit failure on that branch leaves a restored workspace with zero audit evidence.
- Confidence: high
- Marked by authors: yes (Marked as deliberate exceptions in CP-03.6)

### F-13 — Render verification harness hardcodes default-tenant hydration marker and fails all other frameworks
- Topics: EN-05.2, CP-14.4
- Evidence: `ADR-028 §4.3` and `EN-05.2` describe `ws-render` as an objective gate ensuring pages compile and render cleanly in headless Chromium. In `enclosure/tools/ws-verify/ws-render.mjs:148-179`, `ws-render` waits specifically for a custom DOM attribute: `main[data-campaignbuilder-hydrated="true"]`. If this exact attribute is not found within the timeout window, `ws-render` exits with harness error code 2 (`ws-render.mjs:178-186`). In `control-plane/src/scr/executor.ts:886-890`, exit code 2 immediately aborts the run as a harness failure.
- Failure: If an engineering change is run against a non-default tenant (such as Civic Quest), a template using a different layout, or any page that does not set `data-campaignbuilder-hydrated="true"` on a `<main>` tag, `ws-render` always exits with code 2. The executor refuses the run before comparing diagnostics, immediately causing every legitimate engineering request for that page or tenant to fail and roll back.
- Confidence: high
- Marked by authors: yes (Documented as committed repair for default app in CP-14.4, noting lack of general framework support)

### F-14 — Deploy capabilities bypass audit logging and require host shell access
- Topics: BL-02, CP-10.4, FM-03.4
- Evidence: `README.md:115-119` asserts that deploy authority and trust dials are set per project by operators and recorded. In `control-plane/src/policy/deploy.ts:16-20, 2449` and `docs/how-to/push-live.md:198-200`, deploy grants (`none`, `pr`, `live`) possess no API route, no management console UI, and no audit trail event. Capabilities are parsed dynamically on each request from the host environment variable `CAMPAIGNBUILDER_DEPLOY`.
- Failure: To grant or revoke a user's permission to publish changes to production, an operator must obtain root shell access on the host VM, modify environment variables in host files, and restart the containers. There is no audit logging of who granted or revoked deployment rights, who held them at any given time, or when privileges were escalated, violating the product's core governance and auditability guarantees.
- Confidence: high
- Marked by authors: yes (Marked as Drift in CP-10.4)

### F-15 — WeakMap caching of audit verification permits undetected log tampering
- Topics: CP-03.8, CP-03.1, EN-10.1
- Evidence: `control-plane/src/index.ts:2604` and `control-plane/src/audit/read.ts:136-157` serve audit trail pages to administrators via `GET /api/audit`. In `control-plane/src/audit/read.ts:143-148`, the cryptographic verification verdict is memoized in a `WeakMap` keyed on the spool object, reusing the cached verdict as long as `head`, `bytes`, `checkpoints`, and `file` match the cached state.
- Failure: If an attacker modifies intermediate audit log entries directly on disk (for example, altering event payloads, actor emails, or action types) while preserving the file size and leaving the file's final line (`head`) and checkpoint count intact, `readSpoolPage` will continue to return a cached "Chain intact" verification verdict. The operator console will report that the audit trail is verified and untampered even though past audit records have been altered.
- Confidence: medium

---

## Not judgeable from this material

1. **Host-Level Sudoers Regex Syntax Compatibility**: `enclosure/docker/Dockerfile.control-plane:235` uses an anchored regular expression pattern to restrict `/usr/local/sbin/workspace-reassert` and `workspace-migrations-open` to arguments matching `/workspaces/[a-z0-9-]+`. Standard `sudoers` specifications use shell wildcards (`fnmatch`) rather than POSIX or PCRE regular expressions; if the installed sudo binary fails to parse this syntax, non-default workspace re-assertions will fail with permission errors.
2. **Actual Live Performance of Single-Subprocess RPC under Load**: The system routes all conversational turns and prompt submissions across all users into a single `pi` subprocess. Whether this architecture deadlocks, exhausts memory, or degrades catastrophically under multi-user concurrent production traffic cannot be verified without real load telemetry.
3. **External DNS & SNI Handling for Tenant Ingress Routing**: The material shows Caddy routing multiple tenant domains on ports 80/443 with internal certificate management and `publicHost` directives, but whether Let's Encrypt / ACME issuance functions reliably for dynamic, runtime-registered tenant hostnames behind corporate firewalls or external DNS without wildcard certificates is unverified.
4. **Behavior of `pg_dump` 17 Against Diverse Target Postgres Versions**: The control-plane container bundles `pg_dump` version 17 to back up the internal database and tenant databases. Whether client applications or external Supabase instances running older PostgreSQL versions (e.g., 14 or 15) reject dumps or fail during `pg_restore` operations cannot be determined from the code citations alone.
5. **Real-World Execution of the Backup Restoration Pipeline on Host Infrastructure**: `EN-10` notes that file-level decryption and digest verification are unit-tested, but an actual bare-metal or VM restoration of a running pod estate from a sealed `.tar` archive has only been documented in runbooks and never automatically tested end-to-end.

### F-01 — Shared Agent UID Across Tenants Breaks Workspace Isolation
- Topics: BL-06, ES-03.1, EN-04.1, EN-04.2
- Evidence: BL-06 cites ADR-019 (`ADR-019-single-pod-multi-project-tenancy.md:694`, `705-708`), noting that the agent UID (1004) is shared across all tenants, and "the agents of both tenants run as the same system account, so a request on one can change the other's source; until each project gets its own account, the two tenants are one trust domain." EN-04.2 (`workspace-seed.sh:70`, `115-120`) establishes that the agent UID owns the five surface directories across every tenant workspace mounted by the control plane.
- Failure: A prompt injection or untrusted change executed on Tenant A operates as UID 1004. Because the control plane mounts secondary workspaces (such as `/workspaces/civic-quest`) into the same filesystem namespace under UID 1004 ownership, the agent for Tenant A can read, modify, or plant backdoors in Tenant B's source code, completely bypassing cross-tenant isolation.
- Confidence: High
- Marked by authors: Yes (Acknowledged as an open limitation in ADR-019 and BL-06)

---

### F-02 — Unrestricted Network and Script Execution During Tenant Provisioning (`npm ci`)
- Topics: EN-04.3, EN-08.5, ES-03.1
- Evidence: `tenant-provision-entrypoint.sh:219` runs `npm ci --include=dev` as `surface` under `env -i`. However, the outbound firewall and egress fence are not installed until `web-surface-entrypoint.sh:469`, which runs only after provisioning and seeding complete (`ADR-019-single-pod-multi-project-tenancy.md:808-810`, `web-surface-entrypoint.sh:469-470`).
- Failure: When a new tenant repository is provisioned on the host, `npm ci` executes repository dependencies and arbitrary lifecycle scripts (`preinstall`, `postinstall`) before any egress firewall or name resolver fence is erected. Malicious or compromised dependencies in the tenant repository have unrestricted outbound internet access and full access to internal services on `pod-net` (including the database and control plane), enabling credentials and internal topology to be exfiltrated.
- Confidence: High
- Marked by authors: Yes (Acknowledged in ADR-019 addendum cited in ES-03.1)

---

### F-03 — Flat Pod Network Bypasses Ingress Security Stripping and Forward-Auth Controls
- Topics: ES-03.1, EN-02.2, EN-03.1, EN-03.2
- Evidence: EN-02.2 and ES-03.1 document that all tenant web containers, the control plane, PostgreSQL, and the surface overlay share a flat bridge network (`pod-net`). While Caddy strips identity headers (`X-Auth-Request-*`) on inbound requests (`Caddyfile:67-78`), containers on `pod-net` communicate directly without passing through Caddy. Furthermore, `next dev -H 0.0.0.0` binds to all interfaces in tenant containers (`web-surface-entrypoint.sh:537`).
- Failure: Any compromised tenant container or rogue script running inside `next dev` can bypass Caddy completely and send direct HTTP requests across `pod-net` to peer tenant web servers on port 3000, PostgreSQL on port 5432, or the control plane on port 8788, forging internal forward-auth headers and directly accessing peer tenant network listeners.
- Confidence: High
- Marked by authors: Yes (Identified as a boundary limitation in ES-03.1)

---

### F-04 — Global Vault Namespace Allows Cross-Tenant Credential and Token Hijacking
- Topics: BL-06, CP-08.1, CP-09.3, CP-09.4
- Evidence: BL-06 cites CP-08.1 as an Open question noting that "nothing binds a token reference to the project that names it: registration admits any reference the vault holds (`control-plane/src/index.ts:3178-3179`) and the registry resolves it as given (`control-plane/src/project/registry.ts:316-317`)." CP-09 confirms that VaultStore stores secrets and slots in a flat, global namespace keyed by reference string.
- Failure: During project registration, Tenant B can specify a repository token reference (`gitHubPatRef`) or database reference (`supabaseConnRef`) belonging to Tenant A (e.g. `tenant-a-pat`). The registry resolves the secret from the global vault without verifying project tenancy and writes Tenant A's plaintext credentials into Tenant B's mounted environment file (`<dir>/tenant-b/env`), granting Tenant B full custody of Tenant A's private repository and database.
- Confidence: High
- Marked by authors: Yes (Marked as an Open question in BL-06 and CP-08.1)

---

### F-05 — Database Migration Rollback Wipes All Concurrent Tenant Production Writes
- Topics: CP-18.5, CP-18.7, CP-15.8
- Evidence: In `control-plane/src/scr/migration-runner.ts:284-304`, if an applied migration fails post-migration verification or encounters a non-atomic SQL failure, `applyMigration` executes `pg_restore --clean` against the pre-migration `pg_dump` snapshot (`migration-runner.ts:284`). The code explicitly notes that "every write since the restore point was discarded" (`migration-runner.ts:302-304`).
- Failure: When a database migration is applied to a live database, site visitors continue submitting form leads, updating records, and generating data. If the post-migration schema verification check throws or fails, the system executes a full restore of the pre-migration snapshot, permanently destroying all user transactions, leads, and edits created between the start of the migration and the failure.
- Confidence: High
- Marked by authors: Yes (Acknowledged in CP-18.5 code comments and documentation)

---

### F-06 — In-Memory Audit Queue Drops Events on Restart or Backlog, Breaking Off-Box Integrity
- Topics: BL-05, CP-03.1, CP-03.2, CP-03.7
- Evidence: CP-03 states: "The copy is handed to an in-memory queue (`control-plane/src/audit/emit.ts:151`, `control-plane/src/audit/ship.ts:148`)... it never blocks the caller... and it is lost for whatever is still queued at a restart or evicted past the queue bound (`control-plane/src/audit/ship.ts:13-14`, `control-plane/src/audit/ship.ts:205-207`)." Furthermore, "Nothing re-sends them, and the copy service cannot tell that it is missing anything." This directly contradicts ADR-006's guarantee that the audit plane is "never lost" and BL-05's claim that the off-box copy closes the unkeyed chain's rewrite gap.
- Failure: If the control plane restarts under load or if the audit sidecar experiences a transient network/processing hiccup that backs up more than 10,000 events, events are permanently evicted from memory. Because the sidecar cannot detect missing sequence numbers or request replays, the off-box audit trail diverges from the local disk spool, defeating non-repudiation and off-box tamper detection.
- Confidence: High
- Marked by authors: Yes (Marked as a known Tension in CP-03.1)

---

### F-07 — Pre-Migration Database Restore Points Are Excluded from System Backups
- Topics: BL-05, CP-15.8, EN-10.1
- Evidence: `enclosure/tools/backup/backup.mjs:134, 595, 626` explicitly excludes `restore-points/` when tarring `CAMPAIGNBUILDER_STATE_DIR`, confirmed in BL-05 ("the state directory is archived without its migration restore points... each of which is a `pg_dump` of the client's database") and CP-15.8.
- Failure: If a database migration leaves a client's database in a corrupted or partially migrated state, and the VM suffers an infrastructure failure or disk corruption before an administrator can manually restore it, restoring from the system backup provides no migration restore points. The client database cannot be rolled back to its pre-migration state.
- Confidence: High
- Marked by authors: No

---

### F-08 — Provisioner Status File Size Overflow Causes Replay of Entire Historical Docker Log
- Topics: EN-06.2, EN-06.3
- Evidence: `enclosure/tools/provisioner/provisioner.mjs:173-189` reads `status.json` with a hard limit of 64 KiB: "a file past it reads back as offset 0 and re-runs the whole log (`enclosure/tools/provisioner/provisioner.mjs:112-114`)."
- Failure: Over extended operational periods, as tenant lifecycle events accumulate, if `status.json` grows beyond 64 KiB (or if ack pruning fails), the provisioner resets its processed offset to 0. Upon the next poll or container restart, the provisioner replays every historic `up` and `down` intent record from the beginning of the intent log, triggering mass unexpected container recreation, restarts, and shutdowns of production tenant services.
- Confidence: High
- Marked by authors: No

---

### F-09 — Secondary Tenants Cannot Publish Content or Use the Fast Content Lane
- Topics: BL-06, BL-07, CP-12.4, CP-15.7
- Evidence: CP-15.7 and BL-06 document that `POST /api/surface/proposals/:id/pushlive` hard-refuses any target project other than `SURFACE_SOURCE_PROJECT_ID` (`control-plane/src/index.ts:1327-1333`), and secondary tenant data substrates fail closed (`control-plane/src/project/registry.ts:654-655`). Furthermore, table and column names (`PageTestVariant`, `sectionDrafts`) are hardcoded to the default tenant (`control-plane/src/surface/store.ts:741-751`).
- Failure: Any secondary tenant (such as `civic-quest`) is completely barred from using fast-lane page proposals or pushing content live. Marketers on secondary tenants receive 409 conflict errors, making multi-tenancy a non-functional claim for content operations.
- Confidence: High
- Marked by authors: Yes (Marked as ADR-019 second-tenant backlog item)

---

### F-10 — Unhandled Rollback Write Failure Leaves Broken Page Live and Permanently Wedges Approvals
- Topics: CP-12.3, CP-12.6, CP-15.7
- Evidence: In `control-plane/src/surface/gate.ts:267-271`, if a render check fails and the subsequent automatic rollback write also fails, the proposal remains stuck in `applied` with `verify.ok === false`. The approve route permanently refuses re-approval (409) (`control-plane/src/surface/gate.ts:346`), and `isLiveUnverified` permanently blocks all future pushes to live for that entire page (`control-plane/src/index.ts:1340`, `control-plane/src/index.ts:930`).
- Failure: If a database transient failure or constraint violation interrupts an automatic rollback write, the broken, unverified page change remains live in preview, all future fast-lane proposals for that page are blocked from approval with 409, and publishing that page is blocked indefinitely. The system provides no UI or API recovery mechanism, requiring manual database intervention to restore service.
- Confidence: High
- Marked by authors: Yes (Documented in CP-12.3 with runtime failure receipt)

---

### F-11 — Failed Bracket Restore Permanently Locks Project Execution Lane with No Automatic Recovery
- Topics: BL-03, CP-13.1, CP-13.9, CP-14.2
- Evidence: BL-03 and CP-13.1 specify that a failed request whose restore failed re-claims the project's single execution lane (`control-plane/src/scr/bracket.ts:162-170`, `control-plane/src/scr/gate.ts:861-865`). Because the row remains active, both the 24-hour park TTL and 72-hour request TTL expiry sweeps refuse to touch it (`control-plane/src/scr/transition.ts:225-239`).
- Failure: If a bracket restore or restart recovery fails (e.g. filesystem permission issue or disk full error), the request enters `FAILED` while retaining the lane lock indefinitely. Because held rows never expire, all future engineering requests for that project are rejected with 503/busy indefinitely until an operator manually discovers the lock and issues a close command.
- Confidence: High
- Marked by authors: No

---

### F-12 — Ingress Routing Does Not Dynamically Apply Tenant Changes Without Manual Container Restart
- Topics: CP-08.4, EN-03.1, ES-07.1
- Evidence: EN-03.1 and CP-08.4 note that Caddy's admin API is disabled and configuration globs (`/etc/caddy/tenants/*.caddy`) are read strictly when the process boots (`enclosure/docker/Caddyfile:469-474`). Adding, modifying, or tearing down a tenant's public host updates files on disk, but Caddy never reloads them dynamically. The control plane returns `ingressRecreateRequired`, but does not trigger a reload.
- Failure: When a new tenant is registered or brought up, its public URL is unreachable (returning 404 or SSL errors) until an operator manually executes `docker restart ingress` on the host. When a tenant is torn down, its public hostname continues routing traffic to a dead internal port (returning 502 Bad Gateway) until the container is manually restarted.
- Confidence: High
- Marked by authors: Yes (Documented in CP-08.4, EN-03.1)

---

### F-13 — Shared Engine Process Imposes Head-of-Line Blocking and Cascade Aborts on All Chat Users
- Topics: CP-05.4, CP-05.8
- Evidence: CP-05.8 details that all chat and prompt requests across all users and tenants share a single sequential queue for one shared `pi` subprocess. The queue limit is 4 (`CAMPAIGNBUILDER_CHAT_QUEUE_MAX`), and each turn is bounded by a 10-minute timeout (`CAMPAIGNBUILDER_CHAT_TURN_TIMEOUT_MS`). When a turn times out, `abortWaiting` aborts all waiting turns in the queue with `TurnBusyError` / 503 `AGENT_BUSY` (`control-plane/src/index.ts:772`).
- Failure: If User A issues a complex or hung prompt, all subsequent chat requests from all marketers across all projects queue up behind User A. If more than 4 users interact with chat simultaneously, incoming requests immediately receive 503 `AGENT_BUSY`. If User A's turn times out at 10 minutes, every user waiting in the queue is aborted and returned 503, causing systemic availability collapse for conversational features.
- Confidence: High
- Marked by authors: No

---

### F-14 — Production Deploy Grants Cannot Be Managed via API and Require Undocumented Pod Restarts
- Topics: BL-02, CP-10.4, FM-03.4
- Evidence: CP-10.4 notes an unresolved contradiction: while the code comments in `control-plane/src/policy/deploy.ts:17-20` state that grants are configuration with no restart required, in reality grants are parsed strictly from the environment variable `CAMPAIGNBUILDER_DEPLOY` on the container (`control-plane/src/index.ts:2449`). Grant changes cannot be performed via the API or Foreman, are not audited, and require editing container configuration and restarting the control plane.
- Failure: An administrator cannot grant, modify, or revoke production deployment capabilities for operators or marketers through the API or UI. Grant changes require modifying deployment configuration and restarting the control plane container, causing service interruption, and granting or revoking deploy authority leaves zero audit trail on the hash chain.
- Confidence: High
- Marked by authors: Yes (Identified as a documented Drift in CP-10.4)

---

### F-15 — Unauthenticated Health Endpoint Leaks All Hosted Project Identifiers
- Topics: CP-01.4, CP-01.6, FM-07.1
- Evidence: CP-01.4 states: "GET `/api/health` Action: none, Rule: open, Shape: none... pinned: container healthcheck must not need a token". CP-01.6 notes: "its list of parts names every hosted project, so anyone who can reach it learns which projects the system hosts".
- Failure: Because `/api/health` requires no authentication and is accessible through the ingress, any anonymous external client or unauthenticated caller can query `/api/health` and enumerate the complete roster of internal client projects, tenant IDs, and dependencies hosted on the pod.
- Confidence: High
- Marked by authors: No

---

## Not judgeable from this material

1. **Sudoers Regex Compatibility in `Dockerfile.control-plane`:**
   Whether Debian Bookworm's `sudo` version natively matches `/usr/local/sbin/workspace-reassert /workspaces/[a-z0-9-]+` as an anchored regex or treats it as an invalid pattern/literal string cannot be verified without running the exact binary. If `sudo` rejects regex syntax, bracket restores for all secondary tenants immediately fail with `EACCES`.

2. **Next.js `next dev` File Watching Under UID 1005 with Sticky Directories:**
   Whether Next.js compilation and hot-reloading crash when executed by UID 1005 over directories (`lib/`, `app/`) owned by UID 1001 with sticky bits (1775) when the agent (UID 1004) creates and modifies files cannot be verified without observing filesystem notify events in the running container.

3. **Secret Broker Cryptographic Concurrency and Integrity Under High Load:**
   The internal concurrency behavior of `secret-broker.mjs` handling atomic renames of `credentials.json` and append-only hash chains over Unix domain sockets under high concurrent request volume cannot be determined from the distillation alone.

4. **Engine Subprocess Behavior on Provider Rate Limiting (HTTP 429):**
   Whether the `pi` subprocess RPC driver cleanly recovers or permanently wedges when encountering upstream model API 429 rate-limiting responses during automated fix-up loops cannot be judged without the driver's RPC message handling implementation.

5. **Tenant Outbound Egress Probe Coverage on Live Infrastructure:**
   Whether the `init-firewall.sh` owner-UID iptables filtering cleanly prevents DNS rebinding or IP-aliasing attacks from tenant scripts calling unmapped external services cannot be verified without direct testing against a multi-homed Linux network environment.

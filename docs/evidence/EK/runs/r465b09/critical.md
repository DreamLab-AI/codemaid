### F-01 — Cross-tenant source code injection and credential compromise via shared agent UID
- Topics: ES-03.1, BL-06.1, CP-17.5, EN-04.2, EN-01.4
- Evidence: In BL-06.1 and ES-03.1, the agent process across all tenants executes under the exact same operating system user (`uid 1004`, `enclosure/docker/workspace-seed.sh:119`), and the control plane container mounts all tenant workspaces simultaneously (`enclosure/docker/compose.pod.yaml:531`, `535`). The tool-call contract-scope gate checks only write tools, not shell execution (`control-plane/src/scr/contract-scope.ts:273`, `control-plane/pi-extensions/no-install-gate.ts:350`), and `executor.ts:1181-1191` only diffs the current project's workspace directory against its restore point.
- Failure: When an engineering request runs for Tenant B, the agent can issue a shell command writing arbitrary code into Tenant A's workspace (`/workspaces/campaignbuilder/...`). Because Tenant A's container serves that workspace directly via `next dev` (`enclosure/docker/web-surface-entrypoint.sh:537`), Tenant A hot-compiles and executes the payload in its own runtime container, exposing Tenant A's `DATABASE_URL` with `BYPASSRLS` privileges and environment secrets (`enclosure/docker/web-surface-entrypoint.sh:374`).
- Confidence: High
- Marked by authors: Yes (Marked as `warn` in BL-06.1 and documented as a known limit in ES-03.1 and ADR-019)

### F-02 — Total machine disaster recovery impossible due to host-bound signing key requirement
- Topics: EN-10.1, EN-10.3, EN-10.6, BL-05.2
- Evidence: `campaignbuilder-backup` signs the manifest using HMAC-SHA256 with `CAMPAIGNBUILDER_BACKUP_KEY_FILE` (`enclosure/tools/backup/backup.mjs:546`, `646`), which is generated and stored locally on the VM host (`enclosure/docker/seed-bootstrap-secrets.sh:72`, `enclosure/docker/compose.pod.yaml:934`). `restore` mandates verifying the signature before any decryption or restoration, exiting with code 6 if invalid (`enclosure/tools/backup/backup.mjs:778-782`). In BL-05.2, authors note: "the archive's signing key is a host file the operator holds, so it shares a fate with the machine (`backup.mjs:56-61`)".
- Failure: Following a catastrophic host failure (hardware destruction or accidental VM deletion), the operator has the offsite `.tar` archive and their private `age` decryption key. However, because the signing key was destroyed with the host and is never stored offsite or in the archive, `campaignbuilder-backup restore` immediately aborts with Exit 6. The offsite encrypted backups cannot be restored on a new machine.
- Confidence: High
- Marked by authors: Yes (Identified as limit `x4` in BL-05.2, but the fatal consequence on `restore` is unmarked in EN-10)

### F-03 — Live production data destruction on migration check failure via blind `pg_restore --clean`
- Topics: CP-18.5, CP-15.8, CP-18.6
- Evidence: `applyMigration` creates a full database dump (`pg_dump`) prior to migration (`control-plane/src/scr/migration-runner.ts:233-234`). In CP-18.6, the brief acknowledges: "tenant writes are not paused (`control-plane/src/scr/migration.ts:143-147`)". If a migration commits but post-migration verification fails (or `psql` exits with anything other than server rejection code 3), the runner executes `pg_restore --clean` of the pre-migration dump (`control-plane/src/scr/migration-runner.ts:284`, `358-364`).
- Failure: While a schema migration is in progress (with timeouts up to 15 minutes, `control-plane/src/scr/migration-runner.ts:42-46`), visitors to the live site continue writing data (form submissions, leads, user accounts). When a migration check fails, `pg_restore --clean` wipes the entire database back to the pre-migration dump, permanently obliterating all customer transactions and data collected during the migration window.
- Confidence: High
- Marked by authors: Partially marked (The runner logs discarded writes, but the catastrophic data loss to live users is treated as normal rollback)

### F-04 — Unfenced network and container access during tenant first-boot `npm ci`
- Topics: EN-04.3, EN-08.5, EN-02.2
- Evidence: `tenant-provision-entrypoint.sh:219` runs `npm ci` as `surface` (uid 1005) inside the tenant container on `pod-net`. In EN-04.3 and EN-08.5, the tenant firewall (`init-firewall.sh`) is not installed until `web-surface-entrypoint.sh:469`, which runs after the provision entrypoint finishes (`enclosure/docker/tenant-provision-entrypoint.sh:330`).
- Failure: When a new tenant repository is provisioned, any package dependency containing a `postinstall` lifecycle script executes arbitrary code as `surface` before the egress firewall or DNS restriction exists. It has unrestricted internet access and direct network access to every container on `pod-net`, including `campaignbuilder-db` (Postgres) and sibling tenants' dev servers, bypassing all egress controls.
- Confidence: High
- Marked by authors: Partially marked (ADR-019 mentions `npm ci` runs before the fence, but systemic pod-net exposure is unmarked)

### F-05 — Session hijacking risk: Ingress deliberately passes admin session cookie to tenant code
- Topics: EN-03.1, EN-03.2, WS-01.3, ES-03.1
- Evidence: `(tenant_session_strip)` deletes the session cookie (`cb_session`) from all tenant proxies (`enclosure/docker/Caddyfile:114-116`) except `@tenant_intake` (`/api/ai/structural-change`, `enclosure/docker/Caddyfile:211-215`). The tenant app at `web-surface/app/api/ai/structural-change/route.ts:45-57` receives the raw `cb_session` cookie containing the marketer/admin's Google `id_token`.
- Failure: The tenant container runs `next dev` over an agent-editable workspace. An agent (or attacker) editing the tenant codebase can inject middleware or route logic to log or forward incoming `cb_session` cookies. When an admin visits the tenant admin or clicks the change panel, their authenticated control-plane bearer credential is handed directly to tenant code, allowing account takeover.
- Confidence: High
- Marked by authors: No (The exception is marked as intentional, but the vulnerability of handing admin bearer tokens to agent-editable code is unaddressed)

### F-06 — Self-approval bypass: Missing `requestedBy` allows admins to approve their own escalations
- Topics: CP-13.8, CP-02.3, CP-13.1, BL-02
- Evidence: The system claims: "nobody approves their own ask... the approve route refuses the admin who asked with SELF_APPROVAL" (`control-plane/src/index.ts:1818-1823`, `control-plane/src/auth/authorise.ts:148`). However, CP-13.8 states: "A row with no requestedBy answers to the role rule alone. A row that predates requestedBy reads false for everyone (`control-plane/src/index.ts:1738-1742`, `control-plane/src/index.ts:338-341`)". Furthermore, `ScrRow` does not strictly require `requestedBy` (`control-plane/src/scr/types.ts:118`).
- Failure: Any request created without `requestedBy` evaluates `email === row.requestedBy` as false. The self-approval check in `index.ts:1818` passes, allowing an administrator who filed a dangerous Tier 1 or Tier 2 code change to approve and deploy their own request without peer review.
- Confidence: High
- Marked by authors: No

### F-07 — Audit desynchronization: Failed state changes permanently recorded as successful
- Topics: CP-03.6, CP-03.2, CP-01
- Evidence: The core guarantee is "no record, no change" (`BL-01.1`, `BL-05.2`, `CP-03.6`). If an act fails after a durable audit record is appended, the system attempts to append a compensating event (`PROJECT_REGISTRATION_ROLLED_BACK`, `AUTONOMY_CHANGE_FAILED`, etc.) unchecked (`control-plane/src/index.ts:239-240`, `2175`, `3310`).
- Failure: When an operation fails due to system exhaustion (disk full, I/O errors, crash), the planned change event has already been committed to the hash chain, but the compensating event fails to append or is lost. The immutable, hash-linked audit spool permanently records that a sensitive state change succeeded (e.g. tenant registered, slot created, dial moved), when the mutation was actually aborted.
- Confidence: High
- Marked by authors: Partially marked (CP-03 notes the compensating append is unchecked)

### F-08 — Off-box audit reliability compromised by unmonitored in-memory queue eviction
- Topics: CP-03.2, CP-03.7, BL-05.2
- Evidence: The authors claim the audit copy in the ingest sidecar prevents unkeyed hash-chain rewriting (`control-plane/src/audit/chain.ts:29-35`, `control-plane/src/audit/spool.ts:489-497`). However, `createHttpShipper` queues events in an in-memory array (`control-plane/src/audit/ship.ts:148`) capped at 10,000 items (`control-plane/src/audit/ship.ts:98`). If the sidecar is restarting or backlogged, the oldest events are dropped without notice (`control-plane/src/audit/ship.ts:13-14`, `205-207`), and "nothing re-sends them".
- Failure: During an extended container restart, network hiccup, or high-volume burst, the memory queue silently evicts events. The off-box sidecar permanently loses chunks of the audit trail. Because the local spool uses an unkeyed hash chain, an attacker with container root can rewrite the local spool across the dropped window, and the off-box sidecar cannot detect the tampering because it never received the original hashes.
- Confidence: High
- Marked by authors: Yes (Marked as "Tension" / "Lost from the copy")

### F-09 — Irreversible project lane deadlock on restore failure in `rollbackBracket` and `close`
- Topics: CP-13.8, CP-13.2, CP-15.2, CP-14.2
- Evidence: In CP-13.8 and CP-15.2, if `store.restore(ref)` throws during a rollback or close, `active` is re-claimed, transitioning to `FAILED` with `active: true` (`control-plane/src/scr/bracket.ts:158-175`, `control-plane/src/scr/gate.ts:858-866`). In CP-13.2, "a FAILED row that still holds the lane... never expires (`control-plane/src/scr/transition.ts:225-227`, `239`)".
- Failure: If a bracket restore encounters a disk I/O error, permission glitch, or git corruption during rollback or close, the project lane is re-locked with `active: true`. The 72-hour TTL sweep explicitly ignores it, the gate refuses new requests with `409` (`control-plane/src/scr/executor.ts:321`), and repeated close attempts fail with the same restore error. The project lane is permanently deadlocked until manual database/ledger surgery is performed.
- Confidence: High
- Marked by authors: Yes (Marked as an intentional "safety reversal", but consequences in production are unmitigated)

### F-10 — Chat bubble browser refresh leaves untracked running requests and deadlocks the project lane
- Topics: CP-06.1, CP-14.1, BL-02, FM-03
- Evidence: In BL-02 and CP-06.1, when a marketer submits an engineering turn via the chat bubble, the request is dispatched detached (`control-plane/src/index.ts:1124-1141`), taking up to 25 minutes. Status polling exists purely in transient browser memory (`control-plane/public/bubble.js:44`, `237`).
- Failure: If a user closes or reloads their browser tab while a 20-minute engineering task is building, the polling loop terminates permanently. When the user returns and types in chat, a new request is generated (`control-plane/src/index.ts:1124`), which immediately fails with "another request in flight" (`control-plane/src/scr/executor.ts:321`). The chat bubble provides no interface to discover, inspect, approve, or cancel the orphaned in-flight request, locking the marketer out of chat edits until operator intervention in Foreman.
- Confidence: High
- Marked by authors: Yes (Marked in BL-02 as "Where delivery stops: the follow lives in page memory")

### F-11 — Single JSON parse error causes unrecoverable control-plane crash loop and pod-wide outage
- Topics: CP-01.7, CP-04.4, CP-12.5, CP-13.9
- Evidence: The control plane adheres to a strict "corrupt-file posture": `runtime-projects.json`, `scr-ledger.<id>.json`, `supabase-slots.json`, `world.json`, and `proposals.json` will refuse to boot if malformed (`control-plane/src/project/runtime-store.ts:107`, `control-plane/src/scr/gate.ts:268`, `control-plane/src/vault/store.ts:246`, `control-plane/src/world/store.ts:386`, `control-plane/src/surface/store.ts:463`).
- Failure: An ungraceful host shutdown, storage hiccup, or truncated write during atomic rename on any of these five files results in unparseable JSON. Upon restart, the control plane immediately crashes with Exit 1 (`control-plane/src/index.ts:4396`). Because ingress routes depend on the control plane for authentication (`enclosure/docker/Caddyfile:121`), all tenant campaign previews and administrative consoles across all hosted projects crash and return 502 Bad Gateway until an engineer manually edits the JSON files.
- Confidence: High
- Marked by authors: Yes (Described as intentional fail-closed posture, but creates a single-point-of-failure boot loop)

### F-12 — Configuration tab and rebuild plan are non-functional simulations on live systems
- Topics: FM-06.1, FM-06.2, FM-06.3, CP-01.8
- Evidence: In FM-06.1 and FM-06.2, `ConfigTab` is completely read-only on live builds (`foreman/src/features/config/controls.tsx:242-244`). The control plane registers `GET /api/config` but has no `PUT` route (`control-plane/src/index.ts:2579`). Staging edits and the "Rebuild Plan" run only in demo mode (`foreman/src/features/config/ConfigTab.tsx:37`) using arbitrary 950ms timers (`foreman/src/features/config/RebuildPlan.tsx:62`).
- Failure: Operators expecting Foreman's "Configuration" tab to manage infrastructure settings (such as toolchains, network allowlists, or identity configs) cannot change anything on live pods. Applying changes locally in demo mode simulates success without ever persisting to `foreman.toml`. Configuration changes require manual SSH access, out-of-band file editing, and container recreation, directly contradicting the console's UI affordances.
- Confidence: High
- Marked by authors: Partially marked (Marked as "Drift" and "Debt" in ADR-004 addenda)

### F-13 — Multi-tenant content publishing completely blocked for all secondary tenants
- Topics: BL-06.1, BL-06.2, CP-15.7, CP-07.4
- Evidence: In BL-06.1, BL-07, and CP-15.7: "The host's editing source is the default project's alone... a tenant has no fast lane and its content push is refused, named, before anything is read (`control-plane/src/project/registry.ts:654-655`, `control-plane/src/index.ts:1327-1333`)". The data push route aborts with 409 `CROSS_PROJECT_PUSH` if `target !== SURFACE_SOURCE_PROJECT_ID` (`control-plane/src/index.ts:1327`).
- Failure: Despite marketing the system as a multi-tenant pod (`ADR-019`), secondary tenants (such as `civic-quest`) cannot publish any marketing content or page configuration edits to production. Attempting to push content fails with 409, completely breaking tenant self-service for all projects except the default hard-coded tenant.
- Confidence: High
- Marked by authors: Yes (Marked as "Open" and "Backlog" in ADR-019 second-tenant addendum)

### F-14 — Unreviewed workspace contamination retained after promote refusal
- Topics: CP-15.3, CP-13.7
- Evidence: In CP-15.3, `runPromote` checks if the staged tree matches the `reviewedChangeSet` (`control-plane/src/scr/gate.ts:636`, `control-plane/src/scr/promote-git.ts:185`). If files were modified or planted between review and promotion (e.g. by concurrent processes or another tenant's agent), promotion is refused with 409 `CHANGED_SINCE_REVIEW` (`control-plane/src/scr/gate.ts:604`). However, the transition reverts to `REPORTED` with NO ROLLBACK (`control-plane/src/scr/gate.ts:598-601`).
- Failure: When an unexpected or malicious file is added to the workspace after review, the deployment is blocked, but the dirty file is left directly in the workspace. Because the tenant dev server serves this directory directly (`enclosure/docker/web-surface-entrypoint.sh:537`), the unverified/planted file continues to be executed and served in the authenticated preview environment indefinitely until manually deleted.
- Confidence: High
- Marked by authors: Yes (Marked in CP-15.3 note as "what remains: the planted file stays in the tree")

### F-15 — Transient network partition and 502 errors during tenant-to-control-plane boot race
- Topics: EN-08.5, EN-04.3, EN-02.2
- Evidence: In EN-08.5, the tenant container `init-firewall.sh` fences `surface` (uid 1005) using an allowlist of resolved IP addresses. On startup, the control plane container starts only after the default tenant is healthy (`enclosure/docker/compose.pod.yaml:549`). Thus, the tenant entrypoint cannot resolve the control plane's IP when first installing the fence.
- Failure: If the background refresh loop encounters a delay, fails, or defaults to the standard 300-second interval (`enclosure/docker/web-surface-entrypoint.sh:241`), the tenant container cannot resolve or connect to the control plane. Incoming requests to the tenant change panel or `/api/ai/structural-change` that make outbound calls to `GET /auth/me` fail with 502/network timeouts for up to 5–10 minutes after every roll or reboot.
- Confidence: Medium (inferred from timing constraints and documented EW-1 walk defect)
- Marked by authors: Yes (Documented as defect EW-1 in ADR-019 addendum)

---

## Not judgeable from this material

1. **Sudoers Regex Support on Debian 12**: Whether the `/etc/sudoers.d` drop-in regex `^/workspaces/[a-z0-9-]+$` in `Dockerfile.control-plane:235` actually matches command arguments under Debian's installed `sudo` binary, or if it is treated as an invalid pattern / literal string that causes runtime `sudo` execution failures during `workspace-reassert`.
2. **Resource Starvation Under Concurrency**: Whether a 4-vCPU 16GB host can survive concurrent execution of headless Chromium (`ws-render` / `verify-surface`), the `next dev` compilation server, Vitest test suites, and the Pi LLM agent without triggering Linux kernel OOM kills.
3. **Behavior of Live `pg_restore` Against Complex Schemas**: The exact failure modes and data corruption risks of `pg_restore --clean` against production Supabase instances containing active foreign keys, partitioned tables, extensions, and trigger-generated records.
4. **Target Repository CI/CD Pipeline Semantics**: How downstream target GitHub repositories handle pull requests created by `promote-git.ts`, specifically whether automated CI/CD deployment runs tests, applies migrations, or handles merge conflicts safely.
5. **Disk Full Behavior on SQLite/JSON Stores Under Stress**: How the custom `writeLedger` and `persistRuntimeProject` temporary-file-and-rename logic behaves when the underlying disk filesystem reports `ENOSPC` midway through writing state files.

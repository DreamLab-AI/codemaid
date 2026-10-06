### F-01 — Same-origin preview and control-plane API collocation enables full administrative takeover via XSS
- Topics: EN-03.1, EN-03.2, ES-03.1, CP-02.7
- Evidence: The public ingress hosts both the campaign preview (`/@campaignbuilder_preview`, `enclosure/docker/Caddyfile:147-151`) and the control plane API (`/@control`, `enclosure/docker/Caddyfile:261-266`) on the exact same origin (`enclosure/docker/Caddyfile:131`). The ingress security policy explicitly retains `script-src 'self' 'unsafe-inline'` (`enclosure/docker/Caddyfile:56`). The preview serves hot-reloaded code that the AI agent actively edits (`enclosure/docker/compose.pod.yaml:182-184`, `enclosure/docker/web-surface-entrypoint.sh:530-537`). The control plane relies on the browser's `cb_session` cookie on that same origin (`control-plane/src/auth/verify.ts:287`, `control-plane/src/auth/login.ts:313-314`).
- Failure: An agent editing a campaign page introduces an inline script or DOM sink into a previewed component. An administrative operator inspects the preview at `/v/<shortcode>`. The injected script runs under the operator's origin, issues `fetch('/api/...', { credentials: 'same-origin' })`, and takes any administrative action (approving database migrations, creating projects, changing dials, or reading system metadata) using the administrator's active session.
- Confidence: high
- Marked by authors: no

### F-02 — Shared agent UID across tenants allows cross-tenant arbitrary code execution
- Topics: BL-06.1, ES-03.1, EN-01.1, EN-04.2, CP-17.5
- Evidence: The control plane mounts all tenant workspaces simultaneously into its container (`enclosure/docker/compose.pod.yaml:531-535`). The agent process runs under a single system UID (1004) across all tenants (`enclosure/docker/Dockerfile.control-plane:180-186`, `enclosure/docker/workspace-seed.sh:119`). Shell tool calls (`bash`) are only evaluated by the no-install gate for package manager verbs and git commands (`control-plane/pi-extensions/no-install-gate.ts:280-323`); they are never checked against filesystem contract scope (`control-plane/src/scr/contract-scope.ts:273`). Post-turn git diff inspection only verifies the requesting tenant's repository (`control-plane/src/scr/executor.ts:1181-1191`).
- Failure: While servicing a prompt on Tenant A, the agent executes a shell command writing to `/workspaces/tenant-b/app/page.tsx`. Because the agent owns the surface directories across all workspaces under UID 1004, the write succeeds. Tenant A's scope checker never inspects Tenant B's repository. Tenant B's running `next dev` server immediately compiles and executes the modified file with Tenant B's database and Supabase credentials in its runtime environment.
- Confidence: high
- Marked by authors: yes (partially acknowledged in BL-06.1 Mermaid diagram and ES-03.1 note `XT`)

### F-03 — Global vault references permit cross-tenant credential theft at registration
- Topics: BL-06.1, CP-07.4, CP-08.1, CP-09.2, CP-10.5
- Evidence: Credential references in the vault (`supabaseConnRef`, `githubPatRef`, `modelKeyRef`) are stored in a single flat namespace in `VaultStore` without tenant ownership tags (`control-plane/src/vault/store.ts:110-120`). `POST /api/projects` accepts any reference present in the vault (`control-plane/src/index.ts:3178-3179`). During tenant provisioning, `resolveSurfaceEnv` resolves the given reference against the global store without verifying project ownership (`control-plane/src/vault/resolve.ts:51-56`).
- Failure: An administrator registers Tenant B and supplies `supabaseConnRef: "tenant-a"`. The control plane resolves Tenant A's database connection and writes it into `/run/campaignbuilder/surface-env/env` inside Tenant B's isolated volume. Tenant B's runtime container boots with full read/write access to Tenant A's database.
- Confidence: high
- Marked by authors: yes (marked as "Open question" in BL-06.1 and CP-08.1)

### F-04 — Unauthenticated `/api/health` endpoint leaks all tenant project IDs
- Topics: BL-01, CP-01.4, CP-01.6, EN-03.1
- Evidence: `GET /api/health` requires no authentication (`control-plane/src/index.ts:613`, `control-plane/src/index.auth.test.ts:77`). The public ingress forwards all `/api/*` requests directly to the control plane without authentication or forward-auth (`enclosure/docker/Caddyfile:261-266`). The health checker inspects every registered project in the system and emits an explicit check object named `tenant:<id>` for each one (`control-plane/src/health/checks.ts:187-192`).
- Failure: An unauthenticated external attacker requests `https://<domain>/api/health` and obtains a complete enumeration of all tenant project identifiers hosted on the cluster.
- Confidence: high
- Marked by authors: no

### F-05 — Database rollback via `pg_restore --clean` destroys concurrent tenant writes
- Topics: CP-18.5, CP-15.8, BL-05.2
- Evidence: The migration system claims tenant writes are not paused during execution (`control-plane/src/scr/migration.ts:143-147`). If an applied migration fails post-apply verification or fails execution outside of `psql` exit code 3, the runner triggers a full restore using `pg_restore --clean` against the pre-migration snapshot (`control-plane/src/scr/migration-runner.ts:284`, `:358-364`).
- Failure: While a migration is running and being verified (bounded by a 15-minute timeout), public visitors submit leads or update campaign records. When the migration fails post-apply verification, `pg_restore --clean` drops and replaces database objects back to the pre-migration dump, irrevocably wiping out all visitor data written during the migration window.
- Confidence: high
- Marked by authors: yes (partially acknowledged in CP-18.5 notes stating "writes since the restore point were discarded")

### F-06 — Failed rollback writes leave unverified page changes that are subsequently published
- Topics: CP-12.3, CP-12.6, CP-15.7
- Evidence: When an applied page proposal fails render verification in `enforce` mode, the previous configuration is written back. If that rollback write fails, the proposal remains marked as `applied` with an error message (`control-plane/src/surface/gate.ts:267-271`). When `pushlive` is called, it pushes the current database row directly from the database (`control-plane/src/scr/substrate.ts:113-125`). While `isLiveUnverified` blocks publishing that specific unverified proposal, once a subsequent proposal on the same page is applied and verified, the page is deemed clean, and publishing pushes the entire current row.
- Failure: Proposal 1 modifies a page, fails render verification, and its rollback write fails. The invalid content remains in the preview database. Proposal 2 modifies a different section of the same page and passes render verification. A user initiates `/pushlive`; the database row containing the corrupted changes from Proposal 1 is published live to production.
- Confidence: high
- Marked by authors: no

### F-07 — Unkeyed audit hash chain and co-located sidecar allow undetectable history tampering
- Topics: BL-05.2, CP-03.2, CP-03.7
- Evidence: The audit hash chain uses unkeyed SHA-256 (`control-plane/src/audit/chain.ts:79`). Daily checkpoint files stored on the local disk are unkeyed and unsigned (`control-plane/src/audit/spool.ts:17-22`, `:489-497`). The audit sidecar is co-located on the same physical Docker daemon (`enclosure/docker/compose.pod.yaml:943`). The shipping queue is in-memory and drops all queued events on restart or when queue size exceeds 10,000 (`control-plane/src/audit/ship.ts:13-14`, `:205-207`).
- Failure: A compromised process running as `app` rewrites historical audit logs on disk, recomputes the SHA-256 hashes from genesis, and updates the local checkpoints file without tripping verification. If the host restarts during heavy traffic, in-flight audit events are permanently dropped from the sidecar without resend or alert.
- Confidence: high
- Marked by authors: yes (marked as Tension in CP-03.2 and "WHAT IT DOES NOT PROMISE" in BL-05.2)

### F-08 — Restore failures permanently deadlock the project's single engineering lane
- Topics: CP-13.1, CP-13.2, CP-13.8, CP-13.9
- Evidence: Each project maintains a single in-flight engineering lock (`control-plane/src/scr/types.ts:118`, `control-plane/src/scr/transition.ts:184`). If a bracket restore throws during a rollback, close, or restart recovery, the row enters `FAILED`, but the lock is explicitly re-claimed (`control-plane/src/scr/bracket.ts:162-170`, `control-plane/src/scr/gate.ts:858-866`). Rows in `FAILED` with an active lock never expire (`control-plane/src/scr/transition.ts:225-226`, `:239`). Subsequent calls to `closeRequest` re-attempt `restore`, fail again, and re-assert the lock (`control-plane/src/scr/gate.ts:850-865`).
- Failure: A corrupt git object or file permission issue causes `git read-tree` to fail during a rollback. The row enters `FAILED` and holds `active`. The operator cannot close it, the background sweeper cannot expire it, and server restarts re-assert the lock on recovery. The project's engineering lane is permanently deadlocked until an operator manually mutates `scr-ledger.<id>.json` on disk.
- Confidence: high
- Marked by authors: no

### F-09 — Shared engine timeout cascades into mass denial-of-service for all queued users
- Topics: CP-05.4, CP-05.8
- Evidence: All chat and prompt requests share a single `pi` engine subprocess governed by `turnQueue` (`control-plane/src/index.ts:739`). The queue is bounded to 4 waiting requests (`control-plane/src/engine/turn-queue.ts:34-40`). Each turn is allowed up to 10 minutes (`control-plane/src/index.ts:673`, `:762`). If a running turn times out, `abortWaiting` aborts all waiting requests, kills the engine, and returns 503 `AGENT_BUSY` (`control-plane/src/index.ts:772-776`, `control-plane/src/engine/turn-queue.ts:78-82`).
- Failure: A user triggers a prompt that hangs for 10 minutes. While it runs, 4 subsequent users enter the queue; any further users receive an immediate 503. When the 10-minute timeout expires, the running prompt fails, and all 4 queued requests are aborted with 503 errors. A single hung prompt denies service to the entire user base for 10 minutes and discards all pending work.
- Confidence: high
- Marked by authors: no

### F-10 — Non-transactional backup restore leaves partially overwritten targets on failure
- Topics: EN-10.1, EN-10.3
- Evidence: `restore` performs three sequential extractions without a transaction: Write 1 restores state, Write 2 restores world data, and Write 3 writes the database dump (`enclosure/tools/backup/backup.mjs:957-959`). There is no rollback mechanism if a later step fails (`enclosure/tools/backup/backup.mjs:973-974`). Targets must be completely empty prior to extraction (`enclosure/tools/backup/backup.mjs:952-956`).
- Failure: During disaster recovery, Write 1 extracts the state directory. Write 2 throws an unhandled error (e.g. out of disk space). The process exits; the state directory remains partially populated, while world data is missing. Because the target directory is no longer empty, re-running `restore` immediately exits with code 8, leaving the system in an unrecoverable, inconsistent state.
- Confidence: high
- Marked by authors: yes (acknowledged in EN-10.3 as "FAILURE BOUNDARY: a throw in write 2 or 3 leaves the earlier targets populated")

### F-11 — Port and triple modifications go live unrecorded upon audit failures
- Topics: CP-03.6, CP-07.3, BL-01.1
- Evidence: The system commits to "record first, act second" for all state changes (`control-plane/src/index.ts:230`). However, port and triple updates register with the in-memory registry *before* appending to the audit log (`control-plane/src/index.ts:3649`, `:3651`, `:3772`, `:3778`). If audit appending fails, `putBackUnrecorded` attempts to revert the change. If the revert fails, the change remains "live but unrecorded until restart" (`control-plane/src/index.ts:3581-3585`).
- Failure: An administrator updates a project triple or port while the audit spool is degraded. The in-memory registry updates immediately. The subsequent audit append fails, and the reversion step encounters an error. The live registry continues running with the modified configuration, routing traffic or applying new compiler rules, without any record on the audit trail.
- Confidence: high
- Marked by authors: yes (acknowledged in CP-03.6 and CP-07.3 as an accepted exception)

### F-12 — Keys exceeding 200 characters silently bypass idempotency enforcement
- Topics: CP-13.4
- Evidence: When processing requests with an `Idempotency-Key` header, the server scopes the key by route, but drops any key longer than 200 characters, returning `undefined` (`control-plane/src/index.ts:381`). When the key is `undefined`, the server falls back to non-idempotent execution (`control-plane/src/scr/transition.ts:463`).
- Failure: A client sends a standard structured or cryptographically signed idempotency token that exceeds 200 characters. Instead of returning a 400 validation error, the server silently ignores the key. When network timeouts trigger automatic client retries, the server executes duplicate operations (such as approving, declining, or closing requests multiple times).
- Confidence: high
- Marked by authors: no

### F-13 — `POST /auth/logout` lacks CSRF protection, enabling forced session termination
- Topics: CP-02.7, EN-03.1, EN-03.7
- Evidence: `POST /auth/logout` validates no origin, no referer, and requires no CSRF token (`control-plane/src/auth/login.ts:322-327`). The public ingress routes `/auth/*` directly without authentication (`enclosure/docker/Caddyfile:261-266`). The handler immediately deletes the `cb_session` cookie across both the marketer preview and operator console (`control-plane/src/auth/login.ts:325`).
- Failure: An attacker embeds an unauthenticated cross-origin POST request (`<form action="https://<pod>/auth/logout" method="POST">`) on an external website. An operator or marketer visiting the attacker's page has their active session terminated across both workspaces without warning or consent.
- Confidence: high
- Marked by authors: no

### F-14 — Hardcoded single-tenant dependencies prevent second-tenant operation
- Topics: BL-06.2, CP-08.6, CP-12.4, CP-15.5, CP-15.7
- Evidence: Despite architectural claims of single-pod multi-tenancy, critical paths are hardcoded to the default tenant:
  1. Content publishing (`pushlive`) explicitly rejects non-default projects with a 409 (`control-plane/src/index.ts:1327-1333`).
  2. Fast-lane chat editing only operates against the default project (`control-plane/src/index.ts:1009-1011`).
  3. Tenant page validation has no schema field for layout rules, defaulting to `lib/layout-rules.ts` or the default project's environment (`control-plane/src/project/config.ts:49-55`, `control-plane/src/surface/page-validator.ts:141`).
  4. The surface overlay proxy only fronts the default tenant (`control-plane/src/project/caddy-tenants.ts:76-77`).
  5. The ingress hardcodes the default tenant's specific API route list (`enclosure/docker/Caddyfile:217-222`).
- Failure: An operator provisions a second tenant (`civic-quest`). The tenant cannot use conversational page editing, cannot publish content live, cannot use the visual overlay, and is forced to conform to the default tenant's specific internal route structure.
- Confidence: high
- Marked by authors: yes (partially acknowledged in BL-06.2 and CP-08.6 as a second-tenant backlog)

### F-15 — Approval binding fails to hash the working tree files
- Topics: CP-13.3, CP-15.3
- Evidence: An engineering approval is bound to `ApprovalBinding` (`control-plane/src/scr/transition.ts:302`). `diffHash` is calculated strictly over `row.structuralDelta`, `row.plan`, and `row.blessedPlan` (`control-plane/src/scr/transition.ts:365-369`). In live engineering requests, these fields start empty (`control-plane/src/index.ts:1574`). The actual files in the workspace tree are explicitly omitted from `diffHash` (`control-plane/src/scr/transition.ts:345-354`).
- Failure: An operator reviews and approves a change. If files in the workspace are modified out-of-band (e.g. by a secondary process or script) while keeping `baseRevision` and `configVersion` unchanged, `staleApprovalFields` reports that the approval is still valid. While `runPromote` later halts deployment if the change set does not match `reviewedChangeSet`, the system remains in a misleading state where an invalid approval is displayed as current.
- Confidence: high
- Marked by authors: yes (acknowledged as "THE HONEST LIMIT" in CP-13.3)

---

## Not judgeable from this material

1. **Kernel Egress Bypass via IPv6/Packet Smuggling**: Whether the `init-firewall.sh` iptables rules can be bypassed inside the container via raw socket creation, packet encapsulation, or DNS tunneling, since network namespace routing and Docker daemon daemon-level bridge filtering rules are not provided.
2. **PostgreSQL Connection Pool Exhaustion under Concurrent Deploys**: Whether rapid `/pushlive` operations or database schema exports exhaust the PostgreSQL connection pool or cause deadlocks during `pg_dump` operations under production transaction volume.
3. **Memory Limits during Massive `pg_dump` Backups**: Whether `dumpDatabase` in `backup.mjs` buffers the entire database dump into Node.js heap memory, which would trigger an unrecoverable out-of-memory crash when backing up a database exceeding 1–2 GB.
4. **Third-Party OIDC Token Validation Edge Cases**: The exact handling of token revocation, clock skew tolerances, and JWKS key rotation failures within the `jose` library implementation during network partitions with Google's identity servers.
5. **Full Disaster-Recovery Execution from Bare Metal**: Whether an operator can successfully execute a complete cold restore of all tenant services, databases, and ingress certificates starting from a blank VM using only the backup archive, without relying on uncommitted local environment variables.

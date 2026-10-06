### F-01 — Shared Agent UID Enables Cross-Tenant Workspace Compromise and Planted Code
- Topics: BL-06, CP-15.3, CP-17, EN-04.2, ES-03.1
- Evidence: ADR-019 addendum acknowledges that per-project UIDs are unbuilt and the agent UID is shared across all tenants (`docs/reference/adr/ADR-019-single-pod-multi-project-tenancy.md:694`, `705-708`). The workspace seed assigns ownership of surface directories across every tenant workspace to the exact same numeric UID 1004 (`enclosure/docker/workspace-seed.sh:60`, `119`). The control plane mounts all tenant workspaces as sibling directories in the same container (`enclosure/docker/compose.pod.yaml:524-531`).
- Failure: When the AI agent executes a change on Tenant A, its process runs as UID 1004. Because it shares UID 1004 with Tenant B's files, it can read, modify, or plant files in `/workspaces/civic-quest` while ostensibly working inside `/workspace`. The per-turn contract-scope check only checks diffs within Tenant A's workspace (`control-plane/src/scr/executor.ts:1181-1195`), making cross-workspace modifications completely invisible to the gating logic.
- Confidence: high
- Marked by authors: yes (ADR-019 addendum; CP-15.3 notes the cross-tenant plant finding)

### F-02 — Flat `pod-net` Bridge Allows Direct Ingress-Bypass Between Tenant Containers
- Topics: BL-06, EN-02.2, EN-03.1, ES-03.1
- Evidence: All tenant containers, the database, the overlay, and the control plane attach to a single bridge network (`enclosure/docker/compose.pod.yaml:707`, `1065-1066`). While Caddy ingress strips identity headers and enforces `tenant_preview_auth` on external requests (`enclosure/docker/Caddyfile:67-78`, `121-124`), inter-container traffic on `pod-net` does not traverse Caddy.
- Failure: Compromised code running in Tenant A's container (e.g. via an agent-modified route handler or SSR exploit) can connect directly to Tenant B's container on port 3000 over `pod-net`. This bypasses Caddy's session stripping, route matchers, and authentication checks, accessing internal Next.js dev server endpoints without credentials.
- Confidence: high
- Marked by authors: yes (ADR-019 addendum; ES-03.1 threat model)

### F-03 — Project Registration Permits Arbitrary Cross-Tenant Credential Vault Reference Binding
- Topics: BL-06, CP-07.2, CP-08.1, CP-09.1
- Evidence: During project registration, `POST /api/projects` accepts arbitrary strings for `vaultRef`, `supabaseConnRef`, and `githubPatRef` (`control-plane/src/index.ts:3178-3179`). The registry resolves references against the vault without verifying that the referenced secret belongs to or was created for the requesting project (`control-plane/src/project/registry.ts:316-317`).
- Failure: An administrator registering Tenant B can set `githubPatRef` or `supabaseConnRef` to the identifier of Tenant A's existing secrets. During container initialization, `materialiseSurfaceEnv` and `materialiseProvisionEnv` resolve the secret from the vault and write Tenant A's live credentials into Tenant B's private `/env` hand-off file (`control-plane/src/vault/inject.ts:122`, `control-plane/src/vault/provision-inject.ts:82`).
- Confidence: high
- Marked by authors: yes (BL-06 Open question; CP-08.1 Open question)

### F-04 — Agent Shell Execution Completely Bypasses Contract Scope Tool-Call Enforcement
- Topics: CP-05.3, CP-17.1, ES-03.1
- Evidence: `enforceContractScope` explicitly limits enforcement to write tools (`control-plane/src/scr/contract-scope.ts:273`). Shell executions are routed past the check: "Shell and read tools never reach the scope check" (`control-plane/src/scr/contract-scope.ts:253-254`). While git diff checks inspect the workspace at turn end, shell commands execute interactively during the turn.
- Failure: During an active execution turn, the agent can issue shell commands (`bash`, scripts) that write outside granted paths (such as into `/tmp`, sibling mounted workspaces, or modifying files via shell redirection). Furthermore, since `next dev` continuously serves the workspace live without rebuilding (`enclosure/docker/compose.pod.yaml:182-184`), malicious shell-written modifications are executed immediately on the live preview server while the turn is in progress.
- Confidence: high
- Marked by authors: no

### F-05 — Tenant Provisioning Executes Unfenced `npm ci` with Outbound Internet Access
- Topics: EN-04.3, EN-08.5, ES-03.1
- Evidence: In `tenant-provision-entrypoint.sh:217-219`, `npm ci` runs as the runtime user (`surface`) before the container environment is sealed. The egress firewall and DNS filtering are initialized only in `web-surface-entrypoint.sh:433-470`, which executes after `tenant-provision-entrypoint.sh` completes.
- Failure: When provisioning a new tenant from a repository, any malicious lifecycle script (`preinstall`, `postinstall`) embedded in a dependency runs with unconstrained outbound internet access before the network fence is activated. Malicious packages can exfiltrate local environment information or pull untrusted remote payloads.
- Confidence: high
- Marked by authors: yes (ADR-019 addendum; EN-04.3 residual)

### F-06 — Shared Database Role Bypasses Row-Level Security Across All Schemas
- Topics: CP-09.1, EN-04.3, ES-03.1
- Evidence: The tenant application connects to PostgreSQL as `campaignbuilder_app`, which is explicitly granted `BYPASSRLS` and DML permissions on every non-system schema (`enclosure/docker/web-surface-entrypoint.sh:327-342`).
- Failure: In deployments where multiple tenants share `campaignbuilder-db`, or where tenant data resides on the same PostgreSQL instance without database-level segregation, any code running inside the tenant's Next.js application (including code written by the agent) can query and modify all rows across all schemas on that database, as PostgreSQL row-level security is explicitly disabled for that user.
- Confidence: high
- Marked by authors: yes (ADR-019 addendum; ES-03.1)

### F-07 — Unauthenticated Health Route Enumerates Full Tenant Roster
- Topics: CP-01.6, FM-07.1
- Evidence: `GET /api/health` requires no credentials by design (`control-plane/src/index.auth.test.ts:77`). However, `getHealth` dynamically checks every registered project and returns an array naming every hosted tenant (`control-plane/src/health/checks.ts:186-197`).
- Failure: Any unauthenticated caller who reaches the control-plane port or an exposed health route learns the exact tenant IDs and names of every customer hosted on the system, providing targeted reconnaissance for multi-tenant exploitation.
- Confidence: high
- Marked by authors: no

### F-08 — Pre-Migration Database Restore Points Are Excluded from Encrypted Backups
- Topics: BL-05, CP-01.7, EN-10.1
- Evidence: BL-05 claims the backup archives the system's memory and approved pages, but explicitly omits pre-migration database snapshots (`enclosure/tools/backup/backup.mjs:134`, `595`, `626`). These files in `restore-points/` are full `pg_dump` archives taken before schema modifications (`control-plane/src/state/dir.ts:60-62`, `control-plane/src/scr/migration-runner.ts:233-234`).
- Failure: If an operator restores a system from a backup after an incident that occurred during or immediately after a failed migration, the pre-migration database snapshots are absent. The operator cannot rollback the database to its pre-migration state using the recorded restore point references.
- Confidence: high
- Marked by authors: no

### F-09 — Failed Migration Rollback Irrevocably Destroys Concurrent Production Data
- Topics: CP-18.5
- Evidence: When an applied migration fails verification, the runner invokes `pg_restore --clean` to revert the database to the snapshot taken before the migration was applied (`control-plane/src/scr/migration-runner.ts:284`). The system acknowledges: "the detail says every write since the restore point was discarded" (`control-plane/src/scr/migration-runner.ts:302-304`).
- Failure: Migration execution and verification can take up to 15 minutes (`CAMPAIGNBUILDER_MIGRATION_TIMEOUT_MS`, `migration-runner.ts:42-46`). If user traffic continues to write leads, customer records, or orders to the production database during this window, a subsequent migration verification failure overwrites the database completely with the pre-migration dump, permanently destroying all user data created during that period.
- Confidence: high
- Marked by authors: yes (CP-18.5 notes commit 1d41dde rationale)

### F-10 — Secondary Audit Copy Drops Events on Control Plane Restart and Queue Backlog
- Topics: BL-05, CP-03.1, CP-03.7
- Evidence: ADR-028 §2.2 promises an audit trail that is tamper-evident and shipped off-box. However, the audit sidecar runs on the exact same VM/daemon (`enclosure/docker/compose.pod.yaml:943-945`). Furthermore, event shipping uses an in-memory queue (`control-plane/src/audit/ship.ts:148`). If the control plane restarts, or if the queue reaches its bound of 10,000 events, unsent events are discarded: "a restart, or an outage long enough to reach its bound, loses the copy of whatever was waiting and nothing re-sends it" (`control-plane/src/audit/ship.ts:13-17`).
- Failure: During an incident involving a crash, restart, or sustained sidecar unreachability, the secondary audit copy permanently loses event history. The secondary audit store becomes incomplete with no reconciliation mechanism to backfill from the primary spool.
- Confidence: high
- Marked by authors: yes (CP-03.1, CP-03.7)

### F-11 — Registry Updates Mutate Live State Before Audit Append
- Topics: CP-01.4, CP-03.6, CP-07.1
- Evidence: Despite the system-wide invariant that mutations must record to audit before executing (`control-plane/src/index.ts:229-245`), the port and triple update routes register live in the registry *before* appending to the audit log (`control-plane/src/index.ts:3649`, `3651`, `3772`, `3778`).
- Failure: If appending to the audit log fails (e.g. disk full), the route attempts to restore previous configuration via `putBackUnrecorded`. If that rollback fails, the state remains modified in the live registry while completely absent from the audit trail until the server restarts (`control-plane/src/index.ts:3581`, `3585`).
- Confidence: high
- Marked by authors: yes (ADR-028 addendum; CP-03.6)

### F-12 — Unchecked In-Flight Claim Storage Leaves Gate Operations Vulnerable to Crash Desynchronization
- Topics: CP-13.6, CP-18.5
- Evidence: The concurrency claim `acting` across gate verbs (confirm, close, decline, approve) is kept strictly in volatile process memory (`control-plane/src/scr/gate.ts:412`, `785`). For migrations, the claim is only written to disk when the HTTP verb returns (`control-plane/src/scr/gate.ts:779`).
- Failure: If the control plane crashes or restarts while `approveMigration` is executing or while a gate verb has claimed a row, the in-memory claim vanishes. After restart, the system can attempt re-execution or conflicting operations on the same request row, or in the case of migrations, inherit a state where the database was altered without a persisted lock record.
- Confidence: high
- Marked by authors: no

### F-13 — Single-Lane Concurrency Deadlocks Indefinitely on Failed Restores Due to Expiry Exemption
- Topics: BL-03, CP-13.1, CP-13.2
- Evidence: Projects have a strict limit of one in-flight request on the engineering lane. A failed row whose git bracket restore fails continues holding the lane (`control-plane/src/scr/bracket.ts:162-170`, `control-plane/src/scr/gate.ts:861-865`). Held rows are explicitly exempted from both the 24-hour and 72-hour TTL expiry sweeps (`control-plane/src/scr/transition.ts:225-226`, `239`, `262`).
- Failure: If a workspace corruption or transient I/O fault causes a rollback to fail, the request enters FAILED while retaining `active: true`. Because held rows never expire, the project's engineering lane is locked indefinitely. All subsequent requests for that tenant fail immediately with 409/busy, requiring manual operator triage to free the lane.
- Confidence: high
- Marked by authors: no

### F-14 — Caddy Static Configuration Blocks Automated Public Host Activation
- Topics: CP-08.4, EN-03.1, FM-04.4
- Evidence: Caddy operates with `admin off` (`enclosure/docker/Caddyfile:469-472`), disabling dynamic runtime configuration. When a tenant is registered with a `publicHost` or torn down, the control plane generates or unlinks a `.caddy` file on disk (`control-plane/src/project/caddy-tenants.ts:277-286`), but Caddy does not reload.
- Failure: A newly registered public tenant cannot receive traffic, and a torn-down tenant continues to be routed (returning 502 Bad Gateway to visitors), until an operator manually executes `docker compose restart ingress` on the host VM (`foreman/src/features/projects/ProjectsTab.tsx:270-275`). Automated tenant lifecycle management is broken without external manual intervention.
- Confidence: high
- Marked by authors: yes (CP-08.4; FM-04.4)

### F-15 — Fast-Lane Content Operations Are Restricted to Default Project
- Topics: BL-06, CP-12.4, CP-15.7
- Evidence: Content proposals and `/pushlive` data deployments rely exclusively on `SURFACE_SOURCE_PROJECT_ID` (`control-plane/src/index.ts:862`). The push route explicitly rejects any target other than the default project with 409 `CROSS_PROJECT_PUSH` (`control-plane/src/index.ts:1327-1333`). Non-default tenant data substrates throw named refusals (`control-plane/src/project/registry.ts:654-655`).
- Failure: Fast-lane content changes (Tier 0 copy/layout edits) and database `/pushlive` publishing cannot be performed for any secondary tenant. All non-default tenants can only be modified through the slower engineering lane via code pull requests, breaking content-editing parity across tenants.
- Confidence: high
- Marked by authors: yes (ADR-019 addendum; CP-15.7)

---

## Not judgeable from this material

1. **Host-level container isolation and Linux kernel hardening**: The materials state that the daemon shares a single host, but the actual Docker daemon security configurations (seccomp profiles, AppArmor/SELinux confinement, user namespace remapping on the daemon level) are not detailed.
2. **Behavior under concurrent agent execution and resource starvation**: The material does not show CPU/memory limits (`deploy.resources.limits`) on container definitions or process execution, making it impossible to evaluate whether an agent running memory-heavy builds/tests can cause an Out-Of-Memory kill on the supervisor or database.
3. **Internal logic and schema resilience of vendored tenant code (`web-surface/`)**: The proprietary Next.js application logic, server actions, and Prisma repository queries are not included in the text, preventing verification of whether tenant code itself respects isolation or introduces SSRF/injection vulnerabilities.
4. **Behavior of the proprietary `pi` RPC subprocess under model stream corruption**: The exact RPC protocol implementation between `pi.ts` and the `pi` binary cannot be fully verified regarding edge-case handling of malformed JSON streaming tokens, protocol framing errors, or model provider rate-limit backoff.
5. **Real-world performance of full-database `pg_dump` on live production databases**: The backup and migration systems rely on synchronous, blocking `pg_dump` executions within 15-minute bounds. Whether this locks production tables or causes transaction stalls under high-throughput production workloads cannot be judged from the distillation.

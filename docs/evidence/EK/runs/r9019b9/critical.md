### F-01 — Cross-tenant workspace compromise via shared agent UID and unscoped shell execution
- Topics: CP-17.1, BL-06, ES-03.1, CP-15.3
- Evidence: In `control-plane/src/scr/contract-scope.ts:253-254`, shell and read tools never reach the contract scope check. In `docs/reference/adr/ADR-019-single-pod-multi-project-tenancy.md:694, 705-708` and `enclosure/docker/workspace-seed.sh:60, 119`, the agent UID (1004) is shared across all tenants and owns the engineering surface directories in every tenant workspace (`/workspaces/<id>`). In `control-plane/src/scr/executor.ts:1181-1195`, post-turn scope enforcement only inspects git diffs against the restore point of the *current* project's workspace.
- Failure: An agent executing an engineering request for Tenant A executes a shell command modifying files in `/workspaces/tenant-b`. Because shell tools bypass the scope check, the OS permissions allow UID 1004 to write to Tenant B's files, and the post-turn diff check inspects only Tenant A's git status, Tenant B's workspace is compromised without triggering a scope failure or audit alert.
- Confidence: high
- Marked by authors: yes (marked as an open gap in `docs/explanation/cloud-isolation.md:30-34` and `docs/reference/adr/ADR-019-single-pod-multi-project-tenancy.md:694`)

### F-02 — Plaintext model key exposure via preview application injection
- Topics: CP-05.3, BL-05, ES-03.1
- Evidence: In `control-plane/src/engine/pi.ts:392-413` and `enclosure/docker/model-key.sh:13-14`, environment scrubbing explicitly passes through the model provider API key (`*_API_KEY`) to the spawned agent process. In `control-plane/src/scr/contract-scope.ts:79-85`, the agent has write access to the application code (`app/`, `components/`, etc.). In `enclosure/docker/compose.pod.yaml:182-184`, the tenant's `next dev` server directly mounts and serves the workspace tree on the authenticated preview.
- Failure: A compromised or prompt-injected agent reads `process.env.OPENAI_API_KEY` (or provider equivalent) and writes it into a client-rendered component (such as `app/page.tsx`). The key is immediately rendered and accessible to anyone with preview access over HTTP, completely bypassing vault custody and egress firewall controls.
- Confidence: high
- Marked by authors: no

### F-03 — Planted workspace code survives promote refusal without rollback
- Topics: CP-15.3, CP-13.6
- Evidence: In `control-plane/src/scr/promote-git.ts:181-186` and `control-plane/src/scr/gate.ts:636-637`, if the staged git tree differs from `reviewedChangeSet`, promotion is refused with `CHANGED_SINCE_REVIEW`. In `control-plane/src/scr/gate.ts:573-586, 657-664`, this refusal deliberately does not roll back the workspace to avoid discarding another tenant's potential changes.
- Failure: If an unauthorized file is written to a workspace while a request is in `REPORTED` status, the promotion aborts with a 409 error, but the unverified, planted file remains in the active workspace served by `next dev`. The preview site serves the modified code indefinitely until an operator manually cleans the directory or closes the request.
- Confidence: high
- Marked by authors: yes (noted as an architectural tension in `CP-15.3` and `docs/reference/adr/ADR-015-commit-custody-control-plane.md:245-255`)

### F-04 — Database restore drops all concurrent visitor data on migration verification failure
- Topics: CP-18.5, CP-18
- Evidence: In `control-plane/src/scr/migration-runner.ts:284-304, 358-364`, if a migration script executes in PostgreSQL but post-migration schema verification (`ws-schema-check`) fails or throws, the runner executes a full database restore using `pg_restore --clean`. In `control-plane/src/scr/migration-runner.ts:42-46`, the migration execution timeout (`CAMPAIGNBUILDER_MIGRATION_TIMEOUT_MS`) allows up to 15 minutes.
- Failure: A migration applies successfully in `psql`, but a subsequent verification check fails. The system wipes the database back to the pre-migration snapshot taken up to 15 minutes earlier. All real customer data written to the database during that window (form submissions, visitor leads, signups) is permanently discarded.
- Confidence: high
- Marked by authors: yes (acknowledged in `control-plane/src/scr/migration-runner.ts:302-304`)

### F-05 — Broken fast-lane content remains live in database when rollback write fails
- Topics: CP-12.4, CP-12.6
- Evidence: In `control-plane/src/surface/gate.ts:227, 267-269`, when post-apply browser render verification fails (`verify.ok === false`) in enforce mode, the system attempts to restore the previous page configuration. If this rollback write fails, the proposal remains marked `applied` with an error, but the broken configuration remains active in the database.
- Failure: A marketer applies an edit that causes client-side rendering exceptions. Post-apply verification catches the error and attempts to restore the previous page, but the restore write fails (e.g. database connection hiccup or disk contention). The site continues serving the broken, crashing page to visitors, and re-approving the fix is blocked with HTTP 409.
- Confidence: high
- Marked by authors: yes (documented via local failure receipt in `CP-12.4`)

### F-06 — Subsequent verified apply publishes previously failed unrolled changes to production
- Topics: CP-15.7, CP-12.4
- Evidence: In `control-plane/src/scr/substrate.ts:113-125`, `pushPageConfig` pushes the current database row of a page to production, rather than applying individual verified patches. In `control-plane/src/surface/gate.ts:483-503` and `control-plane/src/index.ts:1340`, `/pushlive` checks whether the most recent applied proposal on that page is unverified.
- Failure: Edit A is applied, fails render verification, and fails its rollback write (leaving the broken edit in the database row). Edit B is later applied to a different section of the same page and passes verification. Because the most recent proposal (Edit B) is verified, `/pushlive` succeeds and pushes the entire current page row to production, deploying the broken changes from Edit A that failed verification.
- Confidence: high
- Marked by authors: no

### F-07 — Permanent engineering lane deadlock on restore point failure
- Topics: BL-03, CP-13.2
- Evidence: In `control-plane/src/scr/bracket.ts:162-170` and `control-plane/src/scr/gate.ts:861-865`, when a request fails and its rollback restore throws, the row transitions to `FAILED` but retains its active lane claim (`active = true`). In `control-plane/src/scr/transition.ts:225-226, 239, 262`, rows holding the active claim are explicitly barred from expiring under both the 24-hour and 72-hour TTL sweeps.
- Failure: A build or test fails, and the subsequent git restore fails (due to a locked file, process contention, or permission fault). The lane is permanently locked for that project. Background and on-read expirations refuse to clear it, and all new engineering requests are rejected with HTTP 409 until an operator manually intervenes via the console.
- Confidence: high
- Marked by authors: no

### F-08 — Audit event shipper drops records during restarts or backlogs
- Topics: CP-03, CP-03.7, BL-05
- Evidence: In `control-plane/src/audit/emit.ts:151` and `control-plane/src/audit/ship.ts:8-17, 148, 205-207`, audit shipping to the sidecar uses an unpersisted Node.js in-memory queue capped at 10,000 items. Events are queued after being fsynced to the local spool. In `enclosure/docker/compose.pod.yaml:943`, the audit sidecar runs on the same Docker daemon.
- Failure: If the control plane crashes or restarts while the audit sidecar is unreachable or slow, all pending audit events in the queue are destroyed. The sidecar has no protocol to request missing historical records from the spool, permanently breaking the secondary hash chain and losing audit parity without warning.
- Confidence: high
- Marked by authors: yes (marked in `CP-03` and `BL-05`)

### F-09 — Ingress routing crashes across all tenants on public host collision
- Topics: BL-06, CP-07.1, EN-03.1
- Evidence: In `control-plane/src/project/validate.ts:481-494` and `control-plane/src/project/caddy-tenants.ts:277-286`, Caddy imports generated tenant configuration files (`*.caddy`) via a glob. If two site blocks specify the same public host, Caddy's configuration parsing aborts and the entire ingress fails to route traffic.
- Failure: A committed configuration file or manually imported tenant file contains a duplicate public hostname (or an invalid directive). When Caddy is restarted, the entire ingress fails to load its configuration, taking down routing, TLS termination, and web traffic for all tenants on the host simultaneously.
- Confidence: high
- Marked by authors: no

### F-10 — Permanent tenant deletion deadlock on failed provisioner stop
- Topics: CP-08.4, EN-06.2, FM-04.4
- Evidence: In `control-plane/src/index.ts:3421` and `control-plane/src/project/provision-intent.ts:149-161`, deleting a project (`DELETE /api/projects/:id`) requires `tenantStopConfirmed`, which verifies that the latest `down` intent in the status log completed with `ok: true` and `running: false`. In `enclosure/tools/provisioner/provisioner.mjs:355-361`, failed Docker commands are acknowledged with `ok: false` and are never retried automatically.
- Failure: An operator triggers tenant teardown, but `docker stop` fails (due to an unresponsive container or Docker daemon error). The provisioner records `ok: false`. When the operator subsequently attempts to remove the project, the control plane permanently returns HTTP 409 because the stop was not confirmed clean. The tenant cannot be removed through the API or UI.
- Confidence: high
- Marked by authors: no

### F-11 — Automated render check fails on all non-default pages and secondary tenants
- Topics: CP-14.4, EN-05.1
- Evidence: In `enclosure/tools/ws-verify/ws-render.mjs:148-179`, the render verification check requires that the rendered page contain `main[data-campaignbuilder-hydrated="true"]` before accepting HTTP 200 as a valid render, otherwise exiting with code 2. This attribute is only set by the default tenant's `LandingPage` component.
- Failure: Any engineering request affecting a sub-route (e.g. `/admin`, `/login`), a custom template, or a second tenant (such as Civic Quest) is run through `ws-render`. Because those pages do not contain the hardcoded attribute, the check exits with code 2 (harness error), failing verification and rejecting valid code changes.
- Confidence: high
- Marked by authors: yes (documented as a known constraint in `CP-14.4` and `EN-05.1`)

### F-12 — Head-of-line blocking and cascading aborts in shared chat engine queue
- Topics: CP-05.3, CP-05.8
- Evidence: In `control-plane/src/engine/turn-queue.ts:58-61, 78-82` and `control-plane/src/index.ts:739, 757, 772`, all chat and prompt interactions share a single concurrency queue for the `pi` engine. Each turn timeout defaults to 10 minutes (`CAMPAIGNBUILDER_CHAT_TURN_TIMEOUT_MS`). When an active turn times out, the queue invokes `abortWaiting()`, which immediately rejects all queued turns with HTTP 503 `AGENT_BUSY`.
- Failure: A single long-running or stalled agent prompt blocks the engine for 10 minutes. When it reaches the timeout, all other users' chat requests queued behind it are simultaneously aborted without execution, causing wide user-facing outages across the interface.
- Confidence: high
- Marked by authors: no

### F-13 — Destructive workspace debris cleanup executes prior to audit record persistence
- Topics: CP-03.6, CP-15.1
- Evidence: In `control-plane/src/scr/bracket.ts:94, 102`, opening an initial bracket executes a debris sweep (`git clean` and reset) before the `SCR_BRACKET_OPENED` audit event is emitted or validated. In `control-plane/src/index.ts:230` and `control-plane/src/audit/emit.ts:242-248`, the system claims state mutations occur strictly after audit persistence.
- Failure: An engineering run starts in a dirty workspace. The debris sweep wipes all untracked files and uncommitted modifications. If the audit spool then fails to write (due to disk full or permission errors), the operation aborts with HTTP 503, but the deleted workspace files are permanently destroyed without any audit trail of what was removed.
- Confidence: high
- Marked by authors: yes (noted as an exception in `CP-03.6`)

### F-14 — Idempotency key reuse across routes yields contradictory cached state
- Topics: CP-13.4, CP-13.5
- Evidence: In `control-plane/src/scr/transition.ts:475` and `control-plane/src/scr/gate.ts:775-779`, idempotency outcomes are stored on the request row per route. In `foreman/src/features/work/EscalationsSection.test.tsx`, an `Idempotency-Key` sent to the decline route records `FAILED`, and the same key sent to the close route records `ABORTED`.
- Failure: A client framework reuses a request idempotency key across different actions (decline and close). The request transitions to `ABORTED`. When a client subsequently replays the decline route with that key, the control plane serves the cached 200 `FAILED` payload rather than reflecting that the entity is currently `ABORTED`, masking the true lifecycle state from callers.
- Confidence: high
- Marked by authors: no

### F-15 — Client disconnect orphans in-flight engineering requests and locks workspace
- Topics: BL-02, CP-06.1, CP-14.1
- Evidence: In `control-plane/public/bubble.js:44, 237`, request status tracking is maintained exclusively in browser memory by polling `GET /api/scr/:id/status` every 15 seconds. In `control-plane/src/index.ts:1124`, every new chat message mints a completely new request ID (`scr-chat-<uuid>`).
- Failure: A marketer submits a request in chat that escalates to the engineering lane, then closes or refreshes the tab. The client-side polling terminates. When the marketer reopens the chat and submits another message, the control plane mints a new request ID and immediately rejects it with HTTP 409 because the previous request is still running and holding the project's single engineering lane.
- Confidence: high
- Marked by authors: no

---

## Not judgeable from this material

1. **Host-level container breakout via the provisioner's Docker socket:** The materials confirm that `provisioner` mounts `/var/run/docker.sock` and restricts commands through an internal service inventory (`pod-services.json`), but the Docker daemon configuration on the host VM (user namespaces, rootless execution, capability boundaries) is not present. Without inspecting host Docker daemon settings, it cannot be verified whether compromise of the provisioner container grants root control of the host.
2. **Timing attack resilience of authentication secrets:** The code uses `crypto.timingSafeEqual` for checking `X-Break-Glass` and bearer tokens, but explicitly notes that length checks occur prior to `timingSafeEqual` and that timing properties are untested (`control-plane/src/auth/verify.ts:93-95`). Real-world timing leak susceptibility across network boundaries cannot be evaluated without empirical latency distribution measurements.
3. **Database query and transaction isolation in PostgreSQL:** Tenant data access relies on `campaignbuilder_app` having `BYPASSRLS` privileges on the default tenant database (`enclosure/docker/web-surface-entrypoint.sh:342`). Whether concurrent operations between the control plane, tenant background tasks, and customer transactions can deadlock or leak data across schemas cannot be determined without reviewing PostgreSQL connection pool and transaction isolation configurations.
4. **Resilience of git operations under concurrent file modifications:** Restore points rely on `git commit-tree`, `git read-tree -u`, and `git clean` executed directly in the shared workspace directory while `next dev` is actively running. Whether `next dev` file watchers, webpack compilation locks, or open file descriptors can cause intermittent git index locks and restore crashes cannot be determined from code citations alone.
5. **Real-world throughput and rate limiting of the OIDC provider integration:** Authentication routes (`/auth/callback`, `/auth/preview`) query the OIDC provider's JWKS and userinfo endpoints. Without deployment traffic models and caching configuration for identity lookups, it cannot be determined whether an influx of preview requests can exhaust Google OIDC rate limits and lock operators out of the system.

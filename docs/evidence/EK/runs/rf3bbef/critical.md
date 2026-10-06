### F-01 — Shared agent UID across tenants permits cross-tenant filesystem modification
- Topics: BL-06, CP-15.3, CP-17.1, ES-03.1, EN-04.2
- Evidence: In BL-06 and ADR-019 (`docs/reference/adr/ADR-019-single-pod-multi-project-tenancy.md:694`, `705-708`), the agent UID (1004) is shared across all tenants. In CP-17.1 (`control-plane/src/scr/contract-scope.ts:253-254`), shell executions are explicitly exempted from the tool-call scope check. In CP-15.3 and CP-17.4 (`control-plane/src/scr/executor.ts:1181-1195`), the post-turn scope and review-record checks compare only files within the active project's own workspace (`/workspace`).
- Failure: When an engineering turn executes for Tenant A, the agent can issue shell commands (`cp`, `rm`, `echo`) targeting another tenant's workspace directory (`/workspaces/tenant-b`), which is mounted in the same container and owned by UID 1004 (`enclosure/docker/workspace-seed.sh:60`, `119`). Because the post-turn git diff check only scans Tenant A's workspace, Tenant A's turn passes all scope gates, while Tenant B's source tree is silently modified or corrupted.
- Confidence: high
- Marked by authors: yes (ADR-019 addendum records shared UID as unbuilt isolation; CP-15.3 notes cross-tenant plant risk)

### F-02 — Next.js development server automatically executes untrusted agent edits with database credentials
- Topics: ES-03.1, EN-04.3, WS-01.1
- Evidence: In ES-03.1, the host threat model notes that any file write made to `/workspace` is hot-compiled and run by `next dev` in real time. In EN-04.3 (`enclosure/docker/web-surface-entrypoint.sh:374`, `342`), `next dev` runs as `surface` (UID 1005) with `DATABASE_URL` holding the `campaignbuilder_app` role with `BYPASSRLS`. In EN-08.5 (`enclosure/docker/web-surface-entrypoint.sh:209-211`), the tenant egress firewall explicitly allowlists outbound connections to the tenant's database and Supabase hosts.
- Failure: During an engineering attempt, before human review or check verification, the agent edits an existing API route or server component to query the database and send rows to the tenant's Supabase instance or database host. Next.js immediately hot-reloads and executes the server-side code with `BYPASSRLS` privileges, dumping or altering data before the request is evaluated, failed, or rolled back.
- Confidence: high
- Marked by authors: yes (acknowledged as the core tension in ES-03.1)

### F-03 — Failed database migration with post-apply verification fault wipes live production data via `pg_restore --clean`
- Topics: CP-18.5, EN-10.1
- Evidence: In CP-18.5 (`control-plane/src/scr/migration-runner.ts:284-304`, `358-364`), if a database migration SQL executes successfully via `psql --single-transaction`, but the subsequent `ws-schema-check` verification fails or throws, the runner executes `pg_restore --clean` using the pre-migration dump taken before the apply. Commit `1d41dde` exempted exit code 3 `psql` rejections from restoration, but left post-apply verification failures on the full restore path (`migration-runner.ts:358-364`).
- Failure: An administrator approves a database migration. The migration runs and commits to PostgreSQL. While the verifier executes, production visitors write new data (user leads, form responses). The verifier fails due to a schema check regression. The runner invokes `pg_restore --clean` against the pre-migration dump, permanently wiping all live records written since the dump began.
- Confidence: high
- Marked by authors: no

### F-04 — Failed rollback on applied surface proposal deadlocks publishing and preview recovery
- Topics: CP-12.6, CP-15.7
- Evidence: In CP-12.6 (`control-plane/src/surface/gate.ts:267-271`), when an applied proposal fails render verification and the compensatory rollback write to the database also throws, the proposal remains stuck in state `applied` with `verify.ok === false`. Re-approval returns 409 (`control-plane/src/surface/gate.ts:346`). In CP-15.7 (`control-plane/src/index.ts:1340`, `control-plane/src/surface/gate.ts:497-503`), both the push route and `/pushlive` refuse *any* proposal on a page whose latest applied proposal is live-unverified.
- Failure: A surface edit fails browser verification and hits a database write error during rollback. The broken draft remains live in the preview database. Because the page is now flagged `PAGE_LIVE_UNVERIFIED`, neither this proposal nor any past or future proposal targeting that page can be pushed live. The application has no API route or UI mechanism to unstick the page without manual, direct SQL intervention on the database.
- Confidence: high
- Marked by authors: yes (observed in watched local runtime test `failed-rollback-write-ui-2026-10-05.json`)

### F-05 — Multi-tenancy is inoperable: content publishing and fast lane fail-closed for non-default tenants
- Topics: BL-06, CP-15.7, CP-07.1, WS-01.1
- Evidence: In BL-06 and CP-15.7 (`control-plane/src/index.ts:1327-1333`), the content push route unconditionally refuses any target project other than `SURFACE_SOURCE_PROJECT_ID` (`default`) with 409 `CROSS_PROJECT_PUSH`. In `control-plane/src/project/registry.ts:654-655`, non-default projects are assigned a fail-closed data substrate that rejects page writes. In `control-plane/src/surface/store.ts:741`, database tables, columns (`PageTestVariant`, `pageConfig`), and Prisma mappings are hardcoded literals for the default tenant.
- Failure: An agency adds a second tenant project. The tenant can be built via the engineering lane, but cannot use page chat, cannot use the fast content lane, and any attempt to publish content via `/pushlive` or the API is hard-rejected with 409. Serving multiple tenants requires editing host TypeScript source code.
- Confidence: high
- Marked by authors: yes (marked as second-tenant backlog in ADR-019 and BL-06)

### F-06 — Tenant peer fence permits unauthorized control-plane access for unresolvable or non-tenant pod-net containers
- Topics: CP-02.6, EN-02.2, ES-03.1
- Evidence: In CP-02.6 (`control-plane/src/auth/tenant-fence.ts:59-67`, `control-plane/src/index.ts:122-127`), `tenantPeerFence` checks the incoming connection's peer IP against registered tenant hostnames. If the IP matches a registered tenant, it restricts the request to `POST /api/scr/intake` and `GET /auth/me`. If the peer IP does *not* resolve to a registered tenant hostname, it classifies the caller as "not a tenant: the ingress, the operator, anyone else" and calls `next()`.
- Failure: Multiple services share `pod-net` (`campaignbuilder`, `civic-quest`, `surface-overlay`, `demo`). If a non-tenant container on `pod-net` (or an unmapped internal service, or a connection arriving from an IP that fails DNS reverse mapping) connects directly to `control-plane:8788`, the fence treats it as non-tenant and passes the request through. The caller can then access all administrative endpoints without ingress header stripping.
- Confidence: medium (inferred from `tenantPeerFence` flow in CP-02.6)
- Marked by authors: no

### F-07 — Audit trail "off-box" durability claim is false; in-memory shipping queue drops records on restart
- Topics: CP-03.1, CP-03.7, EN-01.2
- Evidence: Code comments and architectural documentation repeatedly describe the audit sidecar as "off-box" (`control-plane/src/audit/chain.ts:31`, `control-plane/src/audit/spool.ts:31`, `control-plane/src/audit/ship.ts:3`). In `enclosure/docker/compose.pod.yaml:943`, the sidecar runs on the exact same host and Docker daemon. In CP-03.1 (`control-plane/src/audit/ship.ts:13-14`, `148`, `205-207`), the shipping queue between the control plane and sidecar is held strictly in Node process memory.
- Failure: If the host VM restarts or the control-plane container crashes while the sidecar is temporarily unreachable, all audit events in the in-memory queue (up to 10,000 items) are permanently lost from the secondary copy. No replay or catch-up mechanism exists, causing the primary spool and secondary copy to silently diverge. Furthermore, root compromise of the host compromises both spools simultaneously.
- Confidence: high
- Marked by authors: yes (marked as Tension in CP-03.7)

### F-08 — Policy and dial modifications bump configuration version and invalidate all pending approvals and plans
- Topics: CP-10.3, CP-13.3, BL-03.1
- Evidence: In CP-10.3 (`control-plane/src/index.ts:2143`), changing any dial signal or escalation spending limit via `PUT /api/projects/:id/autonomy` validates the entire configuration and increments `configVersion`. In CP-13.3 (`control-plane/src/scr/transition.ts:370-371`, `392-393`), human approvals bind live to `configVersion`. In `control-plane/src/scr/transition.ts:429-438`, an approved plan whose `configVersion` moved is marked `SCR_BLESSED_PLAN_LAPSED` and wiped.
- Failure: An operator approves an engineering change or plan. Before the change is pushed live, an administrator updates an unrelated autonomy dial switch or adjusts the spending ceiling. This increments `configVersion`. The approved change immediately becomes `APPROVAL_STALE`, and Push Live rejects it with 409. If an approved Tier-2 plan was queued, the plan is discarded and forced back through planning.
- Confidence: high
- Marked by authors: no

### F-09 — Single-process chat turn queue creates 10-minute head-of-line blocking and aborts concurrent users
- Topics: CP-05.4, CP-05.8
- Evidence: In CP-05.4 and CP-05.8 (`control-plane/src/engine/turn-queue.ts:58-61`, `control-plane/src/index.ts:673`, `762-772`), chat requests share one `Pi` engine subprocess and queue FIFO. The per-turn timeout `CAMPAIGNBUILDER_CHAT_TURN_TIMEOUT_MS` defaults to 10 minutes. When an active turn times out, the subprocess is killed and `abortWaiting` aborts all waiting turns in the queue with 503 `AGENT_BUSY` (`control-plane/src/index.ts:772`).
- Failure: User A submits a complex or hanging prompt that consumes the 10-minute timeout. Users B and C submit prompts shortly after and wait. After 10 minutes, User A's turn fails with 504. Users B and C are immediately terminated with 503 `AGENT_BUSY` without their prompts ever reaching the model, resulting in a 10-minute denial of service for all users.
- Confidence: high
- Marked by authors: no

### F-10 — In-memory chat bubble tracking loses state on reload and spawns duplicate engineering requests
- Topics: BL-01, BL-02, CP-14.1
- Evidence: In BL-02 (`control-plane/public/bubble.js:44`), request status polling is held exclusively in page JavaScript memory. In `control-plane/src/index.ts:1124`, every escalated engineering turn mints a new random request ID (`scr-chat-<uuid>`).
- Failure: A marketer requests an engineering change in the chat bubble. Because engineering requests take multiple minutes, the marketer reloads the page or navigates away. All status polling terminates. When the marketer returns and enters another message, the bubble does not reconnect to the pending request; it submits a brand new request, which is rejected by the lane lock or the open-request cap, or creates conflicting work in the repository.
- Confidence: high
- Marked by authors: no

### F-11 — Project deregistration deadlocks permanently if provisioner is stopped or unacknowledged
- Topics: CP-08.4, EN-06.2
- Evidence: In CP-08.4 (`control-plane/src/index.ts:3421`, `control-plane/src/project/provision-intent.ts:149-161`), `DELETE /api/projects/:id` strictly refuses deregistration unless `tenantStopConfirmed` finds an acknowledged `down` intent with `running: false`. In EN-02.1 (`enclosure/docker/compose.pod.yaml:808`), the provisioner runs only under the optional `--profile provisioner`.
- Failure: An administrator starts a project container, but later stops the provisioner service (or Docker fails to execute the stop command). Any call to `DELETE /api/projects/:id` returns 409 indefinitely. The project, its ledger, and its database references can never be deregistered through the API or console.
- Confidence: high
- Marked by authors: no

### F-12 — Deploy permissions require container restarts, bypass audit logging, and default to total denial
- Topics: BL-02, CP-10.4, CP-15.6
- Evidence: In `control-plane/src/policy/deploy.ts:17-18`, documentation claims deploy grants can be changed dynamically without restarts. In reality (`control-plane/src/index.ts:2449`), grants are read from `process.env.CAMPAIGNBUILDER_DEPLOY`. In `control-plane/src/policy/deploy.ts:210`, missing grants default to `podDefault: 'none'`. There is no API route to update deploy grants, and grant updates produce no audit records.
- Failure: Modifying a user's deploy rights requires editing host environment variables and recreating the control-plane container, causing service downtime. Operators cannot audit when deploy permissions were changed or by whom. Additionally, the `all` wildcard project grant only applies to the default project (`control-plane/src/index.ts:313`), causing deploy attempts on tenant projects to fail unexpectedly.
- Confidence: high
- Marked by authors: yes (marked as Drift in CP-10.4)

### F-13 — Render verification rejects all non-default pages and tenants due to hardcoded hydration attribute
- Topics: CP-14.4, EN-05.1
- Evidence: In CP-14.4 and EN-05.1 (`enclosure/tools/ws-verify/ws-render.mjs:148-186`), `ws-render` waits for `main[data-campaignbuilder-hydrated="true"]` after HTTP 200. If this DOM selector is absent, the script exits with code 2 (harness fault), which the executor treats as a verify failure (`control-plane/src/scr/executor.ts:886-890`). This attribute is only defined on the default project's `LandingPage`.
- Failure: When an engineering request modifies an internal route (such as `/admin`), an error page, or any page in a secondary tenant (e.g. `civic-quest`), `ws-render` fails to find the hydration attribute and exits with code 2. The executor fails the run as a harness error and rolls back the change, preventing legitimate code changes on non-default pages from ever reaching `REPORTED`.
- Confidence: high
- Marked by authors: yes (marked as Debt/limitation in CP-14.4)

### F-14 — Ingress lacks dynamic reload, causing 502 errors on teardown and outage on syntax errors
- Topics: CP-08.4, EN-03.1, FM-04.4
- Evidence: In EN-03.1 (`enclosure/docker/Caddyfile:469-474`), Caddy disables the admin API and loads tenant snippets via a static startup glob. In CP-08.4 (`control-plane/src/project/caddy-tenants.ts:89-92`), writing or removing a tenant `.caddy` snippet does not trigger a reload.
- Failure: When an administrator tears down a tenant via the console, the backend removes the tenant container and deletes its snippet, but Caddy does not reload. Ingress continues forwarding requests to the stopped container port, returning 502 Bad Gateway to visitors until an operator manually runs `docker restart ingress` on the host. If a generated snippet contains an invalid hostname or syntax conflict, Caddy fails on restart, taking down all ingress routes for the entire estate.
- Confidence: high
- Marked by authors: yes (marked as manual requirement in CP-08.4 and FM-04.4)

### F-15 — First bracket debris sweep permanently destroys untracked files before audit record confirmation
- Topics: CP-03.6, CP-15.1, BL-05
- Evidence: In CP-03.6 and CP-15.1 (`control-plane/src/scr/bracket.ts:94-102`), opening the first bracket executes a "debris sweep to HEAD" (`git clean -fdx` excluding root dependencies) *prior* to recording `SCR_BRACKET_OPENED`.
- Failure: If an operator, developer, or background tool leaves untracked files in `/workspace`, opening a bracket wipes them out immediately. If the subsequent call to `appendOrThrow(SCR_BRACKET_OPENED)` fails (e.g., due to audit disk full or spool failure), the request aborts with 503, but the untracked files have already been permanently erased with no restore point and no audit record.
- Confidence: high
- Marked by authors: yes (marked as exception in CP-03.6)

---

## Not judgeable from this material

1. **Host-level iptables and Docker bridge interaction:** The worker and tenant firewalls rely on iptables UID owner matches (`-m owner --uid-owner`) within shared container network namespaces. Whether Docker's host iptables rules and bridge routing bypass or interfere with container-internal owner matching cannot be determined without host network inspection.
2. **Behavior under concurrent git operations:** The bracket store and promoter execute git commands directly against `/workspace`. Whether rapid concurrent calls between the background dev server, git status checks, and bracket restores cause index lock collisions (`.git/index.lock`) cannot be determined from the text.
3. **True isolation of tenant runtime environments during multi-tenant load:** The materials show that per-project agent UIDs are unbuilt. Whether concurrent execution of two tenant agents causes race conditions in shared directories (`/tmp`, `/usr/local`) cannot be evaluated without runtime benchmarks.
4. **Behavior during power loss / abrupt SIGKILL:** While `spool.ts` asserts `fsync`, the interaction between Node.js file streams, SQLite/JSON ledgers, and Docker volume caches under sudden kernel panic or power loss cannot be verified from the distillation.
5. **Real-world model provider cost tracking accuracy:** Pi engine cost metering relies on parsing model names from provider responses and stderr strings ("Using custom model id"). Whether newer model IDs (e.g. Claude 3.5 Sonnet, GPT-4o) trigger silent unmetered spend or budget overruns cannot be verified without live provider API tests.

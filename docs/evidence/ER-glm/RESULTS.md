# ER-glm: GLM-5.3-Flash as an exploratory third ER reviewer

> **Exploratory, dated 2026-10-06, added after ER finished.** ER's decision ("otherwise" branch:
> build the hunk-overlap shadow checker) was taken on Gemini (primary) and Luna (secondary) and
> **cannot be changed by this addition**. Nothing here was pre-registered. The procedure copies ER's
> as closely as the tools allowed, and every deviation is listed under §Deviations.

## Result

GLM agrees with Gemini and Luna. Arm B (full source) had more confirmed findings than arm A (the
corpus pack), no arm found any hidden defect, and A was not cheaper to adjudicate. ER's rule would
take the "otherwise" branch on GLM's findings as well.

| Reviewer · arm | gen confirmed | var confirmed | prefix confirmed | **Confirmed** | Unsupported | Hidden defects | Adjudication tokens / s (#E9) | Review wall-clock / tokens (sum of 3) |
|---|---|---|---|---|---|---|---|---|
| Gemini · A (corpus) | 2/3 | 1/3 | 1/3 | **4** | 5 | 0/3 | 297,348 / 206 | 399 s; 156k prompt + 43.6k thinking |
| Gemini · B (source) | 2/3 | 1/3 | 2/3 | **5** | 4 | 0/3 | 487,468 / 261 | 1,183 s; 535k prompt + 176.8k thinking |
| Luna · A (corpus) | 1/3 | 1/3 | 1/3 | **3** | 6 | 0/3 | 271,821 / 196 | 583 s; 597k input (425k cached) + 40.3k output |
| Luna · B (source) | 3/3 | 2/3 | 2/3 | **7** | 2 | 0/3 | 81,711 / 137 | 1,091 s; 1,334k input (854k cached) + 69.9k output |
| **GLM · A (corpus)** | 3/3 | 1/3 | 2/3 | **6** | 3 | 0/3 | 357,204 / 455 | 1,312 s; 141.5k input + 50.3k output |
| **GLM · B (source)** | 3/3 | 2/3 | 3/3 | **8** | 1 | 0/3 | 184,144 / 293 | 1,445 s; 455.8k input + 64.7k output |

The Gemini and Luna rows are copied from `ER/RESULTS.md`. GLM's adjudication used Claude subagents
only (see Deviations D3), so its adjudication cost column is not comparable with the Gemini rows,
whose second opinions went through gpt-6-astra.

- **ER's rule on GLM's credits:** A + 2 ≥ B? 6 vs 8, **no**. Hidden defects A ≥ B? 0 vs 0, yes.
  Adjudication cost A ≤ B? 357k vs 184k, **no**. The A cost is higher because two A items were
  merges, each charged in full to both arms, and the 25% second-opinion sample drew five A-only
  items and one merged item, but no B-only item.
- **Primary-only sensitivity (#E13):** A 6, B 8. All 6 second opinions agreed with the primary
  verdicts, so the merge changed nothing.
- **Precision:** 12 of 16 pooled items were confirmed: 14 of 18 arm credits (A 6/9, B 8/9), the highest
  confirmed share of the three reviewers. Gemini's was 9/18 and Luna's 10/18.
- **Hidden defects:** none in either arm. No GLM item reached even a "partial" match. The three
  defects are narrow resolver and label bugs, and the top-3 production-damage lens does not reach
  them (ER threat 2).

## What GLM found (final verdicts)

| Case | Item | From | Final | Gist (from the primary adjudicator) |
|---|---|---|---|---|
| gen | X-01 | A F-03 + B F-03 | confirmed | Load-time drops (size cap, non-UTF-8, pruned dirs) are silent while parse failures are surfaced |
| gen | X-02 | B F-01 | confirmed | Same-named crates in different repos collapse into one id space, silently |
| gen | X-03 | A F-02 | confirmed | Codebase name defaults to the checkout directory, breaking cross-machine byte identity (also Gemini A in ER) |
| gen | X-04 | A F-01 | confirmed | A deeply nested file aborts the whole run, and isolation cannot catch the stack overflow (second opinion agreed) |
| gen | X-05 | B F-02 | confirmed | After `--no-model`, a stale `_model.json` persists and `verify` stays green (also Luna B in ER) |
| var | X-01 | B F-02 | unsupported | Seals are self-attesting: accurate, but a documented design decision |
| var | X-02 | A F-02 | unsupported | Claim that `verify` skips unreadable docs forever: an expected non-UTF-8 doc is Missing, so verify fails |
| var | X-03 | B F-03 | confirmed | Same-named crates across repos merge (same defect as gen X-02) |
| var | X-04 | A F-01 | unsupported | `write` deletes header-bearing files: by design, in a generated tree |
| var | X-05 | A F-03 | confirmed | Checkout-directory codebase name (overstated: only overview and index differ) |
| var | X-06 | B F-01 | confirmed | With no diagrams dir and no lock, the seal CI gate exits 0 with zero coverage (fail-open) |
| prefix | X-01 | A F-02 + B F-01 | confirmed | `seals.lock` read-modify-write is unlocked and non-atomic, so concurrent or crashed signing loses entries |
| prefix | X-02 | A F-01 | unsupported | `Self::` calls dropped as PRELUDE: false at dd88e1f, because d227dd8 had fixed it but the corpus still described it (the same stale-corpus false finding Gemini A made, ER threat 6) |
| prefix | X-03 | A F-03 | confirmed | Fingerprints are not reflow-stable inside macro arguments (narrow, and registered by the authors as DB-20) |
| prefix | X-04 | B F-03 | confirmed | Stale `_model.json` after `--no-model` (same defect as gen X-05) |
| prefix | X-05 | B F-02 | confirmed | Reproduced: a 40 KB nested-paren file aborts `sealmap model` with exit 134 |

A cross-case inconsistency of the kind ER threat 5 describes: the stack-overflow abort was confirmed
here twice (gen X-04 with a second opinion, prefix X-05). ER's var adjudication overturned the same
defect as "documented as intended".

## GLM cost and time

| Review | Transport | Wall-clock | Input tokens | Output tokens (incl. thinking) |
|---|---|---|---|---|
| gen-A | Claude Code `-p --bare --tools ""` | 359 s | 43,536 | 13,094 |
| gen-B | Claude Code | 416 s | 117,376 | 19,449 |
| var-A | Claude Code | 381 s | 43,765 | 15,128 |
| var-B | direct Messages API (D2) | 643 s | 150,352 | 28,765 |
| prefix-A | Claude Code | 572 s | 54,191 | 22,096 |
| prefix-B | direct Messages API (D2) | 387 s | 188,052 | 16,484 |
| **Total** | | **2,757 s summed; 11.5 min elapsed (all six concurrent, 09:02–09:13:39Z)** | **597,272** | **115,016** |

Z.AI reports no separate thinking count: `thinking_tokens` is 0 in every usage block, while every
transcript holds one thinking block of 47k–108k characters. Output tokens therefore include thinking.
No cache was used. No price is applied, and the `costUSD` that Claude Code prints for this
unrecognised model id has an "unknown" cost basis, so it is ignored.

Adjudication: 3 primary Claude subagents took 271,305 tokens and 478 s in all (gen 88.0k / 162 s,
var 85.8k / 99 s, prefix 97.6k / 218 s). 3 second-opinion Claude subagents took 202,996 tokens and
178 s.

## Procedure

1. **Reviewer.** GLM-5.3-Flash through Claude Code pointed at Z.AI: `claude -p --bare
   --strict-mcp-config --tools "" --model glm-5.3-flash --output-format json`, run from an empty
   directory under `env -i` with a private HOME (see D1). No thinking budget was set, so GLM ran at
   its default maximum effort. The prompt is ER's Luna `prompt.txt` for that case and arm, byte for
   byte (sha256 matches, see `reviews/*/prompt.sha256`): the same pack, a blank line, then
   `lenses/critical-3.md`, sent on stdin.
2. **No file access, verified.** A probe in the same configuration reported "Tools: NONE" and
   "CANNOT" read `/etc/hostname`. Every review session's transcript has 0 `tool_use` blocks and
   `num_turns` = 1. The direct-API calls declare no tools.
3. **Parsing and pooling.** Each review returned 3 findings (18 in all). ER's `pool.cjs` was copied
   unchanged, including the tolerant extractor and the mulberry32(20261006) shuffle, cases in sorted
   order. Deduplication followed #E6 (same defect, same code location, within a case): gen A F-03 =
   B F-03 (silent load drops, `source.rs`), and prefix A F-02 = B F-01 (seal-lock write path). That
   gives 16 items.
4. **Adjudication.** One fresh, blind Claude subagent per case read a plain copy of the snapshot (gen
   28c62d8, var 6e0604c, prefix dd88e1f, from `git archive`). They used ER's primary rubric, in which
   unsupported means "wrong, already handled, not present at this revision, or not evidenced", and
   ER's hidden-defect texts. Second opinions (#E7) went to a ceil(25%) seeded sample, using ER's
   `sample.cjs` with GLM origin routed like Luna's to a second, independent Claude subagent. That
   subagent used ER's second-opinion wording, which adds "or deliberately documented as intended".
   The conservative merge, the #E8 hidden-defect rule and #E9 cost attribution follow ER's
   `score.cjs` logic (`score.cjs` here). Every snapshot ended unchanged: no file newer than the
   pooling time.

## Deviations from ER's procedure

- **D1, environment isolation.** The parent session exports `CLAUDE_EFFORT=medium` and
  `CLAUDE_CONFIG_DIR` (the operator's real config, with an Explanatory output style and plugins). A
  plain `env HOME=… claude` inherits both, which would lower GLM's effort and inject an output
  style. Every GLM call therefore ran under `env -i`, with only HOME, PATH, TERM, the Z.AI base URL
  and key, and `CLAUDE_CODE_MAX_OUTPUT_TOKENS=128000`.
- **D2, transport for var-B and prefix-B.** Claude Code refused both packs client-side with "Prompt
  is too long" (0 API ms). It assumes a 200K window for the unrecognised model id and estimates the
  tokens itself. Z.AI accepted both packs: 150,352 and 188,052 input tokens. Each was sent once,
  as-is, over the Anthropic Messages API (`direct.cjs`): the same prompt bytes, no tools, no
  thinking parameter, `max_tokens` 128000, streamed. The only difference from the Claude Code path
  is that Claude Code's minimal `--bare` system prompt (an identity line, the cwd and the date) is
  absent. This is the analogue of ER #E11.
- **D3, adjudicator family.** ER sent Gemini-origin second opinions to gpt-6-astra. GLM's went to a
  second Claude subagent, as Luna's did, at the team lead's direction. The pool contains GLM
  findings only, so adjudicators never saw ER's items.
- **D4, adjudicator copies.** Adjudicators read plain `git archive` copies rather than worktrees, so
  no git history was reachable.

## Files

`reviews/<case>-<arm>-glm/`: `critical.md`, `findings.json` (the script's `parseFindings`),
`manifest.json` or `direct-manifest.json`, `out.json` (Claude Code result), `prompt.sha256`.
`dedup.json`, `pool.cjs`, `sample.cjs`, `score.cjs`, `direct.cjs`, `run-one.sh`.
`adjudication/<case>/`: `blind.md`, `key.json`, `primary-prompt.md`, `primary.json`,
`claude2-prompt.md`, `claude2.json`, plus `second-selection.json`, `primary-cost.json` and
`claude2-cost.json`. `results.json` holds the machine-readable results.

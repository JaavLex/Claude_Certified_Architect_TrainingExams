# Practice Exam #2 — CCAR-F (60 items, 120 minutes)

> Different questions, full 120-minute timed mock. Same sits rules as Practice Exam 1. Answer key at bottom.
>
> Distribution: D1 (16) · D2 (11) · D3 (12) · D4 (12) · D5 (9).

---

## Scenario E — Developer Productivity with Claude (Q1–12)

**1. (Select 1)** You want to onboard engineers to an unfamiliar legacy codebase with Claude. Which built-in tool finds all imports of a deprecated module across the codebase?
- A. Glob `**/*`
- B. Grep for the module name
- C. Read all files
- D. Bash `ls -R`

**2. (Select 1)** An Edit operation fails repeatedly — the target code string appears in many files. The reliable fix?
- A. Retry Edit at higher precision.
- B. Use Read to load the full file, then Write the corrected content.
- C. Use Glob to find unique lines.
- D. Restart Claude Code.

**3. (Select 1)** Recommended order to build a mental model of a new codebase?
- A. Read every file once.
- B. Start with Grep for entry points, then Read to trace imports and flows — incrementally.
- C. Glob all files, then Write notes.
- D. Use Bash to grep and never use Read.

**4. (Select 1)** You trace usage of an exported function across wrapper modules. Best approach?
- A. Identify all exported names first; then search (Grep) for each name across the codebase.
- B. Read each file once.
- C. Use Glob on `*.md`.
- D. Add console logs everywhere.

**5. (Select 1)** You want Claude to integrate an MCP-based Jira server. Do you prefer a community server or a custom one?
- A. Always custom for full control.
- B. Prefer the existing community MCP server for standard integrations like Jira; reserve custom servers for team-specific workflows.
- C. Neither — disable MCP.
- D. Use built-in Grep instead.

**6. (Select 1)** The agent keeps preferring built-in `Grep` over your MCP `search_kb` even though `search_kb` is more capable. Root cause / fix?
- A. MCP servers must be project-scoped.
- B. Enhance the MCP tool description to explain capabilities and outputs in detail.
- C. Disable Grep.
- D. Use `tool_choice` to force it.

**7. (Select 1)** Expose a catalog of available documentation/issue summaries so the agent knows what data exists without exploratory calls. Use which MCP primitive?
- A. MCP Tools
- B. MCP Resources (content catalogs)
- C. CLAUDE.md imports
- D. Skills

**8. (Select 1)** Personal/experimental MCP server you don't want to share via version control. Where?
- A. `.mcp.json` at repo root.
- B. `~/.claude.json` (user-scoped).
- C. `CLAUDE.md`.
- D. `.claude/skills/`.

**9. (Select 1)** You need `${GITHUB_TOKEN}` in `.mcp.json` for credentials without committing the secret. Supported?
- A. No — hard-code tokens.
- B. Yes — `.mcp.json` supports environment variable expansion like `${GITHUB_TOKEN}`.
- C. Only on Linux.
- D. Only with a managed policy.

**10. (Select 1)** Are tools from all configured MCP servers available at once?
- A. Only one server at a time.
- B. Yes — discovered at connection time and available simultaneously.
- C. Only after `/memory`.
- D. Only project-scoped servers.

**11. (Select 2)** Two ways to make a codebase-exploration workflow not pollute the main conversation? (select exactly 2)
- A. Use the **Explore** subagent for verbose discovery output.
- B. Use a skill with **`context: fork`**.
- C. Add all exploration output to CLAUDE.md.
- D. Disable `/compact`.

**12. (Select 1)** Long exploration sessions suffer context degradation. Best recovery design?
- A. Each agent exports state to a known location (manifest); coordinator loads manifest on resume and injects into prompts.
- B. Always restart from scratch.
- C. Increase the model's max_tokens.
- D. Switch to a smaller context window.

---

## Scenario F — Structured Data Extraction (Q13–24)

**13. (Select 1)** Strict JSON schema via tool use eliminates which errors?
- A. Semantic errors (sum mismatches).
- B. Syntax errors only — not semantic.
- C. All errors.
- D. None.

**14. (Select 1)** Best field for analyzing why reviewers dismiss findings?
- A. `severity`
- B. `detected_pattern` field tracking the code construct that triggered the finding.
- C. `author`
- D. `timestamp`

**15. (Select 1)** Batch of 100 docs: 8 failed with "context length exceeded." Correct resubmission?
- A. Resubmit all 100.
- B. Resubmit only the 8 failed (by `custom_id`) with modification: chunk oversized documents.
- C. Resubmit with `tool_choice: any`.
- D. Switch to real-time for everything.

**16. (Select 1)** Before batch-processing large volumes, what maximizes first-pass success?
- A. Run prompt refinement on a sample set, then batch the volume.
- B. Submit everything at max temperature.
- C. Use a single mega-prompt.
- D. Disable validation.

**17. (Select 1)** Field-level confidence scores — how to use them?
- A. Ignore; use overall confidence.
- B. Output field-level confidence; calibrate review thresholds using labeled validation sets.
- C. Hard-code at 0.9.
- D. Only for the first field.

**18. (Select 1)** Why must you analyze accuracy by document type and field before automating high-confidence extractions?
- A. To save tokens.
- B. Aggregate metrics (e.g., 97%) can mask poor performance on specific document types or fields.
- C. To reduce model size.
- D. To speed up JSON parsing.

**19. (Select 1)** Document uses inline citations; another uses a bibliography; another embeds details in prose. Best few-shot strategy?
- A. One universal example.
- B. 2–4 targeted few-shot examples demonstrating extraction from varied document structures.
- C. 10 examples of the same style.
- D. No few-shot; strict schema is enough.

**20. (Select 1)** Required-field model fabricated a phone number to satisfy the schema when absent from source. Fix?
- A. Make the phone field optional/nullable.
- B. Add a retry.
- C. Increase temperature.
- D. Make it required + enum.

**21. (Select 1)** Format inconsistency between sources (mix of `2024-01-05`, `Jan 5, 2024`, epoch). Best practice?
- A. Add format normalization rules in the prompt alongside strict output schemas.
- B. Remove the date field.
- C. Use a regex parser post-hoc.
- D. Switch to free-text.

**22. (Select 1)** Time budget: ~50% cost savings matter; result tolerance is "overnight." Choose API:
- A. Synchronous.
- B. Message Batches API.
- C. Streaming.
- D. Single-call.

**23. (Select 1)** Routes for review when source data is ambiguous/contradictory:
- A. Auto-accept.
- B. Route to human review (limited capacity), prioritize.
- C. Mark confidence 1.0.
- D. Discard.

**24. (Select 1)** In a self-correction flow, you'd add:
- A. `calculated_total` alongside `stated_total` and `conflict_detected` booleans.
- B. A second model.
- C. A hard stop.
- D. Few-shot only.

---

## Domain knowledge items — mixed (Q25–42)

**25. (Select 1)** The agentic loop continues when:
- A. `stop_reason == "end_turn"`
- B. `stop_reason == "tool_use"`
- C. `stop_reason == "max_tokens"`
- D. The user sends a new message.

**26. (Select 1)** Correct loop mechanism when the model requests a tool:
- A. Ignore and respond with text.
- B. Execute the tool, append an assistant message with `tool_use` blocks and a user message with `tool_result` blocks, then re-send.
- C. Append only the tool's name to history.
- D. Restart with a new system prompt.

**27. (Select 1)** PreToolUse hook returns `permissionDecision: "deny"` for a Bash `rm -rf` command. This is an example of:
- A. Prompt-based guidance.
- B. Programmatic, deterministic enforcement of policy.
- C. A suggestion system.
- D. A tool description.

**28. (Select 1)** PostToolUse hook updates a tool's `updatedToolOutput` to a canonical ISO timestamp. This is:
- A. Data normalization before the model processes results.
- B. Few-shot.
- C. Plan mode.
- D. /compact.

**29. (Select 1)** Two main hook patterns and their jobs:
- A. PostToolUse transforms results; PreToolUse/tool-call intercept blocks/redirects calls.
- B. Both block calls.
- C. Both transform outputs.
- D. Neither is programmatic.

**30. (Select 1)** Giving a synthesis agent all web-search tools to avoid round-trips is:
- A. Correct — least privilege.
- B. Wrong — violates separation of concerns; over-provisions.
- C. Required.
- D. Faster.

**31. (Select 1)** Recommended tool count per agent (sweet spot):
- A. 1.
- B. 4–5, scoped per role.
- C. 18+.
- D. Unlimited.

**32. (Select 1)** Replace broad `fetch_url` for a doc-analysis agent with:
- A. `fetch_url` AND all other tools.
- B. A constrained alternative like `load_document` that validates document URLs.
- C. `Grep`.
- D. No tools.

**33. (Select 1)** Conventions that span multiple directories (e.g., all `*.tsx` files anywhere) — best mechanism:
- A. Subdirectory CLAUDE.md per folder.
- B. `.claude/rules/` with `paths: ["**/*.tsx"]` glob frontmatter (path-specific rules).
- C. One giant CLAUDE.md.
- D. A skill.

**34. (Select 1)** Team-shared `/security-audit` command available to all who clone the repo. Location?
- A. `~/.claude/commands/`.
- B. `.claude/commands/` in the repository.
- C. `CLAUDE.md`.
- D. `.claude/config.json`.

**35. (Select 1)** Modularity of many CLAUDE.md files. Best practice?
- A. Use `@import` to selectively include relevant standards files.
- B. Concatenate all into one file.
- C. Convert to JSON.
- D. Remove CLAUDE.md.

**36. (Select 1)** For "add comprehensive tests to a legacy codebase," best decomposition strategy?
- A. Rigid prompt chaining.
- B. Dynamic adaptive decomposition: map structure → identify high-impact areas → prioritized plan that adapts as dependencies are discovered.
- C. Single mega-prompt.
- D. No plan.

**37. (Select 1)** Splitting large code reviews into per-file local passes + cross-file integration passes is an example of:
- A. Multi-pass review / prompt chaining.
- B. Routing.
- C. /compact.
- D. Hooks.

**38. (Select 1)** TDD-style iteration with Claude:
- A. Write test suites first (expected + edge + perf), then iterate by sharing test failures.
- B. Generate tests after.
- C. Skip tests.
- D. Use only prose.

**39. (Select 1)** Interview pattern usage:
- A. Have Claude ask questions to surface considerations before implementing in unfamiliar domains.
- B. Interview the model for trivia.
- C. Wait for the user to interview.
- D. Use only after bugs.

**40. (Select 1)** Interacting vs independent problems when fixing:
- A. Always sequential.
- B. Always one message.
- C. Interacting problems → one detailed message; independent → sequential iteration.
- D. Never fix.

**41. (Select 1)** `/memory` command does what?
- A. Resets memory.
- B. Shows which memory files are loaded; diagnose inconsistent behavior.
- C. Compacts context.
- D. Backs up to cloud.

**42. (Select 1)** `/compact` does what?
- A. Deletes history.
- B. Reduces context usage by summarizing conversation so far.
- C. Installs new MCP.
- D. Resumes a session.

---

## Scenario mix items (Q43–60)

**43. (Select 1)** Coordinator/subagent multi-agent topology where the coordinator manages ALL inter-subagent communication is called:
- A. Peer-to-peer.
- B. Hub-and-spoke.
- C. Pipeline-only.
- D. Federated.

**44. (Select 1)** Why route all subagent comms through the coordinator?
- A. To reduce model calls.
- B. For observability, consistent error handling, controlled information flow.
- C. Because subagents can't call tools.
- D. To reduce tokens.

**45. (Select 1)** Anti-pattern in multi-agent error handling:
- A. Returning structured error context.
- B. Returning empty results as success when the tool timed out.
- C. Local recovery for transient errors.
- D. Propagating only unresolvable errors.

**46. (Select 1)** If `lookup_order` legitimately returns zero matching orders:
- A. Report this as a transient error to retry.
- B. Represent as a **successful** query with no matches — NOT an error.
- C. Treat as business error.
- D. Treat as permission error.

**47. (Select 1)** Best escalation response when policy is ambiguous/silent (e.g., competitor price match when policy only addresses own-site adjustments):
- A. Decide unilaterally.
- B. Escalate because policy is ambiguous/silent on the specific request.
- C. Apply the closest matching rule.
- D. Always refuse.

**48. (Select 1)** Why might aggregate 97% accuracy be misleading before automating high-confidence extractions?
- A. It's always reliable.
- B. It may mask poor performance on specific document types or fields.
- C. It's a pass/fail marker.
- D. It guarantees no FP.

**49. (Select 1)** Forge-independence principle for code review of AI-generated code: use a:
- A. Second **independent** Claude instance (no prior reasoning context).
- B. Verification pass in the same session.
- C. Higher temperature.
- D. Self-review with extended thinking.

**50. (Select 1)** For plan mode, a good trigger is:
- A. A single-file bug fix with a clear stack trace.
- B. A monolith → microservices restructuring affecting 45+ files.
- C. Adding one validation check.
- D. A typo fix.

**51. (Select 1)** For direct execution, a good trigger is:
- A. Library migration affecting 45+ files.
- B. Architectural restructuring.
- C. Adding a single date validation conditional in one function.
- D. Choosing between two integration approaches with different infra.

**52. (Select 1)** Reviewing generated code in CI in the **same** Claude session that generated it:
- A. More thorough — remembers reasoning.
- B. Less effective than an independent review instance (self-review blind spot).
- C. Required by CI.
- D. Free of cost.

**53. (Select 1)** A skill's frontmatter to prompt for required parameters when called without args:
- A. `argument-hint`
- B. `context: fork`
- C. `allowed-tools`
- D. `disable-model-invocation`

**54. (Select 1)** Personal skill that shouldn't affect teammates lives where (with a different name)?
- A. `.claude/skills/` (shared).
- B. `~/.claude/skills/` with a different name.
- C. `.mcp.json`.
- D. `CLAUDE.md`.

**55. (Select 1)** Choose skills vs CLAUDE.md:
- A. Both always-loaded.
- B. Skills = on-demand, task-specific; CLAUDE.md = always-loaded universal standards.
- C. CLAUDE.md is on-demand.
- D. Skills are universal.

**56. (Select 1)** Coordinator prompts should specify _______, not _______:
- A. Step-by-step procedures; goals.
- B. Research goals + quality criteria; step-by-step procedural instructions.
- C. All possible questions; criteria.
- D. URLs; tools.

**57. (Select 1)** Three programmatic enforcement mechanisms for compliance (select best set) — pick the option that lists ONLY programmatic mechanisms:
- A. Hooks, prerequisite gates, `tool_choice`.
- B. Hooks, system-prompt instructions, few-shot.
- C. System-prompt, routing classifier, schema.
- D. Few-shot, examples, retries.

**58. (Select 1)** Two factors that make retry-with-error-feedback succeed (best pair):
- A. Information exists in source + format/structural error.
- B. Model temperature + token count.
- C. Number of tools + batch size.
- D. Confidence score + sentiment.

**59. (Select 1)** To recover from a long-session crash, restore from:
- A. A random backup.
- B. Structured agent state exports (manifests) that the coordinator loads on resume and injects.
- C. /memory.
- D. None — must restart fresh.

**60. (Select 1)** Out of scope (Exam Guide) — which topic will NOT appear?
- A. CLAUDE.md configuration.
- B. Fine-tuning Claude models / Constitutional AI.
- C. `tool_choice` options.
- D. Message Batches API.

---

# ✅ ANSWER KEY — Practice Exam #2

| Q | Ans | Domain | Why |
|---|-----|--------|-----|
| 1 | **B** | D2 | Grep finds content; module imports are content patterns. |
| 2 | **B** | D2 | Read + Write fallback for non-unique Edit text. |
| 3 | **B** | D2 | Incremental: Grep entry points → Read imports. |
| 4 | **A** | D2 | Identify exported names → Grep each across codebase. |
| 5 | **B** | D2 | Prefer community MCP for standard integrations; custom only for team-specific. |
| 6 | **B** | D2 | Enhance MCP tool descriptions so agent prefers them over built-ins. |
| 7 | **B** | D2 | MCP Resources expose content catalogs. |
| 8 | **B** | D2 | `~/.claude.json` user-scoped for personal/experimental. |
| 9 | **B** | D2 | `${VAR}` env expansion supported in `.mcp.json`. |
| 10 | **B** | D2 | Tools from all servers available simultaneously at connection. |
| 11 | **A, B** | D3 | Explore subagent + `context: fork` skill both isolate verbose output. |
| 12 | **A** | D5 | Structured state manifests for crash recovery. |
| 13 | **B** | D4 | Strict schema eliminates syntax errors only; not semantic. |
| 14 | **B** | D4 | `detected_pattern` field for FP-pattern analysis. |
| 15 | **B** | D4 | Resubmit only failed by `custom_id`; chunk oversized. |
| 16 | **A** | D4 | Prompt refinement on sample set first maximizes first-pass success. |
| 17 | **B** | D5 | Field-level confidence; calibrate thresholds with labeled validation sets. |
| 18 | **B** | D5 | Aggregate masks weak segments. |
| 19 | **B** | D4 | Targeted few-shot for varied document structures. |
| 20 | **A** | D4 | Optional/nullable prevents fabrication. |
| 21 | **A** | D4 | Format normalization rules alongside strict schemas. |
| 22 | **B** | D4 | Batches for overnight; ~50% savings. |
| 23 | **B** | D5 | Route ambiguous/contradictory to human review. |
| 24 | **A** | D4 | Computed cross-checks + `conflict_detected` for self-correction. |
| 25 | **B** | D1 | `tool_use` continues the loop. |
| 26 | **B** | D1 | Execute → append assistant(tool_use) + user(tool_result) → resend. |
| 27 | **B** | D1 | Programmatic/deterministic enforcement. |
| 28 | **A** | D1 | PostToolUse data normalization. |
| 29 | **A** | D1 | PostToolUse transforms; PreToolUse/intercept blocks. |
| 30 | **B** | D2 | Over-provision violates separation of concerns. |
| 31 | **B** | D2 | 4–5 scoped per role; 18+ degrades selection. |
| 32 | **B** | D2 | Constrained alternative (validates URLs). |
| 33 | **B** | D3 | `.claude/rules/` w/ globs span directories; subdirectory CLAUDE.md is directory-bound. |
| 34 | **B** | D3 | `.claude/commands/` project-shared. |
| 35 | **A** | D3 | `@import` for modular CLAUDE.md. |
| 36 | **B** | D1 | Dynamic adaptive decomposition for open-ended tasks. |
| 37 | **A** | D4/1 | Multi-pass review = prompt chaining variant. |
| 38 | **A** | D3 | TDD: tests first, then iterate on failures. |
| 39 | **A** | D3 | Interview pattern (Claude asks questions). |
| 40 | **C** | D3 | Interacting → one message; independent → sequential. |
| 41 | **B** | D3 | `/memory` shows loaded files. |
| 42 | **B** | D3 | `/compact` summarizes to free context. |
| 43 | **B** | D1 | Hub-and-spoke. |
| 44 | **B** | D1 | Observability, consistent errors, controlled flow. |
| 45 | **B** | D5 | Suppress-as-success is an anti-pattern. |
| 46 | **B** | D2/5 | Zero matches = successful empty result; not an error. |
| 47 | **B** | D5 | Escalate on policy ambiguity/silence. |
| 48 | **B** | D5 | Aggregate masks per-segment weakness. |
| 49 | **A** | D4 | Independent instance (no reasoning context). |
| 50 | **B** | D3 | Architectural/large → plan mode. |
| 51 | **C** | D3 | Simple, well-scoped → direct execution. |
| 52 | **B** | D4 | Self-review < independent instance. |
| 53 | **A** | D3 | `argument-hint` for missing-arg prompts. |
| 54 | **B** | D3 | `~/.claude/skills/` with a different name. |
| 55 | **B** | D3 | Skills on-demand; CLAUDE.md always-loaded. |
| 56 | **B** | D1 | Goals + quality criteria, not procedures. |
| 57 | **A** | D1/2/4 | Hooks, prerequisite gates, `tool_choice` are the programmatic set (no prompt/few-shot). |
| 58 | **A** | D4 | Info exists + format/structural error → retry works; absence → ineffective. |
| 59 | **B** | D5 | State manifests (structured agent exports). |
| 60 | **B** | Misc | Fine-tuning/Constitutional AI explicitly out-of-scope. |

---

## Scoring & Targets
- **≥90%** first attempt → exam-ready.
- **85–89%** → review cluster misses.
- **70–84%** → re-read missed domains + redo missed Qs.
- **<70%** → re-read all 5 domain guides in priority order; do labs.

### Top 5 patterns if you missed many
1. Calling something "programmatic" when an option is prompt-based → **reread D1.4–1.5, D2.2.**
2. Choosing "first step" over-engineering (routing/classifier) → **reread D2.1.**
3. Misjudging escalation triggers → **reread D5.2.**
4. Forgetting Batches have no multi-turn tools / no SLA → **reread D4.5.**
5. Confusing Grep vs Glob, Edit vs Read+Write → **reread D2.5.**

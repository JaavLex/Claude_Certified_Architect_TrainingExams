# Practice Exam #1 — CCAR-F (60 items, 120 minutes)

> **Instructions:** Set a 120-minute timer. No notes, no docs. Each item states how many to select (1 = single; 2/3 = multiple-response — select EXACTLY that many). When done, score with the answer key at the bottom; for every miss, note the **domain + reason**, then drill the relevant domain guide.
>
> **Target:** ≥85% on first attempt = on track. Misses cluster in one domain? Re-read that domain guide.
>
> Distribution: D1 (16) · D2 (11) · D3 (12) · D4 (12) · D5 (9).

---

## Scenario A — Customer Support Resolution Agent (Q1–15)

**1. (Select 1)** Production data shows that in 8% of cases your agent skips `get_customer` and calls `lookup_order` using only the customer's stated name, occasionally leading to misidentified accounts and incorrect refunds. What change most **effectively** addresses this?
- A. Add a programmatic prerequisite blocking `lookup_order` and `process_refund` until `get_customer` has returned a verified customer ID.
- B. Enhance the system prompt to state customer verification is mandatory before any order operations.
- C. Add few-shot examples showing the agent always calling `get_customer` first.
- D. Implement a routing classifier that enables only the relevant tool subset per request type.

**2. (Select 1)** Your agent calls `get_customer` for order queries ("check my order #12345") instead of `lookup_order`. Both tools have minimal descriptions ("Retrieves customer information" / "Retrieves order details"). What's the most **effective first step**?
- A. Add 5–8 few-shot examples to the system prompt for order queries.
- B. Expand each tool's description with input formats, example queries, edge cases, and boundaries explaining when to use it vs alternatives.
- C. Implement a routing layer that parses input and pre-selects the tool.
- D. Consolidate into a single `lookup_entity` tool accepting any identifier.

**3. (Select 1)** FCR is 55% (target 80%). Logs show it escalates straightforward cases while autonomously handling complex policy-exception cases. Most effective way to improve escalation **calibration**?
- A. Add explicit escalation criteria with few-shot examples demonstrating when to escalate vs resolve.
- B. Have the agent self-report a confidence score (1–10); route below threshold to humans.
- C. Deploy a classifier trained on historical tickets to predict escalations.
- D. Sentiment analysis → auto-escalate when negative sentiment exceeds a threshold.

**4. (Select 1)** A customer immediately replies "I want to speak to a human" before any investigation. The issue is a routine return within policy. What should the agent do?
- A. Investigate first since the issue is clearly within capability; escalate only if resolution fails.
- B. Acknowledge the request, but proceed to resolve because FCR is the priority.
- C. Escalate immediately to a human agent.
- D. Ask the customer to confirm three times before honoring the request.

**5. (Select 1)** `lookup_order` returns five customer matches for "John Smith." What's correct behavior?
- A. Select the most recent match by last-order date.
- B. Choose the account with the highest lifetime value.
- C. Ask the customer for additional identifiers to disambiguate.
- D. Process a partial refund on the first match and ask for clarification.

**6. (Select 1)** Business policy: refunds above $500 must always go to a human. Production shows the agent occasionally processes >$500 refunds. Best fix?
- A. Add stronger wording in the system prompt about the $500 limit.
- B. Implement a tool-call interception hook that blocks `process_refund` above $500 and redirects to `escalate_to_human`.
- C. Increase the model's reasoning across retry attempts.
- D. Add few-shot examples with $500 thresholds.

**7. (Select 1)** Your agent must aggregate `get_customer`/`lookup_order` outputs from different MCP tools with inconsistent formats (Unix timestamps, ISO 8601, numeric codes). Where should normalization happen?
- A. In the system prompt instructing the agent to interpret formats.
- B. In a PostToolUse hook that normalizes tool results before the agent processes them.
- C. By rewriting each backend to return a single canonical format.
- D. By asking the model to detect format and convert during reasoning.

**8. (Select 1)** During escalation, the human agent lacks access to the conversation transcript. What must the handoff include?
- A. A link to the full transcript.
- B. Customer ID, root cause analysis, refund amount (if any), and recommended action.
- C. Only the customer's original message.
- D. The agent's full internal reasoning trace.

**9. (Select 1)** A message: "I want to return my last order, dispute a billing charge, and update my email." Best agent behavior?
- A. Handle one at a time in the order received, ignoring the others until the first resolves.
- B. Decompose into distinct items; investigate each (often in parallel, shared context); synthesize a unified resolution.
- C. Route the entire message to `escalate_to_human` since it's multi-concern.
- D. Process only the return; ignore the rest as out of scope.

**10. (Select 1)** What `stop_reason` value causes the agentic loop to **continue**?
- A. `end_turn`
- B. `tool_use`
- C. `max_tokens`
- D. `stop_sequence`

**11. (Select 1)** Which is an anti-pattern for controlling the agentic loop?
- A. Inspecting `stop_reason` from the API response.
- B. Appending `tool_result` blocks to conversation history between iterations.
- C. Parsing the assistant's natural-language text for the word "done" to decide termination.
- D. Executing requested tools when `stop_reason == "tool_use"`.

**12. (Select 2)** Which two are appropriate escalation triggers? (select exactly 2)
- A. The customer explicitly requests a human.
- B. The customer uses several exclamation marks.
- C. The agent cannot make meaningful progress after reasoning + tools.
- D. The case is routine and fully within policy.

**13. (Select 1)** Why is self-reported confidence score an unreliable escalation signal?
- A. It requires extra tokens.
- B. LLM self-confidence is poorly calibrated, especially on hard cases.
- C. It can't be implemented in the Agent SDK.
- D. Customers can see the score and get confused.

**14. (Select 1)** Where must tool results be placed for the model to incorporate new information between iterations?
- A. In a separate external log only.
- B. Appended to conversation history as `tool_result` blocks (matched to `tool_use_id`).
- C. Compressed into a summary string.
- D. Replaced into the system prompt.

**15. (Select 1)** Long sessions start giving inconsistent answers and citing "typical patterns." What's the best remedy?
- A. Use a larger context-window model.
- B. Maintain scratchpad files of key findings and reference them for subsequent questions.
- C. Re-summarize the entire history more aggressively.
- D. Restart the loop on each user message.

---

## Scenario B — Multi-Agent Research System (Q16–30)

**16. (Select 1)** For "impact of AI on creative industries," only visual arts coverage; coordinator decomposed into digital art, graphic design, photography. Root cause?
- A. Synthesis agent lacks gap-detection instructions.
- B. Coordinator's task decomposition is too narrow, omitting music/writing/film.
- C. Web search queries aren't comprehensive enough.
- D. Document analysis filtered out non-visual sources.

**17. (Select 1)** Web search subagent times out. Which error propagation best enables intelligent recovery?
- A. Structured error context: failure type + attempted query + partial results + alternative approaches.
- B. Auto-retry with exponential backoff; return generic "search unavailable" after exhaustion.
- C. Catch timeout, return empty result set marked as successful.
- D. Propagate exception to a top-level handler that terminates the workflow.

**18. (Select 1)** Synthesis needs fact-checks in 85% simple cases (dates/names) and 15% deep investigations; routing all via coordinator adds 40% latency. Best approach?
- A. Give synthesis a scoped `verify_fact` tool for simple lookups; complex cases delegate via coordinator.
- B. Give synthesis all web search tools.
- C. Batch all verifications to end-of-pass.
- D. Have web search proactively cache context anticipating needs.

**19. (Select 1)** For the coordinator to spawn subagents, what must its config include?
- A. A "coordinator" role in the system prompt.
- B. `allowedTools` containing `"Task"`.
- C. An MCP resource catalog.
- D. fork_session enabled.

**20. (Select 1)** Do subagents inherit the coordinator's conversation history?
- A. Yes, automatically.
- B. No — context is isolated; everything must be passed explicitly in the prompt.
- C. Only if `share_context: true`.
- D. Only for the first three turns.

**21. (Select 1)** To run subagents in parallel, the coordinator should:
- A. Emit one Task call per turn across multiple turns.
- B. Emit multiple Task tool calls in a single coordinator response.
- C. Use a higher concurrency limit setting.
- D. Configure `parallel: true` per subagent.

**22. (Select 1)** How should the coordinator's prompt be structured for best subagent adaptability?
- A. Step-by-step procedural instructions.
- B. Research goals + quality criteria.
- C. A rigid pipeline definition.
- D. Enumeration of every possible question.

**23. (Select 1)** Conflicting credible statistics appear in two sources. Synthesis should:
- A. Pick the higher-value figure.
- B. Pick the more recent publication regardless of source quality.
- C. Include both values with explicit source attribution; mark as contested.
- D. Average them.

**24. (Select 1)** Subagent outputs must include which for temporal correctness during synthesis?
- A. A model name.
- B. Publication/collection dates.
- C. A page count.
- D. Author email addresses.

**25. (Select 1)** Synthesis should render content types how?
- A. Convert everything to a uniform bullet list.
- B. Render appropriately: financial → tables, news → prose, technical → structured lists.
- C. Render everything as prose paragraphs.
- D. Render everything as JSON.

**26. (Select 1)** What should the synthesis pass include when some sources were unavailable?
- A. Coverage annotations indicating well-supported vs gap areas.
- B. A note saying "no gaps."
- C. Removal of the affected section silently.
- D. Estimation from training knowledge.

**27. (Select 1)** Refinement loop in multi-agent research:
- A. Coordinator evaluates synthesis for gaps; re-delegates targeted queries; re-invokes synthesis until coverage sufficient.
- B. Always run three independent synthesis passes and pick the best.
- C. Restart the entire pipeline from scratch on any gap.
- D. Skip gap evaluation to save latency.

**28. (Select 1)** Why structure subagent outputs (separating content from metadata like URLs/page numbers)?
- A. To reduce token count only.
- B. To preserve attribution/source provenance when passing between agents.
- C. To make JSON parsing faster.
- D. To enforce a length limit.

**29. (Select 1)** When downstream subagents have limited context budgets, what should upstream agents do?
- A. Send full reasoning chains.
- B. Return structured data (key facts, citations, relevance scores) instead of verbose content.
- C. Append large appendices.
- D. Send only the most recent message.

**30. (Select 1)** Best use of `fork_session` in research?
- A. Resume a named prior session.
- B. Create parallel branches from a shared analysis baseline to explore divergent approaches.
- C. Compress context.
- D. Disable hooks.

---

## Scenario C — Claude Code for Continuous Integration (Q31–42)

**31. (Select 1)** CI job: `claude "Analyze this PR for security issues"` hangs waiting for input. Correct approach?
- A. `claude -p "Analyze this PR for security issues"`
- B. Set `CLAUDE_HEADLESS=true`.
- C. `claude ... < /dev/null`
- D. `claude --batch "..."`

**32. (Select 1)** Manager proposes switching BOTH a blocking pre-merge check and an overnight debt report to Message Batches (50% off). How to evaluate?
- A. Batch for the debt report only; real-time for the blocking pre-merge.
- B. Switch both to batch with status polling.
- C. Keep both real-time to avoid ordering issues.
- D. Switch both to batch with a real-time timeout fallback.

**33. (Select 1)** 14-file PR produces inconsistent depth + contradictory feedback (flag a pattern in one file, approve identical code elsewhere). Restructure how?
- A. Per-file local analysis pass + separate cross-file integration pass.
- B. Require devs to split PRs into 3–4 files.
- C. Use a higher-tier model with a bigger context window.
- D. Run 3 independent full-PR passes; flag issues in ≥2.

**34. (Select 1)** Best mechanism for machine-parseable findings to post as inline PR comments?
- A. `--output-format json` + `--json-schema`
- B. Pipe stdout through grep
- C. `-p --verbose`
- D. `--markdown`

**35. (Select 1)** Avoid duplicate review comments when re-running after new commits:
- A. Reset all comments every run.
- B. Include prior findings in context; instruct Claude to report only new/unaddressed issues.
- C. Disable comments on rerun.
- D. Use a different model each time.

**36. (Select 1)** Generated tests duplicate existing test scenarios. Best fix?
- A. Stop generating tests.
- B. Provide existing test files in context + document testing standards/fixtures in CLAUDE.md.
- C. Generate tests only for new files.
- D. Increase temperature.

**37. (Select 1)** Review of generated code — best practice?
- A. Same Claude session that generated the code reviews it.
- B. Use an independent Claude instance (no prior reasoning context).
- C. Ask the generator to "be extra careful."
- D. Add extended thinking to the same session.

**38. (Select 1)** Review FP rate is high in the "documentation accuracy" category, damaging trust. Proportionate first response?
- A. Define explicit categorical criteria ("flag only when claimed behavior contradicts actual code") and improve prompts for that category.
- B. Replace the model.
- C. Add a confidence filter.
- D. Disable all categories.

**39. (Select 1)** "Be conservative; only report high-confidence findings" — effective for precision?
- A. Yes, always.
- B. No — specific categorical criteria outperform vague confidence-based filtering.
- C. Only with 10+ examples.
- D. Only for non-security findings.

**40. (Select 2)** Which CLAUDE.md CLI/context measures improve CI test-generation quality? (select exactly 2)
- A. Document testing standards, valuable criteria, and fixtures in CLAUDE.md.
- B. Provide existing test files in context.
- C. Increase model temperature.
- D. Disable all memory files.

**41. (Select 1)** Message Batches API supports multi-turn tool calling within a single request?
- A. Yes, fully.
- B. No — it does not support multi-turn tool calling mid-request.
- C. Only with MCP tools.
- D. Only if under 100 requests.

**42. (Select 1)** To guarantee overall SLA of 30 hours using Batches (≤24h processing), you should:
- A. Submit batches at 4-hour intervals so a 24-hour batch still beats the 30-hour SLA.
- B. Submit one batch and hope it's fast.
- C. Never use batches.
- D. Use real-time API for all.

---

## Scenario D — Structured Data Extraction (Q43–52)

**43. (Select 1)** Most reliable way to guarantee schema-compliant structured output?
- A. Free-text JSON with "output valid JSON only."
- B. `tool_use` with a JSON schema as input parameters.
- C. Asking the model to self-validate.
- D. Regex parsing of model text.

**44. (Select 1)** `tool_choice` setting when document type is unknown and several extraction schemas exist?
- A. `{"type":"auto"}`
- B. `{"type":"any"}`
- C. `{"type":"none"}`
- D. `{"type":"text"}`

**45. (Select 1)** To force `extract_metadata` to run before enrichment tools:
- A. `tool_choice: {"type":"tool","name":"extract_metadata"}`, then process in follow-up turns.
- B. Put `extract_metadata` first in the system prompt.
- C. Restrict `allowedTools` to `extract_metadata` only permanently.
- D. Add few-shot.

**46. (Select 1)** A field may legitimately be absent from some source documents. Best schema design?
- A. Make all fields required and instruct the model to guess.
- B. Make such fields optional/nullable so the model returns null rather than fabricating.
- C. Remove the field entirely.
- D. Always default to "N/A" string.

**47. (Select 1)** Best pattern for an extensible categorical field?
- A. Fixed enum with no fallback.
- B. Enum with "other" + a detail string (and "unclear" for ambiguous cases).
- C. Free-text field.
- D. A second model pass.

**48. (Select 1)** Validation failed: line items don't sum to the stated total. Retry strategy?
- A. Append the original document, the failed extraction, and the specific validation errors to the prompt and retry.
- B. Retry with no changes until it passes.
- C. Reduce the schema's required fields.
- D. Switch to free-text.

**49. (Select 1)** When is retry-with-error-feedback **ineffective**?
- A. When the formatting mismatch is the issue.
- B. When the required information is simply absent from the source document.
- C. When the schema has optional fields.
- D. When the model is large.

**50. (Select 1)** To catch semantic errors (sum mismatch, wrong field), best design?
- A. Trust strict schema alone.
- B. Add computed cross-checks like `calculated_total` vs `stated_total`, `conflict_detected` booleans.
- C. Increase model temperature.
- D. Add more required fields.

**51. (Select 1)** Review a 100-doc batch result: aggregate accuracy is 97%, but you suspect weak segments. Best verification?
- A. Trust the aggregate.
- B. Stratified random sampling; analyze accuracy by document type and field.
- C. Re-extract all 100.
- D. Lower the confidence threshold.

**52. (Select 1)** Routing rule for human review (limited capacity):
- A. Review all extractions sequentially.
- B. Route low-confidence OR ambiguous/contradictory-source extractions to human review, prioritizing capacity.
- C. Skip review entirely if aggregate > 95%.
- D. Review only the first document.

---

## Domain knowledge items — all scenarios (Q53–60)

**53. (Select 1)** CLAUDE.md hierarchy from highest to lowest precedence:
- A. Directory → project → user.
- B. Enterprise managed → user (`~/.claude/`) → project (`./` or `.claude/`) → directory.
- C. Project → user → managed.
- D. User → enterprise → project.

**54. (Select 1)** A new team member doesn't receive coding conventions you wrote. Diagnosis?
- A. The conventions are in user-level config, not shared via version control — move to project-level.
- B. CLAUDE.md is project-only and breaks.
- C. They need a managed policy.
- D. Reset their `~/.claude`.

**55. (Select 1)** Best mechanism to enforce that test-file conventions apply to `*.test.tsx` files scattered across the repo?
- A. Root CLAUDE.md with a "Tests" section.
- B. `.claude/rules/` file with `paths: ["**/*.test.tsx"]` glob frontmatter.
- C. A subdirectory CLAUDE.md per directory.
- D. A skill that loads on demand.

**56. (Select 1)** You want a verbose codebase-analysis skill not to pollute the main conversation. Frontmatter?
- A. `allowed-tools: read-only`
- B. `context: fork`
- C. `argument-hint: codebase`
- D. `disable-model-invocation: true`

**57. (Select 1)** Find all callers of `processRefund` across the codebase. Tool?
- A. Glob
- B. Grep
- C. Bash `ls`
- D. Write

**58. (Select 1)** Find all test files anywhere in the repo. Tool?
- A. Grep
- B. Glob with `**/*.test.*`
- C. Read
- D. Edit

**59. (Select 1)** You try to `Edit` a common string that appears 12 times. Correct fallback?
- A. Use Bash `sed`.
- B. Read the full file, then Write the modified content.
- C. Increase retries.
- D. Use Glob.

**60. (Select 1)** Which is OUT OF SCOPE for the exam (per the Exam Guide)?
- A. `tool_choice` semantics.
- B. Deploying/hosting MCP servers (container orchestration).
- C. Few-shot examples.
- D. The CLAUDE.md hierarchy.

---

# ✅ ANSWER KEY — Practice Exam #1

| Q | Ans | Domain | Why |
|---|-----|--------|-----|
| 1 | **A** | D1 | Deterministic compliance → prerequisite gate; B/C probabilistic; D addresses availability not ordering. |
| 2 | **B** | D2 | Tool descriptions are the primary selection mechanism; few-shot/routing/consolidation are over-engineering or root-cause misses for "first step." |
| 3 | **A** | D5 | Explicit criteria + few-shot is the proportionate first response; B/C/D are unreliable/over-engineered. |
| 4 | **C** | D5 | Honor explicit human-agent request **immediately**; don't gate on FCR. |
| 5 | **C** | D5 | Ask for additional identifiers; heuristic selection is forbidden. |
| 6 | **B** | D1 | Programmatic hook = guarantee for the >$500 policy; prompt is probabilistic. |
| 7 | **B** | D1 | PostToolUse hook normalizes before model sees results; deterministic across heterogeneous tools. |
| 8 | **B** | D1 | Self-contained handoff: ID + root cause + amount + recommended action. |
| 9 | **B** | D1 | Decompose → investigate (parallel, shared) → synthesize unified resolution. |
| 10 | **B** | D1 | `tool_use` continues the loop. |
| 11 | **C** | D1 | Parsing NL text for "done" is an explicit anti-pattern. |
| 12 | **A, C** | D5 | Explicit human request and inability to progress are valid triggers. (B sentiment, D routine = wrong.) |
| 13 | **B** | D5 | Self-confidence poorly calibrated, especially hard cases. |
| 14 | **B** | D1 | Append `tool_result` to history matched to `tool_use_id`. |
| 15 | **B** | D5 | Scratchpad files counteract context degradation. (Bigger model/re-summarize = wrong.) |
| 16 | **B** | D1 | Coordinator's narrow decomposition; subagents worked correctly within scope. |
| 17 | **A** | D5 | Structured error context enables recovery. |
| 18 | **A** | D2 | Scoped tool = principle of least privilege; preserve pattern for complex cases. |
| 19 | **B** | D1 | `Task` in `allowedTools` is required to spawn subagents. |
| 20 | **B** | D1 | Isolated context; explicit passing only. |
| 21 | **B** | D1 | Multiple Task calls in a single response. |
| 22 | **B** | D1 | Goals + quality criteria enable adaptability. |
| 23 | **C** | D5 | Annotate conflict with attribution; don't pick arbitrarily. |
| 24 | **B** | D5 | Publication/collection dates prevent temporal differences being misread as contradictions. |
| 25 | **B** | D5 | Render types appropriately; don't unify. |
| 26 | **A** | D5 | Coverage annotations flag well-supported vs gaps. |
| 27 | **A** | D1 | Coordinator's iterative refinement on gaps. |
| 28 | **B** | D1 | Preserve attribution/provenance when passing between agents. |
| 29 | **B** | D5 | Return structured data not verbose content for limited downstream budgets. |
| 30 | **B** | D1 | `fork_session` = branches from shared baseline. |
| 31 | **A** | D3 | `-p` (`--print`) = non-interactive; others are non-existent. |
| 32 | **A** | D4 | Batch for overnight (no SLA); sync for blocking pre-merge. |
| 33 | **A** | D4 | Per-file + integration passes fix attention dilution. |
| 34 | **A** | D3 | `--output-format json` + `--json-schema` for structured CI output. |
| 35 | **B** | D3 | Include prior findings → only new/unaddressed. |
| 36 | **B** | D3 | Provide existing tests + standards/fixtures in CLAUDE.md. |
| 37 | **B** | D4 | Independent instance > self-review. |
| 38 | **A** | D4 | Explicit criteria + improve prompts; proportionate response. |
| 39 | **B** | D4 | Vague "be conservative" fails vs specific criteria. |
| 40 | **A, B** | D3 | Document standards in CLAUDE.md + provide existing tests. (C/D wrong.) |
| 41 | **B** | D4 | Batches: no multi-turn tool calling within a request. |
| 42 | **A** | D4 | Cadence ensures 24h max batch fits in 30h SLA. |
| 43 | **B** | D4 | `tool_use` + JSON schema = most reliable. |
| 44 | **B** | D4 | `"any"` guarantees a tool call; choose which. |
| 45 | **A** | D4 | Forced tool_choice then follow-up turns. |
| 46 | **B** | D4 | Optional/nullable prevents fabrication. |
| 47 | **B** | D4 | Enum + "other" + detail; "unclear" for ambiguity. |
| 48 | **A** | D4 | Retry with original doc + failed extraction + specific errors. |
| 49 | **B** | D4 | Ineffective when info absent from source. |
| 50 | **B** | D4 | Computed cross-checks catch semantic errors. |
| 51 | **B** | D5 | Stratified sampling + segment by type/field detects masked poor segments. |
| 52 | **B** | D5 | Route low-confidence/ambiguous to humans, prioritizing capacity. |
| 53 | **B** | D3 | Managed → user → project → directory. |
| 54 | **A** | D3 | User-level not shared via VCS; move to project-level. |
| 55 | **B** | D3 | `.claude/rules/` with `paths:` globs handles scattered files. |
| 56 | **B** | D3 | `context: fork` isolates skill output. |
| 57 | **B** | D2 | Grep = content search. |
| 58 | **B** | D2 | Glob = path search. |
| 59 | **B** | D2 | Read + Write fallback for non-unique Edit. |
| 60 | **B** | Misc | MCP hosting/containers explicitly out-of-scope. |

---

## Scoring
- Single-response: 1 pt each (54 items × 1 = 54).
- Multiple-response: 1 pt only if exactly correct (Q12, Q40 × full credit; partial none) → 6 items × 1 = 6.
- **Total 60.** Convert to %: (correct / 60) × 100.

> **Interpreting your score:**
> - **≥90%**: Exam-ready. Skim weak-area guides.
> - **85–89%**: On track. Re-read missed domain guides.
> - **70–84%**: Review domain guides for clusters; redo missed Qs.
> - **<70%**: Re-read domains in priority order (1 → 2 → 3 → 4 → 5); do hands-on labs.

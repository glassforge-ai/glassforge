# GlassForge — Product Vision

## Mission

Take any iOS codebase and modernize it for iOS 26 / Liquid Glass — safely, verifiably, at scale.

## The Problem

- April 28, 2026: All App Store submissions must use Xcode 26 / iOS 26 SDK
- ~1.2M apps need modernization (80-90% of existing apps still UIKit-based)
- Manual cost: $15K-$120K+ per app
- No iOS-specific tooling exists — every competitor (Grit.io, Moderne, Sourcegraph) is horizontal

## The Product

One binary. Four verbs. One score.

```
glassforge scan     → deterministic rules, 0-100 score
glassforge plan     → prioritized task list, grouped by file
glassforge execute  → AI agents fix code, one file at a time
glassforge verify   → re-scan, measure delta ("59 → 82")
```

The score is the deliverable. Not "we fixed some stuff" — it's a measurable improvement.

## Business Model

| Tier | What | Price |
|------|------|-------|
| Free | Open-source scanner (lead gen) | $0 |
| Assessment | Readiness report + migration plan | $2K-$25K |
| Migration sprint | Agent-powered code fixes, 4-8 weeks | $25K-$100K |

Unit economics: $1K-$3K LLM compute per 100K LOC app → $25K+ revenue → 80-95% gross margin.

---

## Architecture — Purpose-Built, Not Ported

Every component exists to serve one of the four verbs. Nothing else.

### The Pipeline

```
┌─────────┐     ┌─────────┐     ┌─────────────┐     ┌──────────┐
│  SCAN   │────▶│  PLAN   │────▶│  EXECUTE    │────▶│  VERIFY  │
│         │     │         │     │             │     │          │
│ Rules   │     │ Group   │     │ Agent per   │     │ Re-scan  │
│ engine  │     │ by file │     │ file in     │     │ Compare  │
│ 0-100   │     │ Priority│     │ worktree    │     │ scores   │
│ score   │     │ Prompt  │     │ Persona     │     │ Delta    │
└─────────┘     └─────────┘     └─────────────┘     └──────────┘
```

### Crate Map — What Each Does and Why

```
SCAN
  gf-scan         Rules engine. Replaces the Swift scanner.
                  Rust + tree-sitter for AST-aware detection.
                  Loads versioned JSON rules. Produces artifact JSON.
                  WHY: The Swift scanner uses substring matching —
                  false positives on comments/strings, misses multiline.
                  tree-sitter gives scope-aware, type-aware detection.

PLAN
  gf-plan         Reads scan artifact. Groups findings by file.
                  Assigns priority (blocking first). Generates
                  constrained prompts. Routes to specialist persona.
                  WHY: Separation from scan. Plan is the intelligence
                  layer — it decides what to fix and in what order.

EXECUTE
  gf-agent        Agent spawn + lifecycle. Worktree per file.
                  Pipes prompt to Claude via Max subscription.
                  Captures stream-json output. Emits events.
                  WHY: One concern — run the agent, capture output,
                  handle success/failure. No business logic.

  gf-persona      iOS specialist personas (deprecated API fixer,
                  Liquid Glass migrator, migration verifier).
                  System prompts with domain expertise.
                  WHY: Domain knowledge separate from execution.
                  Easy to add new specialists.

  gf-safety       Rate limiter, circuit breaker, cost tracker.
                  Prevents runaway agents.
                  WHY: Agents are expensive. Safety is non-negotiable.

  gf-git          Git worktree create/remove per file.
                  WHY: Isolation. If an agent breaks a file,
                  discard the worktree. Zero damage.

VERIFY
  (gf-scan)       Same scan engine, run again on the modified code.
                  Compare before/after artifacts.
                  WHY: The score IS the product. Must be re-measurable.

INFRASTRUCTURE
  gf-core         Event types, IDs, error types. The type system
                  shared across all crates.
                  WHY: Consistent types, no circular deps.

  gf-db           SQLite persistence. Sessions, events, scan results,
                  migration results. BatchWriter for async persistence.
                  WHY: Auditability. Every scan, every agent action,
                  every result — queryable.

  gf-api          HTTP server + embedded SvelteKit frontend.
                  REST endpoints for scan/plan/execute/verify.
                  WebSocket for real-time agent streaming.
                  WHY: Not everyone wants CLI. Web UI for monitoring,
                  reviewing results, team collaboration.

  gf-app          Binary entry point. CLI commands + serve command.
                  WHY: The thing you install and run.
```

### What's Removed (vs. current state)

| Old Crate | Verdict | Why |
|-----------|---------|-----|
| gf-process | **Absorbed into gf-agent** | Process spawning is agent execution. One crate. |
| gf-migrate | **Split into gf-plan** | "Migrate" was doing plan + execute. Separate concerns. |
| gf-governance | **Cut** | Enterprise feature. Not v1. |
| gf-org | **Cut** | Enterprise feature. Not v1. |
| gf-mcp | **Cut for now** | MCP exposure can be added later. |
| gf-diagram | **Cut** | Frontend-only concern. |
| gf-graph | **Cut** | Not needed for migration. |
| gf-harvest | **Cut** | Not needed. |
| gf-learn | **Cut** | Not needed. |
| gf-llm | **Cut** | Only one backend (Claude). Abstraction adds nothing. |
| gf-metrics | **Cut** | Scoring lives in gf-scan. |
| gf-narrate | **Cut** | Not needed. |
| gf-parse | **Absorbed into gf-scan** | tree-sitter parsing IS scanning. |
| gf-search | **Cut** | Not needed. |
| gf-export | **In gf-api** | Export is a route, not a crate. |

### Target: 10 crates

```
gf-scan       NEW    Rust scanner with tree-sitter (replaces Swift CLI)
gf-plan       NEW    Planner (extracted from gf-migrate)
gf-agent      REWORK Merged gf-process + gf-agent + spawn logic
gf-persona    KEEP   Persona loader + finding→specialist routing
gf-safety     KEEP   Rate limiter, circuit breaker, cost tracker
gf-git        KEEP   Worktree isolation
gf-core       KEEP   Event types, IDs, errors, EventBus
gf-db         KEEP   SQLite, repos, BatchWriter
gf-api        KEEP   HTTP server + embedded frontend
gf-app        KEEP   CLI binary entry point
```

---

## The Scanner Question: Swift → Rust

### Current Swift Scanner (1,280 LOC)

**How it works:** Line-by-line `string.contains(pattern)` matching.

**Critical flaws:**
- `"keyWindow"` in a comment → flagged (false positive)
- `UIApplication.shared\n  .keyWindow` → missed (false negative)
- `class KeyWindowManager` → flagged (false positive)
- Single `.glassEffect()` call = "adopted" (misleading readiness)
- No scope awareness, no type resolution, no dead code detection

**What it gets right:**
- Deterministic (same input → same output)
- Fast (~1s for 100K LOC)
- Offline (no cloud, no data leaves machine)
- JSON rules are easy to update

### Why Rust + tree-sitter

tree-sitter has a Swift grammar. With AST parsing:

```
BEFORE (string matching):
  line.contains("keyWindow")  →  matches comments, strings, type names

AFTER (AST-aware):
  find all member_access where member == "keyWindow"
    and parent is call_expression on UIApplication.shared
  →  only matches actual deprecated API usage
```

Gains:
- Eliminate false positives (comments, strings, type names)
- Catch multiline patterns
- Scope-aware (skip `#if DEBUG`, `@available` guards)
- Type-aware (distinguish `UIApplication.keyWindow` from `myObj.keyWindow`)
- Dead code detection (unreachable paths)

Cost:
- Estimated 3,000-4,000 LOC (vs 1,280 Swift)
- tree-sitter Swift grammar maintenance
- Rules need to describe AST patterns, not just substrings

### Recommendation

Port the scanner to Rust. The false positive/negative rate of substring matching undermines trust — and trust is the product. If the scan says "score 66" but half the findings are false positives, the assessment is worthless.

The rules JSON can still drive detection — but instead of `{ "symbol": "keyWindow" }` meaning "does this string appear on any line", it means "does this AST node appear as a member access in active code".

---

## Frontend — 4 Pages, One Story

The web UI tells the same story as the CLI: scan → plan → execute → verify.

| Page | What it shows | Key UI |
|------|--------------|--------|
| **Scan** | Trigger scan, view readiness score | Score gauge (0-100), findings breakdown, recent scans |
| **Plan** | Review migration tasks before executing | Task table: file, priority, persona, finding count |
| **Execute** | Watch agents work in real-time | Swim-lane columns (Pending→Running→Done→Failed), streaming output |
| **Verify** | Compare before/after | Score delta visualization, resolved findings count |

Stack: SvelteKit 5, adapter-static, embedded in binary via rust-embed.

---

## What Success Looks Like

### Week 1-2
- `glassforge scan` runs on 3 OSS iOS apps, produces accurate scores
- `glassforge plan` + `execute` fixes 10+ findings on SwiftFormat
- `glassforge verify` shows measurable score improvement
- Publish first case study

### Month 1
- 3 free pilot assessments delivered to real iOS teams
- Open-source scanner published
- Landing page + content live

### Month 2-3
- First paid assessment ($2K-$5K)
- First paid migration sprint ($25K+)
- 3+ published case studies with before/after scores

### Month 6
- $100K+ revenue from assessments + migration sprints
- Web dashboard available for team collaboration
- Scanner covers 50+ deprecated APIs, 200+ pod mappings

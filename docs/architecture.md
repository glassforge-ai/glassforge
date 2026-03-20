# GlassForge Architecture

## Mission

Take any iOS codebase and modernize it for iOS 26 / Liquid Glass — safely, verifiably, and at scale.

## The Pipeline

```
Scan → Plan → Execute → Verify
```

This is the core product. Each stage is a distinct system. The value is in the chain, not any single piece.

---

## Stage 1: Scan (Swift CLI)

**Component:** `scanner/` (Swift 5.9+, SPM)

**What it does:** Analyzes an iOS codebase and produces a structured JSON artifact with a readiness score (0-100).

**Key technique: Rules-driven detection, not AI-driven.**

Most tools use AI to detect what's wrong. We don't. The scanner uses versioned JSON rule files:

- `deprecated_apis_ios26.json` — 24+ patterns with exact symbol names, severity, replacement, notes
- `dependencies_ios26.json` — 150+ CocoaPods mapped to SPM alternatives with migration difficulty
- `liquid_glass_ios26.json` — positive signals, migration targets, anti-patterns

Detection is deterministic. Same input always produces the same scan. No hallucinations, no false positives from AI guessing, no inconsistency across runs.

**Why this matters:** If your diagnosis is unreliable, everything downstream is wrong. Deterministic rules make the scan trustworthy enough to build automation on top of.

### Readiness Score

7-category weighted score (sums to 100):

| Category | Weight | What it measures |
|----------|--------|------------------|
| Deprecated APIs | 22% | Count and severity of deprecated API usage |
| Dependencies | 18% | CocoaPods presence, SPM migration difficulty |
| UI Stack | 15% | Storyboard/XIB count, SwiftUI adoption |
| Objective-C | 15% | ObjC file ratio, mixed modules |
| Liquid Glass | 10% | Positive signals vs. anti-patterns |
| Layout | 10% | Hard-coded frames, fixed constants |
| Testing | 10% | Test file presence and count |

Severity weights for deprecated APIs:
- Blocking: 3.0x (removed from SDK — app won't compile)
- High: 2.0x (deprecated and risky)
- Medium: 1.0x (works but wrong)
- Low: 0.5x (cosmetic or future-proofing)

Score bands:
- 75-100 (green): Ready / low effort
- 50-74 (yellow): Moderate effort
- 0-49 (red): High/critical effort

### Output

JSON artifact (`glassforge-artifact.json`) containing:
- Readiness score and band
- Every finding with: symbol, file, line, severity, replacement, notes
- Liquid Glass signals (positive, migration targets, anti-patterns)
- Dependency risk assessment
- UI stack metrics
- Metadata (repo, branch, commit, scan duration)

---

## Stage 2: Plan (Rust — forge-migrate)

**Component:** `crates/gf-process/` + migration planner (currently in `agentforge-hq/crates/forge-migrate/`)

**What it does:** Reads a scan artifact and generates a prioritized migration plan — a list of scoped tasks, each targeting one file.

### Key technique: Finding-to-prompt compilation

A scan finding is data: "line 154 has `UIScreen.main.bounds`."
An agent needs a job: "open this file, understand the context, replace it, verify it compiles."

The planner bridges this gap:

1. **Group by file.** All findings for a single file become one task. One agent handles all issues in that file — fewer context switches, no merge conflicts.

2. **Sort by severity.** Blocking first, then high, medium, low. Fix what breaks the build before what looks ugly.

3. **Generate constrained prompts.** Each task gets a prompt with:
   - Exact file path
   - Exact line numbers and symbols to fix
   - Exact replacements from the rules
   - Explicit constraints: "do not change behavior", "keep function signatures", "only modify this file"

### Why constrained prompts matter

"Please modernize this iOS app" → hallucinated code, changed behavior, broken tests.

"Fix these 3 specific deprecated APIs in this specific file with these specific replacements. Do not change behavior. Run swift build after." → reliable, verifiable change.

The prompt is a scalpel, not a sledgehammer. The rules provide the intelligence; the agent provides the hands.

### Example output

```
54 tasks across 54 files (76 findings)

1. [blocking] ArticleViewController+TableOfContents.swift (1 finding)
   → Replace UIWebView with WKWebView
2. [high] WMFNavigationBarHiding.swift (5 findings)
   → Replace keyWindow, statusBarFrame
3. [medium] NotificationsCenterViewController.swift (3 findings)
   → Replace UIScreen.main.bounds, .bar material
...
```

---

## Stage 3: Execute (Rust — agent orchestrator)

**Components:**
- `crates/gf-process/` — spawns Claude Code CLI, parses streaming output
- `crates/gf-safety/` — circuit breaker, rate limiter, cost tracker
- `crates/gf-git/` — git worktree isolation
- `crates/gf-agent/` — agent model, 10 presets

### Key technique: One-agent-per-file isolation

**Problem:** Multiple agents editing the same file simultaneously → merge conflicts, broken code.

**Solution:**
- Each MigrationTask targets exactly one file
- All findings for that file are bundled into one prompt
- The agent runs in an isolated git worktree (a separate checkout of the repo)
- If the agent fails, the worktree is deleted — zero damage to the main branch

This is what makes "run 54 migration tasks" safe. Each one is isolated, reversible, and scoped.

### Safety controls

| Control | What it does | Why |
|---------|-------------|-----|
| Circuit breaker | 3-state FSM (closed/open/half-open). Opens after N consecutive failures. | Stops wasting money on a broken model |
| Rate limiter | Token bucket algorithm. Configurable max requests per time window. | Prevents API abuse |
| Cost tracker | Per-session and per-company budget enforcement. Warn and hard-stop thresholds. | Controls spend |
| Loop detector | Sliding-window hash dedup on agent output. | Catches agents stuck in loops |
| Quality gates | Post-execution checks before accepting changes. | Rejects bad output |
| Exit gates | Pre-merge validation. | Final safety check |

### Execution flow

```
For each MigrationTask (sorted by priority):
  1. Create git worktree from main branch
  2. Spawn Claude Code agent with constrained prompt
  3. Stream output via WebSocket (real-time visibility)
  4. Agent edits the file, runs swift build
  5. If success: keep worktree, collect diff
  6. If failure: delete worktree, log error, continue
  7. Apply safety checks (cost, quality, exit gates)
```

### Agent presets

10 built-in presets, each with a tuned system prompt:

| Preset | Role |
|--------|------|
| CodeWriter | Write and modify code |
| Reviewer | Review code changes for correctness |
| Tester | Write and run tests |
| Debugger | Diagnose and fix bugs |
| Architect | Design system structure |
| Documenter | Write documentation |
| SecurityAuditor | Find security issues |
| Refactorer | Improve code structure |
| Explorer | Understand codebases |
| Coordinator | Orchestrate multi-agent work |

For migration, the primary preset is CodeWriter with the constrained prompt from the planner.

---

## Stage 4: Verify (Swift CLI — re-scan)

**Component:** Same scanner as Stage 1.

**What it does:** Re-scans the codebase after migration and compares scores.

### Key technique: Scan → Fix → Re-scan loop

The same tool that found the problem verifies the fix:

- Before: `glassforge analyze ./app` → Score: 59/100, 32 deprecated APIs, 1 blocking
- Agent fixes the blocking `UIWebView` usage
- After: `glassforge analyze ./app` → Score: 64/100, 31 deprecated APIs, 0 blocking

**Why this matters:** The score is the product's unit of value. "We took your app from 59 to 82" is a concrete, verifiable deliverable. Not "we ran some AI on your code" — a number that went up, with a diff to prove it.

### Verification report

The before/after comparison shows:
- Score delta (e.g., 59 → 82)
- Findings resolved (count and list)
- Findings remaining
- Files modified (with diffs)
- Any new issues introduced

---

## Design Decisions

### Why deterministic scan + AI execution?

| Approach | Detection | Execution | Risk |
|----------|-----------|-----------|------|
| All AI | AI finds issues, AI fixes them | Hallucinated findings, inconsistent across runs |
| All rules | Rules find issues, rules fix them | Can't handle context-dependent fixes |
| **Our approach** | Rules find issues, AI fixes them | Reliable diagnosis, intelligent treatment |

### Why one binary?

The platform compiles to a single binary (Rust + embedded SvelteKit frontend):
- No Docker, no cloud, no setup
- `./glassforge` starts the web UI on localhost
- `./glassforge scan ./repo` runs the scanner (once Swift scanner is integrated as subprocess)
- Customer's code never leaves their machine

### Why file-scoped tasks, not repo-scoped?

A repo-scoped prompt ("modernize this entire app") fails because:
- Context window limits: large apps exceed token limits
- Conflict risk: touching many files in one pass → merge conflicts
- Debugging: if something breaks, you don't know which change caused it
- Cost: one failed attempt wastes the entire run

File-scoped tasks are:
- Small enough to fit in context
- Isolated enough to be safe
- Specific enough to verify
- Cheap enough to retry

### Why severity ordering?

Blocking issues first because:
- A blocking issue (UIWebView removed from SDK) means the app won't compile on iOS 26
- Fixing it first ensures the codebase stays buildable throughout migration
- Each subsequent fix can be verified with `swift build`
- You deliver value immediately: "your app compiles on iOS 26 now"

---

## What's Novel

| Technique | Standard? | Our adaptation |
|-----------|-----------|---------------|
| Rules-based static analysis | Standard in linting | Applied to iOS 26 / Liquid Glass — nobody else has these specific rules |
| Finding-to-prompt compilation | Novel | Scan artifact → scoped agent task with constrained prompt |
| Deterministic score + AI fix | Novel split | Detection is trustworthy, execution is intelligent |
| Severity-weighted prioritization | Standard | Tuned for iOS 26 deadline (blocking = SDK removal) |
| Git worktree per agent | Standard in CI | Applied to multi-agent code migration |
| Scan-fix-rescan loop | Novel as product | Score delta is the deliverable |

**The key insight:** No single technique is groundbreaking. The pipeline is the product. Nobody else has: iOS-specific rules → deterministic scan → prioritized plan → constrained agent prompts → isolated execution → automated re-verification.

---

## Component Map

```
glassforge-platform/
├── scanner/              ← Stage 1: Scan (Swift CLI)
│   ├── Core/Scanner.swift
│   ├── Core/SwiftAnalyzer.swift
│   ├── Core/LiquidGlassAnalyzer.swift
│   ├── Core/Scoring.swift
│   └── Rules/*.json
│
├── crates/
│   ├── gf-core/          ← Foundation: events, IDs, errors
│   ├── gf-db/            ← Foundation: SQLite, repos, migrations
│   ├── gf-agent/         ← Stage 3: agent model, presets
│   ├── gf-process/       ← Stage 2+3: prompt execution, streaming
│   ├── gf-safety/        ← Stage 3: circuit breaker, rate limiter
│   ├── gf-git/           ← Stage 3: worktree isolation
│   ├── gf-api/           ← Web UI: Axum HTTP + WebSocket
│   └── gf-app/           ← Binary: CLI + web server
│
├── personas/             ← 112 agent role definitions
├── rules/ios26/          ← Versioned detection rules
├── migrations/           ← SQLite schema
└── frontend/             ← SvelteKit 5 dashboard
```

---

## Data Flow

```
                    ┌─────────────┐
                    │  iOS Repo   │
                    └──────┬──────┘
                           │
                    ┌──────▼──────┐
                    │   Scanner   │  Swift CLI, deterministic rules
                    │  (Stage 1)  │
                    └──────┬──────┘
                           │
                    JSON artifact
                    (score, findings)
                           │
                    ┌──────▼──────┐
                    │   Planner   │  Group by file, sort by severity
                    │  (Stage 2)  │  Generate constrained prompts
                    └──────┬──────┘
                           │
                    Vec<MigrationTask>
                           │
               ┌───────────┼───────────┐
               │           │           │
        ┌──────▼──┐  ┌─────▼───┐  ┌───▼──────┐
        │ Agent 1 │  │ Agent 2 │  │ Agent N  │  Claude Code in worktrees
        │ (file A)│  │ (file B)│  │ (file N) │  with safety controls
        └────┬────┘  └────┬────┘  └────┬─────┘
             │            │            │
             └────────────┼────────────┘
                          │
                   Modified repo
                          │
                   ┌──────▼──────┐
                   │  Re-scan    │  Same scanner, compare scores
                   │  (Stage 4)  │
                   └──────┬──────┘
                          │
                   Before/after report
                   (score: 59 → 82)
```

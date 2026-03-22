# GlassForge — Crate Map

Purpose-built for scan → plan → execute → verify. Nothing else.

## Pipeline Crates

```
SCAN ─────────────────────────────────────────────────────────────
gf-scan       Rust scanner with tree-sitter Swift/ObjC parsing.
              Loads versioned JSON rules. AST-aware detection.
              Produces glassforge-artifact.json.
              Replaces the Swift CLI scanner.

PLAN ─────────────────────────────────────────────────────────────
gf-plan       Reads scan artifact. Groups findings by file.
              Assigns priority (blocking → high → medium → low).
              Generates constrained prompts per file.
              Routes each task to a specialist persona.
              Produces MigrationPlan with ordered MigrationTasks.

EXECUTE ──────────────────────────────────────────────────────────
gf-agent      Spawns Claude Code agents. One per file.
              Manages process lifecycle (start → stream → done/fail).
              Parses stream-json output into ForgeEvents.
              Routes through Max subscription (env vars).
              Includes concurrent runner + spawn limiter.

gf-persona    Loads iOS specialist personas from markdown files.
              Maps finding source → persona:
                DeprecatedApi  → iOS Deprecated API Fixer
                LiquidGlass    → iOS Liquid Glass Migrator
                (post-task)    → iOS Migration Verifier

gf-git        Creates git worktree per migration task.
              Removes on failure, keeps on success.
              Lists active worktrees for cleanup.

gf-safety     Rate limiter (token bucket).
              Circuit breaker (3-state FSM).
              Cost tracker (budget warn/limit).

VERIFY ───────────────────────────────────────────────────────────
              Re-runs gf-scan on modified code.
              Compares before/after artifacts.
              (No separate crate — verify is scan + diff.)
```

## Infrastructure Crates

```
gf-core       ForgeEvent enum (process lifecycle events).
              EventBus (broadcast + persist channels).
              Typed IDs (SessionId, AgentId, EventId).
              ForgeError + ForgeResult.

gf-db         SQLite WAL mode. Connection pool (r2d2).
              Repos: sessions, events, scans, migrations.
              BatchWriter (async event persistence).
              Migrator (SQL migrations).

gf-api        Axum HTTP server.
              Routes: /scan, /plan, /execute, /verify, /sessions, /ws.
              WebSocket: real-time ForgeEvent streaming.
              rust-embed: embedded SvelteKit frontend.
              AppState: wires all crates together.

gf-app        Binary entry point. clap CLI.
              Commands: scan, plan, execute, verify, serve.
              Startup: DB, EventBus, safety, backends, personas.
              Graceful shutdown.
```

## Total: 10 crates

| Crate | Pipeline stage | Current status | Action |
|-------|---------------|----------------|--------|
| gf-scan | Scan | NEW (Swift CLI exists but needs rewrite) | Build with tree-sitter |
| gf-plan | Plan | Partial (gf-migrate has planner) | Extract + rename |
| gf-agent | Execute | Partial (gf-process + gf-agent exist) | Merge into one |
| gf-persona | Execute | Done (built today) | Keep |
| gf-git | Execute | Done (existed) | Keep |
| gf-safety | Execute | Done (existed) | Keep |
| gf-core | Infra | Done (existed) | Keep |
| gf-db | Infra | Done (existed) | Extend (scans + migrations tables) |
| gf-api | Infra | Partial (built today, needs domain routes) | Extend |
| gf-app | Infra | Partial (built today, CLI works) | Extend |

## What's Cut

17 stub crates removed. No gf-governance, gf-org, gf-mcp, gf-diagram, gf-graph,
gf-harvest, gf-learn, gf-llm, gf-metrics, gf-narrate, gf-parse, gf-search, gf-export,
gf-analyze.

These can be added back if the product grows. For now: 10 crates, 4 verbs, 1 score.

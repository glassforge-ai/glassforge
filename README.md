# GlassForge

**AI-powered iOS modernization platform.** Scan, plan, execute, and verify your iOS 26 / Liquid Glass migration — in one system.

## What GlassForge Does

```
Scan → Plan → Execute → Learn
```

1. **Scan** — Swift CLI analyzes your iOS codebase: readiness score, deprecated APIs, Liquid Glass signals, dependency risks, layout issues. Deterministic. No cloud.
2. **Plan** — AI agents generate a prioritized migration roadmap based on your scan results and Apple's HIG.
3. **Execute** — Multi-agent orchestrator runs parallel code migrations with safety controls: circuit breaker, budget enforcement, quality gates, git worktree isolation.
4. **Learn** — Gamified codebase exploration with AI-generated chapters, 3D dependency graphs, quizzes, and XP tracking — so your team understands the modernized code.

## Architecture

Single binary (Rust + embedded SvelteKit 5 frontend) + Swift CLI scanner.

```
glassforge-platform/
├── crates/          23 Rust crates (Axum + SQLite + Tokio)
├── scanner/         Swift CLI (SPM, iOS 26 rules engine)
├── frontend/        SvelteKit 5 + Tailwind CSS 4
├── personas/        112 AI agent personas across 11 divisions
├── rules/           Versioned JSON rules for iOS 26 analysis
├── migrations/      Unified SQLite schema
└── docs/            Liquid Glass guides + HIG reference
```

### Rust Crates (23)

**Foundation:**
- `gf-core` — Events, shared types, error types, typed IDs
- `gf-db` — SQLite WAL, connection pool, repos, batch writer

**Agent Orchestration:**
- `gf-agent` — Agent model, 10 presets, validation
- `gf-process` — Claude CLI spawn, stream parsing, concurrent runner
- `gf-safety` — Circuit breaker, rate limiter, cost tracker
- `gf-git` — Git worktree isolation
- `gf-persona` — 112 persona catalog loader
- `gf-org` — Company, department, org chart
- `gf-governance` — Approval workflows

**Codebase Analysis:**
- `gf-parse` — Tree-sitter incremental parsing (12 languages)
- `gf-graph` — Dependency graph, Louvain community detection
- `gf-metrics` — SLOC, complexity, fan-in/out, churn
- `gf-analyze` — Pattern detection, code smells
- `gf-diagram` — Architecture diagram generation
- `gf-search` — BM25 full-text symbol search
- `gf-llm` — LLM provider abstraction
- `gf-narrate` — AI-generated narrative explanations
- `gf-learn` — Curriculum, quizzes, XP, badges, streaks
- `gf-harvest` — Trending repo tracking
- `gf-export` — Static page export

**Service Layer:**
- `gf-api` — Axum HTTP + WebSocket, embedded SPA
- `gf-mcp` — MCP server (36 tools)
- `gf-app` — Binary entry point, DB init, cron, graceful shutdown

### Swift Scanner

Standalone CLI that produces JSON artifacts. Invoked by the Rust backend as a subprocess.

- 150+ CocoaPods → SPM migration rules
- 24+ deprecated API detection rules
- Liquid Glass readiness signals (positive, migration targets, anti-patterns)
- 7-category weighted readiness score (0-100)
- CI-friendly with `--fail-under` gating

### Frontend

Unified SvelteKit 5 dashboard with four domains:

- **Scanner** — iOS 26 readiness dashboard, scan history, remediation guidance
- **Forge** — Agent management, sessions, workflows, personas, org charts, approvals, analytics
- **Code** — 3D graph explorer, file tree, metrics, architecture diagrams
- **Learn** — Gamified learning paths, AI chapters, quizzes, streaks

## Quick Start

```bash
# Build everything
make build

# Run the platform
./target/release/glassforge
# Opens http://localhost:4173

# Or just run the scanner standalone
cd scanner && swift build -c release
.build/release/glassforge-scan analyze /path/to/ios-app
```

## Requirements

- Rust 1.75+
- Swift 5.9+ / macOS 13+
- Node.js 20+ / pnpm 9+

## License

MIT License. See [LICENSE](LICENSE).

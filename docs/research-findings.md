# Research Findings & Strategic Decisions

Compiled from 3 rounds of deep research (Perplexity, March 2026) plus internal analysis.

---

## Market

- **~2.0-2.2M** live apps on the App Store (2025-2026)
- **80-90%** of existing apps still powered by UIKit
- **~60-70%** have sizeable UIKit/storyboard/ObjC surfaces = **~1.2M modernization candidates**
- Manual modernization costs: **$15K-$120K+** per app
- Senior iOS dev rates: **$61-$140/hr** in Western markets

### TAM by Segment

| Segment | Effective Apps | Typical Project Value |
|---------|---------------|---------------------|
| Enterprise | 60-100K | $50K-$250K+ |
| Mid-market SaaS | 60-100K | $25K-$100K |
| Indie/Agency | 60-120K | $5K-$40K |

### Apple's Forcing Function

- Liquid Glass (iOS 26) is the biggest design shift since iOS 7 (2013)
- Apple deprecation pattern: **soft pressure, not hard breaks** — editorial bias, platform features, Apple Intelligence integration
- Frameworks kept for 8-12+ years; backwards compatibility expected
- Position as "Apple rewards compliance" not "Apple punishes non-compliance"

---

## Competitive Landscape

| Competitor | Focus | iOS-specific? |
|-----------|-------|---------------|
| Grit.io | JS/TS/Java migrations | No |
| Moderne | JVM refactoring (OpenRewrite) | No |
| Sourcegraph Agents | Enterprise migration | No |
| Devin | General AI engineer | No |
| Cursor/Copilot | AI coding assistant | No |

**Nobody is vertically positioned around iOS UIKit → SwiftUI / Liquid Glass.**

Our differentiation: vertical focus + governance + deterministic scanning + supervised multi-agent execution.

---

## Business Model

### Recommended: Managed Service + Platform (Early Stage)

**Tier 1 — Readiness Assessment**
- SMBs: $1K-$5K
- Enterprise multi-app: $10K-$25K

**Tier 2 — Guided Modernization Sprints**
- 4-8 week engagement, pre-quoted by complexity
- Price at 30-60% of manual cost ($25K-$100K range)

### Unit Economics

- 100K-200K LOC app: **$1K-$3K** in LLM compute
- Charge: **$25K+**
- Gross margin: **80-95%**

---

## Go-to-Market

### Target First: Mid-market + Agencies
- Mid-market: serious iOS revenue, small team, tech debt pain
- Agencies: many legacy client apps, margin pressure

### Channels
- X/Twitter iOS dev circles
- iOS newsletters, Swift forums
- WWDC remote watch parties
- CI/CD partnerships (Bitrise, Codemagic, GitHub Actions)

### Open-Source Funnel
GlassForge scanner (free, OSS) → paid assessment → paid migration

### WWDC Timeline
- Pre-WWDC: ship scanner, publish content
- WWDC window: update scanner for new APIs within days
- Post-WWDC: push modernization sprints

---

## Product Strategy

### What Perplexity Recommended
- **One wedge:** GlassForge-powered iOS modernization for teams
- **Everything else supports it:** AgentForge = execution engine, codeilus = onboarding feature, apple-glass-doc = knowledge base
- **Do not** launch multiple products simultaneously
- **Do not** build consumer App Store app yet

### Wedge Product vs. Platform Play
Pattern from Vercel, Supabase, Linear, Raycast: one sharp wedge + OSS/free tier → land → widen.

### Revenue Stacking (Strict Hierarchy)
1. **Primary:** High-ticket B2B modernization assessments and guided migrations
2. **Lead-gen:** Free OSS scanner CLI
3. **Optional (later):** iOS dev copilot on App Store — only if it feeds the B2B brand

### What NOT to Do
- Don't launch multi-product landing page
- Don't monetize 3 SKUs in first 6 months
- Don't build codeilus as standalone product yet
- Don't target consumer AI app market

---

## Technical Design Decisions

### Why Deterministic Scan + AI Execution?
- AI for detection = hallucinations, inconsistency
- Rules for detection = trustworthy, reproducible
- AI for execution = context-aware, intelligent fixes
- Split eliminates false positives in diagnosis while leveraging AI for treatment

### Why One-Agent-Per-File?
- Repo-scoped prompts fail: context limits, conflicts, debugging impossibility
- File-scoped tasks: small, isolated, verifiable, cheap to retry
- All findings for one file bundled into one prompt = fewer agents, fewer conflicts

### Why Constrained Prompts?
- "Modernize this app" → hallucinated code, changed behavior
- "Fix line 154: replace UIScreen.main.bounds with scene bounds. Don't change behavior." → reliable
- Rules provide intelligence; agent provides hands

### Why Severity Ordering?
- Blocking first: app won't compile without these fixes
- High: deprecated and risky
- Medium: works but wrong
- Low: cosmetic
- Each fix is verifiable with swift build before moving to next

---

## Risk Assessment

### Top 5 Existential Risks
1. Apple ships own migration tooling (Xcode-integrated)
2. Liquid Glass stays optional — no urgency
3. AI code quality plateaus — stuck as services business
4. Vendor concentration on Anthropic/Claude
5. Teams decide generic AI tools are "good enough"

### Mitigations
- Own analysis + planning + governance layers (Apple won't build org-specific planning)
- Multi-model architecture (pluggable LLMs)
- Expand beyond Liquid Glass → "continuous modernization autopilot"
- Position as "iOS modernization platform" not "Liquid Glass tool"

---

## 6-Month Sequencing

### Month 1-2: Sharpen the wedge
- Ship GlassForge OSS with readiness score
- Landing page + content marketing
- 2-3 pilot assessments (free)

### Month 3-4: Monetize
- Paid Assessment + Plan product ($2K-$25K)
- Agent-powered codemods (narrow: storyboard removal, deprecated APIs)
- GitHub integration for PRs

### Month 5-6: Decide on expansion
- Web dashboard for scan history + migration progress
- Optional: minimal iOS dev copilot experiment
- If modernization revenue working → double down
- If weak → pivot to dev tooling for iOS developers

---

## The Validation Question

> "Will mid-market iOS teams pay $5K-$25K for a rigorous iOS 26 readiness assessment + migration roadmap that saves them 1-2 engineer-months of planning?"

This is what we're testing with pilot customers.

---

*Sources: Perplexity Deep Research (March 2026), AppFigures AI App Report 2025, various cited in PERPLEXITY_RESEARCH.md*

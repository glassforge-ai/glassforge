# Next Steps — Prioritized Action Plan

## Current State (March 2026)

| Asset | Status |
|-------|--------|
| Scanner (Swift CLI) | Working, tested on 3 OSS apps |
| Migration planner (forge-migrate) | Working dry-run, tested on Wikipedia iOS (54 tasks, 76 findings) |
| Agent orchestrator (agentforge-hq) | Working, can spawn Claude Code agents |
| Unified platform (glassforge-ai/glassforge) | 6 real crates compile, 17 stubs |
| Landing page | Live on Cloudflare + GitHub Pages |
| Content (Substack, X, LinkedIn) | Drafted, not published |

**Gap: We have never actually modernized a single line of iOS code with this system.**

---

## Week 1: Prove it works

### 1. Run a live migration (Day 1-2)
- Pick a small open-source iOS app with 5-10 fixable findings
- Run `forge-migrate` without `--dry-run`
- Let Claude Code agents fix deprecated APIs in git worktrees
- Re-scan with GlassForge, measure score improvement
- Record the before/after as proof

### 2. Publish content (Day 2-3)
- Substack: "I Scanned 3 Popular iOS Apps for iOS 26 Readiness. None Scored Above 66."
- X thread (7 tweets)
- LinkedIn post
- DEV.to cross-post
- All drive to: scanner repo + landing page + contact links

### 3. Find 3 pilot users (Day 3-7)
- DM iOS leads: "I scanned Wikipedia iOS, scored 59/100. Want me to scan yours for free?"
- Offer free scan + readiness report
- Goal: get 3 real codebases, learn what they care about

---

## Week 2-3: First paid value

### 4. Productize the assessment
- Scan pilot repos → generate reports → deliver via PDF or Notion
- Include: score, top 10 findings, prioritized fix list, effort estimate
- Price: free for first 3 (case studies), then $2K-$5K

### 5. Run a guided migration for one pilot
- Pick most willing pilot
- Run forge-migrate on blocking + high findings only
- Open PRs with before/after
- Re-scan, show score improvement
- First real customer case study

---

## Week 4-6: Unify the product

### 6. Port forge-migrate into unified platform
- Move migration pipeline from agentforge-hq into gf-* crates
- Wire gf-app as CLI (`glassforge scan`, `glassforge migrate`) + web server (`glassforge serve`)
- Single binary

### 7. Build minimal web dashboard
- Scanner results page (upload artifact → see score + findings)
- Migration progress page (watch agents in real-time)
- Before/after comparison view

---

## Month 2-3: Scale

### 8. Open-source scanner under glassforge-ai org
- Clean CLI as `glassforge-ai/scanner`
- "Run in CI, get your readiness score"
- Top-of-funnel: free scanner → paid assessment → paid migration

### 9. Publish 3+ case studies
- Open-source app migrations with before/after scores
- Customer testimonials (anonymized if needed)
- Landing page + Substack + social

### 10. Bootstrap or raise decision
- 3+ paying customers → bootstrap
- Strong demand + need speed → small seed with case studies as proof

---

## What NOT to do

- Don't polish unified repo architecture before proving live migration works
- Don't build learning/gamification features until core pipeline pays for itself
- Don't launch multiple products (scanner + copilot + education)
- Don't build the 17 stub crates until the 6 real ones + migration pipeline ship
- Don't spend time on investor decks before having a customer

---

## The One Thing

**Run a live migration. Get a before/after score. Record it.**

Everything else flows from proof that this actually works.

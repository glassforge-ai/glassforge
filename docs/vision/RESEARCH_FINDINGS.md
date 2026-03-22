# Research Findings — March 21, 2026

Source: Perplexity research on 10 key questions.

---

## Scanner Architecture

### tree-sitter-swift: USE IT
- Actively maintained on crates.io, targets Swift 5.x
- Good enough for structural queries (method calls, property access, imports)
- Known gaps: result builders, macros, complex SwiftUI DSL — treat as "best effort"
- **Decision: tree-sitter as primary, regex fallback for parse errors**

### SwiftSyntax: OPTIONAL SIDECAR
- 100% accurate but it's pure Swift — can't call directly from Rust
- Possible via C FFI: small Swift helper, `@_cdecl` exports, JSON over boundary
- Complex to build and maintain, requires Xcode on machine
- **Decision: Optional "SwiftSyntax sidecar" for exact type resolution. Not v1.**

### Design for gf-scan
- tree-sitter-swift as primary parser (no Xcode dependency)
- Regex/heuristic fallback when tree-sitter reports errors
- Future: Optional SwiftSyntax sidecar for enterprise accuracy

---

## iOS 26 Deprecations

### NO SINGLE OFFICIAL LIST EXISTS
- Apple hasn't published a consolidated API-diff page for iOS 26
- Most breaking changes are design-system (Liquid Glass), not mass API removal
- Temporary Info.plist opt-out key exists for Liquid Glass compatibility
- **Must mine SDK headers ourselves**: scan for `API_DEPRECATED` / `API_DEPRECATED_WITH_REPLACEMENT`

### Known deprecations (confirmed)
- UIWebView (removed), keyWindow, UIScreen.main.bounds, statusBarFrame
- Layout/material changes driven by Liquid Glass adoption

### Strategy
- Ship with curated list (current 21 rules)
- Build SDK header scanner to auto-generate rules from Xcode 26 headers
- Update rules as community lists emerge

---

## Liquid Glass API Surface

### CONFIRMED PUBLIC APIs
- `.glassEffect()` modifier (SwiftUI)
- `GlassEffectContainer` type (for composing glass effects)
- Info.plist opt-out key ("UI design requires compatibility")

### UNCONFIRMED / DISCOVER LATER
- `.glassEffectID()` — not confirmed in public sources
- UIKit equivalents — no public evidence of UIGlassEffectView
- Full modifier variants (.regular, .clear, .identity, .tint) — mentioned in earlier research but not independently confirmed

### Strategy
- Scanner checks for: `.glassEffect(` usage, `GlassEffectContainer`, Info.plist opt-out
- Don't hard-code longer API list until SDK is available
- Update rules when Xcode 26 SDK ships

---

## Claude Max Subscription (CRITICAL)

### ENV VARS ARE UNDOCUMENTED AND BRITTLE
- `CLAUDE_CODE_ENTRYPOINT=sdk-max`, `CLAUDE_USE_SUBSCRIPTION=true`, `CLAUDE_BYPASS_BALANCE_CHECK=true`
- **NOT in any official Anthropic docs**
- Appear in reverse-engineered contexts only
- **Could break without notice in any CLI update**

### Rate Limits (account-level, not per-process)
- 5-hour rolling window
- $100 Max plan: 15-35 hrs Opus/week, 140-280 hrs Sonnet/week
- $200 Max plan: 24-40 hrs Opus/week, 240-480 hrs Sonnet/week
- Multiple concurrent `claude -p` sessions share the SAME account limit
- Can buy additional usage at API rates

### Design Implications
1. **Must support both paths**: Max subscription (env vars) AND API credits (ANTHROPIC_API_KEY)
2. **Don't assume unlimited usage** — respect rate limits, implement backoff
3. **gf-safety rate limiter becomes critical** — not just cost, but subscription quotas
4. **Consider Sonnet for bulk work, Opus for complex fixes** — budget the hours
5. **Have a fallback plan** for when env vars break — graceful error, suggest API credits

---

## Market Validation

### NO iOS-SPECIFIC COMPETITOR — CONFIRMED
- Grit.io: language-agnostic, no iOS module
- Sourcegraph Cody: general AI assistant, no iOS pipeline
- Cursor: general AI IDE, no migration product
- Educational guides exist (Kite Metric etc.) but no automation tools
- **GlassForge has a real, uncontested wedge**

### Consulting Pricing
- Top-tier agencies: $150-250+/hr (US/Western Europe)
- Multi-month modernization: low to mid six figures
- WillowTree, Accenture etc. — no public pricing, but case studies imply six-figure engagements
- **10-20% automation on a large app = tens of thousands displaced**

### April 2026 Deadline: HARD GATE
- All App Store submissions must use Xcode 26 / iOS 26 SDK
- CI/CD pipelines break if pinned to older toolchains
- Cannot ship updates (including security patches) without compliance
- Flutter/React Native ecosystems already releasing Xcode 26 compatible versions
- **Teams who delay WILL be blocked from shipping**

---

## Distribution

### Homebrew tap + direct binary download
- iOS devs expect Homebrew (SwiftLint, SwiftFormat pattern)
- `cargo install` is wrong audience (Rust devs, not iOS devs)
- Plan:
  1. Homebrew tap with prebuilt macOS binaries (arm64 + x86_64)
  2. Direct .tar.gz download + `curl | bash` installer
  3. Optional: `cargo install glassforge` for Rust users
  4. Sign the binary if possible (notarization for macOS)

---

## Key Design Decisions (Updated)

| Decision | Before research | After research |
|----------|----------------|----------------|
| Scanner parser | "Port to Rust + tree-sitter" | **tree-sitter primary, regex fallback, optional SwiftSyntax sidecar** |
| Deprecation rules | "Build complete list" | **Ship with curated 21, auto-mine SDK headers, update incrementally** |
| Liquid Glass rules | "Hard-code full API" | **Only confirmed APIs (.glassEffect, GlassEffectContainer, plist opt-out)** |
| Max subscription | "Rely on env vars" | **Support both Max AND API credits. Env vars are brittle.** |
| Agent model | "Always Opus" | **Sonnet for bulk, Opus for complex. Budget the hours.** |
| Distribution | "cargo install" | **Homebrew tap + direct binary. cargo install as bonus.** |
| Xcode dependency | "Maybe" | **No. Pure Rust binary. SwiftSyntax sidecar is optional, macOS-only.** |

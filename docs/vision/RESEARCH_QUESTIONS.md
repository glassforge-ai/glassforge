# Research Questions — Open Items

Questions that need answers before building the final product. Use Perplexity, Apple docs, or direct testing.

---

## Scanner (gf-scan)

### tree-sitter for Swift
1. **What is the current state of tree-sitter-swift?** Is the grammar maintained? What Swift version does it support? How complete is the AST for detecting method calls, property access, import statements?
2. **Can tree-sitter resolve qualified names?** E.g., distinguish `UIApplication.shared.keyWindow` (deprecated) from `myObj.keyWindow` (not deprecated). Or does that require a type checker beyond tree-sitter?
3. **What other iOS migration tools use for static analysis?** Does SwiftLint use SwiftSyntax (Apple's official AST)? Should we use SwiftSyntax compiled as a library instead of tree-sitter?
4. **SwiftSyntax vs tree-sitter tradeoffs:** SwiftSyntax is Apple's official parser (100% accurate AST). tree-sitter is language-agnostic (works for ObjC too). Which fits better for a Rust tool that needs to parse both Swift and ObjC?

### Rules accuracy
5. **What are ALL the APIs deprecated or removed in iOS 26?** Apple's official deprecation list. The current scanner has 21 rules — how many should there be?
6. **What are the exact Liquid Glass APIs?** `.glassEffect()`, `GlassEffectContainer`, `.glassEffectID()` — is this the complete surface? What about UIKit equivalents?
7. **Which CocoaPods have SPM equivalents as of 2026?** The current list has 40 pods — the target is 200+.

## Agent Execution

### Claude CLI + Max subscription
8. **Is `CLAUDE_CODE_ENTRYPOINT=sdk-max` an official/stable env var?** Or is it an implementation detail that could break? What's the risk of depending on it?
9. **What are the rate limits for Max subscription programmatic usage?** If we spawn 10 agents, do they share a rate limit? What's the 5-hour window?
10. **Can we run concurrent `claude -p` sessions on Max?** Or does each session queue behind the other?

### Agent quality
11. **What's the success rate of Claude fixing iOS deprecated APIs?** Has anyone benchmarked this? What's the failure mode — wrong fix, compilation error, behavior change?
12. **Should agents run `swift build` after each fix?** The personas say yes, but does Claude Code actually do this reliably in `-p` mode?

## Market

### Competitors
13. **Has anyone else shipped iOS-specific migration tooling since our last research (March 2026)?** Check: Grit.io iOS support, Sourcegraph Cody iOS capabilities, any new entrants.
14. **What do consulting firms actually charge for iOS modernization?** WillowTree, Accenture, Thoughtbot — any public pricing or case studies?

### Customer validation
15. **Where do iOS teams discuss migration pain?** Swift Forums, iOS Dev Weekly, specific Slack/Discord communities? Where should we publish content?
16. **What's the actual urgency of the April 2026 deadline?** Are teams already blocked? Or is this mostly forward-looking prep?

## Product

### Distribution
17. **Should we distribute as a Homebrew formula?** `brew install glassforge` — what's the process?
18. **Should the scanner be a separate npm/pip/cargo package** for maximum reach? Or always bundled in the binary?

### Pricing validation
19. **Would a team pay $2K for a PDF readiness report?** What format/depth makes a $2K-$5K assessment feel worth it?
20. **What's included in a $25K migration sprint?** PRs opened? Diff reviewed by human? Score guarantee?

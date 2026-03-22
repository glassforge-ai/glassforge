# tree-sitter-swift Prototype — Technical Reference

Source: Perplexity research, March 21, 2026.

## Cargo.toml

```toml
[dependencies]
tree-sitter = "0.24"
tree-sitter-swift = "0.7"
walkdir = "2"
anyhow = "1"
```

## Query Syntax for Deprecated API Detection

### Member access (e.g., `keyWindow`, `statusBarFrame`)
```scheme
(member_access
  member: (identifier) @member_name
  (#eq? @member_name "keyWindow"))
```

### Function call on deprecated member
```scheme
(call_expression
  function: (member_access
              member: (identifier) @member_name)
  (#eq? @member_name "keyWindow"))
```

### Import detection
```scheme
(import_declaration
  path: (identifier) @module_name)
```

### Class with protocol conformance
```scheme
(class_declaration
  name: (identifier) @class_name
  (inheritance_clause
    (type_identifier) @protocol_name))
```

## Key Node Types in tree-sitter-swift

| Swift construct | Node type | Fields |
|----------------|-----------|--------|
| `import Foundation` | `import_declaration` | path: identifier |
| `foo.bar` | `member_access` | base, member: identifier |
| `foo.bar()` | `call_expression` | function, arguments |
| `class Foo` | `class_declaration` | name, inheritance_clause |
| `struct Foo` | `struct_declaration` | name, inheritance_clause |
| `protocol Foo` | `protocol_declaration` | name |
| `// comment` | `comment` or `line_comment` | — |
| `"string"` | `string_literal` | — |

**Source of truth:** `node-types.json` in tree-sitter-swift crate sources on docs.rs.

## Filtering Out Comments and Strings

Walk ancestors — if any parent is `comment`, `line_comment`, `block_comment`,
`string_literal`, or `multiline_string_literal`, skip the match.

```rust
fn is_in_trivia(mut node: tree_sitter::Node) -> bool {
    while let Some(parent) = node.parent() {
        let kind = parent.kind();
        if matches!(kind,
            "comment" | "line_comment" | "block_comment"
            | "string_literal" | "multiline_string_literal"
        ) {
            return true;
        }
        node = parent;
    }
    false
}
```

## Performance

- Parsing is linear in file size. Tens of ms per file.
- 100K LOC Swift with a handful of queries: well under a second.
- Parse each file once, reuse tree for multiple queries.
- Use `QueryCursor` for all pattern matching (not manual traversal).

## Rule Engine Design

Instead of one query per deprecated API, load rules from JSON and generate queries:

```json
{
  "symbol": "keyWindow",
  "query_type": "member_access",
  "severity": "high",
  "replacement": "UIWindowScene.keyWindow",
  "notes": "Use window scene API"
}
```

At startup, compile each rule into a tree-sitter Query. Run all queries per file.
This keeps detection data-driven (JSON rules) with AST-aware execution (tree-sitter).

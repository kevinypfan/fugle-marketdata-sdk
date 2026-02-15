# Phase 15: Documentation & Migration - Research

**Researched:** 2026-02-15
**Domain:** Technical documentation, API migration tooling
**Confidence:** HIGH

## Summary

Phase 15 completes the v0.3.0 release by documenting the new options-object constructor API and providing migration tooling for users upgrading from v0.2.x. Research reveals that comprehensive documentation and migration support requires: (1) **README updates** with before/after examples for all 5 languages, (2) **configuration reference documentation** for all exposed options, (3) **migration guide** following Keep a Changelog conventions with semantic versioning, (4) **automated migration scripts** using libCST (Python) and jscodeshift (JavaScript), and (5) **CI validation** using markdownlint and markdown-link-check to prevent documentation drift.

The critical insight is that migration tooling must handle **positional-to-keyword argument transformation** (Python) and **string-to-object constructor transformation** (JavaScript/TypeScript). For Python, libCST provides lossless CST parsing that preserves formatting while transforming constructors. For JavaScript, jscodeshift offers AST-based transformation with built-in TypeScript support. Both tools can run as standalone scripts and integrate into CI pipelines.

**Primary recommendation:** Create language-specific codemods as standalone scripts (not requiring package installation), include migration validation in CI, and provide both automated and manual migration paths. Documentation should follow Keep a Changelog v1.1.0 format with ISO 8601 dates and semantic versioning.

## Standard Stack

### Core Documentation Tools
| Tool | Version | Purpose | Why Standard |
|------|---------|---------|--------------|
| mdBook | 0.4.40+ | Long-form guides, multi-page docs | Rust ecosystem standard, used by Rust Programming Language book |
| rustdoc | 1.85+ | API reference from doc comments | Built into Rust toolchain, zero-config |
| Keep a Changelog | v1.1.0 | Changelog format standard | Industry-wide adoption, semantic versioning integration |

### Python Migration Tools
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| libCST | 1.5.0+ | Lossless Python CST parser/transformer | Constructor API migrations, preserves formatting |
| black | 24.10.0+ | Code formatter | Post-transformation formatting |
| pytest | 8.3+ | Test transformed code | Validate migration correctness |

**Why libCST over alternatives:**
- Bowler (built on lib2to3) deprecated in Python 3.12+
- libCST actively maintained by Instagram/Meta, 6.3k+ stars
- Lossless transformation preserves comments, whitespace
- Used at scale: Instagram, Instawork production codemods

### JavaScript Migration Tools
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| jscodeshift | 17.1.0+ | JavaScript AST transformer | Constructor API migrations, TypeScript support |
| @babel/parser | 7.26+ | Parse JavaScript/TypeScript to AST | TypeScript import handling |
| prettier | 3.4.2+ | Code formatter | Post-transformation formatting |

**Why jscodeshift:**
- Facebook standard for React codemod migrations
- Built-in TypeScript support via Babel parser
- 9.3k+ stars, production-proven at Meta scale
- Query API similar to jQuery for AST traversal

### Documentation Validation Tools
| Tool | Version | Purpose | When to Use |
|------|---------|---------|-------------|
| markdownlint-cli | 0.43+ | Markdown style enforcement | CI linting for consistency |
| markdown-link-check | 3.12+ | Dead link detection | CI validation for all markdown files |
| vale | 3.9+ | Prose style checker (optional) | Enforce writing style guide |

**Installation:**
```bash
# Python migration tools
pip install libcst black pytest

# JavaScript migration tools
npm install --save-dev jscodeshift @babel/parser prettier

# Documentation validation
npm install --save-dev markdownlint-cli markdown-link-check
```

## Architecture Patterns

### Documentation Structure
```
docs/
├── README.md                    # Quick start + API overview
├── CHANGELOG.md                 # Keep a Changelog format
├── MIGRATION.md                 # v0.2.x → v0.3.0 guide
├── guides/
│   ├── configuration.md         # All config options reference
│   ├── authentication.md        # Auth method comparison
│   └── error-handling.md        # Error codes + handling patterns
├── examples/
│   ├── python/
│   │   ├── rest_basic.py
│   │   └── websocket_basic.py
│   ├── javascript/
│   │   ├── rest_basic.js
│   │   └── websocket_basic.ts
│   ├── java/
│   │   └── RestExample.java
│   ├── go/
│   │   └── rest_example.go
│   └── csharp/
│       └── RestExample.cs
└── migration/
    ├── migrate-python.py        # libCST codemod
    ├── migrate-javascript.js     # jscodeshift transform
    └── validate-migration.sh    # Post-migration validation
```

### Pattern 1: Keep a Changelog Format
**What:** Industry-standard changelog format with semantic versioning
**When to use:** All SDK/library projects with semantic versioning
**Example:**
```markdown
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] - 2026-02-28

### Added
- Options object constructor pattern for RestClient and WebSocketClient (all languages)
- Configuration exposure: ReconnectConfig and HealthCheckConfig
- Migration scripts for Python (libCST) and JavaScript (jscodeshift)

### Changed
- **BREAKING:** RestClient/WebSocketClient constructors now require options object
- Health check default changed from `true` to `false` (aligned with official SDKs)

### Deprecated
- Python: Positional string constructors (removed in v0.4.0)
- Node.js: Single string constructors (removed in v0.4.0)

### Fixed
- Configuration validation now happens at construction time (not connection time)

## [0.2.0] - 2026-01-31
...
```
**Source:** [Keep a Changelog v1.1.0](https://keepachangelog.com/en/1.1.0/)

### Pattern 2: Before/After Migration Examples
**What:** Side-by-side code comparison showing v0.2.x → v0.3.0 changes
**When to use:** All breaking change documentation
**Example:**
```markdown
### Python REST Client

**Before (v0.2.x):**
```python
from marketdata_py import RestClient

client = RestClient("your-api-key")
quote = client.stock.intraday.quote("2330")
```

**After (v0.3.0):**
```python
from marketdata_py import RestClient

client = RestClient(api_key="your-api-key")
quote = client.stock.intraday.quote("2330")
```

**Migration:** Run automated codemod:
```bash
python migration/migrate-python.py --path src/
```
```

### Pattern 3: Configuration Reference Table
**What:** Comprehensive table documenting all config options
**When to use:** Documenting complex configuration APIs
**Example:**
```markdown
### ReconnectConfig Options

| Option | Type | Default | Range | Description |
|--------|------|---------|-------|-------------|
| `max_attempts` | u32 | 5 | 1-100 | Maximum reconnection attempts before giving up |
| `initial_delay_ms` | u64 | 1000 | 100-30000 | Initial backoff delay in milliseconds |
| `max_delay_ms` | u64 | 60000 | 1000-300000 | Maximum backoff delay (caps exponential growth) |

**Python:**
```python
from marketdata_py import WebSocketClient, ReconnectConfig

config = ReconnectConfig(max_attempts=10, initial_delay_ms=2000)
client = WebSocketClient(api_key="key", reconnect=config)
```

**JavaScript:**
```typescript
import { WebSocketClient } from '@fugle/marketdata-js';

const client = new WebSocketClient({
  apiKey: 'key',
  reconnect: { maxAttempts: 10, initialDelayMs: 2000 }
});
```
```

### Pattern 4: libCST Codemod Transform
**What:** Python constructor transformation using libCST
**When to use:** Positional → keyword argument migrations
**Example:**
```python
# migration/migrate-python.py
import libcst as cst
from libcst import matchers as m

class RestClientMigrator(cst.CSTTransformer):
    def leave_Call(self, original_node, updated_node):
        # Match: RestClient("api-key") or RestClient.with_bearer_token("token")
        if m.matches(updated_node.func, m.Name("RestClient")):
            if len(updated_node.args) == 1:
                arg = updated_node.args[0]
                # Transform to keyword argument
                return updated_node.with_changes(
                    args=[cst.Arg(
                        keyword=cst.Name("api_key"),
                        value=arg.value
                    )]
                )
        return updated_node
```
**Source:** [libCST Codemods Tutorial](https://libcst.readthedocs.io/en/latest/codemods_tutorial.html)

### Pattern 5: jscodeshift Transform
**What:** JavaScript/TypeScript constructor transformation using jscodeshift
**When to use:** String → object constructor migrations
**Example:**
```javascript
// migration/migrate-javascript.js
module.exports = function(fileInfo, api) {
  const j = api.jscodeshift;
  const root = j(fileInfo.source);

  // Find: new RestClient('api-key')
  root.find(j.NewExpression, {
    callee: { name: 'RestClient' },
    arguments: args => args.length === 1 && args[0].type === 'Literal'
  }).replaceWith(path => {
    const apiKey = path.value.arguments[0].value;

    // Replace with: new RestClient({ apiKey: 'api-key' })
    return j.newExpression(
      path.value.callee,
      [j.objectExpression([
        j.property('init', j.identifier('apiKey'), j.literal(apiKey))
      ])]
    );
  });

  return root.toSource();
};
```
**Source:** [jscodeshift README](https://github.com/facebook/jscodeshift)

### Pattern 6: CI Documentation Validation
**What:** GitHub Actions workflow for documentation quality
**When to use:** All projects with Markdown documentation
**Example:**
```yaml
# .github/workflows/docs-validation.yml
name: Documentation Validation

on: [pull_request]

jobs:
  validate-docs:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Lint Markdown
        run: |
          npm install -g markdownlint-cli
          markdownlint '**/*.md' --ignore node_modules

      - name: Check Dead Links
        run: |
          npm install -g markdown-link-check
          find . -name '*.md' -not -path './node_modules/*' \
            -exec markdown-link-check {} \;

      - name: Validate Examples
        run: |
          # Check examples use v0.3.0 API patterns
          ! grep -r "RestClient(\"" examples/ || \
            (echo "ERROR: Examples use deprecated v0.2.x API"; exit 1)
```
**Source:** [markdown-link-check GitHub](https://github.com/tcort/markdown-link-check)

### Anti-Patterns to Avoid
- **Inline examples without before/after:** Users can't see what changed
- **Generic migration instructions:** "Update your code" without specifics
- **Manual-only migration:** No automated tooling for common cases
- **Broken links in docs:** Dead links erode trust and waste user time
- **Inconsistent terminology:** "options" vs "config" vs "settings" confusion
- **Missing version constraints:** "Works with Python 3" (which 3.x versions?)

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Markdown linting | Custom regex checker | markdownlint-cli | 50+ rules, configurable, battle-tested |
| Dead link detection | wget/curl scripts | markdown-link-check | Handles relative links, 429 rate limits, XML reports |
| Python AST transforms | Manual regex replacement | libCST | Preserves formatting, handles edge cases (nested calls, kwargs) |
| JavaScript AST transforms | String manipulation | jscodeshift | TypeScript support, handles imports, production-proven |
| Changelog formatting | Freeform text | Keep a Changelog | Industry standard, tooling support, semantic versioning |
| Code formatting post-codemod | Manual spacing | black (Python), prettier (JS) | Consistent style, integrates with editors |

**Key insight:** Migration tooling is deceptively complex. Manual regex replacement breaks on:
- Multi-line constructor calls
- Comments between arguments
- Existing keyword arguments mixed with positional
- Import alias variations (`from x import y as z`)

CST/AST parsers handle these cases correctly by understanding language semantics, not just text patterns.

## Common Pitfalls

### Pitfall 1: Migration Script Overwrites User Code
**What goes wrong:** Codemod runs on entire project, modifies unrelated code
**Why it happens:** Overly broad pattern matching (e.g., all `Client()` calls)
**How to avoid:**
- Scope transforms to specific import sources
- Add `--dry-run` mode showing diffs before writing
- Validate no unintended changes with pre/post test runs
**Warning signs:**
- Test failures after migration
- Modified files outside expected scope

### Pitfall 2: Documentation Examples Drift from Code
**What goes wrong:** README examples use old API, code uses new API
**Why it happens:** No automated validation that examples are executable
**How to avoid:**
- Extract examples to standalone files
- Run examples in CI as integration tests
- Use doctest (Python) or ts-node (TypeScript) for inline examples
**Warning signs:**
- User issues citing "example doesn't work"
- Copy-paste from docs causes immediate errors

### Pitfall 3: Changelog Has No Structure
**What goes wrong:** Freeform changelog text is hard to parse for tools
**Why it happens:** No format standard, treating changelog as prose
**How to avoid:**
- Follow Keep a Changelog format strictly
- Use semantic versioning for version numbers
- Group changes by type (Added, Changed, Fixed, etc.)
**Warning signs:**
- Can't generate release notes from changelog
- Users can't find breaking changes quickly

### Pitfall 4: Migration Guide Assumes Expert Users
**What goes wrong:** Guide skips basic steps, assumes deep SDK knowledge
**Why it happens:** Author curse of knowledge (familiar with internals)
**How to avoid:**
- Test migration guide with fresh eyes (pair with non-expert)
- Include "Prerequisites" section (Python version, package versions)
- Show complete before/after file contents, not just diffs
**Warning signs:**
- High volume of "migration didn't work" issues
- Users asking for clarification on basic steps

### Pitfall 5: Dead Links in Documentation Go Unnoticed
**What goes wrong:** Links to external docs break over time
**Why it happens:** No CI check for link validity
**How to avoid:**
- Add markdown-link-check to CI
- Configure retry logic for transient failures
- Use relative links for internal docs (less likely to break)
**Warning signs:**
- Users report broken links
- External API docs moved/renamed

### Pitfall 6: Configuration Documentation Missing Validation Rules
**What goes wrong:** Users set invalid config values, get runtime errors
**Why it happens:** Docs show valid examples but don't document constraints
**How to avoid:**
- Document valid ranges for numeric options
- Show error messages users will see for invalid values
- Include "Common Mistakes" section in config docs
**Warning signs:**
- Repeated issues about "ConfigError" at runtime
- Users trial-and-error to find valid values

## Code Examples

Verified patterns from official sources:

### Python libCST Codemod
```python
# migration/migrate-python.py
# Source: https://libcst.readthedocs.io/en/latest/codemods_tutorial.html
import libcst as cst
from libcst.codemod import CodemodContext, VisitorBasedCodemodCommand
from libcst import matchers as m

class MigrateRestClient(VisitorBasedCodemodCommand):
    """
    Migrate RestClient from positional to keyword arguments:
    - RestClient("api-key") → RestClient(api_key="api-key")
    - WebSocketClient("api-key") → WebSocketClient(api_key="api-key")
    """

    DESCRIPTION = "Migrate to v0.3.0 options object API"

    def leave_Call(self, original_node: cst.Call, updated_node: cst.Call) -> cst.Call:
        # Match RestClient or WebSocketClient with single positional arg
        if m.matches(
            updated_node.func,
            m.Name("RestClient") | m.Name("WebSocketClient")
        ):
            if len(updated_node.args) == 1:
                arg = updated_node.args[0]
                if arg.keyword is None:  # Positional argument
                    # Convert to keyword argument
                    new_arg = arg.with_changes(
                        keyword=cst.Name("api_key")
                    )
                    return updated_node.with_changes(args=[new_arg])

        return updated_node

# Usage:
# python -m libcst.tool codemod migrate_python.MigrateRestClient src/
```

### JavaScript jscodeshift Transform
```javascript
// migration/migrate-javascript.js
// Source: https://github.com/facebook/jscodeshift
module.exports = function transformer(fileInfo, api, options) {
  const j = api.jscodeshift;
  const root = j(fileInfo.source);
  let modified = false;

  // Transform: new RestClient('api-key') → new RestClient({ apiKey: 'api-key' })
  root.find(j.NewExpression, {
    callee: {
      type: 'Identifier',
      name: name => ['RestClient', 'WebSocketClient'].includes(name)
    }
  }).forEach(path => {
    const args = path.value.arguments;

    // Only transform single string argument
    if (args.length === 1 && args[0].type === 'Literal') {
      const apiKey = args[0].value;

      path.value.arguments = [
        j.objectExpression([
          j.property('init', j.identifier('apiKey'), j.literal(apiKey))
        ])
      ];

      modified = true;
    }
  });

  return modified ? root.toSource({ quote: 'single' }) : null;
};

// Usage:
// jscodeshift -t migrate-javascript.js src/
```

### Markdown Link Check Configuration
```json
// .markdown-link-check.json
// Source: https://github.com/tcort/markdown-link-check
{
  "ignorePatterns": [
    {
      "pattern": "^http://localhost"
    }
  ],
  "timeout": "10s",
  "retryOn429": true,
  "retryCount": 3,
  "fallbackRetryDelay": "5s",
  "aliveStatusCodes": [200, 206, 302, 307, 308],
  "replacementPatterns": [
    {
      "pattern": "^/",
      "replacement": "https://docs.fugle.tw/"
    }
  ]
}
```

### CI Validation Script
```bash
#!/bin/bash
# migration/validate-migration.sh
# Run after migration to validate correctness

set -e

echo "=== Validating Python Migration ==="
cd py
python -m pytest tests/ -v
python -m mypy marketdata_py --strict
echo "✓ Python validation passed"

echo ""
echo "=== Validating JavaScript Migration ==="
cd ../js
npm run test
npm run typecheck
echo "✓ JavaScript validation passed"

echo ""
echo "=== Validating Examples ==="
# Check no examples use deprecated API
if grep -r 'RestClient("' ../examples/; then
  echo "✗ Found deprecated string constructors in examples"
  exit 1
fi
echo "✓ Examples use v0.3.0 API"

echo ""
echo "=== All Validations Passed ==="
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Manual regex find/replace | libCST/jscodeshift codemods | 2019-2020 | Lossless transforms, handles edge cases |
| Freeform changelog text | Keep a Changelog format | 2017 standard | Machine-parseable, tooling integration |
| Manual link checking | markdown-link-check CI | 2020+ | Automated validation, prevents drift |
| Inline code examples | Extracted testable examples | 2021+ | Executable docs, always up-to-date |
| Markdown only | mdBook for guides | 2023+ | Multi-page navigation, search, themes |

**Deprecated/outdated:**
- **Bowler (Python codemod):** Built on lib2to3 which is deprecated in Python 3.12+, replaced by libCST
- **Manual regex migrations:** Error-prone, doesn't preserve formatting, replaced by AST/CST tools
- **Unversioned documentation:** "Latest" docs with no version tags, replaced by versioned docs with changelog

## Open Questions

1. **Should migration scripts be in-repo or distributed via PyPI/npm?**
   - What we know: In-repo scripts are zero-install, but harder to version separately
   - What's unclear: Best practice for SDK migration script distribution
   - Recommendation: Start in-repo (`migration/` directory), can publish later if demand exists

2. **How long should we maintain v0.2.x compatibility (deprecated constructors)?**
   - What we know: Python/Node.js have deprecated constructors until v0.4.0
   - What's unclear: Industry standard deprecation lifecycle for SDKs
   - Recommendation: 2 major versions (keep in v0.3.x, remove in v0.4.0) with clear warnings

3. **Should examples be in-repo or separate repository?**
   - What we know: In-repo examples can be tested in CI, out-of-repo allows independent versioning
   - What's unclear: Maintenance burden vs. discoverability tradeoff
   - Recommendation: In-repo for basic examples, consider separate repo if examples grow >10 files per language

4. **Do we need mdBook for long-form documentation or is README sufficient?**
   - What we know: Current project has only README per language
   - What's unclear: User demand for structured guides (authentication, error handling, etc.)
   - Recommendation: Start with enhanced README + MIGRATION.md, defer mdBook until user feedback indicates need

## Sources

### Primary (HIGH confidence)
- [Keep a Changelog v1.1.0](https://keepachangelog.com/en/1.1.0/) - Changelog format standard
- [libCST Documentation](https://libcst.readthedocs.io/) - Python CST parser and codemod framework
- [jscodeshift GitHub](https://github.com/facebook/jscodeshift) - JavaScript AST transformation toolkit
- [markdown-link-check GitHub](https://github.com/tcort/markdown-link-check) - Dead link detection for CI
- [markdownlint GitHub](https://github.com/markdownlint/markdownlint) - Markdown linting tool

### Secondary (MEDIUM confidence)
- [Refactoring Python with LibCST - ChairNerd](https://chairnerd.seatgeek.com/refactoring-python-with-libcst/) - Production codemod patterns
- [Toptal: Refactoring With Codemods and jscodeshift](https://www.toptal.com/javascript/write-code-to-rewrite-your-code) - JavaScript migration guide
- [AI SDK 6.0 Migration Guide](https://ai-sdk.dev/docs/migration-guides/migration-guide-6-0) - Modern SDK migration example
- [MegaLinter markdown-link-check](https://megalinter.io/v5/descriptors/markdown_markdown_link_check/) - CI integration patterns

### Tertiary (LOW confidence, for reference)
- [mdBook Documentation](https://rust-lang.github.io/mdBook/) - Long-form documentation tool
- [rustdoc Book](https://doc.rust-lang.org/rustdoc/) - API documentation from comments
- [Common Changelog](https://common-changelog.org/) - Alternative changelog format

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - libCST and jscodeshift are industry standards with extensive usage
- Architecture: HIGH - Keep a Changelog is widely adopted, migration patterns verified from production examples
- Pitfalls: MEDIUM-HIGH - Derived from SDK migration best practices, validated against recent 2026 migrations

**Research date:** 2026-02-15
**Valid until:** 60 days (documentation tools are stable, format standards change slowly)

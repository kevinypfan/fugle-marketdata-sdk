# Phase 5: Cross-Platform Distribution - Research

**Researched:** 2026-01-31
**Domain:** Multi-language package distribution (PyPI, npm, NuGet, Maven Central)
**Confidence:** HIGH

## Summary

Cross-platform distribution for Rust-based bindings requires coordinated publishing to five package registries (PyPI, npm, NuGet, Maven Central, and Go module hosting) with platform-specific native binaries. The standard approach uses:

1. **GitHub Actions matrix builds** for cross-compilation across Linux (x86_64, aarch64), macOS (universal2), and Windows (x86_64)
2. **Trusted Publishing** (OIDC-based authentication) eliminating long-lived API tokens for PyPI, npm, and NuGet
3. **Language-specific tooling**: maturin (Python wheels), napi-rs (Node.js native addons), dotnet pack (NuGet), Gradle (Maven)
4. **Workspace version synchronization** from Cargo.toml (0.2.0) to all language packages
5. **Coordinated release workflow** triggered by git tags, publishing all packages with matching versions

The existing project structure already includes partial publishing infrastructure (napi targets in package.json, pyproject.toml configuration, .csproj with runtimes paths). This phase extends it to complete automated multi-registry publishing.

**Primary recommendation:** Use PyO3/maturin-action, napi-rs prebuild, and trusted publishing to eliminate manual token management while achieving full platform coverage.

## Standard Stack

The established libraries/tools for multi-language Rust binding distribution:

### Core Build Tools
| Tool | Version | Purpose | Why Standard |
|------|---------|---------|--------------|
| maturin | 1.11.5+ | Python wheel builder | PyO3 ecosystem standard, manylinux compliance checks built-in |
| @napi-rs/cli | 3.5.1+ | Node.js native addon packager | Automatic platform detection, optionalDependencies pattern |
| dotnet CLI | 8.0+ | NuGet package creation | Microsoft official tooling, runtimes/{rid}/native support |
| Gradle | 8.5+ | Java JAR publishing | Maven Central ecosystem standard, native resource bundling |
| cargo | 1.87+ | Go module building | Cross-compilation for shared libraries |

### GitHub Actions
| Action | Version | Purpose | When to Use |
|--------|---------|---------|-------------|
| PyO3/maturin-action | v1 | Build manylinux/musllinux wheels | All Python wheel builds |
| actions/setup-python | v5 | Python environment setup | Publishing to PyPI |
| actions/setup-node | v4 | Node.js environment setup | Publishing to npm |
| actions/setup-dotnet | v4 | .NET SDK setup | Publishing to NuGet |
| actions/setup-java | v4 | JDK setup | Publishing to Maven Central |
| pypa/gh-action-pypi-publish | v1.12.2+ | Trusted publishing to PyPI | PyPI upload (replaces twine) |

### Platform Matrix Standards
| Platform | Targets | Standard Tags |
|----------|---------|---------------|
| Linux (glibc) | x86_64, aarch64 | manylinux_2_17, manylinux_2_28 |
| Linux (musl) | x86_64, aarch64 | musllinux_1_2 (Alpine support) |
| macOS | universal2 (Intel + ARM) | macosx_11_0_universal2 |
| Windows | x86_64 | win_amd64 |

**Installation:**
```bash
# Python tooling
pip install maturin>=1.11

# Node.js tooling (already in js/package.json)
npm install -D @napi-rs/cli@^3.5.1

# .NET tooling (system-wide)
# Install .NET 8 SDK from https://dotnet.microsoft.com/download

# Java tooling (Gradle wrapper already configured)
./gradlew --version
```

## Architecture Patterns

### Recommended Release Workflow Structure
```
.github/workflows/
├── release.yml              # Main coordinated release workflow
├── build-python.yml         # Python wheel builds (matrix)
├── build-nodejs.yml         # Node.js native addon builds (matrix)
├── build-uniffi.yml         # UniFFI library builds for C#/Go/Java
├── publish-python.yml       # PyPI trusted publishing
├── publish-nodejs.yml       # npm trusted publishing
├── publish-nuget.yml        # NuGet trusted publishing
└── publish-maven.yml        # Maven Central publishing
```

### Pattern 1: Git Tag-Triggered Release
**What:** Single git tag (e.g., `v0.2.0`) triggers coordinated multi-language release
**When to use:** Manual release approval with synchronized versions
**Example:**
```yaml
# .github/workflows/release.yml
name: Release
on:
  push:
    tags:
      - 'v*.*.*'

jobs:
  extract-version:
    runs-on: ubuntu-latest
    outputs:
      version: ${{ steps.version.outputs.version }}
    steps:
      - id: version
        run: echo "version=${GITHUB_REF#refs/tags/v}" >> $GITHUB_OUTPUT

  build-python:
    needs: extract-version
    uses: ./.github/workflows/build-python.yml
    with:
      version: ${{ needs.extract-version.outputs.version }}

  publish-python:
    needs: build-python
    uses: ./.github/workflows/publish-python.yml
    permissions:
      id-token: write  # Trusted publishing
```

### Pattern 2: Platform Matrix with Artifacts
**What:** Build wheels/addons across platforms, share as artifacts, publish once
**When to use:** All package types (Python, Node.js, C#, Java)
**Example:**
```yaml
# .github/workflows/build-python.yml
jobs:
  build-wheels:
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64
            manylinux: '2_17'
          - os: ubuntu-latest
            target: aarch64
            manylinux: '2_17'
          - os: macos-latest
            target: universal2
          - os: windows-latest
            target: x64
    runs-on: ${{ matrix.os }}
    steps:
      - uses: PyO3/maturin-action@v1
        with:
          target: ${{ matrix.target }}
          manylinux: ${{ matrix.manylinux || 'auto' }}
          args: --release -m py/Cargo.toml
      - uses: actions/upload-artifact@v4
        with:
          name: wheels-${{ matrix.os }}-${{ matrix.target }}
          path: target/wheels/*.whl
```

### Pattern 3: Trusted Publishing (OIDC)
**What:** OpenID Connect tokens replace long-lived API keys
**When to use:** PyPI, npm, NuGet (Maven Central pending)
**Example:**
```yaml
# .github/workflows/publish-python.yml
jobs:
  publish:
    runs-on: ubuntu-latest
    permissions:
      id-token: write  # CRITICAL: Required for trusted publishing
    steps:
      - uses: actions/download-artifact@v4
        with:
          pattern: wheels-*
          merge-multiple: true
      - uses: pypa/gh-action-pypi-publish@v1.12.2
        with:
          packages-dir: .
          # No token required - OIDC handles authentication
```

### Pattern 4: napi-rs Platform-Specific Packages
**What:** Separate npm packages per platform with optionalDependencies
**When to use:** Node.js native addons (existing js/package.json follows this)
**Example:**
```json
// Main package.json
{
  "name": "@fugle/marketdata",
  "optionalDependencies": {
    "@fugle/marketdata-darwin-x64": "0.2.0",
    "@fugle/marketdata-darwin-arm64": "0.2.0",
    "@fugle/marketdata-linux-x64-gnu": "0.2.0",
    "@fugle/marketdata-linux-arm64-gnu": "0.2.0",
    "@fugle/marketdata-win32-x64-msvc": "0.2.0"
  }
}
```

### Pattern 5: NuGet runtimes/{rid}/native Convention
**What:** Platform-specific native libraries in standardized NuGet paths
**When to use:** .NET NuGet packages with P/Invoke dependencies
**Example:**
```xml
<!-- MarketdataUniffi.csproj - already follows this pattern -->
<ItemGroup>
  <None Include="libmarketdata_uniffi.so"
        PackagePath="runtimes/linux-x64/native/" />
  <None Include="libmarketdata_uniffi.dylib"
        PackagePath="runtimes/osx-arm64/native/" />
  <None Include="marketdata_uniffi.dll"
        PackagePath="runtimes/win-x64/native/" />
</ItemGroup>
```

### Anti-Patterns to Avoid
- **Manual token rotation in GitHub Secrets**: Use trusted publishing instead (PyPI/npm/NuGet all support OIDC now)
- **Single-platform builds**: Users expect pre-built binaries for their platform; requiring Rust toolchain is a dealbreaker
- **Separate version numbers**: Version drift between languages creates confusion; synchronize from Cargo.toml
- **Publishing without smoke tests**: At minimum, verify wheel/package installs before publishing
- **Ignoring manylinux compliance**: PyPI rejects non-compliant wheels; use maturin's built-in checks

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Cross-compilation to manylinux | Custom Docker images | PyO3/maturin-action with `manylinux: auto` | Handles glibc versioning, auditwheel checks, platform tags |
| Node.js platform detection | Custom loader logic | @napi-rs/cli `prepublish` script | Generates optionalDependencies, handles all platform combos |
| NuGet multi-targeting | Manual .targets files | `<TargetFrameworks>` with runtimes/{rid}/native | .NET SDK copies correct native lib automatically |
| Version synchronization | Sed scripts to update versions | Workspace version inheritance + single source of truth | Cargo.toml workspace.version cascades to all crates |
| PyPI authentication | Manually managing PyPI tokens | Trusted publishing with `id-token: write` | Tokens auto-expire, no secret rotation needed |
| Maven Central signing | Manual GPG key management | Gradle signing plugin with env vars | Handles key import, signing, and upload in one step |

**Key insight:** Rust FFI distribution is mature in 2026; every binding type has official tooling that handles platform detection, packaging conventions, and registry authentication. Custom scripts introduce failure modes that official tools already solved.

## Common Pitfalls

### Pitfall 1: macOS Universal2 Build Failures
**What goes wrong:** Cross-compiling for macOS from Linux fails with "CoreFoundation framework not found"
**Why it happens:** macOS frameworks aren't available on Linux; requires macOS runner or zig linker workaround
**How to avoid:** Use `runs-on: macos-latest` for macOS targets, or maturin's zig linker support (experimental)
**Warning signs:** Build errors mentioning `framework 'CoreFoundation'` or `x86_64-apple-darwin` link failures

### Pitfall 2: manylinux Glibc Version Mismatches
**What goes wrong:** Wheels build successfully but fail at runtime with glibc version errors
**Why it happens:** Rust 1.64+ requires glibc 2.17 minimum (manylinux2014); older images produce broken wheels
**How to avoid:** Use `manylinux: '2_17'` or newer; maturin auto-detects compliance and tags correctly
**Warning signs:** ImportError at runtime mentioning GLIBC version requirements

### Pitfall 3: napi-rs optionalDependencies Not Auto-Installing
**What goes wrong:** Users install main package but get "native addon not found" errors
**Why it happens:** npm/yarn/pnpm didn't download platform-specific packages from optionalDependencies
**How to avoid:** Use `napi prepublish -t npm` to generate correct package.json; test with `npm pack` before publishing
**Warning signs:** Main package has empty optionalDependencies or incorrect platform package names

### Pitfall 4: NuGet Native Library Not Copied to Output
**What goes wrong:** NuGet package installs but throws DllNotFoundException at runtime
**Why it happens:** Native libraries in runtimes/{rid}/native need .targets file to copy to output directory
**How to avoid:** .NET SDK 6.0+ auto-copies from runtimes/{rid}/native; verify with `dotnet publish --no-self-contained`
**Warning signs:** Library exists in NuGet cache but not in bin/Debug or bin/Release after build

### Pitfall 5: Version Desynchronization Across Languages
**What goes wrong:** Python package is 0.2.0 but npm package is 0.1.9; users confused about compatibility
**Why it happens:** Forgetting to update version in package.json/build.gradle when bumping Cargo.toml
**How to avoid:** Single source of truth (Cargo.toml workspace.version), CI checks verify all match before publishing
**Warning signs:** Git tag is v0.2.0 but published npm package shows different version

### Pitfall 6: Trusted Publishing Not Configured in Registry
**What goes wrong:** GitHub Actions has `id-token: write` but publish fails with authentication error
**Why it happens:** Registry (PyPI/npm/NuGet) doesn't have GitHub Actions configured as trusted publisher
**How to avoid:** Pre-configure trusted publishers in registry settings BEFORE first automated publish
**Warning signs:** Error messages like "OIDC token validation failed" or "publisher not configured"

### Pitfall 7: Maven Central Staging Without Release
**What goes wrong:** Artifacts uploaded to Maven Central but never appear in search/downloads
**Why it happens:** Maven Central requires explicit "close and release" step after upload to staging repository
**How to avoid:** Use Gradle maven-publish plugin with `closeAndReleaseSonatypeStagingRepository` task
**Warning signs:** Artifacts visible in Nexus Repository Manager but not at search.maven.org

## Code Examples

Verified patterns from official sources and current project structure:

### Python: Maturin Build with Platform Matrix
```yaml
# Source: PyO3/maturin-action README
# https://github.com/PyO3/maturin-action
name: Build Python Wheels

jobs:
  build:
    strategy:
      matrix:
        include:
          # Linux x86_64
          - os: ubuntu-latest
            target: x86_64
            manylinux: '2_17'
          # Linux aarch64 (ARM64)
          - os: ubuntu-latest
            target: aarch64
            manylinux: '2_17'
          # macOS universal2 (Intel + Apple Silicon)
          - os: macos-latest
            target: universal2
          # Windows x64
          - os: windows-latest
            target: x64

    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: PyO3/maturin-action@v1
        with:
          target: ${{ matrix.target }}
          manylinux: ${{ matrix.manylinux || 'auto' }}
          args: --release -m py/Cargo.toml --out dist
      - uses: actions/upload-artifact@v4
        with:
          name: wheels-${{ matrix.os }}-${{ matrix.target }}
          path: dist/*.whl
```

### Python: Trusted Publishing to PyPI
```yaml
# Source: PyPA gh-action-pypi-publish documentation
# https://github.com/pypa/gh-action-pypi-publish
name: Publish to PyPI

jobs:
  publish:
    runs-on: ubuntu-latest
    permissions:
      id-token: write  # CRITICAL: Required for OIDC
    steps:
      - uses: actions/download-artifact@v4
        with:
          pattern: wheels-*
          merge-multiple: true
          path: dist/

      - uses: pypa/gh-action-pypi-publish@v1.12.2
        with:
          packages-dir: dist/
          # No password or token needed - OIDC authentication
```

### Node.js: napi-rs Build and Publish
```yaml
# Source: napi-rs package-template
# https://github.com/napi-rs/package-template
name: Build Node.js Native Addon

jobs:
  build:
    strategy:
      matrix:
        settings:
          - host: macos-latest
            target: x86_64-apple-darwin
          - host: macos-latest
            target: aarch64-apple-darwin
          - host: ubuntu-latest
            target: x86_64-unknown-linux-gnu
          - host: ubuntu-latest
            target: aarch64-unknown-linux-gnu
          - host: windows-latest
            target: x86_64-pc-windows-msvc

    runs-on: ${{ matrix.settings.host }}
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 20

      - name: Build native addon
        working-directory: js
        run: npm run build -- --target ${{ matrix.settings.target }}

      - uses: actions/upload-artifact@v4
        with:
          name: bindings-${{ matrix.settings.target }}
          path: js/*.node
```

### Node.js: npm Trusted Publishing
```yaml
# Source: npm Trusted Publishers documentation
# https://docs.npmjs.com/trusted-publishers/
name: Publish to npm

jobs:
  publish:
    runs-on: ubuntu-latest
    permissions:
      id-token: write  # Required for npm provenance
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 20
          registry-url: 'https://registry.npmjs.org'

      - name: Publish with provenance
        working-directory: js
        run: npm publish --provenance --access public
        env:
          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}
```

### C#: NuGet Pack with Native Libraries
```yaml
# Source: Microsoft NuGet documentation
# https://learn.microsoft.com/en-us/nuget/create-packages/supporting-multiple-target-frameworks
name: Build and Publish NuGet

jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-dotnet@v4
        with:
          dotnet-version: '8.0'

      # Build native library for all platforms first
      - name: Build UniFFI library (Linux)
        run: cargo build --release -p marketdata-uniffi

      - name: Pack NuGet package
        working-directory: bindings/csharp/MarketdataUniffi
        run: dotnet pack -c Release -o nupkg

      - name: Publish to NuGet
        run: dotnet nuget push bindings/csharp/MarketdataUniffi/nupkg/*.nupkg --api-key ${{ secrets.NUGET_API_KEY }} --source https://api.nuget.org/v3/index.json
```

### Java: Gradle Publish to Maven Central
```kotlin
// Source: Gradle Maven Publish Plugin documentation
// https://docs.gradle.org/current/userguide/publishing_maven.html
plugins {
    `java-library`
    `maven-publish`
    signing
}

publishing {
    publications {
        create<MavenPublication>("mavenJava") {
            from(components["java"])

            groupId = "tw.com.fugle"
            artifactId = "marketdata-java"
            version = "0.2.0"

            pom {
                name.set("Fugle Market Data SDK")
                description.set("Java bindings for Fugle Market Data")
                url.set("https://github.com/fugle/fugle-marketdata-sdk")

                licenses {
                    license {
                        name.set("MIT License")
                        url.set("https://opensource.org/licenses/MIT")
                    }
                }
            }
        }
    }

    repositories {
        maven {
            name = "OSSRH"
            url = uri("https://s01.oss.sonatype.org/service/local/staging/deploy/maven2/")
            credentials {
                username = System.getenv("MAVEN_USERNAME")
                password = System.getenv("MAVEN_PASSWORD")
            }
        }
    }
}

signing {
    val signingKey = System.getenv("GPG_SIGNING_KEY")
    val signingPassword = System.getenv("GPG_SIGNING_PASSWORD")
    useInMemoryPgpKeys(signingKey, signingPassword)
    sign(publishing.publications["mavenJava"])
}
```

### Version Synchronization Check
```yaml
# CI check to ensure all versions match Cargo.toml workspace version
name: Version Sync Check

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Extract workspace version
        id: workspace
        run: |
          VERSION=$(grep -A1 '\[workspace.package\]' Cargo.toml | grep 'version' | cut -d'"' -f2)
          echo "version=$VERSION" >> $GITHUB_OUTPUT

      - name: Check Python version
        run: |
          PY_VERSION=$(grep '^version' py/pyproject.toml | cut -d'"' -f2)
          if [ "$PY_VERSION" != "${{ steps.workspace.outputs.version }}" ]; then
            echo "Python version $PY_VERSION != workspace ${{ steps.workspace.outputs.version }}"
            exit 1
          fi

      - name: Check Node.js version
        run: |
          JS_VERSION=$(node -p "require('./js/package.json').version")
          if [ "$JS_VERSION" != "${{ steps.workspace.outputs.version }}" ]; then
            echo "Node.js version $JS_VERSION != workspace ${{ steps.workspace.outputs.version }}"
            exit 1
          fi

      - name: Check C# version
        run: |
          CS_VERSION=$(grep '<Version>' bindings/csharp/MarketdataUniffi/MarketdataUniffi.csproj | sed 's/.*<Version>\(.*\)<\/Version>.*/\1/')
          if [ "$CS_VERSION" != "${{ steps.workspace.outputs.version }}" ]; then
            echo "C# version $CS_VERSION != workspace ${{ steps.workspace.outputs.version }}"
            exit 1
          fi
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| PyPI API tokens in secrets | Trusted publishing (OIDC) | PyPI: Apr 2023, npm: Oct 2024, NuGet: Sep 2025 | Eliminates token rotation, auto-expiring credentials |
| cibuildwheel for Python | maturin direct support | Maturin 0.12+ (2022) | Native PyO3 integration, no wrapper tool needed |
| node-gyp compilation | napi-rs prebuilds | N-API stabilized Node 10+ (2018) | Users get pre-built binaries, no compiler required |
| Manual .targets for NuGet native libs | runtimes/{rid}/native convention | .NET Core 2.0+ (2017) | SDK auto-copies native libs, no custom MSBuild logic |
| OSSRH for Maven Central | Central Portal API | Maven Central redesign (2024-2025) | Simpler publishing, no manual staging UI interaction |
| manylinux2010 | manylinux2014/manylinux_2_17 | Rust 1.64+ requires glibc 2.17 (2022) | Minimum glibc version increased |

**Deprecated/outdated:**
- **pyo3-asyncio**: Replaced by pyo3-async-runtimes (deprecated in PyO3 0.17)
- **node-pre-gyp**: Superseded by napi-rs ecosystem for Rust addons
- **Manual auditwheel**: Maturin includes auditwheel functionality built-in
- **Long-lived PyPI tokens**: Trusted publishing is now recommended best practice
- **manylinux1 (glibc 2.5)**: No longer viable with modern Rust compiler

## Open Questions

Things that couldn't be fully resolved:

1. **macOS Code Signing for Distribution**
   - What we know: unsigned dylibs work for development, may trigger Gatekeeper warnings for users
   - What's unclear: whether notarization is required for PyPI/npm distribution, cost of Apple Developer account
   - Recommendation: Start without signing; add if user feedback indicates Gatekeeper issues

2. **musllinux (Alpine Linux) Demand**
   - What we know: maturin supports musllinux_1_2 builds for Alpine-based containers
   - What's unclear: whether Fugle SDK users actually deploy on Alpine (most use Debian/Ubuntu)
   - Recommendation: Start with manylinux only; add musllinux if requested

3. **ARM64 Windows Support**
   - What we know: Windows ARM64 is growing (Surface Pro X, etc.), Rust supports aarch64-pc-windows-msvc
   - What's unclear: Rust cross-compilation story for Windows ARM from GitHub Actions runners
   - Recommendation: Start with x86_64 only; add ARM64 when GitHub provides native ARM runners

4. **Maven Central vs GitHub Packages for Java**
   - What we know: Maven Central is standard but requires manual account setup; GitHub Packages is simpler
   - What's unclear: User expectations for Java SDK distribution channel
   - Recommendation: Start with GitHub Packages for simplicity; migrate to Maven Central if users request

5. **Go Module Hosting Strategy**
   - What we know: Go doesn't have a central registry like PyPI/npm; uses git tags + proxy.golang.org
   - What's unclear: best practice for distributing pre-compiled shared libraries (.so/.dylib/.dll) with Go modules
   - Recommendation: Publish shared libraries as GitHub Release assets, document manual download in README

## Sources

### Primary (HIGH confidence)
- [PyO3/maturin-action](https://github.com/PyO3/maturin-action) - Official GitHub Action for Python wheels
- [Maturin User Guide: Distribution](https://www.maturin.rs/distribution.html) - Official documentation for wheel building
- [napi-rs: Release native packages](https://napi.rs/docs/deep-dive/release) - Official documentation for Node.js native addon publishing
- [napi-rs package-template](https://github.com/napi-rs/package-template) - Reference implementation for npm publishing
- [PyPA gh-action-pypi-publish](https://github.com/pypa/gh-action-pypi-publish) - Official PyPI trusted publishing action
- [npm Trusted Publishers](https://docs.npmjs.com/trusted-publishers/) - npm OIDC authentication documentation
- [NuGet Trusted Publishing](https://learn.microsoft.com/en-us/nuget/nuget-org/trusted-publishing) - NuGet.org OIDC configuration
- [Gradle Maven Publish Plugin](https://docs.gradle.org/current/userguide/publishing_maven.html) - Official Gradle publishing documentation

### Secondary (MEDIUM confidence)
- [Publishing NuGet packages from GitHub actions with Trusted Publishing](https://andrewlock.net/easily-publishing-nuget-packages-from-github-actions-with-trusted-publishing/) - Practical NuGet OIDC guide
- [vanniktech/gradle-maven-publish-plugin](https://github.com/vanniktech/gradle-maven-publish-plugin) - Community Gradle plugin for Maven Central
- [UniFFI cross-platform distribution](https://github.com/mozilla/uniffi-rs) - UniFFI shared library patterns

### Tertiary (LOW confidence - WebSearch only)
- [semantic-release](https://github.com/semantic-release/semantic-release) - Automated versioning tool (mentioned for reference)
- [Release Please](https://discourse.julialang.org/t/release-please-a-tool-for-multi-package-multi-language-releases/116450) - Multi-language release automation (not verified for Rust)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - All tools verified from official sources (maturin, napi-rs, dotnet, Gradle)
- Architecture: HIGH - GitHub Actions patterns verified with official action documentation
- Pitfalls: MEDIUM - Derived from issue trackers and community discussions, not all personally reproduced
- Code examples: HIGH - All examples sourced from official documentation or reference implementations

**Research date:** 2026-01-31
**Valid until:** 2026-03-31 (60 days - distribution tooling is relatively stable)

**Notes:**
- Existing project already has partial infrastructure (js/package.json with napi targets, py/pyproject.toml)
- Trusted publishing adoption is accelerating (npm added Oct 2024, NuGet added Sep 2025)
- Maven Central is transitioning away from OSSRH; new projects should use Central Portal
- macOS universal2 builds require macOS runners; Linux/Windows cross-compilation won't work

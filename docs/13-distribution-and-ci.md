# 13 — Distribution & CI

`Status: Draft` · `Owner: Release Engineering` · `Last-updated: 2026-07-11` ·
`Related: [../delivery/CONVENTIONS.md](../delivery/CONVENTIONS.md), [01-tech-stack.md](01-tech-stack.md), [02-architecture.md](02-architecture.md), [12-testing-strategy.md](12-testing-strategy.md), [14-security.md](14-security.md), [ROADMAP](ROADMAP.md)`

Implements **R9** (open-source; tarball primary + Tauri bundles for Linux/Windows/
macOS), **R10** (single self-contained binary / portable, runnable without install),
and **R11** (GitHub Actions builds all targets on each tag and publishes a release).
Delivered in **P7** ([ROADMAP](ROADMAP.md#p7--packaging-distribution--ci-release-9-ed)).
The YAML skeletons below are the contract the scaffolding agent implements under
`.github/workflows/` — **job names and matrix entries must match**.

---

## 1. Artifact matrix

| Artifact | Platforms | Contents / notes |
| :--- | :--- | :--- |
| **Portable tarball (PRIMARY)** | Linux, macOS: `.tar.gz`; Windows: `.zip` | `git-purge` binary + `LICENSE` + `LICENSE-APACHE` + `README.md` + `install.sh`/`install.ps1` helper (wraps `git-purge install-cli`). Zero-setup, no runtime deps. |
| Tauri `.deb` | Debian, Ubuntu (and derivatives) | Covers the APT world. |
| Tauri `.rpm` | RedHat, Fedora (and derivatives) | Covers the RPM world. |
| Tauri `.AppImage` | **Any Linux** (Arch, Manjaro, others) | **Universal Linux** fallback — a self-contained, distro-agnostic bundle. Between `.deb`/`.rpm`/`.AppImage`, every named distro (Debian/Ubuntu → deb, RedHat/Fedora → rpm, Arch/Manjaro/anything-else → AppImage) is covered. |
| Tauri `.msi` | Windows | WiX installer (per-machine/per-user). |
| Tauri NSIS `.exe` | Windows | NSIS installer; friendlier UX, bootstraps WebView2. |
| Tauri `.dmg` + `.app` | macOS | `.app` bundle inside a `.dmg` disk image. |

The **tarball is the primary distribution** per [CONVENTIONS §14](../delivery/CONVENTIONS.md):
it ships the CLI only, is the smallest thing that satisfies R10, and needs nothing
installed. The Tauri bundles ship the desktop UI (which embeds `gitpurge-core`, so the
UI works with no CLI installed — [architecture §7](02-architecture.md#7-desktop-process-model)).

### 1.1 Tarball layout

```
git-purge-1.0.0-x86_64-unknown-linux-musl/
├── git-purge              # the single self-contained binary
├── install.sh             # convenience: exec ./git-purge install-cli "$@"
├── LICENSE
├── LICENSE-APACHE
└── README.md
```

`install-cli` is a **built-in subcommand** of `git-purge` (CONVENTIONS §9), so the
tarball needs no separate installer program; `install.sh`/`install.ps1` are thin
wrappers for discoverability.

## 2. Build targets & triples

| Target triple | Produces | Notes |
| :--- | :--- | :--- |
| `x86_64-unknown-linux-gnu` | Tauri bundles (deb/rpm/AppImage), CLI | glibc — required by WebKitGTK for the desktop app. |
| `aarch64-unknown-linux-gnu` | Tauri bundles, CLI | ARM64 Linux. |
| `x86_64-unknown-linux-musl` | **CLI tarball (fully static)** | musl → no glibc dependency; single portable binary. |
| `aarch64-unknown-linux-musl` | **CLI tarball (fully static)** | ARM64 static CLI. |
| `x86_64-pc-windows-msvc` | CLI `.zip`, Tauri `.msi`/NSIS | MSVC toolchain; WebView2 for the app. |
| `aarch64-pc-windows-msvc` | CLI `.zip`, Tauri (best-effort) | ARM64 Windows; CLI always, bundles where runners allow. |
| `x86_64-apple-darwin` | CLI tarball, `.dmg`/`.app` | Intel macOS. |
| `aarch64-apple-darwin` | CLI tarball, `.dmg`/`.app` | Apple Silicon. |

### 2.1 Why the CLI is a single self-contained binary — and the app isn't

- **CLI:** Rust links statically by default (no interpreter/runtime), and per
  [ADR-0002](adr/) reads use **`gix` (pure Rust)** while pushes use `git2`. On Linux we
  build the tarball against **musl** (`*-unknown-linux-musl`) so libc is statically
  linked too → a **fully static** binary that runs on any Linux with **no system `git`,
  no glibc version coupling, nothing installed** (the whole point of R10, and the fix
  for the legacy scripts' bash+python+git prerequisites). On Windows/macOS the CLI is
  self-contained against the platform's stable system libc. If a build config needs
  libgit2's C code, it is **vendored/statically linked** (git2's bundled build), never
  a dynamic system dependency.
- **Desktop app:** Tauri deliberately **does not bundle a browser** — it uses the OS
  webview. So the `.app`/bundle depends on the platform webview at runtime:
  **WebView2 on Windows** (the NSIS/`.msi` installer bootstraps/installs the WebView2
  runtime if absent) and **WebKitGTK on Linux** (a system dependency the `.deb`/`.rpm`
  declare; the `.AppImage` bundles what it can). macOS uses the always-present system
  WKWebView. This is the accepted trade for tiny bundles ([01-tech-stack.md](01-tech-stack.md)).

## 3. `install-cli --user | --system`

`git-purge install-cli` places the binary on `PATH` (and thereby enables the
`git purge` subcommand form). Logic lives in `gitpurge-core` so the CLI and the UI
share it. Dry-run first: it prints the exact target path and PATH change and asks for
confirmation unless `--yes`.

| OS | `--user` (default, no privileges) | `--system` (needs admin/sudo) |
| :--- | :--- | :--- |
| Linux / macOS | Copy (or symlink with `--symlink`) to `~/.local/bin/git-purge`; if that dir isn't on `PATH`, append an export line to the user's shell profile and print the manual step. | Copy to `/usr/local/bin/git-purge` (requires `sudo`). |
| Windows | Copy to `%LOCALAPPDATA%\Programs\git-purge\git-purge.exe`; add that dir to the **user** `PATH` (via registry/`setx`), no elevation. | Copy to `%ProgramFiles%\git-purge\`; add to the **machine** `PATH` (elevation required). |

- Copy is the default (portable, survives moving the tarball away); `--symlink`
  (Unix) points at the extracted binary instead.
- Idempotent: re-running updates in place; `install-cli --uninstall` reverses it.
- **The Tauri app offers it:** a Settings action calls the same core `install_cli`
  logic via an IPC command and shows the resulting path + any PATH note — so UI users
  get a CLI without touching a terminal, satisfying the "UI can set up the CLI"
  convenience (not a dependency; the two are independent per
  [architecture §7](02-architecture.md#7-desktop-process-model)).

## 4. GitHub Actions overview

Two workflows under `.github/workflows/`:

- **`ci.yml`** — runs on every push and PR: fmt, clippy (`-D warnings`), nextest,
  cargo-deny, and frontend lint/typecheck/test. Fast, no artifacts.
- **`release.yml`** — runs on tag `v*`: matrix-builds all targets, bundles via
  `tauri-action`, packages tarballs, generates SHA-256 checksums, signs, and creates
  the GitHub Release with everything attached.

## 5. `ci.yml` (continuous integration)

```yaml
name: ci
on:
  push: { branches: [main] }
  pull_request:

concurrency:
  group: ci-${{ github.ref }}
  cancel-in-progress: true

env:
  CARGO_TERM_COLOR: always
  RUSTFLAGS: "-D warnings"

jobs:
  lint:                          # fmt + clippy, single fast runner
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable        # honors rust-toolchain.toml (MSRV 1.82)
        with: { components: rustfmt, clippy }
      - uses: Swatinem/rust-cache@v2
      - run: cargo fmt --all -- --check
      - run: cargo clippy --all-targets --all-features -- -D warnings

  test:                          # unit + feature + CLI, across OSes
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - uses: taiki-e/install-action@v2
        with: { tool: cargo-nextest,cargo-llvm-cov }
      - run: cargo nextest run --all
      - name: Coverage (core ≥ 80%)          # gate from CONVENTIONS §13
        if: matrix.os == 'ubuntu-latest'
        run: cargo llvm-cov nextest -p gitpurge-core --fail-under-lines 80 --lcov --output-path lcov.info

  deny:                          # supply-chain gate (see 14-security.md)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: EmbarkStudios/cargo-deny-action@v2   # licenses + advisories + bans + sources

  frontend:                      # UI lint / typecheck / unit (+ headless e2e)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: pnpm/action-setup@v4
        with: { version: 9 }
      - uses: actions/setup-node@v4
        with: { node-version: 20, cache: pnpm, cache-dependency-path: apps/desktop/pnpm-lock.yaml }
      - run: sudo apt-get update && sudo apt-get install -y libwebkit2gtk-4.1-dev xvfb   # WebKitGTK for e2e
      - run: pnpm -C apps/desktop install --frozen-lockfile
      - run: pnpm -C apps/desktop lint
      - run: pnpm -C apps/desktop exec vue-tsc --noEmit
      - run: pnpm -C apps/desktop test               # Vitest
      - run: xvfb-run -a pnpm -C apps/desktop test:e2e   # tauri-driver + WebKitWebDriver, headless
```

Job names — `lint`, `test`, `deny`, `frontend` — are canonical; branch protection
requires all four green. These map 1:1 to the local gate in
[AGENT_GUIDE §3](../delivery/AGENT_GUIDE.md).

## 6. `release.yml` (tag-triggered release)

```yaml
name: release
on:
  push:
    tags: ["v*"]                 # R11: every new tag builds & publishes

permissions:
  contents: write                # create the GitHub Release + upload assets

jobs:
  # ---- 1) CLI tarballs (incl. fully-static musl) --------------------------------
  build-cli:
    strategy:
      fail-fast: false
      matrix:
        include:
          - { os: ubuntu-latest,  target: x86_64-unknown-linux-musl,   ext: tar.gz }
          - { os: ubuntu-latest,  target: aarch64-unknown-linux-musl,  ext: tar.gz }
          - { os: ubuntu-latest,  target: x86_64-unknown-linux-gnu,    ext: tar.gz }
          - { os: ubuntu-latest,  target: aarch64-unknown-linux-gnu,   ext: tar.gz }
          - { os: windows-latest, target: x86_64-pc-windows-msvc,      ext: zip }
          - { os: windows-latest, target: aarch64-pc-windows-msvc,     ext: zip }
          - { os: macos-latest,   target: x86_64-apple-darwin,         ext: tar.gz }
          - { os: macos-latest,   target: aarch64-apple-darwin,        ext: tar.gz }
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with: { targets: ${{ matrix.target }} }
      - uses: taiki-e/install-action@v2
        with: { tool: cross }          # cross for musl/aarch64 Linux builds
      - run: cross build --release -p gitpurge-cli --target ${{ matrix.target }}
      - name: Package tarball
        run: ./ci/package-tarball.sh ${{ matrix.target }} ${{ matrix.ext }}   # binary + LICENSEs + README + install.*
      - uses: actions/upload-artifact@v4
        with: { name: cli-${{ matrix.target }}, path: dist/* }

  # ---- 2) Tauri desktop bundles -------------------------------------------------
  build-desktop:
    strategy:
      fail-fast: false
      matrix:
        include:
          - { os: ubuntu-latest,  target: x86_64-unknown-linux-gnu }   # deb + rpm + AppImage
          - { os: windows-latest, target: x86_64-pc-windows-msvc }     # msi + nsis
          - { os: macos-latest,   target: aarch64-apple-darwin }       # dmg + app
          - { os: macos-latest,   target: x86_64-apple-darwin }
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with: { targets: ${{ matrix.target }} }
      - uses: pnpm/action-setup@v4
        with: { version: 9 }
      - uses: actions/setup-node@v4
        with: { node-version: 20, cache: pnpm, cache-dependency-path: apps/desktop/pnpm-lock.yaml }
      - if: runner.os == 'Linux'
        run: sudo apt-get update && sudo apt-get install -y libwebkit2gtk-4.1-dev librsvg2-dev patchelf
      - run: pnpm -C apps/desktop install --frozen-lockfile
      - uses: tauri-apps/tauri-action@v0            # builds frontend + bundles
        env:
          # signing / notarization (used only if the secrets are present, see 14-security.md)
          TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
          APPLE_CERTIFICATE: ${{ secrets.APPLE_CERTIFICATE }}
          APPLE_ID: ${{ secrets.APPLE_ID }}
          WINDOWS_CERTIFICATE: ${{ secrets.WINDOWS_CERTIFICATE }}
        with:
          projectPath: apps/desktop
          args: --target ${{ matrix.target }}
      - uses: actions/upload-artifact@v4
        with: { name: desktop-${{ matrix.target }}, path: apps/desktop/src-tauri/target/**/release/bundle/** }

  # ---- 3) Checksums, signatures, GitHub Release ---------------------------------
  release:
    needs: [build-cli, build-desktop]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/download-artifact@v4
        with: { path: artifacts, merge-multiple: true }
      - name: SHA-256 checksums
        run: (cd artifacts && sha256sum * > SHA256SUMS)
      - name: Sign checksums (minisign)
        run: minisign -S -s <(echo "$MINISIGN_SECRET_KEY") -m artifacts/SHA256SUMS   # public key committed in repo
        env: { MINISIGN_SECRET_KEY: ${{ secrets.MINISIGN_SECRET_KEY }} }
      - uses: softprops/action-gh-release@v2
        with:
          draft: true                 # maintainer reviews, then publishes
          generate_release_notes: true
          files: |
            artifacts/*
            artifacts/SHA256SUMS
            artifacts/SHA256SUMS.minisig
```

Notes on **signing / integrity** (detail in [14-security.md](14-security.md)):

- **Checksums:** `SHA256SUMS` for every artifact, always produced.
- **Provenance signature:** `SHA256SUMS` is signed with **minisign** (public key
  committed to the repo); this works on all platforms with no OS-vendor account. A
  `cosign` keyless signature is an acceptable alternative for the same purpose.
- **OS code-signing / notarization caveats:** Windows **Authenticode** signing needs a
  purchased cert; macOS needs a **Developer ID** cert + **notarytool** notarization
  (and stapling). These require paid accounts and org secrets; when those secrets are
  **absent**, `tauri-action` produces **unsigned** bundles and the release notes flag
  it (users may see SmartScreen/Gatekeeper prompts). Minisign covers integrity
  regardless.

## 7. Versioning & release process

- **SemVer**, single source of truth = the workspace `version` in the root
  `Cargo.toml`; the Tauri app version and npm package version are derived from it.
- **CHANGELOG.md** in "Keep a Changelog" format, grouped by
  Added/Changed/Fixed/Security. Entries follow the Conventional Commits history
  ([CONVENTIONS §12](../delivery/CONVENTIONS.md)).
- **Cutting a release (maintainer):**
  1. Bump the workspace version; update `CHANGELOG.md` (move "Unreleased" → `vX.Y.Z`).
  2. `git commit -m "chore(release): vX.Y.Z"` on a release branch → PR → merge to `main`.
  3. Tag: `git tag vX.Y.Z && git push origin vX.Y.Z`.
  4. `release.yml` fires, builds the full matrix, and opens a **draft** release with
     all artifacts + `SHA256SUMS` + `.minisig`.
  5. Maintainer verifies artifacts/checksums and **publishes** the release.
- **Pre-releases:** tags like `v1.0.0-rc.1` follow the same path; mark the GitHub
  Release as "pre-release".

## 8. Traceability

| Requirement | Where satisfied |
| :--- | :--- |
| **R9** — open-source; tarball + deb/rpm/AppImage/msi/dmg bundles | §1 artifact matrix; `build-cli` + `build-desktop` jobs (§6); Apache-2.0 license files shipped in every artifact |
| **R10** — single self-contained/portable binary; runs without install | §2.1 (static musl CLI); §1.1 tarball; §3 `install-cli`; verified by `tarball_binary_runs_no_deps` ([12-testing-strategy.md](12-testing-strategy.md#9-test--requirement--phase-mapping)) |
| **R11** — GH Actions builds all targets on each tag → release with attached binaries | §6 `release.yml` on `v*`, matrix build, checksums + signatures, GitHub Release |

See [14-security.md](14-security.md) for `cargo-deny`/`cargo-audit`, SBOM, and the signing
posture, and [12-testing-strategy.md](12-testing-strategy.md) for the CI test gates.

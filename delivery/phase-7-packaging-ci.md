# Phase 7 — Packaging, distribution & CI release

`Status: Draft` · `Owner: Delivery` · `Last-updated: 2026-07-11` ·
`Related: [ROADMAP](../docs/ROADMAP.md#p7--packaging-distribution--ci-release-9-ed), [CONVENTIONS §1/§14](CONVENTIONS.md), [13-distribution-and-ci.md](../docs/13-distribution-and-ci.md)`

## Goal

Make Git Purge shippable. Produce the self-contained portable **tarball** per platform
(the primary distribution — a `git-purge` binary with no runtime deps plus the
`install-cli` helper), the Tauri desktop **bundles** for every target
(`.deb`/`.rpm`/`.AppImage`, `.msi`/NSIS `.exe`, `.dmg`/`.app`), and the CI/release
automation: `ci.yml` runs the full gate matrix on push/PR, and `release.yml` triggers
on a `v*` tag, builds the whole matrix, and attaches all artifacts plus checksums and
signatures to a GitHub Release. Consistent with
[13-distribution-and-ci.md](../docs/13-distribution-and-ci.md) and licensed
**Apache-2.0**.

**Milestone:** M5 — Shippable 1.0 (packaging + release automation).

**Dependencies:** **P3** (the CLI binary + `install-cli`) and **P4** (the desktop app
to bundle) merged. Extends the P0 `ci.yml`.

## Tasks

| Task ID | Title | Description | Files (repo-relative) | Depends-on | ∥? | Est (ED) | Acceptance test |
| :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- |
| **P7-T1** | Self-contained tarball + `install-cli` | Build a portable tarball per platform (`linux-x86_64`, `linux-aarch64`, `macos-*`, `windows-x86_64`) containing the statically-linked-as-possible `git-purge` binary, `install-cli`, LICENSE files, and README. No runtime deps ([CONVENTIONS §14](CONVENTIONS.md)). | `packaging/tarball/*`, `.cargo/config.toml`, `xtask/` (or `packaging/build.sh`) | P3-T7 | no | 1.5 | The tarball's `git-purge` runs on a clean container with **no toolchain/git installed** and `install-cli --user` puts it on `PATH`; **R9/R10**. |
| **P7-T2** | Tauri bundles (all targets) | Configure and produce Tauri v2 bundles: Linux `.deb`/`.rpm`/`.AppImage`, Windows `.msi` + NSIS `.exe`, macOS `.dmg`/`.app`; bundle id `com.gitpurge.desktop`; WebView2 bootstrap on Windows. | `apps/desktop/src-tauri/tauri.conf.json` (bundle targets), `apps/desktop/src-tauri/bundle/*` | P4-T1 | yes | 2 | `tauri build` on each OS emits the expected bundle formats; the Linux `.AppImage` and `.deb` launch on a clean VM; **R9**. |
| **P7-T3** | `release.yml` matrix on tag | GitHub Actions workflow triggered on `v*` tags: build the full CLI + desktop matrix across OS runners, collect artifacts, and create a GitHub Release with all binaries/bundles attached. Triggers only on tag ([CONVENTIONS §14](CONVENTIONS.md)). | `.github/workflows/release.yml` | P7-T1, P7-T2 | no | 2 | Pushing `vX.Y.Z` on a fixture branch produces a (draft) Release with tarballs + all bundles attached across the matrix; **R11**. |
| **P7-T4** | Checksums + signing | Generate `SHA256SUMS` for every artifact and sign artifacts/checksums (minisign/cosign or GPG); publish public key + verification instructions. | `.github/workflows/release.yml` (sign step), `packaging/sign/*`, `docs/13-distribution-and-ci.md` (verify steps) | P7-T3 | yes | 1.5 | Every released artifact has a matching checksum; a signature verifies against the published key; a tampered artifact fails verification; **R9/R11**. |
| **P7-T5** | `ci.yml` full gate matrix | Extend P0's `ci.yml` to a cross-platform matrix: Rust gate (fmt/clippy/nextest/deny/audit) on Linux+macOS+Windows, frontend gate (`pnpm lint`/`test`/`vue-tsc`), and coverage upload. | `.github/workflows/ci.yml` | P0-T7 | yes | 1 | PR CI runs the full matrix green; a failing frontend or clippy check blocks merge; coverage report published; **R8**. |
| **P7-T6** | Release smoke + licensing | End-to-end release verification: tag → artifacts → checksum/signature verification, plus dual-license (`LICENSE-MIT`, `LICENSE-APACHE`) and third-party license aggregation present in artifacts. | `LICENSE-MIT`, `LICENSE-APACHE`, `packaging/verify_release.sh`, `THIRD-PARTY-LICENSES` | P7-T3, P7-T4 | yes | 1 | `verify_release.sh` downloads a draft release's artifacts, checks all checksums+signatures, and confirms both license files ship; **R9/R11**. |

Total ≈ 9 ED.

## Exit criteria

- Pushing a `vX.Y.Z` tag yields a full release with all artifacts and **verified
  checksums** (ROADMAP P7 exit).
- The portable tarball binary runs with no dependencies on a clean container;
  `install-cli` works.
- Tauri bundles exist for deb/rpm/AppImage/msi/nsis/dmg; artifacts are signed and
  dual-licensed.

### Requirements & safety invariants satisfied

- **R9** (open-source; tarball + bundles for all targets; licenses): P7-T1, P7-T2,
  P7-T6.
- **R10** (single-binary/portable, no-deps run): P7-T1.
- **R11** (GitHub Actions release on tag with attached binaries): P7-T3, P7-T4.
- **R8** (CI gates): P7-T5.
- Safety invariants are unchanged by packaging; P7 must not weaken them (e.g., no debug
  logging of secrets in release builds — verified again in P8).

## Risks & open questions

- **Fully-static Linux builds** — glibc vs musl trade-offs; the gix-first design
  (ADR-0002) minimizes C deps, but `git2`/OpenSSL linkage can force musl or vendored
  OpenSSL. Decide per-target and document in [13](../docs/13-distribution-and-ci.md).
- **macOS/Windows code signing & notarization** — requires paid certs/secrets in CI;
  scope whether 1.0 ships notarized or with documented Gatekeeper/SmartScreen steps.
- **Cross-compilation vs. native runners** — decide between cross toolchains and
  per-OS GitHub runners for the matrix; native runners are simpler but slower.
- **Signing tool choice** — minisign vs cosign vs GPG affects the verify UX; pick one
  and publish the key + instructions.
- **AppImage/WebKitGTK deps** — the Linux desktop bundle depends on the OS webview;
  document the minimum runtime requirements.

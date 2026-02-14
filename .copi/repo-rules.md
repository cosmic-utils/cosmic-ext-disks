# copi repo rules — cosmic-ext-disks

Last updated: 2026-01-24

This document is **authoritative** for agents working in this repository.

## Confirmed conventions (authoritative)

### A) Branching & PR workflow

- **Default branch:** `main`
- **Branch naming:** Use the repo’s existing convention: `feature/{slug}`, plus `fix/{slug}`, `chore/{slug}`.
  - Examples: `feature/unpick-disksrs`, `fix/partition-size`, `chore/update-deps`
- **Issue IDs in branches:** Not required.
- **Merge strategy:** Squash merge.

### B) Project identity

- **PROJECT_SHORTNAME:** `DSK`
- **Purpose (one sentence):** Disk utility application for the COSMIC desktop (UI + DBus abstraction).
- **Primary owner/team:** cosmic-utils (GitHub org).

### C) Environments & release

- **Environment names:** `local` → `test` → `prod`
- **Release process:** Use detected workflow: publish on push to `main` + GitHub Release created.
- **Versioning:** Use detected versioning (SemVer with `v` tag prefix).

### D) Repo management

- **Runtime/tooling baseline:** Rust stable, **edition 2024**, toolchain unpinned.
- **Supported OS:** Linux only.
- **Quality gates:** CI must pass `cargo test --workspace --all-features`, `cargo clippy --workspace --all-features`, and `cargo fmt --all --check`.
- **Commit messages:** Conventional Commits.
  - **Version bumps:** Do not infer bumps from commit messages; bump via tags/releases.
- **Secrets policy:** Secrets only via CI secret stores / local environment; never commit real secrets.
- **Module organization:**
  - **Small modules (≤3 files):** Use sibling files declared in parent (e.g., `parent/mod.rs` declares `mod child;`, child lives in `parent/child.rs`)
  - **Large module hierarchies (4+ files):** Use folder with `mod.rs` pattern (e.g., `disks/drive/mod.rs` with siblings `actions.rs`, `discovery.rs`, etc.)
  - **Rationale:** Keeps simple modules flat and readable; uses folders only when hierarchy aids organization.

### E) Documentation expectations

- **Doc depth:** Standard.
- **Runbooks location:** Prefer existing README locations plus `.copi/` for agent docs.
- **Compliance constraints:** None.

## Detected facts (non-authoritative, with evidence)

- **Git remote:** `git@github.com:cosmic-utils/cosmic-ext-disks.git`
  - Evidence: `git remote -v`
- **Main CI quality gates (PR):** tests, clippy, rustfmt on Ubuntu.
  - Evidence: `.github/workflows/ci.yml`
- **Publish workflow:** on push to `main`, computes version, updates crate versions, publishes crates, creates GitHub Release.
  - Evidence: `.github/workflows/main.yml`
- **Rust edition:** `2024` in crates.
  - Evidence: `storage-ui/Cargo.toml`, `storage-dbus/Cargo.toml`

## Unknown / TBD

- **PR templates / CODEOWNERS:** Not detected; add if desired.
- **Issue tracker key format:** Not used; if you later adopt issue keys (e.g., `DSK-123`), update branch naming rules.
- **Security model for privileged disk ops:** needs explicit documentation (Polkit, permission prompts, etc.).

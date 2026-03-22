# Iteration: 002 — CI/CD Setup

> Configure GitHub Actions workflows for build and publish.

## Metadata

- **Type:** chore
- **Status:** done
- **Created:** 2026-03-12
- **Completed:** 2026-03-12
- **Author:** Francisco Tomé Barros Jr

## Description

Set up GitHub Actions CI/CD workflows for the stand-in workspace:
- **build.yml** — CI on push to feature/bug/issue branches and PRs to main/release
- **publish.yml** — Publish to crates.io on version tags (`v*`)

## Acceptance Criteria

- [x] Build workflow runs on Linux, macOS, Windows
- [x] Build workflow checks formatting, lints, tests, builds, and docs
- [x] Publish workflow runs security audit before publishing
- [x] Publish workflow publishes workspace to crates.io
- [x] No Node.js deprecation warnings

## Implementation Tasks

- [x] Create `.github/workflows/build.yml`
- [x] Create `.github/workflows/publish.yml`
- [x] Add `FORCE_JAVASCRIPT_ACTIONS_TO_NODE24=true` env var
- [x] Upgrade `actions/checkout@v4` → `@v6` for Node.js 24

## Quality Checks

- [x] Build workflow passes on GitHub Actions
- [x] No deprecation warnings
- [x] All project quality gates included in CI

## Decisions Made

| Decision | Rationale | Date |
|----------|-----------|------|
| No binaries workflow | stand-in is a library, not a CLI | 2026-03-12 |
| Use cargo-release for publishing | Handles workspace publish order | 2026-03-12 |
| Use nextest for publish tests | Faster test runner for CI | 2026-03-12 |
| actions/checkout@v6 | Node.js 24 native support | 2026-03-12 |

## Blockers & Notes

- `CRATES_IO_API_TOKEN` secret must be configured in GitHub repo settings before publishing

## References

- **Specification:** N/A
- **PR/MR:** —
- **Commits:** 65f7d53, 7e700cd, 0622a6d
- **Related Issues:** —

# Security scan, 2026-10-22

A scan of `main` at `5d85e6a`: known CVEs in the dependencies, secrets, CI workflows and scripts.
Each line names the command that produced it. The CI fixes are in the same PR as this file.

## Dependencies: no known vulnerability

| Lock | Packages | Command | Result |
|---|---|---|---|
| `extension/package-lock.json` | 135 | `nix run nixpkgs#osv-scanner -- scan source --recursive .`, `npm audit` | 0 |
| `site/manifest.toml`, `site/content/manifest.toml` (Hex) | 8 | OSV API `querybatch`, ecosystem `Hex` | 0 |
| `extension/Cargo.lock` (PR #87, not on `main`) | 37 | `nix run nixpkgs#cargo-audit -- audit` | 0 |
| node of the dev shell | 24.22.1 | `nix develop -c node --version` | the October 2026 fix (#83) |

OSV-Scanner reads only `extension/package-lock.json`: it has no extractor for Gleam's `manifest.toml`
or for `flake.lock` (`could not determine extractor suitable to this file`), so `cve.yml` does not
cover the Hex packages. The submodules `forks/picasso` and `vendor/archify` were not scanned here.

## Secrets: none

- `gitleaks git --log-opts=--all`: every ref of the repository, 0 leaks.
- `git grep` for Anthropic, OpenAI-style, GitHub, AWS, Slack and Google keys and private-key
  headers: 0. No tracked `.env`, `.pem`, `.key` or credentials file.
- `gitleaks dir` flags 71 `generic-api-key` strings, all in `build/figure/*/profile/Default/Preferences`:
  Chrome's own profile values from the figure build. `build/` is in `.gitignore` and has no tracked file.

## CI workflows: fixed in this PR

`nix run nixpkgs#zizmor -- .github/workflows/` before: 19 high, 8 medium. After: 0 high, 0 medium.

- 19 `uses:` pinned to a commit SHA, with the version in a comment (`unpinned-uses`): a moved tag
  can no longer change what CI runs. The SHAs are the current commits of the tags already used
  (`checkout` v4.4.0, `install-nix-action` v31.11.2, `setup-node` v4.4.0, `upload-artifact` v4.6.2,
  `upload-pages-artifact` v4.0.0, `deploy-pages` v4.0.5).
- `persist-credentials: false` on the 8 checkouts (`artipacked`): no workflow pushes with git;
  `release.yml` uses `GH_TOKEN` through `gh`.

Already fine: no `pull_request_target`, `workflow_run` or `issue_comment`; no `${{ }}` inside a
`run:`; top-level `permissions: contents: read` (or `{}`); the one secret (`ANTHROPIC_API_KEY`)
reaches only the eval job, on `workflow_dispatch`. `actionlint`: 0.

## Open, not fixed here

- `plugin.yml` installs `@anthropic-ai/claude-code@$CLAUDE_CODE_VERSION` with `npm install -g`, outside
  a lock file (zizmor `adhoc-packages`, low). The version is pinned; its dependencies are not.
- `scripts/install-opencode.sh` downloads the tag's tarball with `curl | tar` and checks no hash. A
  git tag can be moved; a sha256 per release, checked by the script, would close it.
- `scripts/install_current.sh` is run as `curl … | bash` from `main`, by design (trying unreleased
  work). It trusts whoever can push to `main`; the tag install stays the stable path.
- `cve.yml` runs `nix run nixpkgs#osv-scanner` from the registry's `nixpkgs`, not `flake.lock`, and
  `npx --yes is-my-node-vulnerable@1.6.1` without a lock: the scanners themselves are not pinned.
- `cve.yml` misses the Hex packages (above). A step that sends `site/**/manifest.toml` to the OSV API
  would cover them.

## Checked, no issue

- `extension/src/options.ts:31` writes `innerHTML` from a fixed template; host names and model names
  go through `textContent`.
- The `ptbr` binaries are checked against `plugins/gramatico/bin/tools.lock` (sha256) by
  `just tools-check`.

set shell := ["bash", "-euo", "pipefail", "-c"]

# ptbr-writter — recipes for developing and running the skill with OpenCode + local Qwen.

default:
    @just --list

# Run OpenCode with this repo's configuration (expects a llamacpp server at 127.0.0.1:8080).
opencode *args:
    opencode {{ args }}

# Serve Qwen3.6-27B (Q4_K_M) with llama.cpp CUDA and run OpenCode against it.
# Requires: nix, NVIDIA driver, ~24 GB GPU, 16 GB RAM, 17 GB disk for the one-time download.
# Override llamacpp parameters with LLAMA_ARG_* env vars.
opencode-qwen *args:
    #!/usr/bin/env bash
    set -euo pipefail
    export LLAMA_ARG_HF_REPO="${LLAMA_ARG_HF_REPO:-unsloth/Qwen3.6-27B-GGUF:Q4_K_M}"
    export LLAMA_ARG_CTX_SIZE="${LLAMA_ARG_CTX_SIZE:-131072}" LLAMA_ARG_N_GPU_LAYERS="${LLAMA_ARG_N_GPU_LAYERS:-99}"
    export LLAMA_ARG_CACHE_TYPE_K="${LLAMA_ARG_CACHE_TYPE_K:-q8_0}" LLAMA_ARG_CACHE_TYPE_V="${LLAMA_ARG_CACHE_TYPE_V:-q8_0}"
    export LLAMA_ARG_FLASH_ATTN=on LLAMA_ARG_JINJA=1 LLAMA_ARG_THREADS="${LLAMA_ARG_THREADS:-16}"
    export LLAMA_ARG_NO_MMPROJ=1 LLAMA_ARG_N_PARALLEL=1 LLAMA_ARG_ALIAS=qwen3.6-27b LLAMA_ARG_HOST=127.0.0.1 LLAMA_ARG_PORT=8080
    srv=$(NIXPKGS_ALLOW_UNFREE=1 nix build --impure --no-link --print-out-paths --expr '
      let p = import (builtins.getFlake (toString ./.)).inputs.nixpkgs {
        system = builtins.currentSystem; config = { allowUnfree = true; cudaSupport = true; };
      }; in p.llama-cpp')/bin/llama-server
    log="${TMPDIR:-/tmp}/llama-server-ptbr-writter.log"
    echo "llama-server log: $log"
    LD_LIBRARY_PATH="/usr/lib/x86_64-linux-gnu:/run/opengl-driver/lib" "$srv" >"$log" 2>&1 &
    trap 'kill $!' EXIT
    until curl -sf http://127.0.0.1:8080/health >/dev/null; do
        kill -0 $! 2>/dev/null || { tail -n 20 "$log" >&2; echo "llama-server exited" >&2; exit 1; }
        sleep 2
    done
    opencode -m llamacpp/qwen3.6-27b {{ args }}

# Health-check the local llamacpp server.
opencode-status:
    #!/usr/bin/env bash
    set -euo pipefail
    if curl -sf http://127.0.0.1:8080/health >/dev/null; then
        echo "llama-server: healthy (127.0.0.1:8080)"
        curl -sf http://127.0.0.1:8080/v1/models | python3 -m json.tool 2>/dev/null || true
    else
        echo "llama-server: not reachable at 127.0.0.1:8080" >&2
        exit 1
    fi

# Stop the llamacpp server on port 8080.
opencode-stop:
    #!/usr/bin/env bash
    set -euo pipefail
    pid="$(lsof -ti:8080 2>/dev/null || ss -ltnp 2>/dev/null | grep -m1 ':8080' | grep -oP 'pid=\K[0-9]+' || true)"
    if [ -z "$pid" ]; then echo "no process on port 8080"; exit 0; fi
    echo "stopping llama-server pid $pid"
    kill "$pid" || true

# Tail the llamacpp log.
opencode-logs *args="-n 40":
    #!/usr/bin/env bash
    set -euo pipefail
    log="${TMPDIR:-/tmp}/llama-server-ptbr-writter.log"
    [ -f "$log" ] || { echo "no log at $log" >&2; exit 1; }
    tail {{ args }} "$log"

# Validate flake.nix and flake.lock without building.
nix-check:
    nix flake check --no-build
    nix flake metadata --json >/dev/null
    echo "nix flake check: ok"

# Update flake.lock.
nix-update *args="":
    nix flake update {{ args }}

# Format the justfile.
fmt:
    just --fmt --unstable

# Build the site into site/dist/ in picasso's gleam-website toolchain: content, tests, prerender.
site:
    nix develop --no-update-lock-file .#site --command just --justfile site/justfile --working-directory site ci

# Build the site and serve it on http://localhost:8000.
site-serve: site
    nix develop --no-update-lock-file .#site --command python3 -m http.server -d site/dist 8000

# Check that flake.lock pins gleam-website to the forks/picasso submodule's revision.
pins:
    #!/usr/bin/env bash
    set -euo pipefail
    lock="$(jq -r '.nodes["gleam-website"].locked.rev' flake.lock)"
    sub="$(git ls-files -s forks/picasso | cut -d' ' -f2)"
    [ "$lock" = "$sub" ] || { echo "pins: flake.lock has picasso $lock, forks/picasso is $sub" >&2; exit 1; }
    echo "pins: picasso $sub"

# Lint prose against Saramago's punctuation: no dash, quotes, ? ! ; : or ellipsis.
saramago +files:
    vale --config lint/saramago/.vale.ini {{ files }}

# Self-test of the Saramago linter: the fixture in his manner passes, the plain one trips every rule.
saramago-check:
    #!/usr/bin/env bash
    set -euo pipefail
    cfg=lint/saramago/.vale.ini
    vale --config "$cfg" lint/saramago/fixtures/saramaguiano.md >/dev/null
    out="$(vale --config "$cfg" --no-exit --output=line lint/saramago/fixtures/convencional.md)"
    for rule in Travessao Aspas Entonacao Pausa; do
        grep -q "Saramago.$rule:" <<<"$out" || { echo "saramago-check: Saramago.$rule did not fire" >&2; exit 1; }
    done
    echo "saramago-check: ok"

# Basic quality gate: justfile parses, required files exist, toolchain pins agree, linters self-test.
# Pick the skill and the level for a writing request: just route "reescreva este runbook".
route +request:
    python3 skills/ptbr-routing/scripts/route.py "{{ request }}"

# Check the Claude Code plugin: manifests parse, every skill link resolves, the version has a CHANGELOG entry.
plugin-check:
    #!/usr/bin/env bash
    set -euo pipefail
    python3 -m json.tool .claude-plugin/marketplace.json >/dev/null
    v=$(python3 -c 'import json; print(json.load(open("plugins/ptbr-writter/.claude-plugin/plugin.json"))["version"])')
    [[ $v =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo "plugin-check: version $v is not X.Y.Z" >&2; exit 1; }
    grep -q "^## v$v " CHANGELOG.md || { echo "plugin-check: CHANGELOG.md has no v$v" >&2; exit 1; }
    for f in plugins/ptbr-writter/skills/*/SKILL.md; do test -e "$f" || { echo "plugin-check: $f is a broken link" >&2; exit 1; }; done
    bash skills/ptbr-routing/scripts/session-check.sh --self-test
    echo "plugin-check: ok (v$v)"

# What ships in the plugin changed since the last v* tag: lists the files and exits 1; installs keep the old text until a release.
release-due:
    #!/usr/bin/env bash
    set -euo pipefail
    files=$(find -L plugins/ptbr-writter .claude-plugin -path "*/evals" -prune -o -type f -exec realpath --relative-to=. {} + | sort -u)
    last=$(git describe --tags --abbrev=0 --match 'v*' 2>/dev/null || true)
    if [ -z "$last" ]; then changed=$files; since="(no release yet)"
    else changed=$(git diff --name-only "$last" HEAD -- $files); since="since $last"; fi
    [ -n "$changed" ] || { echo "release-due: nothing in the plugin changed $since"; exit 0; }
    echo "release-due: the plugin changed $since; installs keep the old text until just release X.Y.Z:"
    sed 's/^/  /' <<<"$changed"
    exit 1

# Install the plugin from this checkout into an empty Claude Code config and check what a third party gets.
plugin-smoke:
    scripts/plugin-smoke.sh

# Run the plugin's eval cases on your own Claude login, once each: just plugin-eval --case 'estrito*'.
plugin-eval *args:
    claude plugin eval plugins/ptbr-writter --trust-plugin --no-publish --runs 1 {{ args }}

# Release X.Y.Z in two runs: a chore/release-vX.Y.Z PR (plugin.json, CHANGELOG), then, after its merge, the tag on main.
release version:
    #!/usr/bin/env bash
    set -euo pipefail
    v="{{ version }}"; json=plugins/ptbr-writter/.claude-plugin/plugin.json
    [[ $v =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo "release: $v is not X.Y.Z" >&2; exit 1; }
    [ -z "$(git status --porcelain)" ] || { echo "release: the tree is not clean" >&2; exit 1; }
    git fetch -q origin main --tags
    ! git rev-parse -q --verify "refs/tags/v$v" >/dev/null || { echo "release: v$v exists" >&2; exit 1; }
    now=$(git show "origin/main:$json" | python3 -c 'import json,sys; print(json.load(sys.stdin)["version"])')
    if [ "$now" != "$v" ]; then
        git switch -q -c "chore/release-v$v" origin/main
        # A release with nothing new for the plugin would reach no one: refuse it.
        due=$(just release-due | tail -n +2 || true)
        [ -n "$due" ] || { git switch -q -; git branch -q -D "chore/release-v$v"; echo "release: nothing in the plugin changed since the last tag" >&2; exit 1; }
        python3 - "$json" "$v" <<'PY'
    import json, sys
    path, v = sys.argv[1], sys.argv[2]
    d = json.load(open(path)); d["version"] = v
    open(path, "w").write(json.dumps(d, indent=2, ensure_ascii=False) + "\n")
    PY
        python3 - "$v" "$(date -u +%F)" "$due" <<'PY'
    import sys
    v, day, due = sys.argv[1], sys.argv[2], sys.argv[3]
    s = open("CHANGELOG.md").read()
    head, rest = s.split("\n## ", 1)
    files = ", ".join(f"`{f.strip()}`" for f in due.splitlines())
    open("CHANGELOG.md", "w").write(f"{head}\n## v{v} — {day}\n\n- (descreva a release) mudou: {files}\n\n## {rest}")
    PY
        ${EDITOR:-vi} CHANGELOG.md
        ! grep -q '(descreva a release)' CHANGELOG.md || { echo "release: CHANGELOG.md still has the placeholder" >&2; exit 1; }
        just plugin-check
        git commit -qam "release: v$v" -m "Tested: just plugin-check" -m "Co-Authored-By: Claude <noreply@anthropic.com>"
        git push -q -u origin "chore/release-v$v"
        gh pr create --base main --title "release: v$v" --body "plugin.json e CHANGELOG.md em v$v. Depois do merge: \`just release $v\` no main cria a tag."
    else
        [ "$(git rev-parse HEAD)" = "$(git rev-parse origin/main)" ] || { echo "release: check out origin/main first (git switch main && git pull)" >&2; exit 1; }
        just release-check "v$v"
        git tag -a "v$v" -m "ptbr-writter v$v"
        git push -q origin "v$v"
        echo "release: v$v pushed; release.yml publishes it"
    fi

# The tag a release pushes must be the plugin's version: just release-check v0.1.0 (release.yml runs it).
release-check tag: plugin-check
    test "{{ tag }}" = "v$(python3 -c 'import json; print(json.load(open("plugins/ptbr-writter/.claude-plugin/plugin.json"))["version"])')"

quality: pins saramago-check plugin-check
    python3 skills/ptbr-routing/scripts/route.py --self-test
    python3 skills/ptbr-routing/scripts/route.py --check
    just --list >/dev/null
    test -f .opencode.json
    test -f flake.nix
    test -f flake.lock
    python3 -m json.tool .opencode.json >/dev/null
    # The README copies the tables of site/data/ by hand: each repo, roadmap item, rule and planned level must be in it.
    tail -n +2 site/data/awesome.tsv site/data/roadmap.tsv site/data/origins.tsv site/data/planned.tsv | grep -v '^==>' | cut -f1 | grep . | while read -r x; do grep -qF -- "$x" README.md || { echo "quality: README.md lacks $x" >&2; exit 1; }; done
    echo "quality: ok"

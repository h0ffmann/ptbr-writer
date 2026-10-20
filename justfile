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

quality: pins saramago-check
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

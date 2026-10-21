set shell := ["bash", "-euo", "pipefail", "-c"]

# ptbr-writer — recipes for developing and running the skill with OpenCode + local Qwen.

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
    log="${TMPDIR:-/tmp}/llama-server-ptbr-writer.log"
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
    log="${TMPDIR:-/tmp}/llama-server-ptbr-writer.log"
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
site: figure-init
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
    pdf="$(sed -n 's/^PICASSO_REV=//p' plugins/ptbr-writer/scripts/to-pdf.sh)"
    [ "$pdf" = "$sub" ] || { echo "pins: to-pdf.sh has picasso $pdf, forks/picasso is $sub" >&2; exit 1; }
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

# The pinned Archify (vendor/archify, sparse to archify/): once per clone. Not part of the plugin.
figure-init:
    git submodule update --init --depth 1 vendor/archify
    git -C vendor/archify sparse-checkout set --no-cone /archify/
    @echo "figure-init: $(git -C vendor/archify rev-parse --short HEAD)"

# Draw every docs/archify/<slug>.json to docs/img/<slug>.png: Archify validates and checks it (finalize),
# picasso's look (docs/archify/picasso.css) goes over it, headless Chrome takes the picture. Needs node,
# Chrome (CHROME names another) and the Google Fonts; writes docs/archify/figures.lock.
figure: figure-init
    #!/usr/bin/env bash
    set -euo pipefail
    mkdir -p build/figure docs/img
    for src in docs/archify/*.json; do
        slug="$(basename "$src" .json)"; out="build/figure/$slug"
        rm -rf "$out"; mkdir -p "$out"
        ARCHIFY_UPDATE_CHECK_DISABLED=1 node vendor/archify/archify/bin/archify.mjs finalize "$(jq -r .diagram_type "$src")" "$src" "$out/page.html" --out-dir "$out/receipts" --quality showcase
        read -r w h < <(grep -o '<svg viewBox="[^"]*"' "$out/page.html" | head -1 | sed -E 's/.*0 0 ([0-9.]+) ([0-9.]+)"/\1 \2/')
        crop="$(awk -F'\t' -v s="$slug" '$1==s {print $2}' docs/archify/crop.tsv)"; h="${crop:-$h}"
        python3 - "$out/page.html" "$out/frame.html" "$w" "$h" <<'PY'
    import sys, re
    src, dst, w, h = sys.argv[1:]
    page = open(src, encoding="utf-8").read()
    css = open("docs/archify/picasso.css", encoding="utf-8").read()
    fonts = '<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Fjalla+One&family=Inter:wght@400;600&display=block">'
    page = page.replace("</head>", fonts + "<style>" + css + "</style></head>", 1)
    page = re.sub(r'<svg viewBox="0 0 [0-9.]+ [0-9.]+"', f'<svg viewBox="0 0 {w} {h}"', page, count=1)
    open(dst, "w", encoding="utf-8").write(page)
    PY
        "${CHROME:-google-chrome}" --headless --disable-gpu --hide-scrollbars --force-dark-mode --force-device-scale-factor=2 --user-data-dir="$PWD/$out/profile" --window-size="$w,$h" --virtual-time-budget=3000 --screenshot="$PWD/docs/img/$slug.png" "file://$PWD/$out/frame.html" 2>/dev/null
        echo "figure: docs/img/$slug.png"
    done
    just _figure-lock > docs/archify/figures.lock

# The inputs of each figure (its source, the look, the crop, the Archify pin), one line per slug.
_figure-lock:
    #!/usr/bin/env bash
    set -euo pipefail
    pin="$(git ls-files -s vendor/archify | cut -d' ' -f2)"
    for src in docs/archify/*.json; do
        printf '%s\t%s\n' "$(basename "$src" .json)" "$(cat "$src" docs/archify/picasso.css docs/archify/crop.tsv | sha256sum | cut -c1-16)-${pin:0:7}"
    done

# Cheap and offline: each docs/archify/<slug>.json has its PNG, and both match the last `just figure`.
figure-check:
    #!/usr/bin/env bash
    set -euo pipefail
    for src in docs/archify/*.json; do
        test -s "docs/img/$(basename "$src" .json).png" || { echo "figure-check: no PNG for $src; run just figure" >&2; exit 1; }
    done
    just _figure-lock | cmp -s - docs/archify/figures.lock || { echo "figure-check: a source, the look or the Archify pin changed since the PNG; run just figure" >&2; exit 1; }
    echo "figure-check: ok"

# Basic quality gate: justfile parses, required files exist, toolchain pins agree, linters self-test.
# Pick the skill and the level for a writing request: just route "reescreva este runbook".
route +request:
    python3 skills/ptbr-routing/scripts/route.py "{{ request }}"

# Check the Claude Code plugin: manifests parse, every skill link resolves, the version has a CHANGELOG entry.
plugin-check:
    #!/usr/bin/env bash
    set -euo pipefail
    python3 -m json.tool .claude-plugin/marketplace.json >/dev/null
    v=$(python3 -c 'import json; print(json.load(open("plugins/ptbr-writer/.claude-plugin/plugin.json"))["version"])')
    [[ $v =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo "plugin-check: version $v is not X.Y.Z" >&2; exit 1; }
    grep -q "^## v$v " CHANGELOG.md || { echo "plugin-check: CHANGELOG.md has no v$v" >&2; exit 1; }
    for f in plugins/ptbr-writer/skills/*/SKILL.md; do test -e "$f" || { echo "plugin-check: $f is a broken link" >&2; exit 1; }; done
    bash skills/ptbr-routing/scripts/session-check.sh --self-test
    bash plugins/ptbr-writer/hooks/ask-levels.sh --self-test
    python3 -I plugins/ptbr-writer/scripts/reading_time.py --self-test
    echo "plugin-check: ok (v$v)"

# What ships in the plugin changed since the last v* tag: lists the files and exits 1; installs keep the old text until a release.
release-due:
    #!/usr/bin/env bash
    set -euo pipefail
    files=$(find -L plugins/ptbr-writer .claude-plugin -path "*/evals" -prune -o -type f -exec realpath --relative-to=. {} + | sort -u)
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
    claude plugin eval plugins/ptbr-writer --trust-plugin --no-publish --runs 1 {{ args }}

# What the .pdf output downloads and builds on a machine with Nix and no pandoc, timed into an empty store.
pdf-bench:
    scripts/pdf-bench.sh

# Release X.Y.Z in two runs: a chore/release-vX.Y.Z PR (plugin.json, CHANGELOG), then, after its merge, the tag on main.
release version:
    #!/usr/bin/env bash
    set -euo pipefail
    v="{{ version }}"; json=plugins/ptbr-writer/.claude-plugin/plugin.json
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
        git tag -a "v$v" -m "ptbr-writer v$v"
        git push -q origin "v$v"
        echo "release: v$v pushed; release.yml publishes it"
    fi

# The tag a release pushes must be the plugin's version: just release-check v0.2.0 (release.yml runs it).
release-check tag: plugin-check
    test "{{ tag }}" = "v$(python3 -c 'import json; print(json.load(open("plugins/ptbr-writer/.claude-plugin/plugin.json"))["version"])')"

# Write each README's reading time (the plugin's calculator) under its language line.
reading-time:
    #!/usr/bin/env bash
    set -euo pipefail
    for r in "README.md pt-BR" "README.en.md en"; do
        set -- $r
        line=$(python3 -I plugins/ptbr-writer/scripts/reading_time.py --lang "$2" "$1")
        python3 -I - "$1" "$line" <<'PY'
    import re, sys
    path, line = sys.argv[1], sys.argv[2]
    s = open(path, encoding="utf-8").read()
    pat = re.compile(r"^(Leitura estimada|Estimated reading time):.*$", re.M)
    if pat.search(s):
        s = pat.sub(line, s, count=1)
    else:
        marker = "\n## "
        i = s.index(marker)
        s = s[:i] + "\n" + line + "\n" + s[i:]
    open(path, "w", encoding="utf-8").write(s)
    PY
        echo "$1: $line"
    done

quality: pins saramago-check plugin-check figure-check
    python3 skills/ptbr-routing/scripts/route.py --self-test
    python3 skills/ptbr-routing/scripts/route.py --check
    just --list >/dev/null
    test -f .opencode.json
    test -f flake.nix
    test -f flake.lock
    python3 -m json.tool .opencode.json >/dev/null
    # The README copies the tables of site/data/ by hand: each repo, roadmap item, rule and planned level must be in it.
    tail -n +2 site/data/awesome.tsv site/data/roadmap.tsv site/data/origins.tsv site/data/planned.tsv | grep -v '^==>' | cut -f1 | grep . | while read -r x; do grep -qF -- "$x" README.md || { echo "quality: README.md lacks $x" >&2; exit 1; }; done
    # Every Archify figure is indexed in both READMEs, by its published page.
    for j in docs/archify/*.json; do s=$(basename "$j" .json); for r in README.md README.en.md; do grep -qF "figuras/$s.html" "$r" || { echo "quality: $r does not index the figure $s" >&2; exit 1; }; done; done
    # Each README carries its own reading time, from the plugin's calculator: just reading-time writes it.
    for r in "README.md pt-BR" "README.en.md en"; do set -- $r; want=$(python3 -I plugins/ptbr-writer/scripts/reading_time.py --lang "$2" "$1"); grep -qxF "$want" "$1" || { echo "quality: $1 reading time is not \"$want\": run just reading-time" >&2; exit 1; }; done
    echo "quality: ok"

# Create or update the GitHub labels from the tables of ISSUE_LABELS.md (needs `gh` logged in).
labels:
    grep -E '^\| `[^`]+` \| `#[0-9A-Fa-f]{6}` \|' ISSUE_LABELS.md | while IFS='|' read -r _ n c d _; do n=${n//\`/}; c=${c//\`/}; n=${n// /}; c=${c// /}; d=${d# }; d=${d% }; gh label create "$n" --color "${c#\#}" --description "$d" --force; done

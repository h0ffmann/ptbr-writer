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
        curl -sf http://127.0.0.1:8080/v1/models | jq . 2>/dev/null || true
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
site: figure-init picasso-init
    nix develop --no-update-lock-file .#site --command just --justfile site/justfile --working-directory site ci

# Build the site and serve it on http://localhost:8000.
site-serve: site
    nix develop --no-update-lock-file .#site --command nix run nixpkgs#caddy -- file-server --root site/dist --listen :8000

# Check that every picasso pin (flake.lock's gleam-website and code-graph, to-pdf.sh, tools.lock) is the forks/picasso submodule's revision.
pins:
    #!/usr/bin/env bash
    set -euo pipefail
    lock="$(jq -r '.nodes["gleam-website"].locked.rev' flake.lock)"
    sub="$(git ls-files -s forks/picasso | cut -d' ' -f2)"
    [ "$lock" = "$sub" ] || { echo "pins: flake.lock has picasso $lock, forks/picasso is $sub" >&2; exit 1; }
    pdf="$(sed -n 's/^PICASSO_REV=//p' plugins/ptbr-writer/scripts/to-pdf.sh)"
    [ "$pdf" = "$sub" ] || { echo "pins: to-pdf.sh has picasso $pdf, forks/picasso is $sub" >&2; exit 1; }
    graph="$(jq -r '.nodes["code-graph"].locked.rev' flake.lock)"
    [ "$graph" = "$sub" ] || { echo "pins: flake.lock has code-graph $graph, forks/picasso is $sub" >&2; exit 1; }
    tools="$(sed -n 's/^rev=//p' plugins/ptbr-writer/bin/tools.lock)"
    [ "$tools" = "$sub" ] || { echo "pins: tools.lock has picasso $tools, forks/picasso is $sub" >&2; exit 1; }
    echo "pins: picasso $sub"
    just tools-check

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

# The pinned picasso (forks/picasso), whose code graph the site publishes beside this repo's: once per clone.
picasso-init:
    git submodule update --init forks/picasso
    @echo "picasso-init: $(git -C forks/picasso rev-parse --short HEAD)"

# The pinned Archify (vendor/archify, sparse to archify/): once per clone. Not part of the plugin.
figure-init:
    git submodule update --init --depth 1 vendor/archify
    git -C vendor/archify sparse-checkout set --no-cone /archify/
    @echo "figure-init: $(git -C vendor/archify rev-parse --short HEAD)"

# Draw every docs/archify/<slug>.json to docs/img/<slug>.png, and its English twin docs/archify/en/<slug>.json to docs/img/en/<slug>.png: Archify validates and checks it (finalize),
# picasso's look (docs/archify/picasso.css) goes over it, headless Chrome takes the picture. Needs node,
# Chrome (CHROME names another) and the Google Fonts; writes docs/archify/figures.lock.
figure: figure-init
    #!/usr/bin/env bash
    set -euo pipefail
    mkdir -p build/figure docs/img/en
    for src in docs/archify/*.json docs/archify/en/*.json; do
        slug="${src#docs/archify/}"; slug="${slug%.json}"; out="build/figure/$slug"
        rm -rf "$out"; mkdir -p "$out"
        ARCHIFY_UPDATE_CHECK_DISABLED=1 node vendor/archify/archify/bin/archify.mjs finalize "$(jq -r .diagram_type "$src")" "$src" "$out/page.html" --out-dir "$out/receipts" --quality showcase
        read -r w h < <(grep -o '<svg viewBox="[^"]*"' "$out/page.html" | head -1 | sed -E 's/.*0 0 ([0-9.]+) ([0-9.]+)"/\1 \2/')
        crop="$(awk -F'\t' -v s="$(basename "$slug")" '$1==s {print $2}' docs/archify/crop.tsv)"; h="${crop:-$h}"
        FONTS='<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Fjalla+One&family=Inter:wght@400;600&display=block">' CSS="$(cat docs/archify/picasso.css)" W="$w" H="$h" \
            perl -0pe 's|</head>|$ENV{FONTS}<style>$ENV{CSS}</style></head>|; s|<svg viewBox="0 0 [0-9.]+ [0-9.]+"|<svg viewBox="0 0 $ENV{W} $ENV{H}"|' "$out/page.html" > "$out/frame.html"
        "${CHROME:-google-chrome}" --headless --disable-gpu --hide-scrollbars --force-dark-mode --force-device-scale-factor=2 --user-data-dir="$PWD/$out/profile" --window-size="$w,$h" --virtual-time-budget=3000 --screenshot="$PWD/docs/img/$slug.png" "file://$PWD/$out/frame.html" 2>/dev/null
        echo "figure: docs/img/$slug.png"
    done
    just _figure-lock > docs/archify/figures.lock

# The inputs of each figure (its source, the look, the crop, the Archify pin), one line per slug (en/<slug> for a twin).
_figure-lock:
    #!/usr/bin/env bash
    set -euo pipefail
    pin="$(git ls-files -s vendor/archify | cut -d' ' -f2)"
    for src in docs/archify/*.json docs/archify/en/*.json; do
        slug="${src#docs/archify/}"
        printf '%s\t%s\n' "${slug%.json}" "$(cat "$src" docs/archify/picasso.css docs/archify/crop.tsv | sha256sum | cut -c1-16)-${pin:0:7}"
    done

# Cheap and offline: each docs/archify/<slug>.json has its PNG, and both match the last `just figure`.
# Each figure has an English twin (docs/archify/en/<slug>.json, locale en) with the same drawing, only
# the words changed; README.md embeds the pt-BR PNG and README.en.md the English one, never the other.
figure-check:
    #!/usr/bin/env bash
    set -euo pipefail
    shape='{d: .diagram_type, l: [.lanes[].id], m: .mainPath, n: [.nodes[] | del(.label, .sublabel, .tag)], e: [.edges[] | del(.label)]}'
    for src in docs/archify/*.json; do
        s="$(basename "$src" .json)"; en="docs/archify/en/$s.json"
        test -s "docs/img/$s.png" || { echo "figure-check: no PNG for $src; run just figure" >&2; exit 1; }
        test -s "$en" && test -s "docs/img/en/$s.png" || { echo "figure-check: no English twin $en or its PNG; write it and run just figure" >&2; exit 1; }
        test "$(jq -r .meta.locale "$en")" = en || { echo "figure-check: $en is not meta.locale en" >&2; exit 1; }
        cmp -s <(jq -S "$shape" "$src") <(jq -S "$shape" "$en") || { echo "figure-check: $en is not the same drawing as $src (lanes, nodes or edges differ)" >&2; exit 1; }
        ! grep -qF "](docs/img/$s.png)" README.en.md || { echo "figure-check: README.en.md embeds the pt-BR docs/img/$s.png; use docs/img/en/$s.png" >&2; exit 1; }
        ! grep -qF "](docs/img/en/$s.png)" README.md || { echo "figure-check: README.md embeds the English docs/img/en/$s.png; use docs/img/$s.png" >&2; exit 1; }
        ! grep -qF "](docs/img/$s.png)" README.md || grep -qF "](docs/img/en/$s.png)" README.en.md || { echo "figure-check: README.md shows $s but README.en.md does not show docs/img/en/$s.png" >&2; exit 1; }
    done
    for en in docs/archify/en/*.json; do test -s "docs/archify/$(basename "$en")" || { echo "figure-check: $en has no pt-BR source" >&2; exit 1; }; done
    just _figure-lock | cmp -s - docs/archify/figures.lock || { echo "figure-check: a source, the look or the Archify pin changed since the PNG; run just figure" >&2; exit 1; }
    echo "figure-check: ok"

# Basic quality gate: justfile parses, required files exist, toolchain pins agree, linters self-test.
# Pick the skill and the level for a writing request: just route "reescreva este runbook".
route +request:
    bash skills/ptbr-routing/bin/run.sh "{{ request }}"

# The plugin's and the routing skill's static binaries (picasso labs/ptbr-tools) at picasso REV: build, copy, write tools.lock.
tools-fetch rev:
    #!/usr/bin/env bash
    set -euo pipefail
    lock=plugins/ptbr-writer/bin/tools.lock
    { echo "rev={{ rev }}"; echo "lab=labs/ptbr-tools"; echo "# sha256  size  path"; } > "$lock.new"
    for t in "x86_64 static" "aarch64 static-aarch64"; do
        set -- $t
        out=$(nix build "github:h0ffmann/picasso/{{ rev }}?dir=labs/ptbr-tools#$2" --no-link --print-out-paths 2>/dev/null) || { echo "tools-fetch: no $1 build, skipped" >&2; continue; }
        for d in plugins/ptbr-writer/bin/linux-$1 skills/ptbr-routing/bin/linux-$1; do
            mkdir -p "$d"; install -m 755 "$out/bin/ptbr" "$d/ptbr"
            echo "$(sha256sum "$d/ptbr" | cut -d' ' -f1)  $(stat -c %s "$d/ptbr")  $d/ptbr" >> "$lock.new"
        done
    done
    mv "$lock.new" "$lock"
    cat "$lock"

# Each committed binary matches tools.lock (the rev there is the source of truth: tools-fetch writes it, nothing else reads it).
tools-check:
    #!/usr/bin/env bash
    set -euo pipefail
    lock=plugins/ptbr-writer/bin/tools.lock
    grep -qE '^rev=[0-9a-f]{40}$' "$lock" || { echo "tools-check: $lock has no rev" >&2; exit 1; }
    n=0
    while read -r sha size path; do
        case $sha in rev=*|lab=*|'#') continue;; esac
        [ "$(sha256sum "$path" | cut -d' ' -f1)" = "$sha" ] && [ "$(stat -c %s "$path")" = "$size" ] || { echo "tools-check: $path differs from tools.lock" >&2; exit 1; }
        n=$((n+1))
    done < "$lock"
    [ "$n" -gt 0 ] || { echo "tools-check: tools.lock lists no binary" >&2; exit 1; }
    echo "tools-check: $n binaries match $(sed -n 's/^rev=//p' "$lock" | cut -c1-9)"

# Check the Claude Code plugin: manifests parse, every skill link resolves, the version has a CHANGELOG entry.
plugin-check:
    #!/usr/bin/env bash
    set -euo pipefail
    jq . .claude-plugin/marketplace.json >/dev/null
    v=$(jq -r .version plugins/ptbr-writer/.claude-plugin/plugin.json)
    [[ $v =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo "plugin-check: version $v is not X.Y.Z" >&2; exit 1; }
    grep -q "^## v$v " CHANGELOG.md || { echo "plugin-check: CHANGELOG.md has no v$v" >&2; exit 1; }
    for f in plugins/ptbr-writer/skills/*/SKILL.md; do test -e "$f" || { echo "plugin-check: $f is a broken link" >&2; exit 1; }; done
    bash skills/ptbr-routing/scripts/session-check.sh --self-test
    bash plugins/ptbr-writer/hooks/ask-levels.sh --self-test
    bash plugins/ptbr-writer/bin/run.sh reading-time --self-test
    bash scripts/opencode-smoke.sh
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
    now=$(git show "origin/main:$json" | jq -r .version)
    if [ "$now" != "$v" ]; then
        git switch -q -c "chore/release-v$v" origin/main
        # A release with nothing new for the plugin would reach no one: refuse it.
        due=$(just release-due | tail -n +2 || true)
        [ -n "$due" ] || { git switch -q -; git branch -q -D "chore/release-v$v"; echo "release: nothing in the plugin changed since the last tag" >&2; exit 1; }
        jq --arg v "$v" '.version = $v' "$json" > "$json.new" && mv "$json.new" "$json"
        files=$(sed 's/^ *//; s/ *$//; s/.*/`&`/' <<<"$due" | paste -sd, - | sed 's/,/, /g')
        awk -v v="$v" -v day="$(date -u +%F)" -v files="$files" '!done && /^## / {print "## v" v " — " day "\n\n- (descreva a release) mudou: " files "\n"; done=1} {print}' CHANGELOG.md > CHANGELOG.new && mv CHANGELOG.new CHANGELOG.md
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
    test "{{ tag }}" = "v$(jq -r .version plugins/ptbr-writer/.claude-plugin/plugin.json)"

# Write each README's reading time (the plugin's calculator) under its language line.
reading-time:
    #!/usr/bin/env bash
    set -euo pipefail
    for r in "README.md pt-BR" "README.en.md en"; do
        set -- $r
        badge=$(just _reading-badge "$1" "$2")
        alt=$(sed -E 's/^\[!\[([^:]+):.*/\1/' <<<"$badge")
        sed -i -E '/^(Leitura estimada|Estimated reading time):/d' "$1"
        if grep -q "^\[!\[$alt:" "$1"; then
            grep -v "^\[!\[$alt:" "$1" | awk -v b="$badge" '{print} /img.shields.io\/badge\/license/{print b}' > "$1.tmp"
        else
            awk -v b="$badge" '{print} /img.shields.io\/badge\/license/{print b}' "$1" > "$1.tmp"
        fi
        mv "$1.tmp" "$1"
        echo "$1: $badge"
    done

# The reading-time badge of one README: the calculator's minutes in a shields.io badge, linked to the section that explains it.
_reading-badge file lang:
    #!/usr/bin/env bash
    set -euo pipefail
    line=$(bash plugins/ptbr-writer/bin/run.sh reading-time --lang "{{ lang }}" "{{ file }}")
    min=$(sed -E 's/^[^:]+: ([^(]*[^ (]) *\(.*$/\1/' <<<"$line")
    enc=$(sed -e 's/ /%20/g' -e 's/</%3C/g' <<<"$min")
    if [ "{{ lang }}" = en ]; then
        echo "[![estimated reading time: $min](https://img.shields.io/badge/reading%20time-$enc-1682D4?style=flat-square)](#reading-time)"
    else
        echo "[![leitura estimada: $min](https://img.shields.io/badge/leitura%20estimada-$enc-1682D4?style=flat-square)](#tempo-de-leitura)"
    fi

quality: pins saramago-check plugin-check figure-check ext-test
    bash skills/ptbr-routing/bin/run.sh --self-test
    bash skills/ptbr-routing/bin/run.sh --check
    just --list >/dev/null
    test -f .opencode.json
    test -f flake.nix
    test -f flake.lock
    jq . .opencode.json >/dev/null
    # The README copies the tables of site/data/ by hand: each repo, roadmap item, rule and planned level must be in it.
    tail -n +2 site/data/awesome.tsv site/data/roadmap.tsv site/data/origins.tsv site/data/planned.tsv | grep -v '^==>' | cut -f1 | grep . | while read -r x; do grep -qF -- "$x" README.md || { echo "quality: README.md lacks $x" >&2; exit 1; }; done
    # Every Archify figure is indexed in both READMEs, by its published page.
    for j in docs/archify/*.json; do s=$(basename "$j" .json); for r in README.md README.en.md; do grep -qF "figuras/$s.html" "$r" || { echo "quality: $r does not index the figure $s" >&2; exit 1; }; done; done
    # Each README carries its own reading time, from the plugin's calculator: just reading-time writes it.
    for r in "README.md pt-BR" "README.en.md en"; do set -- $r; want=$(just _reading-badge "$1" "$2"); grep -qxF "$want" "$1" || { echo "quality: $1 reading-time badge is not current: run just reading-time" >&2; exit 1; }; done
    echo "quality: ok"

# Create or update the GitHub labels from the tables of ISSUE_LABELS.md (needs `gh` logged in).
labels:
    grep -E '^\| `[^`]+` \| `#[0-9A-Fa-f]{6}` \|' ISSUE_LABELS.md | while IFS='|' read -r _ n c d _; do n=${n//\`/}; c=${c//\`/}; n=${n// /}; c=${c// /}; d=${d# }; d=${d% }; gh label create "$n" --color "${c#\#}" --description "$d" --force; done

# Chrome extension: build into extension/dist.
ext-build:
    cd extension && npm ci && node build.mjs

# The same extension for Firefox (128+), into extension/dist-firefox.
ext-build-firefox:
    cd extension && npm ci && node build.mjs --firefox

# Serve Qwen with Ollama for the extension's provider "Ollama + Qwen (local)", at 127.0.0.1:11434.
# Ollama answers 403 to an extension's origin unless started with OLLAMA_ORIGINS; this sets it.
# Requires: ollama on PATH; qwen3.6:27b is the model of opencode-qwen (~24 GB GPU), qwen3:8b is lighter.
ext-ollama model="qwen3.6:27b":
    #!/usr/bin/env bash
    set -euo pipefail
    command -v ollama >/dev/null || { echo "ext-ollama: no ollama on PATH (https://ollama.com/download)" >&2; exit 1; }
    export OLLAMA_HOST=127.0.0.1:11434 OLLAMA_ORIGINS="chrome-extension://*,moz-extension://*"
    api="http://$OLLAMA_HOST/api/tags"
    pull() { ollama list | awk 'NR>1 {print $1}' | grep -qx "{{ model }}" || ollama pull "{{ model }}"; }
    if curl -sf -m 2 "$api" >/dev/null; then
        # One already serves: it must accept the extension's origin, or every check is a 403.
        code=$(curl -s -o /dev/null -w '%{http_code}' -m 2 -H 'Origin: chrome-extension://ptbr-writer' "$api")
        if [ "$code" != 200 ]; then
            echo "ext-ollama: the Ollama at $OLLAMA_HOST refuses extensions (HTTP $code). Stop it, or for the" >&2
            echo "systemd service run 'sudo systemctl edit ollama', add under [Service]" >&2
            echo "  Environment=\"OLLAMA_ORIGINS=$OLLAMA_ORIGINS\"" >&2
            echo "then 'sudo systemctl restart ollama', and run this again." >&2
            exit 1
        fi
        pull
        echo "ext-ollama: {{ model }} ready at $OLLAMA_HOST"
        exit 0
    fi
    log="${TMPDIR:-/tmp}/ollama-ptbr-writer.log"
    echo "ollama log: $log"
    ollama serve >"$log" 2>&1 &
    srv=$!
    trap 'kill $srv' EXIT
    until curl -sf -m 1 "$api" >/dev/null; do
        kill -0 $srv 2>/dev/null || { tail -n 20 "$log" >&2; echo "ollama exited" >&2; exit 1; }
        sleep 1
    done
    pull
    echo "ext-ollama: serving {{ model }} at $OLLAMA_HOST for the extension; Ctrl-C stops it"
    wait $srv

# Chrome extension: type check (tsconfig has noEmit) and unit tests.
ext-test: ext-build
    cd extension && npx tsc -p . && npx vitest run

# Chrome extension: end-to-end tests (Playwright) against extension/dist-test.
ext-e2e:
    cd extension && npx playwright install chromium && node build.mjs --test && npx playwright test

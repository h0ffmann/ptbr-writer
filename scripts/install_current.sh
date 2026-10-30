#!/usr/bin/env bash
# install_current — install the gramatico plugin from the tip of main, no release needed, in the
# user's Claude Code config. Inside a session: `! bash scripts/install_current.sh`, or from anywhere:
#   ! curl -fsSL https://raw.githubusercontent.com/piabrota/gramatico/main/scripts/install_current.sh | bash
# Then /reload-plugins (or a restart) loads it. PTBR_REF=<branch or tag> installs another ref.
# main keeps the last release's version number, so a plain `claude plugin update` would see nothing
# new: this removes the old install and the marketplace, then installs again.
set -euo pipefail

repo=piabrota/gramatico
ref=${PTBR_REF:-main}
command -v claude >/dev/null || { echo "install_current: claude not on PATH" >&2; exit 1; }
sha=$(git ls-remote "https://github.com/$repo" "refs/heads/$ref" "refs/tags/$ref" 2>/dev/null | head -1 | cut -c1-12)
[ -n "$sha" ] || { echo "install_current: no branch or tag $ref in $repo" >&2; exit 1; }

claude plugin uninstall gramatico@gramatico >/dev/null 2>&1 || true
claude plugin marketplace remove gramatico >/dev/null 2>&1 || true
claude plugin install gramatico --marketplace "$repo#$ref" >/dev/null
claude plugin details gramatico@gramatico | grep -E '^ *Skills' || true
echo "install_current: gramatico from $repo#$ref ($sha); run /reload-plugins or restart Claude Code"

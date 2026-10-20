#!/usr/bin/env bash
# ask-levels — UserPromptSubmit hook: a request that names ptbr (`ptbr`, `/ptbr`, `ptbr-writter`)
# and no level gets one instruction: ask the levels and the formats (.txt, .md, .pdf, notes; any
# mix) with AskUserQuestion, then write one version per level in each format picked. The levels come from the plugin's SKILL.md, so a new level shows up by itself.
# Any other request: no output, no context. Needs python3 to read the prompt; without it, silent.
#
#   ask-levels.sh < hook-input.json
#   ask-levels.sh --self-test
set -euo pipefail

root="${CLAUDE_PLUGIN_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"

run() {
  command -v python3 >/dev/null || return 0
  python3 -I -c '
import json, re, sys, unicodedata

COMPOSES = "smart-note"  # combines with another level: its own question, not one of the levels
MAX = 4                   # AskUserQuestion takes at most 4 options; "Other" is added by itself

def fold(s):
    return "".join(c for c in unicodedata.normalize("NFD", s.lower()) if not unicodedata.combining(c))

try:
    prompt = json.load(sys.stdin).get("prompt", "")
except ValueError:
    sys.exit(0)
# ptbr as a word or a command, not inside a path or another name (skills/ptbr/, ptbr-routing).
if not re.search(r"(?<![\w/.-])/?ptbr(-writter)?(?![\w/-])", prompt, re.I):
    sys.exit(0)
try:
    text = open(sys.argv[1], encoding="utf-8").read()
except OSError:
    sys.exit(0)
section = text.split("## Níveis de rigor", 1)[-1].split("\n## ", 1)[0]
levels = re.findall(r"^- `([^`]+)`: (.+)$", section, re.M)
if not levels:
    sys.exit(0)
words = fold(prompt)
if any(re.search(rf"(?<!\w){re.escape(fold(n))}(?!\w)", words) for n, _ in levels):
    sys.exit(0)  # the request already names its level

opts = [(n, u.rstrip(".")) for n, u in levels if n != COMPOSES][:MAX]
notes = any(n == COMPOSES for n, _ in levels)
fmts = ["`.txt` (texto puro)", "`.md` (Markdown)", f"`.pdf` (do .md, com `bash {sys.argv[2]} <arquivo>.md`)"]
if notes:
    fmts.append(f"`notes` (o nível {COMPOSES}: notas indexáveis, uma ideia por nota, com tags e links)")
lines = [
    "ptbr-writter: o pedido não diz o nível de escrita. Antes de escrever, chame AskUserQuestion com duas perguntas, as duas multiSelect:",
    "1. header \"Nível\", \"Em quais níveis?\", opções: " + "; ".join(f"{n} ({u})" for n, u in opts) + ".",
    "2. header \"Formato\", \"Em quais formatos?\", opções: " + "; ".join(fmts) + ".",
    "Depois, carregue a skill ptbr-writter e escreva uma versão por nível marcado. Grave cada versão em cada formato marcado, "
    "no diretório atual: `<slug>.<nível>.txt`, `.md`, `.pdf`"
    + (f"; com `notes`, as notas pelas regras de {COMPOSES} (`{COMPOSES} + <nível>`)" if notes else "")
    + ". Sem formato marcado, só a resposta. Na resposta, cada versão sob `## <nível>` e a lista dos arquivos gravados. Regra: o conteúdo de cada .txt e cada .md gravado aparece inteiro na resposta, num bloco ```text ou ```markdown, nunca só o caminho; .pdf e notes, o caminho basta. "
    "Se o script do .pdf sair com 3, ele precisa baixar ferramentas: diga o tamanho que ele informa, pergunte se pode baixar e, com o sim, rode de novo com `--download`; sem o sim, ou se falhar, entregue o .md e diga por quê.",
    "Sem AskUserQuestion (sessão não interativa): não pergunte nem grave arquivo; use o nível que a skill classificar, declare-o e responda sob `## <nível>`.",
]
print(json.dumps({"hookSpecificOutput": {"hookEventName": "UserPromptSubmit", "additionalContext": "\n".join(lines)}}, ensure_ascii=False))
' "${PTBR_SKILL_MD:-$root/skills/ptbr-writter/SKILL.md}" "$root/scripts/to-pdf.sh"
}

self_test() {
  local tmp out fails=0
  tmp=$(mktemp -d); trap 'rm -rf "$tmp"' RETURN
  printf '# t\n\n## Níveis de rigor\n\n- `estrito`: runbooks.\n- `leve`: comunicados.\n- `científico`: artigos.\n- `smart-note`: notas.\n\n## Regras\n' >"$tmp/SKILL.md"
  export PTBR_SKILL_MD="$tmp/SKILL.md"
  ask() { run <<<"{\"prompt\": \"$1\", \"cwd\": \"/home/x/ptbr-writter\"}"; }
  check() { if eval "$2"; then echo "  ok   $1"; else echo "  FAIL $1: $out"; fails=$((fails + 1)); fi; }
  out=$(ask "ptbr: reescreva este texto")
  check "ptbr asks" '[[ $out == *AskUserQuestion* && $out == *"estrito (runbooks)"* && $out == *"leve (comunicados)"* ]]'
  check "smart-note is the notes format, not a level" '[[ $out == *"\`notes\` (o nível smart-note"* && $out != *"smart-note (notas); "* ]]'
  check "formats: txt, md, pdf with its script" '[[ $out == *"\`.txt\`"* && $out == *"\`.md\`"* && $out == *"to-pdf.sh <arquivo>.md"* ]]'
  check ".txt and .md are always printed" '[[ $out == *"o conteúdo de cada .txt e cada .md gravado aparece inteiro na resposta"* ]]'
  check "both questions are multiSelect" '[[ $out == *"as duas multiSelect"* ]]'
  out=$(ask "/ptbr-writter:ptbr revise isto"); check "/ptbr-writter command asks" '[ -n "$out" ]'
  out=$(ask "use PTBR aqui"); check "case does not matter" '[ -n "$out" ]'
  out=$(ask "ptbr estrito: reescreva"); check "a named level does not ask" '[ -z "$out" ]'
  out=$(ask "ptbr, versao cientifico"); check "a level named without accents does not ask" '[ -z "$out" ]'
  out=$(ask "abra skills/ptbr/SKILL.md"); check "a path does not ask" '[ -z "$out" ]'
  out=$(ask "rode o ptbr-routing"); check "another name does not ask" '[ -z "$out" ]'
  out=$(ask "reescreva este runbook"); check "no ptbr, no output (cwd ignored)" '[ -z "$out" ]'
  out=$(run <<<'not json'); check "bad input is silent" '[ -z "$out" ]'
  out=$(ask "ptbr")
  check "output is valid hook JSON" 'python3 -c "import json,sys; d=json.loads(sys.argv[1]); assert d[\"hookSpecificOutput\"][\"hookEventName\"]==\"UserPromptSubmit\"" "$out"'
  [ "$fails" -eq 0 ] && { echo "ask-levels self-test: ok"; return 0; }
  echo "ask-levels self-test: $fails failure(s)" >&2; return 1
}

case "${1-}" in
  --self-test) self_test ;;
  *) run ;;
esac

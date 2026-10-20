#!/usr/bin/env python3
"""Pick the skill and the level for a writing request, before any skill is loaded.

route "<pedido>"     one line: `<skill> · <nível>`, from routes.tsv beside the skill
route --check        routes.tsv against SKILL.md: one row per level, every skill exists
route --self-test    the answers for a few known requests

Stdlib only: a repo that vendors skills/ptbr-routing/ into .claude/skills/ runs it with no Nix.
"""

from __future__ import annotations

import re
import sys
import unicodedata
from pathlib import Path

HERE = Path(__file__).resolve().parent.parent
ROOT = HERE.parent.parent
COMPOSES = "smart-note"


def fold(text: str) -> str:
    # "Relatório" and "relatorio" are the same word to a person typing fast.
    nfd = unicodedata.normalize("NFD", text.lower())
    return "".join(c for c in nfd if not unicodedata.combining(c))


def routes(tsv: str) -> list[tuple[str, str, list[str]]]:
    rows = []
    for line in tsv.splitlines()[1:]:
        if line.strip():
            skill, level, words = line.split("\t")
            rows.append((skill, level, [fold(w.strip()) for w in words.split(",") if w.strip()]))
    return rows


def score(request: str, words: list[str]) -> int:
    text = fold(request)
    return sum(1 for w in words if re.search(rf"(?<!\w){re.escape(w)}(?!\w)", text))


def route(request: str, rows: list[tuple[str, str, list[str]]]) -> str:
    scored = [(score(request, w), s, lv) for s, lv, w in rows]
    note = any(n for n, _, lv in scored if lv == COMPOSES)
    rest = [x for x in scored if x[2] != COMPOSES and x[0]]
    best = max((n for n, _, _ in rest), default=0)
    top = [(s, lv) for n, s, lv in rest if n == best]
    if not top:
        return "ptbr-writter · smart-note" if note else "ptbr-writter · sem nível: a skill classifica e declara"
    line = " | ".join(f"{s} · {COMPOSES + ' + ' if note else ''}{lv}" for s, lv in top)
    return line + (" (empate: escolha um e declare)" if len(top) > 1 else "")


def levels(skill_md: str) -> list[str]:
    section = skill_md.split("## Níveis de rigor", 1)[-1].split("\n## ", 1)[0]
    return re.findall(r"^- `([^`]+)`:", section, re.M)


def check() -> list[str]:
    rows = routes((HERE / "routes.tsv").read_text())
    have = [lv for _, lv, _ in rows]
    want = levels((ROOT / "SKILL.md").read_text())
    errors = [f"routes.tsv: no row for level {lv}" for lv in want if lv not in have]
    errors += [f"routes.tsv: {lv} is not a level in SKILL.md" for lv in have if lv not in want]
    errors += [f"routes.tsv: {lv} appears {have.count(lv)} times" for lv in set(have) if have.count(lv) > 1]
    skills = {"ptbr-writter"} | {p.parent.name for p in ROOT.glob("skills/*/SKILL.md")}
    errors += [f"routes.tsv: no skill {s}" for s, _, _ in rows if s not in skills]
    return errors


def self_test() -> None:
    rows = routes((HERE / "routes.tsv").read_text())
    assert route("reescreva este runbook de incidente", rows) == "ptbr-writter · estrito"
    assert route("revise o RELATORIO e as citações", rows) == "cientifico · científico"
    assert route("transforme em notas do Obsidian este comunicado", rows) == "ptbr-writter · smart-note + leve"
    assert route("guarde no meu segundo cérebro", rows) == "ptbr-writter · smart-note"
    assert route("melhore isto", rows).startswith("ptbr-writter · sem nível")
    assert route("o comunicado sobre o runbook", rows).endswith("(empate: escolha um e declare)")
    assert route("um postmortem", rows).startswith("ptbr-writter · sem nível")  # "post" is a word, not a prefix
    assert levels("## Níveis de rigor\n\n- `a`: x.\n- `b`: y.\n\n## Regras\n- `c`: z.\n") == ["a", "b"]
    print("route self-test: ok")


def main(argv: list[str]) -> int:
    if argv == ["--self-test"]:
        self_test()
        return 0
    if argv == ["--check"]:
        errors = check()
        print("\n".join(errors) or "route check: ok", file=sys.stderr if errors else sys.stdout)
        return 1 if errors else 0
    if not argv or argv[0].startswith("-"):
        print(__doc__.strip().split("\n\n")[1], file=sys.stderr)
        return 2
    print(route(" ".join(argv), routes((HERE / "routes.tsv").read_text())))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

"""ask-levels: the UserPromptSubmit hook's logic, run by ask-levels.sh.

A request that starts with `ptbr` (`ptbr: ...`, `ptbr ...`, `/ptbr`, `/ptbr-writer:...`) gets one
instruction for Claude; any other request gets nothing, so the hook costs no context.

The choices live in ptbr-writer.toml, read in this order, each overriding the one before:
the plugin's ptbr-writer.default.toml, ~/.config/ptbr-writer/ptbr-writer.toml, then
./ptbr-writer.toml; the request itself overrides all. With no config file, Claude asks the
levels, the formats and the folder once and writes ./ptbr-writer.toml. `ptbr reconfigurar`
asks again.

    python3 -I ask_levels.py SKILL.md TO_PDF DEFAULTS < hook-input.json
    python3 -I ask_levels.py --self-test
"""

import json
import os
import re
import sys
import unicodedata

try:
    import tomllib
except ImportError:  # python < 3.11: Claude reads the file itself
    tomllib = None

COMPOSES = "smart-note"  # combines with another level: the `notes` format, not one of the levels
MAX = 4  # AskUserQuestion takes at most 4 options; "Other" is added by itself
FORMATS = ("txt", "md", "pdf", "notes")
LANGUAGES = ("pt-BR", "asd-ste100")  # pt-BR is the default; ASD-STE100 is Simplified Technical English
TRIGGER = re.compile(r"^\s*/?ptbr(-writer)?(:\S*)?(?=[\s:,.]|$)", re.I)
RECONFIG = re.compile(r"^\s*/?ptbr(-writer)?\s*:?\s*(reconfigur\w*|config\w*)\b", re.I)
NAME = "ptbr-writer-<AAAAMMDDThhmmZ>-<objetivo>-<nível>.<ext>"


def fold(s):
    return "".join(c for c in unicodedata.normalize("NFD", s.lower()) if not unicodedata.combining(c))


def skill_levels(text):
    section = text.split("## Níveis de rigor", 1)[-1].split("\n## ", 1)[0]
    return [(n, u.rstrip(".")) for n, u in re.findall(r"^- `([^`]+)`: (.+)$", section, re.M)]


def load(path):
    """The table in path, {} when absent, None when it cannot be read (no tomllib, bad TOML)."""
    if not os.path.isfile(path):
        return {}
    if tomllib is None:
        return None
    try:
        with open(path, "rb") as f:
            return tomllib.load(f)
    except (OSError, tomllib.TOMLDecodeError):
        return None


def clean(cfg, levels, problems, where):
    """Keep only values the plugin knows; a folder must stay inside the session's root."""
    out = {}
    names = {n for n, _ in levels if n != COMPOSES}
    for key, allowed in (
        ("levels", names),
        ("formats", set(FORMATS)),
        ("print", set(FORMATS)),
        ("languages", set(LANGUAGES)),
    ):
        if key in cfg:
            vals = cfg[key] if isinstance(cfg[key], list) else []
            bad = [v for v in vals if v not in allowed]
            if bad or not isinstance(cfg[key], list):
                problems.append(f"{where}: `{key}` tem valores desconhecidos {bad or cfg[key]}; ignorados")
            out[key] = [v for v in vals if v in allowed]
    if "folder" in cfg:
        f = cfg["folder"]
        if isinstance(f, str) and f and not os.path.isabs(f) and ".." not in f.replace("\\", "/").split("/"):
            out["folder"] = f
        else:
            problems.append(f"{where}: `folder` precisa ser relativa à raiz, sem `..`; ignorada")
    return out


def instruction(prompt, skill_md, to_pdf, defaults_path, home, cwd):
    if not TRIGGER.search(prompt):
        return None
    try:
        levels = skill_levels(open(skill_md, encoding="utf-8").read())
    except OSError:
        return None
    if not levels:
        return None
    project = os.path.join(cwd, "ptbr-writer.toml")
    user = os.path.join(home, ".config", "ptbr-writer", "ptbr-writer.toml")
    problems, cfg, found = [], {}, []
    for path, where in ((defaults_path, "padrões"), (user, user), (project, "./ptbr-writer.toml")):
        table = load(path)
        if table is None:
            problems.append(f"{where}: não consegui ler; leia o arquivo você mesmo e siga-o")
            if path != defaults_path:
                found.append(where)
            continue
        if table and path != defaults_path:
            found.append(where)
        cfg.update(clean(table, levels, problems, where))
    named = [n for n, _ in levels if re.search(rf"(?<!\w){re.escape(fold(n))}(?!\w)", fold(prompt))]
    ste = re.search(r"(?<!\w)(asd-?)?ste-?100(?!\w)", prompt, re.I) is not None

    opts = [(n, u) for n, u in levels if n != COMPOSES][:MAX]
    notes = any(n == COMPOSES for n, _ in levels)
    fmts = ["`txt` (texto puro)", "`md` (Markdown)", f"`pdf` (do .md, com `bash {to_pdf} <arquivo>.md`)"]
    if notes:
        fmts.append(f"`notes` (o nível {COMPOSES}: notas indexáveis, uma ideia por nota, com tags e links)")
    write = (
        f"Grave cada versão em cada formato, na pasta configurada (crie se faltar), com o nome `{NAME}`: "
        "o carimbo em UTC, o mesmo para todos os arquivos do pedido; o objetivo em 2 a 4 palavras, minúsculas sem acento, "
        "unidas por hífen; o nível sem acento; a versão em ASD-STE100 leva `-ste100` antes da extensão"
        + (f"; com `notes`, as notas pelas regras de {COMPOSES} (`{COMPOSES} + <nível>`)" if notes else "")
        + ". Uma versão por nível em cada idioma de `languages`: `pt-BR` pelas regras da skill; `asd-ste100` em inglês técnico "
        "simplificado (ASD-STE100: vocabulário controlado, voz ativa, frases curtas, uma instrução por frase), com o mesmo conteúdo, "
        "sem inventar nada. Na resposta, cada versão sob `## <nível>` (ou `## <nível> · ASD-STE100`) e a lista dos arquivos gravados; o conteúdo dos formatos em `print` "
        "aparece inteiro na resposta, num bloco ```text ou ```markdown, nunca só o caminho. Se o script do pdf sair com 3, "
        "ele precisa baixar ferramentas: diga o tamanho que ele informa, pergunte se pode baixar e, com o sim, rode de novo "
        "com `--download`; sem o sim, ou se falhar, entregue o .md e diga por quê."
    )
    ask = [
        "Chame AskUserQuestion com quatro perguntas numa chamada só:",
        "1. header \"Nível\", multiSelect, \"Em quais níveis?\", opções: " + "; ".join(f"{n} ({u})" for n, u in opts) + ".",
        "2. header \"Formato\", multiSelect, \"Em quais formatos?\", opções: " + "; ".join(fmts) + ".",
        "3. header \"Pasta\", uma escolha, \"Onde gravar os arquivos?\", opções nesta ordem: raiz (Recommended), "
        "a pasta da sessão, o padrão; docs/. Outra pasta vem pelo Other.",
        "4. header \"Idioma\", multiSelect, \"Em quais idiomas?\", opções nesta ordem: pt-BR (Recommended), o padrão; "
        "ASD-STE100 (inglês técnico simplificado). Sem marcação, pt-BR.",
        "Depois grave as respostas em ./ptbr-writer.toml, com `levels`, `formats`, `folder` (\".\" para a raiz), "
        "`languages` (\"pt-BR\" e/ou \"asd-ste100\") e "
        f"`print = {json.dumps(cfg.get('print', ['txt', 'md']))}`, uma linha de comentário dizendo que o arquivo "
        "pode ser editado ou refeito com `ptbr reconfigurar`, e diga isso à pessoa numa frase.",
    ]
    head = "ptbr-writer: um pedido de escrita com o plugin."
    no_ask = "Sem AskUserQuestion (sessão não interativa): não pergunte nem grave arquivo; use o nível que a skill classificar, declare-o e responda sob `## <nível>`."
    lines = [head]
    if RECONFIG.search(prompt):
        lines += ["A pessoa pediu para reconfigurar."] + ask + ["Não escreva texto agora, só a configuração.", no_ask]
    elif found or named or ste:
        use = dict(cfg)
        if named:
            use["levels"] = [n for n in named if n != COMPOSES]
            if COMPOSES in named and "notes" not in use.get("formats", []):
                use["formats"] = use.get("formats", []) + ["notes"]
        if ste and "asd-ste100" not in use.get("languages", []):
            use["languages"] = use.get("languages", ["pt-BR"]) + ["asd-ste100"]
        lines.append(
            "Configuração em uso (" + (", ".join(found) if found else "padrões") + "; o pedido vence): "
            + json.dumps({k: use.get(k) for k in ("levels", "formats", "folder", "print", "languages")}, ensure_ascii=False)
            + ". Não pergunte nada disso."
        )
        lines.append(
            "Carregue a skill ptbr-writer e escreva uma versão por nível de `levels` (vazio: a skill classifica e declara o nível) "
            "em cada idioma de `languages`. "
            + ("Sem formatos: só a resposta, sem arquivo; com `asd-ste100`, a versão em inglês técnico simplificado sob `## <nível> · ASD-STE100`. " if not use.get("formats") else write)
        )
    else:
        lines += ["Não há ptbr-writer.toml ainda: esta é a única vez que se pergunta."] + ask
        lines.append("Então carregue a skill ptbr-writer e escreva uma versão por nível marcado. " + write)
        lines.append(no_ask)
    lines += [f"Aviso da configuração: {p}." for p in problems]
    return {"hookSpecificOutput": {"hookEventName": "UserPromptSubmit", "additionalContext": "\n".join(lines)}}


def self_test():
    import tempfile

    with tempfile.TemporaryDirectory() as tmp:
        skill = os.path.join(tmp, "SKILL.md")
        with open(skill, "w", encoding="utf-8") as f:
            f.write("# t\n\n## Níveis de rigor\n\n- `estrito`: runbooks.\n- `leve`: comunicados.\n- `científico`: artigos.\n- `smart-note`: notas.\n\n## Regras\n")
        defaults = os.path.join(tmp, "defaults.toml")
        with open(defaults, "w", encoding="utf-8") as f:
            f.write('levels = []\nformats = []\nfolder = "."\nprint = ["txt", "md"]\nlanguages = ["pt-BR"]\n')
        home, cwd = os.path.join(tmp, "home"), os.path.join(tmp, "proj")
        os.makedirs(os.path.join(home, ".config", "ptbr-writer"))
        os.makedirs(cwd)

        def ask(p):
            out = instruction(p, skill, "/p/to-pdf.sh", defaults, home, cwd)
            return out and out["hookSpecificOutput"]["additionalContext"]

        def project(text):
            with open(os.path.join(cwd, "ptbr-writer.toml"), "w", encoding="utf-8") as f:
                f.write(text)

        checks = []

        def check(name, ok):
            checks.append((name, bool(ok)))

        out = ask("ptbr: reescreva este texto")
        check("no config: asks once, four questions", out and "única vez" in out and "quatro perguntas" in out)
        check("no config: writes ./ptbr-writer.toml", out and "grave as respostas em ./ptbr-writer.toml" in out)
        check("notes is a format, smart-note is not a level option", out and "`notes` (o nível smart-note" in out and "smart-note (notas)" not in out)
        check("pdf points at to-pdf", out and "bash /p/to-pdf.sh <arquivo>.md" in out)
        check("folder question: raiz first, docs/ offered", out and "raiz (Recommended)" in out and "docs/" in out)
        check("language question: pt-BR default, ASD-STE100 offered, multiSelect", out and "quatro perguntas" in out and "header \"Idioma\", multiSelect" in out and "pt-BR (Recommended)" in out and "ASD-STE100 (inglês técnico simplificado)" in out)
        check("languages saved in the config", out and "`languages`" in out)
        check("non-interactive fallback", out and "Sem AskUserQuestion" in out)
        for p in ("/ptbr revise isto", "/ptbr-writer:ptbr revise", "PTBR reescreva", "ptbr"):
            check(f"trigger: {p!r}", ask(p))
        for p in ("for ptbr-writer", "abra skills/ptbr/SKILL.md", "rode o ptbr-routing", "reescreva este runbook", "o plugin ptbr é bom"):
            check(f"no trigger: {p!r}", ask(p) is None)
        check("bad input is silent", instruction("", skill, "x", defaults, home, cwd) is None)

        project('levels = ["estrito"]\nformats = ["md", "pdf"]\nfolder = "docs"\n')
        out = ask("ptbr: reescreva")
        check("config: no question", out and "AskUserQuestion com quatro" not in out and "Não pergunte" in out)
        check("config: values in use", out and '"levels": ["estrito"]' in out and '"folder": "docs"' in out)
        check("config: pt-BR is the default language", out and '"languages": ["pt-BR"]' in out)
        out = ask("ptbr ste100: reescreva")
        check("ste100 in the request adds ASD-STE100", out and '"languages": ["pt-BR", "asd-ste100"]' in out and "-ste100" in out)
        check("config: file name pattern", out and NAME in out)
        out = ask("ptbr leve: reescreva")
        check("the request overrides the config", out and '"levels": ["leve"]' in out)
        out = ask("ptbr reconfigurar")
        check("reconfigure asks again", out and "reconfigurar" in out and "quatro perguntas" in out)

        with open(os.path.join(home, ".config", "ptbr-writer", "ptbr-writer.toml"), "w", encoding="utf-8") as f:
            f.write('formats = ["txt"]\nfolder = "notas"\n')
        project('folder = "docs"\n')
        out = ask("ptbr: x")
        check("project overrides user, user overrides defaults", out and '"folder": "docs"' in out and '"formats": ["txt"]' in out)
        project('folder = "../fora"\nformats = ["exe"]\n')
        out = ask("ptbr: x")
        check("a folder outside the root is refused", out and "sem `..`" in out and '"folder": "notas"' in out)
        check("an unknown format is dropped", out and "`formats` tem valores desconhecidos" in out)
        project("levels = [\n")
        out = ask("ptbr: x")
        if tomllib is not None:
            check("broken TOML is reported", out and "não consegui ler" in out)
        out = json.dumps(instruction("ptbr", skill, "x", defaults, home, cwd))
        check("valid hook JSON", json.loads(out)["hookSpecificOutput"]["hookEventName"] == "UserPromptSubmit")

    fails = [n for n, ok in checks if not ok]
    for n, ok in checks:
        print(("  ok   " if ok else "  FAIL ") + n)
    print("ask-levels self-test: " + ("ok" if not fails else f"{len(fails)} failure(s)"), file=sys.stdout if not fails else sys.stderr)
    return 1 if fails else 0


def main(argv):
    if argv[1:] == ["--self-test"]:
        return self_test()
    try:
        prompt = json.load(sys.stdin).get("prompt", "")
    except ValueError:
        return 0
    out = instruction(prompt, argv[1], argv[2], argv[3], os.path.expanduser("~"), os.getcwd())
    if out:
        print(json.dumps(out, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))

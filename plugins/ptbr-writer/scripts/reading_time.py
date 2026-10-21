"""reading_time: an honest, deterministic estimate of how long a person takes to read a text.

    python3 -I reading_time.py [--lang pt-BR|en] [--json] [FILE]     (stdin when no FILE)
    python3 -I reading_time.py --self-test

Prints one line, e.g. `Leitura estimada: ~3 min (612 palavras, 4 links, 2 blocos de código)`.

Formula (seconds, then rounded UP to whole minutes; 60 s or less prints "< 1 min"):

    seconds = words * complexity / 238 * 60  +  code_lines * 10  +  links * 5

- words: prose only. Fenced code, URLs, Markdown syntax, the front matter and the reading-time
  line itself are left out; inline `code` counts as words.
- 238 words per minute: silent reading of English non-fiction by adults, the mean of the
  meta-analysis of Brysbaert (2019), Journal of Memory and Language, DOI 10.1016/j.jml.2019.104047.
  The paper gives no Portuguese rate we could read, so pt-BR uses the same 238, unscaled (unverified).
- complexity: technical prose is slower. 1.0, plus 0.15 if the mean sentence has more than 20
  words, plus 0.15 if more than 25% of the words have 10 letters or more. Bounds: 1.0 to 1.3.
  The two thresholds and the 0.15 steps are this script's choice, not the paper's.
- code_lines: non-blank lines inside fenced blocks, 10 s each (a choice: code is read, not skimmed).
- links: 5 s each, to decide whether to follow it; the target's own reading time is not counted.
"""

import json
import math
import re
import sys

WPM = 238  # Brysbaert (2019), silent reading, English non-fiction
SECONDS_PER_CODE_LINE = 10
SECONDS_PER_LINK = 5
LONG_WORD = 10  # letters
LINE = re.compile(r"^(Leitura estimada|Estimated reading time):.*$", re.M)
FENCE = re.compile(r"^ {0,3}(```|~~~).*?^ {0,3}\1[ \t]*$", re.M | re.S)
URL = re.compile(r"https?://[^\s)>\]]+|www\.[^\s)>\]]+")
WORD = re.compile(r"[^\W_][\w'’-]*", re.U)

TEXT = {
    "pt-BR": ("Leitura estimada", "< 1 min", "palavra", "palavras", "link", "links", "bloco de código", "blocos de código"),
    "en": ("Estimated reading time", "< 1 min", "word", "words", "link", "links", "code block", "code blocks"),
}


def analyse(text):
    text = text.replace("\r\n", "\n")
    text = re.sub(r"\A---\n.*?\n---\n", "", text, flags=re.S)
    text = LINE.sub("", text)
    blocks = [m.group(0) for m in FENCE.finditer(text)]
    code_lines = sum(1 for b in blocks for ln in b.splitlines()[1:-1] if ln.strip())
    text = FENCE.sub("", text)
    links = len(re.findall(r"(?<!!)\[[^\]]*\]\([^)\s]+[^)]*\)|<https?://[^>]+>", text))
    text = re.sub(r"!\[[^\]]*\]\([^)]*\)", " ", text)  # images
    text = re.sub(r"\[([^\]]*)\]\([^)]*\)", r"\1", text)  # keep the link text
    bare = len(URL.findall(re.sub(r"<https?://[^>]+>", "", text)))
    links += bare
    text = URL.sub(" ", text)
    text = re.sub(r"^\s*(\|?[-:| ]+\|?)\s*$", " ", text, flags=re.M)  # table rules, hr
    words = WORD.findall(text)
    sentences = [s for s in re.split(r"[.!?]+(?:\s|$)|\n\s*\n|\n\s*(?:[-*]|\d+\.)\s", text) if WORD.search(s)]
    mean_len = len(words) / len(sentences) if sentences else 0
    long_share = sum(1 for w in words if sum(c.isalpha() for c in w) >= LONG_WORD) / len(words) if words else 0
    complexity = 1.0 + (0.15 if mean_len > 20 else 0) + (0.15 if long_share > 0.25 else 0)
    seconds = len(words) * complexity / WPM * 60 + code_lines * SECONDS_PER_CODE_LINE + links * SECONDS_PER_LINK
    minutes = 0 if seconds <= 60 else math.ceil(seconds / 60)
    return {
        "minutes": minutes, "seconds": round(seconds), "words": len(words), "links": links,
        "code_blocks": len(blocks), "code_lines": code_lines, "complexity": complexity,
        "mean_sentence_words": round(mean_len, 1), "long_word_share": round(long_share, 2),
    }


def line(r, lang="pt-BR"):
    label, under, w1, wn, l1, ln, b1, bn = TEXT[lang]
    n = lambda k, one, many: f"{r[k]} {one if r[k] == 1 else many}"
    parts = [n("words", w1, wn), n("links", l1, ln)]
    if r["code_blocks"]:
        parts.append(n("code_blocks", b1, bn))
    return f"{label}: {'~%d min' % r['minutes'] if r['minutes'] else under} ({', '.join(parts)})"


def self_test():
    cases = [
        ("empty", "", "pt-BR", "Leitura estimada: < 1 min (0 palavras, 0 links)"),
        ("short", "Reinicie o serviço.", "pt-BR", "Leitura estimada: < 1 min (3 palavras, 0 links)"),
        ("short en", "Restart the service.", "en", "Estimated reading time: < 1 min (3 words, 0 links)"),
        ("long: 2400 words = 10.1 min -> 11", "Reinicie o serviço agora. " * 600, "pt-BR", "Leitura estimada: ~11 min (2400 palavras, 0 links)"),
        ("links cost 5 s, url and target not read", "Veja [a doc](https://example.com/muito/longa) e https://x.org agora. " + "a " * 300, "pt-BR", "Leitura estimada: ~2 min (305 palavras, 2 links)"),
        ("code: 29 lines = 290 s -> 5 min", "Rode:\n\n```bash\n" + "make test\n" * 29 + "```\n", "pt-BR", "Leitura estimada: ~5 min (1 palavra, 0 links, 1 bloco de código)"),
        ("code blocks (en)", "Run:\n\n```\na\n```\n\n```\nb\n```\n", "en", "Estimated reading time: < 1 min (1 word, 0 links, 2 code blocks)"),
        ("inline code is words; the line itself is ignored", "Leitura estimada: ~9 min (1 palavra, 0 links)\n\nUse `git commit` hoje.", "pt-BR", "Leitura estimada: < 1 min (4 palavras, 0 links)"),
        ("long sentences and long words: x1.3, 6 min instead of 5", ("responsabilidades " * 99 + "agora. ") * 10, "pt-BR", "Leitura estimada: ~6 min (1000 palavras, 0 links)"),
    ]
    bad = 0
    for name, text, lang, want in cases:
        got = line(analyse(text), lang)
        ok = got == want
        bad += not ok
        print(("  ok   " if ok else f"  FAIL got {got!r}, want {want!r}: ") + name)
    r = analyse("a [b](http://x.y) c")
    ok = r["links"] == 1 and r["words"] == 3
    bad += not ok
    print(("  ok   " if ok else "  FAIL ") + "json parts")
    print("reading-time self-test: " + ("ok" if not bad else f"{bad} failure(s)"))
    return 1 if bad else 0


def main(argv):
    args, lang, as_json, path = argv[1:], "pt-BR", False, None
    if args == ["--self-test"]:
        return self_test()
    while args:
        a = args.pop(0)
        if a == "--json":
            as_json = True
        elif a == "--lang" and args and args[0] in TEXT:
            lang = args.pop(0)
        elif (a == "-" or not a.startswith("-")) and path is None:
            path = a
        else:
            print(__doc__.split("\n\n")[1], file=sys.stderr)
            return 2
    try:
        text = open(path, encoding="utf-8").read() if path not in (None, "-") else sys.stdin.read()
    except OSError as e:
        print(f"reading_time: {e}", file=sys.stderr)
        return 1
    r = analyse(text)
    print(json.dumps({**r, "line": line(r, lang)}, ensure_ascii=False) if as_json else line(r, lang))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))

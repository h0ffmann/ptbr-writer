# EXT_DESIGN.md

The look of the browser extension (`extension/`): the options page, the popup and the in-page
overlay (underlines, badge, suggestion card). It is the website's look (`DESIGN.md`,
`site/static/style.css`) applied to three small surfaces. Where this file and `DESIGN.md`
disagree, `DESIGN.md` wins.

## Where the look comes from

`DESIGN.md:3-4` says the site follows the manner of the author's sister sites "with its own
palette". The manner is picasso's (`forks/picasso/site/static/style.css` at `b919f9e`): tokens on
`:root` with `color-scheme` (lines 2-29), pills at `999px` (line 101), a card with a 1 px border
and a 3 px status edge (line 114), monospace for ids (line 121), fonts from Google Fonts by one
`<link>` (`forks/picasso/site/src/picasso_site/page.gleam:48`). The palette and the typefaces are
not picasso's: picasso is dark with Inter and Fjalla One, this site is paper with Fraunces and
Source Serif 4. The extension takes its tokens from this site's `style.css`, not from picasso's.

The site is light only (`DESIGN.md:24`, `style.css:19`: `color-scheme: light`). The extension
is light only too: a paper card on a dark host page is still a paper card. The current
`prefers-color-scheme: dark` blocks (`options.html:6`, `popup.html:6`, `overlay.rs` STYLE lines
58-64) go. A dark set would be new colours, so it starts in `DESIGN.md`, not here.

## Tokens

Every value is from `site/static/style.css` unless the source says otherwise.

| Token | Light | Dark | Source | Means |
|---|---|---|---|---|
| `--paper` | `#fbf8f1` | none (light only) | `:4` | behind everything |
| `--sheet` | `#ffffff` | none | `:5` | under code, cards, inputs |
| `--rule` | `#e4ddcd` | none | `:6` | lines between sections, card borders |
| `--ink` | `#1b1a17` | none | `:7` | text |
| `--muted` | `#6b665c` | none | `:8` | secondary text, "checking", input borders |
| `--pen` | `#1d3fd0` | none | `:10`, `DESIGN.md:17` | the writer: title, links, main button, the badge, the new text |
| `--review` | `#c8102e` | none | `:11`, `DESIGN.md:18` | the correction: a rule's id, an error, the removed text |
| `--rule-ink` | `rgba(29,63,208,.16)` | none | `:16` | the notebook's blue rules (options page only) |
| `--margin-ink` | `rgba(200,16,46,.35)` | none | `:17` | the notebook's red margin (options page only) |
| `--display` | `"Fraunces", Georgia, serif` | | `:12` | h1, h2, buttons |
| `--prose` | `"Source Serif 4", Georgia, serif` | | `:13` | body text |
| `--mono` | `"JetBrains Mono", ui-monospace, monospace` | | `:14` | rule ids, levels, hosts, table heads, the badge |

Not tokens, kept as they are (they carry meaning, see *What must not change*): the underline
colours `#d93025` (erro) and `#e8a400` (aviso), `overlay.rs` STYLE lines 7-8.

### Type

| Use | Site | Extension |
|---|---|---|
| body | `400 1.15rem/1.6 var(--prose)` (`:26`) | options `400 14px/1.5`, popup `400 13px/1.4`, card `400 13px/1.45` (the sizes already in `options.html:7`, `popup.html:7`, `overlay.rs` STYLE line 15; the card's line height from `.rules li`, `:141`) |
| h1 | `--display` 800, `--pen`, `letter-spacing: -0.02em`, review stroke under it (`:40-59`) | same, `2rem` |
| h2 | `--display` 800 (`:60`) | same, `1.25rem` |
| data | `--mono`, `0.85em` (`:73`) | same |
| rule id | `--mono` 600 `0.8em`, `--review`, `letter-spacing: .04em` (`.id`, `:145`) | same, for `.ptbr-rule` |
| table head | `--mono` 600 `0.75rem` uppercase `--muted`, `letter-spacing: .06em` (`.eli5 th`, `:121`) | same |
| button | `600 1rem var(--display)` (`:89`) | same, `inherit` size |

### Spacing, borders, radii, shadows

| Thing | Value | Source |
|---|---|---|
| pill | `border-radius: 999px` | `:89`, `:132`, `DESIGN.md:40` |
| code, input | `border: 1px solid; border-radius: 4px` | `:73` |
| card (a rule) | `--sheet`, `1px solid var(--rule)`, `border-left: 3px solid var(--review)`, `border-radius: 0 8px 8px 0`, `padding: 10px 12px` | `:141` |
| section | `border-top: 1px solid var(--rule)` | `:126` |
| table cell | `padding: 8px 10px; border-bottom: 1px dashed var(--rule)` | `:120` |
| main button | `--pen` on `--sheet`, `padding: 0.6em 1.3em`, hover `--ink`, shadow `0 6px 18px rgba(29,63,208,.25)` | `:89-91` |
| ghost button | `1px solid var(--pen)`, `--sheet` | `:92` |
| floating shadow | `0 6px 18px rgba(27,26,23,.18)` | `.install-box`, `:110` |
| notebook | blue rules every 32 px, red margin line | `:28-32`, `DESIGN.md:89-90` |

### Focus

`outline: 2px solid var(--pen); outline-offset: 3px; border-radius: 4px` on `:focus-visible`
(`:38`); inside a filled `--pen` button, `outline: 2px solid var(--sheet); outline-offset: -4px`
(`:115`). `DESIGN.md:114`: tab through, every control shows its ring.

### Contrast

WCAG 2.1 ratio, `(L1 + 0.05) / (L2 + 0.05)` with sRGB relative luminance, computed by this
script (`node contrast.mjs '#1b1a17/#fbf8f1' ...`):

```js
const lum = (hex) => { const c = hex.slice(1).match(/../g).map((h) => parseInt(h, 16) / 255)
  .map((v) => (v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4));
  return 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]; };
const ratio = (a, b) => { const [h, l] = [lum(a), lum(b)].sort((x, y) => y - x); return ((h + 0.05) / (l + 0.05)).toFixed(2); };
for (const p of process.argv.slice(2)) { const [fg, bg] = p.split('/'); console.log(fg, 'on', bg, ratio(fg, bg)); }
```

| Pair | Ratio | Needs | Where |
|---|---|---|---|
| `--ink` on `--paper` | 16.41 | 4.5 | all text |
| `--ink` on `--sheet` | 17.40 | 4.5 | card, inputs |
| `--muted` on `--paper` | 5.38 | 4.5 | secondary text |
| `--muted` on `--sheet` | 5.71 | 4.5 | table heads, input borders (3 for a boundary) |
| `--pen` on `--paper` | 7.40 | 4.5 | links, `#status` |
| `--pen` on `--sheet` | 7.85 | 4.5 | new text in the card |
| `--sheet` on `--pen` | 7.85 | 4.5 | main button, badge |
| `--review` on `--sheet` | 5.88 | 4.5 | rule id, removed text |
| `--review` on `--paper` | 5.55 | 4.5 | `#site-error` |
| `--sheet` on `--review` | 5.88 | 4.5 | badge in error |
| `--sheet` on `--muted` | 5.71 | 4.5 | badge checking |
| `#d93025` on `--sheet` | 4.77 | 3 (non-text) | erro underline on a white field |
| `#e8a400` on `--sheet` | 2.16 | 3 (non-text) | aviso underline: below 3, unchanged by this file; the hover tint and the `title` carry the meaning (⚠ open, not this spec's to fix) |
| `--rule` on `--sheet` | 1.35 | 3 (boundary) | why inputs get a `--muted` border, not `--rule`; `--rule` stays for decorative lines |

Ratios on a host page's own background are not computed: the card and the badge paint their own
background.

## Fonts

The site loads Fraunces 600 and 800 (`opsz 9..144`), Source Serif 4 400, 600 and 400 italic, and
JetBrains Mono 400 from Google Fonts (`site/src/gramatico_site/page.gleam:45`), and stores no
font file (`DESIGN.md:34`). The extension cannot do that: a content script must not fetch a font
from another host, and an extension page should work offline. So the extension bundles the files.

| Family | Repository | Licence | Files to bundle |
|---|---|---|---|
| Fraunces | github.com/undercasetype/Fraunces | OFL-1.1, `OFL.txt` | 600, 800 (or the one variable file) |
| Source Serif 4 | github.com/adobe-fonts/source-serif (release branch, `WOFF2/`) | OFL-1.1, `LICENSE.md` | 400, 600, 400 italic |
| JetBrains Mono | github.com/JetBrains/JetBrainsMono | OFL-1.1, `OFL.txt` | 400 |

Licences read on the three repositories' GitHub pages on 2026-10-22; the paths of the woff2
files inside each repository were not read (⚠), nor the sizes (⚠): check both when pinning. Rules:

- Pin a tag or commit for each, note it in `README.md` (credits never go in a skill body;
  this is not a skill, the README is the place) and put each `OFL` file next to its woff2 in
  `extension/static/fonts/`. OFL asks the licence to travel with the font and the Reserved Font
  Names not to be changed: the `font-family` names stay `Fraunces`, `Source Serif 4`,
  `JetBrains Mono`.
- `extension/xtask/src/main.rs:111` copies `static/` into `dist/`, so `static/fonts/*.woff2` ships with no build
  change.
- Extension pages (options, popup): `@font-face` with `url(fonts/<file>.woff2)`. The page CSP is
  `script-src 'self' 'wasm-unsafe-eval'; object-src 'self'` (`extension/xtask/src/main.rs:168`),
  no `font-src`, so a font from the package loads.
- Content script: `@font-face` goes in `content.css`, in the document's scope, not in the shadow
  root's `<style>`: Chrome does not apply a `@font-face` declared inside a shadow root (⚠ Chromium
  issue 336876, from memory; confirm on the current Chrome). The URL is
  `chrome-extension://__MSG_@@extension_id__/fonts/<file>.woff2` (Firefox resolves the same
  placeholder to `moz-extension://`), and the files are added to `web_accessible_resources`
  (`extension/xtask/src/main.rs:169`) with the content-script matches. ⚠ A host page's own `font-src` CSP
  (GitHub's is strict) may still block the fetch; then the stacks fall back to Georgia and
  `ui-monospace`, and the card is still the card. Every `font-family` keeps its fallback for this
  reason. If this proves unreliable, the lazy answer is to bundle fonts for the two extension
  pages only and let the overlay use the fallbacks.
- Fewer bytes: static instances of the weights above, never the whole Google Fonts request.
  `font-display: swap`.

```css
/* fonts.css, inlined in options.html and popup.html; in content.css with the chrome-extension:// URL */
@font-face { font-family: "Fraunces"; font-weight: 600; src: url(fonts/Fraunces-SemiBold.woff2) format("woff2"); font-display: swap; }
@font-face { font-family: "Fraunces"; font-weight: 800; src: url(fonts/Fraunces-ExtraBold.woff2) format("woff2"); font-display: swap; }
@font-face { font-family: "Source Serif 4"; font-weight: 400; src: url(fonts/SourceSerif4-Regular.woff2) format("woff2"); font-display: swap; }
@font-face { font-family: "Source Serif 4"; font-weight: 400; font-style: italic; src: url(fonts/SourceSerif4-It.woff2) format("woff2"); font-display: swap; }
@font-face { font-family: "Source Serif 4"; font-weight: 600; src: url(fonts/SourceSerif4-Semibold.woff2) format("woff2"); font-display: swap; }
@font-face { font-family: "JetBrains Mono"; font-weight: 400; src: url(fonts/JetBrainsMono-Regular.woff2) format("woff2"); font-display: swap; }
```

The file names above are the pattern, not a fact (⚠): use the names the pinned release has.

## What must not change

- The underline colours and their meaning: `#d93025` is erro, `#e8a400` is aviso, 2 px
  `border-bottom` (`overlay.rs` STYLE lines 5, 7-8). Mapping erro to `--review` would be a new
  meaning for the pen; it starts in `DESIGN.md`, not here.
- `HIT_STYLE` (`overlay.rs:94`): the 4 px strip at the bottom of each segment that takes the
  click. It is inline, not in STYLE.
- `.ptbr-mirror` (`overlay.rs` STYLE line 11 and lines 718-771): it copies the host field's font
  to measure text. It never gets our font.
- `:host { all: initial; }` (STYLE line 2) and `#gramatico-root` in `content.css` (position,
  zero size, `z-index: 2147483647`, `pointer-events: none`, all `!important`). They are the
  isolation: nothing of the host leaks in, nothing of ours leaks out. Our styles live inside the
  shadow root, `content.css` touches only the host element (and now `@font-face`, which is
  document-wide by nature but changes nothing unless a `font-family` names it).
- Every id, class and data attribute: options `#app #enabled #provider-fields #provider #model
  #key #pull-model #pull-status #ollaya-style #save #status #sites #new-host #add-site
  #site-error`; popup `#app section #enabled #site-level #toggle-site #save-key #open-options`;
  overlay `.ptbr-layer .ptbr-mirror .ptbr-underline .ptbr-underline-erro .ptbr-underline-aviso
  .ptbr-seg .ptbr-hit .ptbr-card .ptbr-rule .ptbr-change .ptbr-actions .ptbr-badge`,
  `button[data-action="accept"|"ignore"]`, `.ptbr-badge[data-state="idle"|"checking"|"error"]`,
  `role="dialog"`, `role="status"`, `role="alert"`.
- The Portuguese UI text, lower case where the page speaks (`DESIGN.md:38`). Text is set in
  `options.rs`, `popup.rs`, `overlay.rs`, out of this file's reach.
- The popup width, 300 px (`popup.html:7`), and the card's `max-width: 320px` (STYLE line 20).
- `color-scheme` becomes `light`: a native `<select>` then draws light, which is what the paper
  wants.

## Options page (`extension/static/options.html`, the inline `<style>`)

The site's body on a 560 px column: paper, the notebook, Fraunces title with the review stroke,
sections by a line, a data table in mono, pills for buttons.

```css
:root {
  --paper: #fbf8f1; --sheet: #ffffff; --rule: #e4ddcd; --ink: #1b1a17; --muted: #6b665c;
  --pen: #1d3fd0; --review: #c8102e;
  --display: "Fraunces", Georgia, serif; --prose: "Source Serif 4", Georgia, serif; --mono: "JetBrains Mono", ui-monospace, monospace;
  --rule-ink: rgba(29, 63, 208, 0.16); --margin-ink: rgba(200, 16, 46, 0.35);
  --margin: max(12px, calc((100vw - 560px) / 2 - 16px));
  color-scheme: light;
}
/* the @font-face block from Fonts, with url(fonts/...) */
* { box-sizing: border-box; }
html { background: var(--paper); }
body {
  margin: 0; color: var(--ink); font: 400 14px/1.5 var(--prose);
  background:
    linear-gradient(to right, transparent 0 var(--margin), var(--margin-ink) var(--margin) calc(var(--margin) + 2px), transparent calc(var(--margin) + 2px)),
    repeating-linear-gradient(to bottom, transparent 0 31px, var(--rule-ink) 31px 32px),
    var(--paper);
  background-attachment: local;
}
#app { max-width: 560px; margin: 0 auto; padding: 24px 16px; }
h1 { font: 800 2rem/1.1 var(--display); color: var(--pen); letter-spacing: -0.02em; margin: 0 0 16px; position: relative; display: inline-block; }
h1::after { content: ""; position: absolute; left: 0.04em; bottom: -0.08em; width: 2.6em; height: 0.09em; background: var(--review); border-radius: 999px; transform: rotate(-1.2deg); transform-origin: left; }
h2 { font: 800 1.25rem/1.1 var(--display); margin: 24px 0 8px; padding-top: 16px; border-top: 1px solid var(--rule); }
p { margin: 0 0 12px; }
a { color: var(--pen); text-decoration-thickness: 1px; text-underline-offset: 3px; }
a:hover { color: var(--ink); }
:focus-visible { outline: 2px solid var(--pen); outline-offset: 3px; border-radius: 4px; }
#provider-fields { display: grid; gap: 10px; margin: 16px 0 12px; padding-top: 16px; border-top: 1px solid var(--rule); }
#provider-fields label { display: grid; grid-template-columns: 80px 1fr; align-items: center; gap: 12px; }
select, input[type=text], input[type=password] { font: inherit; padding: 4px 8px; color: var(--ink); background: var(--sheet); border: 1px solid var(--muted); border-radius: 4px; }
#provider-fields select, #provider-fields input { width: 100%; max-width: 320px; }
#model, #site-level, #sites select, #new-host { font-family: var(--mono); font-size: 0.85em; }
button { font-family: var(--display); font-weight: 600; font-size: inherit; padding: 0.4em 1.1em; border-radius: 999px; border: 1px solid var(--pen); background: var(--sheet); color: var(--ink); cursor: pointer; transition: transform 0.15s ease; }
button:hover { transform: translateY(-1px); }
#save { background: var(--pen); border-color: var(--pen); color: var(--sheet); box-shadow: 0 6px 18px rgba(29, 63, 208, 0.25); }
#save:hover { background: var(--ink); border-color: var(--ink); }
#save:focus-visible { outline: 2px solid var(--sheet); outline-offset: -4px; }
#status { color: var(--pen); margin-left: 8px; }
#pull-status { color: var(--muted); font-family: var(--mono); font-size: 0.85em; }
table { border-collapse: collapse; width: 100%; font-size: 0.95em; }
td, th { padding: 8px 10px; text-align: left; border-bottom: 1px dashed var(--rule); vertical-align: baseline; }
th { font: 600 0.75rem var(--mono); color: var(--muted); text-transform: uppercase; letter-spacing: 0.06em; }
#sites td:first-child { font-family: var(--mono); font-size: 0.85em; }
table + p { margin-top: 12px; }
#new-host { width: 16em; }
#site-error { color: var(--review); margin-left: 8px; }
@media (prefers-reduced-motion: reduce) { button { transition: none; } }
```

Skipped: the `§ 01` counter (one h2 only) and the site's entrance animation.

## Popup (`extension/static/popup.html`, the inline `<style>`)

Paper without the notebook (300 px is too narrow for a margin line), sections by a line, pills.

```css
:root {
  --paper: #fbf8f1; --sheet: #ffffff; --rule: #e4ddcd; --ink: #1b1a17; --muted: #6b665c;
  --pen: #1d3fd0; --review: #c8102e;
  --display: "Fraunces", Georgia, serif; --prose: "Source Serif 4", Georgia, serif; --mono: "JetBrains Mono", ui-monospace, monospace;
  color-scheme: light;
}
/* the @font-face block from Fonts, with url(fonts/...) */
* { box-sizing: border-box; }
body { font: 400 13px/1.4 var(--prose); color: var(--ink); background: var(--paper); margin: 0; width: 300px; }
#app { padding: 12px; }
section { padding: 10px 0; border-bottom: 1px solid var(--rule); }
section:first-child { padding-top: 0; font: 600 1rem var(--display); color: var(--pen); }
p { margin: 0 0 6px; }
section:first-child label { display: flex; align-items: center; gap: 6px; }
label { display: grid; grid-template-columns: 64px 1fr; align-items: center; gap: 8px; margin: 6px 0; }
select, input[type=password] { width: 100%; font: inherit; padding: 3px 6px; color: var(--ink); background: var(--sheet); border: 1px solid var(--muted); border-radius: 4px; }
#site-level, #model { font-family: var(--mono); font-size: 0.85em; }
:focus-visible { outline: 2px solid var(--pen); outline-offset: 3px; border-radius: 4px; }
button { font-family: var(--display); font-weight: 600; font-size: inherit; padding: 0.35em 1em; border-radius: 999px; border: 1px solid var(--pen); background: var(--sheet); color: var(--ink); cursor: pointer; }
#save-key { display: block; margin: 6px 0 0 auto; background: var(--pen); border-color: var(--pen); color: var(--sheet); }
#save-key:hover { background: var(--ink); border-color: var(--ink); }
#save-key:focus-visible { outline: 2px solid var(--sheet); outline-offset: -4px; }
#toggle-site { margin-top: 4px; }
#open-options { display: block; width: 100%; margin-top: 10px; }
```

## Overlay (`extension/src/content/overlay.rs`, the `STYLE` const)

Inside the shadow root. The card is the site's rule card (`style.css:141`) made to float; the
badge is the `feito` pill (`style.css:135`, `--pen` on `--sheet`); the rule id is `.id`
(`style.css:145`); the removed text is the review pen, the new text the writer's pen; the
buttons are the site's pills. Lines 2-11 stay as they are.

```css
:host { all: initial; }
.ptbr-layer, .ptbr-mirror { position: absolute; top: 0; left: 0; }
.ptbr-underline { pointer-events: none; }
.ptbr-seg { position: absolute; box-sizing: border-box; border-bottom: 2px solid; }
.ptbr-hit { cursor: pointer; }
.ptbr-underline-erro .ptbr-seg { border-color: #d93025; }
.ptbr-underline-aviso .ptbr-seg { border-color: #e8a400; }
.ptbr-underline:hover .ptbr-seg { background: rgba(232, 164, 0, 0.15); }
.ptbr-underline-erro:hover .ptbr-seg { background: rgba(217, 48, 37, 0.12); }
.ptbr-mirror { visibility: hidden; overflow: hidden; box-sizing: border-box; border-style: solid; border-color: transparent; }
.ptbr-card, .ptbr-badge {
  pointer-events: auto;
  position: absolute;
  font: 400 13px/1.45 "Source Serif 4", Georgia, serif;
  color: #1b1a17;
}
.ptbr-card {
  box-sizing: border-box;
  max-width: 320px;
  padding: 10px 12px;
  background: #ffffff;
  border: 1px solid #e4ddcd;
  border-left: 3px solid #c8102e;
  border-radius: 0 8px 8px 0;
  box-shadow: 0 6px 18px rgba(27, 26, 23, 0.18);
}
.ptbr-rule { margin: 0 0 6px; font: 600 11px/1.4 "JetBrains Mono", ui-monospace, monospace; color: #c8102e; letter-spacing: 0.04em; }
.ptbr-change { margin: 0 0 8px; }
.ptbr-change s { color: #c8102e; }
.ptbr-change strong { color: #1d3fd0; font-weight: 600; }
.ptbr-actions { display: flex; gap: 6px; }
.ptbr-actions button {
  font: 600 13px/1.4 "Fraunces", Georgia, serif;
  padding: 3px 12px;
  border-radius: 999px;
  border: 1px solid #1d3fd0;
  background: #ffffff;
  color: #1b1a17;
  cursor: pointer;
}
.ptbr-actions button:hover { background: #1b1a17; border-color: #1b1a17; color: #ffffff; }
.ptbr-actions button:focus-visible { outline: 2px solid #1d3fd0; outline-offset: 3px; }
.ptbr-actions button[data-action="accept"] { background: #1d3fd0; border-color: #1d3fd0; color: #ffffff; }
.ptbr-actions button[data-action="accept"]:focus-visible { outline: 2px solid #ffffff; outline-offset: -4px; }
.ptbr-badge {
  box-sizing: border-box;
  min-width: 20px;
  height: 20px;
  padding: 0 7px;
  border-radius: 999px;
  background: #1d3fd0;
  color: #ffffff;
  font: 600 11px/20px "JetBrains Mono", ui-monospace, monospace;
  letter-spacing: 0.04em;
  text-align: center;
  white-space: nowrap;
  cursor: pointer;
}
.ptbr-badge[data-state="checking"] { background: #6b665c; }
.ptbr-badge[data-state="error"] { background: #c8102e; }
```

Hex literals, not `var()`: `:host { all: initial }` resets inherited custom properties and the
shadow root has no `:root`; literals keep the block self-contained. No `prefers-color-scheme`
block: the card is paper on every host.

## Checklist

1. Options page and popup at the OS light and dark theme: both stay paper (`color-scheme: light`),
   no dark block left. Screenshots of each, light and dark.
2. The overlay on a light field (GitHub) and on a dark one (WhatsApp Web dark): the card is white
   with the review edge, the badge is `--pen`; erro is still `#d93025`, aviso `#e8a400`; the 4 px
   strip still opens the card. Screenshots of each.
3. Fonts offline: disconnect, reload the options page, popup and a card; DevTools → Network shows
   the woff2 from `chrome-extension://` (or `moz-extension://`), nothing from `fonts.gstatic.com`.
   On GitHub, check whether the host CSP blocks the font (⚠ above); if it does, the fallback is
   Georgia and the card still reads.
4. Contrast: run the script above on every pair of the table; all text pairs at 4.5 or more.
5. Tab through the options page, the popup and the card: every control shows the `--pen` ring,
   the filled buttons the `--sheet` ring.
6. `just ext-test` and `just ext-e2e` pass: no id, class or data attribute moved.
7. `README.md` and `README.en.md` name the three fonts, their OFL and the pinned commits, in the
   same PR.

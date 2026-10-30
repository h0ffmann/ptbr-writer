# Drawings

## The icon

A big G, as the G of the square and compass, in the colours of the flag of Brazil: the favicon of
the site, the picture at the top of the README and the icon of the app. The Nix files in
`drawings/` write it in two formats from one geometry; the flake's `icon-art` package holds the
output:

| File | Command | What it is |
|---|---|---|
| `site/static/favicon.svg` | `just icon` | The icon |
| `app/src-tauri/icons/icon.svg` | `just icon` | The same file, the source of the app's icons |
| `docs/drawings/icon.desmos.json` | `just icon` | The same drawing as a Desmos graph state |

The app's PNG, ICNS and ICO files, desktop and Android, are made from `icon.svg` by
`scripts/app.sh icons` (`cargo tauri icon`); they are not checked.

The tests run in `nix flake check` (the `icon` check, `lib.runTests` in `drawings/default.nix`).
They check that the committed files are the output of `icon-art`, that the G is a G (its bowl
opens on the right, heaviest on the left, the bar and the spur on the right), that the square and
the compass are symmetric and make a rhombus, that every line stays on the green field and the G
on the blue globe, and that every Desmos expression, read back from its LaTeX, stays within 0.01 of
the line or disc it stands for.

**What it shows.** The flag of Brazil, drawn as the square and compass: a green disc for the field;
on it, in yellow, the square (its corner down) and the compass (its hinge up), whose legs cross the
square's arms, making the rhombus; over them a blue disc, the globe; and on the globe a big white
G, brushed like a pen letter, with its bar, its spur and a serif where it starts.

**How it is drawn.** As piabrota/nix-os draws its 无 and h0ffmann/rax its 代理: each line is a list
of hand-placed points with a brush width at each. A Catmull-Rom spline runs through them; each piece
is a cubic Bézier and each Bézier is one Desmos parametric expression, `(a+bt+ct^{2}+dt^{3}, …)` for
`0 ≤ t ≤ 1`, cut in four so the width changes smoothly. The discs are Desmos points. The pen is
`drawings/pen.nix`, nix-os's pen with rax's dots and two additions: layers, drawn in order, and an
ink for each line or disc; the drawing is `drawings/g.nix`. Both are pure Nix, evaluated without a
build. To see the Desmos expressions over the geometry, run `nix build .#icon-art` and open
`result/icon-check.svg`: the geometry in light blue, each expression sampled in orange inside it.

The colours are the flag's: green `#009C3B`, yellow `#FFDF00`, blue `#002776` and white.

### Open the Desmos file

The file is a Desmos graph state (`version` 11). Desmos uses y up, so each y value is `128 - y` of
the SVG (the canvas is 128 × 128). Desmos draws line widths and point sizes in pixels, so the
widths match the SVG only at one zoom.

1. Open `https://www.desmos.com/calculator`.
2. Open the browser console.
3. Run `Calc.setState(<the content of icon.desmos.json>)`.

⚠ Not checked in Desmos itself; the check is local, as in nix-os and rax.

### Credits

- The pen and the method are piabrota/nix-os's (`drawings/pen.nix`), itself
  [pirsig](https://github.com/h0ffmann/pirsig)'s (`crates/pirsig-drawings`, at `450e4da`) ported to
  Nix, with the dots of [h0ffmann/rax](https://github.com/h0ffmann/rax)'s copy.
- The points and widths are placed by hand; no path is copied from any emblem or font.

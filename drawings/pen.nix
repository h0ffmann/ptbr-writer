# A pen for Desmos line art, in pure Nix: pirsig's pen (crates/pirsig-drawings, at 450e4da)
# ported from Rust, as piabrota/nix-os carries it (drawings/pen.nix), with dots, as
# h0ffmann/rax carries it, and two additions of its own: layers drawn in order, and an `ink` for
# each mark or dot.
#
# A stroke is a list of points. The pen runs a Catmull-Rom spline through them, so the line is
# smooth and passes through every point. Each piece of the spline is a cubic Bézier, and each
# Bézier is one Desmos parametric expression, `(a+bt+ct^2+dt^3, …)` for `0 ≤ t ≤ 1`.
#
# The SVG has y down and Desmos has y up: a Desmos y is `H - y` of the SVG.
# Nix has no fused multiply-add and no square root: `fma` stands in for Rust's `mul_add`, so the
# numbers come out as the Rust pen's did, and the check compares squared distances.
{ lib }:
let
  inherit (builtins) concatStringsSep elemAt floor genList length;

  # The ink of the drawing, light and dark, as in h0ffmann/picasso's drawings.
  ink = "#1f2328";
  inkDark = "#e6edf3";
  # The blue of the check render, the same as in piabrota/alerts, and the orange of its Desmos layer.
  blue = "#1682D4";
  orange = "#FF4F00";

  x = p: elemAt p 0;
  y = p: elemAt p 1;

  # Rust's f64::round, half away from zero, as an integer.
  round = v: if v < 0 then -(floor (0.5 - v)) else floor (v + 0.5);
  abs = v: if v < 0 then -v else v;

  # a * b + c as one rounding, as Rust's f64::mul_add: the exact product as a sum of two floats
  # (Dekker's split), then a compensated sum (Ogita, Rump and Oishi, 2005). Rounding a * b first
  # moves some numbers of the drawing by 0.001.
  fma = a: b: c:
    let
      split = v:
        let t = 134217729.0 * v; hi = t - (t - v);
        in [ hi (v - hi) ];
      sa = split a;
      sb = split b;
      p = a * b;
      e = elemAt sa 0 * elemAt sb 0 - p + elemAt sa 0 * elemAt sb 1 + elemAt sa 1 * elemAt sb 0
        + elemAt sa 1 * elemAt sb 1;
      s = p + c;
      z = s - p;
      err = p - (s - z) + (c - z);
    in
    s + (err + e);

  # The integer `a` with its last `d` digits as decimals, after a minus sign when `neg`.
  decimals = d: neg: a:
    let p = lib.foldl' (acc: _: acc * 10) 1 (genList (_: null) d);
    in "${lib.optionalString neg "-"}${toString (a / p)}.${lib.fixedWidthNumber d (a - a / p * p)}";

  # `v` with one decimal, as Rust's `{:.1}`: the nearest of the exact value, a tie to even.
  tenths = v:
    let
      f = floor (v * 10.0);
      r = fma v 10.0 (-f * 1.0);
      n = if r > 0.5 || (r == 0.5 && lib.mod f 2 != 0) then f + 1 else f;
    in
    decimals 1 (v < 0) (abs n);

  # A number with at most three decimals, without trailing zeros: `12`, `-3.5`, `0.125`.
  num = v:
    let
      n = round (v * 1000.0);
      m = builtins.match "(-?[0-9]+)\\.([0-9]*[1-9])?0*" (decimals 3 (n < 0) (abs n));
    in
    if elemAt m 1 == null then elemAt m 0 else "${elemAt m 0}.${elemAt m 1}";

  # `a+bt+ct^{2}+dt^{3}`, skipping the terms that round to zero.
  poly = c:
    num (elemAt c 0) + concatStringsSep "" (genList
      (i:
        let
          k = i + 1;
          s = num (elemAt c k);
          pow = if k == 1 then "t" else "t^{${toString k}}";
        in
        lib.optionalString (s != "0") "${lib.optionalString (!lib.hasPrefix "-" s) "+"}${s}${pow}")
      3);

  # The power form `a + bt + ct² + dt³` of each axis of a Bézier: [[a b c d] [a b c d]].
  coeffs = bz:
    let
      f = axis:
        let
          a = axis (elemAt bz 0);
          b = axis (elemAt bz 1);
          c = axis (elemAt bz 2);
          d = axis (elemAt bz 3);
        in
        [ a (3.0 * (b - a)) (3.0 * (fma 2.0 (-b) a + c)) (fma 3.0 (b - c) (d - a)) ];
    in
    [ (f x) (f y) ];

  # The point of a Bézier at `t`.
  at = bz: t:
    let
      u = 1.0 - t;
      f = axis:
        fma (u * u * u) (axis (elemAt bz 0)) (fma (3.0 * u * u * t) (axis (elemAt bz 1))
          (fma (3.0 * u * t * t) (axis (elemAt bz 2)) (t * t * t * axis (elemAt bz 3))));
    in
    [ (f x) (f y) ];

  # The same curve between `t0` and `t1`, as a Bézier of its own.
  part = bz: t0: t1:
    let
      h = t1 - t0;
      # derivative at t, for the control points of the piece
      deriv = t:
        let
          u = 1.0 - t;
          g = axis:
            let
              a = axis (elemAt bz 0);
              b = axis (elemAt bz 1);
              c = axis (elemAt bz 2);
              e = axis (elemAt bz 3);
            in
            3.0 * fma (u * u) (b - a) (fma (2.0 * u * t) (c - b) (t * t * (e - c)));
        in
        [ (g x) (g y) ];
      a = at bz t0;
      b = at bz t1;
      da = deriv t0;
      db = deriv t1;
    in
    [
      a
      [ (fma (x da) (h / 3.0) (x a)) (fma (y da) (h / 3.0) (y a)) ]
      [ (fma (x db) (-h / 3.0) (x b)) (fma (y db) (-h / 3.0) (y b)) ]
      b
    ];

  # The Béziers of the open Catmull-Rom spline through `pts`, held at its ends.
  spline = pts:
    let
      n = length pts;
      get = i: elemAt pts (lib.max 0 (lib.min (n - 1) i));
    in
    genList
      (i:
        let
          p0 = get (i - 1);
          p1 = get i;
          p2 = get (i + 1);
          p3 = get (i + 2);
        in
        [
          p1
          [ (x p1 + (x p2 - x p0) / 6.0) (y p1 + (y p2 - y p0) / 6.0) ]
          [ (x p2 - (x p3 - x p1) / 6.0) (y p2 - (y p3 - y p1) / 6.0) ]
          p2
        ])
      (n - 1);

  # Pieces each segment of a brush stroke is cut into, so its width changes smoothly.
  pieces = 4;

  # A brush stroke through `pts`: the smooth line of `spline`, whose width follows `widths` (one
  # per point) as a brush presses down and lifts. Each segment is cut into `pieces` Béziers, each
  # a mark { bezier; width; } drawn at the width of its middle.
  brush = pts: widths:
    assert length pts == length widths;
    lib.concatLists (lib.imap0
      (i: bz: genList
        (j:
          let
            t0 = j * 1.0 / pieces;
            t1 = (j + 1) * 1.0 / pieces;
            mid = (t0 + t1) / 2.0;
            w0 = elemAt widths i;
          in
          {
            bezier = part bz t0 t1;
            width = fma (elemAt widths (i + 1) - w0) mid w0;
          })
        pieces)
      (spline pts));

  # Pretty JSON as serde_json writes it: two spaces, sorted keys, a float with its `.0`.
  json = indent: v:
    let next = indent + "  ";
    in
    if builtins.isAttrs v then
      "{\n${concatStringsSep ",\n" (map (k: "${next}${builtins.toJSON k}: ${json next v.${k}}") (builtins.attrNames v))}\n${indent}}"
    else if builtins.isList v then
      "[\n${concatStringsSep ",\n" (map (e: next + json next e) v)}\n${indent}]"
    else if builtins.isFloat v then
      (if floor v == v then "${toString (floor v)}.0" else throw "json: ${toString v} is not whole")
    else builtins.toJSON v;

  # The two coordinates of `\left(x,y\right)`, each as polynomial coefficients by power of t.
  parsePair = latex:
    let
      m = builtins.match "\\\\left\\(([^,]*),([^,]*)\\\\right\\)" latex;
    in
    if m == null then throw "cannot read ${latex}" else map parsePoly m;

  # `12.5-3t+0.25t^{2}` → [12.5 -3 0.25 0].
  parsePoly = s:
    let
      parts = builtins.split "([+-]?[^+-]+)" s;
      read = term:
        let m = builtins.match "\\+?(-?[0-9.]+)(t(\\^\\{([0-9])})?)?" term;
        in
        if m == null then throw "cannot read the term ${term} of ${s}"
        else {
          c = builtins.fromJSON (elemAt m 0) * 1.0;
          k = if elemAt m 1 == null then 0 else if elemAt m 3 == null then 1 else lib.toInt (elemAt m 3);
        };
      terms = map (m: read (elemAt m 0)) (builtins.filter builtins.isList parts);
    in
    assert builtins.all (p: builtins.isList p || p == "") parts;
    genList (k: lib.foldl' (acc: r: if r.k == k then acc + r.c else acc) 0.0 terms) 4;

  evalPoly = c: t: lib.foldr (v: acc: fma acc t v) 0.0 c;
in
{
  inherit fma num poly coeffs at part evalPoly parsePoly brush tenths;

  # A finished drawing: its canvas `w` × `h`, its label and description, and its layers, drawn
  # in order, each one over the last: { marks; dots; }, the dots of a layer over its marks.
  # `marks` and `dots` alone are one layer. A dot { at = [x y]; r; } is a filled disc; in Desmos
  # it is a point. A mark or a dot with an `ink` is drawn in it, light page or dark, and its
  # Desmos expression takes that colour.
  drawing = { w, h, label, desc, marks ? [ ], dots ? [ ], layers ? [{ inherit marks dots; }] }:
    let
      # Each mark and each dot of the drawing, in drawing order, with its Desmos expression.
      items = lib.imap1
        (i: item:
          let id = toString i; in
          if item ? mark then
            let
              mark = item.mark;
              c = coeffs mark.bezier;
              cx = elemAt c 0;
              cy = elemAt c 1;
              px = poly cx;
              py = poly ([ (h - elemAt cy 0) ] ++ map (v: -v) (lib.drop 1 cy));
            in
            item // {
              expression = {
                type = "expression";
                inherit id;
                color = mark.ink or blue;
                latex = "\\left(${px},${py}\\right)";
                parametricDomain = { min = "0"; max = "1"; };
                lineWidth = num mark.width;
              };
            }
          else
            let dot = item.dot; in
            item // {
              expression = {
                type = "expression";
                inherit id;
                color = dot.ink or blue;
                latex = "\\left(${num (x dot.at)},${num (h - y dot.at)}\\right)";
                pointSize = num (2.0 * dot.r);
              };
            })
        (lib.concatMap
          (l: map (mark: { inherit mark; }) (l.marks or [ ]) ++ map (dot: { inherit dot; }) (l.dots or [ ]))
          layers);

      lines = builtins.filter (item: item ? mark) items;
      points = builtins.filter (item: item ? dot) items;

      # The attribute that gives a mark or a dot its own ink, if it has one.
      inkOf = attr: e: lib.optionalString (e ? ink) " ${attr}=\"${e.ink}\"";

      # The geometry as SVG: a path per mark, a disc per dot.
      shape = mark:
        let
          b = mark.bezier;
          pt = p: "${num (x p)} ${num (y p)}";
        in
        "<path d=\"M${pt (elemAt b 0)} C${pt (elemAt b 1)} ${pt (elemAt b 2)} ${pt (elemAt b 3)}\" stroke-width=\"${num mark.width}\"${inkOf "stroke" mark}/>";
      disc = dot: "<circle cx=\"${num (x dot.at)}\" cy=\"${num (y dot.at)}\" r=\"${num dot.r}\"${inkOf "fill" dot}/>";

      # The layers as SVG groups, in order: `lineAttrs` on the marks' group, `dotAttrs` on the dots'.
      groups = lineAttrs: dotAttrs: concatStringsSep "\n  " (lib.concatMap
        (l:
          lib.optional ((l.marks or [ ]) != [ ]) "<g ${lineAttrs}>\n    ${concatStringsSep "\n    " (map shape l.marks)}\n  </g>"
          ++ lib.optional ((l.dots or [ ]) != [ ]) "<g ${dotAttrs}>\n    ${concatStringsSep "\n    " (map disc l.dots)}\n  </g>")
        layers);

      sw = num w;
      sh = num h;
    in
    {
      # The drawing as a Desmos graph state (`version` 11).
      desmos = json ""
        {
          version = 11;
          graph = {
            viewport = { xmin = 0; xmax = w; ymin = 0; ymax = h; };
            showGrid = false;
            showXAxis = false;
            showYAxis = false;
          };
          expressions.list = map (item: item.expression) items;
        } + "\n";

      # The drawing as SVG. What has no ink of its own is in h0ffmann/picasso's ink: near black on
      # light pages, near white on dark ones (`prefers-color-scheme`).
      svg = ''
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${sw} ${sh}" width="${sw}" height="${sh}" role="img" aria-label="${label}">
          <title>${label}</title>
          <desc>${desc}</desc>
          <style>.l{fill:none;stroke:${ink};stroke-linecap:round;stroke-linejoin:round}.d{fill:${ink};stroke:none}@media (prefers-color-scheme:dark){.l{stroke:${inkDark}}.d{fill:${inkDark}}}</style>
          ${groups "class=\"l\"" "class=\"d\""}
        </svg>
      '';

      # The check render: the geometry in blue and, on top, each Desmos expression read back from
      # its LaTeX and sampled in orange. Where the two agree, the orange line runs inside the blue
      # and each orange point sits in its blue disc.
      checkSvg =
        let
          blueLayers = map
            (l: { marks = map (m: removeAttrs m [ "ink" ]) (l.marks or [ ]); dots = map (d: removeAttrs d [ "ink" ]) (l.dots or [ ]); })
            layers;
          orangeLines = map
            (item:
              let
                f = parsePair item.expression.latex;
                pts = genList
                  (i:
                    let t = i / 24.0;
                    in "${tenths (evalPoly (elemAt f 0) t)},${tenths (h - evalPoly (elemAt f 1) t)}")
                  25;
                lw = builtins.fromJSON item.expression.lineWidth * 1.0;
              in
              "<polyline points=\"${concatStringsSep " " pts}\" stroke-width=\"${num (lib.min (0.4 * lw) 0.8)}\"/>")
            lines;
          orangePoints = map
            (item:
              let
                f = parsePair item.expression.latex;
                r = builtins.fromJSON item.expression.pointSize * 0.5;
              in
              "<circle cx=\"${num (elemAt (elemAt f 0) 0)}\" cy=\"${num (h - elemAt (elemAt f 1) 0)}\" r=\"${num (lib.min (0.4 * r) 0.8)}\"/>")
            points;
        in
        ''
          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${sw} ${sh}" width="${sw}" height="${sh}" role="img" aria-label="${label}, with its Desmos expressions over it">
            <g fill="none" stroke="${blue}" stroke-opacity="0.35" stroke-linecap="round" stroke-linejoin="round">
              ${concatStringsSep "\n    " (map shape (lib.concatMap (l: l.marks) blueLayers))}
            </g>
            <g fill="${blue}" fill-opacity="0.35">
              ${concatStringsSep "\n    " (map disc (lib.concatMap (l: l.dots) blueLayers))}
            </g>
            <g fill="none" stroke="${orange}" stroke-linecap="round" stroke-linejoin="round">
              ${concatStringsSep "\n    " orangeLines}
            </g>
            <g fill="${orange}">
              ${concatStringsSep "\n    " orangePoints}
            </g>
          </svg>
        '';

      # The square of how far, at most, any Desmos expression strays from what it stands for:
      # each line is read back from its LaTeX and compared with its Bézier at eleven points, and
      # each point with the centre of its dot.
      worstSquared =
        let
          ofLine = item:
            let f = parsePair item.expression.latex;
            in lib.foldl' lib.max 0.0 (genList
              (i:
                let
                  t = i / 10.0;
                  p = at item.mark.bezier t;
                  dx = evalPoly (elemAt f 0) t - x p;
                  dy = h - evalPoly (elemAt f 1) t - y p;
                in
                dx * dx + dy * dy)
              11);
          ofPoint = item:
            let
              f = parsePair item.expression.latex;
              dx = elemAt (elemAt f 0) 0 - x item.dot.at;
              dy = h - elemAt (elemAt f 1) 0 - y item.dot.at;
            in
            dx * dx + dy * dy;
        in
        lib.foldl' lib.max 0.0 (map ofLine lines ++ map ofPoint points);
    };
}

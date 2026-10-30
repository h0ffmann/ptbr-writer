# The icon, a G in the colours of the flag of Brazil, in pure Nix: `svg`, `desmos` and `checkSvg`
# are the files' text, `failures` the tests that failed (lib.runTests), empty when all pass.
{ lib }:
let
  pen = import ./pen.nix { inherit lib; };
  icon = import ./g.nix { inherit lib pen; };
  near = a: b: (if a < b then b - a else a - b) < 1.0e-9;
  bz = [ [ 0.0 0.0 ] [ 10.0 30.0 ] [ 40.0 (-10.0) ] [ 50.0 20.0 ] ];
  inherit (builtins) elemAt length;
  first = s: elemAt s 0;
  last = s: elemAt s (length s - 1);
  col = k: p: elemAt p k;
  inherit (icon.strokes) g square compass;
  bowl = first g;
  # The square of the distance from the centre of the canvas.
  fromCentre = p: let dx = col 0 p - 64.0; dy = col 1 p - 64.0; in dx * dx + dy * dy;
in
{
  inherit (icon.drawing) svg desmos checkSvg;

  failures = lib.runTests {
    testNumbersAreShort = {
      expr = map pen.num [ 12.0 (-3.5) 0.1234 (-0.0001) ];
      expected = [ "12" "-3.5" "0.123" "0" ];
    };
    testAPolynomialReadsBack = {
      expr = let s = pen.poly [ 12.5 (-3.0) 0.0 0.25 ]; in [ s (pen.parsePoly s) ];
      expected = [ "12.5-3t+0.25t^{3}" [ 12.5 (-3.0) 0.0 0.25 ] ];
    };
    testAPartIsTheSameCurve = {
      expr = builtins.all
        (k:
          let
            s = k / 4.0;
            p = pen.at (pen.part bz 0.25 0.75) s;
            q = pen.at bz (0.5 * s + 0.25);
          in
          near (col 0 p) (col 0 q) && near (col 1 p) (col 1 q))
        (lib.range 0 4);
      expected = true;
    };
    testThePowerFormIsTheBezier = {
      # Midpoint of a cubic Bézier: (p0 + 3p1 + 3p2 + p3) / 8.
      expr = let cx = elemAt (pen.coeffs bz) 0; in
        near (pen.evalPoly cx 0.5) 25.0 && near (pen.evalPoly cx 1.0) 50.0;
      expected = true;
    };
    testTheGIsAG = {
      expr = [
        (length g)
        # the bowl opens on the right: it starts and ends right of the centre, and goes round the left
        (col 0 (first bowl) > 64.0 && col 0 (last bowl) > 64.0 && lib.foldl' lib.min 128.0 (map (col 0) bowl) < 44.0)
        # it starts at the top and ends at the bottom
        (col 1 (first bowl) < 64.0 && col 1 (last bowl) > 64.0)
        # it is heaviest on its left side
        (lib.foldl' lib.max 0.0 (map (col 2) bowl) == col 2 (elemAt bowl 4))
        # the bar runs in from the right at the height of the centre, and the spur drops from its end
        (near (col 1 (first (elemAt g 1))) (col 1 (last (elemAt g 1))) && col 0 (first (elemAt g 1)) > 64.0
          && near (col 0 (last (elemAt g 1))) (col 0 (first (elemAt g 2))) && col 1 (last (elemAt g 2)) > col 1 (last bowl))
      ];
      expected = [ 4 true true true true ];
    };
    testTheSquareAndCompassMakeARhombus = {
      expr = [
        # the square's corner is down, on the axis; the compass's hinge is up, on the axis
        (near (col 0 (last (first square))) 64.0 && col 1 (last (first square)) > 100.0)
        (near (col 0 (first (first compass))) 64.0 && col 1 (first (first compass)) < 20.0)
        # each is symmetric
        (near (col 0 (first (first square)) + col 0 (last (last square))) 128.0)
        (near (col 0 (last (first compass)) + col 0 (last (last compass))) 128.0)
      ];
      expected = [ true true true true ];
    };
    # Every line stays on the green field, its brush width included.
    testEverythingIsOnTheField = {
      expr = builtins.all
        (p: let r = 62.0 - col 2 p / 2.0; in fromCentre p < r * r)
        (lib.concatLists (g ++ square ++ compass));
      expected = true;
    };
    # The G stays on the blue globe, its brush width included.
    testTheGIsOnTheGlobe = {
      expr = builtins.all
        (p: let r = 38.0 - col 2 p / 2.0; in fromCentre p < r * r)
        (lib.concatLists g);
      expected = true;
    };
    # Every Desmos expression, read back from its LaTeX, stays within 0.01 of what it stands for.
    testTheDesmosFileIsTheDrawing = {
      expr = icon.drawing.worstSquared < 0.01 * 0.01;
      expected = true;
    };
  };
}

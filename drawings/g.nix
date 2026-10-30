# The icon of gramatico: a big G, as the G of the square and compass, in the colours of the flag
# of Brazil. A green disc is the field; on it the square (below) and the compass (above, its
# hinge on top) in yellow make the rhombus; over them a blue disc, the globe; and on the globe the
# G, in white.
#
# Each line is a brush stroke through hand-placed points with a width at each point, as
# piabrota/nix-os draws its 无 and h0ffmann/rax its 代理. The G is brushed like a pen letter:
# heavy on its left side, light where it starts at the top right, then the bar and the spur.
{ lib, pen }:
let
  inherit (builtins) elemAt;

  # The canvas, square: 128 × 128, centred on [64 64].
  w = 128.0;
  h = 128.0;

  # The flag of Brazil's four colours (Lei 5.700, de 1971, as the usual sRGB values).
  green = "#009C3B";
  yellow = "#FFDF00";
  blue = "#002776";
  white = "#FFFFFF";

  # A stroke from [x y width] points, in one ink.
  stroke = ink: pts:
    map (m: m // { inherit ink; })
      (pen.brush (map (p: [ (elemAt p 0) (elemAt p 1) ]) pts) (map (p: elemAt p 2) pts));

  # The square, its corner down: two arms of one width, meeting at the corner.
  square = [
    [ [ 24.0 76.0 8.0 ] [ 64.0 116.0 8.0 ] ]
    [ [ 64.0 116.0 8.0 ] [ 104.0 76.0 8.0 ] ]
  ];

  # The compass, its hinge up: two legs that taper to their points.
  compass = [
    [ [ 64.0 13.0 7.0 ] [ 45.0 55.5 5.5 ] [ 26.0 98.0 1.5 ] ]
    [ [ 64.0 13.0 7.0 ] [ 83.0 55.5 5.5 ] [ 102.0 98.0 1.5 ] ]
  ];

  # The G, in its writing order: the bowl from the top right round the left to the bottom right
  # (on a circle of radius 25 round the centre), the bar, the spur, and the serif on the top end.
  g = [
    [
      [ 83.2 47.9 3.8 ]
      [ 70.5 39.9 5.8 ]
      [ 55.4 40.5 7.2 ]
      [ 43.5 49.7 8.6 ]
      [ 39.0 64.0 9.2 ]
      [ 43.5 78.3 8.6 ]
      [ 55.4 87.5 7.2 ]
      [ 70.5 88.1 6.4 ]
      [ 85.7 76.5 6.4 ]
    ]
    [ [ 71.0 64.5 6.2 ] [ 87.5 64.5 6.2 ] ]
    [ [ 87.5 64.5 6.8 ] [ 87.5 82.0 6.8 ] ]
    [ [ 84.0 41.5 3.4 ] [ 84.0 54.0 3.4 ] ]
  ];
in
{
  strokes = { inherit square compass g; };
  inherit w h;

  drawing = pen.drawing {
    inherit w h;
    label = "gramatico: a big G, as in the square and compass, in the colours of the flag of Brazil";
    desc = "Drawn from the Desmos expressions of icon.desmos.json: cubic Béziers through hand-placed points, each piece at its brush width, and points for the discs.";
    layers = [
      # the field
      { dots = [{ at = [ 64.0 64.0 ]; r = 62.0; ink = green; }]; }
      # the rhombus: the square, the compass over it, and the compass's hinge with its pin
      {
        marks = lib.concatMap (stroke yellow) (square ++ compass);
        dots = [
          { at = [ 64.0 13.0 ]; r = 6.5; ink = yellow; }
          { at = [ 64.0 13.0 ]; r = 2.0; ink = green; }
        ];
      }
      # the globe
      { dots = [{ at = [ 64.0 64.0 ]; r = 38.0; ink = blue; }]; }
      # the G
      { marks = lib.concatMap (stroke white) g; }
    ];
  };
}

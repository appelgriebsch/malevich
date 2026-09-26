# Terminals

How a chart meets a terminal: the charset and color ladders, what detection
reads, and the overrides. The argument is
[Degradation is the contract](principles/degradation-is-the-contract.md).
This file is the mechanics.

## The charset ladder

A charset is the glyph tier the subpixel surface encodes through. Glyph
tables are data, not code.

| charset | subpixels per cell | standing |
|---|---|---|
| `Octants` | 2×4 solid blocks | Unicode 16; densest ink, explicit opt-in |
| `Sextants` | 2×3 solid blocks | Unicode 13; explicit opt-in |
| `Braille` | 2×4 dots | dense opt-in; dots, not blocks |
| `Quadrants` | 2×2 solid blocks | the conservative UTF-8 default |
| `HalfBlocks` | 1×2 | the lowest Unicode rung |
| `Ascii` | 1×1 | the guaranteed fallback |

The dense tiers are opt-in. No environment variable can prove the configured
font covers them. A terminal name is not a font. `Frame::detect` picks
quadrants in any UTF-8 environment, and ASCII otherwise. Octants, sextants,
and braille are choices you make for fonts you know.

The gallery's `charsets` example renders one curve at every rung:
`cargo run --example charsets`.

## The color ladder

Four tiers, quantized honestly downhill: `TrueColor`, `Ansi256`, `Ansi16`,
`Plain`. The sixteen-color pick is made in OKLab, by lightness and hue, the
two things sixteen colors can carry. A teal drops to green, never to the
grey nearer in RGB. Heatmap half-blocks carry independent upper and lower
colors. Plain output keeps an averaged shade on the charset's own ramp
(`░▒▓█`, or `.:#@` in ASCII). In colorless output, `color_by` categories
cycle portable marker shapes (`•`, `+`, `x`, `*`, `o`), so groups never
vanish in a pipe.

## What detection reads

`Frame::detect` sniffs. It never writes to the terminal. The rules, in
order:

- **Charset.** `MALEVICH_CHARSET` wins if set to a known name. `TERM=dumb`
  or `unknown` means ASCII. Then the locale, POSIX precedence: `LC_ALL`,
  `LC_CTYPE`, `LANG`. A locale without `utf` means ASCII. Otherwise
  quadrants.
- **Color.** `NO_COLOR` (any value) means plain. Output that is not a
  terminal means plain, unless `CLICOLOR_FORCE` or `FORCE_COLOR` is set and
  not `0`. `TERM=dumb` or `unknown` means plain. `COLORTERM=truecolor` or
  `24bit`, or a `TERM` ending in `-direct`, means truecolor. A `TERM`
  starting with `screen` caps at 256, since the multiplexer re-encodes
  what passes through it. A `TERM` containing `256color` means 256.
  Otherwise 16.
- **Size.** The terminal's reported size. The plot takes a third of the
  height. No terminal: `COLUMNS` and `LINES` when a shell exports them,
  else 80×16.
- **Theme.** `COLORFGBG` distinguishes dark from light backgrounds.

A pipe is clean plain text by default. Detection sees a non-terminal and
drops color, and the charset never emits anything a file cannot hold.

## Overrides

- `MALEVICH_CHARSET` — `ascii`, `half`, `quad`, `sextants`, `octants`,
  `braille`, or `auto`.
- `MALEVICH_GRAPHICS` — `kitty`, `sixel`, `iterm2`, or `none`: the pixel
  protocol, outranking the sniff and skipping the probe (feature `pixel`).
- `NO_COLOR` — force plain output ([no-color.org](https://no-color.org)).
- `CLICOLOR_FORCE` or `FORCE_COLOR` — keep color when piping.
- `COLUMNS` / `LINES` — size a render that has no terminal to measure.
- An explicit `Frame` — set in code. It consults nothing.

## Text discipline

- CJK labels are measured in display cells and stay aligned.
- Combining marks are deliberately dropped at the cell grid.
- Control characters are dropped at the cell grid. A title, label, or
  category carrying escape bytes can never smuggle them into any encoder's
  output. The only escapes in ANSI output are the encoder's own SGR
  sequences. A regression test pins this.
- `NaN` is always a visible gap, never interpolated away.

## Small frames

When the frame shrinks, furniture sheds before data: legend, then titles,
then tick density. The data region is the last thing standing, and
`TERM=dumb` at any width still gets a correct chart.

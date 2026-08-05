# kaz

**Pipe data to an honest terminal plot.** A stdin-first CLI over
[malevich](https://crates.io/crates/malevich): the first look at any data,
straight from the shell.

```sh
cat loss.tsv | kaz line -t training
awk '{print $5}' access.log | kaz hist
cut -f2 species.tsv | kaz count
```

The plot goes to **stderr** by default, so stdout stays the data channel — and
`-O` echoes the input through, so the plot can sit in the *middle* of a pipeline
without breaking it:

```sh
cat data.tsv | kaz line -O | next-tool     # plot on stderr, data flows on
```

```text
                        sine
20 ┤      ⡠⠤⠒⠉⠉⠉⠉⠉⠒⠤⣀
   │   ⡠⠔⠊           ⠉⠢⡀
   │ ⠔⠊                ⠈⠢⢄                       ⢀⡠⠊
10 ┤                      ⠑⠤⡀                  ⢀⠔⠁
   │                        ⠈⠢⢄⡀            ⢀⠤⠒⠁
   │                           ⠈⠢⣀⡀     ⢀⡠⠔⠊⠁
 0 ┤                              ⠈⠉⠉⠒⠉⠉⠁
   └┬───────────┬───────────┬──────────┬───────────┬
    0          10          20         30          40
```

`count` tallies bare labels — the log-wrangler's friend, no `sort | uniq -c`:

```sh
awk '{print $9}' access.log | kaz count -t 'status codes'
```

```text
              status codes
5 ┤  ███████
  │  ███████  ▁▁▁▁▁▁
  │  ███████  ██████
  │  ███████  ██████  ▂▂▂▂▂▂▂  ▂▂▂▂▂▂
0 ┤  ███████  ██████  ███████  ██████
  └─────────────────────────────────────
       200      404      301     500
```

## Install

```sh
cargo install malevich-cli      # installs the `kaz` binary
```

Shell completions (bash, zsh, fish) live in [`completions/`](completions/) and a
man page in [`man/kaz.1`](man/kaz.1) — for example:

```sh
cp completions/kaz.fish ~/.config/fish/completions/   # fish
source completions/kaz.bash                            # bash
man ./man/kaz.1
```

## Charts

| Command | Alias | What | Input shape |
|---|---|---|---|
| `line` | `l` | line chart, one line per series | `y` \| `xy` \| `xyy` \| `xyxy` \| `yx` |
| `scatter` | `s` | scatter plot | `xy` \| `xyy` |
| `bar` | `b` | one bar per label | `label value` |
| `hist` | — | histogram (`--bins N` to fix the count) | columns of numbers |
| `count` | `c` | value frequencies as bars | one column of labels |
| `density` | `d` | kernel density estimate | columns of numbers |
| `ecdf` | — | empirical cumulative distribution | columns of numbers |
| `box` | — | a box plot per column | columns are groups |
| `violin` | — | a violin plot per column | columns are groups |
| `hist2d` | — | 2D histogram (density grid) | `xy` |
| `heatmap` | — | shade a row-major matrix | rows of numbers |

`ecdf`, `violin`, and `hist2d` are charts no other CLI plotter ships.

## Input

Fields are separated by **any run of whitespace** by default — bare numbers,
TSV, and `column`-style output all just work. `-d CHAR` sets one explicit
separator (`-d,` for CSV-shaped data). `-H` reads a header row and uses its
names to label the series.

`--fmt` decides how columns map onto axes:

- `y` — each column is a y-series over its row index *(default: one column)*
- `xy` — first column x, second column y
- `xyy` — first column x, every remaining column a series *(default: 2+ columns)*
- `xyxy` — columns pair up: `(x0,y0) (x1,y1) …`
- `yx` — first column y, second column x (YouPlot compatibility)

A field that will not parse becomes an honest gap in the plot, and a one-line
tally (`3 values could not be parsed`) goes to stderr afterward — silenced with
`-q`. This parses *fields*, not CSV: for quotes and embedded delimiters, shape
the data upstream (`xsv select …`, `mlr --c2t …`) and pipe the result in.

## Options

```
-o TARGET      plot destination: stderr (default), - for stdout, or a FILE
-O             pass input through to stdout (mid-pipeline mode)
-d CHAR        field separator (default: any run of whitespace)
-H             first row is a header; its names label the series
--fmt FMT      column mapping: y | xy | xyy | xyxy | yx
-w N, -h N     frame width and height in cells (default: detected)
-t TITLE       plot title
--xlabel TEXT  --ylabel TEXT
--xlim A,B     --ylim A,B         fix an axis range
--log-x  --log-y
--time-x       read the x column as time (unix seconds or ISO 8601)
--bins N       histogram bin count (hist; default: automatic)
--color WHEN   auto | always | never
--charset SET  auto | ascii | half | quad | sextant | braille | octant
--pixels WHEN  auto | always | never   — sixel/kitty/iTerm2 image panel from a pipe
-q             suppress the unparsed-values tally
--version      --help
```

Color and glyph tier auto-detect from the destination stream, and where the
terminal speaks a pixel protocol the plot panel upgrades to a real image — even
mid-pipeline. `-h` is height; help is `--help` only.

## Live

`--live` reads stdin forever, one value per line, and repaints a sliding line in
place — no alt-screen, so the final frame stays in your scrollback, and Ctrl-C
restores the cursor. Line only.

```sh
ping -i.2 host | grep -oE 'time=[0-9.]+' | tr -d 'time=' | kaz line --live -t ping
vmstat 1 | awk 'NR>2{print $1}' | kaz line --live -t runnable
```

`--window N` sets the window length, `--fps N` the repaint rate (default 10),
and `--rate` plots the per-interval delta of a monotonic counter.

If a live plot looks frozen, the *producer* is buffering — pipes hold output
until a block fills. Unbuffer at the source: `stdbuf -oL producer`,
`grep --line-buffered`, or `awk '{print; fflush()}'`.

## Design

`kaz` contains **zero rendering logic**: it parses arguments, frames stdin, and
calls the public malevich API. Every flag names an existing library concept — a
frame field, a preset argument, a scale option, or plot furniture. It is the
proof of the library's central claim, that a pure string-renderer is enough.

## License

MIT or Apache-2.0, matching malevich.

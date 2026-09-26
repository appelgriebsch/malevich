# malevich

<section class="hero">

<div class="hero-text">

<h1>Terminal plotting for Rust.</h1>

<p class="lede">A small grammar of marks, honest axes, millions of points. A plot is a plain value. Hand it a frame and it draws; hand it another frame and it draws again. It still answers <code>TERM=dumb</code>, and it still answers a pipe.</p>

```sh
cargo add malevich
```

<p><a class="cta" href="guide/start/">Start here</a> <a class="cta red" href="playground/">Open the playground</a> <a class="cta" href="gallery/">See the gallery</a></p>

</div>

{{figure hero nocode}}

</section>

<section class="manifesto">

<div>

<h3>Eight marks, and that's the catalog</h3>

<p>Line, Points, Bars, Area, Cells, Range, Rule, Text. That's the alphabet. Give them a statistics layer and shared scales, and the charts with ordinary names — a histogram, a box plot, a density — are words you spell with it. A chart type is a preset: the short spelling. A test checks that the short spelling and the long one print the same bytes.</p>

</div>

<div>

<h3>Every brag has a test behind it</h3>

<p>The true chart draws every point. The fast one is welcome, as long as it lands on the same pixels, including a spike one sample wide. If we publish a number, a benchmark produced it. If you see a chart in the docs, or on this site, the program drew it, and the build diffs it. Nobody types the little blocks by hand.</p>

</div>

<div>

<h3>A bad terminal still gets a chart</h3>

<p>Real pixels when the terminal can draw them. Otherwise it steps down: octants, quadrants, then ASCII, which always works. Color fades the same way, from truecolor down to plain. It never fails, never sends a probe where escapes aren't safe, and never takes over the screen. A pipe gets clean text, the kind a log, a diff, or a language model can read.</p>

</div>

</section>

## One call, then the lid comes off

The front door is a preset. You can walk through it without knowing the grammar. `line(&values)` and `Plot::new().layer(Line::y(&values))` are the same call, the second one with the lid off, and moving from one to the other doesn't change a byte of the picture.

```rust
println!("{}", malevich::line(&[1.0, 5.0, 2.0, 8.0][..]));
```

{{figure start_line nocode}}

```rust
use malevich::{Frame, Line, Plot, Rule};

let steps: Vec<f64> = (0..100).map(f64::from).collect();
let loss: Vec<f64> = steps.iter().map(|s| 4.0 * (-0.05 * s).exp() + 0.4).collect();
let chart = Plot::new()
    .layer(Line::xy(&steps[..], &loss[..]).label("loss"))
    .layer(Rule::h(0.5).label("target"))
    .title("training");
println!("{}", chart.render(&Frame::plain(60, 14)));
```

`Plot::render` never fails. When the frame is too small for a title or a legend, those go first and the data stays, so you can build a plot inline with no error handling. `Frame::plain` gives a test the same string every time. `Frame::detect()` colors it and sizes it for the terminal you are actually in.

## The charts no other terminal library ships

<div class="tiles">

<div class="tile">

{{figure stat_kde nocode}}

<h3>A real statistics layer</h3>

<p>Quartiles the way the textbooks define them (type-7), and Tukey whiskers. A density from a real kernel. Least squares you can stream, with R² and a band around the line. ECDFs with a DKW band, ROC curves that tell you their area, and one <code>Reducer</code> word that means the same thing in a bin, a group, or a rolling window.</p>

</div>

<div class="tile">

{{figure scale_time_years nocode}}

<h3>Axes that are actually good</h3>

<p>Ticks placed by the extended Wilkinson algorithm, labeled with exact decimals you can parse back to the number. One SI prefix on an axis, not a new one every tick. Log axes by the decade. Calendar time that says <code>14:05</code>, or <code>Aug 2</code>, or <code>2027</code>, whichever the span actually wants. Band axes when what you have is a matrix.</p>

</div>

<div class="tile">

{{figure scale_colormap_log nocode}}

<h3>The ML set</h3>

<p>Attention maps and confusion matrices with the tokens written on the axes. Decision boundaries as cells of color. Images as RGB cells. A loss landscape with the optimizer's path on top of it. Each of these is a few marks put together. None of them got its own chart type.</p>

</div>

<div class="tile">

{{figure stat_m4 nocode}}

<h3>Millions of points, measured</h3>

<p>A long line is reduced with M4, one bucket per column on the screen, and the pixels match drawing every point. Ten million of them take tens of milliseconds on the <a href="benchmarks/">recorded baseline</a>. A spike one sample wide stays visible.</p>

</div>

<div class="tile">

{{figure stat_describe nocode}}

<h3>The first look is sometimes a table</h3>

<p><code>describe</code> prints the summary you usually compute before you plot anything, as a table. Text on two band axes, each column formatted like a tiny axis and lined up on the decimal. A gap is <code>—</code>.</p>

</div>

<div class="tile">

{{figure mark_bars_horizontal nocode}}

<h3>Sideways, stacked, grouped — composed</h3>

<p><code>Bars::horizontal</code> turns a bar on its side, <code>Bars::base</code> stacks the next one on top, <code>stat::dodge</code> sets them shoulder to shoulder. Volcano plots, Manhattan plots, candlesticks, waffles, and tornadoes are a few of those lines in the <a href="gallery/">gallery</a>. They never graduated to presets.</p>

</div>

</div>

## Everywhere cells go

One plot, a lot of doors. Escape codes for a tty. A `<pre>` of colored spans for a notebook. An SVG card for a README. Cells dropped into a ratatui or Ink buffer. A real sixel, kitty, or iTerm2 image when the terminal speaks one of those. From the shell, [`kaz`](cli/) pipes a column of numbers into that same renderer. From JavaScript, the [npm package](js/) is this engine compiled to wasm.

{{charsets start_layers | The same plot, rung by rung. A terminal's name can't prove the font has the dense blocks, so those rungs are something you ask for. Quadrants are the UTF-8 default. ASCII always works.}}

## Where to go

<div class="map">

<section>
<h3>Start here</h3>
<ul>
<li><a href="guide/start/">Getting started</a><span>Install, the first plot, the presets, and what a pipe does to it.</span></li>
<li><a href="guide/grammar/">The grammar</a><span>One chart, built up mark by mark, with a plate at every step.</span></li>
<li><a href="playground/">Playground</a><span>Paste numbers, pick a chart, resize the terminal, read the Rust it would take.</span></li>
</ul>
</section>

<section>
<h3>The guide</h3>
<ul>
<li><a href="guide/marks/">The eight marks</a><span>Every channel of every mark, one plate each.</span></li>
<li><a href="guide/stats/">The statistics layer</a><span>Bins, densities, quartiles, fits, windows, stacks, M4.</span></li>
<li><a href="guide/scales/">Scales and axes</a><span>Ticks that are exact. Time, log, band, and unit axes. Colormaps and palettes.</span></li>
<li><a href="guide/terminals/">Frames and terminals</a><span>The ladders, what detection is allowed to read, and a live explorer.</span></li>
<li><a href="guide/interaction/">Interaction</a><span>Zoom, pan, and crosshairs, without taking over the terminal.</span></li>
</ul>
</section>

<section>
<h3>Gallery</h3>
<ul>
<li><a href="gallery/">The gallery</a><span>Fifty-odd charts, in order, each one drawn by the program with its source beside it.</span></li>
<li><a href="gallery/live/">In the browser</a><span>Cells beside pixels, and ten million points you can zoom.</span></li>
</ul>
</section>

<section>
<h3>Why it is shaped this way</h3>
<ul>
<li><a href="principles/">Vision</a><span>The argument, and the five rules.</span></li>
<li><a href="principles/full-draw-oracle/">The full draw is the oracle</a><span>Why the fast chart and the honest chart have to be the same picture.</span></li>
<li><a href="guide/refusals/">What it will not be</a><span>No pies, no twin axes, no tick callbacks. And the thing you do instead.</span></li>
</ul>
</section>

<section>
<h3>Reference</h3>
<ul>
<li><a href="concepts/">Terminology</a><span>The vocabulary contract, illustrated.</span></li>
<li><a href="https://docs.rs/malevich">API reference</a><span>Every type and function, on docs.rs.</span></li>
<li><a href="cli/">kaz</a><span>The command line.</span></li>
<li><a href="js/">JavaScript</a><span>The npm package and the Ink widget.</span></li>
<li><a href="changelog/">Changelog</a><span>Every release, written for humans.</span></li>
</ul>
</section>

</div>

## The name

Kazimir Malevich painted a black square on a plain ground and meant it: a small vocabulary of geometric forms, composed deliberately. That is the design budget of this library — and, as it happens, a fair description of a terminal, which draws everything it will ever draw from a grid of small rectangles.

{{figure changelog_releases nocode}}

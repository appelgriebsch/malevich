# malevich

<section class="hero">

<div class="hero-text">

<h1>Terminal plotting for Rust.</h1>

<p class="lede">A small grammar of marks, honest axes, millions of points. A plot is a plain value; rendering is a pure function of that value and a frame; every terminal is answered, down to <code>TERM=dumb</code> and a pipe.</p>

```sh
cargo add malevich
```

<p><a class="cta" href="guide/start/">Start here</a> <a class="cta red" href="playground/">Open the playground</a> <a class="cta" href="gallery/">See the gallery</a></p>

</div>

{{figure hero nocode}}

</section>

<section class="manifesto">

<div>

<h3>The whole catalog, from eight marks</h3>

<p>Line, Points, Bars, Area, Cells, Range, Rule, Text. Marks × a statistics layer × shared scales compose into everything the basic chart catalog names. A chart type is a preset: a name for a grammar expansion, proven byte-identical to it in tests.</p>

</div>

<div>

<h3>Every claim provable</h3>

<p>The oracle is drawing every point; the fast path reproduces its pixels exactly. Every advertised number has a bench behind it. Every chart in the docs — and every chart on this site — is program output, regenerated and diffed. No figure is typed by hand.</p>

</div>

<div>

<h3>Every terminal answered</h3>

<p>Output degrades down declared ladders — real pixels, octants, quadrants, ASCII; truecolor to plain — and never fails, never probes where escapes are unsafe, never owns the screen. Piped output is clean text a log, a diff, or a language model can read.</p>

</div>

</section>

## One call, then the lid comes off

The front door is a preset. The grammar is discovered, not required: `line(&values)` and `Plot::new().layer(Line::y(&values))` are the same call with the lid off, and graduating from one to the other changes nothing about the output.

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

`Plot::render` never fails: it sheds what it cannot draw, so building a plot inline needs no error handling. Pass `Frame::plain` in a test and the string is deterministic; pass `Frame::detect()` and it is colored and sized for the terminal you are in.

## The charts no other terminal library ships

<div class="tiles">

<div class="tile">

{{figure stat_kde nocode}}

<h3>A real statistics layer</h3>

<p>Type-7 quartiles and Tukey whiskers, densities from a real KDE, streaming least squares with R² and a confidence band, ECDFs with a DKW band, ROC curves with their area, and one <code>Reducer</code> vocabulary across bins, groups, and rolling windows.</p>

</div>

<div class="tile">

{{figure scale_time_years nocode}}

<h3>Axes that are actually good</h3>

<p>Extended-Wilkinson tick placement, exact-decimal labels that parse back to their values, one SI prefix per axis, log decades, calendar time that says <code>14:05</code> or <code>Aug 2</code> or <code>2027</code> as the span demands, and band axes for matrices.</p>

</div>

<div class="tile">

{{figure scale_colormap_log nocode}}

<h3>The ML set</h3>

<p>Attention maps and confusion matrices on token-labeled band axes, decision boundaries as categorical cells, images as RGB cells, loss landscapes with optimizer trajectories — every one a grammar composition, none a preset.</p>

</div>

<div class="tile">

{{figure stat_m4 nocode}}

<h3>Millions of points, measured</h3>

<p>Large lines reduce by M4, bucketed by the rendered column, pixel-identical to drawing every point. Ten million points render in tens of milliseconds on the <a href="benchmarks/">recorded baseline</a>. A one-sample spike cannot vanish.</p>

</div>

<div class="tile">

{{figure stat_describe nocode}}

<h3>The first look is sometimes a table</h3>

<p><code>describe</code> renders the summary that usually precedes a chart as a stat table: text on two band axes, every column formatted like a tiny axis and aligned at the decimal point, gaps as <code>—</code>.</p>

</div>

<div class="tile">

{{figure mark_bars_horizontal nocode}}

<h3>Sideways, stacked, grouped — composed</h3>

<p><code>Bars::horizontal</code> is a channel, <code>Bars::base</code> stacks, <code>stat::dodge</code> groups. Volcano plots, Manhattan plots, candlesticks, waffles, and tornadoes are a few grammar lines each in the <a href="gallery/">gallery</a>, never presets.</p>

</div>

</div>

## Everywhere cells go

The same plot value renders as escape codes for a tty, as a `<pre>` of colored spans for a notebook, as an SVG card for a README, as cells into a ratatui or Ink buffer, and as a real sixel, kitty, or iTerm2 image where the terminal speaks one. From the shell, [`kaz`](cli/) pipes any column of numbers into the same renderer; from JavaScript, the [npm package](js/) is the same engine compiled to wasm.

{{charsets start_layers | The same plot value at every rung of the charset ladder. A terminal name cannot prove font coverage, so the dense tiers are explicit choices; quadrants are the UTF-8 default and ASCII always works.}}

## Where to go

<div class="map">

<section>
<h3>Start here</h3>
<ul>
<li><a href="guide/start/">Getting started</a><span>Install, the first plot, the presets, and what happens when you pipe it.</span></li>
<li><a href="guide/grammar/">The grammar</a><span>One chart built up mark by mark, with a plate for every step.</span></li>
<li><a href="playground/">Playground</a><span>Paste numbers, pick a chart, resize the terminal, read the Rust it would take.</span></li>
</ul>
</section>

<section>
<h3>The guide</h3>
<ul>
<li><a href="guide/marks/">The eight marks</a><span>Every channel of every mark, one plate each.</span></li>
<li><a href="guide/stats/">The statistics layer</a><span>Bins, densities, quartiles, fits, windows, stacks, M4.</span></li>
<li><a href="guide/scales/">Scales and axes</a><span>Ticks that are exact; time, log, band, and unit axes; colormaps and palettes.</span></li>
<li><a href="guide/terminals/">Frames and terminals</a><span>The ladders, detection, and a live explorer.</span></li>
<li><a href="guide/interaction/">Interaction</a><span>Zoom, pan, and crosshairs without owning the terminal.</span></li>
</ul>
</section>

<section>
<h3>Gallery</h3>
<ul>
<li><a href="gallery/">The gallery</a><span>Fifty-odd charts as a ladder, every one real program output with its source.</span></li>
<li><a href="gallery/live/">In the browser</a><span>Cells beside pixels, and ten million points you can zoom.</span></li>
</ul>
</section>

<section>
<h3>Why it is shaped this way</h3>
<ul>
<li><a href="principles/">Vision</a><span>The argument and the five rules.</span></li>
<li><a href="principles/full-draw-oracle/">The full draw is the oracle</a><span>Why the fast path and the honest path are the same path.</span></li>
<li><a href="guide/refusals/">What it will not be</a><span>No pies, no twin axes, no tick callbacks — and the answer that exists instead.</span></li>
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

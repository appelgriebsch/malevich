//! The page table: every URL on the site, what it is built from, and how the
//! navigation groups it.

/// Where a page's body comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// A markdown file under `site/content/`.
    Content(&'static str),
    /// A markdown file in the repository, rendered with its links rewritten.
    Repo(&'static str),
    /// A repository markdown file with site material inserted after named
    /// headings: the repository stays the single source of the text, the
    /// site adds the plates.
    RepoWith(&'static str, &'static [(&'static str, &'static str)]),
    /// The front page.
    Home,
    /// The gallery, built from `EXAMPLES.md` and the example sources.
    Gallery,
    /// The in-browser plates, drawn by the wasm build.
    Live,
    /// The playground.
    Playground,
}

/// One page.
#[derive(Debug, Clone, Copy)]
pub struct Page {
    /// The site-absolute URL, always a directory (`/guide/marks/`).
    pub url: &'static str,
    /// The title, as the navigation and the browser tab show it.
    pub title: &'static str,
    /// One line for the section index and the search results.
    pub blurb: &'static str,
    /// Where the body comes from.
    pub source: Source,
}

impl Page {
    /// The output path relative to the output directory.
    pub fn path(&self) -> String {
        format!("{}index.html", self.url.trim_start_matches('/'))
    }

    /// The relative prefix that reaches the site root from this page.
    pub fn root(&self) -> String {
        let depth = self
            .url
            .trim_matches('/')
            .split('/')
            .filter(|part| !part.is_empty())
            .count();
        "../".repeat(depth)
    }

    /// The repository directory relative links in this page's source resolve
    /// against.
    pub fn source_dir(&self) -> &'static str {
        match self.source {
            Source::Repo(path) | Source::RepoWith(path, _) => {
                path.rsplit_once('/').map_or("", |(dir, _)| dir)
            }
            _ => "",
        }
    }

    /// Whether the search index lists the page.
    pub fn searchable(&self) -> bool {
        !matches!(self.source, Source::Home)
    }
}

/// A navigation section.
pub struct Section {
    pub title: &'static str,
    pub pages: &'static [Page],
}

const fn page(url: &'static str, title: &'static str, blurb: &'static str, source: Source) -> Page {
    Page {
        url,
        title,
        blurb,
        source,
    }
}

/// The navigation, in order.
pub const SECTIONS: &[Section] = &[
    Section {
        title: "Start here",
        pages: &[
            page(
                "/",
                "malevich",
                "Terminal plotting for Rust: a small grammar of marks, honest axes, millions of points.",
                Source::Home,
            ),
            page(
                "/guide/start/",
                "Getting started",
                "Install, the first plot, the presets, and what happens when you pipe it.",
                Source::Content("guide/start.md"),
            ),
            page(
                "/playground/",
                "Playground",
                "Paste numbers, pick a chart, resize the terminal, and read the Rust it would take.",
                Source::Playground,
            ),
        ],
    },
    Section {
        title: "The guide",
        pages: &[
            page(
                "/guide/grammar/",
                "The grammar",
                "Layers on shared scales: one chart built up mark by mark.",
                Source::Content("guide/grammar.md"),
            ),
            page(
                "/guide/marks/",
                "The eight marks",
                "Line, Points, Bars, Area, Cells, Range, Rule, Text — every channel, with a plate each.",
                Source::Content("guide/marks.md"),
            ),
            page(
                "/guide/stats/",
                "The statistics layer",
                "Bins, densities, quartiles, fits, windows, stacks, M4: what runs before the scales see the data.",
                Source::Content("guide/stats.md"),
            ),
            page(
                "/guide/scales/",
                "Scales and axes",
                "Linear, integer, log, time, and band axes; ticks that are exact; colormaps and palettes.",
                Source::Content("guide/scales.md"),
            ),
            page(
                "/guide/furniture/",
                "Furniture",
                "Titles, labels, legends, colorbars, and what sheds first when the frame shrinks.",
                Source::Content("guide/furniture.md"),
            ),
            page(
                "/guide/terminals/",
                "Frames and terminals",
                "The charset and color ladders, what detection reads, and the overrides — with a live explorer.",
                Source::RepoWith(
                    "docs/terminal.md",
                    &[
                        (
                            "## The charset ladder",
                            "{{charsets start_layers | One plot value at every rung. Block tiers become crisp rectangles on this page because the SVG card draws them as the rectangles they denote; braille and box drawing stay text the font draws.}}",
                        ),
                        (
                            "## The color ladder",
                            "{{colors ladder_heat | The encoder's own SGR bytes at every color mode, decoded the way a terminal would draw them: truecolor, the 256-color cube, the sixteen named colors picked in OKLab, and the plain shade ramp with marker cycling.}}\n\n{{explorer}}",
                        ),
                        ("## Small frames", "{{resizer}}"),
                    ],
                ),
            ),
            page(
                "/guide/composition/",
                "Composition",
                "Small multiples, tables beside charts, shared windows, and the pie's honest forms.",
                Source::Content("guide/composition.md"),
            ),
            page(
                "/guide/interaction/",
                "Interaction",
                "Mapping, Viewport, and the widget: zoom, pan, and crosshairs without owning the terminal.",
                Source::RepoWith(
                    "docs/interaction.md",
                    &[(
                        "## The physics",
                        "{{pair inter_full inter_zoomed | A zoom is a domain window. The whole series on the left; a `Viewport` over three thousand of its hundred thousand points on the right, where M4 re-aggregates to the new columns and the ripple the wide view could only hint at is drawn in full.}}",
                    )],
                ),
            ),
            page(
                "/guide/pixels/",
                "Real pixels",
                "Sixel, kitty, and iTerm2 panels with text chrome around them.",
                Source::RepoWith(
                    "docs/pixels.md",
                    &[(
                        "## Turn it on",
                        "![Loss curves, a calendar time axis, and smoothing: cell rendering beside pixel rendering](../examples/showcase-lines.png)\n\n![A 2D density, contour lines, and a vector field: cell rendering beside pixel rendering](../examples/showcase-2d.png)\n\n*The showcase in a kitty terminal: every chart twice, cells on the left and the same plot value as a real image on the right. Title, axes, and legend stay text; only the plot rectangle becomes pixels.*",
                    )],
                ),
            ),
            page(
                "/guide/notebooks/",
                "Notebooks and cards",
                "Evcxr, the HTML card, the SVG card, and the terminal-card contract.",
                Source::RepoWith(
                    "docs/notebooks.md",
                    &[
                        (
                            "## What a cell shows",
                            "{{html start_layers | The HTML card itself, embedded in this page exactly as `Plot::to_html` emitted it: one `<pre>` of colored spans, nothing external.}}",
                        ),
                        (
                            "## Custom frames",
                            "{{svgsource start_line}}\n\n{{light start_layers | The SVG card on the light theme. Every figure on this site is this card, inlined.}}",
                        ),
                        (
                            "## The terminal-card contract",
                            "{{plain start_layers | The same grid, colorless: what a stripped card, a log, or a language model reads.}}",
                        ),
                    ],
                ),
            ),
            page(
                "/guide/streaming/",
                "Live charts",
                "A sliding window, an in-place repaint, and a CLI that plots forever.",
                Source::Content("guide/streaming.md"),
            ),
            page(
                "/guide/serde/",
                "Specs as data",
                "Documents: the versioned envelope a plot travels in.",
                Source::RepoWith(
                    "docs/serde.md",
                    &[(
                        "## Version 1",
                        "{{figure start_layers nocode}}\n\nThe plot above, as the document `Document::plot` produces for it — every layer, scale, and piece of furniture, with the series inline and gaps as `null`:\n\n{{json start_layers}}",
                    )],
                ),
            ),
            page(
                "/guide/performance/",
                "Performance",
                "M4 to the raster, bucket-exact grids, and the numbers with a bench behind them.",
                Source::RepoWith(
                    "docs/performance.md",
                    &[
                        (
                            "## The mechanisms",
                            "{{pair stat_m4 stat_stride | Why speed and honesty are the same claim. M4 (left) keeps first, last, minimum, and maximum per rendered column, so the three one-sample spikes survive by construction. A stride sampler (right) is just as fast and silently lost all three.}}\n\n{{figure mark_cells_extents nocode}}",
                        ),
                        (
                            "## Measured",
                            "{{figure perf_bench | The benchmark table, drawn by the library it measures: horizontal bars on a log axis with an SI unit, from the 2026-09-24 baseline.}}",
                        ),
                    ],
                ),
            ),
            page(
                "/guide/recipes/",
                "Recipes",
                "Benchmarks through jq, the pie, tornado bars, two scales, and out-of-range rules.",
                Source::RepoWith(
                    "docs/recipes.md",
                    &[
                        (
                            "## Tornado and breakdown bars",
                            "{{figure comp_tornado nocode}}",
                        ),
                        (
                            "## Two series, two scales: a `Grid` and a shared window",
                            "{{pair comp_shared_a comp_shared_b}}",
                        ),
                        (
                            "## Positions clip, colors squish",
                            "{{pair scale_domain_fixed scale_colormap_domain | Positions clip (left): the curve leaves the fixed window and is drawn nowhere. Colors squish (right): values past the fixed color domain take the `under` and `over` colors, and the colorbar shows the range.}}",
                        ),
                        ("## Plain text is agent-legible", "{{plain hero}}"),
                    ],
                ),
            ),
            page(
                "/guide/refusals/",
                "What it will not be",
                "The requests it declines, each with its reason and the answer that exists.",
                Source::Content("guide/refusals.md"),
            ),
        ],
    },
    Section {
        title: "Gallery",
        pages: &[
            page(
                "/gallery/",
                "The gallery",
                "Fifty-odd charts as a ladder, every one real program output with its source.",
                Source::Gallery,
            ),
            page(
                "/gallery/live/",
                "In the browser",
                "The same engine as wasm: cells beside pixels, and ten million points you can zoom.",
                Source::Live,
            ),
        ],
    },
    Section {
        title: "Why it is shaped this way",
        pages: &[
            page(
                "/principles/",
                "Vision",
                "The argument and the five rules.",
                Source::Repo("docs/vision.md"),
            ),
            page(
                "/principles/presets-are-packaging/",
                "Presets are packaging",
                "A preset is a name for a grammar expansion, proven byte-identical to it.",
                Source::Repo("docs/principles/presets-are-packaging.md"),
            ),
            page(
                "/principles/frame-is-run-state/",
                "The frame is run state",
                "A plot describes a chart; a frame describes one rendering of it.",
                Source::Repo("docs/principles/frame-is-run-state.md"),
            ),
            page(
                "/principles/full-draw-oracle/",
                "The full draw is the oracle",
                "Anything faster than drawing every point must reproduce its pixels exactly.",
                Source::Repo("docs/principles/full-draw-oracle.md"),
            ),
            page(
                "/principles/what-earns-a-concept/",
                "What earns a concept",
                "Real charts demand it, and no composition of the rest can draw it.",
                Source::Repo("docs/principles/what-earns-a-concept.md"),
            ),
            page(
                "/principles/degradation-is-the-contract/",
                "Degradation is the contract",
                "Every terminal gets the best chart it can carry, and no terminal gets a failure.",
                Source::Repo("docs/principles/degradation-is-the-contract.md"),
            ),
            page(
                "/principles/axes-are-the-product/",
                "The axes are the product",
                "The differentiators live exactly where everyone else got bored.",
                Source::Repo("docs/principles/axes-are-the-product.md"),
            ),
            page(
                "/principles/conversion-at-the-rim/",
                "Conversion lives at the rim",
                "The core computes in f64; every other numeric shape converts once, at ingestion.",
                Source::Repo("docs/principles/conversion-at-the-rim.md"),
            ),
        ],
    },
    Section {
        title: "Reference",
        pages: &[
            page(
                "/concepts/",
                "Terminology",
                "The vocabulary contract: every public concept, what it means, and what it maps to — illustrated.",
                Source::RepoWith(
                    "docs/terminology.md",
                    &[
                        ("## Plot", "{{figure hero nocode}}"),
                        ("## Layer", "{{figure grammar_3 nocode}}"),
                        ("## Mark", "{{figure mark_rule nocode}}"),
                        ("## Channel", "{{figure mark_points_color_by nocode}}"),
                        ("## Series", "{{figure start_gap nocode}}"),
                        ("## Stat", "{{figure stat_window nocode}}"),
                        ("## Reducer", "{{figure stat_binned nocode}}"),
                        ("## Scale", "{{figure scale_log nocode}}"),
                        ("## Ticks", "{{figure scale_context nocode}}"),
                        (
                            "## Frame",
                            "{{sizes start_layers 60x12 36x8 | One plot, two frames.}}",
                        ),
                        ("## Viewport", "{{pair inter_full inter_zoomed}}"),
                        ("## Card", "{{light hero}}"),
                        ("## Charset", "{{charsets start_line}}"),
                        (
                            "## Theme",
                            "{{pair palette_okabe_ito palette_muted | The categorical `Palette` lives in the spec; the `Theme` is the frame's.}}",
                        ),
                        ("## Preset", "{{figure grammar_preset nocode}}"),
                        ("## Stream", "{{figure stream_tail nocode}}"),
                    ],
                ),
            ),
            page(
                "/cli/",
                "kaz, the command line",
                "Pipe data to an honest plot: one subcommand per chart, plot on stderr, data flows on.",
                Source::Repo("cli/README.md"),
            ),
            page(
                "/js/",
                "JavaScript",
                "The same engine as wasm on npm, with an Ink widget.",
                Source::Repo("js/README.md"),
            ),
            page(
                "/benchmarks/",
                "Benchmarks",
                "The dated record behind every number the docs quote.",
                Source::Repo("BENCHMARKS.md"),
            ),
            page(
                "/changelog/",
                "Changelog",
                "Every release, written for humans.",
                Source::Repo("CHANGELOG.md"),
            ),
            page(
                "/acknowledgements/",
                "Acknowledgements",
                "The algorithms, libraries, and grammars this project learned from.",
                Source::Repo("ACKNOWLEDGEMENTS.md"),
            ),
        ],
    },
];

/// Every page, flattened.
pub fn all() -> impl Iterator<Item = &'static Page> {
    SECTIONS.iter().flat_map(|section| section.pages.iter())
}

/// The site URL a repository path is published at, if any.
pub fn url_for(repo_path: &str) -> Option<&'static str> {
    let path = repo_path.trim_start_matches("./");
    match path {
        "README.md" => return Some("/"),
        "EXAMPLES.md" => return Some("/gallery/"),
        "docs/README.md" => return Some("/guide/start/"),
        "docs/terminology.md" | "TERMINOLOGY.md" => return Some("/concepts/"),
        "docs/terminal.md" => return Some("/guide/terminals/"),
        "docs/notebooks.md" => return Some("/guide/notebooks/"),
        "docs/serde.md" | "SERDE.md" => return Some("/guide/serde/"),
        "docs/performance.md" => return Some("/guide/performance/"),
        "docs/principles" | "docs/principles/" => return Some("/principles/"),
        "gallery/README.md" | "site/README.md" => return Some("/gallery/live/"),
        "demos/README.md" | "demos" | "demos/" => return Some("/guide/interaction/"),
        "js/examples" | "js/examples/" => return Some("/js/"),
        "cli" | "cli/" => return Some("/cli/"),
        _ => {}
    }
    all()
        .find(|page| matches!(page.source, Source::Repo(source) | Source::RepoWith(source, _) if source == path))
        .map(|page| page.url)
}

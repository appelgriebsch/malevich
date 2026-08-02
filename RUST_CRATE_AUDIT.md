# Malevich Rust Crate Audit

**Audit date:** 2026-08-02  
**Target:** the current working tree of malevich 0.10.0  
**Primary focus:** public API design, correctness, robustness, and engineering quality  
**Method:** source review, public-API inspection, build/lint/doc checks, and targeted runtime probes

> Line numbers in this report refer to the audited working tree and may move after edits.
> The tree already contained uncommitted ndarray and quiver work. This audit treats that
> state as the target and does not attribute or alter those changes.

## Executive summary

Malevich has a notably coherent core: a small grammar of marks, explicit Frame-driven
rendering, a useful Cow-based ownership model, no unsafe code, limited default
dependencies, and broad happy-path tests. All targets compile with all features, 190
library tests pass, and all 21 doctests pass.

The current tree is nevertheless **not ready to publish unchanged**. The audit found:

- **6 high-severity correctness findings**, including silent histogram data loss,
  incorrect Moments::default extrema, invalid serde states that later panic, incorrect
  fixed-domain behavior, potentially unbounded off-screen raster loops, and M4 gap
  corruption.
- **9 medium-severity correctness findings**, concentrated in extreme numeric inputs,
  log scales, grid sizing, degenerate 2D data, categorical-scale precedence, and
  paired-series validation.
- **2 low-severity robustness findings**, plus substantial API-design and release-quality
  debt.
- The current tree fails formatting, generated-gallery consistency, all-feature Clippy,
  and warnings-as-errors rustdoc checks.

The highest-value corrective theme is to establish a validated boundary before
rendering. Constructors currently enforce many invariants, but deserialization,
cross-option combinations, transforms, and rendering itself can bypass or violate
them. A small fallible validation layer would eliminate several otherwise unrelated
panic and silent-corruption paths.

### Recommended release decision

Before publishing this working tree:

1. Fix COR-01 through COR-06 and add focused regressions.
2. Make formatting, the example gallery, all-feature Clippy, and strict rustdoc clean.
3. Add validation for serde payloads and incompatible scale/domain combinations.
4. Replace the circular M4 rendering test with a true raw-render reference.
5. Decide and document the crate's MSRV and serialized-format policy.

## Scope and limitations

This is a crate-level design and correctness audit, not a formal security assessment or
proof of numerical accuracy. It covers:

- the public modules and re-exports;
- marks, plot layout, rasterization, scales, statistics, presets, streaming, serde,
  ratatui, and ndarray integration;
- documented contracts and their test coverage;
- behavior on small frames, invalid combinations, mismatched series, non-finite data,
  large magnitudes, degenerate distributions, and malformed serialized values;
- the repository's build, lint, documentation, and release checks.

It does not benchmark every algorithm, test every terminal emulator, or validate visual
perception claims experimentally. cargo-audit and cargo-deny were not installed, and a
full dependency-index operation was unavailable in the environment, so this report
does **not** claim that the dependency graph is advisory- or license-clean.

## Verification results

| Check | Result | Notes |
|---|---:|---|
| cargo fmt --all -- --check | **Fail** | Formatting drift in src/presets.rs around the quiver call |
| cargo check --all-targets | Pass | Default feature set |
| cargo check --all-targets --all-features | Pass | serde, ratatui, and ndarray included |
| cargo test --all-targets --all-features | Pass | 190 library tests; examples and benches also built in test mode |
| cargo test --doc --all-features | Pass | 21 doctests |
| cargo clippy --all-targets -- -D warnings | Pass | Matches the current CI's default-feature lint scope |
| cargo clippy --lib --all-features -- -D warnings | Pass | Library code is clean |
| cargo clippy --all-targets --all-features -- -D warnings | **Fail** | Approximate constant 6.28 in src/data/tests/ndarray_tests.rs:25 |
| cargo doc --no-deps --all-features | Pass with warning | Ambiguous intra-doc link line at src/lib.rs:5 |
| RUSTDOCFLAGS=-Dwarnings cargo doc --no-deps --all-features | **Fail** | The ambiguous link becomes an error |
| cargo run --quiet --example regen_docs -- --check | **Fail** | New quiver example is absent from the generated gallery registry |
| Stable Rust 1.90 all-target/all-feature check | Pass | Useful observation, but not an MSRV guarantee |
| Stable Rust 1.90 all-feature doctests | Pass | Same limitation |
| Automated dependency advisory/license scan | Not run | cargo-audit and cargo-deny unavailable |

## Severity model

- **High:** can silently produce materially wrong results, violate a central documented
  rendering contract, panic from accepted external state, or consume unbounded work for
  ordinary public API inputs.
- **Medium:** incorrect or panic-prone behavior under valid but less common inputs, an
  important cross-option inconsistency, or a substantial API trap.
- **Low:** bounded robustness defect, contract drift, or maintainability issue with
  limited immediate impact.

## Correctness and robustness findings

### COR-01 — Fixed domains are widened and marks are not clipped to the plot rectangle

**Severity: High**

Plot::x_domain and Plot::y_domain promise fixed limits and say outside data “clips
honestly” in src/plot/plot.rs:53-81. Layout instead expands the supplied domain to the
first and last generated ticks:

- src/plot/layout.rs:161 expands y;
- src/plot/layout.rs:182-185 expands x when ticks are present;
- src/plot/layout.rs:224-228 implements the expansion.

Rendering then adds the plot offset and sends primitives to Surface. Surface::line clips
to the **entire frame** at src/render/surface.rs:92-106, not the plot rectangle. Points
and text similarly have no plot-local clip.

Two targeted probes confirmed both effects:

- a plot fixed to x = [0.1, 0.9] rendered differently when points at x = 0 and x = 1
  were present, proving that the fixed range was widened;
- with x/y fixed to [0, 1], a slightly out-of-range point changed a cell in the gutter,
  proving that mark ink can escape the data region.

**Impact**

- Explicit limits are not reliable for comparison plots or zooming.
- Out-of-domain data can overwrite otherwise blank gutter/chrome cells.
- Tick choice changes data visibility, coupling presentation to semantics.

**Recommendation**

Keep manual domains exact. Generate or filter ticks inside those bounds without feeding
tick extents back into the scale domain. Introduce a plot-rectangle clip object and
apply it to every mark primitive before adding frame offsets. Add regressions for every
mark type on all four plot edges and for reversed domain arguments.

### COR-02 — Off-screen area and bar spans can perform effectively unbounded loops

**Severity: High**

Surface line drawing clips before rasterization, but several higher-level mark loops
enumerate mapped coordinates before Surface can reject them:

- draw_bars iterates left..right at src/plot/draw.rs:365-378;
- draw_area iterates every rounded subpixel between consecutive mapped points at
  src/plot/draw.rs:628-645;
- range bodies also enumerate a horizontal span at src/plot/draw.rs:469-479.

With a fixed narrow domain and very distant finite data, scale mapping can produce
enormous finite coordinates that saturate to i64. A range over those coordinates can
take impractically long even though every eventual cell is off-screen.

**Impact**

Accepted public inputs can hang a render thread or cause denial of service when plots
are built from untrusted or merely poorly scaled data.

**Recommendation**

Clamp each span to the plot's local subpixel bounds **before** constructing an integer
range. Reject non-finite mapped values. Centralize bounded span conversion so bars,
areas, ranges, rules, and future marks use the same checked path. Add timeout-backed
tests with values near f64 limits and with domains many orders of magnitude narrower
than the data.

### COR-03 — M4 cannot preserve a gap that occurs inside a raster bucket

**Severity: High**

M4's bucket stores only first, last, min, max, and one boolean gap flag. A non-finite y
sets that flag at src/stat/m4.rs:59-63. emit then writes a single NaN **before** all
retained points from the bucket at src/stat/m4.rs:125-137.

That representation cannot distinguish:

- gap before the first finite point;
- gap between two finite points;
- gap after the last finite point;
- multiple finite segments in one bucket.

When a gap falls between finite values in a bucket, emit can reconnect values that were
separated in the input. A 1,000-point monotonic-x probe with a NaN between a jump from
-1 to +1 rendered differently under automatic M4 downsampling than under raw
rendering.

The existing “full versus reduced” plot test does not catch this because the purported
full plot is itself large enough to trigger the same automatic M4 path. It compares M4
against M4, not M4 against the unaggregated raster.

**Impact**

Missing-data boundaries can become visible lines. The documented “pixel-exact” and
gap-preserving claims are false for this case.

**Recommendation**

Aggregate finite segments rather than only buckets, or flush/reset a bucket at every
gap so segment boundaries remain ordered. Build a test-only raw-render path and compare
M4 output against it. Add property tests placing gaps at every position within every
bucket, including multiple gaps and gaps at bucket boundaries.

### COR-04 — Derived serde deserialization bypasses constructor invariants

**Severity: High**

Public types derive Deserialize directly over private representation. This permits
states that normal constructors reject. Confirmed examples:

- Colormap with an empty stops array deserializes, then Colormap::color panics while
  indexing stops at src/scale/colormap.rs:51-55.
- Grid with columns = 0 deserializes, then Grid::render divides by zero at
  src/plot/grid.rs:65-66.
- A Range whose x/low/high lengths differ deserializes inside a Plot, then rendering
  indexes past an array at src/plot/draw.rs:457-471.

Equivalent risks apply anywhere construction-time assertions are the only invariant
guard: mark lengths, matrix geometry, domains, and nested plot state.

**Impact**

The optional serde API accepts syntactically valid payloads that panic later, often far
from deserialization. Persisted or network-provided plot specifications are therefore
unsafe to render without an undocumented validation pass.

**Recommendation**

Deserialize into explicit wire structs and validate via TryFrom, or use serde's
try_from/into support. Reject zero dimensions, inconsistent series lengths, invalid
colormaps, non-finite/invalid domains, and incompatible scale combinations at the
boundary. Add malformed-payload tests and a deserialize-then-render fuzz target.

### COR-05 — Bins::auto can silently discard finite observations and exceed its cap

**Severity: High**

The documentation says automatic bins cover the data and are capped at limit
(src/stat/bin.rs:36-39). The implementation selects a nice width, computes the required
bin count, then truncates the count with:

    bins.min(limit.max(1) * 2)

at src/stat/bin.rs:83 without widening bins or shifting the end. Bins::add silently
ignores values beyond that truncated range at src/stat/bin.rs:90-105.

A deterministic parameter sweep found a finite 101-value input with limit = 1 for
which the resulting counts summed to 96. The function also returned two bins for some
limit = 1 cases, directly violating “capped at limit.”

**Impact**

Histograms can undercount data without any error, and the public limit contract is
unreliable. The hist preset inherits the undercount.

**Recommendation**

After snapping to a nice width, recompute a width/start/end that covers [min, max]
within the requested cap. Make these postconditions executable tests:

- sum(counts) equals the number of finite inputs;
- counts.len() is at most max(limit, 1);
- start is at most min and end is at least max;
- the maximum is counted exactly once.

Use property tests across offsets, spans, subnormal values, large magnitudes, and limits
from zero upward. Use checked arithmetic for limit-derived sizes.

### COR-06 — Moments::default is not an empty Moments accumulator

**Severity: High**

Moments derives Default at src/stat/moments.rs:9, which initializes min and max to 0.
Moments::new initializes them to +infinity and -infinity at lines 20-27.

Confirmed behavior:

- Moments::new followed by add(5) reports min = 5, max = 5;
- Moments::default followed by add(5) reports min = 0, max = 5;
- the symmetric error occurs for all-negative streams, where max remains 0.

**Impact**

A conventional and advertised Rust construction path silently produces incorrect
statistics. Derive(Default) is especially likely to be used through generic code.

**Recommendation**

Remove the derive and implement Default by calling Moments::new. Add an equivalence
test between new and default before and after positive-only, negative-only, mixed, and
merged streams.

### COR-07 — KDE can panic for finite, large-offset degenerate samples

**Severity: Medium**

kde uses a naive mean/variance pass at src/stat/kde.rs:15-18. For a constant sample at
1e20 it selects fallback bandwidth 1, but adding/subtracting 3 is below the value's ULP.
start and end therefore round to the same number, step becomes zero, and the kernel
radius conversion at lines 65-71 attempts an enormous allocation.

The probe kde(&[1e20], 16) panicked with “capacity overflow.”

**Recommendation**

Center data before moment and grid calculations, use the existing stable Moments
algorithm, and explicitly require finite positive step/bandwidth before allocation.
Cap the kernel radius relative to points. Return a documented degenerate density or an
error rather than panicking.

### COR-08 — Tick generation has contradictory contracts and extreme-input panics

**Severity: Medium**

Confirmed issues:

- Ticks::linear(-f64::MAX, f64::MAX, 6) panics in debug arithmetic around
  src/scale/ticks.rs:219, even though both documented bounds are finite.
- 10 * target + 10 at src/scale/ticks.rs:199 can overflow usize for a large target.
- Ticks::time(1.1, 3.2, 2) returns no ticks because the selected aligned interval has
  no boundary inside the range.
- len says there is at least one tick and is_empty says it is never true at
  src/scale/ticks.rs:152-159.
- step says it is adjacent spacing and zero only for a singleton, but log and time
  ticks always store zero at src/scale/ticks.rs:134 and src/scale/ticks.rs:137-140.

**Recommendation**

Use checked/saturating target arithmetic and overflow-safe range calculations. Define
whether endpoints are fallback ticks when no aligned time tick exists. Represent
spacing honestly, for example Option<f64> or an enum distinguishing Uniform,
NonUniform, and Singleton. Add property tests over all finite exponent ranges and large
targets.

### COR-09 — Log scale/domain combinations are accepted and then panic or mishandle gaps

**Severity: Medium**

Plot domain setters validate only finiteness. Layout uses a fixed domain before calling
Ticks::log10, which asserts positivity. A plot using y_domain(-1, 10).log_y() therefore
constructs successfully and panics during render.

For data-driven log plots, draw_series checks raw coordinate finiteness at
src/plot/draw.rs:215-225 but does not check the mapped coordinate. A finite nonpositive
value maps to a non-finite coordinate and is stored as the previous point instead of
resetting the gap. The first subsequent valid point can consequently disappear or be
connected incorrectly.

**Recommendation**

Validate scale/domain composition before layout and make render fallible for invalid
runtime configuration. Treat a non-finite mapped position exactly like a source gap in
all marks. Test zero/negative values at the beginning, middle, and end of log series.

### COR-10 — Grid can return output larger than its requested Frame

**Severity: Medium**

Grid::render forces each cell to at least one column and one row at
src/plot/grid.rs:65-70. If the frame is smaller than the number of grid columns or rows,
that minimum makes the composed output exceed frame.width or frame.height.

Probes confirmed:

- 10 grid columns rendered into a three-column frame produced a line much wider than
  three display columns;
- four one-column plots rendered into height two produced four output lines.

**Recommendation**

Define a strict size contract and enforce it. Either omit cells that cannot fit, render
an explicit layout error, or allocate zero-sized cells and clip composition. Test every
small frame dimension against grids with more rows/columns than available cells and
measure display width, not byte length.

### COR-11 — hist2d renders degenerate finite datasets as blank

**Severity: Medium**

bins2 retains equal x/y extents for constant coordinates at src/stat/bin.rs:168-193.
Cells inversion rejects equal mapped endpoints at src/plot/draw.rs:566-578. Thus
all-identical data, and some one-axis-constant data, accumulate counts but render no
cells.

There are currently no direct bins2/hist2d regressions for these cases.

**Recommendation**

Pad a degenerate extent by a scale-aware, ULP-safe amount, or define a single-coordinate
cell mapping. Test all-identical points, vertical lines, horizontal lines, large-offset
constants, and one-point inputs.

### COR-12 — Categorical layers override explicitly selected numeric x scales

**Severity: Medium**

Layout chooses explicit nonempty Bands, otherwise it infers categories from the first
Bars or band-placed Range layer at src/plot/layout.rs:77-91. The Plot starts with
Scale::Linear, so there is no state distinguishing “default linear” from “the caller
explicitly selected Linear.” Explicit Linear, Log, or Time can therefore be silently
ignored when a categorical layer exists, despite Plot::x_scale saying inference occurs
only when none is set explicitly.

Multiple categorical layers with differing category lists also silently use the first
list, allowing later values to be clipped or mislabeled.

**Recommendation**

Introduce an Auto/default scale state, or store whether a scale was explicitly set.
Validate that all categorical layers share the same ordered category domain. Reject
conflicts rather than choosing the first layer.

### COR-13 — Public paired-series transforms silently truncate length mismatches

**Severity: Medium**

lttb zips x and y without checking lengths at src/stat/lttb.rs:15-22. m4 calculates the
domain from all x values but then zips x/y at src/stat/m4.rs:161-182. A probe with
x = [0, 1, 2], y = [10], target = 10 returned a one-point result rather than rejecting
the mismatch.

This differs from marks and other statistics APIs, which assert equal lengths.

**Recommendation**

Use a consistent contract: return Result for runtime data errors, or at minimum assert
equal lengths with a documented Panics section. Add tests for x shorter, y shorter, and
empty/non-finite combinations.

### COR-14 — Range body values do not participate in automatic y-domain calculation

**Severity: Medium**

ResolvedLayer::y_extent unions Range low, high, and marker at
src/plot/resolve.rs:153-155, but omits the optional body. Range::body validates only
lengths at src/mark/range.rs:117-130 and does not require its bounds to lie within the
whiskers.

Valid body values outside low/high can therefore be clipped or distort the intended
visual without contributing to scale fitting.

**Recommendation**

Either include body_low/body_high in the extent or validate and document that the body
must be contained by the outer interval. The more composable choice is to include all
encoded coordinates.

### COR-15 — Cells use a linear inverse even when an axis is logarithmic

**Severity: Medium**

position_on maps the data extents forward, interpolates linearly in data space, and
states that cells are not drawn on log axes at src/plot/draw.rs:564-578. No API or
layout validation actually prohibits Cells with Log scales. With positive explicit
extents, such a plot renders using the wrong inverse and samples incorrect cells.

**Recommendation**

Give Map an explicit inverse implementation and use it, or reject Cells on unsupported
scales during validation. Avoid relying on a private comment as a cross-module
invariant.

### COR-16 — contour can skip its documented geometry validation

**Severity: Low**

The contour preset documents a panic for zero or non-dividing columns. It first returns
an empty plot when the finite value extent is constant at src/presets.rs:158-169, before
calling the lower-level contour function that validates geometry. contour(0, [1.0])
therefore returns successfully despite its contract.

The lower-level contour implementation also filters NaN rather than all non-finite
values, which is inconsistent with the rest of the crate and permits infinity-derived
coordinates.

**Recommendation**

Validate matrix geometry at the start of the preset and use is_finite consistently.

### COR-17 — Surface and Frame size arithmetic is unchecked

**Severity: Low**

Surface::new multiplies width * height for allocation and subpixel_size multiplies by
charset density at src/render/surface.rs:48-69. Plot resolution also multiplies frame
width by subpixels at src/plot/plot.rs:184-185. Very large caller-provided Frame values
can overflow or request an uncontrolled allocation.

**Recommendation**

Add checked multiplication and an explicit maximum raster budget. Consider
Surface::try_new and Plot::try_render so allocation failure/oversize can be reported
without panicking.

## Targeted probe summary

The following probes were run through the public API in debug mode; the temporary probe
file was removed after the audit.

| Probe | Observed behavior |
|---|---|
| Moments::default().add(5) | min = 0 instead of 5 |
| Ticks::linear(-f64::MAX, f64::MAX, 6) | Arithmetic panic |
| Ticks::time(1.1, 3.2, 2) | Empty tick list |
| Ticks::log10(1, 1000, 5).step() | 0 despite multiple ticks |
| kde(&[1e20], 16) | Capacity-overflow panic |
| lttb with unequal x/y lengths | Silently truncated to the shorter input |
| fixed x domain [0.1, 0.9] with points at 0 and 1 | Outside points changed the raster |
| fixed [0, 1] domains with a nearby outside point | Mark ink changed a gutter cell |
| nonpositive fixed y domain plus log_y | Render panic |
| 1,000-point line with an intra-bucket NaN | M4 raster differed from raw raster |
| undersized Grid | Output exceeded requested width and height |
| contour(0, constant values) | Returned instead of honoring documented panic |
| serde Colormap with no stops | Deserialized, then color() panicked |
| serde Grid with zero columns | Deserialized, then render panicked |
| serde Range with unequal channels | Deserialized, then render panicked |
| Bins::auto finite input, limit 1 | Dropped observations and could return 2 bins |

## API design review

### API-01 — Add a fallible validation/rendering boundary

Panicking constructors are defensible for obvious programmer mistakes such as unequal
literal lengths. They are less suitable for runtime domains, serialized plots, frame
sizes, statistical parameters, and scale combinations. Today these cases fail through
assertions in different layers, making error handling and diagnosis inconsistent.

Recommended direction:

- define a focused Error enum with variants for invalid dimensions, unequal series,
  invalid domains, incompatible scales, oversize frames, and invalid serialized state;
- add try_* constructors or TryFrom implementations for runtime-facing types;
- add Plot::validate and Plot::try_render;
- keep concise panicking convenience constructors where they materially improve
  ergonomics, implemented over the fallible core.

This one design change would directly address COR-02, COR-04, COR-07, COR-09, COR-10,
COR-13, COR-15, and COR-17.

### API-02 — Materializing presets unnecessarily retain input lifetimes

Several presets fully consume borrowed input and build owned output, yet return
Plot<'a>: hist, stairs, ecdf, hist2d, contour, quiver, box_plot, error_bars, density,
and violin. The returned plot is consequently tied to the source borrow even though it
does not contain it. Callers cannot drop/reuse the input as early as the representation
would permit.

Return Plot<'static> from presets that materialize all channels, or redesign the return
lifetime so it reflects actual storage. Add compile tests demonstrating that the
source can be dropped while the result remains usable.

### API-03 — Line::function requires a static closure despite carrying a lifetime

Line::function accepts a closure with a 'static bound at src/mark/line.rs:86-89, while
Line itself is parameterized by 'a. This prevents natural closures that borrow a local
configuration. The bound simplifies into_owned, but that tradeoff is not surfaced.

Options:

- store Arc<dyn Fn(...) + 'a> and make promotion to owned fallible/unavailable for
  borrowed closures;
- add a separate borrowed_function constructor;
- keep 'static but return Line<'static> and document the ownership choice explicitly.

Documentation should also avoid claiming functions are pure or non-panicking: Fn +
Send + Sync does not enforce determinism, purity, or panic freedom.

### API-04 — Ticks::step cannot represent the supported tick families

A single f64 step is a poor model for logarithmic and calendar ticks. Returning 0
creates an ambiguous sentinel shared by singleton, nonuniform, and unknown spacing.
The implementation already recognizes this internally.

Prefer an explicit representation, such as:

- TickSpacing::Uniform(f64);
- TickSpacing::NonUniform;
- TickSpacing::Singleton.

This is clearer than repairing the current documentation around a lossy value.

### API-05 — Scale inference needs an explicit Auto state

Scale::Linear currently doubles as both a real caller choice and the implicit default.
That makes documented precedence impossible to implement reliably for band layers.
Scale::Auto, or a separate “explicitly set” flag, would make inference and conflict
validation deterministic.

### API-06 — Display and Frame::detect are tied to stdout, not the actual writer

Plot's Display implementation calls Frame::detect at src/plot/plot.rs:206-210. Output
from format!, logging, files, and test snapshots can therefore vary with terminal size,
environment variables, and stdout TTY state.

Frame::detect's color decision also probes stdout at src/plot/frame.rs:115-134. The
streaming example writes through Live to stderr, so stdout redirection can incorrectly
disable color for an interactive stderr.

Recommended direction:

- keep Plot::render(Frame) as the primary deterministic API;
- make Display deterministic and plain, or clearly designate it as terminal-only;
- add Frame::detect_for_stdout / detect_for_stderr or accept explicit writer
  capabilities;
- separate terminal dimensions, color capability, and charset selection into
  independently overridable values.

### API-07 — The serde wire format is private representation, unversioned

Derived serialization exposes enum shapes and private fields as a de facto persistence
format. Internal refactors, enum variants, or ownership changes can become breaking
format changes even when the Rust API remains source compatible.

If serde is intended only for transient round trips, say so. If persisted plot specs
are a product feature, define a versioned public DTO, publish compatibility rules, and
validate during conversion. This should be resolved before 1.0.

### API-08 — Colormap cannot be built from runtime-owned stops

Colormap stores Cow<'static, ...>, but its public constructor accepts only a
&'static slice at src/scale/colormap.rs:29-41. This conflicts with the documentation's
“any custom map is just a list of stops” wording and prevents palettes loaded from
configuration or computed at runtime. Ironically, serde can create an owned variant
that ordinary Rust callers cannot construct safely.

Add a validated from_vec/Into<Cow<'static, ...>> constructor and a stops accessor.

### API-09 — IntoSeries feature claims are broader than the implementations

The core conversion documentation says every primitive numeric type, while the macro
set omits i128 and u128. The new ndarray integration accepts borrowed f64 arrays/views,
but broad feature wording can be read as accepting common f32 data and by-value views.

Either expand support or narrow the wording. Compile-time API tests are appropriate
because doctests do not exhaust generic conversion coverage.

### API-10 — Public evolution points should be settled before 1.0

Mark and Scale are non_exhaustive, but several other public enums are not, including
Charset, Color, ColorMode, and LineStyle. Adding a variant later would break exhaustive
matches. Frame and Theme expose public fields, so adding required fields is also a
source-breaking change for struct literals. Theme's palette has exactly six entries,
which is simple but inflexible for plots with more layers.

This is not inherently wrong before 1.0, but it should be an explicit stability
decision:

- mark evolving enums non_exhaustive;
- prefer constructors/builders for structs expected to grow;
- decide whether arbitrary-length palettes are part of the intended API.

### API-11 — Statistical configuration and naming need refinement

- The root plotting Grid and stat::Grid are unrelated public types with the same name.
  A name such as Histogram2d or BinnedGrid would improve diagnostics and imports.
- Presets hard-code important parameters: histogram limit 60, hist2d 48 × 32, density
  256 samples, and contour level target 7. Add configurable variants while retaining
  short defaults.
- Grid has render but no Display parity with Plot.
- Opaque Plot/mark types have almost no inspection API, which limits tooling,
  validation, transformation, and GUI editors. A deliberate visitor/spec API would be
  preferable to exposing fields ad hoc.

## Engineering and release quality

### REL-01 — The audited tree is not gate-clean

Four concrete checks are red:

1. rustfmt rejects src/presets.rs.
2. the generated example gallery says quiver is missing.
3. all-target/all-feature Clippy rejects 6.28 as an approximate TAU constant.
4. strict rustdoc rejects the ambiguous line link in src/lib.rs.

At minimum, a release branch should be clean under all four before tag/publish.

### REL-02 — CI lints less than it tests

.github/workflows/ci.yml runs default-feature Clippy, then tests all features. This is
why the ndarray lint failure is invisible to CI. Documentation is also built only with
default features and without warnings-as-errors.

Recommended CI commands:

    cargo fmt --all -- --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test --all-targets --all-features
    cargo test --doc --all-features
    RUSTDOCFLAGS="-Dwarnings" cargo doc --no-deps --all-features
    cargo run --quiet --example regen_docs -- --check

Also test each optional feature alone or use cargo-hack for the feature powerset. An
all-features build cannot detect accidental coupling between optional features.

### REL-03 — The M4 pixel-equivalence test is circular

The test in src/plot/tests/plot_tests.rs:58-70 renders a large “full” series through the
same automatic downsampler it intends to verify. It can pass when both sides are
equally wrong. This is a test-oracle defect, not merely a missing edge case.

Provide a test-only switch or lower-level renderer that bypasses M4, and make that raw
raster the reference.

### REL-04 — Documentation makes claims broader than the evidence

Examples observed during review:

- README/terminology material says every preset is grammar-identical, while equality
  tests cover only a small subset.
- M4 is called pixel-exact without qualification for gaps.
- stat module prose calls the aggregation surface mergeable, although several
  aggregators do not expose merge and floating-point reduction is not strictly
  associative.
- a “every charset” test omits Sextants and Octants.
- terminology documentation describes a Charset::Auto variant that does not exist.
- purity, determinism, and no-panic phrasing is too strong for arbitrary function
  closures and accepted invalid combinations.

Narrow claims to demonstrated guarantees, or add tests that make the stronger claims
true.

### REL-05 — Release metadata and generated docs are out of sync

Cargo.toml says version 0.10.0 while the changelog heading still says “Unreleased
(toward 0.10.0).” The ndarray and quiver additions are not represented consistently in
the changelog, README feature list, examples gallery, and showcase. The README's chart
list also omits existing/new presets.

Use one release checklist or xtask to verify version, changelog heading, feature docs,
example registry, generated gallery, and package contents together.

### REL-06 — MSRV is neither declared nor tested

There is no rust-version in Cargo.toml. Edition 2024 establishes a compiler floor, but
does not document the actual support policy. Rust 1.90 happened to pass this audit; that
is not a commitment or proof of the minimum.

Choose an MSRV, declare rust-version, pin a CI job to it, and avoid allowing dependency
updates to raise it silently.

### REL-07 — Edge/property testing is thin in the highest-risk numerical code

The suite is broad in ordinary examples but undersamples adversarial invariants:

- no property that Bins::auto counts every finite value and respects its cap;
- no extreme finite exponent coverage for Ticks and KDE;
- no deserialize-then-render malformed-state corpus;
- no true raw-versus-M4 oracle with arbitrary gaps;
- no bins2/hist2d tests for degenerate axes;
- no plot-rectangle clipping matrix across marks;
- little fuzz coverage for mixed finite/non-finite channels and tiny frames.

Add proptest-style invariants and fuzz targets at transform and serde boundaries.
Snapshot tests are useful for visual regressions, but should complement rather than
replace numerical postconditions.

### REL-08 — Several public transforms have avoidable scaling costs

These are not immediate correctness defects, but will be visible on data sizes for
which a plotting/statistics crate is useful:

- Agg::by linearly searches existing keys for every item at src/stat/agg.rs:50-57,
  yielding O(n × groups);
- Window reductions rescan each window, and mean allocates a Vec for every output at
  src/stat/window.rs:25-73;
- indexed series materializes an x Vec on resolution at src/plot/resolve.rs:405-413;
- error_bars duplicates x and y data;
- several preset parameters force work independent of frame resolution.

Use an insertion-order map or a HashMap plus ordered output, rolling sum/count/deque
algorithms, a borrowed/indexed x representation, and frame-aware preset variants.
Add benchmarks for high-cardinality groups, large windows, and million-point render
paths.

### REL-09 — The ratatui adapter may leave stale wide-glyph continuation cells

Surface::cells omits continuation cells at src/render/surface.rs:223-234. The adapter
writes only yielded glyph cells at src/adapter.rs:79-92 and never clears the covered
cell to the right of a wide glyph. On a reused/prepopulated Buffer, stale symbols or
styles can remain in that logical continuation cell and confuse buffer diffing.

Add a test that renders CJK/wide text over a nonblank Buffer and then replaces it with
narrow/blank content. Either emit explicit continuation/reset information or use a
ratatui API that maintains wide-cell invariants.

### REL-10 — Live writer state can desynchronize after partial I/O failure

Live::draw updates drawn_rows only after write_all and flush succeed at
src/stream/mod.rs:129-142. A partial write followed by an error can change terminal
state while the handle still believes the old frame is active. The row count based on
str::lines also deserves explicit tests for trailing blank rows.

Document recovery semantics, reset/detach state conservatively after write failure, and
add get_ref/get_mut/into_inner so callers can recover or inspect the writer.

### REL-11 — Dependency and compatibility assurance is incomplete

Positive: the default runtime dependency set is small and the crate forbids unsafe
code. Missing release checks include:

- automated RustSec/OSV advisory scanning;
- license/source policy via cargo-deny;
- cargo package verification from the packaged artifact;
- cargo-semver-checks against the prior release;
- Windows CI if Windows is an intended target.

Dev-only duplicate dependency versions appeared in cargo tree, but no production
dependency defect was established. Do not describe the graph as security-clean until
an actual advisory scan runs.

## Positive observations

The audit also found strong design and implementation choices worth preserving:

- The crate has #![forbid(unsafe_code)] and contains no unsafe implementation escape
  hatch.
- The mark grammar is compact and understandable; presets generally compose it instead
  of introducing disconnected rendering paths.
- Plot::render with an explicit Frame is a good deterministic core boundary.
- Cow-backed Series plus into_owned gives callers useful control over borrowing versus
  ownership.
- Normal mark constructors consistently validate paired channel lengths.
- Mark and Scale are already non_exhaustive, showing awareness of pre-1.0 evolution.
- Default dependencies are few, and optional integrations are feature-gated.
- The library compiles on all targets and features exercised here.
- 190 tests and 21 doctests pass, including snapshots, algorithm tests, serde happy
  paths, ownership/send-sync assertions, and small-frame cases.
- Library-only all-feature Clippy is clean; the current lint failure is isolated to a
  newly added integration test.
- Surface performs finite checks and clips individual lines defensively.
- Public documentation coverage is extensive enough that rustdoc reports only one
  warning under the audited tree.

These strengths make the recommended fixes evolutionary rather than a request for a
rewrite.

## Remediation roadmap

### Phase 0 — Before publishing the current release

1. Correct Bins::auto coverage/cap and Moments::default.
2. Make fixed domains exact and clip every mark to the plot rectangle.
3. Clamp all pre-raster spans before integer iteration.
4. Preserve exact M4 segment boundaries and build a raw-render oracle.
5. Validate serde states through fallible conversion.
6. Validate scale/domain/category combinations before layout.
7. Repair the four red repository gates: fmt, gallery, all-feature Clippy, rustdoc.
8. Add focused regression tests for each corrected high-severity finding.

### Phase 1 — Next minor release

1. Stabilize Ticks spacing semantics and extreme-range behavior.
2. Make KDE and hist2d robust to large-offset/degenerate data.
3. Enforce equal lengths in every paired transform.
4. Enforce Grid's Frame-size contract.
5. Correct Range extents and Cells/log behavior.
6. Add configurable statistical presets.
7. Expand CI to feature powersets, strict docs, package verification, and the chosen
   MSRV.

### Phase 2 — Before 1.0

1. Establish the fallible Error/validate/try_render API.
2. Decide the serde wire-format and compatibility policy.
3. Set enum/struct extensibility policy and palette representation.
4. Correct unnecessary preset lifetimes and decide closure borrowing semantics.
5. Define Display and terminal capability detection behavior.
6. Rename ambiguous public types and add inspection/tooling APIs where intended.
7. Add semver, advisory, license, fuzz, property, and performance gates.

## Suggested acceptance criteria

The crate is in a substantially safer release state when all of the following hold:

- every accepted serialized Plot either validates and renders or returns a typed error;
- a fixed domain never changes because of ticks and no mark writes outside its plot
  rectangle;
- every pre-raster loop is bounded by frame dimensions;
- sum(Bins::auto(...).counts()) equals the finite input count and bin count respects the
  documented cap;
- Moments::new and Moments::default are behaviorally identical;
- M4 matches a raw raster for finite monotonic-x series with arbitrary gap placement;
- Ticks never panic for documented finite bounds/targets and its spacing API is honest;
- KDE/hist2d handle constant large-magnitude finite data without panic or blank output;
- Grid output never exceeds the requested Frame in display columns or rows;
- fmt, all-feature/all-target Clippy, tests, doctests, strict rustdoc, and gallery checks
  are clean in CI;
- MSRV, serde compatibility, and dependency/license policies are explicit and tested.

## Final assessment

Malevich's architecture is promising and unusually well documented for a pre-1.0
terminal plotting crate. Most ordinary paths are clean and well tested. The audit's
main concern is not architectural complexity; it is that several strong public
guarantees are enforced only on the happy constructor path and fail under composition,
deserialization, extreme finite values, or clipping.

Addressing the six high-severity issues and making validation a first-class boundary
would materially improve correctness without disrupting the crate's central grammar.
The remaining API and quality work is best handled now, while the project still has
pre-1.0 freedom to refine contracts.

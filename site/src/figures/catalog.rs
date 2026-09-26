//! The list of figures. Each name is a file under `site/figures/`; the block in
//! that file both runs and is shown.

use malevich::render::Charset;

use super::{Figure, card, card_in};

pub fn all() -> Vec<Figure> {
    let mut figures = Vec::new();
    figures.extend(start());
    figures.extend(grammar());
    figures.extend(marks());
    figures.extend(stats());
    figures.extend(scales());
    figures.extend(rest());
    figures
}

fn start() -> Vec<Figure> {
    catalog![
        "hero" => ("A training run: two lines, a dark fill under the training loss, a dashed target rule, and a note at data coordinates — one plot value, drawn as a terminal card.", card(80, 18)),
        "start_line" => ("The first call: `malevich::line` over four values.", card(48, 10)),
        "start_bar" => ("Bars on a band axis from a zero baseline.", card(48, 10)),
        "start_hist" => ("`hist` over 342 penguin body masses: automatic bins, a count axis that never labels a half.", card(60, 14)),
        "start_scatter" => ("`scatter` over two columns of the same dataset.", card(60, 14)),
        "start_layers" => ("The lid off: a `Plot` with two layers and a title.", card(60, 12)),
        "start_gap" => ("Two missing readings. The line breaks; nothing is invented.", card(48, 10)),
    ]
}

fn grammar() -> Vec<Figure> {
    catalog![
        "grammar_1" => ("One layer: a `Points` mark bound to two columns.", card(72, 18)),
        "grammar_2" => ("The `color_by` channel: categories take palette colors and name themselves in the legend.", card(72, 18)),
        "grammar_3" => ("Three more layers: a least-squares fit per species, one `Line` each, from the same `Fit` accumulator the `trend` preset uses.", card(72, 18)),
        "grammar_4" => ("A `Rule` at the pooled mean and a `Text` note. The pooled slope runs the other way — Simpson's paradox in a chart.", card(72, 18)),
        "grammar_5" => ("Furniture last: a title and axis labels. Six layers, one plot value.", card(72, 18)),
        "grammar_preset" => ("`scatter(x, y)` is the first step through the front door: byte-identical to the one-layer plot.", card(72, 18)),
    ]
}

fn marks() -> Vec<Figure> {
    catalog![
        "mark_line_styles" => ("`Line`: the subpixel default, the one-glyph-per-column corners style, and a dash.", card(72, 16)),
        "mark_line_function" => ("`Line::function` samples a closure once per subpixel column.", card(72, 14)),
        "mark_line_grade" => ("`Line::grade` colors the line by a third series through a colormap; `glow` thickens it.", card(72, 16)),
        "mark_points_styles" => ("`Points`: five marker styles. In colorless output `color_by` cycles through them so groups stay apart.", card(72, 16)),
        "mark_points_color_by" => ("`Points::color_by`: three species, Okabe–Ito colors, a legend.", card(72, 18)),
        "mark_bars_bands" => ("`Bars::new`: one bar per category on a band axis.", card(60, 12)),
        "mark_bars_spans" => ("`Bars::spans`: contiguous bins on a numeric axis.", card(60, 12)),
        "mark_bars_intervals" => ("`Bars::intervals`: each bar between its own edges.", card(60, 12)),
        "mark_bars_base" => ("`Bars::base`: the second layer starts where the first ends — a stack, with no stacking mode.", card(60, 14)),
        "mark_bars_horizontal" => ("`Bars::horizontal`: bands down the y axis, values along x, long names in the label gutter.", card(72, 12)),
        "mark_area" => ("`Area`: a fill from the baseline, and a band between two series.", card(72, 16)),
        "mark_area_horizontal" => ("`Area::horizontal`: a fill along y. A violin is two of these.", card(60, 16)),
        "mark_cells_matrix" => ("`Cells::matrix`: a value grid under a colormap, rows labeled in matrix order.", card(64, 14)),
        "mark_cells_rgb" => ("`Cells::rgb`: direct colors, no colormap. An image is one more cell grid.", card(56, 14)),
        "mark_cells_classes" => ("`Cells::classes`: categorical regions through the palette, with legend swatches.", card(64, 18)),
        "mark_cells_extents" => ("`Cells::extents` and `reduce`: a grid denser than the raster, max-reduced so its spikes survive.", card(72, 16)),
        "mark_range_xy" => ("`Range::xy`: an interval at each x — error bars.", card(60, 14)),
        "mark_range_over" => ("`Range::over` with `body` and `marker`: a box plot from the grammar.", card(60, 16)),
        "mark_rule" => ("`Rule`: horizontal and vertical lines, dashed or not, and spans that wash a band across the plot.", card(72, 16)),
        "mark_text" => ("`Text::at` with `align`: annotations at data coordinates, centered on their bands.", card(56, 14)),
    ]
}

fn stats() -> Vec<Figure> {
    catalog![
        "stat_bins" => ("`Bins::auto` and `Bars::spans`: the histogram preset, spelled out.", card(64, 14)),
        "stat_bins_normalized" => ("`HistogramOptions`: bins rescaled to percent and accumulated left to right.", card(64, 14)),
        "stat_kde" => ("`kde_with`: the same data at three bandwidths.", card(72, 16)),
        "stat_box_whiskers" => ("`BoxOptions::whiskers`: a percentile rule instead of Tukey's.", card(60, 16)),
        "stat_fit" => ("`trend` and the `Fit` behind it: points, the line, a confidence band, and the numbers.", card(72, 18)),
        "stat_window" => ("`Window`: a rolling mean and a rolling p95, one reducer vocabulary.", card(80, 18)),
        "stat_window_anchor" => ("`WindowAnchor`: trailing, centered, and strict.", card(72, 16)),
        "stat_ewma" => ("`ewma` over a real training log.", card(72, 16)),
        "stat_stack" => ("`stack`: each area sits on the sum of the ones below.", card(72, 16)),
        "stat_stack_normalize" => ("`StackOffset::Normalize`: the 100 % stack.", card(72, 16)),
        "stat_dodge" => ("`dodge`: side-by-side positions within each band.", card(60, 14)),
        "stat_m4" => ("200,000 points reduced by M4: the three one-sample spikes survive, by construction.", card(80, 14)),
        "stat_stride" => ("The same series sampled every 400th point. The spikes are gone, and nothing says so.", card(80, 14)),
        "stat_lttb" => ("`lttb`: an explicit, disclosed approximation the caller applies.", card(72, 16)),
        "stat_ecdf" => ("`ecdf_with` and its DKW band.", card(64, 16)),
        "stat_roc" => ("`roc` and `auc`: the curve, the chance line, the area in the title.", card(64, 18)),
        "stat_binned" => ("`binned` with `Reducer::Mean`: a reliability diagram from 0/1 outcomes.", card(64, 16)),
        "stat_agg" => ("`Agg::by` and a reducer: one bar per group.", card(56, 12)),
        "stat_jitter" => ("`jitter`: every measurement spread evenly across its band.", card(64, 16)),
        "stat_steps" => ("`steps`: the three places a step can change.", card(72, 16)),
        "stat_maps" => ("`cumsum` and `normalize`: the series maps.", card(72, 14)),
        "stat_calendar" => ("`calendar_bins`: counts per calendar month, drawn between each month's true edges.", card(72, 14)),
        "stat_contours" => ("`contour`: marching squares over a grid, levels chosen like ticks.", card(64, 20)),
        "stat_describe" => ("`describe`: the summary that usually precedes a chart, as a stat table.", card(80, 8)),
    ]
}

fn scales() -> Vec<Figure> {
    catalog![
        "scale_linear" => ("A count on a linear axis and a tall frame labels 0.5 and 1.5.", card(40, 14)),
        "scale_integer" => ("`Scale::Integer`: the tick step never drops below one.", card(40, 14)),
        "scale_log" => ("`log_x` and `log_y`: decade ticks with superscript labels.", card(64, 16)),
        "scale_time_hours" => ("A calendar axis over one session: hour labels, and the date printed once as a context note.", card(72, 14)),
        "scale_time_days" => ("A calendar axis over six weeks: day labels, the month and year once.", card(72, 14)),
        "scale_time_years" => ("A calendar axis over sixty-seven years of real data.", card(72, 16)),
        "scale_bands_y" => ("Bands on both axes: a confusion matrix with counts centered in their cells.", card(48, 12)),
        "scale_domain_auto" => ("The automatic domain fits the data.", card(60, 12)),
        "scale_domain_fixed" => ("Fixed domains: what falls outside is clipped, never smeared onto the border.", card(60, 12)),
        "scale_one_sided" => ("`y_min(0.0)`: one end fixed, the other fitted.", card(64, 12)),
        "scale_unit_si" => ("`Unit::si`: one SI prefix per axis, before the unit.", card(44, 10)),
        "scale_unit_bytes" => ("`Unit::Bytes`: ticks nice in binary units.", card(44, 10)),
        "scale_unit_suffix" => ("`Unit::suffix`: a bare suffix, never a prefix.", card(44, 10)),
        "scale_ticks" => ("Ticks stepping by 0.2, the float-artifact trap: every label is an exact decimal.", card(64, 12)),
        "scale_context" => ("The context note: labels read relative to a base printed once.", card(64, 12)),
        "cm_viridis" => ("Viridis, sequential.", card(40, 3)),
        "cm_magma" => ("Magma, sequential.", card(40, 3)),
        "cm_cividis" => ("Cividis, sequential, colorblind-safe.", card(40, 3)),
        "cm_greys" => ("Greys, sequential.", card(40, 3)),
        "cm_red_blue" => ("Red–blue, diverging.", card(40, 3)),
        "cm_purple_orange" => ("Purple–orange, diverging.", card(40, 3)),
        "scale_colormap_centered" => ("`centered_at`: a diverging map anchored at a data value.", card(56, 14)),
        "scale_colormap_log" => ("`log`: decades share equal color steps; the mask's zeros are gaps.", card(60, 14)),
        "scale_colormap_domain" => ("`domain`, `under`, `over`: a fixed range that discloses what falls outside.", card(64, 14)),
        "scale_colormap_steps" => ("`steps`: the ramp quantized into bands the colorbar labels.", card(64, 16)),
        "palette_okabe_ito" => ("Okabe–Ito, the default.", card(60, 12)),
        "palette_bright" => ("Paul Tol's Bright.", card(60, 12)),
        "palette_muted" => ("Paul Tol's Muted.", card(60, 12)),
    ]
}

fn rest() -> Vec<Figure> {
    catalog![
        "furniture_all" => ("Title, axis labels, a legend, and a colorbar.", card(72, 16)),
        "furniture_sparkline" => ("`sparkline`: the plot with `axes(false)` in a one-row frame.", card(20, 1)),
        "furniture_axes_off" => ("`axes(false)` on a full plot: the data region fills the frame.", card(60, 8)),
        "ladder_heat" => ("A heatmap with a colorbar and a line: the shapes that show a color tier.", card_in(Charset::Quadrants, 64, 14)),
        "comp_shared_a" => ("Two panels, one x window, each with its own honest y.", card(44, 10)),
        "comp_shared_b" => ("The second panel of the pair.", card(44, 10)),
        "comp_table" => ("`table_with`: a matrix as an aligned table, colored per column.", card(60, 8)),
        "comp_tornado" => ("A tornado from horizontal bars and a rule at zero.", card(56, 10)),
        "inter_full" => ("The whole window of a 100,000-point series.", card(72, 12)),
        "inter_zoomed" => ("A `Viewport` window over the same plot: M4 re-aggregates to the new columns.", card(72, 12)),
        "stream_tail" => ("`Viewport::tail`: the follow-the-stream window.", card(72, 12)),
        "perf_bench" => ("The benchmark suite, plotted by the library it measures.", card(80, 16)),
        "changelog_releases" => ("Every release, counted from the changelog's own headings.", card(72, 12)),
    ]
}

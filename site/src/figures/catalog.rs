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
        "hero" => ("One plot value, drawn as a terminal card: a training run with two lines, a dark fill under the training loss, a dashed target rule, and a note at data coordinates.", card(80, 18)),
        "start_line" => ("The first call is `malevich::line` over four values.", card(48, 10)),
        "start_bar" => ("Bars rise from a zero baseline on a band axis.", card(48, 10)),
        "start_hist" => ("`hist` over 342 penguin body masses chooses the bins, and its count axis never labels a half.", card(60, 14)),
        "start_scatter" => ("`scatter` plots two columns of the same dataset.", card(60, 14)),
        "start_layers" => ("The lid is off: a `Plot` with two layers and a title.", card(60, 12)),
        "start_gap" => ("Two missing readings. The line breaks; nothing is invented.", card(48, 10)),
    ]
}

fn grammar() -> Vec<Figure> {
    catalog![
        "grammar_1" => ("One layer binds a `Points` mark to two columns.", card(72, 18)),
        "grammar_2" => ("The `color_by` channel gives categories palette colors and names them in the legend.", card(72, 18)),
        "grammar_3" => ("Three more layers add a least-squares fit per species, one `Line` each, from the same `Fit` accumulator the `trend` preset uses.", card(72, 18)),
        "grammar_4" => ("A `Rule` at the pooled mean and a `Text` note: the pooled slope runs the other way, Simpson's paradox in a chart.", card(72, 18)),
        "grammar_5" => ("Furniture comes last: a title and axis labels on six layers, one plot value.", card(72, 18)),
        "grammar_preset" => ("`scatter(x, y)` is the first step through the front door: byte-identical to the one-layer plot.", card(72, 18)),
    ]
}

fn marks() -> Vec<Figure> {
    catalog![
        "mark_line_styles" => ("`Line` shows the subpixel default, the one-glyph-per-column corners style, and a dash.", card(72, 16)),
        "mark_line_function" => ("`Line::function` samples a closure once per subpixel column.", card(72, 14)),
        "mark_line_grade" => ("`Line::grade` colors the line by a third series through a colormap, and `glow` thickens it.", card(72, 16)),
        "mark_points_styles" => ("`Points` has five marker styles, and in colorless output `color_by` cycles through them so groups stay apart.", card(72, 16)),
        "mark_points_color_by" => ("`Points::color_by` gives three species Okabe–Ito colors and a legend.", card(72, 18)),
        "mark_bars_bands" => ("`Bars::new` draws one bar per category on a band axis.", card(60, 12)),
        "mark_bars_spans" => ("`Bars::spans` draws contiguous bins on a numeric axis.", card(60, 12)),
        "mark_bars_intervals" => ("`Bars::intervals` places each bar between its own edges.", card(60, 12)),
        "mark_bars_base" => ("`Bars::base` starts the second layer where the first ends, a stack with no stacking mode.", card(60, 14)),
        "mark_bars_horizontal" => ("`Bars::horizontal` runs bands down the y axis, values along x, long names in the label gutter.", card(72, 12)),
        "mark_area" => ("`Area` is a fill from the baseline, and a band between two series.", card(72, 16)),
        "mark_area_horizontal" => ("`Area::horizontal` fills along y, and a violin is two of these.", card(60, 16)),
        "mark_cells_matrix" => ("`Cells::matrix` puts a value grid under a colormap, with rows labeled in matrix order.", card(64, 14)),
        "mark_cells_rgb" => ("`Cells::rgb` uses direct colors and no colormap, and an image is one more cell grid.", card(56, 14)),
        "mark_cells_classes" => ("`Cells::classes` paints categorical regions from the palette, with legend swatches.", card(64, 18)),
        "mark_cells_extents" => ("`Cells::extents` and `reduce` max-reduce a grid denser than the raster, so its spikes survive.", card(72, 16)),
        "mark_range_xy" => ("`Range::xy` draws an interval at each x: the error bars.", card(60, 14)),
        "mark_range_over" => ("`Range::over` with `body` and `marker` builds a box plot from the grammar.", card(60, 16)),
        "mark_rule" => ("`Rule` draws horizontal and vertical lines, dashed or not, and spans that wash a band across the plot.", card(72, 16)),
        "mark_text" => ("`Text::at` with `align` puts annotations at data coordinates, centered on their bands.", card(56, 14)),
    ]
}

fn stats() -> Vec<Figure> {
    catalog![
        "stat_bins" => ("`Bins::auto` and `Bars::spans` spell out the histogram preset.", card(64, 14)),
        "stat_bins_normalized" => ("`HistogramOptions` rescales the bins to percent and accumulates them left to right.", card(64, 14)),
        "stat_kde" => ("`kde_with` draws the same data at three bandwidths.", card(72, 16)),
        "stat_box_whiskers" => ("`BoxOptions::whiskers` uses a percentile rule instead of Tukey's.", card(60, 16)),
        "stat_fit" => ("`trend` and the `Fit` behind it show the points, the line, a confidence band, and the numbers.", card(72, 18)),
        "stat_window" => ("`Window` draws a rolling mean and a rolling p95 from one reducer vocabulary.", card(80, 18)),
        "stat_window_anchor" => ("`WindowAnchor` is trailing, centered, or strict.", card(72, 16)),
        "stat_ewma" => ("`ewma` runs over a real training log.", card(72, 16)),
        "stat_stack" => ("`stack` sits each area on the sum of the ones below.", card(72, 16)),
        "stat_stack_normalize" => ("`StackOffset::Normalize` makes the 100 % stack.", card(72, 16)),
        "stat_dodge" => ("`dodge` sets side-by-side positions within each band.", card(60, 14)),
        "stat_m4" => ("M4 reduces 200,000 points, and the three one-sample spikes survive by construction.", card(80, 14)),
        "stat_stride" => ("The same series sampled every 400th point. The spikes are gone, and nothing says so.", card(80, 14)),
        "stat_lttb" => ("`lttb` is an explicit, disclosed approximation the caller applies.", card(72, 16)),
        "stat_ecdf" => ("`ecdf_with` draws its curve and DKW band.", card(64, 16)),
        "stat_roc" => ("`roc` and `auc` show the curve, the chance line, and the area in the title.", card(64, 18)),
        "stat_binned" => ("`binned` with `Reducer::Mean` builds a reliability diagram from 0/1 outcomes.", card(64, 16)),
        "stat_agg" => ("`Agg::by` and a reducer draw one bar per group.", card(56, 12)),
        "stat_jitter" => ("`jitter` spreads every measurement evenly across its band.", card(64, 16)),
        "stat_steps" => ("`steps` shows the three places a step can change.", card(72, 16)),
        "stat_maps" => ("`cumsum` and `normalize` are the series maps.", card(72, 14)),
        "stat_calendar" => ("`calendar_bins` counts per calendar month and draws each count between that month's true edges.", card(72, 14)),
        "stat_contours" => ("`contour` runs marching squares over a grid, with levels chosen like ticks.", card(64, 20)),
        "stat_describe" => ("`describe` is the summary that usually precedes a chart, drawn as a stat table.", card(80, 8)),
    ]
}

fn scales() -> Vec<Figure> {
    catalog![
        "scale_linear" => ("A count on a linear axis, in a tall frame, labels 0.5 and 1.5.", card(40, 14)),
        "scale_integer" => ("`Scale::Integer` never lets the tick step drop below one.", card(40, 14)),
        "scale_log" => ("`log_x` and `log_y` tick by decades, with superscript labels.", card(64, 16)),
        "scale_time_hours" => ("Over one session the calendar axis labels the hours, and prints the date once as a context note.", card(72, 14)),
        "scale_time_days" => ("A calendar axis over six weeks labels the days, and the month and year once.", card(72, 14)),
        "scale_time_years" => ("A calendar axis covers sixty-seven years of real data.", card(72, 16)),
        "scale_bands_y" => ("Bands on both axes form a confusion matrix, with the counts centered in their cells.", card(48, 12)),
        "scale_domain_auto" => ("The automatic domain fits the data.", card(60, 12)),
        "scale_domain_fixed" => ("Fixed domains clip what falls outside, and never smear it onto the border.", card(60, 12)),
        "scale_one_sided" => ("`y_min(0.0)` fixes one end and fits the other.", card(64, 12)),
        "scale_unit_si" => ("`Unit::si` uses one SI prefix per axis, before the unit.", card(44, 10)),
        "scale_unit_bytes" => ("`Unit::Bytes` keeps the ticks nice, in binary units.", card(44, 10)),
        "scale_unit_suffix" => ("`Unit::suffix` adds a bare suffix and never a prefix.", card(44, 10)),
        "scale_ticks" => ("Ticks step by 0.2, the float-artifact trap, and every label is an exact decimal.", card(64, 12)),
        "scale_context" => ("The context note prints a base once, and the labels read relative to it.", card(64, 12)),
        "cm_viridis" => ("Viridis is sequential.", card(40, 3)),
        "cm_magma" => ("Magma is sequential.", card(40, 3)),
        "cm_cividis" => ("Cividis is sequential, and colorblind-safe.", card(40, 3)),
        "cm_greys" => ("Greys is sequential.", card(40, 3)),
        "cm_red_blue" => ("Red–blue is diverging.", card(40, 3)),
        "cm_purple_orange" => ("Purple–orange is diverging.", card(40, 3)),
        "scale_colormap_centered" => ("`centered_at` anchors a diverging map at a data value.", card(56, 14)),
        "scale_colormap_log" => ("`log` gives decades equal color steps, and the mask's zeros are gaps.", card(60, 14)),
        "scale_colormap_domain" => ("`domain`, `under`, and `over` fix a range and disclose what falls outside.", card(64, 14)),
        "scale_colormap_steps" => ("`steps` quantizes the ramp into bands the colorbar labels.", card(64, 16)),
        "palette_okabe_ito" => ("Okabe–Ito is the default.", card(60, 12)),
        "palette_bright" => ("The palette is Paul Tol's Bright.", card(60, 12)),
        "palette_muted" => ("The palette is Paul Tol's Muted.", card(60, 12)),
    ]
}

fn rest() -> Vec<Figure> {
    catalog![
        "furniture_all" => ("The plot has a title, axis labels, a legend, and a colorbar.", card(72, 16)),
        "furniture_sparkline" => ("`sparkline` is the plot with `axes(false)`, in a one-row frame.", card(20, 1)),
        "furniture_axes_off" => ("`axes(false)` on a full plot lets the data region fill the frame.", card(60, 8)),
        "ladder_heat" => ("A heatmap with a colorbar and a line, the shapes that show a color tier.", card_in(Charset::Quadrants, 64, 14)),
        "comp_shared_a" => ("Two panels share one x window, and each keeps its own honest y.", card(44, 10)),
        "comp_shared_b" => ("This is the second panel of the pair.", card(44, 10)),
        "comp_table" => ("`table_with` shows a matrix as an aligned table, colored per column.", card(60, 8)),
        "comp_tornado" => ("Horizontal bars and a rule at zero make a tornado.", card(56, 10)),
        "inter_full" => ("This is the whole window of a 100,000-point series.", card(72, 12)),
        "inter_zoomed" => ("A `Viewport` over the same plot makes M4 re-aggregate to the new columns.", card(72, 12)),
        "stream_tail" => ("`Viewport::tail` is the window that follows the stream.", card(72, 12)),
        "perf_bench" => ("The benchmark suite is plotted by the library it measures.", card(80, 16)),
        "changelog_releases" => ("Every release is counted from the changelog's own headings.", card(72, 12)),
    ]
}

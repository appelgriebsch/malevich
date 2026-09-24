//! Tick placement for linear axes.
//!
//! Implements the extended Wilkinson algorithm (Justin Talbot, Sharon Lin, Pat
//! Hanrahan, "An Extension of Wilkinson's Algorithm for Positioning Tick Labels on
//! Axes", IEEE InfoVis 2010): a branch-and-bound search over step mantissas, skip
//! amounts, label counts, and magnitudes, scored by a weighted sum of simplicity,
//! coverage, density, and legibility.
//!
//! One refinement over the paper: chosen tick values are carried as exact decimals
//! (integer mantissa times a power of ten), so labels are produced by integer math —
//! no binary-float artifacts, and a uniform number of decimals across the axis.

use super::format;
use super::unit::{BINARY_UNITS, Unit, binary_prefix};

/// How a linear axis labels its ticks: the unit the labels carry, and whether
/// the step may drop below one.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[non_exhaustive]
pub struct TickOptions {
    /// The unit on every label; [`Unit::Plain`] by default.
    pub unit: Unit,
    /// Whole-number ticks only: the step never drops below one.
    pub integer: bool,
}

impl TickOptions {
    /// Plain labels, any step — exactly [`Ticks::linear`].
    pub const fn new() -> TickOptions {
        TickOptions {
            unit: Unit::Plain,
            integer: false,
        }
    }

    /// Sets the unit.
    #[must_use]
    pub fn unit(mut self, unit: Unit) -> TickOptions {
        self.unit = unit;
        self
    }

    /// Restricts ticks to whole numbers.
    #[must_use]
    pub const fn integer(mut self) -> TickOptions {
        self.integer = true;
        self
    }
}

/// Step mantissas in preference order, as `(integer mantissa, value)` with
/// `value = mantissa / 10`, so that every tick value stays an exact decimal.
const STEPS: [(i128, f64); 6] = [
    (10, 1.0),
    (50, 5.0),
    (20, 2.0),
    (25, 2.5),
    (40, 4.0),
    (30, 3.0),
];

/// Score weights for simplicity, coverage, density, and legibility.
const WEIGHTS: [f64; 4] = [0.25, 0.2, 0.5, 0.05];

/// Powers of ten that are exactly representable in `f64`.
const POW10: [f64; 23] = [
    1e0, 1e1, 1e2, 1e3, 1e4, 1e5, 1e6, 1e7, 1e8, 1e9, 1e10, 1e11, 1e12, 1e13, 1e14, 1e15, 1e16,
    1e17, 1e18, 1e19, 1e20, 1e21, 1e22,
];

/// One axis tick: a position in data coordinates and its label.
#[derive(Debug, Clone, PartialEq)]
pub struct Tick {
    /// Position in data coordinates.
    pub value: f64,
    /// Label text: an exact decimal rendering of `value`, which parses back to it.
    /// Axes reaching ten thousand (or a ten-thousandth) carry one shared SI prefix
    /// (`20k`, `2.5M`, `100µ`); the numeric part times the prefix factor still
    /// equals `value` exactly. Zero is always plain `0`. The one exception is a
    /// range no nice step can cover — equal bounds, or a span past what the
    /// exact mantissa holds — whose two endpoint ticks carry the bounds
    /// rounded to the shared significant-digit budget (`1.798e308`), the same
    /// formatter every readout uses.
    pub label: String,
}

/// Ticks chosen for a linear axis: ascending, uniformly spaced, decimal-exact.
///
/// All labels of a set share the same number of fraction digits, so they align when
/// stacked on an axis.
#[derive(Debug, Clone, PartialEq)]
pub struct Ticks {
    ticks: Vec<Tick>,
    step: Option<f64>,
}

impl Ticks {
    /// Places approximately `target` ticks over `[min, max]` with the extended
    /// Wilkinson algorithm.
    ///
    /// The bounds may be given in either order. A `target` below 2 is treated as 2,
    /// and the returned count is close to, not exactly, `target`. Equal bounds yield
    /// a single tick at that value; a span the search cannot cover (one that
    /// overflows, or a huge magnitude with a tiny span) yields the two bounds,
    /// labeled at the shared significant-digit budget. The chosen ticks may
    /// extend beyond the data range (that is the algorithm's coverage
    /// trade-off), typically by less than one step on either side.
    ///
    /// # Panics
    ///
    /// Panics if `min` or `max` is not finite.
    pub fn linear(min: f64, max: f64, target: usize) -> Ticks {
        Ticks::linear_with(min, max, target, &TickOptions::new())
    }

    /// [`Ticks::linear`] with a [`TickOptions`]: a unit on the labels (one SI
    /// prefix per axis plus the unit, binary bytes nice in their own unit, or
    /// a bare suffix) and, with `integer`, a step of at least one.
    ///
    /// # Panics
    ///
    /// Panics if `min` or `max` is not finite.
    pub fn linear_with(min: f64, max: f64, target: usize, options: &TickOptions) -> Ticks {
        assert!(
            min.is_finite() && max.is_finite(),
            "Ticks::linear requires finite bounds, got {min} and {max}"
        );
        let (lo, hi) = if min <= max { (min, max) } else { (max, min) };
        let target = target.max(2);
        // Binary bytes search in their own unit, so the ticks are nice there.
        let binary = matches!(options.unit, Unit::Bytes).then(|| {
            let (power, factor) = binary_prefix(lo.abs().max(hi.abs()));
            (factor, BINARY_UNITS[power])
        });
        let (factor, unit_name) = binary.unwrap_or((1.0, ""));
        let (slo, shi) = (lo / factor, hi / factor);
        let min_step = if options.integer { 1.0 } else { 0.0 };
        if slo == shi {
            return Ticks {
                ticks: with_unit(endpoint_ticks(slo, shi), factor, unit_name, &options.unit),
                step: None,
            };
        }
        match search(slo, shi, target, min_step) {
            Some(best) => materialize(&best, options, factor, unit_name),
            None => Ticks {
                ticks: with_unit(endpoint_ticks(slo, shi), factor, unit_name, &options.unit),
                step: Some(hi - lo),
            },
        }
    }

    /// One tick per band category at its index, labels fitted to `budget` cells
    /// with `ellipsis`. Band geometry decides where they land; this only carries
    /// the (position, label) pairs through the tick pipeline, so band axes reuse
    /// the same chrome as numeric ones.
    pub(crate) fn bands(categories: &[String], budget: usize, ellipsis: char) -> Ticks {
        Ticks {
            ticks: categories
                .iter()
                .enumerate()
                .map(|(index, category)| Tick {
                    value: index as f64,
                    label: crate::render::fit_width_with(category, budget, ellipsis),
                })
                .collect(),
            step: Some(1.0),
        }
    }

    /// Places decade ticks (`10²`-style labels) over the positive range
    /// `[min, max]`, striding decades when there are many more than `target`.
    ///
    /// Ranges narrower than one full decade fall back to [`Ticks::linear`] —
    /// value labels read better than fractional powers there.
    ///
    /// # Panics
    ///
    /// Panics if the bounds are not finite and positive.
    pub fn log10(min: f64, max: f64, target: usize) -> Ticks {
        assert!(
            min.is_finite() && max.is_finite() && min > 0.0 && max > 0.0,
            "Ticks::log10 requires finite positive bounds, got {min} and {max}"
        );
        let (lo, hi) = if min <= max { (min, max) } else { (max, min) };
        let target = target.max(2);
        let first = lo.log10().ceil() as i32;
        let last = hi.log10().floor() as i32;
        if last - first < 1 {
            return Ticks::linear(lo, hi, target);
        }
        let decades = (last - first + 1) as usize;
        let stride = decades.div_ceil(target).max(1) as i32;
        // Strided decades prefer multiples of the stride (10⁰, 10³, 10⁶ rather
        // than 10¹, 10⁴, 10⁷) whenever that phase yields as many ticks as any
        // other; a phase that keeps one more tick in range wins otherwise.
        let start_for = |phase: i32| first + (phase - first).rem_euclid(stride);
        let count_for = |phase: i32| {
            let start = start_for(phase);
            if start > last {
                0
            } else {
                (last - start) / stride + 1
            }
        };
        let most = (0..stride).map(count_for).max().unwrap_or(0);
        let phase = (0..stride)
            .find(|&phase| count_for(phase) == most)
            .unwrap_or(0);
        let ticks: Vec<Tick> = (start_for(phase)..=last)
            .step_by(stride as usize)
            .map(|power| Tick {
                value: 10f64.powi(power),
                label: power_of_ten_label(power),
            })
            .collect();
        Ticks { ticks, step: None }
    }

    /// Builds a set from precomputed ticks (time axes build these); no uniform step.
    pub(crate) fn from_time(ticks: Vec<Tick>) -> Ticks {
        Ticks { ticks, step: None }
    }

    /// The ticks, ascending.
    pub fn as_slice(&self) -> &[Tick] {
        &self.ticks
    }

    /// Iterates over the ticks, ascending.
    pub fn iter(&self) -> std::slice::Iter<'_, Tick> {
        self.ticks.iter()
    }

    /// The number of ticks; at least 1.
    pub fn len(&self) -> usize {
        self.ticks.len()
    }

    /// Whether there are no ticks. Never true for values produced by this crate.
    pub fn is_empty(&self) -> bool {
        self.ticks.is_empty()
    }

    /// The spacing between adjacent ticks in data coordinates, or `None` when there
    /// is no single spacing: a lone tick, or the non-uniform ticks of a log or time
    /// axis. Returned as an option rather than a sentinel `0`, which a caller could
    /// mistake for a real spacing.
    pub fn step(&self) -> Option<f64> {
        self.step
    }
}

impl<'a> IntoIterator for &'a Ticks {
    type Item = &'a Tick;
    type IntoIter = std::slice::Iter<'a, Tick>;

    fn into_iter(self) -> Self::IntoIter {
        self.ticks.iter()
    }
}

/// A labeling candidate: `count` values at `(start + t * skip) * step_mantissa`,
/// scaled by `10^exp10`.
struct Candidate {
    start: i128,
    skip: i128,
    step_mantissa: i128,
    exp10: i32,
    count: usize,
}

/// Search caps. The score-based pruning below terminates the loops on its own for
/// sane inputs; the caps are defensive bounds so the library can never spin.
const MAX_SKIP: i128 = 20;
const MAX_MAGNITUDE_STEPS: i32 = 60;

/// Start indices beyond this magnitude would overflow the exact-decimal mantissa;
/// such ranges (huge value, tiny span) fall back to plain endpoint ticks.
const MAX_INDEX: f64 = 1e14;

fn search(dmin: f64, dmax: f64, target: usize, min_step: f64) -> Option<Candidate> {
    let range = dmax - dmin;
    // A span so wide it overflows to infinity has no nice ticks; fall back to the
    // endpoints rather than driving the magnitude search into non-finite arithmetic.
    if !range.is_finite() {
        return None;
    }
    let target_f = target as f64;
    // Ten times the target covers the useful density range; cap it absolutely so an
    // extreme target cannot blow the search up (no axis wants hundreds of ticks).
    let count_cap = target.saturating_mul(10).saturating_add(10).min(200);
    let mut best: Option<Candidate> = None;
    let mut best_score = f64::NEG_INFINITY;

    'search: for skip in 1..=MAX_SKIP {
        let skip_f = skip as f64;
        for (rank, &(step_mantissa, step_value)) in STEPS.iter().enumerate() {
            let s_max = simplicity_max(rank, skip_f);
            if score(s_max, 1.0, 1.0) <= best_score {
                // Simplicity only decreases for later mantissas and larger skips.
                break 'search;
            }
            for count in 2..=count_cap {
                let d_max = density_max(count, target_f);
                if score(s_max, 1.0, d_max) <= best_score {
                    break;
                }
                let delta = range / (count as f64 + 1.0) / (skip_f * step_value);
                let z0 = delta.log10().ceil() as i32;
                for dz in 0..MAX_MAGNITUDE_STEPS {
                    let z = z0 + dz;
                    let step = skip_f * step_value * 10f64.powi(z);
                    if step < min_step {
                        // Whole-number axes: a finer step is not a candidate.
                        continue;
                    }
                    let span = step * (count as f64 - 1.0);
                    let c_max = coverage_max(dmin, dmax, span);
                    if score(s_max, c_max, d_max) <= best_score {
                        break;
                    }
                    let min_start = (dmax / step).floor() * skip_f - (count as f64 - 1.0) * skip_f;
                    let max_start = (dmin / step).ceil() * skip_f;
                    if !(min_start.abs() <= MAX_INDEX && max_start.abs() <= MAX_INDEX) {
                        continue;
                    }
                    if min_start > max_start {
                        continue;
                    }
                    // The useful grid offsets span at most a few tick counts; a
                    // window wider than that means a step so fine the candidate is
                    // hopeless anyway. Skip it rather than walk 10^14 start indices.
                    if max_start - min_start > 4.0 * count as f64 {
                        continue;
                    }
                    let unit = step_value * 10f64.powi(z);
                    for start in (min_start as i128)..=(max_start as i128) {
                        let l_min = start as f64 * unit;
                        let l_max = l_min + span;
                        let s = simplicity(rank, skip_f, zero_included(start, skip, count));
                        let c = coverage(dmin, dmax, l_min, l_max);
                        let d = density(count, target_f, dmin, dmax, l_min, l_max);
                        let total = score_full(s, c, d, 1.0);
                        if total > best_score {
                            best_score = total;
                            best = Some(Candidate {
                                start,
                                skip,
                                step_mantissa,
                                exp10: z - 1,
                                count,
                            });
                        }
                    }
                }
            }
        }
    }
    best
}

fn materialize(
    candidate: &Candidate,
    options: &TickOptions,
    factor: f64,
    unit_name: &str,
) -> Ticks {
    let mut mantissas: Vec<i128> = (0..candidate.count)
        .map(|t| (candidate.start + t as i128 * candidate.skip) * candidate.step_mantissa)
        .collect();
    let mut exp10 = candidate.exp10;
    while exp10 < 0 && mantissas.iter().all(|mantissa| mantissa % 10 == 0) {
        for mantissa in &mut mantissas {
            *mantissa /= 10;
        }
        exp10 += 1;
    }
    let scaling = match (&options.unit, scaling(&mantissas, exp10)) {
        // Binary bytes carry their own prefix and a bare suffix carries none:
        // neither takes an SI prefix; only the exponent form survives.
        (Unit::Bytes | Unit::Suffix(_), Scaling::Prefix { .. }) => Scaling::Plain,
        (_, scaling) => scaling,
    };
    let ticks: Vec<Tick> = mantissas
        .iter()
        .map(|&mantissa| {
            let numeric = match scaling {
                // On a prefixed or exponent axis zero is deliberately bare.
                Scaling::Prefix { .. } | Scaling::Exponent(_) if mantissa == 0 => "0".to_string(),
                Scaling::Prefix { shift, .. } => format::decimal(mantissa, exp10 - shift),
                Scaling::Exponent(exponent) => {
                    format!("{}e{exponent}", format::decimal(mantissa, exp10 - exponent))
                }
                Scaling::Plain => format::decimal(mantissa, exp10),
            };
            let prefix = match scaling {
                // A plain prefixed zero stays a bare `0`; with a unit the
                // prefix is the axis's and every label reads in it.
                Scaling::Prefix { .. } if mantissa == 0 && options.unit.is_plain() => String::new(),
                Scaling::Prefix { suffix, .. } => suffix.to_string(),
                _ => String::new(),
            };
            let label = match &options.unit {
                Unit::Plain => format!("{numeric}{prefix}"),
                Unit::Si(unit) => format!("{numeric} {prefix}{unit}"),
                Unit::Suffix(suffix) => format!("{numeric}{suffix}"),
                Unit::Bytes => format!("{numeric} {unit_name}"),
            };
            Tick {
                value: value_of(mantissa, exp10) * factor,
                label,
            }
        })
        .collect();
    // Computed from the integer mantissa difference, so the step itself is
    // decimal-exact (0.8, never 0.8000000000000003).
    let step = value_of(mantissas[1] - mantissas[0], exp10) * factor;
    Ticks {
        ticks,
        step: Some(step),
    }
}

/// Endpoint fallback ticks carrying the axis's unit: the bytes factor
/// restored, the unit or suffix appended, an SI prefix on the label spaced
/// off from the number.
fn with_unit(ticks: Vec<Tick>, factor: f64, unit_name: &str, unit: &Unit) -> Vec<Tick> {
    ticks
        .into_iter()
        .map(|tick| {
            let label = match unit {
                Unit::Plain => tick.label,
                Unit::Suffix(suffix) => format!("{}{suffix}", tick.label),
                Unit::Bytes => format!("{} {unit_name}", tick.label),
                Unit::Si(unit) => {
                    let prefixed = tick
                        .label
                        .chars()
                        .last()
                        .is_some_and(|last| "kMGTµnp".contains(last));
                    if prefixed {
                        let (number, prefix) = tick.label.split_at(
                            tick.label.len() - tick.label.chars().last().map_or(0, char::len_utf8),
                        );
                        format!("{number} {prefix}{unit}")
                    } else {
                        format!("{} {unit}", tick.label)
                    }
                }
            };
            Tick {
                value: tick.value * factor,
                label,
            }
        })
        .collect()
}

/// Formats `10^power` with Unicode superscripts: `1`, `10`, `10²`, `10⁻³`.
fn power_of_ten_label(power: i32) -> String {
    const DIGITS: [char; 10] = ['⁰', '¹', '²', '³', '⁴', '⁵', '⁶', '⁷', '⁸', '⁹'];
    match power {
        0 => "1".to_string(),
        1 => "10".to_string(),
        _ => {
            let mut label = String::from("10");
            if power < 0 {
                label.push('⁻');
            }
            let mut digits = Vec::new();
            let mut remaining = power.unsigned_abs();
            while remaining > 0 {
                digits.push(DIGITS[(remaining % 10) as usize]);
                remaining /= 10;
            }
            label.extend(digits.into_iter().rev());
            label
        }
    }
}

/// How one axis's labels are scaled: plain decimals, one SI prefix, or one
/// power of ten.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Scaling {
    Plain,
    Prefix { shift: i32, suffix: char },
    Exponent(i32),
}

/// Chooses one scaling for a whole axis, from the magnitude of its largest tick:
/// an SI prefix at ten thousand and up (`k`, `M`, `G`, `T`) or below a
/// thousandth (`µ`, `n`, `p`), and one shared power of ten beyond the prefix
/// table (`8.796e-100`), so no label ever spells a hundred zeros. Zero keeps
/// its bare label. The numeric part of a scaled label times its factor equals
/// the tick value exactly.
fn scaling(mantissas: &[i128], exp10: i32) -> Scaling {
    let Some(max) = mantissas.iter().map(|m| m.unsigned_abs()).max() else {
        return Scaling::Plain;
    };
    if max == 0 {
        return Scaling::Plain;
    }
    let digits = max.to_string().len() as i32;
    let magnitude = digits - 1 + exp10;
    if magnitude < 4 && magnitude > -4 {
        return Scaling::Plain;
    }
    let shift = 3 * magnitude.div_euclid(3);
    let suffix = match shift {
        3 => 'k',
        6 => 'M',
        9 => 'G',
        12 => 'T',
        -6 => '\u{00B5}',
        -9 => 'n',
        -12 => 'p',
        _ => return Scaling::Exponent(magnitude),
    };
    Scaling::Prefix { shift, suffix }
}

/// The fallback ticks for a range the search cannot cover — equal bounds, a
/// span that overflows, a magnitude whose start index the exact mantissa
/// cannot hold: the bounds themselves, formatted by the shared set formatter
/// at its significant-digit budget (one SI prefix or one power of ten for
/// both), widened just enough that two distinct bounds read differently.
/// These labels are the data's bounds rounded to a budget and never a chosen
/// tick; the search's ticks stay exact decimals.
pub(crate) fn endpoint_ticks(lo: f64, hi: f64) -> Vec<Tick> {
    if lo == hi {
        return vec![Tick {
            value: lo,
            label: format::NumberFormat::for_values(&[lo]).format(lo),
        }];
    }
    let mut set = format::NumberFormat::for_values(&[lo, hi]);
    for _ in 0..MAX_ENDPOINT_WIDENING {
        if set.format(lo) != set.format(hi) {
            break;
        }
        set = set.widened(1);
    }
    vec![
        Tick {
            value: lo,
            label: set.format(lo),
        },
        Tick {
            value: hi,
            label: set.format(hi),
        },
    ]
}

/// Fraction digits an endpoint fallback may add to separate two bounds;
/// seventeen significant digits distinguish any two `f64`s that differ.
const MAX_ENDPOINT_WIDENING: usize = 20;

/// Converts `mantissa * 10^exp10` to the nearest `f64`.
///
/// For `|exp10| <= 22` the power of ten is exact and the single multiplication or
/// division rounds correctly, so the result equals what parsing the decimal label
/// produces. Beyond that the decimal is parsed outright — the standard parser
/// rounds correctly where a chain of multiplications would not — so a label
/// still decodes to its tick at every magnitude.
fn value_of(mantissa: i128, exp10: i32) -> f64 {
    let m = mantissa as f64;
    let e = exp10.unsigned_abs() as usize;
    match POW10.get(e) {
        Some(&p) if exp10 >= 0 => m * p,
        Some(&p) => m / p,
        None => format!("{mantissa}e{exp10}")
            .parse()
            .unwrap_or_else(|_| m * 10f64.powi(exp10)),
    }
}

fn zero_included(start: i128, skip: i128, count: usize) -> bool {
    start <= 0 && start + (count as i128 - 1) * skip >= 0 && start.rem_euclid(skip) == 0
}

fn simplicity(rank: usize, skip: f64, zero: bool) -> f64 {
    let n = (STEPS.len() - 1) as f64;
    1.0 - rank as f64 / n - skip + if zero { 1.0 } else { 0.0 }
}

fn simplicity_max(rank: usize, skip: f64) -> f64 {
    let n = (STEPS.len() - 1) as f64;
    1.0 - rank as f64 / n - skip + 1.0
}

/// The paper's coverage term, `1 − ½ · ((dmax − lmax)² + (dmin − lmin)²) / (0.1 · range)²`,
/// computed on overshoot *ratios* so a range near `1e300` (whose square
/// overflows) or `1e-200` (whose square underflows) scores like any other:
/// a `NaN` here would defeat every pruning comparison and leave the search
/// enumerating its whole space before giving up.
fn coverage(dmin: f64, dmax: f64, l_min: f64, l_max: f64) -> f64 {
    let range = dmax - dmin;
    let high = (dmax - l_max) / range;
    let low = (dmin - l_min) / range;
    1.0 - 50.0 * (high * high + low * low)
}

fn coverage_max(dmin: f64, dmax: f64, span: f64) -> f64 {
    let range = dmax - dmin;
    if span > range {
        let half = (span - range) / 2.0 / range;
        1.0 - 100.0 * half * half
    } else {
        1.0
    }
}

fn density(count: usize, target: f64, dmin: f64, dmax: f64, l_min: f64, l_max: f64) -> f64 {
    let r = (count as f64 - 1.0) / (l_max - l_min);
    let rt = (target - 1.0) / (dmax.max(l_max) - dmin.min(l_min));
    2.0 - (r / rt).max(rt / r)
}

fn density_max(count: usize, target: f64) -> f64 {
    if count as f64 >= target {
        2.0 - (count as f64 - 1.0) / (target - 1.0)
    } else {
        1.0
    }
}

/// Upper-bound score used for pruning; legibility is at its maximum of 1.
fn score(simplicity: f64, coverage: f64, density: f64) -> f64 {
    score_full(simplicity, coverage, density, 1.0)
}

fn score_full(simplicity: f64, coverage: f64, density: f64, legibility: f64) -> f64 {
    WEIGHTS[0] * simplicity + WEIGHTS[1] * coverage + WEIGHTS[2] * density + WEIGHTS[3] * legibility
}

#[cfg(test)]
#[path = "tests/ticks_tests.rs"]
mod tests;

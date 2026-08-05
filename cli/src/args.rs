//! Argument model and the lexopt parser.
//!
//! Every flag names an existing malevich concept — a [`Frame`](malevich::Frame)
//! field, a preset argument, a scale option, or plot furniture (D-C11). Parsing is
//! flag-uniform: the subcommand only selects the chart mapping and the help text,
//! so one loop handles every option regardless of which chart follows.

use std::path::PathBuf;

use lexopt::prelude::*;
use malevich::Charset;

/// The chart subcommand.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Line,
    Scatter,
    Bar,
    Hist,
    Count,
    Density,
    Box,
    Ecdf,
    Violin,
    Hist2d,
    Heatmap,
}

impl Command {
    /// Resolves a subcommand name or its one-letter alias.
    fn parse(name: &str) -> Option<Command> {
        Some(match name {
            "line" | "l" => Command::Line,
            "scatter" | "s" => Command::Scatter,
            "bar" | "b" => Command::Bar,
            "hist" => Command::Hist,
            "count" | "c" => Command::Count,
            "density" | "d" => Command::Density,
            "box" => Command::Box,
            "ecdf" => Command::Ecdf,
            "violin" => Command::Violin,
            "hist2d" => Command::Hist2d,
            "heatmap" => Command::Heatmap,
            _ => return None,
        })
    }

    /// Whether `--time-x` applies: only charts with a numeric x drawn from an
    /// input column.
    pub fn has_time_axis(self) -> bool {
        matches!(self, Command::Line | Command::Scatter | Command::Hist2d)
    }

    /// The canonical name, for messages.
    pub fn name(self) -> &'static str {
        match self {
            Command::Line => "line",
            Command::Scatter => "scatter",
            Command::Bar => "bar",
            Command::Hist => "hist",
            Command::Count => "count",
            Command::Density => "density",
            Command::Box => "box",
            Command::Ecdf => "ecdf",
            Command::Violin => "violin",
            Command::Hist2d => "hist2d",
            Command::Heatmap => "heatmap",
        }
    }
}

/// Where the plot is written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Output {
    /// The default: plot on stderr, stdout free for data.
    Stderr,
    /// `-o -`: plot on stdout (disables `-O`).
    Stdout,
    /// `-o FILE`: a plain frame written to a file.
    File(PathBuf),
}

/// How columns map onto axes (D-C6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fmt {
    /// Each column a y-series; x is the row index.
    Y,
    /// First column x, second column y (a single series).
    Xy,
    /// First column x, every remaining column a y-series sharing it.
    Xyy,
    /// Columns pair up: `(x0,y0)`, `(x1,y1)`, … — each pair its own series.
    Xyxy,
    /// First column y, second column x (YouPlot compatibility).
    Yx,
}

impl Fmt {
    fn parse(name: &str) -> Option<Fmt> {
        Some(match name {
            "y" => Fmt::Y,
            "xy" => Fmt::Xy,
            "xyy" => Fmt::Xyy,
            "xyxy" => Fmt::Xyxy,
            "yx" => Fmt::Yx,
            _ => return None,
        })
    }
}

/// The `--color` escape hatch. `Auto` and the two overrides ride the NO_COLOR /
/// CLICOLOR_FORCE precedence malevich already documents (see `output`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorChoice {
    #[default]
    Auto,
    Always,
    Never,
}

/// The `--pixels` ladder (D-C10).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PixelsChoice {
    /// Real image when the destination is a terminal that speaks a protocol.
    #[default]
    Auto,
    /// Attempt pixels even from a pipe (falls back to cells when undetected).
    Always,
    /// Never pixels; always cell output.
    Never,
}

/// The `--charset` override. `Auto` keeps the detected tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CharsetChoice {
    #[default]
    Auto,
    Fixed(Charset),
}

impl CharsetChoice {
    fn parse(name: &str) -> Option<CharsetChoice> {
        Some(match name {
            "auto" => CharsetChoice::Auto,
            "ascii" => CharsetChoice::Fixed(Charset::Ascii),
            "half" => CharsetChoice::Fixed(Charset::HalfBlocks),
            "quad" => CharsetChoice::Fixed(Charset::Quadrants),
            "sextant" => CharsetChoice::Fixed(Charset::Sextants),
            "octant" => CharsetChoice::Fixed(Charset::Octants),
            "braille" => CharsetChoice::Fixed(Charset::Braille),
            _ => return None,
        })
    }
}

/// A fully parsed invocation of one chart subcommand.
#[derive(Debug, Clone)]
pub struct Args {
    pub command: Command,
    /// A positional input file, or stdin when absent.
    pub input: Option<PathBuf>,
    pub output: Output,
    pub passthrough: bool,
    pub delimiter: Option<char>,
    pub header: bool,
    pub fmt: Option<Fmt>,
    pub width: Option<usize>,
    pub height: Option<usize>,
    pub title: Option<String>,
    pub xlabel: Option<String>,
    pub ylabel: Option<String>,
    pub xlim: Option<(f64, f64)>,
    pub ylim: Option<(f64, f64)>,
    pub log_x: bool,
    pub log_y: bool,
    pub time_x: bool,
    /// Explicit histogram bin count (`--bins`); auto when absent.
    pub bins: Option<usize>,
    pub color: ColorChoice,
    pub charset: CharsetChoice,
    pub pixels: PixelsChoice,
    pub quiet: bool,
    /// Live streaming mode (`--live`): read stdin forever, repaint in place.
    pub live: bool,
    /// Sliding-window length (`--window`); the frame width when absent.
    pub window: Option<usize>,
    /// Repaint throttle in frames per second (`--fps`); 10 when absent.
    pub fps: Option<usize>,
    /// Plot the per-sample delta of a monotonic counter (`--rate`).
    pub rate: bool,
}

/// What a parse resolved to: run a chart, or a meta action that prints and exits.
#[derive(Debug, Clone)]
pub enum Outcome {
    Run(Box<Args>),
    /// `--help`: top-level when no subcommand was seen, else that subcommand's page.
    Help(Option<Command>),
    Version,
}

/// A usage error. Printed as `kaz: {0}` to stderr; exit code 2.
#[derive(Debug)]
pub struct Fail(pub String);

impl std::fmt::Display for Fail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<lexopt::Error> for Fail {
    fn from(error: lexopt::Error) -> Fail {
        Fail(error.to_string())
    }
}

/// Parses the process arguments into an [`Outcome`].
pub fn parse() -> Result<Outcome, Fail> {
    parse_from(lexopt::Parser::from_env())
}

fn parse_from(mut parser: lexopt::Parser) -> Result<Outcome, Fail> {
    let mut command: Option<Command> = None;
    let mut input: Option<PathBuf> = None;
    let mut output = Output::Stderr;
    let mut passthrough = false;
    let mut delimiter = None;
    let mut header = false;
    let mut fmt = None;
    let mut width = None;
    let mut height = None;
    let mut title = None;
    let mut xlabel = None;
    let mut ylabel = None;
    let mut xlim = None;
    let mut ylim = None;
    let mut log_x = false;
    let mut log_y = false;
    let mut time_x = false;
    let mut bins = None;
    let mut color = ColorChoice::Auto;
    let mut charset = CharsetChoice::Auto;
    let mut pixels = PixelsChoice::Auto;
    let mut quiet = false;
    let mut live = false;
    let mut window = None;
    let mut fps = None;
    let mut rate = false;

    while let Some(arg) = parser.next()? {
        match arg {
            // `--help` is long-only: `-h` is reserved for height (D: the flag budget
            // is one screen, and height earns the short form more than help does).
            Long("help") => return Ok(Outcome::Help(command)),
            Short('V') | Long("version") => return Ok(Outcome::Version),
            Value(value) => {
                if command.is_none() {
                    let name = value.to_string_lossy();
                    command = Some(Command::parse(&name).ok_or_else(|| {
                        Fail(format!("unknown subcommand `{name}` (try `kaz --help`)"))
                    })?);
                } else if input.is_none() {
                    input = Some(PathBuf::from(value));
                } else {
                    return Err(Fail(format!(
                        "unexpected extra argument `{}`",
                        value.to_string_lossy()
                    )));
                }
            }
            Short('o') | Long("output") => {
                let value = parser.value()?.string()?;
                output = if value == "-" {
                    Output::Stdout
                } else {
                    Output::File(PathBuf::from(value))
                };
            }
            Short('O') => passthrough = true,
            Short('d') | Long("delimiter") => {
                let value = parser.value()?.string()?;
                let mut chars = value.chars();
                match (chars.next(), chars.next()) {
                    (Some(c), None) => delimiter = Some(c),
                    _ => {
                        return Err(Fail(format!(
                            "-d takes a single character, got `{value}` \
                             (whitespace is the default; real CSV: pipe through `xsv`/`mlr`)"
                        )));
                    }
                }
            }
            Short('H') | Long("header") => header = true,
            Long("fmt") => {
                let value = parser.value()?.string()?;
                fmt = Some(Fmt::parse(&value).ok_or_else(|| {
                    Fail(format!("--fmt is one of y|xy|xyy|xyxy|yx, got `{value}`"))
                })?);
            }
            Short('w') | Long("width") => width = Some(parser.value()?.parse()?),
            Short('h') | Long("height") => height = Some(parser.value()?.parse()?),
            Short('t') | Long("title") => title = Some(parser.value()?.string()?),
            Long("xlabel") => xlabel = Some(parser.value()?.string()?),
            Long("ylabel") => ylabel = Some(parser.value()?.string()?),
            Long("xlim") => xlim = Some(parse_pair("--xlim", &parser.value()?.string()?)?),
            Long("ylim") => ylim = Some(parse_pair("--ylim", &parser.value()?.string()?)?),
            Long("log-x") => log_x = true,
            Long("log-y") => log_y = true,
            Long("time-x") => time_x = true,
            Long("bins") => {
                let n: usize = parser.value()?.parse()?;
                if n == 0 {
                    return Err(Fail("--bins needs at least one bin".into()));
                }
                bins = Some(n);
            }
            Long("color") => {
                let value = parser.value()?.string()?;
                color = match value.as_str() {
                    "auto" => ColorChoice::Auto,
                    "always" => ColorChoice::Always,
                    "never" => ColorChoice::Never,
                    _ => return Err(Fail(format!("--color is auto|always|never, got `{value}`"))),
                };
            }
            Long("charset") => {
                let value = parser.value()?.string()?;
                charset = CharsetChoice::parse(&value).ok_or_else(|| {
                    Fail(format!(
                        "--charset is auto|ascii|half|quad|sextant|braille|octant, got `{value}`"
                    ))
                })?;
            }
            Long("pixels") => {
                let value = parser.value()?.string()?;
                pixels = match value.as_str() {
                    "auto" => PixelsChoice::Auto,
                    "always" => PixelsChoice::Always,
                    "never" => PixelsChoice::Never,
                    _ => {
                        return Err(Fail(format!(
                            "--pixels is auto|always|never, got `{value}`"
                        )));
                    }
                };
            }
            Short('q') | Long("quiet") => quiet = true,
            Long("live") => live = true,
            Long("window") => {
                let n: usize = parser.value()?.parse()?;
                if n == 0 {
                    return Err(Fail("--window needs at least one sample".into()));
                }
                window = Some(n);
            }
            Long("fps") => {
                let n: usize = parser.value()?.parse()?;
                if n == 0 {
                    return Err(Fail("--fps needs at least one frame per second".into()));
                }
                fps = Some(n);
            }
            Long("rate") => rate = true,
            _ => return Err(Fail(arg.unexpected().to_string())),
        }
    }

    let Some(command) = command else {
        // No subcommand: `kaz` alone shows the top-level page.
        return Ok(Outcome::Help(None));
    };

    // `-o -` puts the plot on stdout; there is no data channel left to pass through.
    if passthrough && output == Output::Stdout {
        return Err(Fail(
            "-O passes input to stdout, but -o - already sends the plot there".into(),
        ));
    }

    if live {
        if command != Command::Line {
            return Err(Fail(format!(
                "--live streams a single line; `{}` is not supported",
                command.name()
            )));
        }
        if passthrough {
            return Err(Fail("-O is not supported with --live".into()));
        }
        if matches!(output, Output::File(_)) {
            return Err(Fail(
                "--live repaints a terminal; -o FILE is not supported".into(),
            ));
        }
        // The x axis is the sliding window itself, and input is one value per
        // line; flags that shape a data x axis or reframe columns would
        // silently do nothing — reject them like the stray live flags below.
        if time_x || xlim.is_some() || log_x || fmt.is_some() || header {
            return Err(Fail(
                "--live plots a sliding window of single values; \
                 --time-x/--xlim/--log-x/--fmt/-H do not apply"
                    .into(),
            ));
        }
        if pixels == PixelsChoice::Always {
            return Err(Fail(
                "--live repaints cells in place; --pixels always is not supported".into(),
            ));
        }
    } else if window.is_some() || fps.is_some() || rate {
        return Err(Fail("--window/--fps/--rate only apply with --live".into()));
    }

    // A flag the chosen chart would silently ignore is a lie, not a no-op.
    if bins.is_some() && command != Command::Hist {
        return Err(Fail(format!(
            "--bins only applies to hist, not `{}`",
            command.name()
        )));
    }
    if fmt.is_some() && !matches!(command, Command::Line | Command::Scatter) {
        return Err(Fail(format!(
            "--fmt only applies to line and scatter, not `{}`",
            command.name()
        )));
    }
    if time_x && !command.has_time_axis() {
        return Err(Fail(format!(
            "--time-x only applies to line, scatter, and hist2d, not `{}`",
            command.name()
        )));
    }

    Ok(Outcome::Run(Box::new(Args {
        command,
        input,
        output,
        passthrough,
        delimiter,
        header,
        fmt,
        width,
        height,
        title,
        xlabel,
        ylabel,
        xlim,
        ylim,
        log_x,
        log_y,
        time_x,
        bins,
        color,
        charset,
        pixels,
        quiet,
        live,
        window,
        fps,
        rate,
    })))
}

/// Parses a `A,B` numeric pair for `--xlim` / `--ylim`.
fn parse_pair(flag: &str, value: &str) -> Result<(f64, f64), Fail> {
    let (a, b) = value
        .split_once(',')
        .ok_or_else(|| Fail(format!("{flag} takes two numbers as A,B, got `{value}`")))?;
    let parse = |part: &str| {
        part.trim()
            .parse::<f64>()
            .ok()
            .filter(|value| value.is_finite())
            .ok_or_else(|| Fail(format!("{flag}: `{part}` is not a finite number")))
    };
    Ok((parse(a)?, parse(b)?))
}

#[cfg(test)]
#[path = "tests/args_tests.rs"]
mod tests;

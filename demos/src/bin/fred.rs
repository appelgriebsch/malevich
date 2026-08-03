//! FRED — a Federal Reserve economic-data browser in the terminal, built on malevich
//! and ratatui.
//!
//! Ships with a vendored snapshot of public-domain US-federal series (see
//! `demos/data/README.md`); press `f` to refresh the selected series live from
//! <https://fred.stlouisfed.org> (no API key — the CSV endpoint is open).
//!
//! Run with `cargo run -p malevich-demos --bin fred`.
//!
//! Keys: `↑/↓` or `j/k` pick a series · `t` cycles transform (level / YoY % / log) ·
//! `s` toggles NBER recession shading · `f` fetches live · `q` quits.

use std::time::Duration;

use malevich::{Area, Color, Line, Plot};
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::Line as TextLine;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};

/// One economic series: its identity, sampling frequency, and observations.
struct Series {
    id: &'static str,
    title: &'static str,
    source: &'static str,
    unit: &'static str,
    /// Observations per year — 12 monthly, 4 quarterly — used for year-over-year.
    per_year: usize,
    dates: Vec<f64>,
    values: Vec<f64>,
}

/// What to draw: the level as reported, its year-over-year percent change, or the
/// level on a logarithmic axis.
#[derive(Clone, Copy, PartialEq)]
enum Transform {
    Level,
    YearOverYear,
    Log,
}

impl Transform {
    fn next(self) -> Transform {
        match self {
            Transform::Level => Transform::YearOverYear,
            Transform::YearOverYear => Transform::Log,
            Transform::Log => Transform::Level,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Transform::Level => "level",
            Transform::YearOverYear => "year-over-year %",
            Transform::Log => "level (log axis)",
        }
    }
}

struct App {
    series: Vec<Series>,
    recessions: Vec<(f64, f64)>,
    selected: usize,
    transform: Transform,
    shade_recessions: bool,
    status: String,
}

fn main() -> std::io::Result<()> {
    let mut app = App {
        series: vendored_series(),
        recessions: recession_periods(include_str!("../../data/recession.csv")),
        selected: 0,
        transform: Transform::Level,
        shade_recessions: true,
        status: String::from("vendored snapshot — press f to refresh live from FRED"),
    };

    // Headless: `--render [ID]` prints one chart and exits — handy for piping into a
    // file or a screenshot, and it exercises the whole pipeline without a terminal.
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--render") {
        if let Some(id) = args.iter().skip_while(|a| *a != "--render").nth(1)
            && let Some(index) = app
                .series
                .iter()
                .position(|s| s.id.eq_ignore_ascii_case(id))
        {
            app.selected = index;
        }
        println!("{}", app.plot().render(&malevich::Frame::plain(110, 30)));
        return Ok(());
    }

    let mut terminal = ratatui::init();
    let mut list_state = ListState::default();
    let result = loop {
        list_state.select(Some(app.selected));
        terminal.draw(|frame| app.draw(frame, &mut list_state))?;

        if !event::poll(Duration::from_millis(250))? {
            continue;
        }
        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => break Ok(()),
            KeyCode::Down | KeyCode::Char('j') => {
                app.selected = (app.selected + 1) % app.series.len();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                app.selected = (app.selected + app.series.len() - 1) % app.series.len();
            }
            KeyCode::Char('t') => app.transform = app.transform.next(),
            KeyCode::Char('s') => app.shade_recessions = !app.shade_recessions,
            KeyCode::Char('f') => app.refresh_selected(),
            _ => {}
        }
    };
    ratatui::restore();
    result
}

impl App {
    fn draw(&self, frame: &mut ratatui::Frame, list_state: &mut ListState) {
        let [sidebar, main] =
            Layout::horizontal([Constraint::Length(30), Constraint::Fill(1)]).areas(frame.area());
        let [chart, footer] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(4)]).areas(main);

        // The series list, each with its latest value.
        let items: Vec<ListItem> = self
            .series
            .iter()
            .map(|series| {
                let latest = last_finite(&series.values)
                    .map(|v| format!("{v:.1}"))
                    .unwrap_or_else(|| "—".into());
                ListItem::new(format!("{:<8} {:>10}", series.id, latest))
            })
            .collect();
        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" FRED series "),
            )
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
        frame.render_stateful_widget(list, sidebar, list_state);

        // The chart.
        frame.render_widget(self.plot().widget(), chart);

        // Stats + keys.
        let series = &self.series[self.selected];
        let (lo, hi) = extent(&series.values);
        let change = year_change(series);
        let stats = TextLine::from(format!(
            " {}  ·  {}  ·  latest {}  ·  range {:.1}–{:.1}  ·  1y {}{:.1}%",
            series.title,
            self.transform.label(),
            last_finite(&series.values)
                .map(|v| format!("{v:.2} {}", series.unit))
                .unwrap_or_else(|| "—".into()),
            lo,
            hi,
            if change >= 0.0 { "+" } else { "" },
            change,
        ));
        let keys = TextLine::from(
            " [↑↓/jk] series   [t] transform   [s] recessions   [f] fetch live   [q] quit ",
        )
        .style(Style::default().add_modifier(Modifier::DIM));
        let status = TextLine::from(format!(" {} · source: {}", self.status, series.source))
            .style(Style::default().add_modifier(Modifier::DIM));
        frame.render_widget(
            Paragraph::new(vec![stats, keys, status]).block(Block::default().borders(Borders::ALL)),
            footer,
        );
    }

    /// Builds the malevich plot for the current series and view.
    fn plot(&self) -> Plot<'static> {
        let series = &self.series[self.selected];
        let (x, y) = match self.transform {
            Transform::Level | Transform::Log => (series.dates.clone(), series.values.clone()),
            Transform::YearOverYear => year_over_year(series),
        };

        let mut plot = Plot::new()
            .title(format!("{} ({})", series.title, series.id))
            .time_x()
            .y_label(match self.transform {
                Transform::YearOverYear => "percent",
                _ => series.unit,
            });

        // NBER recessions as a ribbon in a reserved strip just *below* the data — a
        // full-height band would fill every subpixel and swallow the line (terminals
        // have no translucency). Linear axes only; a log axis must stay positive.
        let (lo, hi) = extent(&y);
        if self.shade_recessions && self.transform != Transform::Log && lo.is_finite() && hi > lo {
            let strip = (hi - lo) * 0.06;
            plot = plot.y_domain(lo - strip, hi);
            let (first, last) = (x[0], *x.last().unwrap_or(&0.0));
            for &(start, end) in &self.recessions {
                if end >= first && start <= last {
                    plot = plot.layer(
                        Area::between([start, end], [lo - strip, lo - strip], [lo, lo])
                            .color(Color::Red),
                    );
                }
            }
        }

        plot = plot.layer(Line::xy(x, y).color(Color::Cyan));
        if self.transform == Transform::Log {
            plot = plot.log_y();
        }
        plot
    }

    /// Fetches the selected series fresh from FRED, replacing its observations.
    fn refresh_selected(&mut self) {
        let series = &mut self.series[self.selected];
        let url = format!(
            "https://fred.stlouisfed.org/graph/fredgraph.csv?id={}",
            series.id
        );
        let fetched = ureq::get(&url)
            .call()
            .map_err(|error| error.to_string())
            .and_then(|response| response.into_string().map_err(|error| error.to_string()));
        self.status = match fetched {
            Ok(body) => {
                let (dates, values) = parse_csv(&body);
                let points = values.len();
                series.dates = dates;
                series.values = values;
                format!("refreshed {} live ({points} observations)", series.id)
            }
            Err(error) => format!("live fetch failed: {error}"),
        };
    }
}

/// The vendored snapshot: public-domain US-federal series.
fn vendored_series() -> Vec<Series> {
    let spec: [(&str, &str, &str, &str, usize, &str); 6] = [
        (
            "UNRATE",
            "Unemployment Rate",
            "U.S. Bureau of Labor Statistics",
            "%",
            12,
            include_str!("../../data/unrate.csv"),
        ),
        (
            "CPIAUCSL",
            "Consumer Price Index (all items)",
            "U.S. Bureau of Labor Statistics",
            "index",
            12,
            include_str!("../../data/cpi.csv"),
        ),
        (
            "GDPC1",
            "Real Gross Domestic Product",
            "U.S. Bureau of Economic Analysis",
            "bil. chained $",
            4,
            include_str!("../../data/gdp.csv"),
        ),
        (
            "FEDFUNDS",
            "Federal Funds Effective Rate",
            "Federal Reserve Board",
            "%",
            12,
            include_str!("../../data/fedfunds.csv"),
        ),
        (
            "GS10",
            "10-Year Treasury Yield",
            "Federal Reserve Board",
            "%",
            12,
            include_str!("../../data/gs10.csv"),
        ),
        (
            "PAYEMS",
            "Total Nonfarm Payrolls",
            "U.S. Bureau of Labor Statistics",
            "thousands",
            12,
            include_str!("../../data/payems.csv"),
        ),
    ];
    spec.into_iter()
        .map(|(id, title, source, unit, per_year, csv)| {
            let (dates, values) = parse_csv(csv);
            Series {
                id,
                title,
                source,
                unit,
                per_year,
                dates,
                values,
            }
        })
        .collect()
}

/// Parses a FRED CSV (`observation_date,VALUE`) into unix-second dates and values,
/// with `.` (missing) becoming a `NaN` gap.
fn parse_csv(csv: &str) -> (Vec<f64>, Vec<f64>) {
    let mut dates = Vec::new();
    let mut values = Vec::new();
    for line in csv.lines().skip(1) {
        let Some((date, value)) = line.split_once(',') else {
            continue;
        };
        let Some(seconds) = parse_date(date) else {
            continue;
        };
        dates.push(seconds);
        values.push(value.trim().parse().unwrap_or(f64::NAN));
    }
    (dates, values)
}

/// `YYYY-MM-DD` to unix seconds (UTC midnight).
fn parse_date(date: &str) -> Option<f64> {
    let mut parts = date.split('-');
    let year: i64 = parts.next()?.parse().ok()?;
    let month: i64 = parts.next()?.parse().ok()?;
    let day: i64 = parts.next()?.parse().ok()?;
    Some(days_from_civil(year, month, day) as f64 * 86_400.0)
}

/// Days since the unix epoch for a proleptic-Gregorian date (Howard Hinnant's
/// algorithm) — the same civil-date math malevich's time axis uses internally.
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let day_of_year = (153 * (if month > 2 { month - 3 } else { month + 9 }) + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

/// Contiguous recession periods `(start, end)` in unix seconds from the USREC 0/1
/// series.
fn recession_periods(csv: &str) -> Vec<(f64, f64)> {
    let (dates, flags) = parse_csv(csv);
    let mut periods = Vec::new();
    let mut start: Option<f64> = None;
    for (&date, &flag) in dates.iter().zip(&flags) {
        match (flag >= 0.5, start) {
            (true, None) => start = Some(date),
            (false, Some(begin)) => {
                periods.push((begin, date));
                start = None;
            }
            _ => {}
        }
    }
    if let Some(begin) = start
        && let Some(&last) = dates.last()
    {
        periods.push((begin, last));
    }
    periods
}

/// The year-over-year percent change series, aligned to the later date.
fn year_over_year(series: &Series) -> (Vec<f64>, Vec<f64>) {
    let step = series.per_year;
    let mut y = vec![f64::NAN; series.values.len()];
    for (index, (then, now)) in series
        .values
        .iter()
        .zip(&series.values[step.min(series.values.len())..])
        .enumerate()
    {
        if now.is_finite() && then.is_finite() && *then != 0.0 {
            y[index + step] = (now / then - 1.0) * 100.0;
        }
    }
    (series.dates.clone(), y)
}

fn year_change(series: &Series) -> f64 {
    let (_, y) = year_over_year(series);
    last_finite(&y).unwrap_or(0.0)
}

fn last_finite(values: &[f64]) -> Option<f64> {
    values.iter().rev().copied().find(|v| v.is_finite())
}

fn extent(values: &[f64]) -> (f64, f64) {
    values
        .iter()
        .filter(|v| v.is_finite())
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), &v| {
            (lo.min(v), hi.max(v))
        })
}

//! Prints the benchmark record block for `BENCHMARKS.md` from Criterion's saved
//! results — the same honesty rule as every chart in the docs: a number in the
//! record is program output, never typed by hand.
//!
//! Run the benches first, on an otherwise idle machine, then print the block:
//!
//! ```sh
//! cargo bench --bench render
//! cargo bench --bench widget --features ratatui
//! cargo run --example bench_record                       # every saved result
//! cargo run --example bench_record -- render/line stat/  # ids containing a filter
//! ```
//!
//! The block carries the provenance the record requires (revision, machine, OS,
//! compiler, profile, sample count, measurement date) and one table row per
//! benchmark: Criterion's own point estimate and 95% confidence interval, in
//! Criterion's own units and rounding, so the row matches the `time:` line the
//! bench printed. When Criterion compared the run against a saved previous one,
//! a change column repeats its relative estimate. Paste the block under a dated
//! heading and write the prose around it; the prose is the one part a human
//! authors.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

/// One saved Criterion result.
struct Row {
    id: String,
    /// Criterion's "typical" estimate in nanoseconds: the slope where linear
    /// sampling produced one, otherwise the mean — what its `time:` line shows.
    estimate: f64,
    lower: f64,
    upper: f64,
    /// Relative change against the previous saved run, when Criterion had one.
    change: Option<f64>,
    samples: usize,
    measured: SystemTime,
}

fn main() {
    let filters: Vec<String> = std::env::args().skip(1).collect();
    let root = criterion_root();
    let mut rows = Vec::new();
    collect(&root, &mut rows);
    rows.retain(|row| filters.is_empty() || filters.iter().any(|f| row.id.contains(f.as_str())));
    rows.sort_by(|a, b| a.id.cmp(&b.id));
    if rows.is_empty() {
        eprintln!(
            "no Criterion results under {}; run `cargo bench --bench render` first",
            root.display()
        );
        std::process::exit(1);
    }

    print_provenance(&rows);
    println!();
    print_table(&rows);
}

fn criterion_root() -> PathBuf {
    let target = std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("target"));
    target.join("criterion")
}

/// Walks the Criterion tree: every directory holding `new/benchmark.json` is a
/// result; ids with slashes nest, so the walk recurses through group directories.
fn collect(dir: &Path, rows: &mut Vec<Row>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() || entry.file_name() == "report" {
            continue;
        }
        if let Some(row) = read_row(&path) {
            rows.push(row);
        } else {
            collect(&path, rows);
        }
    }
}

fn read_row(dir: &Path) -> Option<Row> {
    let new = dir.join("new");
    let benchmark = json(&new.join("benchmark.json"))?;
    let estimates = json(&new.join("estimates.json"))?;
    let typical = estimates
        .get("slope")
        .filter(|slope| !slope.is_null())
        .or_else(|| estimates.get("mean"))?;
    let interval = typical.get("confidence_interval")?;
    let samples = json(&new.join("sample.json"))
        .and_then(|sample| sample.get("times")?.as_array().map(Vec::len))
        .unwrap_or(0);
    let change = json(&dir.join("change").join("estimates.json"))
        .and_then(|change| change.get("mean")?.get("point_estimate")?.as_f64());
    let measured = fs::metadata(new.join("estimates.json"))
        .and_then(|meta| meta.modified())
        .unwrap_or(UNIX_EPOCH);
    Some(Row {
        id: benchmark.get("full_id")?.as_str()?.to_owned(),
        estimate: typical.get("point_estimate")?.as_f64()?,
        lower: interval.get("lower_bound")?.as_f64()?,
        upper: interval.get("upper_bound")?.as_f64()?,
        change,
        samples,
        measured,
    })
}

fn json(path: &Path) -> Option<Value> {
    serde_json::from_str(&fs::read_to_string(path).ok()?).ok()
}

fn print_provenance(rows: &[Row]) {
    println!("- Revision: `{}`", revision());
    println!("- Machine: {}", machine());
    println!("- OS: {}", os());
    println!("- Compiler: {}", compiler());
    let (min, max) = rows.iter().fold((usize::MAX, 0), |(lo, hi), row| {
        (lo.min(row.samples), hi.max(row.samples))
    });
    let samples = if min == max {
        format!("{min} samples per row")
    } else {
        format!("{min}–{max} samples per row")
    };
    println!(
        "- Profile: Cargo `bench` / optimized, Criterion {}, {samples}",
        criterion_version()
    );
    let mut dates: Vec<String> = rows.iter().map(|row| date(row.measured)).collect();
    dates.sort();
    dates.dedup();
    println!("- Measured: {}", dates.join(", "));
}

fn print_table(rows: &[Row]) {
    let with_change = rows.iter().any(|row| row.change.is_some());
    if with_change {
        println!("| Measurement | Estimate | 95% interval | Change |");
        println!("| --- | ---: | ---: | ---: |");
    } else {
        println!("| Measurement | Estimate | 95% interval |");
        println!("| --- | ---: | ---: |");
    }
    for row in rows {
        let (scale, unit) = unit(row.estimate);
        let estimate = format!("{} {unit}", short(row.estimate / scale));
        let interval = format!(
            "{}–{} {unit}",
            short(row.lower / scale),
            short(row.upper / scale)
        );
        if with_change {
            let change = row
                .change
                .map_or_else(|| "—".to_owned(), |c| format!("{:+.2}%", c * 100.0));
            println!("| `{}` | {estimate} | {interval} | {change} |", row.id);
        } else {
            println!("| `{}` | {estimate} | {interval} |", row.id);
        }
    }
}

/// Criterion's unit choice for a duration in nanoseconds: the divisor and label.
fn unit(ns: f64) -> (f64, &'static str) {
    if ns < 1e3 {
        (1.0, "ns")
    } else if ns < 1e6 {
        (1e3, "µs")
    } else if ns < 1e9 {
        (1e6, "ms")
    } else {
        (1e9, "s")
    }
}

/// Criterion's rounding: five significant digits down to four decimals.
fn short(n: f64) -> String {
    if n < 10.0 {
        format!("{n:.4}")
    } else if n < 100.0 {
        format!("{n:.3}")
    } else if n < 1000.0 {
        format!("{n:.2}")
    } else if n < 10000.0 {
        format!("{n:.1}")
    } else {
        format!("{n:.0}")
    }
}

fn revision() -> String {
    let short =
        output("git", &["rev-parse", "--short", "HEAD"]).unwrap_or_else(|| "unknown".into());
    let dirty = output("git", &["status", "--porcelain", "--untracked-files=no"])
        .is_some_and(|status| !status.is_empty());
    if dirty {
        format!("{short}-dirty")
    } else {
        short
    }
}

fn machine() -> String {
    if cfg!(target_os = "macos") {
        let model = sysctl("hw.model").unwrap_or_default();
        let cpu = sysctl("machdep.cpu.brand_string").unwrap_or_else(|| "unknown CPU".into());
        let cores = sysctl("hw.ncpu").unwrap_or_default();
        let memory = sysctl("hw.memsize")
            .and_then(|bytes| bytes.parse::<u64>().ok())
            .map(|bytes| format!(", {} GiB RAM", bytes >> 30))
            .unwrap_or_default();
        return format!("{model} — {cpu} ({cores} cores){memory}");
    }
    let cpuinfo = fs::read_to_string("/proc/cpuinfo").unwrap_or_default();
    let model = cpuinfo
        .lines()
        .find_map(|line| line.strip_prefix("model name"))
        .and_then(|rest| rest.split_once(':'))
        .map_or("unknown CPU", |(_, name)| name.trim());
    let cores = cpuinfo
        .lines()
        .filter(|line| line.starts_with("processor"))
        .count();
    let memory = fs::read_to_string("/proc/meminfo")
        .ok()
        .and_then(|info| {
            let line = info.lines().find(|line| line.starts_with("MemTotal:"))?;
            let kib: u64 = line.split_whitespace().nth(1)?.parse().ok()?;
            Some(format!(", {} GiB RAM", kib >> 20))
        })
        .unwrap_or_default();
    let hypervisor = if cpuinfo.contains(" hypervisor") {
        ", virtualized"
    } else {
        ""
    };
    format!("{model} ({cores} cores){memory}{hypervisor}")
}

fn os() -> String {
    let arch = output("uname", &["-m"]).unwrap_or_default();
    if cfg!(target_os = "macos") {
        let version = output("sw_vers", &["-productVersion"]).unwrap_or_default();
        let kernel = output("uname", &["-sr"]).unwrap_or_default();
        return format!("macOS {version} ({kernel}), {arch}");
    }
    let name = fs::read_to_string("/etc/os-release")
        .ok()
        .and_then(|release| {
            let line = release
                .lines()
                .find(|line| line.starts_with("PRETTY_NAME="))?;
            Some(line["PRETTY_NAME=".len()..].trim_matches('"').to_owned())
        })
        .unwrap_or_else(|| "Linux".into());
    let kernel = output("uname", &["-sr"]).unwrap_or_default();
    format!("{name} ({kernel}), {arch}")
}

fn compiler() -> String {
    let Some(verbose) = output("rustc", &["-vV"]) else {
        return "unknown rustc".into();
    };
    let banner = verbose.lines().next().unwrap_or("rustc").to_owned();
    let llvm = verbose
        .lines()
        .find_map(|line| line.strip_prefix("LLVM version: "))
        .map(|version| format!(", LLVM {version}"))
        .unwrap_or_default();
    format!("`{banner}`{llvm}")
}

fn criterion_version() -> String {
    let lock = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.lock");
    fs::read_to_string(lock)
        .ok()
        .and_then(|lock| {
            let mut lines = lock.lines();
            while let Some(line) = lines.next() {
                if line == "name = \"criterion\"" {
                    let version = lines.next()?.strip_prefix("version = ")?;
                    return Some(version.trim_matches('"').to_owned());
                }
            }
            None
        })
        .unwrap_or_else(|| "?".into())
}

fn sysctl(key: &str) -> Option<String> {
    output("sysctl", &["-n", key])
}

fn output(program: &str, args: &[&str]) -> Option<String> {
    let out = Command::new(program).args(args).output().ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_owned())
}

/// A calendar date (UTC) for a timestamp, without a date crate.
fn date(time: SystemTime) -> String {
    let days = time
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_secs() / 86_400) as i64;
    // Howard Hinnant's civil-from-days.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}")
}

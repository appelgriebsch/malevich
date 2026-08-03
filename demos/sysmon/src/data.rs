//! The data layer: sampling the machine and keeping sliding-window history.
//!
//! The history is built from malevich's own streaming primitives — each metric
//! lives in a [`malevich::stream::Ring`], the thread-shared sliding window the
//! library ships for live charts. The sampler thread `push`es and the UI thread
//! `snapshot`s; the ring's single lock is the only synchronization in the app.
//! Network counters go through [`malevich::stream::Rate`], the cumulative-counter
//! to per-interval-delta helper.

use std::time::Instant;

use malevich::stream::{Rate, Ring};
use sysinfo::{Networks, System};

/// One reading of the machine.
pub struct Sample {
    /// Total CPU utilization, percent.
    pub cpu_total: f64,
    /// Per-core utilization, percent, in core order.
    pub per_core: Vec<f64>,
    /// Memory in use, bytes.
    pub mem_used: f64,
    /// Total memory, bytes.
    pub mem_total: f64,
    /// Network receive rate, bytes per second, summed over interfaces.
    pub rx_rate: f64,
    /// Network transmit rate, bytes per second, summed over interfaces.
    pub tx_rate: f64,
}

/// Owns the sysinfo handles and produces [`Sample`]s.
pub struct Sampler {
    system: System,
    networks: Networks,
    rx: Rate,
    tx: Rate,
    last: Instant,
}

impl Sampler {
    pub fn new() -> Sampler {
        Sampler {
            system: System::new(),
            networks: Networks::new_with_refreshed_list(),
            rx: Rate::new(),
            tx: Rate::new(),
            last: Instant::now(),
        }
    }

    /// Reads the machine once. CPU percentages need two refreshes to be
    /// meaningful; call this at a steady interval (≥ 200 ms apart).
    pub fn sample(&mut self) -> Sample {
        self.system.refresh_cpu_usage();
        self.system.refresh_memory();
        self.networks.refresh(true);

        let elapsed = self.last.elapsed().as_secs_f64().max(1e-3);
        self.last = Instant::now();

        // Cumulative byte totals in, per-second rates out: `Rate::delta` returns
        // how much the counter grew since the previous call.
        let (total_rx, total_tx) =
            self.networks
                .iter()
                .fold((0.0, 0.0), |(rx, tx), (_, network)| {
                    (
                        rx + network.total_received() as f64,
                        tx + network.total_transmitted() as f64,
                    )
                });
        let rx_rate = self.rx.delta(total_rx) / elapsed;
        let tx_rate = self.tx.delta(total_tx) / elapsed;

        Sample {
            cpu_total: f64::from(self.system.global_cpu_usage()),
            per_core: self
                .system
                .cpus()
                .iter()
                .map(|cpu| f64::from(cpu.cpu_usage()))
                .collect(),
            mem_used: self.system.used_memory() as f64,
            mem_total: self.system.total_memory() as f64,
            rx_rate,
            tx_rate,
        }
    }
}

impl Default for Sampler {
    fn default() -> Sampler {
        Sampler::new()
    }
}

/// Sliding-window history of every metric, shared between the sampler thread and
/// the UI. Cloning shares the underlying rings — [`Ring`] is a thread-shared
/// window, so a clone is a second handle, not a copy.
#[derive(Clone)]
pub struct History {
    pub cpu: Ring,
    pub mem: Ring,
    pub rx: Ring,
    pub tx: Ring,
    /// One ring per core, in core order.
    pub cores: Vec<Ring>,
    /// Total memory in bytes, fixed at startup.
    pub mem_total: f64,
    /// Seconds between samples — turns ring indices into a time axis.
    pub interval: f64,
}

impl History {
    pub fn new(capacity: usize, core_count: usize, mem_total: f64, interval: f64) -> History {
        History {
            cpu: Ring::new(capacity),
            mem: Ring::new(capacity),
            rx: Ring::new(capacity),
            tx: Ring::new(capacity),
            cores: (0..core_count).map(|_| Ring::new(capacity)).collect(),
            mem_total,
            interval,
        }
    }

    /// Records one sample across all rings.
    pub fn push(&self, sample: &Sample) {
        self.cpu.push(sample.cpu_total);
        self.mem.push(sample.mem_used);
        self.rx.push(sample.rx_rate);
        self.tx.push(sample.tx_rate);
        for (ring, &core) in self.cores.iter().zip(&sample.per_core) {
            ring.push(core);
        }
    }

    /// The per-core history as a row-major grid for `Cells::matrix`: one row per
    /// core with core 0 in row 0 (the bottom row, per Cells convention), one
    /// column per sample, oldest first. Returns `(columns, values)`; empty when
    /// nothing has been sampled yet.
    pub fn core_grid(&self) -> (usize, Vec<f64>) {
        let snapshots: Vec<Vec<f64>> = self.cores.iter().map(Ring::snapshot).collect();
        let columns = snapshots.iter().map(Vec::len).max().unwrap_or(0);
        if columns == 0 {
            return (0, Vec::new());
        }
        let mut grid = Vec::with_capacity(columns * snapshots.len());
        for row in &snapshots {
            // Left-pad short rows with gaps so every row's newest sample sits in
            // the rightmost column.
            grid.extend(std::iter::repeat_n(f64::NAN, columns - row.len()));
            grid.extend_from_slice(row);
        }
        (columns, grid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(cpu: f64, cores: &[f64]) -> Sample {
        Sample {
            cpu_total: cpu,
            per_core: cores.to_vec(),
            mem_used: 8.0e9,
            mem_total: 16.0e9,
            rx_rate: 1000.0,
            tx_rate: 500.0,
        }
    }

    #[test]
    fn history_records_into_every_ring() {
        let history = History::new(8, 2, 16.0e9, 0.5);
        history.push(&sample(50.0, &[40.0, 60.0]));
        history.push(&sample(70.0, &[65.0, 75.0]));
        assert_eq!(history.cpu.snapshot(), [50.0, 70.0]);
        assert_eq!(history.cores[1].snapshot(), [60.0, 75.0]);
    }

    #[test]
    fn the_core_grid_is_row_major_with_core_zero_first() {
        let history = History::new(8, 2, 16.0e9, 0.5);
        history.push(&sample(50.0, &[10.0, 90.0]));
        history.push(&sample(50.0, &[20.0, 80.0]));
        let (columns, grid) = history.core_grid();
        assert_eq!(columns, 2);
        assert_eq!(grid, [10.0, 20.0, 90.0, 80.0]);
    }

    #[test]
    fn a_clone_shares_the_same_rings() {
        let history = History::new(8, 1, 1.0, 0.5);
        let handle = history.clone();
        history.push(&sample(33.0, &[33.0]));
        assert_eq!(handle.cpu.snapshot(), [33.0], "clone sees the push");
    }

    #[test]
    fn rings_slide_once_capacity_is_reached() {
        let history = History::new(3, 1, 1.0, 0.5);
        for value in [1.0, 2.0, 3.0, 4.0] {
            history.push(&sample(value, &[value]));
        }
        assert_eq!(history.cpu.snapshot(), [2.0, 3.0, 4.0]);
    }
}

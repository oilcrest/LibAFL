//! Monitors that log aggregated stats to disk.

use core::{
    fmt::{Debug, Formatter},
    time::Duration,
};
use std::{fs::OpenOptions, io::Write, path::PathBuf};

use libafl_bolts::{current_time, ClientId};
use serde_json::json;

use crate::{monitors::Monitor, statistics::manager::ClientStatsManager};

/// A monitor that logs aggregated stats to a JSON file.
#[derive(Clone)]
pub struct OnDiskJsonAggregateMonitor {
    json_path: PathBuf,
    last_update: Duration,
    update_interval: Duration,
}

impl Debug for OnDiskJsonAggregateMonitor {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("OnDiskJsonAggregateMonitor")
            .field("last_update", &self.last_update)
            .field("update_interval", &self.update_interval)
            .field("json_path", &self.json_path)
            .finish_non_exhaustive()
    }
}

impl Monitor for OnDiskJsonAggregateMonitor {
    fn display(
        &mut self,
        client_stats_manager: &mut ClientStatsManager,
        _event_msg: &str,
        _sender_id: ClientId,
    ) {
        // Write JSON stats if update interval has elapsed
        let cur_time = current_time();
        if cur_time - self.last_update >= self.update_interval {
            self.last_update = cur_time;

            let file = OpenOptions::new()
                .append(true)
                .create(true)
                .open(&self.json_path)
                .expect("Failed to open JSON logging file");

            let mut json_value = json!({
                "run_time": (cur_time - client_stats_manager.start_time()).as_secs(),
                "clients": client_stats_manager.client_stats_count(),
                "corpus": client_stats_manager.corpus_size(),
                "objectives": client_stats_manager.objective_size(),
                "executions": client_stats_manager.total_execs(),
                "exec_sec": client_stats_manager.execs_per_sec(),
            });

            // Add all aggregated values directly to the root
            if let Some(obj) = json_value.as_object_mut() {
                obj.extend(
                    client_stats_manager
                        .aggregated()
                        .iter()
                        .map(|(k, v)| (k.clone(), json!(v))),
                );
            }

            writeln!(&file, "{json_value}").expect("Unable to write JSON to file");
        }
    }
}

impl OnDiskJsonAggregateMonitor {
    /// Creates a new [`OnDiskJsonAggregateMonitor`]
    pub fn new<P>(json_path: P) -> Self
    where
        P: Into<PathBuf>,
    {
        Self::with_interval(json_path, Duration::from_secs(10))
    }

    /// Creates a new [`OnDiskJsonAggregateMonitor`] with custom update interval
    pub fn with_interval<P>(json_path: P, update_interval: Duration) -> Self
    where
        P: Into<PathBuf>,
    {
        Self {
            json_path: json_path.into(),
            last_update: current_time() - update_interval,
            update_interval,
        }
    }
}

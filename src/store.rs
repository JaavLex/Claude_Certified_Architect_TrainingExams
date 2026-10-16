//! Persistent attempt history (JSON in the user's data dir).

use crate::score::ScoreReport;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attempt {
    pub exam_id: String,
    pub exam_title: String,
    /// Unix seconds when the attempt started.
    pub started_at: i64,
    pub duration_secs: u64,
    pub timed_out: bool,
    pub answers: Vec<Vec<char>>,
    pub report: ScoreReport,
}

pub fn path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("exam-tui")
        .join("attempts.json")
}

pub fn load() -> Vec<Attempt> {
    std::fs::read_to_string(path())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save(attempts: &[Attempt]) -> std::io::Result<()> {
    let p = path();
    if let Some(dir) = p.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = p.with_extension("json.tmp");
    std::fs::write(&tmp, serde_json::to_string_pretty(attempts)?)?;
    std::fs::rename(tmp, p)
}

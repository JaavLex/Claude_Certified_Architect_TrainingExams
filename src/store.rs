//! Persistent attempt history and training log (JSON in the user's data dir).

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
    /// Time limit of the exam when it was taken (older records: 120 minutes).
    #[serde(default = "default_limit")]
    pub time_limit_secs: u64,
    pub timed_out: bool,
    pub answers: Vec<Vec<char>>,
    /// Seconds spent on each question (empty for attempts saved before 1.1).
    #[serde(default)]
    pub question_secs: Vec<u64>,
    pub report: ScoreReport,
}

fn default_limit() -> u64 {
    crate::exam::DEFAULT_MINUTES * 60
}

/// One question answered in training mode. Kept separate from exam attempts
/// so training never changes exam scores or history comparisons.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PracticeRecord {
    pub exam_id: String,
    /// The question's `number` in the exam file.
    pub question: usize,
    pub correct: bool,
    /// Unix seconds when it was answered.
    pub at: i64,
}

fn data_file(name: &str) -> PathBuf {
    dirs::data_dir().unwrap_or_else(|| PathBuf::from(".")).join("exam-tui").join(name)
}

pub fn path() -> PathBuf {
    data_file("attempts.json")
}

pub fn practice_path() -> PathBuf {
    data_file("practice.json")
}

fn read<T: serde::de::DeserializeOwned>(p: PathBuf) -> Vec<T> {
    std::fs::read_to_string(p).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
}

fn write<T: Serialize>(p: PathBuf, items: &[T]) -> std::io::Result<()> {
    if let Some(dir) = p.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = p.with_extension("json.tmp");
    std::fs::write(&tmp, serde_json::to_string_pretty(items)?)?;
    std::fs::rename(tmp, p)
}

pub fn load() -> Vec<Attempt> {
    read(path())
}

pub fn save(attempts: &[Attempt]) -> std::io::Result<()> {
    write(path(), attempts)
}

pub fn load_practice() -> Vec<PracticeRecord> {
    read(practice_path())
}

pub fn save_practice(records: &[PracticeRecord]) -> std::io::Result<()> {
    write(practice_path(), records)
}

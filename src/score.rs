//! Scoring per the CCAR-F Exam Guide §10 "How the Exam Is Scored":
//! criterion-referenced, scaled score 100–1,000, cut score 720, pass/fail
//! plus percent-correct by domain (domain percentages do not affect pass/fail).
//!
//! Item scoring follows the practice-exam key: 1 point per item; multiple-response
//! items earn the point only when exactly the correct set is selected (no partial
//! credit). Unanswered items score 0.
//!
//! The official raw→scaled equating is not published, so a linear mapping of
//! raw percent onto the 100–1,000 range is used.

use crate::exam::Exam;
use serde::{Deserialize, Serialize};

pub const SCALE_MIN: u32 = 100;
pub const SCALE_MAX: u32 = 1000;
pub const CUT_SCORE: u32 = 720;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainScore {
    pub domain: String,
    pub correct: usize,
    pub total: usize,
}

impl DomainScore {
    pub fn pct(&self) -> f64 {
        if self.total == 0 { 0.0 } else { self.correct as f64 * 100.0 / self.total as f64 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreReport {
    pub correct: usize,
    pub total: usize,
    pub answered: usize,
    pub scaled: u32,
    pub passed: bool,
    pub domains: Vec<DomainScore>,
}

impl ScoreReport {
    pub fn pct(&self) -> f64 {
        self.correct as f64 * 100.0 / self.total.max(1) as f64
    }
}

pub fn scaled(correct: usize, total: usize) -> u32 {
    let span = (SCALE_MAX - SCALE_MIN) as f64;
    (SCALE_MIN as f64 + span * correct as f64 / total.max(1) as f64).round() as u32
}

pub fn score(exam: &Exam, answers: &[Vec<char>]) -> ScoreReport {
    let mut correct = 0;
    let mut answered = 0;
    let mut domains: Vec<DomainScore> = ["D1", "D2", "D3", "D4", "D5"]
        .iter()
        .map(|d| DomainScore { domain: d.to_string(), correct: 0, total: 0 })
        .collect();

    for (i, q) in exam.questions.iter().enumerate() {
        let chosen = answers.get(i).map(Vec::as_slice).unwrap_or(&[]);
        if !chosen.is_empty() {
            answered += 1;
        }
        let ok = q.is_correct(chosen);
        if ok {
            correct += 1;
        }
        let d = q.primary_domain();
        let slot = match domains.iter_mut().position(|x| x.domain == d) {
            Some(p) => p,
            None => {
                domains.push(DomainScore { domain: d, correct: 0, total: 0 });
                domains.len() - 1
            }
        };
        domains[slot].total += 1;
        if ok {
            domains[slot].correct += 1;
        }
    }

    let total = exam.questions.len();
    let scaled = scaled(correct, total);
    ScoreReport { correct, total, answered, scaled, passed: scaled >= CUT_SCORE, domains }
}

/// Readiness bands from the practice-exam "Interpreting your score" section.
pub fn readiness(pct: f64) -> &'static str {
    if pct >= 90.0 {
        "Exam-ready. Skim weak-area guides."
    } else if pct >= 85.0 {
        "On track. Re-read missed domain guides."
    } else if pct >= 70.0 {
        "Review domain guides for clusters; redo missed questions."
    } else {
        "Re-read domains in priority order (1 → 5); do hands-on labs."
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scale_bounds_and_cut() {
        assert_eq!(scaled(0, 60), 100);
        assert_eq!(scaled(60, 60), 1000);
        assert!(scaled(41, 60) < CUT_SCORE);
        assert!(scaled(42, 60) >= CUT_SCORE);
    }

    #[test]
    fn multi_response_all_or_nothing() {
        let exams = crate::exam::load_all().unwrap();
        let e = &exams[0];
        let mut ans: Vec<Vec<char>> = e.questions.iter().map(|q| q.answer.clone()).collect();
        assert_eq!(score(e, &ans).correct, 60);
        ans[11] = vec!['A']; // Q12 needs A, C
        assert_eq!(score(e, &ans).correct, 59);
        ans[11] = vec!['C', 'A'];
        assert_eq!(score(e, &ans).correct, 60);
    }
}

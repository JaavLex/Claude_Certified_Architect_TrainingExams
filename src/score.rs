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

    /// Scaled points lost in each domain, most first. Under the straight-line
    /// conversion every item is worth (1,000 − 100) / items scaled points.
    pub fn points_lost(&self) -> Vec<(String, f64)> {
        let per_item = (SCALE_MAX - SCALE_MIN) as f64 / self.total.max(1) as f64;
        let mut v: Vec<(String, f64)> = self
            .domains
            .iter()
            .filter(|d| d.total > d.correct)
            .map(|d| (d.domain.clone(), (d.total - d.correct) as f64 * per_item))
            .collect();
        v.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
        v
    }
}

/// Estimate of the real-exam result, re-weighting each domain's percent
/// correct by the blueprint share of items it would have on the real exam
/// (Exam Guide §4: weights are "the approximate proportion of scored items
/// drawn from each domain"). A practice exam's own mix of domains can differ
/// a lot from the blueprint, which skews its raw score.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WeightedEstimate {
    pub pct: f64,
    pub scaled: u32,
    pub passed: bool,
    /// Sum of the weights of the domains that had items (100 when all did).
    pub coverage: f64,
}

pub fn weighted_estimate(exam: &Exam, report: &ScoreReport) -> Option<WeightedEstimate> {
    let mut sum_w = 0.0;
    let mut acc = 0.0;
    for d in &report.domains {
        let Some(w) = exam.domains.iter().find(|x| x.id == d.domain).and_then(|x| x.weight) else { continue };
        if d.total == 0 {
            continue;
        }
        sum_w += w;
        acc += w * d.pct();
    }
    if sum_w <= 0.0 {
        return None;
    }
    let pct = acc / sum_w;
    let scaled = (SCALE_MIN as f64 + (SCALE_MAX - SCALE_MIN) as f64 * pct / 100.0).round() as u32;
    Some(WeightedEstimate { pct, scaled, passed: scaled >= CUT_SCORE, coverage: sum_w })
}

pub fn scaled(correct: usize, total: usize) -> u32 {
    let span = (SCALE_MAX - SCALE_MIN) as f64;
    (SCALE_MIN as f64 + span * correct as f64 / total.max(1) as f64).round() as u32
}

pub fn score(exam: &Exam, answers: &[Vec<char>]) -> ScoreReport {
    let mut correct = 0;
    let mut answered = 0;
    // Domains in the order the exam file lists them; unlisted ones are appended.
    let mut domains: Vec<DomainScore> = exam
        .domains
        .iter()
        .map(|d| DomainScore { domain: d.id.clone(), correct: 0, total: 0 })
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

/// Readiness target for a first timed attempt (study plan §6, pre-exam checklist).
pub const FIRST_ATTEMPT_TARGET: f64 = 85.0;
/// "Exam-ready" mark (practice exam score bands, study plan §6).
pub const EXAM_READY: f64 = 90.0;

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
    fn weighted_estimate_reweights_domains() {
        let e = crate::exam::parse(include_str!("../exams/practice-exam-1.json"), "test").unwrap();
        // Everything right except all 5 D2 items: raw 55/60, but D2 is 18% of the real exam.
        let ans: Vec<Vec<char>> = e
            .questions
            .iter()
            .map(|q| if q.primary_domain() == "D2" { vec!['Z'] } else { q.answer.clone() })
            .collect();
        let r = score(&e, &ans);
        assert_eq!(r.correct, 55);
        let w = weighted_estimate(&e, &r).unwrap();
        assert!((w.pct - 82.0).abs() < 1e-9, "{}", w.pct);
        assert_eq!(w.coverage, 100.0);
        assert_eq!(w.scaled, 838);
        assert_eq!(r.points_lost(), vec![("D2".to_string(), 75.0)]);
    }

    #[test]
    fn multi_response_all_or_nothing() {
        let e = &crate::exam::parse(include_str!("../exams/practice-exam-1.json"), "test").unwrap();
        let mut ans: Vec<Vec<char>> = e.questions.iter().map(|q| q.answer.clone()).collect();
        assert_eq!(score(e, &ans).correct, 60);
        ans[11] = vec!['A']; // Q12 needs A, C
        assert_eq!(score(e, &ans).correct, 59);
        ans[11] = vec!['C', 'A'];
        assert_eq!(score(e, &ans).correct, 60);
    }
}

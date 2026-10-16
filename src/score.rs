//! Scoring per the CCAR-F Exam Guide.
//!
//! - §10: criterion-referenced, scaled score 100–1,000, cut score 720, pass/fail
//!   plus percent-correct by domain.
//! - §4: domain weights are "the approximate proportion of scored items drawn
//!   from each domain". When an exam file gives these weights, the scaled score
//!   is based on each domain's percent correct weighted by them, so a practice
//!   exam whose own mix of domains differs from the blueprint is graded as the
//!   real exam would weigh it. Domains without a weight do not count toward it.
//!   Exams without weights are graded on the plain fraction of items correct.
//!
//! Item scoring follows the practice-exam key: 1 point per item; multiple-response
//! items earn the point only when exactly the correct set is selected (no partial
//! credit). Unanswered items score 0.
//!
//! The official percent→scaled equating is not published, so the percent is
//! mapped linearly onto 100–1,000.

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
    /// Each weighted domain's share of the score, in percent (sums to 100).
    /// Empty when the exam has no domain weights.
    #[serde(default)]
    pub weights: Vec<(String, f64)>,
}

impl ScoreReport {
    /// Raw percent of items correct.
    pub fn pct(&self) -> f64 {
        self.correct as f64 * 100.0 / self.total.max(1) as f64
    }

    pub fn is_weighted(&self) -> bool {
        !self.weights.is_empty()
    }

    /// The percent the scaled score is based on: domain-weighted when the exam
    /// has weights, otherwise the raw percent.
    pub fn score_pct(&self) -> f64 {
        if !self.is_weighted() {
            return self.pct();
        }
        self.weights
            .iter()
            .map(|(id, w)| w * self.domains.iter().find(|d| &d.domain == id).map_or(0.0, |d| d.pct()) / 100.0)
            .sum()
    }

    /// The weight (share of the score) of a domain; None if it does not count.
    pub fn weight_of(&self, domain: &str) -> Option<f64> {
        self.weights.iter().find(|(id, _)| id == domain).map(|(_, w)| *w)
    }

    /// Scaled points lost in each domain, most first.
    pub fn points_lost(&self) -> Vec<(String, f64)> {
        let span = (SCALE_MAX - SCALE_MIN) as f64;
        let per_item = span / self.total.max(1) as f64;
        let mut v: Vec<(String, f64)> = self
            .domains
            .iter()
            .filter(|d| d.total > d.correct)
            .filter_map(|d| {
                let lost = if self.is_weighted() {
                    // A domain's share of the scale, times the part of it missed.
                    self.weight_of(&d.domain)? / 100.0 * (100.0 - d.pct()) / 100.0 * span
                } else {
                    (d.total - d.correct) as f64 * per_item
                };
                Some((d.domain.clone(), lost))
            })
            .collect();
        v.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
        v
    }
}

/// Map a percent (0–100) onto the 100–1,000 scale.
pub fn scale(pct: f64) -> u32 {
    (SCALE_MIN as f64 + (SCALE_MAX - SCALE_MIN) as f64 * pct / 100.0).round() as u32
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

    // Weighted domains that have questions in this exam, normalised to 100.
    let mut weights: Vec<(String, f64)> = exam
        .domains
        .iter()
        .filter_map(|d| Some((d.id.clone(), d.weight?)))
        .filter(|(id, _)| domains.iter().any(|x| &x.domain == id && x.total > 0))
        .collect();
    let sum: f64 = weights.iter().map(|(_, w)| w).sum();
    if sum > 0.0 {
        for (_, w) in &mut weights {
            *w *= 100.0 / sum;
        }
    } else {
        weights.clear();
    }

    let mut report = ScoreReport {
        correct,
        total: exam.questions.len(),
        answered,
        scaled: 0,
        passed: false,
        domains,
        weights,
    };
    report.scaled = scale(report.score_pct());
    report.passed = report.scaled >= CUT_SCORE;
    report
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

    fn exam1() -> Exam {
        crate::exam::parse(include_str!("../exams/practice-exam-1.json"), "test").unwrap()
    }

    #[test]
    fn scale_bounds_and_cut() {
        assert_eq!(scale(0.0), 100);
        assert_eq!(scale(100.0), 1000);
        // 720 needs 68.9% or more.
        assert!(scale(68.8) < CUT_SCORE);
        assert!(scale(68.9) >= CUT_SCORE);
    }

    /// The user's real 54/60 attempt: 15/16, 4/5, 6/9, 15/15, 13/14 and 1/1 Misc.
    #[test]
    fn weights_domains_like_the_blueprint() {
        let e = exam1();
        let wrong = [4, 8, 31, 34, 56, 59];
        let ans: Vec<Vec<char>> = e
            .questions
            .iter()
            .map(|q| if wrong.contains(&q.number) { vec!['Z'] } else { q.answer.clone() })
            .collect();
        let r = score(&e, &ans);
        assert_eq!((r.correct, r.total), (54, 60));
        assert!(r.is_weighted());
        // 27×15/16 + 18×4/5 + 20×6/9 + 20×1 + 15×13/14 = 86.97%; Misc has no weight.
        assert!((r.score_pct() - 86.9744).abs() < 1e-3, "{}", r.score_pct());
        assert_eq!(r.scaled, 883);
        assert!(r.passed);
        assert_eq!(r.weight_of("Misc"), None);
        let lost = r.points_lost();
        assert_eq!(lost[0].0, "D3");
        assert!((lost[0].1 - 60.0).abs() < 1e-9, "{}", lost[0].1);
    }

    #[test]
    fn missing_a_whole_domain_costs_its_weight() {
        let e = exam1();
        // Everything right except all 5 D2 items: raw 55/60, but D2 is 18% of the score.
        let ans: Vec<Vec<char>> = e
            .questions
            .iter()
            .map(|q| if q.primary_domain() == "D2" { vec!['Z'] } else { q.answer.clone() })
            .collect();
        let r = score(&e, &ans);
        assert_eq!(r.correct, 55);
        assert!((r.score_pct() - 82.0).abs() < 1e-9);
        assert_eq!(r.scaled, 838);
        assert_eq!(r.points_lost(), vec![("D2".to_string(), 18.0 / 100.0 * 900.0)]);
    }

    #[test]
    fn unweighted_exams_use_the_raw_fraction() {
        let mut e = exam1();
        for d in &mut e.domains {
            d.weight = None;
        }
        let mut ans: Vec<Vec<char>> = e.questions.iter().map(|q| q.answer.clone()).collect();
        for a in ans.iter_mut().take(6) {
            *a = vec!['Z'];
        }
        let r = score(&e, &ans);
        assert!(!r.is_weighted());
        assert_eq!(r.scaled, 910);
        assert_eq!(r.points_lost().iter().map(|(_, p)| p).sum::<f64>(), 90.0);
    }

    #[test]
    fn multi_response_all_or_nothing() {
        let e = &exam1();
        let mut ans: Vec<Vec<char>> = e.questions.iter().map(|q| q.answer.clone()).collect();
        assert_eq!(score(e, &ans).correct, 60);
        ans[11] = vec!['A']; // Q12 needs A, C
        assert_eq!(score(e, &ans).correct, 59);
        ans[11] = vec!['C', 'A'];
        assert_eq!(score(e, &ans).correct, 60);
    }
}

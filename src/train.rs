//! Weakness training: per-question history across exam attempts and training,
//! and selection of the questions you most often get wrong.
//!
//! A question is "weak" once it has been answered wrong at least once, and stays
//! weak until it is answered correctly `MASTERY_STREAK` times in a row (in exams
//! or training). Unanswered exam questions are ignored: running out of time says
//! nothing about what you know.

use crate::exam::Exam;
use crate::store::{Attempt, PracticeRecord};
use std::collections::HashMap;

pub const MASTERY_STREAK: usize = 2;

/// Session sizes offered in the setup screen; 0 means "all weak questions".
pub const SIZES: [usize; 4] = [10, 20, 30, 0];

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct QStat {
    pub seen: usize,
    pub wrong: usize,
    /// Correct answers in a row at the end of the history.
    pub streak: usize,
    pub last_wrong: bool,
}

impl QStat {
    pub fn is_weak(&self) -> bool {
        self.wrong > 0 && self.streak < MASTERY_STREAK
    }

    fn push(&mut self, correct: bool) {
        self.seen += 1;
        if correct {
            self.streak += 1;
            self.last_wrong = false;
        } else {
            self.wrong += 1;
            self.streak = 0;
            self.last_wrong = true;
        }
    }
}

/// (exam id, question number)
pub type Key = (String, usize);

pub fn stats(exams: &[Exam], attempts: &[Attempt], practice: &[PracticeRecord]) -> HashMap<Key, QStat> {
    let mut events: Vec<(i64, Key, bool)> = Vec::new();
    for a in attempts {
        let Some(e) = exams.iter().find(|e| e.id == a.exam_id) else { continue };
        for (i, q) in e.questions.iter().enumerate() {
            let chosen = a.answers.get(i).map(Vec::as_slice).unwrap_or(&[]);
            if chosen.is_empty() {
                continue;
            }
            events.push((a.started_at, (e.id.clone(), q.number), q.is_correct(chosen)));
        }
    }
    for p in practice {
        events.push((p.at, (p.exam_id.clone(), p.question), p.correct));
    }
    events.sort_by_key(|(t, _, _)| *t);

    let mut map: HashMap<Key, QStat> = HashMap::new();
    for (_, key, ok) in events {
        map.entry(key).or_default().push(ok);
    }
    map
}

#[derive(Debug, Clone)]
pub struct PoolItem {
    pub exam: usize,
    /// Index into the exam's `questions`.
    pub q: usize,
    pub stat: QStat,
}

/// Weak questions, weakest first: missed last time, then most misses, then
/// highest miss rate.
pub fn weak_pool(
    exams: &[Exam],
    stats: &HashMap<Key, QStat>,
    exam: Option<usize>,
    domain: Option<&str>,
) -> Vec<PoolItem> {
    let mut pool = Vec::new();
    for (ei, e) in exams.iter().enumerate() {
        if exam.is_some_and(|x| x != ei) {
            continue;
        }
        for (qi, q) in e.questions.iter().enumerate() {
            if domain.is_some_and(|d| q.primary_domain() != d) {
                continue;
            }
            if let Some(st) = stats.get(&(e.id.clone(), q.number)) {
                if st.is_weak() {
                    pool.push(PoolItem { exam: ei, q: qi, stat: *st });
                }
            }
        }
    }
    pool.sort_by(|a, b| {
        let rate = |s: &QStat| s.wrong as f64 / s.seen.max(1) as f64;
        b.stat
            .last_wrong
            .cmp(&a.stat.last_wrong)
            .then(b.stat.wrong.cmp(&a.stat.wrong))
            .then(rate(&b.stat).total_cmp(&rate(&a.stat)))
            .then(a.exam.cmp(&b.exam))
            .then(a.q.cmp(&b.q))
    });
    pool
}

/// Domains that have weak questions (for the given exam filter), with counts.
pub fn weak_domains(exams: &[Exam], stats: &HashMap<Key, QStat>, exam: Option<usize>) -> Vec<(String, usize)> {
    let mut out: Vec<(String, usize)> = Vec::new();
    for item in weak_pool(exams, stats, exam, None) {
        let d = exams[item.exam].questions[item.q].primary_domain();
        match out.iter_mut().find(|(x, _)| *x == d) {
            Some((_, n)) => *n += 1,
            None => out.push((d, 1)),
        }
    }
    out.sort();
    out
}

/// Take the `size` weakest questions (0 = all) and shuffle their order.
pub fn pick(mut pool: Vec<PoolItem>, size: usize, seed: u64) -> Vec<PoolItem> {
    if size > 0 {
        pool.truncate(size);
    }
    // Fisher–Yates with xorshift; no need for a crypto-grade RNG here.
    let mut x = seed | 1;
    for i in (1..pool.len()).rev() {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        pool.swap(i, (x % (i as u64 + 1)) as usize);
    }
    pool
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weak_until_two_correct_in_a_row() {
        let mut s = QStat::default();
        s.push(true);
        assert!(!s.is_weak(), "never missed");
        s.push(false);
        assert!(s.is_weak());
        s.push(true);
        assert!(s.is_weak(), "one correct is not enough");
        s.push(false);
        s.push(true);
        assert!(s.is_weak());
        s.push(true);
        assert!(!s.is_weak(), "two in a row masters it");
        assert_eq!((s.seen, s.wrong), (6, 2));
    }

    #[test]
    fn pool_orders_and_skips_unanswered() {
        let e = crate::exam::parse(include_str!("../exams/practice-exam-1.json"), "t").unwrap();
        let exams = vec![e];
        let mut answers: Vec<Vec<char>> = exams[0].questions.iter().map(|q| q.answer.clone()).collect();
        answers[0] = vec!['D']; // Q1 wrong
        answers[1] = vec![]; // Q2 unanswered: ignored
        let attempt = |t: i64, answers: Vec<Vec<char>>| Attempt {
            exam_id: "exam1".into(),
            exam_title: String::new(),
            started_at: t,
            duration_secs: 0,
            time_limit_secs: 0,
            timed_out: false,
            report: crate::score::score(&exams[0], &answers),
            answers,
        };
        let mut second = answers.clone();
        second[2] = vec!['D']; // Q3 wrong in the second attempt only
        let attempts = vec![attempt(1, answers.clone()), attempt(2, second)];
        let practice =
            vec![PracticeRecord { exam_id: "exam1".into(), question: 1, correct: false, at: 3 }];
        let st = stats(&exams, &attempts, &practice);
        let pool = weak_pool(&exams, &st, None, None);
        let nums: Vec<usize> = pool.iter().map(|p| exams[0].questions[p.q].number).collect();
        // Q1 missed 3× (once in training) ranks above Q3 missed 1×; Q2 is not there.
        assert_eq!(nums, vec![1, 3]);
        assert_eq!(pick(pool, 1, 42).len(), 1);
    }
}

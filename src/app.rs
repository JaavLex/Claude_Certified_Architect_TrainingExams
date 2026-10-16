//! Application state and input handling.

use crate::exam::{EXAM_MINUTES, Exam};
use crate::score;
use crate::store::{self, Attempt};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::time::{Duration, Instant};

pub struct Session {
    pub exam: usize,
    pub current: usize,
    pub cursor: usize,
    pub answers: Vec<Vec<char>>,
    pub flagged: Vec<bool>,
    pub started: Instant,
    pub started_unix: i64,
    pub notice: Option<(String, Instant)>,
}

impl Session {
    pub fn total(&self) -> Duration {
        Duration::from_secs(EXAM_MINUTES * 60)
    }
    pub fn elapsed(&self) -> Duration {
        self.started.elapsed().min(self.total())
    }
    pub fn remaining(&self) -> Duration {
        self.total().saturating_sub(self.started.elapsed())
    }
    pub fn notice(&self) -> Option<&str> {
        self.notice
            .as_ref()
            .filter(|(_, t)| t.elapsed() < Duration::from_millis(2200))
            .map(|(s, _)| s.as_str())
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ReviewFilter {
    All,
    Incorrect,
    Correct,
}

impl ReviewFilter {
    pub fn label(self) -> &'static str {
        match self {
            ReviewFilter::All => "All",
            ReviewFilter::Incorrect => "Incorrect",
            ReviewFilter::Correct => "Correct",
        }
    }
    fn next(self) -> Self {
        match self {
            ReviewFilter::All => ReviewFilter::Incorrect,
            ReviewFilter::Incorrect => ReviewFilter::Correct,
            ReviewFilter::Correct => ReviewFilter::All,
        }
    }
}

pub enum Screen {
    Menu { cursor: usize },
    Intro { exam: usize },
    Exam(Session),
    Results { attempt: usize, shown: Instant, from_history: bool },
    Review { attempt: usize, filter: ReviewFilter, cursor: usize, scroll: u16, from_history: bool },
    History { cursor: usize, filter: Option<usize> },
}

pub enum Modal {
    ConfirmSubmit,
    ConfirmAbandon,
    ConfirmDelete,
    Goto(String),
}

pub struct App {
    pub exams: Vec<Exam>,
    pub attempts: Vec<Attempt>,
    pub screen: Screen,
    pub modal: Option<Modal>,
    pub quit: bool,
    pub save_error: Option<String>,
}

pub const MENU_ITEMS: usize = 4; // exam 1, exam 2, history, quit

impl App {
    pub fn new(exams: Vec<Exam>) -> Self {
        App {
            exams,
            attempts: store::load(),
            screen: Screen::Menu { cursor: 0 },
            modal: None,
            quit: false,
            save_error: None,
        }
    }

    pub fn exam_index(&self, id: &str) -> Option<usize> {
        self.exams.iter().position(|e| e.id == id)
    }

    /// Attempt indices shown in History (newest first), honouring the exam filter.
    pub fn history_rows(&self, filter: Option<usize>) -> Vec<usize> {
        let mut rows: Vec<usize> = (0..self.attempts.len())
            .filter(|&i| match filter {
                Some(e) => self.attempts[i].exam_id == self.exams[e].id,
                None => true,
            })
            .collect();
        rows.sort_by_key(|&i| std::cmp::Reverse(self.attempts[i].started_at));
        rows
    }

    /// The attempt on the same exam immediately before `idx`.
    pub fn previous_attempt(&self, idx: usize) -> Option<usize> {
        let a = &self.attempts[idx];
        (0..self.attempts.len())
            .filter(|&i| {
                let b = &self.attempts[i];
                b.exam_id == a.exam_id && b.started_at < a.started_at
            })
            .max_by_key(|&i| self.attempts[i].started_at)
    }

    /// Whether anything on screen is animating (drives a faster redraw rate).
    pub fn animating(&self) -> bool {
        match &self.screen {
            Screen::Exam(_) => true,
            Screen::Results { shown, .. } => shown.elapsed() < Duration::from_millis(1600),
            _ => false,
        }
    }

    pub fn tick(&mut self) {
        if let Screen::Exam(s) = &self.screen {
            if s.remaining().is_zero() {
                self.modal = None;
                self.submit(true);
            }
        }
    }

    fn submit(&mut self, timed_out: bool) {
        let Screen::Exam(s) = &self.screen else { return };
        let exam = &self.exams[s.exam];
        let report = score::score(exam, &s.answers);
        self.attempts.push(Attempt {
            exam_id: exam.id.to_string(),
            exam_title: exam.title.clone(),
            started_at: s.started_unix,
            duration_secs: s.elapsed().as_secs(),
            timed_out,
            answers: s.answers.clone(),
            report,
        });
        self.save_error = store::save(&self.attempts).err().map(|e| e.to_string());
        self.screen = Screen::Results {
            attempt: self.attempts.len() - 1,
            shown: Instant::now(),
            from_history: false,
        };
    }

    pub fn review_rows(&self, attempt: usize, filter: ReviewFilter) -> Vec<usize> {
        let a = &self.attempts[attempt];
        let Some(e) = self.exam_index(&a.exam_id) else { return vec![] };
        self.exams[e]
            .questions
            .iter()
            .enumerate()
            .filter(|(i, q)| {
                let ok = q.is_correct(a.answers.get(*i).map(Vec::as_slice).unwrap_or(&[]));
                match filter {
                    ReviewFilter::All => true,
                    ReviewFilter::Incorrect => !ok,
                    ReviewFilter::Correct => ok,
                }
            })
            .map(|(i, _)| i)
            .collect()
    }

    pub fn on_key(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            if matches!(self.screen, Screen::Exam(_)) {
                self.modal = Some(Modal::ConfirmAbandon);
            } else {
                self.quit = true;
            }
            return;
        }
        if self.modal.is_some() {
            self.on_modal_key(key);
            return;
        }
        match &mut self.screen {
            Screen::Menu { cursor } => match key.code {
                KeyCode::Up | KeyCode::Char('k') => *cursor = (*cursor + MENU_ITEMS - 1) % MENU_ITEMS,
                KeyCode::Down | KeyCode::Char('j') => *cursor = (*cursor + 1) % MENU_ITEMS,
                KeyCode::Char('1') => self.screen = Screen::Intro { exam: 0 },
                KeyCode::Char('2') => self.screen = Screen::Intro { exam: 1 },
                KeyCode::Char('h') => self.screen = Screen::History { cursor: 0, filter: None },
                KeyCode::Char('q') | KeyCode::Esc => self.quit = true,
                KeyCode::Enter | KeyCode::Char(' ') => match *cursor {
                    0 | 1 => self.screen = Screen::Intro { exam: *cursor },
                    2 => self.screen = Screen::History { cursor: 0, filter: None },
                    _ => self.quit = true,
                },
                _ => {}
            },
            Screen::Intro { exam } => match key.code {
                KeyCode::Enter => {
                    let n = self.exams[*exam].questions.len();
                    self.screen = Screen::Exam(Session {
                        exam: *exam,
                        current: 0,
                        cursor: 0,
                        answers: vec![Vec::new(); n],
                        flagged: vec![false; n],
                        started: Instant::now(),
                        started_unix: chrono::Local::now().timestamp(),
                        notice: None,
                    });
                }
                KeyCode::Esc | KeyCode::Char('q') => self.screen = Screen::Menu { cursor: *exam },
                _ => {}
            },
            Screen::Exam(_) => self.on_exam_key(key),
            Screen::Results { attempt, from_history, .. } => {
                let (attempt, from_history) = (*attempt, *from_history);
                match key.code {
                    KeyCode::Char('r') => {
                        self.screen = Screen::Review {
                            attempt,
                            filter: ReviewFilter::All,
                            cursor: 0,
                            scroll: 0,
                            from_history,
                        }
                    }
                    KeyCode::Char('w') | KeyCode::Char('i') => {
                        self.screen = Screen::Review {
                            attempt,
                            filter: ReviewFilter::Incorrect,
                            cursor: 0,
                            scroll: 0,
                            from_history,
                        }
                    }
                    KeyCode::Char('h') => {
                        let cursor = self.history_rows(None).iter().position(|&i| i == attempt).unwrap_or(0);
                        self.screen = Screen::History { cursor, filter: None }
                    }
                    KeyCode::Esc | KeyCode::Enter | KeyCode::Char('m') | KeyCode::Char('q') => {
                        if from_history {
                            let cursor =
                                self.history_rows(None).iter().position(|&i| i == attempt).unwrap_or(0);
                            self.screen = Screen::History { cursor, filter: None }
                        } else {
                            self.screen = Screen::Menu { cursor: 0 }
                        }
                    }
                    _ => {}
                }
            }
            Screen::Review { .. } => self.on_review_key(key),
            Screen::History { .. } => self.on_history_key(key),
        }
    }

    fn on_modal_key(&mut self, key: KeyEvent) {
        let Some(modal) = &mut self.modal else { return };
        match modal {
            Modal::Goto(buf) => match key.code {
                KeyCode::Char(c) if c.is_ascii_digit() && buf.len() < 2 => buf.push(c),
                KeyCode::Backspace => {
                    buf.pop();
                }
                KeyCode::Enter => {
                    let n: usize = buf.parse().unwrap_or(0);
                    if let Screen::Exam(s) = &mut self.screen {
                        if n >= 1 && n <= s.answers.len() {
                            s.current = n - 1;
                            s.cursor = 0;
                        }
                    }
                    self.modal = None;
                }
                KeyCode::Esc => self.modal = None,
                _ => {}
            },
            Modal::ConfirmSubmit => match key.code {
                KeyCode::Char('y') | KeyCode::Enter => {
                    self.modal = None;
                    self.submit(false);
                }
                _ => self.modal = None,
            },
            Modal::ConfirmAbandon => match key.code {
                KeyCode::Char('y') => {
                    self.modal = None;
                    self.screen = Screen::Menu { cursor: 0 };
                }
                _ => self.modal = None,
            },
            Modal::ConfirmDelete => {
                if key.code == KeyCode::Char('y') {
                    if let Screen::History { cursor, filter } = self.screen {
                        let rows = self.history_rows(filter);
                        if let Some(&idx) = rows.get(cursor) {
                            self.attempts.remove(idx);
                            self.save_error = store::save(&self.attempts).err().map(|e| e.to_string());
                            let len = self.history_rows(filter).len();
                            self.screen = Screen::History { cursor: cursor.min(len.saturating_sub(1)), filter };
                        }
                    }
                }
                self.modal = None;
            }
        }
    }

    fn on_exam_key(&mut self, key: KeyEvent) {
        let Screen::Exam(s) = &mut self.screen else { return };
        let q = &self.exams[s.exam].questions[s.current];
        let n_opts = q.options.len();
        let last = s.answers.len() - 1;

        let toggle = |s: &mut Session, idx: usize| {
            let Some(&(letter, _)) = q.options.get(idx) else { return };
            s.cursor = idx;
            let ans = &mut s.answers[s.current];
            if let Some(p) = ans.iter().position(|&c| c == letter) {
                ans.remove(p);
            } else if q.select == 1 {
                *ans = vec![letter];
            } else if ans.len() < q.select {
                ans.push(letter);
                ans.sort_unstable();
            } else {
                s.notice = Some((
                    format!("Select exactly {} — deselect one before choosing another", q.select),
                    Instant::now(),
                ));
            }
        };

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => s.cursor = (s.cursor + n_opts - 1) % n_opts,
            KeyCode::Down | KeyCode::Char('j') => s.cursor = (s.cursor + 1) % n_opts,
            KeyCode::Char(' ') | KeyCode::Enter => {
                let c = s.cursor;
                toggle(s, c)
            }
            KeyCode::Char(c @ 'a'..='d') => toggle(s, (c as u8 - b'a') as usize),
            KeyCode::Char(c @ 'A'..='D') => toggle(s, (c as u8 - b'A') as usize),
            KeyCode::Char(c @ '1'..='4') => toggle(s, (c as u8 - b'1') as usize),
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Char('n') | KeyCode::Tab => {
                if s.current < last {
                    s.current += 1;
                    s.cursor = 0;
                }
            }
            KeyCode::Left | KeyCode::Char('h') | KeyCode::Char('p') | KeyCode::BackTab => {
                if s.current > 0 {
                    s.current -= 1;
                    s.cursor = 0;
                }
            }
            KeyCode::Home => {
                s.current = 0;
                s.cursor = 0;
            }
            KeyCode::End => {
                s.current = last;
                s.cursor = 0;
            }
            KeyCode::Char('f') => s.flagged[s.current] = !s.flagged[s.current],
            KeyCode::Char('u') => {
                // Next incomplete question (wrapping).
                let exam = &self.exams[s.exam];
                let n = s.answers.len();
                if let Some(i) = (1..=n)
                    .map(|k| (s.current + k) % n)
                    .find(|&i| s.answers[i].len() != exam.questions[i].select)
                {
                    s.current = i;
                    s.cursor = 0;
                } else {
                    s.notice = Some(("All questions answered".into(), Instant::now()));
                }
            }
            KeyCode::Char('m') => {
                // Next flagged question (wrapping).
                let n = s.answers.len();
                if let Some(i) = (1..=n).map(|k| (s.current + k) % n).find(|&i| s.flagged[i]) {
                    s.current = i;
                    s.cursor = 0;
                } else {
                    s.notice = Some(("No flagged questions".into(), Instant::now()));
                }
            }
            KeyCode::Char('g') => self.modal = Some(Modal::Goto(String::new())),
            KeyCode::Char('s') => self.modal = Some(Modal::ConfirmSubmit),
            KeyCode::Esc | KeyCode::Char('q') => self.modal = Some(Modal::ConfirmAbandon),
            _ => {}
        }
    }

    fn on_review_key(&mut self, key: KeyEvent) {
        let Screen::Review { attempt, filter, from_history, .. } = &self.screen else { return };
        let (attempt, from_history) = (*attempt, *from_history);
        let len = self.review_rows(attempt, *filter).len();
        let Screen::Review { filter, cursor, scroll, .. } = &mut self.screen else { return };
        match key.code {
            KeyCode::Up | KeyCode::Char('k') | KeyCode::Left | KeyCode::Char('h') => {
                *cursor = cursor.saturating_sub(1);
                *scroll = 0;
            }
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Right | KeyCode::Char('l') => {
                if *cursor + 1 < len {
                    *cursor += 1;
                    *scroll = 0;
                }
            }
            KeyCode::Home => {
                *cursor = 0;
                *scroll = 0;
            }
            KeyCode::End => {
                *cursor = len.saturating_sub(1);
                *scroll = 0;
            }
            KeyCode::PageDown | KeyCode::Char('J') => *scroll = scroll.saturating_add(3),
            KeyCode::PageUp | KeyCode::Char('K') => *scroll = scroll.saturating_sub(3),
            KeyCode::Char('f') | KeyCode::Char('w') => {
                *filter = filter.next();
                *cursor = 0;
                *scroll = 0;
            }
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Backspace => {
                self.screen = Screen::Results {
                    attempt,
                    // Skip the count-up when returning.
                    shown: Instant::now() - Duration::from_secs(10),
                    from_history,
                };
            }
            _ => {}
        }
    }

    fn on_history_key(&mut self, key: KeyEvent) {
        let Screen::History { cursor, filter } = &self.screen else { return };
        let c = *cursor;
        let rows = self.history_rows(*filter);
        let n_exams = self.exams.len();
        let Screen::History { cursor, filter } = &mut self.screen else { return };
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => *cursor = cursor.saturating_sub(1),
            KeyCode::Down | KeyCode::Char('j') => {
                if *cursor + 1 < rows.len() {
                    *cursor += 1
                }
            }
            KeyCode::Tab | KeyCode::Char('f') => {
                *filter = match *filter {
                    None => Some(0),
                    Some(i) if i + 1 < n_exams => Some(i + 1),
                    Some(_) => None,
                };
                *cursor = 0;
            }
            KeyCode::Enter => {
                if let Some(&idx) = rows.get(c) {
                    self.screen = Screen::Results { attempt: idx, shown: Instant::now(), from_history: true };
                }
            }
            KeyCode::Char('r') => {
                if let Some(&idx) = rows.get(c) {
                    self.screen = Screen::Review {
                        attempt: idx,
                        filter: ReviewFilter::Incorrect,
                        cursor: 0,
                        scroll: 0,
                        from_history: true,
                    };
                }
            }
            KeyCode::Char('d') | KeyCode::Delete => {
                if rows.get(c).is_some() {
                    self.modal = Some(Modal::ConfirmDelete);
                }
            }
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('m') => self.screen = Screen::Menu { cursor: 2 },
            _ => {}
        }
    }
}

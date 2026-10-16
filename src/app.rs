//! Application state and input handling.

use crate::exam::{self, Exam, Question};
use crate::paths;
use crate::score;
use crate::store::{self, Attempt, PracticeRecord};
use crate::train::{self, PoolItem, QStat};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::path::PathBuf;
use std::time::{Duration, Instant};

pub struct Session {
    pub exam: usize,
    pub current: usize,
    pub cursor: usize,
    pub limit: Duration,
    pub answers: Vec<Vec<char>>,
    pub flagged: Vec<bool>,
    pub started: Instant,
    pub started_unix: i64,
    pub notice: Option<(String, Instant)>,
    /// Time spent on each question before the current visit.
    pub q_time: Vec<Duration>,
    /// When the current visit to `current` began.
    pub q_started: Instant,
}

impl Session {
    pub fn new(exam: usize, n: usize, limit: Duration) -> Self {
        let now = Instant::now();
        Session {
            exam,
            current: 0,
            cursor: 0,
            limit,
            answers: vec![Vec::new(); n],
            flagged: vec![false; n],
            started: now,
            started_unix: chrono::Local::now().timestamp(),
            notice: None,
            q_time: vec![Duration::ZERO; n],
            q_started: now,
        }
    }

    /// Move to question `i`, crediting the time spent on the current one.
    pub fn go_to(&mut self, i: usize) {
        let now = Instant::now();
        self.q_time[self.current] += now - self.q_started;
        self.q_started = now;
        self.current = i;
        self.cursor = 0;
    }

    /// Total time spent on question `i` so far, including the current visit.
    pub fn time_on(&self, i: usize) -> Duration {
        let live = if i == self.current { self.q_started.elapsed() } else { Duration::ZERO };
        self.q_time[i] + live
    }

    /// Target time per question: the time limit spread evenly over the items
    /// (2 minutes for 60 items in 120 minutes, the study plan's pace).
    pub fn pace_per_question(&self) -> Duration {
        self.limit / self.answers.len().max(1) as u32
    }

    /// Answered questions minus the number the pace says should be done by now.
    /// Negative means behind.
    pub fn pace_delta(&self, answered: usize) -> i64 {
        let expected = self.elapsed().as_secs_f64() / self.pace_per_question().as_secs_f64().max(1.0);
        answered as i64 - expected.floor() as i64
    }

    pub fn total(&self) -> Duration {
        self.limit
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

/// Select or deselect option `idx` of `q`, enforcing "select exactly N".
/// Returns a notice when the selection is already full.
fn toggle_answer(q: &Question, ans: &mut Vec<char>, idx: usize) -> Option<String> {
    let letter = q.options.get(idx)?.key;
    if let Some(p) = ans.iter().position(|&c| c == letter) {
        ans.remove(p);
    } else if q.select == 1 {
        *ans = vec![letter];
    } else if ans.len() < q.select {
        ans.push(letter);
        ans.sort_unstable();
    } else {
        return Some(format!("Select exactly {} — deselect one before choosing another", q.select));
    }
    None
}

/// Map an option-picking key to an option index: a–e by key, 1–9 by position.
fn option_index(q: &Question, code: KeyCode) -> Option<usize> {
    match code {
        KeyCode::Char(c @ ('a'..='e' | 'A'..='E')) => {
            let key = c.to_ascii_uppercase();
            q.options.iter().position(|o| o.key == key)
        }
        KeyCode::Char(c @ '1'..='9') => Some((c as u8 - b'1') as usize),
        _ => None,
    }
}

/// Setup choices for a training session.
pub struct TrainSetup {
    /// Highlighted row: 0 exam, 1 domain, 2 size, 3 start.
    pub field: usize,
    pub exam: Option<usize>,
    pub domain: Option<String>,
    /// Index into `train::SIZES`.
    pub size: usize,
}

pub struct TrainSession {
    pub items: Vec<PoolItem>,
    pub current: usize,
    pub cursor: usize,
    pub chosen: Vec<char>,
    /// Whether the current answer has been checked.
    pub revealed: bool,
    /// Result of each checked item.
    pub results: Vec<bool>,
    /// Stat of the current item after checking it.
    pub after: Option<QStat>,
    pub scroll: u16,
    pub notice: Option<(String, Instant)>,
}

impl TrainSession {
    pub fn notice(&self) -> Option<&str> {
        self.notice
            .as_ref()
            .filter(|(_, t)| t.elapsed() < Duration::from_millis(2200))
            .map(|(s, _)| s.as_str())
    }
}

pub struct TrainSummary {
    pub items: Vec<PoolItem>,
    pub results: Vec<bool>,
    /// Items no longer weak after the session.
    pub mastered: usize,
    /// Weak questions left overall.
    pub remaining: usize,
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
    TrainSetup(TrainSetup),
    Train(TrainSession),
    TrainDone(TrainSummary),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MenuItem {
    Exam(usize),
    Train,
    AddExam,
    SaveExample,
    History,
    Quit,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PathAction {
    AddExam,
    SaveExample,
}

/// A one-line path box with a cursor, Tab completion and an inline error.
pub struct PathInput {
    pub action: PathAction,
    pub text: String,
    /// Cursor position, in characters.
    pub cursor: usize,
    pub error: Option<String>,
    /// Names offered by the last Tab press, when there was more than one.
    pub matches: Vec<String>,
}

impl PathInput {
    fn new(action: PathAction, text: String) -> Self {
        let cursor = text.chars().count();
        PathInput { action, text, cursor, error: None, matches: Vec::new() }
    }

    fn byte_at(&self, char_idx: usize) -> usize {
        self.text.char_indices().nth(char_idx).map(|(i, _)| i).unwrap_or(self.text.len())
    }

    pub fn insert(&mut self, s: &str) {
        // Pasted text may end with a newline; paths never contain one.
        let s: String = s.chars().filter(|c| *c != '\n' && *c != '\r').collect();
        let at = self.byte_at(self.cursor);
        self.text.insert_str(at, &s);
        self.cursor += s.chars().count();
        self.error = None;
        self.matches.clear();
    }
}

pub enum Modal {
    Path(PathInput),
    Info { title: String, ok: bool, lines: Vec<String> },
    ConfirmSubmit,
    ConfirmAbandon,
    ConfirmDelete,
    ConfirmEndTraining,
    Goto(String),
    LoadErrors,
}

pub struct App {
    pub exams: Vec<Exam>,
    pub attempts: Vec<Attempt>,
    pub practice: Vec<PracticeRecord>,
    pub screen: Screen,
    pub modal: Option<Modal>,
    pub quit: bool,
    pub save_error: Option<String>,
    /// Exam files that failed to load, with reasons.
    pub load_errors: Vec<String>,
    /// Set when the exam folder could not be created at start.
    pub exam_dir_error: Option<String>,
    /// Extra exam paths given on the command line (kept for reloading).
    extra_paths: Vec<PathBuf>,
}

impl App {
    pub fn new(extra_paths: Vec<PathBuf>) -> Self {
        let exam_dir_error = exam::ensure_user_exam_dir().err();
        let loaded = exam::load_all(&extra_paths);
        let mut app = App {
            exams: loaded.exams,
            attempts: store::load(),
            practice: store::load_practice(),
            screen: Screen::Menu { cursor: 0 },
            modal: None,
            quit: false,
            save_error: None,
            load_errors: loaded.errors,
            exam_dir_error,
            extra_paths,
        };
        app.regrade();
        app
    }

    /// Re-score saved attempts from their answers with the current exams, so
    /// history always reflects the current grading rules and answer keys
    /// (e.g. attempts saved before domain weighting was introduced).
    fn regrade(&mut self) {
        let mut changed = false;
        for a in &mut self.attempts {
            let Some(e) = self.exams.iter().find(|e| e.id == a.exam_id) else { continue };
            if e.questions.len() != a.answers.len() {
                continue; // The exam file changed shape; keep the score it was given.
            }
            let fresh = score::score(e, &a.answers);
            if fresh.scaled != a.report.scaled || fresh.correct != a.report.correct || fresh.weights != a.report.weights {
                a.report = fresh;
                changed = true;
            }
        }
        if changed {
            self.save_error = store::save(&self.attempts).err().map(|e| e.to_string());
        }
    }

    fn open_exam_folder(&mut self) {
        let dir = exam::user_exam_dir();
        if let Err(e) = paths::open_in_file_manager(&dir) {
            self.modal = Some(Modal::Info {
                title: "Exam folder".into(),
                ok: false,
                lines: vec![
                    format!("✗ {e}"),
                    String::new(),
                    "You can open it yourself; it is at:".into(),
                    dir.display().to_string(),
                ],
            });
        }
    }

    /// Menu rows: one per exam, then the tools.
    pub fn menu_items(&self) -> Vec<MenuItem> {
        let mut v: Vec<MenuItem> = (0..self.exams.len()).map(MenuItem::Exam).collect();
        v.extend([MenuItem::Train, MenuItem::AddExam, MenuItem::SaveExample, MenuItem::History, MenuItem::Quit]);
        v
    }

    pub fn menu_pos(&self, item: MenuItem) -> usize {
        self.menu_items().iter().position(|&i| i == item).unwrap_or(0)
    }

    fn open_menu_item(&mut self, item: MenuItem) {
        match item {
            MenuItem::Exam(i) => self.screen = Screen::Intro { exam: i },
            MenuItem::Train => {
                self.screen = Screen::TrainSetup(TrainSetup { field: 3, exam: None, domain: None, size: 1 })
            }
            MenuItem::AddExam => self.modal = Some(Modal::Path(PathInput::new(PathAction::AddExam, String::new()))),
            MenuItem::SaveExample => {
                let default = exam::default_example_path().display().to_string();
                self.modal = Some(Modal::Path(PathInput::new(PathAction::SaveExample, default)))
            }
            MenuItem::History => self.screen = Screen::History { cursor: 0, filter: None },
            MenuItem::Quit => self.quit = true,
        }
    }

    /// Text pasted into the terminal (bracketed paste).
    pub fn on_paste(&mut self, text: &str) {
        if let Some(Modal::Path(p)) = &mut self.modal {
            p.insert(text);
        }
    }

    fn submit_path(&mut self) {
        let Some(Modal::Path(input)) = &mut self.modal else { return };
        match input.action {
            PathAction::AddExam => {
                let src = match paths::resolve_existing(&input.text) {
                    Ok(p) => p,
                    Err(e) => return input.error = Some(e),
                };
                let builtin: Vec<String> =
                    self.exams.iter().filter(|e| e.is_builtin()).map(|e| e.id.clone()).collect();
                let results = exam::import_path(&src, &builtin);
                if results.is_empty() {
                    input.error = Some(format!("No .json files in {}", src.display()));
                    return;
                }
                if let [(_, Err(e))] = results.as_slice() {
                    // A single file that failed: keep the box open to fix the path.
                    input.error = Some(format!("Could not add this exam: {e}"));
                    return;
                }
                self.reload();
                let mut lines = Vec::new();
                let mut first_id = None;
                let mut all_ok = true;
                for (file, r) in &results {
                    match r {
                        Ok(im) => {
                            first_id.get_or_insert(im.id.clone());
                            let verb = if im.updated { "Updated" } else { "Added" };
                            lines.push(format!("✓ {verb} “{}”", im.title));
                        }
                        Err(e) => {
                            all_ok = false;
                            let name = file.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
                            lines.push(format!("✗ {name}: {e}"));
                        }
                    }
                }
                lines.push(String::new());
                match results.as_slice() {
                    [(_, Ok(im))] => lines.push(format!("Copied to {}", im.dest.display())),
                    _ => lines.push(format!("Copied to {}", exam::user_exam_dir().display())),
                }
                lines.push("It will be there every time you start the app.".into());
                lines.push("Press o on the menu to open this folder.".into());
                let cursor = first_id
                    .and_then(|id| self.exam_index(&id))
                    .map(|i| self.menu_pos(MenuItem::Exam(i)))
                    .unwrap_or(0);
                self.screen = Screen::Menu { cursor };
                self.modal = Some(Modal::Info { title: "Add exam".into(), ok: all_ok, lines });
            }
            PathAction::SaveExample => {
                let target = match paths::resolve_new(&input.text) {
                    Ok(p) => p,
                    Err(e) => return input.error = Some(e),
                };
                match exam::save_example(&target) {
                    Ok(dest) => {
                        self.modal = Some(Modal::Info {
                            title: "Example saved".into(),
                            ok: true,
                            lines: vec![
                                "The example exam was saved to:".into(),
                                dest.display().to_string(),
                                String::new(),
                                "Open it in any text editor and replace the questions with".into(),
                                "your own. Then choose “Add exam from file” to load it.".into(),
                            ],
                        })
                    }
                    Err(e) => input.error = Some(format!("Could not save: {e}")),
                }
            }
        }
    }

    pub fn question_stats(&self) -> std::collections::HashMap<train::Key, QStat> {
        train::stats(&self.exams, &self.attempts, &self.practice)
    }

    fn start_training(&mut self, setup: &TrainSetup) {
        let st = self.question_stats();
        let pool = train::weak_pool(&self.exams, &st, setup.exam, setup.domain.as_deref());
        if pool.is_empty() {
            return;
        }
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(1);
        let items = train::pick(pool, train::SIZES[setup.size], seed);
        self.screen = Screen::Train(TrainSession {
            items,
            current: 0,
            cursor: 0,
            chosen: Vec::new(),
            revealed: false,
            results: Vec::new(),
            after: None,
            scroll: 0,
            notice: None,
        });
    }

    fn finish_training(&mut self) {
        let Screen::Train(t) = std::mem::replace(&mut self.screen, Screen::Menu { cursor: 0 }) else { return };
        if t.results.is_empty() {
            self.screen = Screen::Menu { cursor: self.menu_pos(MenuItem::Train) };
            return;
        }
        let st = self.question_stats();
        let items: Vec<PoolItem> = t.items.into_iter().take(t.results.len()).collect();
        let mastered = items
            .iter()
            .filter(|p| {
                let e = &self.exams[p.exam];
                !st.get(&(e.id.clone(), e.questions[p.q].number)).is_some_and(QStat::is_weak)
            })
            .count();
        let remaining = st.values().filter(|s| s.is_weak()).count();
        self.screen = Screen::TrainDone(TrainSummary { items, results: t.results, mastered, remaining });
    }

    /// Re-scan the exam folder and command-line paths.
    fn reload(&mut self) {
        let loaded = exam::load_all(&self.extra_paths);
        self.exams = loaded.exams;
        self.load_errors = loaded.errors;
        self.regrade();
        if !self.load_errors.is_empty() {
            self.modal = Some(Modal::LoadErrors);
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

    /// 1-based position of attempt `idx` among attempts on the same exam.
    pub fn attempt_number(&self, idx: usize) -> usize {
        let a = &self.attempts[idx];
        self.attempts.iter().filter(|b| b.exam_id == a.exam_id && b.started_at <= a.started_at).count()
    }

    /// Readiness checklist (study plan §6 and the pre-exam checklist): the
    /// first timed attempt on each exam, which is what the targets refer to.
    /// Returns (exam index, first-attempt percent) for every exam.
    pub fn first_attempts(&self) -> Vec<(usize, Option<f64>)> {
        (0..self.exams.len())
            .map(|ei| {
                let first = self
                    .attempts
                    .iter()
                    .filter(|a| a.exam_id == self.exams[ei].id)
                    .min_by_key(|a| a.started_at)
                    .map(|a| a.report.score_pct());
                (ei, first)
            })
            .collect()
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
            time_limit_secs: s.limit.as_secs(),
            timed_out,
            answers: s.answers.clone(),
            question_secs: (0..s.answers.len()).map(|i| s.time_on(i).as_secs()).collect(),
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
        let items = self.menu_items();
        let pos_train = self.menu_pos(MenuItem::Train);
        let n_exams = self.exams.len();
        match &mut self.screen {
            Screen::Menu { cursor } => {
                let len = items.len();
                let cur = *cursor;
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') => *cursor = (cur + len - 1) % len,
                    KeyCode::Down | KeyCode::Char('j') => *cursor = (cur + 1) % len,
                    KeyCode::Char(c @ '1'..='9') => {
                        let i = (c as u8 - b'1') as usize;
                        if i < n_exams {
                            self.open_menu_item(MenuItem::Exam(i))
                        }
                    }
                    KeyCode::Char('t') => self.open_menu_item(MenuItem::Train),
                    KeyCode::Char('a') => self.open_menu_item(MenuItem::AddExam),
                    KeyCode::Char('x') => self.open_menu_item(MenuItem::SaveExample),
                    KeyCode::Char('o') => self.open_exam_folder(),
                    KeyCode::Char('h') => self.open_menu_item(MenuItem::History),
                    KeyCode::Char('r') => {
                        self.reload();
                        self.screen = Screen::Menu { cursor: 0 };
                    }
                    KeyCode::Char('e') if !self.load_errors.is_empty() => self.modal = Some(Modal::LoadErrors),
                    KeyCode::Char('q') | KeyCode::Esc => self.quit = true,
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        if let Some(&item) = items.get(cur) {
                            self.open_menu_item(item)
                        }
                    }
                    _ => {}
                }
            }
            Screen::Intro { exam } => match key.code {
                KeyCode::Enter => {
                    let n = self.exams[*exam].questions.len();
                    let limit = Duration::from_secs(self.exams[*exam].time_limit_secs());
                    self.screen = Screen::Exam(Session::new(*exam, n, limit));
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
            Screen::TrainSetup(_) => self.on_train_setup_key(key),
            Screen::Train(_) => self.on_train_key(key),
            Screen::TrainDone(_) => match key.code {
                KeyCode::Char('t') => {
                    self.screen = Screen::TrainSetup(TrainSetup { field: 3, exam: None, domain: None, size: 1 })
                }
                KeyCode::Enter | KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('m') => {
                    self.screen = Screen::Menu { cursor: pos_train }
                }
                _ => {}
            },
        }
    }

    fn on_modal_key(&mut self, key: KeyEvent) {
        if matches!(self.modal, Some(Modal::Path(_))) {
            return self.on_path_key(key);
        }
        let Some(modal) = &mut self.modal else { return };
        match modal {
            Modal::Path(_) => {}
            Modal::Info { .. } => self.modal = None,
            Modal::Goto(buf) => match key.code {
                KeyCode::Char(c) if c.is_ascii_digit() && buf.len() < 3 => buf.push(c),
                KeyCode::Backspace => {
                    buf.pop();
                }
                KeyCode::Enter => {
                    let n: usize = buf.parse().unwrap_or(0);
                    if let Screen::Exam(s) = &mut self.screen {
                        if n >= 1 && n <= s.answers.len() {
                            s.go_to(n - 1);
                        }
                    }
                    self.modal = None;
                }
                KeyCode::Esc => self.modal = None,
                _ => {}
            },
            Modal::LoadErrors => self.modal = None,
            Modal::ConfirmSubmit => match key.code {
                KeyCode::Char('y') | KeyCode::Enter => {
                    self.modal = None;
                    self.submit(false);
                }
                _ => self.modal = None,
            },
            Modal::ConfirmEndTraining => {
                self.modal = None;
                if key.code == KeyCode::Char('y') {
                    self.finish_training();
                }
            }
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
            if idx >= n_opts {
                return;
            }
            s.cursor = idx;
            if let Some(msg) = toggle_answer(q, &mut s.answers[s.current], idx) {
                s.notice = Some((msg, Instant::now()));
            }
        };

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => s.cursor = (s.cursor + n_opts - 1) % n_opts,
            KeyCode::Down | KeyCode::Char('j') => s.cursor = (s.cursor + 1) % n_opts,
            KeyCode::Char(' ') | KeyCode::Enter => {
                let c = s.cursor;
                toggle(s, c)
            }
            code @ KeyCode::Char('a'..='e' | 'A'..='E' | '1'..='9') => {
                if let Some(i) = option_index(q, code) {
                    toggle(s, i)
                }
            }
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Char('n') | KeyCode::Tab => {
                if s.current < last {
                    s.go_to(s.current + 1);
                }
            }
            KeyCode::Left | KeyCode::Char('h') | KeyCode::Char('p') | KeyCode::BackTab => {
                if s.current > 0 {
                    s.go_to(s.current - 1);
                }
            }
            KeyCode::Home => {
                s.go_to(0);
            }
            KeyCode::End => {
                s.go_to(last);
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
                    s.go_to(i);
                } else {
                    s.notice = Some(("All questions answered".into(), Instant::now()));
                }
            }
            KeyCode::Char('m') => {
                // Next flagged question (wrapping).
                let n = s.answers.len();
                if let Some(i) = (1..=n).map(|k| (s.current + k) % n).find(|&i| s.flagged[i]) {
                    s.go_to(i);
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
        let pos_history = self.menu_pos(MenuItem::History);
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
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('m') => {
                self.screen = Screen::Menu { cursor: pos_history }
            }
            _ => {}
        }
    }

    fn on_train_setup_key(&mut self, key: KeyEvent) {
        let n_exams = self.exams.len();
        let pos_train = self.menu_pos(MenuItem::Train);
        let st = self.question_stats();
        let Screen::TrainSetup(t) = &mut self.screen else { return };
        let domains: Vec<String> =
            train::weak_domains(&self.exams, &st, t.exam).into_iter().map(|(d, _)| d).collect();
        let step = |forward: bool, t: &mut TrainSetup| match t.field {
            0 => {
                // None → 0 → 1 → … → None
                t.exam = match (t.exam, forward) {
                    (None, true) => (n_exams > 0).then_some(0),
                    (None, false) => n_exams.checked_sub(1),
                    (Some(i), true) => (i + 1 < n_exams).then_some(i + 1),
                    (Some(i), false) => i.checked_sub(1),
                };
                t.domain = None;
            }
            1 => {
                let pos = t.domain.as_ref().and_then(|d| domains.iter().position(|x| x == d));
                t.domain = match (pos, forward) {
                    (None, true) => domains.first().cloned(),
                    (None, false) => domains.last().cloned(),
                    (Some(i), true) => domains.get(i + 1).cloned(),
                    (Some(i), false) => i.checked_sub(1).and_then(|j| domains.get(j).cloned()),
                };
            }
            2 => {
                let n = train::SIZES.len();
                t.size = if forward { (t.size + 1) % n } else { (t.size + n - 1) % n };
            }
            _ => {}
        };
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => t.field = (t.field + 3) % 4,
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Tab => t.field = (t.field + 1) % 4,
            KeyCode::Left | KeyCode::Char('h') => step(false, t),
            KeyCode::Right | KeyCode::Char('l') => step(true, t),
            KeyCode::Enter | KeyCode::Char(' ') => {
                if t.field == 3 || key.code == KeyCode::Enter {
                    let setup = TrainSetup { field: t.field, exam: t.exam, domain: t.domain.clone(), size: t.size };
                    self.start_training(&setup);
                } else {
                    step(true, t);
                }
            }
            KeyCode::Esc | KeyCode::Char('q') => self.screen = Screen::Menu { cursor: pos_train },
            _ => {}
        }
    }

    fn on_path_key(&mut self, key: KeyEvent) {
        let Some(Modal::Path(p)) = &mut self.modal else { return };
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let len = p.text.chars().count();
        match key.code {
            KeyCode::Enter => return self.submit_path(),
            KeyCode::Esc => {
                self.modal = None;
                return;
            }
            KeyCode::Tab => {
                let (text, matches) = paths::complete(&p.text);
                p.cursor = text.chars().count();
                p.text = text;
                p.matches = matches;
                p.error = None;
                return;
            }
            KeyCode::Char('u') if ctrl => {
                p.text.clear();
                p.cursor = 0;
            }
            KeyCode::Char('a') if ctrl => p.cursor = 0,
            KeyCode::Char('e') if ctrl => p.cursor = len,
            KeyCode::Char(c) if !ctrl => p.insert(&c.to_string()),
            KeyCode::Backspace if p.cursor > 0 => {
                let at = p.byte_at(p.cursor - 1);
                p.text.remove(at);
                p.cursor -= 1;
            }
            KeyCode::Delete if p.cursor < len => {
                let at = p.byte_at(p.cursor);
                p.text.remove(at);
            }
            KeyCode::Left => p.cursor = p.cursor.saturating_sub(1),
            KeyCode::Right => p.cursor = (p.cursor + 1).min(len),
            KeyCode::Home => p.cursor = 0,
            KeyCode::End => p.cursor = len,
            _ => return,
        }
        p.error = None;
        p.matches.clear();
    }

    fn on_train_key(&mut self, key: KeyEvent) {
        let Screen::Train(t) = &mut self.screen else { return };
        let item = t.items[t.current].clone();
        let e = &self.exams[item.exam];
        let q = &e.questions[item.q];
        let n_opts = q.options.len();

        if t.revealed {
            match key.code {
                KeyCode::Enter | KeyCode::Right | KeyCode::Char(' ') | KeyCode::Char('n') | KeyCode::Char('l') => {
                    if t.current + 1 >= t.items.len() {
                        self.finish_training();
                    } else {
                        t.current += 1;
                        t.cursor = 0;
                        t.chosen.clear();
                        t.revealed = false;
                        t.after = None;
                        t.scroll = 0;
                    }
                }
                KeyCode::PageDown | KeyCode::Char('J') | KeyCode::Down | KeyCode::Char('j') => {
                    t.scroll = t.scroll.saturating_add(2)
                }
                KeyCode::PageUp | KeyCode::Char('K') | KeyCode::Up | KeyCode::Char('k') => {
                    t.scroll = t.scroll.saturating_sub(2)
                }
                KeyCode::Esc | KeyCode::Char('q') => self.modal = Some(Modal::ConfirmEndTraining),
                _ => {}
            }
            return;
        }

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => t.cursor = (t.cursor + n_opts - 1) % n_opts,
            KeyCode::Down | KeyCode::Char('j') => t.cursor = (t.cursor + 1) % n_opts,
            KeyCode::Char(' ') => {
                if let Some(msg) = toggle_answer(q, &mut t.chosen, t.cursor) {
                    t.notice = Some((msg, Instant::now()));
                }
            }
            code @ KeyCode::Char('a'..='e' | 'A'..='E' | '1'..='9') => {
                if let Some(i) = option_index(q, code).filter(|&i| i < n_opts) {
                    t.cursor = i;
                    if let Some(msg) = toggle_answer(q, &mut t.chosen, i) {
                        t.notice = Some((msg, Instant::now()));
                    }
                }
            }
            KeyCode::Enter => {
                if t.chosen.len() != q.select {
                    // For single-answer questions Enter picks the highlighted option.
                    if q.select == 1 {
                        t.chosen = vec![q.options[t.cursor].key];
                    } else {
                        t.notice =
                            Some((format!("Select {} options, then press Enter to check", q.select), Instant::now()));
                        return;
                    }
                }
                let ok = q.is_correct(&t.chosen);
                self.practice.push(PracticeRecord {
                    exam_id: e.id.clone(),
                    question: q.number,
                    correct: ok,
                    at: chrono::Local::now().timestamp(),
                });
                self.save_error = store::save_practice(&self.practice).err().map(|e| e.to_string());
                let after = train::stats(&self.exams, &self.attempts, &self.practice)
                    .get(&(e.id.clone(), q.number))
                    .copied();
                let Screen::Train(t) = &mut self.screen else { return };
                t.results.push(ok);
                t.revealed = true;
                t.after = after;
            }
            KeyCode::Esc | KeyCode::Char('q') => self.modal = Some(Modal::ConfirmEndTraining),
            _ => {}
        }
    }
}

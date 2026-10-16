//! Rendering. Only the terminal's own 16-colour palette (named ANSI colours)
//! and text modifiers are used, so the app follows the user's terminal theme.

use crate::app::{App, MENU_ITEMS, Modal, ReviewFilter, Screen, Session};
use crate::exam::{self, EXAM_ITEMS, EXAM_MINUTES, Exam, Question};
use crate::score::{self, CUT_SCORE, SCALE_MAX, SCALE_MIN};
use crate::store::Attempt;
use crate::text::{fmt_hms, md_spans, wrap};
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{
    Bar, BarChart, BarGroup, Block, BorderType, Borders, Cell, Clear, Gauge, Padding, Paragraph, Row,
    Table, TableState, Wrap,
};
use std::time::Duration;

const ACCENT: Color = Color::Cyan;
const GOOD: Color = Color::Green;
const BAD: Color = Color::Red;
const WARN: Color = Color::Yellow;
const FLAG: Color = Color::Magenta;
const DIM: Color = Color::DarkGray;

fn panel(title: impl Into<String>) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(DIM))
        .title(Line::from(format!(" {} ", title.into())).fg(ACCENT).bold())
        .padding(Padding::horizontal(1))
}

fn key_hint(pairs: &[(&str, &str)]) -> Line<'static> {
    let mut spans = Vec::new();
    for (i, (k, v)) in pairs.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled("  ·  ", Style::new().fg(DIM)));
        }
        spans.push(Span::styled(k.to_string(), Style::new().fg(ACCENT).bold()));
        spans.push(Span::styled(format!(" {v}"), Style::new().fg(DIM)));
    }
    Line::from(spans).alignment(Alignment::Center)
}

fn centered(area: Rect, w: u16, h: u16) -> Rect {
    let w = w.min(area.width);
    let h = h.min(area.height);
    Rect { x: area.x + (area.width - w) / 2, y: area.y + (area.height - h) / 2, width: w, height: h }
}

/// Ease-out progress 0→1 over `dur`.
fn ease(elapsed: Duration, dur: Duration) -> f64 {
    let t = (elapsed.as_secs_f64() / dur.as_secs_f64()).clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(3)
}

fn short_title(e: &Exam) -> String {
    e.title.split(" — ").next().unwrap_or(&e.title).to_string()
}

fn fmt_date(ts: i64) -> String {
    chrono::DateTime::from_timestamp(ts, 0)
        .map(|d| d.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M").to_string())
        .unwrap_or_default()
}

pub fn draw(f: &mut Frame, app: &App) {
    let area = f.area();
    match &app.screen {
        Screen::Menu { cursor } => draw_menu(f, area, app, *cursor),
        Screen::Intro { exam } => draw_intro(f, area, app, *exam),
        Screen::Exam(s) => draw_exam(f, area, app, s),
        Screen::Results { attempt, shown, .. } => draw_results(f, area, app, *attempt, shown.elapsed()),
        Screen::Review { attempt, filter, cursor, scroll, .. } => {
            draw_review(f, area, app, *attempt, *filter, *cursor, *scroll)
        }
        Screen::History { cursor, filter } => draw_history(f, area, app, *cursor, *filter),
    }
    if let Some(m) = &app.modal {
        draw_modal(f, area, app, m);
    }
}

// ───────────────────────────── Menu ─────────────────────────────

const LOGO: [&str; 3] = [
    "╭─╮╭─╮╭─╮╭─╮   ╭─╴",
    "│  │  ├─┤├┬╯╶─╴├─╴",
    "╰─╯╰─╯╵ ╵╵╰╴   ╵  ",
];

fn draw_menu(f: &mut Frame, area: Rect, app: &App, cursor: usize) {
    let [_, logo, sub, _, body, _, foot] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(3),
        Constraint::Length(2),
        Constraint::Length(1),
        Constraint::Length(15),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(area);

    let logo_lines: Vec<Line> = LOGO.iter().map(|l| Line::from(*l).fg(ACCENT).bold()).collect();
    f.render_widget(Paragraph::new(logo_lines).alignment(Alignment::Center), logo);
    f.render_widget(
        Paragraph::new(vec![
            Line::from("Claude Certified Architect — Foundations").bold(),
            Line::from("Practice exam simulator").fg(DIM),
        ])
        .alignment(Alignment::Center),
        sub,
    );

    let body = centered(body, 92, 15);
    let [left, right] = Layout::horizontal([Constraint::Length(34), Constraint::Fill(1)]).spacing(2).areas(body);

    let labels = [
        format!("{} ", short_title(&app.exams[0])),
        format!("{} ", short_title(&app.exams[1])),
        "Attempt history ".to_string(),
        "Quit ".to_string(),
    ];
    let keys = ["1", "2", "h", "q"];
    let mut lines = vec![Line::raw("")];
    for i in 0..MENU_ITEMS {
        let sel = i == cursor;
        let line = if sel {
            Line::from(vec![
                Span::styled(" ▌ ", Style::new().fg(ACCENT)),
                Span::styled(labels[i].clone(), Style::new().fg(ACCENT).bold()),
                Span::styled(format!(" {}", keys[i]), Style::new().fg(DIM)),
            ])
        } else {
            Line::from(vec![
                Span::raw("   "),
                Span::raw(labels[i].clone()),
                Span::styled(format!(" {}", keys[i]), Style::new().fg(DIM)),
            ])
        };
        lines.push(line);
        if i == 1 {
            lines.push(Line::raw(""));
        }
    }
    f.render_widget(Paragraph::new(lines).block(panel("Menu")), left);

    // Info panel for the highlighted item.
    let info: Vec<Line> = if cursor < 2 {
        let e = &app.exams[cursor];
        let mine: Vec<&Attempt> = app.attempts.iter().filter(|a| a.exam_id == e.id).collect();
        let mut scen: Vec<String> = Vec::new();
        for q in &e.questions {
            if scen.last() != Some(&q.scenario) {
                scen.push(q.scenario.clone());
            }
        }
        let w = right.width.saturating_sub(4) as usize;
        let mut v = vec![
            Line::from(short_title(e)).bold(),
            Line::from(format!("{EXAM_ITEMS} items · {EXAM_MINUTES} minutes · pass ≥ {CUT_SCORE} scaled")).fg(DIM),
            Line::raw(""),
        ];
        if mine.is_empty() {
            v.push(Line::from("No attempts yet").fg(DIM).italic());
        } else {
            let best = mine.iter().map(|a| a.report.scaled).max().unwrap_or(0);
            let last = mine.iter().max_by_key(|a| a.started_at).unwrap();
            v.push(Line::from(vec![
                Span::styled("Attempts ", Style::new().fg(DIM)),
                Span::raw(format!("{}", mine.len())),
                Span::styled("   Best ", Style::new().fg(DIM)),
                Span::styled(format!("{best}"), Style::new().fg(if best >= CUT_SCORE { GOOD } else { BAD }).bold()),
                Span::styled("   Last ", Style::new().fg(DIM)),
                Span::styled(
                    format!("{}", last.report.scaled),
                    Style::new().fg(if last.report.passed { GOOD } else { BAD }),
                ),
            ]));
        }
        v.push(Line::raw(""));
        for s in scen.iter().take(4) {
            v.extend(wrap(vec![Span::styled("• ", Style::new().fg(ACCENT))], vec![Span::raw(s.clone())], w));
        }
        v
    } else if cursor == 2 {
        let n = app.attempts.len();
        let passed = app.attempts.iter().filter(|a| a.report.passed).count();
        vec![
            Line::from("Review and compare previous attempts").bold(),
            Line::raw(""),
            Line::from(format!("{n} attempt(s) recorded, {passed} passed")),
            Line::raw(""),
            Line::from(crate::store::path().display().to_string()).fg(DIM),
        ]
    } else {
        vec![Line::from("Exit the simulator").fg(DIM)]
    };
    f.render_widget(Paragraph::new(info).wrap(Wrap { trim: false }).block(panel("Details")), right);

    f.render_widget(key_hint(&[("↑↓", "move"), ("enter", "select"), ("q", "quit")]), foot);
}

// ───────────────────────────── Intro ─────────────────────────────

fn draw_intro(f: &mut Frame, area: Rect, app: &App, exam: usize) {
    let e = &app.exams[exam];
    let box_ = centered(area, 76, 18);
    let rule = |k: &str, v: &str| {
        Line::from(vec![Span::styled(format!("{k:<14}"), Style::new().fg(DIM)), Span::raw(v.to_string())])
    };
    let lines = vec![
        Line::from(e.title.clone()).bold().fg(ACCENT),
        Line::raw(""),
        rule("Items", &format!("{EXAM_ITEMS} (multiple-choice and multiple-response)")),
        rule("Time limit", &format!("{EXAM_MINUTES} minutes — auto-submits when time runs out")),
        rule("Passing", &format!("Scaled score ≥ {CUT_SCORE} on {SCALE_MIN}–{SCALE_MAX}")),
        rule("Multi-select", "Select EXACTLY the stated number; no partial credit"),
        rule("Unanswered", "Scored as incorrect — no penalty for guessing"),
        rule("Report", "Pass/fail, scaled score, percent-correct by domain"),
        Line::raw(""),
        Line::from("Rules: no notes, no docs. You can move freely between questions,").fg(DIM),
        Line::from("flag items for later, and change answers until you submit.").fg(DIM),
        Line::raw(""),
        Line::raw(""),
        Line::from(vec![
            Span::styled(" ENTER ", Style::new().fg(Color::Black).bg(ACCENT).bold()),
            Span::raw("  start the timer    "),
            Span::styled(" ESC ", Style::new().fg(Color::Black).bg(DIM).bold()),
            Span::raw("  back"),
        ])
        .alignment(Alignment::Center),
    ];
    f.render_widget(Paragraph::new(lines).block(panel("Before you begin").padding(Padding::uniform(1))), box_);
}

// ───────────────────────────── Exam ─────────────────────────────

fn is_complete(q: &Question, a: &[char]) -> bool {
    a.len() == q.select
}

fn draw_exam(f: &mut Frame, area: Rect, app: &App, s: &Session) {
    let e = &app.exams[s.exam];
    let q = &e.questions[s.current];
    let answered = e.questions.iter().zip(&s.answers).filter(|(q, a)| is_complete(q, a)).count();
    let flagged = s.flagged.iter().filter(|&&x| x).count();

    let [header, timer, body, foot] =
        Layout::vertical([Constraint::Length(1), Constraint::Length(3), Constraint::Fill(1), Constraint::Length(1)])
            .areas(area);

    // Header
    let head = Line::from(vec![
        Span::styled(format!(" {} ", short_title(e)), Style::new().fg(ACCENT).bold()),
        Span::styled("│ ", Style::new().fg(DIM)),
        Span::styled(format!("Question {}/{}", s.current + 1, e.questions.len()), Style::new().bold()),
        Span::styled("  │  ", Style::new().fg(DIM)),
        Span::styled(format!("{answered}"), Style::new().fg(GOOD).bold()),
        Span::styled(format!("/{} answered", e.questions.len()), Style::new().fg(DIM)),
        Span::styled("  │  ", Style::new().fg(DIM)),
        Span::styled(format!("{flagged}"), Style::new().fg(FLAG).bold()),
        Span::styled(" flagged", Style::new().fg(DIM)),
    ]);
    f.render_widget(Paragraph::new(head), header);

    // Timer progress bar (drains as time passes).
    let total = s.total().as_secs_f64();
    let rem = s.remaining();
    let ratio = (rem.as_secs_f64() / total).clamp(0.0, 1.0);
    let color = if rem.as_secs() <= 10 * 60 {
        BAD
    } else if ratio <= 0.25 {
        WARN
    } else {
        GOOD
    };
    let mut label_style = Style::new().bold();
    // Gentle pulse in the last five minutes.
    if rem.as_secs() <= 5 * 60 && rem.as_millis() % 1000 < 500 {
        label_style = label_style.add_modifier(Modifier::REVERSED);
    }
    let gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::new().fg(DIM))
                .title(Line::from(" Time ").fg(ACCENT).bold())
                .title(
                    Line::from(format!(" elapsed {} ", fmt_hms(s.elapsed().as_secs())))
                        .fg(DIM)
                        .right_aligned(),
                ),
        )
        .gauge_style(Style::new().fg(color))
        .ratio(ratio)
        .use_unicode(true)
        .label(Span::styled(format!("{} remaining", fmt_hms(rem.as_secs())), label_style));
    f.render_widget(gauge, timer);

    // Body: question + navigator
    let show_nav = body.width >= 96;
    let [main, nav] = if show_nav {
        Layout::horizontal([Constraint::Fill(1), Constraint::Length(30)]).areas(body)
    } else {
        [body, Rect::default()]
    };

    let chosen = &s.answers[s.current];
    let inner_w = main.width.saturating_sub(4) as usize;
    let mut lines: Vec<Line> = Vec::new();

    let sel_color = if is_complete(q, chosen) { GOOD } else if chosen.is_empty() { DIM } else { WARN };
    let mut badge = vec![
        Span::styled(format!(" Q{} ", q.number), Style::new().fg(Color::Black).bg(ACCENT).bold()),
        Span::raw(" "),
        Span::styled(
            if q.select == 1 { " Select 1 ".to_string() } else { format!(" Select {} ", q.select) },
            Style::new().fg(if q.select > 1 { Color::Black } else { Color::Reset }).bg(if q.select > 1 {
                WARN
            } else {
                Color::Reset
            }).add_modifier(if q.select > 1 { Modifier::BOLD } else { Modifier::empty() }),
        ),
        Span::styled(format!("  {}/{} selected", chosen.len(), q.select), Style::new().fg(sel_color)),
    ];
    if s.flagged[s.current] {
        badge.push(Span::styled("  ⚑ flagged", Style::new().fg(FLAG).bold()));
    }
    lines.push(Line::from(badge));
    lines.push(Line::raw(""));
    lines.extend(wrap(vec![], md_spans(&q.text, Style::new().bold()), inner_w));
    lines.push(Line::raw(""));

    let mut cursor_line = (0, 0);
    for (i, (letter, text)) in q.options.iter().enumerate() {
        let picked = chosen.contains(letter);
        let at = i == s.cursor;
        let mark = match (q.select == 1, picked) {
            (true, true) => "◉",
            (true, false) => "○",
            (false, true) => "▣",
            (false, false) => "□",
        };
        let base = if picked { Style::new().fg(GOOD) } else { Style::new() };
        let base = if at { base.bold() } else { base };
        let prefix = vec![
            Span::styled(if at { " ▌" } else { "  " }, Style::new().fg(ACCENT)),
            Span::styled(format!("{mark} "), if picked { Style::new().fg(GOOD) } else { Style::new().fg(DIM) }),
            Span::styled(
                format!("{letter}. "),
                if at { Style::new().fg(ACCENT).bold() } else { Style::new().fg(DIM) },
            ),
        ];
        let start = lines.len();
        lines.extend(wrap(prefix, md_spans(text, base), inner_w));
        if at {
            cursor_line = (start, lines.len());
        }
        lines.push(Line::raw(""));
    }
    if let Some(n) = s.notice() {
        lines.push(Line::from(format!("  ⚠ {n}")).fg(WARN));
    }

    let inner_h = main.height.saturating_sub(2) as usize;
    let scroll = cursor_line.1.saturating_sub(inner_h) as u16;
    let scen = if q.scenario.len() > inner_w.saturating_sub(4) { short_scenario(&q.scenario) } else { q.scenario.clone() };
    f.render_widget(Paragraph::new(lines).scroll((scroll, 0)).block(panel(scen)), main);

    if show_nav {
        draw_navigator(f, nav, e, &s.answers, Some(&s.flagged), s.current, None);
    }

    let hints: &[(&str, &str)] = if area.width >= 128 {
        &[
            ("↑↓", "option"),
            ("spc/a-d", "select"),
            ("←→", "question"),
            ("f", "flag"),
            ("u", "unanswered"),
            ("m", "flagged"),
            ("g", "go to"),
            ("s", "submit"),
            ("esc", "abandon"),
        ]
    } else {
        &[("↑↓", "option"), ("spc", "select"), ("←→", "nav"), ("f", "flag"), ("g", "goto"), ("s", "submit")]
    };
    f.render_widget(key_hint(hints), foot);
}

fn short_scenario(s: &str) -> String {
    s.split(" — ").next().unwrap_or(s).to_string()
}

/// 6-wide grid of question numbers. With `results`, colours show correctness.
fn draw_navigator(
    f: &mut Frame,
    area: Rect,
    e: &Exam,
    answers: &[Vec<char>],
    flagged: Option<&[bool]>,
    current: usize,
    results: Option<()>,
) {
    let mut lines = vec![Line::raw("")];
    for row in e.questions.chunks(6).enumerate() {
        let (r, qs) = row;
        let mut spans = Vec::new();
        for (c, q) in qs.iter().enumerate() {
            let i = r * 6 + c;
            let a = &answers[i];
            let mut st = if results.is_some() {
                if q.is_correct(a) { Style::new().fg(GOOD) } else { Style::new().fg(BAD) }
            } else if is_complete(q, a) {
                Style::new().fg(GOOD)
            } else if !a.is_empty() {
                Style::new().fg(WARN)
            } else {
                Style::new().fg(DIM)
            };
            if flagged.is_some_and(|fl| fl[i]) {
                st = Style::new().fg(FLAG).bold();
            }
            if i == current {
                st = st.add_modifier(Modifier::REVERSED | Modifier::BOLD);
            }
            spans.push(Span::styled(format!("{:>2}", i + 1), st));
            spans.push(Span::raw("  "));
        }
        lines.push(Line::from(spans));
    }
    lines.push(Line::raw(""));
    let legend = |c: Color, t: &str| Line::from(vec![Span::styled("■ ", Style::new().fg(c)), Span::styled(t.to_string(), Style::new().fg(DIM))]);
    if results.is_some() {
        lines.push(legend(GOOD, "correct"));
        lines.push(legend(BAD, "incorrect"));
    } else {
        lines.push(legend(GOOD, "answered"));
        lines.push(legend(WARN, "incomplete"));
        lines.push(legend(FLAG, "flagged"));
        lines.push(legend(DIM, "unanswered"));
    }
    f.render_widget(Paragraph::new(lines).block(panel("Navigator")), area);
}

// ───────────────────────────── Results ─────────────────────────────

const BIG: [(char, [&str; 5]); 6] = [
    ('P', ["████ ", "█   █", "████ ", "█    ", "█    "]),
    ('A', [" ███ ", "█   █", "█████", "█   █", "█   █"]),
    ('S', [" ████", "█    ", " ███ ", "    █", "████ "]),
    ('F', ["█████", "█    ", "████ ", "█    ", "█    "]),
    ('I', ["███", " █ ", " █ ", " █ ", "███"]),
    ('L', ["█    ", "█    ", "█    ", "█    ", "█████"]),
];

fn big_text(word: &str, style: Style) -> Vec<Line<'static>> {
    (0..5)
        .map(|row| {
            let s: Vec<String> = word
                .chars()
                .filter_map(|c| BIG.iter().find(|(k, _)| *k == c).map(|(_, g)| g[row].to_string()))
                .collect();
            Line::styled(s.join("  "), style)
        })
        .collect()
}

/// A 100–1,000 scale bar with the cut score marked.
fn scale_bar(width: usize, scaled: f64, passed: bool) -> Vec<Line<'static>> {
    let width = width.max(20);
    let pos = |v: f64| (((v - SCALE_MIN as f64) / (SCALE_MAX - SCALE_MIN) as f64) * (width - 1) as f64).round() as usize;
    let fill = pos(scaled);
    let cut = pos(CUT_SCORE as f64);
    let mut spans = Vec::new();
    for i in 0..width {
        if i == cut {
            spans.push(Span::styled("┃", Style::new().fg(WARN).bold()));
        } else if i <= fill {
            spans.push(Span::styled("━", Style::new().fg(if passed { GOOD } else { BAD })));
        } else {
            spans.push(Span::styled("─", Style::new().fg(DIM)));
        }
    }
    let mut labels = vec![' '; width];
    let mut put = |at: usize, s: &str| {
        let at = at.min(width.saturating_sub(s.len()));
        for (k, ch) in s.chars().enumerate() {
            if at + k < width {
                labels[at + k] = ch;
            }
        }
    };
    put(0, &SCALE_MIN.to_string());
    put(cut.saturating_sub(1), &CUT_SCORE.to_string());
    put(width, &SCALE_MAX.to_string());
    vec![Line::from(spans), Line::from(labels.into_iter().collect::<String>()).fg(DIM)]
}

fn mini_bar(pct: f64, width: usize) -> Vec<Span<'static>> {
    let eighths = ["", "▏", "▎", "▍", "▌", "▋", "▊", "▉"];
    let units = (pct / 100.0 * width as f64 * 8.0).round() as usize;
    let full = units / 8;
    let part = eighths[units % 8];
    let color = if pct >= 72.0 { GOOD } else if pct >= 60.0 { WARN } else { BAD };
    let used = full + usize::from(!part.is_empty());
    vec![
        Span::styled("█".repeat(full) + part, Style::new().fg(color)),
        Span::styled("·".repeat(width.saturating_sub(used)), Style::new().fg(DIM)),
    ]
}

fn draw_results(f: &mut Frame, area: Rect, app: &App, idx: usize, since: Duration) {
    let a = &app.attempts[idx];
    let r = &a.report;
    let t = ease(since, Duration::from_millis(1400));
    let color = if r.passed { GOOD } else { BAD };

    let [_, content, foot] =
        Layout::vertical([Constraint::Length(1), Constraint::Fill(1), Constraint::Length(1)]).areas(area);
    let content = centered(content, 96, content.height);
    let [hero, scale, domains] =
        Layout::vertical([Constraint::Length(10), Constraint::Length(5), Constraint::Fill(1)]).areas(content);

    // Hero: big PASS/FAIL + numbers
    let [big, nums] = Layout::horizontal([Constraint::Length(36), Constraint::Fill(1)]).areas(hero);
    let mut big_lines = vec![Line::raw("")];
    // Reveal the verdict after the count-up has mostly finished.
    let verdict_style = if t > 0.6 { Style::new().fg(color).bold() } else { Style::new().fg(DIM) };
    big_lines.extend(big_text(if r.passed { "PASS" } else { "FAIL" }, verdict_style));
    f.render_widget(Paragraph::new(big_lines).alignment(Alignment::Center), big);

    let shown_scaled = (SCALE_MIN as f64 + (r.scaled - SCALE_MIN) as f64 * t).round() as u32;
    let shown_correct = (r.correct as f64 * t).round() as usize;
    let num_lines = vec![
        Line::from(vec![
            Span::styled(format!("{shown_scaled}"), Style::new().fg(color).bold()),
            Span::styled(format!(" / {SCALE_MAX}  scaled score"), Style::new().fg(DIM)),
        ]),
        Line::from(vec![
            Span::styled(format!("{shown_correct}/{}", r.total), Style::new().bold()),
            Span::styled(format!(" correct ({:.1}%)", r.pct() * t), Style::new().fg(DIM)),
            Span::styled(format!("   cut score {CUT_SCORE}"), Style::new().fg(DIM)),
        ]),
        Line::raw(""),
        Line::from(vec![
            Span::styled("Exam      ", Style::new().fg(DIM)),
            Span::raw(a.exam_title.split(" — ").next().unwrap_or("").to_string()),
        ]),
        Line::from(vec![Span::styled("Taken     ", Style::new().fg(DIM)), Span::raw(fmt_date(a.started_at))]),
        Line::from(vec![
            Span::styled("Time used ", Style::new().fg(DIM)),
            Span::raw(format!("{} of {}:00:00", fmt_hms(a.duration_secs), EXAM_MINUTES / 60)),
            if a.timed_out { Span::styled("  (time expired)", Style::new().fg(WARN)) } else { Span::raw("") },
        ]),
        Line::from(vec![
            Span::styled("Answered  ", Style::new().fg(DIM)),
            Span::raw(format!("{}/{}", r.answered, r.total)),
        ]),
        Line::from(vec![
            Span::styled("Readiness ", Style::new().fg(DIM)),
            Span::styled(score::readiness(r.pct()).to_string(), Style::new().italic()),
        ]),
    ];
    f.render_widget(
        Paragraph::new(num_lines).wrap(Wrap { trim: false }).block(Block::default().padding(Padding::top(1))),
        nums,
    );

    let scale_w = scale.width.saturating_sub(4) as usize;
    let shown = SCALE_MIN as f64 + (r.scaled - SCALE_MIN) as f64 * t;
    f.render_widget(Paragraph::new(scale_bar(scale_w, shown, r.passed)).block(panel("Scaled score")), scale);

    // Per-domain table
    let prev = app.previous_attempt(idx);
    let bar_w = (domains.width as usize).saturating_sub(70).clamp(10, 30);
    let mut lines = vec![Line::raw("")];
    for d in r.domains.iter().filter(|d| d.total > 0) {
        let mut spans = vec![
            Span::styled(format!("{:<5}", d.domain), Style::new().fg(ACCENT).bold()),
            Span::raw(format!("{:<40}", exam::domain_name(&d.domain))),
            Span::styled(format!("{:>2}/{:<2} ", d.correct, d.total), Style::new().fg(DIM)),
        ];
        spans.extend(mini_bar(d.pct() * t, bar_w));
        spans.push(Span::styled(format!(" {:>5.1}%", d.pct()), Style::new().bold()));
        if let Some(p) = prev {
            if let Some(pd) = app.attempts[p].report.domains.iter().find(|x| x.domain == d.domain) {
                spans.push(delta_span(d.pct() - pd.pct(), " pp"));
            }
        }
        lines.push(Line::from(spans));
    }
    lines.push(Line::raw(""));
    let note = if let Some(p) = prev {
        let pa = &app.attempts[p];
        let diff = r.scaled as i64 - pa.report.scaled as i64;
        Line::from(vec![
            Span::styled("vs previous attempt ", Style::new().fg(DIM)),
            Span::raw(format!("({}, {}) ", fmt_date(pa.started_at), pa.report.scaled)),
            delta_span(diff as f64, " pts"),
        ])
    } else {
        Line::from("First recorded attempt on this exam").fg(DIM)
    };
    lines.push(note);
    lines.push(
        Line::from("Domain percentages are diagnostic only; pass/fail uses the total scaled score.").fg(DIM).italic(),
    );
    if let Some(err) = &app.save_error {
        lines.push(Line::from(format!("⚠ could not save attempt: {err}")).fg(BAD));
    }
    f.render_widget(Paragraph::new(lines).block(panel("Percent correct by domain")), domains);

    f.render_widget(
        key_hint(&[("r", "review all"), ("w", "review incorrect"), ("h", "history"), ("enter", "back")]),
        foot,
    );
}

fn delta_span(d: f64, unit: &str) -> Span<'static> {
    if d.abs() < 0.05 {
        Span::styled(format!("  ═ 0{unit}"), Style::new().fg(DIM))
    } else if d > 0.0 {
        Span::styled(format!("  ▲ +{:.0}{unit}", d), Style::new().fg(GOOD))
    } else {
        Span::styled(format!("  ▼ {:.0}{unit}", d), Style::new().fg(BAD))
    }
}

// ───────────────────────────── Review ─────────────────────────────

fn draw_review(
    f: &mut Frame,
    area: Rect,
    app: &App,
    idx: usize,
    filter: ReviewFilter,
    cursor: usize,
    scroll: u16,
) {
    let a = &app.attempts[idx];
    let Some(ei) = app.exam_index(&a.exam_id) else {
        f.render_widget(Paragraph::new("Exam for this attempt is no longer available."), area);
        return;
    };
    let e = &app.exams[ei];
    let rows = app.review_rows(idx, filter);

    let [header, body, foot] =
        Layout::vertical([Constraint::Length(1), Constraint::Fill(1), Constraint::Length(1)]).areas(area);
    let wrong = e.questions.iter().enumerate().filter(|(i, q)| !q.is_correct(&a.answers[*i])).count();
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(format!(" Review · {} ", short_title(e)), Style::new().fg(ACCENT).bold()),
            Span::styled("│ ", Style::new().fg(DIM)),
            Span::styled(format!("{} correct", e.questions.len() - wrong), Style::new().fg(GOOD)),
            Span::styled("  ", Style::new()),
            Span::styled(format!("{wrong} incorrect"), Style::new().fg(BAD)),
            Span::styled("  │  filter: ", Style::new().fg(DIM)),
            Span::styled(filter.label(), Style::new().bold()),
        ])),
        header,
    );

    let [list, detail] = Layout::horizontal([Constraint::Length(30), Constraint::Fill(1)]).areas(body);

    // List
    let list_h = list.height.saturating_sub(2) as usize;
    let offset = cursor.saturating_sub(list_h.saturating_sub(1));
    let mut lines = Vec::new();
    if rows.is_empty() {
        lines.push(Line::from("Nothing to show").fg(DIM).italic());
    }
    for (k, &qi) in rows.iter().enumerate().skip(offset).take(list_h) {
        let q = &e.questions[qi];
        let ok = q.is_correct(&a.answers[qi]);
        let at = k == cursor;
        let mark = if ok { Span::styled("✓", Style::new().fg(GOOD)) } else { Span::styled("✗", Style::new().fg(BAD)) };
        let mut st = Style::new();
        if at {
            st = st.fg(ACCENT).bold();
        }
        lines.push(Line::from(vec![
            Span::styled(if at { "▌" } else { " " }, Style::new().fg(ACCENT)),
            mark,
            Span::styled(format!(" Q{:<3}", q.number), st),
            Span::styled(format!("{:<5}", q.domain), Style::new().fg(DIM)),
            Span::styled(
                if a.answers[qi].is_empty() { "—".to_string() } else { a.answers[qi].iter().collect::<String>() },
                if ok { Style::new().fg(GOOD) } else { Style::new().fg(BAD) },
            ),
        ]));
    }
    f.render_widget(Paragraph::new(lines).block(panel(format!("{} items", rows.len()))), list);

    // Detail
    let Some(&qi) = rows.get(cursor) else {
        f.render_widget(Paragraph::new("").block(panel("Detail")), detail);
        f.render_widget(key_hint(&[("f", "filter"), ("esc", "back")]), foot);
        return;
    };
    let q = &e.questions[qi];
    let chosen = &a.answers[qi];
    let ok = q.is_correct(chosen);
    let w = detail.width.saturating_sub(4) as usize;
    let mut d: Vec<Line> = Vec::new();
    d.push(Line::from(vec![
        Span::styled(format!(" Q{} ", q.number), Style::new().fg(Color::Black).bg(ACCENT).bold()),
        Span::raw(" "),
        if ok {
            Span::styled(" ✓ CORRECT ", Style::new().fg(Color::Black).bg(GOOD).bold())
        } else if chosen.is_empty() {
            Span::styled(" — NOT ANSWERED ", Style::new().fg(Color::Black).bg(WARN).bold())
        } else {
            Span::styled(" ✗ INCORRECT ", Style::new().fg(Color::Black).bg(BAD).bold())
        },
        Span::styled(format!("  Select {}", q.select), Style::new().fg(DIM)),
    ]));
    d.push(Line::from(vec![
        Span::styled(format!("{} ", q.domain), Style::new().fg(ACCENT)),
        Span::styled(exam::domain_name(&q.primary_domain()).to_string(), Style::new().fg(DIM)),
        Span::styled(format!("  ·  {}", short_scenario(&q.scenario)), Style::new().fg(DIM)),
    ]));
    d.push(Line::raw(""));
    d.extend(wrap(vec![], md_spans(&q.text, Style::new().bold()), w));
    d.push(Line::raw(""));
    for (letter, text) in &q.options {
        let right = q.answer.contains(letter);
        let picked = chosen.contains(letter);
        let (mark, st, tag) = match (right, picked) {
            (true, true) => ("✓", Style::new().fg(GOOD).bold(), "  ← your answer · correct"),
            (true, false) => ("✓", Style::new().fg(GOOD), "  ← correct answer"),
            (false, true) => ("✗", Style::new().fg(BAD).bold(), "  ← your answer · wrong"),
            (false, false) => (" ", Style::new().fg(DIM), ""),
        };
        let mut spans = md_spans(text, st);
        if !tag.is_empty() {
            spans.push(Span::styled(tag, st.add_modifier(Modifier::ITALIC)));
        }
        d.extend(wrap(
            vec![Span::styled(format!(" {mark} "), st), Span::styled(format!("{letter}. "), st)],
            spans,
            w,
        ));
    }
    d.push(Line::raw(""));
    let fmt = |v: &[char]| if v.is_empty() { "—".to_string() } else { v.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(", ") };
    d.push(Line::from(vec![
        Span::styled("Your answer    ", Style::new().fg(DIM)),
        Span::styled(fmt(chosen), Style::new().fg(if ok { GOOD } else { BAD }).bold()),
    ]));
    d.push(Line::from(vec![
        Span::styled("Correct answer ", Style::new().fg(DIM)),
        Span::styled(fmt(&q.answer), Style::new().fg(GOOD).bold()),
    ]));
    if !ok && q.select > 1 && !chosen.is_empty() {
        d.push(Line::from("Multiple-response items score only when the exact set is selected.").fg(WARN).italic());
    }
    d.push(Line::raw(""));
    d.push(Line::from(if ok { "Why this is right" } else { "Why — and what went wrong" }).fg(ACCENT).bold());
    d.extend(wrap(vec![Span::raw("  ")], md_spans(&q.why, Style::new()), w));
    if !ok && !chosen.is_empty() {
        let wrong: Vec<String> = chosen
            .iter()
            .filter(|c| !q.answer.contains(c))
            .filter_map(|c| q.options.iter().find(|(l, _)| l == c))
            .map(|(l, t)| format!("{l}. {t}"))
            .collect();
        let missed: Vec<char> = q.answer.iter().copied().filter(|c| !chosen.contains(c)).collect();
        if !wrong.is_empty() {
            d.push(Line::raw(""));
            d.push(Line::from("You chose").fg(BAD).bold());
            for wtxt in wrong {
                d.extend(wrap(vec![Span::styled("  ✗ ", Style::new().fg(BAD))], md_spans(&wtxt, Style::new().fg(BAD)), w));
            }
        }
        if !missed.is_empty() && q.select > 1 {
            d.push(Line::from(format!("You missed: {}", fmt(&missed))).fg(WARN));
        }
    }

    let max_scroll = (d.len() as u16).saturating_sub(detail.height.saturating_sub(2));
    f.render_widget(Paragraph::new(d).scroll((scroll.min(max_scroll), 0)).block(panel("Detail")), detail);

    f.render_widget(
        key_hint(&[("↑↓", "question"), ("pgup/pgdn", "scroll"), ("f", "filter all/incorrect/correct"), ("esc", "back")]),
        foot,
    );
}

// ───────────────────────────── History ─────────────────────────────

fn draw_history(f: &mut Frame, area: Rect, app: &App, cursor: usize, filter: Option<usize>) {
    let rows = app.history_rows(filter);
    let [header, top, bottom, foot] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(14),
        Constraint::Length(1),
    ])
    .areas(area);

    let filt = match filter {
        None => "All exams".to_string(),
        Some(i) => short_title(&app.exams[i]),
    };
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" Attempt history ", Style::new().fg(ACCENT).bold()),
            Span::styled("│ showing: ", Style::new().fg(DIM)),
            Span::styled(filt, Style::new().bold()),
            Span::styled(format!("  ({} attempts)", rows.len()), Style::new().fg(DIM)),
        ])),
        header,
    );

    if rows.is_empty() {
        f.render_widget(
            Paragraph::new(vec![
                Line::raw(""),
                Line::from("No attempts yet — finish an exam to see it here.").fg(DIM).italic(),
            ])
            .alignment(Alignment::Center)
            .block(panel("Attempts")),
            top,
        );
        f.render_widget(key_hint(&[("tab", "filter exam"), ("esc", "back")]), foot);
        return;
    }

    let table_rows: Vec<Row> = rows
        .iter()
        .map(|&i| {
            let a = &app.attempts[i];
            let r = &a.report;
            let delta = app.previous_attempt(i).map(|p| r.scaled as i64 - app.attempts[p].report.scaled as i64);
            let delta_cell = match delta {
                None => Cell::from("—").fg(DIM),
                Some(0) => Cell::from("═ 0").fg(DIM),
                Some(d) if d > 0 => Cell::from(format!("▲ +{d}")).fg(GOOD),
                Some(d) => Cell::from(format!("▼ {d}")).fg(BAD),
            };
            let title = app.exam_index(&a.exam_id).map(|e| short_title(&app.exams[e])).unwrap_or(a.exam_id.clone());
            Row::new(vec![
                Cell::from(fmt_date(a.started_at)),
                Cell::from(title.replace("Practice Exam ", "Exam ")),
                Cell::from(format!("{}", r.scaled)).fg(if r.passed { GOOD } else { BAD }).bold(),
                Cell::from(format!("{}/{}", r.correct, r.total)),
                Cell::from(format!("{:.1}%", r.pct())),
                if r.passed { Cell::from("PASS").fg(GOOD).bold() } else { Cell::from("FAIL").fg(BAD).bold() },
                Cell::from(fmt_hms(a.duration_secs) + if a.timed_out { " ⏱" } else { "" }),
                delta_cell,
            ])
        })
        .collect();
    let table = Table::new(
        table_rows,
        [
            Constraint::Length(17),
            Constraint::Length(10),
            Constraint::Length(7),
            Constraint::Length(7),
            Constraint::Length(7),
            Constraint::Length(6),
            Constraint::Length(10),
            Constraint::Length(8),
        ],
    )
    .header(
        Row::new(["Date", "Exam", "Scaled", "Raw", "%", "Result", "Time", "Δ prev"])
            .style(Style::new().fg(DIM).add_modifier(Modifier::UNDERLINED)),
    )
    .row_highlight_style(Style::new().add_modifier(Modifier::REVERSED))
    .highlight_symbol("▌ ")
    .column_spacing(2)
    .block(panel("Attempts"));
    let mut state = TableState::default().with_selected(Some(cursor.min(rows.len() - 1)));
    f.render_stateful_widget(table, top, &mut state);

    // Bottom: comparison with previous attempt + trend chart.
    let [cmp, trend] = Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)]).areas(bottom);
    let sel = rows[cursor.min(rows.len() - 1)];
    let a = &app.attempts[sel];
    let mut lines = vec![];
    match app.previous_attempt(sel) {
        Some(p) => {
            let pa = &app.attempts[p];
            lines.push(Line::from(vec![
                Span::styled("Previous ", Style::new().fg(DIM)),
                Span::raw(format!("{}  ", fmt_date(pa.started_at))),
                Span::styled(format!("{}", pa.report.scaled), Style::new().fg(if pa.report.passed { GOOD } else { BAD })),
                Span::styled(" → ", Style::new().fg(DIM)),
                Span::styled(format!("{}", a.report.scaled), Style::new().fg(if a.report.passed { GOOD } else { BAD }).bold()),
                delta_span(a.report.scaled as f64 - pa.report.scaled as f64, " pts"),
            ]));
            lines.push(Line::raw(""));
            for d in a.report.domains.iter().filter(|d| d.total > 0) {
                let old = pa.report.domains.iter().find(|x| x.domain == d.domain).map(|x| x.pct()).unwrap_or(0.0);
                lines.push(Line::from(vec![
                    Span::styled(format!("{:<5}", d.domain), Style::new().fg(ACCENT).bold()),
                    Span::styled(format!("{:>5.1}%", old), Style::new().fg(DIM)),
                    Span::styled(" → ", Style::new().fg(DIM)),
                    Span::styled(format!("{:>5.1}%", d.pct()), Style::new().bold()),
                    delta_span(d.pct() - old, " pp"),
                ]));
            }
        }
        None => {
            lines.push(Line::from("First attempt on this exam — nothing to compare yet.").fg(DIM).italic());
            lines.push(Line::raw(""));
            for d in a.report.domains.iter().filter(|d| d.total > 0) {
                lines.push(Line::from(vec![
                    Span::styled(format!("{:<5}", d.domain), Style::new().fg(ACCENT).bold()),
                    Span::styled(format!("{:>5.1}%", d.pct()), Style::new().bold()),
                    Span::styled(format!("  {}/{}", d.correct, d.total), Style::new().fg(DIM)),
                ]));
            }
        }
    }
    f.render_widget(Paragraph::new(lines).block(panel("Compare with previous")), cmp);

    // Trend: chronological scaled scores for the selected attempt's exam.
    let mut series: Vec<&Attempt> = app.attempts.iter().filter(|x| x.exam_id == a.exam_id).collect();
    series.sort_by_key(|x| x.started_at);
    let max_bars = (trend.width.saturating_sub(4) / 6).max(1) as usize;
    let skip = series.len().saturating_sub(max_bars);
    let bars: Vec<Bar> = series
        .iter()
        .enumerate()
        .skip(skip)
        .map(|(k, x)| {
            let is_sel = std::ptr::eq(*x, a);
            let mut st = Style::new().fg(if x.report.passed { GOOD } else { BAD });
            if is_sel {
                st = st.add_modifier(Modifier::BOLD);
            }
            Bar::default()
                .value(x.report.scaled as u64)
                .label(Line::from(format!("#{}", k + 1)).style(if is_sel { Style::new().fg(ACCENT).bold() } else { Style::new().fg(DIM) }))
                .text_value(format!("{}", x.report.scaled))
                .style(st)
                .value_style(st.add_modifier(Modifier::REVERSED))
        })
        .collect();
    let title = app.exam_index(&a.exam_id).map(|e| short_title(&app.exams[e])).unwrap_or_default();
    let chart = BarChart::default()
        .block(panel(format!("Trend · {title} (cut {CUT_SCORE})")))
        .data(BarGroup::default().bars(&bars))
        .bar_width(5)
        .bar_gap(1)
        .max(SCALE_MAX as u64);
    f.render_widget(chart, trend);

    f.render_widget(
        key_hint(&[("↑↓", "select"), ("enter", "results"), ("r", "review mistakes"), ("tab", "filter exam"), ("d", "delete"), ("esc", "back")]),
        foot,
    );
}

// ───────────────────────────── Modals ─────────────────────────────

fn draw_modal(f: &mut Frame, area: Rect, app: &App, m: &Modal) {
    let (title, body): (&str, Vec<Line>) = match m {
        Modal::ConfirmSubmit => {
            let mut v = vec![Line::from("Submit your exam for scoring?").bold(), Line::raw("")];
            if let Screen::Exam(s) = &app.screen {
                let e = &app.exams[s.exam];
                let incomplete = e.questions.iter().zip(&s.answers).filter(|(q, a)| !is_complete(q, a)).count();
                let flagged = s.flagged.iter().filter(|&&x| x).count();
                if incomplete > 0 {
                    v.push(Line::from(format!("⚠ {incomplete} question(s) unanswered or incomplete")).fg(WARN));
                }
                if flagged > 0 {
                    v.push(Line::from(format!("⚑ {flagged} question(s) still flagged")).fg(FLAG));
                }
                v.push(Line::from(format!("{} remaining on the clock", fmt_hms(s.remaining().as_secs()))).fg(DIM));
            }
            v.push(Line::raw(""));
            v.push(Line::from(vec![
                Span::styled(" y ", Style::new().fg(Color::Black).bg(GOOD).bold()),
                Span::raw(" submit    "),
                Span::styled(" any key ", Style::new().fg(Color::Black).bg(DIM).bold()),
                Span::raw(" keep working"),
            ]));
            ("Submit", v)
        }
        Modal::ConfirmAbandon => (
            "Abandon",
            vec![
                Line::from("Abandon this attempt?").bold(),
                Line::raw(""),
                Line::from("Your answers will be discarded and not recorded.").fg(DIM),
                Line::raw(""),
                Line::from(vec![
                    Span::styled(" y ", Style::new().fg(Color::Black).bg(BAD).bold()),
                    Span::raw(" abandon    "),
                    Span::styled(" any key ", Style::new().fg(Color::Black).bg(DIM).bold()),
                    Span::raw(" continue exam"),
                ]),
            ],
        ),
        Modal::ConfirmDelete => (
            "Delete",
            vec![
                Line::from("Delete the selected attempt permanently?").bold(),
                Line::raw(""),
                Line::from(vec![
                    Span::styled(" y ", Style::new().fg(Color::Black).bg(BAD).bold()),
                    Span::raw(" delete    "),
                    Span::styled(" any key ", Style::new().fg(Color::Black).bg(DIM).bold()),
                    Span::raw(" cancel"),
                ]),
            ],
        ),
        Modal::Goto(buf) => (
            "Go to question",
            vec![
                Line::from(vec![
                    Span::raw(format!("Question number (1–{EXAM_ITEMS}): ")),
                    Span::styled(format!("{buf}▏"), Style::new().fg(ACCENT).bold()),
                ]),
                Line::raw(""),
                Line::from("enter to jump · esc to cancel").fg(DIM),
            ],
        ),
    };
    let h = body.len() as u16 + 4;
    let r = centered(area, 58, h);
    f.render_widget(Clear, r);
    f.render_widget(
        Paragraph::new(Text::from(body)).alignment(Alignment::Center).block(
            panel(title).border_style(Style::new().fg(ACCENT)).padding(Padding::uniform(1)),
        ),
        r,
    );
}

# CCAR-F Practice Exam TUI

A terminal app for taking timed practice exams for the **Claude Certified Architect – Foundations (CCAR-F)** certification. It runs the exams under the same conditions the official Exam Guide describes: 60 questions, 120 minutes, and a scaled score from 100 to 1,000 with a pass mark of 720.

Built in Rust with [ratatui](https://ratatui.rs). It uses only your terminal's own colours, so it matches your theme.

## Features

- **Two full practice exams.** Pick Practice Exam #1 or #2 from the menu. Each has 60 scenario-based questions.
- **Timer.** A progress bar shows the time remaining and drains as time passes. It turns yellow at 25% left, red in the last 10 minutes, and pulses in the last 5 minutes. The exam submits itself when time runs out.
- **Exam-style answering.** Each question says how many answers to pick ("Select 1" or "Select 2"), and you have to pick exactly that many.
- **Navigator.** A grid of all 60 questions shows which are answered, incomplete, flagged or unanswered. You can flag questions and jump to any number, or to the next unanswered or flagged one.
- **Results.** Shows PASS or FAIL, the scaled score with the 720 mark, the number correct, the time used, the percentage correct per domain, and readiness advice.
- **Review.** Shows all questions, only the wrong ones or only the right ones. For each question you see your answer, the correct answer, and the explanation from the answer key. For "Select 2" questions it also lists which correct option you missed.
- **Attempt history.** Lists every past attempt with the change from your previous try. It also compares domain scores between attempts and charts your scores over time. You can reopen the results or review of any attempt, or delete it.

## Scoring

Scoring follows the CCAR-F Exam Guide, section 10, "How the Exam Is Scored":

| Rule | Implementation |
|---|---|
| Items | 60 |
| Time limit | 120 minutes; submits automatically at zero |
| Item scoring | 1 point per item. "Select 2" items score only when both choices are correct (no partial credit) |
| Unanswered | Scored as incorrect; wrong answers lose nothing extra |
| Scaled score | 100–1,000 |
| Pass mark | 720 |
| Report | Pass/fail, scaled score, and percent correct per domain (the domain figures do not affect pass/fail) |

> **Note:** the official conversion from correct answers to the 100–1,000 scale is not published. This app uses a straight-line conversion, `scaled = 100 + 900 × correct / 60`, so passing takes **42 of 60 correct (70%)**.

For the per-domain breakdown, a question tagged with two domains in the answer key (e.g. `D4/1`) counts under the first one.

## Install and run

You need a [Rust toolchain](https://rustup.rs) (edition 2024, Rust 1.85 or newer).

```sh
git clone git@github.com:JaavLex/Claude_Certified_Architect_TrainingExams.git
cd Claude_Certified_Architect_TrainingExams
cargo run --release
```

To install it as a command you can run from anywhere:

```sh
cargo install --path .
exam-tui
```

A terminal of at least 80×24 is recommended. At 96 columns or wider, the question navigator appears next to the question.

## Keys

| Screen | Key | Action |
|---|---|---|
| Menu | `↑↓` / `j k`, `Enter` | Move, select |
| | `1` / `2` | Start Exam #1 / #2 |
| | `h` | Attempt history |
| | `q` | Quit |
| Exam | `↑↓` / `j k` | Move between options |
| | `Space` / `Enter`, `a`–`d`, `1`–`4` | Select or deselect an option |
| | `←→` / `h l` / `Tab` | Previous / next question |
| | `f` | Flag or unflag the question |
| | `u` / `m` | Next unanswered / next flagged question |
| | `g` | Go to a question number |
| | `s` | Submit (asks for confirmation) |
| | `Esc` | Abandon the attempt (not saved) |
| Results | `r` / `w` | Review all / incorrect only |
| | `h` | History |
| Review | `↑↓` | Move between questions |
| | `PgUp` / `PgDn` (`K` / `J`) | Scroll the explanation |
| | `f` | Switch filter: all / incorrect / correct |
| History | `Enter` | Open the attempt's results |
| | `r` | Review that attempt's mistakes |
| | `Tab` | Filter by exam |
| | `d` | Delete the attempt |

## Data

Attempts are saved as JSON in your user data folder:

- Linux: `~/.local/share/exam-tui/attempts.json`
- macOS: `~/Library/Application Support/exam-tui/attempts.json`
- Windows: `%APPDATA%\exam-tui\attempts.json`

## Exam content

The questions live in [`exams/`](exams/) as Markdown and are built into the program, so it runs without any other files. To change a question or explanation, edit the Markdown and rebuild.

The app reads this format:

```markdown
## Scenario A — Some scenario

**1. (Select 1)** Question text…
- A. Option
- B. Option

# ✅ ANSWER KEY — …

| Q | Ans | Domain | Why |
|---|-----|--------|-----|
| 1 | **A** | D1 | Explanation… |
```

The program refuses to start if an exam doesn't have exactly 60 questions, or if an answer in the key doesn't match the number of choices the question asks for. Run `cargo test` to check both exams.

## Project layout

```
src/
  main.rs   main loop and terminal setup
  app.rs    screens and key handling
  ui.rs     drawing each screen
  exam.rs   reading the exam Markdown and answer key
  score.rs  scoring and pass/fail
  store.rs  saved attempt history
  text.rs   formatting and line wrapping
exams/      practice exam files
```

## Disclaimer

This is an unofficial study tool and is not affiliated with or endorsed by Anthropic. The practice questions are study material, not real exam questions.

## License

[MIT](LICENSE)

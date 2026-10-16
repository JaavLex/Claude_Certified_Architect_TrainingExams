# CCAR-F Practice Exam TUI

A terminal app for taking timed practice exams for the **Claude Certified Architect – Foundations (CCAR-F)** certification. It runs the exams under the same conditions the official Exam Guide describes: 60 questions, 120 minutes, and a scaled score from 100 to 1,000 with a pass mark of 720.

Built in Rust with [ratatui](https://ratatui.rs). It uses only your terminal's own colours, so it matches your theme.

## Features

- **Two full practice exams.** Pick Practice Exam #1 or #2 from the menu. Each has 60 scenario-based questions.
- **Custom exams.** Put a `.json` exam file in your exam folder, or pass it on the command line, and it appears in the menu. See [Custom exams](#custom-exams).
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

> **Note:** the official conversion from correct answers to the 100–1,000 scale is not published. This app uses a straight-line conversion, `scaled = 100 + 900 × correct / items`, so passing a 60-item exam takes **42 of 60 correct (70%)**.

For the per-domain breakdown, a question tagged with two domains (e.g. `D4/1`) counts under the first one.

Custom exams are scored with the same rules. Only the number of items and the time limit can differ.

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

Command-line options:

```sh
exam-tui                          # built-in exams + your exam folder
exam-tui my-exam.json other/      # also load these files or folders
exam-tui my-exam.json --check     # validate exam files and exit
exam-tui --help
```

A terminal of at least 80×24 is recommended. At 96 columns or wider, the question navigator appears next to the question.

## Keys

| Screen | Key | Action |
|---|---|---|
| Menu | `↑↓` / `j k`, `Enter` | Move, select |
| | `1`–`9` | Start that exam |
| | `h` | Attempt history |
| | `r` | Reload exam files |
| | `e` | Show exam files that failed to load |
| | `q` | Quit |
| Exam | `↑↓` / `j k` | Move between options |
| | `Space` / `Enter` | Select or deselect the highlighted option |
| | `a`–`e` | Select the option with that letter |
| | `1`–`9` | Select the option in that position |
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

Attempts are saved as JSON in your user data folder, next to the `exams/` folder for custom exams:

- Linux: `~/.local/share/exam-tui/attempts.json`
- macOS: `~/Library/Application Support/exam-tui/attempts.json`
- Windows: `%APPDATA%\exam-tui\attempts.json`

## Custom exams

Exams are JSON files. The two built-in exams are in [`exams/`](exams/) and are built into the program. You can add your own without rebuilding.

### Where to put them

The app loads every `*.json` file in your exam folder when it starts:

- Linux: `~/.local/share/exam-tui/exams/`
- macOS: `~/Library/Application Support/exam-tui/exams/`
- Windows: `%APPDATA%\exam-tui\exams\`

You can also pass files or folders on the command line: `exam-tui path/to/exam.json`.

Custom exams are marked with ✦ in the menu. Press `r` on the menu to reload after you add or edit a file. A file with the same `id` as another exam replaces it, so you can also override a built-in exam this way.

### Format

[`examples/custom-exam.json`](examples/custom-exam.json) is a complete 3-question example. The full format is in [`schema/exam.schema.json`](schema/exam.schema.json). Add `"$schema"` to your file to get autocomplete and checking in editors such as VS Code:

```json
{
  "$schema": "https://raw.githubusercontent.com/JaavLex/Claude_Certified_Architect_TrainingExams/main/schema/exam.schema.json",
  "id": "my-exam",
  "title": "My Exam — optional subtitle",
  "description": "Shown in the menu.",
  "time_limit_minutes": 120,
  "domains": [
    { "id": "D1", "name": "Agentic Architecture & Orchestration" }
  ],
  "questions": [
    {
      "number": 1,
      "scenario": "Scenario A — Customer Support Agent",
      "domain": "D1",
      "select": 1,
      "text": "Which `stop_reason` keeps the loop going?",
      "options": [
        { "key": "A", "text": "`end_turn`" },
        { "key": "B", "text": "`tool_use`" }
      ],
      "answer": ["B"],
      "why": "Explanation shown in the review screen."
    }
  ]
}
```

**Exam fields**

| Field | Required | Meaning |
|---|---|---|
| `id` | yes | Unique id. Attempt history is grouped by it, so keep it the same when you edit the exam. |
| `title` | yes | Full title. The part before ` — ` is the short name used in menus. |
| `description` | no | Text shown in the menu's Details panel. |
| `time_limit_minutes` | no | Time limit. Default: 120. |
| `domains` | no | `id`/`name` pairs for the per-domain breakdown, in display order. |
| `questions` | yes | At least one question. |

**Question fields**

| Field | Required | Meaning |
|---|---|---|
| `number` | yes | Question number. Must be unique in the exam. |
| `text` | yes | The question. |
| `options` | yes | 2–9 options, each with a capital-letter `key` and `text`. |
| `answer` | yes | Keys of the correct options, e.g. `["A", "C"]`. |
| `select` | no | How many options to pick. Default: 1. Must equal the number of keys in `answer`. |
| `scenario` | no | Heading shown above the question. |
| `domain` | no | Domain id, e.g. `D1`. With `D4/1` the first id is used. |
| `why` | no | Explanation shown in the review screen. |

In question, option, description and explanation text, `` `code` `` and `**bold**` are formatted.

### Checking a file

The app checks every file when it loads. A file with mistakes is skipped and listed on the menu (press `e` to see why). To check files without starting the app:

```sh
exam-tui path/to/exam.json --check
```

It checks, for example, that `answer` has exactly `select` keys, that every answer is one of the option keys, and that question numbers and option keys are not repeated.

Run `cargo test` to check the built-in exams. They must keep the official format of 60 items and 120 minutes.

## Project layout

```
src/
  main.rs   main loop and terminal setup
  app.rs    screens and key handling
  ui.rs     drawing each screen
  exam.rs   loading and checking exam JSON files
  score.rs  scoring and pass/fail
  store.rs  saved attempt history
  text.rs   formatting and line wrapping
exams/      built-in practice exams (JSON)
examples/   example custom exam
schema/     JSON Schema for exam files
```

## Disclaimer

This is an unofficial study tool and is not affiliated with or endorsed by Anthropic. The practice questions are study material, not real exam questions.

## License

[MIT](LICENSE)

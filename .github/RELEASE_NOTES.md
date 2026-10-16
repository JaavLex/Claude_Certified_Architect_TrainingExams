Timed practice exams for the **Claude Certified Architect – Foundations (CCAR-F)** certification, in your terminal. No installation needed: download the file for your system, unzip it, and run it.

## Download

| System | File |
|---|---|
| Windows 10/11 (64-bit) | `exam-tui-v1.1.0-windows-x86_64.zip` |
| macOS (Apple Silicon and Intel) | `exam-tui-v1.1.0-macos-universal.zip` |
| Linux (x86_64, any distro) | `exam-tui-v1.1.0-linux-x86_64.tar.gz` |
| Linux (ARM64, e.g. Raspberry Pi) | `exam-tui-v1.1.0-linux-aarch64.tar.gz` |

`SHA256SUMS.txt` lists the checksums of every file.

## First run

The app is not code-signed, so your system asks for confirmation the first time.

- **Windows:** unzip, then double-click `exam-tui.exe`. If "Windows protected your PC" appears, click **More info → Run anyway**.
- **macOS:** unzip, then double-click `exam-tui`; it opens in Terminal. If macOS says it cannot verify the app, open **System Settings → Privacy & Security** and click **Open Anyway**. You can also run `xattr -d com.apple.quarantine exam-tui` in Terminal, in the unzipped folder.
- **Linux:** `tar -xzf exam-tui-v1.1.0-linux-x86_64.tar.gz`, then run `./exam-tui` in its folder.

## What's new in 1.1

**Graded like the real exam.** Each domain now counts at its share of the real exam (Exam Guide §4: D1 27%, D2 18%, D3 20%, D4 20%, D5 15%), not by how many questions the practice exam happens to have in it. Practice Exam #1, for example, has only 5 D2 questions against about 11 on the real exam, so a weak D2 used to barely show. The raw count (e.g. 54/60) is still shown next to the weighted percent.

> Scores change: your earlier attempts are re-graded from their saved answers the first time you start 1.1, so history and comparisons all use the same rules. For example, 54/60 with a weak D3 goes from 910 to 883.

**Grading guidance from the study plan:**
- **Points lost per domain** and a **Focus next** line naming the domain that cost you the most.
- **First attempt** marked on the results, against the study plan's ≥85% target (≥90% = exam-ready). The **Attempt history** menu entry shows a readiness checklist for each exam.
- **Pacing coach:** an on-pace indicator (2:00 per question), a timer on each question, and a "guess, flag it, come back" prompt when one passes 3:00. Results show your average pace, and the review shows the time spent on each question.

**Custom exams:** domains can now have a `weight` (a percent). With weights, your exam is graded the same way; without them, every question counts the same, as before.

## Also in the app

- Two full practice exams: 60 questions and 120 minutes each, scaled 100–1,000, pass at 720.
- A timer that submits automatically when time runs out, a question navigator, flags, and a full review with explanations.
- Attempt history with comparisons and a score trend chart.
- **Train weaknesses:** practise the questions you get wrong until you answer each right twice in a row.
- **Your own exams:** save the example exam from the menu, edit it, then add it with **Add exam from file**.

See the [README](https://github.com/JaavLex/Claude_Certified_Architect_TrainingExams#readme) for the keys and the exam file format.

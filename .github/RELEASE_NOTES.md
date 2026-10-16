Timed practice exams for the **Claude Certified Architect – Foundations (CCAR-F)** certification, in your terminal. No installation needed: download the file for your system, unzip it, and run it.

## Download

| System | File |
|---|---|
| Windows 10/11 (64-bit) | `exam-tui-v1.0.0-windows-x86_64.zip` |
| macOS (Apple Silicon and Intel) | `exam-tui-v1.0.0-macos-universal.zip` |
| Linux (x86_64, any distro) | `exam-tui-v1.0.0-linux-x86_64.tar.gz` |
| Linux (ARM64, e.g. Raspberry Pi) | `exam-tui-v1.0.0-linux-aarch64.tar.gz` |

`SHA256SUMS.txt` lists the checksums of every file.

## First run

The app is not code-signed, so your system asks for confirmation the first time.

- **Windows:** unzip, then double-click `exam-tui.exe`. If "Windows protected your PC" appears, click **More info → Run anyway**.
- **macOS:** unzip, then double-click `exam-tui`; it opens in Terminal. If macOS says it cannot verify the app, open **System Settings → Privacy & Security** and click **Open Anyway**. You can also run `xattr -d com.apple.quarantine exam-tui` in Terminal, in the unzipped folder.
- **Linux:** `tar -xzf exam-tui-v1.0.0-linux-x86_64.tar.gz`, then run `./exam-tui` in its folder.

## What's in 1.0

- Two full practice exams: 60 questions and 120 minutes each, scored like the official Exam Guide (scaled 100–1,000, pass at 720, percent correct per domain).
- A timer that submits the exam automatically when time runs out, a question navigator, flags, and a full review with explanations.
- Attempt history, with comparisons against your previous attempt and a score trend chart.
- **Train weaknesses:** practise the questions you most often get wrong until you answer each one right twice in a row.
- **Your own exams:** save the example exam from the menu, edit it, then add it with **Add exam from file**. An `example-exam.json` is also in the download.

See the [README](https://github.com/JaavLex/Claude_Certified_Architect_TrainingExams#readme) for the keys and the exam file format.

//! Small text helpers: inline-Markdown spans and span-aware word wrapping
//! with hanging indents.

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use unicode_width::UnicodeWidthStr;

/// Render `code` and **bold** inline Markdown into styled spans.
pub fn md_spans(text: &str, base: Style) -> Vec<Span<'static>> {
    let mut out = Vec::new();
    let mut buf = String::new();
    let mut bold = false;
    let mut code = false;
    let mut chars = text.chars().peekable();

    let style_for = |bold: bool, code: bool| {
        let mut s = base;
        if bold {
            s = s.add_modifier(Modifier::BOLD);
        }
        if code {
            s = s.fg(Color::Cyan);
        }
        s
    };

    while let Some(c) = chars.next() {
        if c == '`' {
            if !buf.is_empty() {
                out.push(Span::styled(std::mem::take(&mut buf), style_for(bold, code)));
            }
            code = !code;
        } else if c == '*' && chars.peek() == Some(&'*') && !code {
            chars.next();
            if !buf.is_empty() {
                out.push(Span::styled(std::mem::take(&mut buf), style_for(bold, code)));
            }
            bold = !bold;
        } else {
            buf.push(c);
        }
    }
    if !buf.is_empty() {
        out.push(Span::styled(buf, style_for(bold, code)));
    }
    out
}

/// Word-wrap styled spans to `width`. The first line starts with `prefix`;
/// continuation lines are indented to the prefix's width.
pub fn wrap(prefix: Vec<Span<'static>>, spans: Vec<Span<'static>>, width: usize) -> Vec<Line<'static>> {
    let indent: usize = prefix.iter().map(|s| s.content.width()).sum();
    let width = width.max(indent + 8);
    let mut lines = Vec::new();
    let mut cur: Vec<Span<'static>> = prefix;
    let mut cur_w = indent;
    let mut line_has_word = false;

    // Tokenise into words (keeping each word's style) and spaces.
    for span in spans {
        let style = span.style;
        let mut word = String::new();
        let flush_word = |word: &mut String,
                          cur: &mut Vec<Span<'static>>,
                          cur_w: &mut usize,
                          lines: &mut Vec<Line<'static>>,
                          line_has_word: &mut bool| {
            if word.is_empty() {
                return;
            }
            let w = word.width();
            if *line_has_word && *cur_w + w > width {
                // Drop trailing space on the finished line.
                if let Some(last) = cur.last_mut() {
                    if last.content == " " {
                        cur.pop();
                    }
                }
                lines.push(Line::from(std::mem::take(cur)));
                cur.push(Span::raw(" ".repeat(indent)));
                *cur_w = indent;
            }
            cur.push(Span::styled(std::mem::take(word), style));
            *cur_w += w;
            *line_has_word = true;
        };
        for c in span.content.chars() {
            if c == ' ' {
                flush_word(&mut word, &mut cur, &mut cur_w, &mut lines, &mut line_has_word);
                if line_has_word && cur_w < width {
                    cur.push(Span::styled(" ", style));
                    cur_w += 1;
                }
            } else {
                word.push(c);
                // Hard-break words too long to fit on a line of their own (e.g. paths).
                if word.width() >= width - indent {
                    flush_word(&mut word, &mut cur, &mut cur_w, &mut lines, &mut line_has_word);
                }
            }
        }
        flush_word(&mut word, &mut cur, &mut cur_w, &mut lines, &mut line_has_word);
    }
    if !cur.is_empty() {
        lines.push(Line::from(cur));
    }
    lines
}

pub fn fmt_hms(secs: u64) -> String {
    format!("{}:{:02}:{:02}", secs / 3600, (secs / 60) % 60, secs % 60)
}

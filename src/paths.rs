//! Turning what people type, paste or drag into a terminal into a real path,
//! on Linux, macOS and Windows alike.
//!
//! - Surrounding quotes are removed ("Copy as path" on Windows adds `"…"`;
//!   Linux file managers often paste `'…'`).
//! - `file://` URIs are decoded (`%20` → space; `file:///C:/x` → `C:/x`).
//! - `~`, `$VAR`, `${VAR}` and `%VAR%` are expanded.
//! - On macOS and Linux, shell escapes from drag-and-drop (`My\ Exam.json`)
//!   are undone. On Windows a backslash is a separator, so it is left alone.

use std::path::{MAIN_SEPARATOR, Path, PathBuf};

/// Remove quotes, decode `file://`, expand `~` and environment variables.
pub fn clean(raw: &str) -> String {
    let mut s = raw.trim().to_string();
    for q in ['"', '\''] {
        if s.len() >= 2 && s.starts_with(q) && s.ends_with(q) {
            s = s[1..s.len() - 1].to_string();
        }
    }
    if let Some(rest) = s.strip_prefix("file://") {
        // file://host/path is rare; treat "localhost" like an empty host.
        let rest = rest.strip_prefix("localhost").unwrap_or(rest);
        let mut p = percent_decode(rest);
        // file:///C:/x → C:/x
        let b = p.as_bytes();
        if b.len() >= 3 && b[0] == b'/' && b[1].is_ascii_alphabetic() && b[2] == b':' {
            p.remove(0);
        }
        s = p;
    }
    expand_tilde(&expand_vars(&s))
}

/// Resolve user input to an existing path, trying the shell-unescaped form too.
pub fn resolve_existing(raw: &str) -> Result<PathBuf, String> {
    if raw.trim().is_empty() {
        return Err("Type or paste a path first.".into());
    }
    let cleaned = clean(raw);
    let mut candidates = vec![PathBuf::from(&cleaned)];
    if !cfg!(windows) && cleaned.contains('\\') {
        candidates.push(PathBuf::from(unescape_shell(&cleaned)));
    }
    for c in &candidates {
        if c.exists() {
            return Ok(absolute(c));
        }
    }
    Err(format!("Nothing found at {}", absolute(&candidates[0]).display()))
}

/// Resolve user input to a path that may not exist yet (for saving).
pub fn resolve_new(raw: &str) -> Result<PathBuf, String> {
    if raw.trim().is_empty() {
        return Err("Type a path first.".into());
    }
    let cleaned = clean(raw);
    let mut p = PathBuf::from(&cleaned);
    if !cfg!(windows) && cleaned.contains('\\') {
        let alt = PathBuf::from(unescape_shell(&cleaned));
        // Prefer the unescaped form when its folder exists (drag-and-drop).
        if alt.parent().is_some_and(|d| d.as_os_str().is_empty() || d.is_dir()) {
            p = alt;
        }
    }
    Ok(absolute(&p))
}

pub fn absolute(p: &Path) -> PathBuf {
    std::path::absolute(p).unwrap_or_else(|_| p.to_path_buf())
}

fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() && b[i + 1].is_ascii_hexdigit() && b[i + 2].is_ascii_hexdigit() {
            let hex = |c: u8| (c as char).to_digit(16).unwrap() as u8;
            out.push(hex(b[i + 1]) * 16 + hex(b[i + 2]));
            i += 3;
            continue;
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn expand_tilde(s: &str) -> String {
    let Some(home) = dirs::home_dir() else { return s.to_string() };
    if s == "~" {
        return home.display().to_string();
    }
    for sep in ["~/", "~\\"] {
        if let Some(rest) = s.strip_prefix(sep) {
            return home.join(rest).display().to_string();
        }
    }
    s.to_string()
}

/// Expand `$VAR`, `${VAR}` and `%VAR%` when the variable is set; leave
/// everything else untouched.
fn expand_vars(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    let is_name = |c: char| c.is_ascii_alphanumeric() || c == '_';
    while i < chars.len() {
        let c = chars[i];
        let mut replaced = false;
        if c == '$' {
            let (name, end) = if chars.get(i + 1) == Some(&'{') {
                match chars[i + 2..].iter().position(|&c| c == '}') {
                    Some(p) => (chars[i + 2..i + 2 + p].iter().collect::<String>(), i + 3 + p),
                    None => (String::new(), i),
                }
            } else {
                let n: String = chars[i + 1..].iter().take_while(|&&c| is_name(c)).collect();
                let len = n.chars().count();
                (n, i + 1 + len)
            };
            if !name.is_empty() {
                if let Ok(v) = std::env::var(&name) {
                    out.push_str(&v);
                    i = end;
                    replaced = true;
                }
            }
        } else if c == '%' {
            if let Some(p) = chars[i + 1..].iter().position(|&c| c == '%') {
                let name: String = chars[i + 1..i + 1 + p].iter().collect();
                if !name.is_empty() && name.chars().all(is_name) {
                    if let Ok(v) = std::env::var(&name) {
                        out.push_str(&v);
                        i += p + 2;
                        replaced = true;
                    }
                }
            }
        }
        if !replaced {
            out.push(c);
            i += 1;
        }
    }
    out
}

/// `My\ Exam\ \(1\).json` → `My Exam (1).json`
fn unescape_shell(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(n) = chars.next() {
                out.push(n);
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Open `dir` in the system file manager (Explorer, Finder, or the desktop's
/// default via `xdg-open`), creating it first if needed.
pub fn open_in_file_manager(dir: &Path) -> Result<(), String> {
    use std::process::{Command, Stdio};
    std::fs::create_dir_all(dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    let program = if cfg!(windows) {
        "explorer"
    } else if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    };
    // Silence the helper so its messages do not draw over the interface.
    let mut child = Command::new(program)
        .arg(dir)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("could not start a file manager ({program}: {e})"))?;
    // Reap the helper when it exits instead of leaving it waiting.
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(())
}

/// Read a text file, accepting UTF-8 (with or without BOM) and UTF-16 with BOM.
/// Notepad and PowerShell on Windows commonly write the latter two.
pub fn read_text(path: &Path) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    if let Some(rest) = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]) {
        return String::from_utf8(rest.to_vec()).map_err(|_| "file is not valid UTF-8 text".into());
    }
    let utf16 = |rest: &[u8], le: bool| {
        let units: Vec<u16> = rest
            .chunks_exact(2)
            .map(|c| if le { u16::from_le_bytes([c[0], c[1]]) } else { u16::from_be_bytes([c[0], c[1]]) })
            .collect();
        String::from_utf16(&units).map_err(|_| "file is not valid UTF-16 text".to_string())
    };
    if let Some(rest) = bytes.strip_prefix(&[0xFF, 0xFE]) {
        return utf16(rest, true);
    }
    if let Some(rest) = bytes.strip_prefix(&[0xFE, 0xFF]) {
        return utf16(rest, false);
    }
    String::from_utf8(bytes).map_err(|_| "file is not UTF-8 text".into())
}

/// A file name stem that is valid on Windows, macOS and Linux.
pub fn safe_stem(s: &str) -> String {
    let mut out: String = s
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' { c } else { '-' })
        .take(64)
        .collect();
    // Windows rejects names ending in a dot or space.
    while out.ends_with('.') || out.ends_with('-') {
        out.pop();
    }
    while out.starts_with('.') || out.starts_with('-') {
        out.remove(0);
    }
    if out.is_empty() {
        out = "exam".into();
    }
    const RESERVED: [&str; 22] = [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9",
        "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];
    let base = out.split('.').next().unwrap_or("").to_ascii_uppercase();
    if RESERVED.contains(&base.as_str()) {
        out = format!("exam-{out}");
    }
    out
}

/// `dir/stem.ext`, or `dir/stem-2.ext`, `dir/stem-3.ext`… if taken.
pub fn unique_path(dir: &Path, stem: &str, ext: &str) -> PathBuf {
    let first = dir.join(format!("{stem}.{ext}"));
    if !first.exists() {
        return first;
    }
    (2..)
        .map(|n| dir.join(format!("{stem}-{n}.{ext}")))
        .find(|p| !p.exists())
        .unwrap()
}

/// Tab completion for the path box. Returns the completed text and the
/// matching names (to show when there is more than one).
pub fn complete(input: &str) -> (String, Vec<String>) {
    let is_sep = |c: char| c == '/' || (cfg!(windows) && c == '\\');
    let split = input.rfind(is_sep).map(|i| i + 1).unwrap_or(0);
    let (dir_part, prefix) = input.split_at(split);
    let dir = if dir_part.is_empty() { PathBuf::from(".") } else { PathBuf::from(clean(dir_part)) };
    // Windows and macOS file systems are case-insensitive by default.
    let fold = |s: &str| if cfg!(any(windows, target_os = "macos")) { s.to_lowercase() } else { s.to_string() };
    let want = fold(prefix);

    let mut matches: Vec<(String, bool)> = std::fs::read_dir(&dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            let hidden = name.starts_with('.') && !prefix.starts_with('.');
            (!hidden && fold(&name).starts_with(&want)).then(|| {
                let is_dir = e.path().is_dir();
                (name, is_dir)
            })
        })
        .collect();
    matches.sort();
    if matches.is_empty() {
        return (input.to_string(), vec![]);
    }
    let names: Vec<String> = matches.iter().map(|(n, d)| if *d { format!("{n}{MAIN_SEPARATOR}") } else { n.clone() }).collect();
    if matches.len() == 1 {
        return (format!("{dir_part}{}", names[0]), vec![]);
    }
    // Longest common prefix (case-folded), using the first match's spelling.
    // Every match starts with `prefix`, so this never shortens what was typed.
    let first: Vec<char> = matches[0].0.chars().collect();
    let mut len = first.len();
    for (m, _) in &matches[1..] {
        let common = first
            .iter()
            .zip(m.chars())
            .take_while(|(a, b)| fold(&a.to_string()) == fold(&b.to_string()))
            .count();
        len = len.min(common);
    }
    let completed: String = first[..len].iter().collect();
    (format!("{dir_part}{completed}"), names)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_quotes_and_uris() {
        assert_eq!(clean("  \"/tmp/a b.json\" "), "/tmp/a b.json");
        assert_eq!(clean("'/tmp/a b.json'"), "/tmp/a b.json");
        assert_eq!(clean("file:///tmp/my%20exam.json"), "/tmp/my exam.json");
        assert_eq!(clean("file:///C:/Users/me/x.json"), "C:/Users/me/x.json");
        assert_eq!(clean("50%"), "50%");
        assert_eq!(percent_decode("caf%C3%A9%2"), "café%2");
    }

    #[test]
    fn expands_home_and_vars() {
        let home = dirs::home_dir().unwrap();
        assert_eq!(PathBuf::from(clean("~/x.json")), home.join("x.json"));
        unsafe { std::env::set_var("EXAM_TUI_TEST_DIR", "/data") };
        assert_eq!(clean("$EXAM_TUI_TEST_DIR/x.json"), "/data/x.json");
        assert_eq!(clean("${EXAM_TUI_TEST_DIR}/x.json"), "/data/x.json");
        assert_eq!(clean("%EXAM_TUI_TEST_DIR%/x.json"), "/data/x.json");
        assert_eq!(clean("$NOT_SET_EXAM_TUI/x"), "$NOT_SET_EXAM_TUI/x");
    }

    #[cfg(not(windows))]
    #[test]
    fn undoes_drag_and_drop_escapes() {
        let dir = std::env::temp_dir().join("exam tui (test)");
        std::fs::create_dir_all(&dir).unwrap();
        let f = dir.join("my exam.json");
        std::fs::write(&f, "{}").unwrap();
        let typed = f.display().to_string().replace(' ', "\\ ").replace('(', "\\(").replace(')', "\\)");
        assert_eq!(resolve_existing(&typed).unwrap(), f);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn safe_names() {
        assert_eq!(safe_stem("my exam: v2?"), "my-exam--v2");
        assert_eq!(safe_stem("CON"), "exam-CON");
        assert_eq!(safe_stem("..."), "exam");
        assert_eq!(safe_stem("ccarf-practice_3"), "ccarf-practice_3");
    }

    #[test]
    fn reads_bom_and_utf16() {
        let dir = std::env::temp_dir();
        let p = dir.join("exam-tui-bom-test.json");
        std::fs::write(&p, [&[0xEF, 0xBB, 0xBF][..], b"{\"a\":1}"].concat()).unwrap();
        assert_eq!(read_text(&p).unwrap(), "{\"a\":1}");
        let mut v = vec![0xFF, 0xFE];
        for u in "{\"é\":1}".encode_utf16() {
            v.extend(u.to_le_bytes());
        }
        std::fs::write(&p, v).unwrap();
        assert_eq!(read_text(&p).unwrap(), "{\"é\":1}");
        std::fs::remove_file(&p).unwrap();
    }
}

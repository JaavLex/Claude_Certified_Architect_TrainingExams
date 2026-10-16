//! Exam model, JSON loading and validation.
//!
//! The two bundled practice exams are embedded at compile time. Custom exams
//! are loaded at runtime from the user's exam folder and from paths given on
//! the command line. See `schema/exam.schema.json` for the file format.

use serde::Deserialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Time limit per the CCAR-F Exam Guide ("Exam Details at a Glance"),
/// used when an exam file does not set `time_limit_minutes`.
pub const DEFAULT_MINUTES: u64 = 120;
/// Number of items on the real exam; the bundled exams must match it.
#[cfg(test)]
pub const OFFICIAL_ITEMS: usize = 60;
/// Most options a question can have (they are picked with keys 1–9).
pub const MAX_OPTIONS: usize = 9;

const BUILTIN: [(&str, &str); 2] = [
    ("practice-exam-1.json", include_str!("../exams/practice-exam-1.json")),
    ("practice-exam-2.json", include_str!("../exams/practice-exam-2.json")),
];

#[derive(Debug, Clone, Deserialize)]
pub struct ExamOption {
    pub key: char,
    pub text: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Question {
    pub number: usize,
    #[serde(default)]
    pub scenario: String,
    #[serde(default)]
    pub domain: String,
    #[serde(default = "one")]
    pub select: usize,
    pub text: String,
    pub options: Vec<ExamOption>,
    pub answer: Vec<char>,
    #[serde(default)]
    pub why: String,
}

fn one() -> usize {
    1
}

impl Question {
    pub fn is_correct(&self, chosen: &[char]) -> bool {
        let mut a: Vec<char> = chosen.to_vec();
        a.sort_unstable();
        a.dedup();
        a == self.answer
    }

    /// Domain used for the per-domain breakdown: "D4/1" → "D4"; empty → "Other".
    pub fn primary_domain(&self) -> String {
        match self.domain.split('/').next().map(str::trim) {
            Some(d) if !d.is_empty() => d.to_string(),
            _ => "Other".to_string(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Domain {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Exam {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_minutes")]
    pub time_limit_minutes: u64,
    #[serde(default)]
    pub domains: Vec<Domain>,
    pub questions: Vec<Question>,
    /// Where the exam came from ("built-in" or a file path). Not part of the file.
    #[serde(skip)]
    pub source: String,
}

fn default_minutes() -> u64 {
    DEFAULT_MINUTES
}

impl Exam {
    pub fn time_limit_secs(&self) -> u64 {
        self.time_limit_minutes * 60
    }

    pub fn domain_name(&self, id: &str) -> String {
        self.domains
            .iter()
            .find(|d| d.id == id)
            .map(|d| d.name.clone())
            .unwrap_or_else(|| id.to_string())
    }

    pub fn short_title(&self) -> String {
        self.title.split(" — ").next().unwrap_or(&self.title).to_string()
    }

    pub fn is_builtin(&self) -> bool {
        self.source == "built-in"
    }
}

/// Parse and validate one exam document.
pub fn parse(json: &str, source: &str) -> Result<Exam, String> {
    let mut exam: Exam = serde_json::from_str(json).map_err(|e| e.to_string())?;
    exam.source = source.to_string();
    for q in &mut exam.questions {
        q.answer.sort_unstable();
    }
    validate(&exam)?;
    Ok(exam)
}

pub fn validate(e: &Exam) -> Result<(), String> {
    if e.id.trim().is_empty() {
        return Err("\"id\" must not be empty".into());
    }
    if e.title.trim().is_empty() {
        return Err("\"title\" must not be empty".into());
    }
    if e.time_limit_minutes == 0 {
        return Err("\"time_limit_minutes\" must be at least 1".into());
    }
    if e.questions.is_empty() {
        return Err("exam has no questions".into());
    }
    let mut numbers = HashSet::new();
    for q in &e.questions {
        let n = q.number;
        if !numbers.insert(n) {
            return Err(format!("question number {n} is used more than once"));
        }
        if q.text.trim().is_empty() {
            return Err(format!("Q{n}: \"text\" is empty"));
        }
        if q.options.len() < 2 || q.options.len() > MAX_OPTIONS {
            return Err(format!("Q{n}: needs 2–{MAX_OPTIONS} options, has {}", q.options.len()));
        }
        let mut keys = HashSet::new();
        for o in &q.options {
            if !o.key.is_ascii_uppercase() {
                return Err(format!("Q{n}: option key '{}' must be a capital letter A–Z", o.key));
            }
            if !keys.insert(o.key) {
                return Err(format!("Q{n}: option key '{}' is used more than once", o.key));
            }
        }
        if q.select == 0 || q.select >= q.options.len() {
            return Err(format!("Q{n}: \"select\" must be between 1 and {}", q.options.len() - 1));
        }
        let mut uniq = q.answer.clone();
        uniq.dedup();
        if uniq.len() != q.answer.len() {
            return Err(format!("Q{n}: \"answer\" repeats a key"));
        }
        if q.answer.len() != q.select {
            return Err(format!(
                "Q{n}: \"select\" is {} but \"answer\" has {} key(s)",
                q.select,
                q.answer.len()
            ));
        }
        if let Some(bad) = q.answer.iter().find(|a| !keys.contains(a)) {
            return Err(format!("Q{n}: answer '{bad}' is not one of the option keys"));
        }
    }
    Ok(())
}

/// The folder custom exams are loaded from automatically.
pub fn user_exam_dir() -> PathBuf {
    dirs::data_dir().unwrap_or_else(|| PathBuf::from(".")).join("exam-tui").join("exams")
}

/// Create the exam folder if it does not exist yet (e.g. on first run of a
/// downloaded executable), so the path shown to the user is a real folder.
pub fn ensure_user_exam_dir() -> Result<PathBuf, String> {
    let dir = user_exam_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    Ok(dir)
}

pub struct Loaded {
    pub exams: Vec<Exam>,
    /// Files that could not be loaded, with the reason.
    pub errors: Vec<String>,
}

/// Load the built-in exams, then custom exams from the user exam folder and
/// from `extra` paths (files or folders). A custom exam with the same `id` as
/// an earlier one replaces it.
pub fn load_all(extra: &[PathBuf]) -> Loaded {
    let mut out = Loaded { exams: Vec::new(), errors: Vec::new() };
    for (name, json) in BUILTIN {
        match parse(json, "built-in") {
            Ok(e) => out.exams.push(e),
            Err(err) => out.errors.push(format!("built-in {name}: {err}")),
        }
    }

    let mut files: Vec<PathBuf> = json_files_in(&user_exam_dir());
    for p in extra {
        if p.is_dir() {
            files.extend(json_files_in(p));
        } else {
            files.push(p.clone());
        }
    }

    for path in files {
        let shown = path.display().to_string();
        let result = crate::paths::read_text(&path).and_then(|s| parse(&s, &shown));
        match result {
            Ok(e) => match out.exams.iter().position(|x| x.id == e.id) {
                Some(i) => out.exams[i] = e,
                None => out.exams.push(e),
            },
            Err(err) => out.errors.push(format!("{shown}: {err}")),
        }
    }
    out
}

/// The example exam, offered to users as a starting point for their own.
const EXAMPLE_JSON: &str = include_str!("../examples/custom-exam.json");
const SCHEMA_URL: &str =
    "https://raw.githubusercontent.com/JaavLex/Claude_Certified_Architect_TrainingExams/main/schema/exam.schema.json";

pub struct Imported {
    pub id: String,
    pub title: String,
    pub dest: PathBuf,
    /// True when an exam with the same id was already in the exam folder.
    pub updated: bool,
}

/// Check an exam file and copy it into the user exam folder so it is loaded
/// on every start. An exam already there with the same id is replaced.
pub fn import_file(src: &Path, builtin_ids: &[String]) -> Result<Imported, String> {
    let text = crate::paths::read_text(src)?;
    let exam = parse(&text, &src.display().to_string())?;
    if builtin_ids.contains(&exam.id) {
        return Err(format!(
            "its id \"{}\" is used by a built-in exam. Change \"id\" in the file to something else.",
            exam.id
        ));
    }
    let dir = user_exam_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;

    let same_file = |a: &Path, b: &Path| match (a.canonicalize(), b.canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    };
    // Reuse the file of an exam with the same id, so it is replaced, not duplicated.
    let existing = json_files_in(&dir).into_iter().find(|p| {
        crate::paths::read_text(p)
            .ok()
            .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
            .is_some_and(|v| v["id"].as_str() == Some(exam.id.as_str()))
    });
    let updated = existing.is_some();
    let dest = existing
        .unwrap_or_else(|| crate::paths::unique_path(&dir, &crate::paths::safe_stem(&exam.id), "json"));
    if !same_file(src, &dest) {
        // Write as UTF-8 without BOM, via a temporary file so a failure never
        // leaves a half-written exam behind.
        let tmp = dest.with_extension("json.tmp");
        std::fs::write(&tmp, text.as_bytes()).map_err(|e| format!("cannot write {}: {e}", tmp.display()))?;
        std::fs::rename(&tmp, &dest).map_err(|e| format!("cannot write {}: {e}", dest.display()))?;
    }
    Ok(Imported { id: exam.id, title: exam.title, dest, updated })
}

/// Import a file, or every `.json` file in a folder.
pub fn import_path(src: &Path, builtin_ids: &[String]) -> Vec<(PathBuf, Result<Imported, String>)> {
    let files = if src.is_dir() { json_files_in(src) } else { vec![src.to_path_buf()] };
    files.into_iter().map(|f| (f.clone(), import_file(&f, builtin_ids))).collect()
}

/// Where the example is saved by default: the Downloads folder, else home.
pub fn default_example_path() -> PathBuf {
    let dir = dirs::download_dir()
        .filter(|d| d.is_dir())
        .or_else(dirs::home_dir)
        .unwrap_or_else(|| PathBuf::from("."));
    crate::paths::unique_path(&dir, "exam-tui-example", "json")
}

/// Write the example exam to `target` (a file path, or a folder to put it in).
/// Never overwrites an existing file.
pub fn save_example(target: &Path) -> Result<PathBuf, String> {
    let dest = if target.is_dir() {
        crate::paths::unique_path(target, "exam-tui-example", "json")
    } else if target.extension().is_none() {
        target.with_extension("json")
    } else {
        target.to_path_buf()
    };
    if dest.exists() {
        return Err(format!("{} already exists. Choose another name.", dest.display()));
    }
    match dest.parent() {
        Some(d) if !d.as_os_str().is_empty() && !d.is_dir() => {
            return Err(format!("the folder {} does not exist.", d.display()));
        }
        _ => {}
    }
    // Point "$schema" at the published schema so editors can check the file
    // wherever it is saved.
    let json = EXAMPLE_JSON.replace("\"../schema/exam.schema.json\"", &format!("\"{SCHEMA_URL}\""));
    std::fs::write(&dest, json).map_err(|e| format!("cannot write {}: {e}", dest.display()))?;
    Ok(dest)
}

fn json_files_in(dir: &Path) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file() && p.extension().is_some_and(|x| x.eq_ignore_ascii_case("json")))
        .collect();
    v.sort();
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_exams_follow_exam_guide() {
        let exams: Vec<Exam> =
            BUILTIN.iter().map(|(_, json)| parse(json, "built-in").unwrap()).collect();
        for e in &exams {
            assert_eq!(e.questions.len(), OFFICIAL_ITEMS, "{}", e.id);
            assert_eq!(e.time_limit_minutes, DEFAULT_MINUTES, "{}", e.id);
            for q in &e.questions {
                assert_eq!(q.options.len(), 4, "{} Q{}", e.id, q.number);
            }
        }
        assert_eq!(exams[0].questions[11].answer, vec!['A', 'C']);
        assert_eq!(exams[1].questions[10].answer, vec!['A', 'B']);
    }

    #[test]
    fn example_exam_is_valid() {
        let json = std::fs::read_to_string("examples/custom-exam.json").unwrap();
        parse(&json, "example").unwrap();
    }

    #[test]
    fn example_round_trip() {
        let dir = std::env::temp_dir().join(format!("exam-tui-example-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let saved = save_example(&dir).unwrap();
        assert_eq!(saved.file_name().unwrap(), "exam-tui-example.json");
        let text = std::fs::read_to_string(&saved).unwrap();
        assert!(text.contains(SCHEMA_URL));
        parse(&text, "saved").unwrap();
        // Never overwrites; a folder gets a fresh name.
        assert!(save_example(&saved).is_err());
        assert_eq!(save_example(&dir).unwrap().file_name().unwrap(), "exam-tui-example-2.json");
        assert!(save_example(&dir.join("missing").join("x.json")).is_err());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn rejects_bad_answers() {
        let base = r#"{"id":"x","title":"X","questions":[{"number":1,"text":"Q","select":SEL,
            "options":[{"key":"A","text":"a"},{"key":"B","text":"b"},{"key":"C","text":"c"}],
            "answer":ANS}]}"#;
        let doc = |sel: &str, ans: &str| base.replace("SEL", sel).replace("ANS", ans);
        assert!(parse(&doc("1", r#"["A"]"#), "t").is_ok());
        assert!(parse(&doc("2", r#"["C","A"]"#), "t").is_ok());
        assert!(parse(&doc("1", r#"["A","B"]"#), "t").is_err());
        assert!(parse(&doc("1", r#"["Z"]"#), "t").is_err());
        assert!(parse(&doc("3", r#"["A","B","C"]"#), "t").is_err());
        assert!(parse(&doc("2", r#"["A","A"]"#), "t").is_err());
    }
}

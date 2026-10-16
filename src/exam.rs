//! Exam model and Markdown parser for the practice-exam files.

/// Number of items and time limit, per the CCAR-F Exam Guide ("Exam Details at a Glance").
pub const EXAM_ITEMS: usize = 60;
pub const EXAM_MINUTES: u64 = 120;

pub struct ExamSource {
    pub id: &'static str,
    pub markdown: &'static str,
}

pub const SOURCES: [ExamSource; 2] = [
    ExamSource {
        id: "exam1",
        markdown: include_str!("../exams/12-Practice-Exam-1.md"),
    },
    ExamSource {
        id: "exam2",
        markdown: include_str!("../exams/13-Practice-Exam-2.md"),
    },
];

#[derive(Debug, Clone)]
pub struct Question {
    pub number: usize,
    pub scenario: String,
    pub select: usize,
    pub text: String,
    pub options: Vec<(char, String)>,
    pub answer: Vec<char>,
    pub domain: String,
    pub why: String,
}

impl Question {
    pub fn is_correct(&self, chosen: &[char]) -> bool {
        let mut a: Vec<char> = chosen.to_vec();
        a.sort_unstable();
        a.dedup();
        a == self.answer
    }

    /// Primary domain used for the per-domain breakdown ("D4/1" → "D4").
    pub fn primary_domain(&self) -> String {
        let d = self.domain.trim();
        match d.split('/').next() {
            Some(first) if first.starts_with('D') => first.to_string(),
            _ => "Misc".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Exam {
    pub id: &'static str,
    pub title: String,
    pub questions: Vec<Question>,
}

pub fn load_all() -> Result<Vec<Exam>, String> {
    SOURCES.iter().map(|s| parse(s.id, s.markdown)).collect()
}

pub fn parse(id: &'static str, md: &str) -> Result<Exam, String> {
    let mut title = String::new();
    let mut scenario = String::new();
    let mut questions: Vec<Question> = Vec::new();
    let mut in_key = false;

    for raw in md.lines() {
        let line = raw.trim_end();
        if title.is_empty() {
            if let Some(t) = line.strip_prefix("# ") {
                title = t.trim().to_string();
                continue;
            }
        }
        if line.starts_with("# ") && line.contains("ANSWER KEY") {
            in_key = true;
            continue;
        }

        if in_key {
            if let Some((num, ans, dom, why)) = parse_key_row(line) {
                if let Some(q) = questions.iter_mut().find(|q| q.number == num) {
                    q.answer = ans;
                    q.domain = dom;
                    q.why = why;
                }
            }
            continue;
        }

        if let Some(s) = line.strip_prefix("## ") {
            scenario = s.trim().to_string();
            continue;
        }

        if let Some((num, select, text)) = parse_question_head(line) {
            questions.push(Question {
                number: num,
                scenario: scenario.clone(),
                select,
                text,
                options: Vec::new(),
                answer: Vec::new(),
                domain: String::new(),
                why: String::new(),
            });
            continue;
        }

        if let Some(q) = questions.last_mut() {
            if let Some(opt) = parse_option(line) {
                q.options.push(opt);
            } else if !line.trim().is_empty() && q.options.is_empty() && !line.starts_with("---") {
                // Question text continued on following line(s).
                q.text.push(' ');
                q.text.push_str(line.trim());
            }
        }
    }

    if questions.len() != EXAM_ITEMS {
        return Err(format!("{id}: expected {EXAM_ITEMS} questions, parsed {}", questions.len()));
    }
    for q in &questions {
        if q.answer.is_empty() {
            return Err(format!("{id}: Q{} has no answer in the key", q.number));
        }
        if q.answer.len() != q.select {
            return Err(format!(
                "{id}: Q{} says select {} but key has {} answers",
                q.number,
                q.select,
                q.answer.len()
            ));
        }
    }
    Ok(Exam { id, title, questions })
}

/// `**12. (Select 2)** Question text…`
fn parse_question_head(line: &str) -> Option<(usize, usize, String)> {
    let rest = line.strip_prefix("**")?;
    let dot = rest.find(". (Select ")?;
    let num: usize = rest[..dot].trim().parse().ok()?;
    let after = &rest[dot + ". (Select ".len()..];
    let close = after.find(")**")?;
    let select: usize = after[..close].trim().parse().ok()?;
    let text = after[close + 3..].trim().to_string();
    Some((num, select, text))
}

/// `- A. Option text`
fn parse_option(line: &str) -> Option<(char, String)> {
    let rest = line.strip_prefix("- ")?;
    let mut chars = rest.chars();
    let letter = chars.next()?;
    if !letter.is_ascii_uppercase() {
        return None;
    }
    let rest = chars.as_str().strip_prefix(". ")?;
    Some((letter, rest.trim().to_string()))
}

/// `| 12 | **A, C** | D5 | Why… |`
fn parse_key_row(line: &str) -> Option<(usize, Vec<char>, String, String)> {
    let inner = line.trim().strip_prefix('|')?.strip_suffix('|')?;
    let mut cells = inner.splitn(4, '|').map(str::trim);
    let num: usize = cells.next()?.parse().ok()?;
    let ans_cell = cells.next()?.replace("**", "");
    let mut ans: Vec<char> = ans_cell
        .split(|c: char| c == ',' || c.is_whitespace())
        .filter_map(|s| {
            let s = s.trim();
            (s.len() == 1).then(|| s.chars().next().unwrap())
        })
        .filter(char::is_ascii_uppercase)
        .collect();
    ans.sort_unstable();
    let dom = cells.next()?.to_string();
    let why = cells.next().unwrap_or("").to_string();
    Some((num, ans, dom, why))
}

pub fn domain_name(d: &str) -> &'static str {
    match d {
        "D1" => "Agentic Architecture & Orchestration",
        "D2" => "Tool Design & MCP Integration",
        "D3" => "Claude Code Configuration & Workflows",
        "D4" => "Prompt Engineering & Structured Output",
        "D5" => "Context Management & Reliability",
        _ => "Miscellaneous / Scope",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_exams_parse() {
        let exams = load_all().unwrap();
        for e in &exams {
            assert_eq!(e.questions.len(), EXAM_ITEMS);
            for q in &e.questions {
                assert_eq!(q.options.len(), 4, "{} Q{}", e.id, q.number);
            }
        }
        assert_eq!(exams[0].questions[11].answer, vec!['A', 'C']);
        assert_eq!(exams[1].questions[10].answer, vec!['A', 'B']);
    }
}

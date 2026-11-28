//! 위키 공통: 위키 루트, 설정, frontmatter, 링크, 페이지 목록, 요약.
use fancy_regex::Regex;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

pub const SKIP_DIRS: &[&str] = &[".git", ".obsidian", ".trash", ".claude", ".githooks", ".yw-wiki", "tools", "_templates"];
/// wiki/ 안에서 템플릿을 따르지 않는 생성 · 기록 파일
pub const SPECIAL: &[&str] = &["index.md", "log.md"];
/// type → 폴더 (wiki/ 기준)
pub const TYPE_DIRS: &[(&str, &str)] = &[
    ("entity", "entities"), ("concept", "concepts"), ("source", "sources"), ("synthesis", "synthesis"),
    ("decision", "decisions"), ("achievement", "career"), ("guide", "guides"),
];

pub fn type_dir(kind: &str) -> Option<&'static str> {
    TYPE_DIRS.iter().find(|(k, _)| *k == kind).map(|(_, d)| *d)
}

pub fn re(cell: &'static OnceLock<Regex>, pat: &str) -> &'static Regex {
    cell.get_or_init(|| Regex::new(pat).expect("정규식"))
}

macro_rules! regex {
    ($pat:expr) => {{
        static CELL: std::sync::OnceLock<fancy_regex::Regex> = std::sync::OnceLock::new();
        $crate::wiki::re(&CELL, $pat)
    }};
}
pub(crate) use regex;

/// 정규식 치환 (모든 일치)
pub fn sub(re: &Regex, text: &str, rep: &str) -> String {
    re.replace_all(text, rep).into_owned()
}

/// 인자가 위키 루트(wiki.toml이 있는 폴더)면 그 경로. 같은 이름 폴더(review 등)는 루트가 아니다.
pub fn wiki_root_arg(arg: &str) -> Option<PathBuf> {
    let p = expand(arg);
    if p.join("wiki.toml").is_file() { p.canonicalize().ok() } else { None }
}

/// 지금 폴더에서 위로 올라가며 wiki.toml이 있는 폴더를 찾는다.
pub fn find_root() -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut cur: &Path = &cwd;
    loop {
        if cur.join("wiki.toml").is_file() {
            return cur.to_path_buf();
        }
        match cur.parent() {
            Some(p) => cur = p,
            None => return cwd,
        }
    }
}

/// 첫 인자(옵션 제외)가 위키 루트면 그 위키를, 아니면 지금 폴더가 속한 위키. 위키 인자를 뺀 나머지도 돌려준다.
pub fn root_and_rest(args: &[String]) -> (PathBuf, Vec<String>) {
    if let Some(first) = args.iter().find(|a| !a.starts_with('-')) {
        if let Some(root) = wiki_root_arg(first) {
            let mut rest = args.to_vec();
            let i = rest.iter().position(|a| a == first).unwrap();
            rest.remove(i);
            return (root, rest);
        }
    }
    (find_root(), args.to_vec())
}

pub fn home() -> PathBuf {
    std::env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("/"))
}

pub fn expand(p: &str) -> PathBuf {
    if p == "~" {
        return home();
    }
    if let Some(rest) = p.strip_prefix("~/") {
        return home().join(rest);
    }
    PathBuf::from(p)
}

/// 경로를 절대 경로로 (있으면 심볼릭 링크까지 풂)
pub fn resolve(p: &Path) -> PathBuf {
    if let Ok(c) = p.canonicalize() {
        return c;
    }
    let abs = if p.is_absolute() { p.to_path_buf() } else { std::env::current_dir().unwrap_or_default().join(p) };
    // 없는 경로: 있는 조상까지 풀고 나머지를 붙인다
    let mut tail = Vec::new();
    let mut cur = abs.clone();
    loop {
        if let Ok(c) = cur.canonicalize() {
            let mut out = c;
            for t in tail.iter().rev() {
                out.push(t);
            }
            return normalize(&out);
        }
        match (cur.file_name().map(|s| s.to_os_string()), cur.parent().map(|p| p.to_path_buf())) {
            (Some(name), Some(parent)) => {
                tail.push(name);
                cur = parent;
            }
            _ => return normalize(&abs),
        }
    }
}

fn normalize(p: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for c in p.components() {
        match c {
            std::path::Component::ParentDir => {
                out.pop();
            }
            std::path::Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}

pub fn read(p: &Path) -> String {
    std::fs::read_to_string(p).unwrap_or_default()
}

pub fn config(root: &Path) -> toml::Table {
    read(&root.join("wiki.toml")).parse::<toml::Table>().unwrap_or_default()
}

pub fn tget<'a>(t: &'a toml::Table, path: &[&str]) -> Option<&'a toml::Value> {
    let mut cur = t.get(path[0])?;
    for k in &path[1..] {
        cur = cur.as_table()?.get(*k)?;
    }
    Some(cur)
}

pub fn tstr(t: &toml::Table, path: &[&str]) -> Option<String> {
    tget(t, path).and_then(|v| v.as_str()).map(String::from)
}

pub fn tlist(t: &toml::Table, path: &[&str]) -> Vec<String> {
    tget(t, path)
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect())
        .unwrap_or_default()
}

/// frontmatter 값: 스칼라 또는 목록
#[derive(Clone, Debug, PartialEq)]
pub enum Val {
    S(String),
    L(Vec<String>),
}

impl Val {
    pub fn truthy(&self) -> bool {
        match self {
            Val::S(s) => !s.is_empty(),
            Val::L(l) => !l.is_empty(),
        }
    }
    /// Python str(값)에 가까운 문자열
    pub fn text(&self) -> String {
        match self {
            Val::S(s) => s.clone(),
            Val::L(l) => format!("[{}]", l.iter().map(|x| py_repr(x)).collect::<Vec<_>>().join(", ")),
        }
    }
}

pub type Meta = BTreeMap<String, Val>;

pub fn mget(meta: &Meta, key: &str) -> String {
    meta.get(key).map(|v| v.text()).unwrap_or_default()
}

pub fn mtruthy(meta: &Meta, key: &str) -> bool {
    meta.get(key).map(|v| v.truthy()).unwrap_or(false)
}

/// Python repr(str)
pub fn py_repr(s: &str) -> String {
    if s.contains('\'') && !s.contains('"') {
        format!("\"{s}\"")
    } else {
        format!("'{}'", s.replace('\\', "\\\\").replace('\'', "\\'"))
    }
}

pub fn py_list(items: &[String]) -> String {
    format!("[{}]", items.iter().map(|x| py_repr(x)).collect::<Vec<_>>().join(", "))
}

fn value(raw: &str) -> Val {
    let raw = raw.trim();
    if raw.starts_with('\'') || raw.starts_with('"') {
        let q = raw.chars().next().unwrap();
        let inner = &raw[1..];
        return Val::S(match inner.find(q) {
            Some(end) if end > 0 => inner[..end].to_string(),
            _ => inner.to_string(),
        });
    }
    let raw = sub(regex!(r"\s+#.*$"), raw, "");
    let raw = raw.trim();
    if raw.starts_with('#') {
        return Val::S(String::new());
    }
    if raw.starts_with('[') && raw.ends_with(']') {
        let inner = raw[1..raw.len() - 1].trim();
        if inner.is_empty() {
            return Val::L(vec![]);
        }
        // Python: 공백만인 조각은 빼고, 따옴표를 벗긴다
        let items = regex!(r#""[^"]*"|'[^']*'|[^,]+"#)
            .find_iter(inner)
            .filter_map(|m| m.ok())
            .filter(|m| !m.as_str().trim().is_empty())
            .map(|m| m.as_str().trim().trim_matches(|c| c == '"' || c == '\'').to_string())
            .collect::<Vec<_>>();
        return Val::L(items);
    }
    Val::S(raw.to_string())
}

/// (meta, body). 스칼라 · 인라인 목록 · 한 단계 블록 목록만 읽는다. frontmatter가 없으면 meta는 None.
pub fn frontmatter(text: &str) -> (Option<Meta>, String) {
    if !text.starts_with("---\n") {
        return (None, text.to_string());
    }
    let Some(end) = text[4..].find("\n---").map(|i| i + 4) else {
        return (None, text.to_string());
    };
    let mut meta = Meta::new();
    let mut key: Option<String> = None;
    for line in text[4..end].lines() {
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            continue;
        }
        if let Ok(Some(c)) = regex!(r"^\s+-\s*(.*)$").captures(line) {
            if let Some(k) = &key {
                let item = match value(c.get(1).unwrap().as_str()) {
                    Val::S(s) => s,
                    Val::L(l) => py_list(&l),
                };
                match meta.get_mut(k) {
                    Some(Val::L(l)) => l.push(item),
                    _ => {
                        meta.insert(k.clone(), Val::L(vec![item]));
                    }
                }
                continue;
            }
        }
        if let Some((k, v)) = line.split_once(':') {
            if !line.starts_with(' ') {
                let k = k.trim().to_string();
                meta.insert(k.clone(), value(v));
                key = Some(k);
            }
        }
    }
    let body = &text[end + 4..];
    (Some(meta), body.trim_start_matches('\n').to_string())
}

/// 링크 · 자리표시를 볼 때 코드 블록 · 인라인 코드 · HTML 주석은 뺀다.
pub fn strip_code(text: &str) -> String {
    let t = sub(regex!(r"(?s)<!--.*?-->"), text, "");
    let t = sub(regex!(r"(?s)```.*?```"), &t, "");
    sub(regex!(r"`[^`\n]*`"), &t, "")
}

pub fn links(text: &str) -> Vec<String> {
    let mut text = text.to_string();
    if text.starts_with("---\n") {
        if let Some(end) = text[4..].find("\n---").map(|i| i + 4) {
            // frontmatter 주석(# …) 속 예시는 링크가 아니다
            let head: Vec<String> = text[..end]
                .lines()
                .map(|l| sub(regex!(r#"(^|\s)#(?![^"']*["']\s*$).*$"#), l, ""))
                .collect();
            text = head.join("\n") + &text[end..];
        }
    }
    regex!(r"\[\[([^\]|#^]+)")
        .captures_iter(&strip_code(&text))
        .filter_map(|c| c.ok())
        .map(|c| {
            let m = c.get(1).unwrap().as_str().trim();
            let last = m.rsplit('/').next().unwrap_or(m);
            last.strip_suffix(".md").unwrap_or(last).to_string()
        })
        .collect()
}

pub fn rel_parts(rel: &Path) -> Vec<String> {
    rel.components().map(|c| c.as_os_str().to_string_lossy().into_owned()).collect()
}

pub fn stem(p: &Path) -> String {
    p.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default()
}

pub fn name(p: &Path) -> String {
    p.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default()
}

pub fn is_md(p: &Path) -> bool {
    p.extension().map(|e| e == "md").unwrap_or(false)
}

/// 폴더 아래 파일 전부 (심볼릭 링크 폴더는 따라가지 않는다), 경로 순
pub fn walk(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    fn rec(d: &Path, out: &mut Vec<PathBuf>) {
        let Ok(rd) = std::fs::read_dir(d) else { return };
        for e in rd.flatten() {
            let p = e.path();
            let Ok(ft) = e.file_type() else { continue };
            if ft.is_dir() {
                rec(&p, out);
            } else if ft.is_file() || (ft.is_symlink() && p.is_file()) {
                out.push(p);
            }
        }
    }
    rec(dir, &mut out);
    out.sort();
    out
}

/// wiki/ 아래 페이지 (path, rel). index · log는 뺀다.
pub fn pages(root: &Path) -> Vec<(PathBuf, PathBuf)> {
    walk(&root.join("wiki"))
        .into_iter()
        .filter(|p| is_md(p))
        .filter_map(|p| {
            let rel = p.strip_prefix(root).ok()?.to_path_buf();
            let parts = rel_parts(&rel);
            if parts.len() == 2 && SPECIAL.contains(&parts[1].as_str()) {
                return None;
            }
            Some((p, rel))
        })
        .collect()
}

/// 링크 대상이 될 수 있는 파일 전부 (볼트 안, 숨김 · 세팅 폴더 제외).
pub fn all_files(root: &Path) -> Vec<(PathBuf, PathBuf)> {
    walk(root)
        .into_iter()
        .filter_map(|p| {
            let rel = p.strip_prefix(root).ok()?.to_path_buf();
            let parts = rel_parts(&rel);
            if parts[0].starts_with('.') || parts[..parts.len() - 1].iter().any(|x| SKIP_DIRS.contains(&x.as_str())) {
                return None;
            }
            Some((p, rel))
        })
        .collect()
}

/// H1 다음 첫 문단 한 줄 (index에 싣는 요약).
pub fn summary(body: &str) -> String {
    let body = sub(regex!(r"(?s)<!--.*?-->"), body, "");
    let mut seen_h1 = false;
    for line in body.lines() {
        let s = line.trim();
        if s.starts_with("# ") {
            seen_h1 = true;
            continue;
        }
        if seen_h1 && !s.is_empty() && !["#", "-", ">", "|", "```"].iter().any(|p| s.starts_with(p)) {
            let chars: Vec<char> = s.chars().collect();
            if chars.len() <= 120 {
                return s.to_string();
            }
            let mut cut: String = chars[..119].iter().collect();
            let open = rfind_chars(&cut, "[[");
            let close = rfind_chars(&cut, "]]");
            if open > close {
                // 링크 중간에서 자르지 않는다
                let idx = cut.char_indices().nth(open as usize).map(|(i, _)| i).unwrap_or(cut.len());
                cut = cut[..idx].trim_end().to_string();
            }
            return cut + "…";
        }
        if s.starts_with("## ") {
            break;
        }
    }
    String::new()
}

/// Python str.rfind (문자 단위, 없으면 -1)
fn rfind_chars(s: &str, pat: &str) -> i64 {
    match s.rfind(pat) {
        Some(b) => s[..b].chars().count() as i64,
        None => -1,
    }
}

/// 오늘 날짜(YYYY-MM-DD). 시험에서는 YW_TODAY로 바꾼다.
pub fn today() -> String {
    match std::env::var("YW_TODAY") {
        Ok(v) if !v.is_empty() => v,
        _ => chrono::Local::now().format("%Y-%m-%d").to_string(),
    }
}

/// %XX 풀기 (URL 경로)
pub fn unquote(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 3 <= b.len() {
            if let Some(v) = std::str::from_utf8(&b[i + 1..i + 3]).ok().and_then(|h| u8::from_str_radix(h, 16).ok()) {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// git 명령 (출력 캡처)
pub fn git(dir: &Path, args: &[&str]) -> std::process::Output {
    std::process::Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .unwrap_or_else(|_| std::process::Output { status: exit_status(127), stdout: vec![], stderr: "git 없음".as_bytes().to_vec() })
}

pub fn out(o: &std::process::Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

pub fn err(o: &std::process::Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

#[cfg(unix)]
pub fn exit_status(code: i32) -> std::process::ExitStatus {
    use std::os::unix::process::ExitStatusExt;
    std::process::ExitStatus::from_raw(code << 8)
}

pub fn write(path: &Path, text: &str) {
    if let Some(p) = path.parent() {
        let _ = std::fs::create_dir_all(p);
    }
    std::fs::write(path, text).unwrap_or_else(|e| fail(&format!("{} 쓰기 실패: {e}", path.display())));
}

/// 사람에게 알리고 끝낸다 (종료 코드 1)
pub fn fail(msg: &str) -> ! {
    eprintln!("{msg}");
    std::process::exit(1)
}

/// 날짜 문자열 → 날짜 (YYYY-MM-DD만)
pub fn parse_date(v: &str) -> Option<chrono::NaiveDate> {
    if !regex!(r"^\d{4}-\d{2}-\d{2}$").is_match(v).unwrap_or(false) {
        return None;
    }
    chrono::NaiveDate::parse_from_str(v, "%Y-%m-%d").ok()
}

/// 표시용 경로 (홈은 ~)
pub fn short(p: &Path) -> String {
    let s = p.to_string_lossy().into_owned();
    let h = home().to_string_lossy().into_owned();
    match s.strip_prefix(&h) {
        Some(rest) if !h.is_empty() => format!("~{rest}"),
        _ => s,
    }
}

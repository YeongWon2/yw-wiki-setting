//! 링크 · 낡은 표현 감사: 위키와 세팅 저장소 전체에서 깨진 링크와 옛 구조를 가리키는 말을 찾는다.
use crate::wiki::{self, regex};
use std::collections::HashMap;
use std::path::Path;

pub const HELP: &str = "yw-wiki audit-links <저장소> [<저장소> …] [--online]
- 마크다운 링크 [..](경로): 없는 파일, 로컬 절대 경로(/Users/… · ~/… — 다른 PC에서 깨짐)
- --online: github.com/<owner>/<repo> 링크(저장소 · PR · 이슈 · 커밋 · blob)가 실제로 있는지 gh api로 확인
- 낡은 표현: 옛 PARA 경로 · 옛 스킬 · 옛 파일 이름이 \"지금\"처럼 남았는지 (기록 문맥은 허용 목록으로 뺀다)
종료 코드: 문제가 있으면 1.";

const SKIP_DIRS: &[&str] = &[".git", "__pycache__", ".yw-wiki", "node_modules", "target"];
// 지금 구조처럼 쓰이면 안 되는 옛 말. (기록으로 남기는 문장은 아래 HISTORY 단어가 같은 줄에 있으면 허용)
const STALE: &[&str] = &[
    r"0 Inbox\(받은함\)", r"1 Project\(프로젝트\)", r"2 Area\(영역\)", r"3 Resource\(자료\)", r"4 Archive\(보관\)",
    r"_templates\(템플릿\)", r"yw-wiki\.toml", r"\[\[홈\]\]", r"홈\.md", r"conflicts\.md", r"yw-wiki-ingest",
    r"## Contradictions", r"(?<![\w.])python3(\.\d+)? (tools|scripts)/", r"\[\[템플릿 목록\]\]", r"🔗 참조", r"\[!hero\]", r"\[!cols\]",
];
const HISTORY: &[&str] = &[
    "옮김", "옛", "이전", "였다", "했다", "었다", "PARA", "aa1ef83", "pre-llm-wiki", "없앤다", "없앰", "제거", "폐기",
    "대체", "바꿨", "바뀜", "지움", "막는다", "잡는다", "오류", "경고", "검사", "감사", "STALE", "check.py", "옛 키",
];

fn gh_exists(owner: &str, repo: &str, kind: &str, r: &str, cache: &mut HashMap<String, bool>) -> bool {
    let key = format!("{owner}/{repo}/{kind}/{r}");
    if let Some(v) = cache.get(&key) {
        return *v;
    }
    let first = r.split('/').next().unwrap_or("");
    let api = match kind {
        "pull" | "issues" if !first.chars().all(|c| c.is_ascii_digit()) || first.is_empty() => format!("repos/{owner}/{repo}"),
        "pull" | "issues" => format!("repos/{owner}/{repo}/issues/{first}"),
        "commit" => format!("repos/{owner}/{repo}/commits/{first}"),
        "blob" | "tree" => {
            let (branch, path) = r.split_once('/').unwrap_or((r, ""));
            format!("repos/{owner}/{repo}/contents/{}?ref={branch}", wiki::unquote(path))
        }
        _ => format!("repos/{owner}/{repo}"),
    };
    let ok = match std::process::Command::new("gh").args(["api", &api, "--silent"]).output() {
        Ok(o) => o.status.success() || wiki::err(&o).to_lowercase().contains("rate limit"),
        Err(_) => true,
    };
    cache.insert(key, ok);
    ok
}

pub fn main(args: &[String]) -> i32 {
    let online = args.iter().any(|a| a == "--online");
    let roots: Vec<_> = args.iter().filter(|a| !a.starts_with("--")).map(|a| wiki::resolve(&wiki::expand(a))).collect();
    if roots.is_empty() {
        println!("{HELP}");
        return 2;
    }
    let md_link = regex!(r#"(?<!!)\[[^\]]*\]\(<?([^)\s>]+)>?(?:\s+"[^"]*")?\)"#);
    let gh = regex!(r#"https://github\.com/([\w.-]+)/([\w.-]+)(?:/(pull|issues|commit|blob|tree)/([^\s)#\]"',`>]+))?"#);
    let stale: Vec<fancy_regex::Regex> = STALE.iter().map(|p| fancy_regex::Regex::new(p).unwrap()).collect();
    let (mut problems, mut cache) = (vec![], HashMap::new());
    for root in &roots {
        let rname = wiki::name(root);
        for p in wiki::walk(root).into_iter().filter(|p| wiki::is_md(p)) {
            let rel = p.strip_prefix(root).unwrap();
            if wiki::rel_parts(rel).iter().any(|x| SKIP_DIRS.contains(&x.as_str())) {
                continue;
            }
            let rel_s = rel.to_string_lossy();
            let body = wiki::strip_code(&wiki::read(&p));
            for c in md_link.captures_iter(&body).filter_map(|c| c.ok()) {
                let target = c.get(1).unwrap().as_str();
                if ["http://", "https://", "mailto:", "#", "obsidian:"].iter().any(|x| target.starts_with(x)) {
                    continue;
                }
                let path = wiki::unquote(target.split('#').next().unwrap_or(""));
                if path.starts_with('/') || path.starts_with('~') {
                    problems.push(format!("{rname}/{rel_s}: 로컬 절대 경로 링크 {target} — 다른 PC에서 깨진다"));
                    continue;
                }
                if !path.is_empty() && !p.parent().unwrap_or(Path::new(".")).join(&path).exists() {
                    problems.push(format!("{rname}/{rel_s}: 없는 파일 링크 {target}"));
                }
            }
            if online {
                for c in gh.captures_iter(&body).filter_map(|c| c.ok()) {
                    let owner = &c[1];
                    let repo = c[2].strip_suffix(".git").unwrap_or(&c[2]).to_string();
                    let kind = c.get(3).map(|m| m.as_str()).unwrap_or("");
                    let r = c.get(4).map(|m| m.as_str()).unwrap_or("");
                    if !gh_exists(owner, &repo, kind, r, &mut cache) {
                        let tail = if kind.is_empty() { String::new() } else { format!("/{kind}/{r}") };
                        problems.push(format!("{rname}/{rel_s}: GitHub에 없음 https://github.com/{owner}/{repo}{tail}"));
                    }
                }
            }
            // 작업 기록은 지난 일을 적는 곳이라 옛 말이 있어도 된다
            if wiki::name(&p) == "log.md" {
                continue;
            }
            for (i, line) in body.lines().enumerate() {
                for (pat, re) in STALE.iter().zip(&stale) {
                    if re.is_match(line).unwrap_or(false) && !HISTORY.iter().any(|h| line.contains(h)) {
                        let short: String = line.trim().chars().take(120).collect();
                        problems.push(format!("{rname}/{rel_s}:{}: 낡은 표현 /{pat}/ — {short}", i + 1));
                    }
                }
            }
        }
    }
    for l in &problems {
        println!("{l}");
    }
    println!("문제 {}개", problems.len());
    if problems.is_empty() { 0 } else { 1 }
}

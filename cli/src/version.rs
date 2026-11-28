//! 이 위키가 최신인지 본다: main 브랜치 · 원격 main · 세팅 최신 릴리스 · Obsidian 플러그인.
use crate::wiki;
use std::path::Path;
use std::time::{Duration, SystemTime};

pub const HELP: &str = "yw-wiki version [위키 경로] [--hook] [--force]
1. git: 지금 브랜치가 main인지, origin/main과 같은지. 고친 파일이 없으면 뒤처진 것은 pull --rebase로 받고, 앞선 커밋은 push한다(force 없음).
2. 세팅: 이 PC에 길잡이가 깔려 있으면(~/.config/wiki/vaults.toml) `yw-wiki setting check --vault <이 위키>`로 넘긴다
   (세팅 사본을 최신 릴리스 태그로 · 전역 스킬 · 실행 파일 · 이 위키 upgrade). 없으면 원격 태그만 보고 알린다.
3. 플러그인: yw-wiki plugins로 맞춘다.
원격 확인은 1시간에 한 번만 한다(.yw-wiki/version-check). --force는 지금 확인한다.
--hook: 세션 시작 훅용. 한 줄 요약만 출력하고 언제나 종료 코드 0.";

const EVERY: u64 = 3600;

fn git(root: &Path, args: &[&str]) -> std::process::Output {
    wiki::git(root, args)
}

fn check_git(root: &Path, online: bool) -> (String, bool) {
    if !root.join(".git").exists() {
        return ("git 저장소 아님".into(), true);
    }
    let branch = wiki::out(&git(root, &["branch", "--show-current"])).trim().to_string();
    let mut notes: Vec<String> = if branch == "main" { vec!["main".into()] } else { vec![] };
    let mut bad = false;
    if branch != "main" {
        notes.push(format!("지금 브랜치 '{}' — main 기준이 아니다", if branch.is_empty() { "(분리됨)" } else { &branch }));
        bad = true;
    }
    if wiki::out(&git(root, &["remote"])).trim().is_empty() {
        notes.push("원격 없음".into());
        return (if notes.is_empty() { "main".into() } else { notes.join(" · ") }, bad);
    }
    if online && !git(root, &["fetch", "--quiet", "origin", "main"]).status.success() {
        notes.push("원격에 접속 못 함".into());
    }
    let counts: Vec<u64> = wiki::out(&git(root, &["rev-list", "--left-right", "--count", "HEAD...origin/main"]))
        .split_whitespace()
        .filter_map(|x| x.parse().ok())
        .collect();
    if counts.len() == 2 {
        let (mut ahead, mut behind) = (counts[0], counts[1]);
        let dirty = !wiki::out(&git(root, &["status", "--porcelain"])).trim().is_empty();
        if behind > 0 && branch == "main" && !dirty {
            if git(root, &["pull", "--rebase", "--quiet", "origin", "main"]).status.success() {
                notes.push(format!("origin/main에서 {behind}개 받음"));
                behind = 0;
            } else {
                git(root, &["rebase", "--abort"]);
                notes.push(format!("origin/main보다 {behind}개 뒤 — 받다가 충돌, 사람이 정한다"));
                bad = true;
            }
        } else if behind > 0 {
            notes.push(format!("origin/main보다 {behind}개 뒤{}", if dirty { " (고친 파일이 있어 받지 않음)" } else { "" }));
            bad = true;
        }
        // 규칙: main은 원격과 같게 (force 없음)
        if ahead > 0 && branch == "main" && !dirty && behind == 0 && online && git(root, &["push", "-q", "origin", "main"]).status.success() {
            notes.push(format!("커밋 {ahead}개 push"));
            ahead = 0;
        }
        if ahead > 0 {
            notes.push(format!("push 안 한 커밋 {ahead}개"));
        }
    }
    if notes == ["main"] {
        notes = vec!["main 최신".into()];
    }
    (notes.join(" · "), bad)
}

fn vt(v: &str) -> Vec<u64> {
    v.split('.').filter_map(|x| x.parse().ok()).collect()
}

fn check_setting(root: &Path, online: bool) -> Result<(String, bool), String> {
    let state: toml::Table = wiki::read(&root.join(".wiki-setting.toml")).parse().map_err(|e| format!(".wiki-setting.toml: {e}"))?;
    let version = state.get("version").and_then(|v| v.as_str()).unwrap_or("?").to_string();
    if crate::setting::vaults_path().exists() {
        if !online {
            return Ok((format!("세팅 v{version}"), false));
        }
        let (lines, rc) = crate::global::check(Some(root));
        let last = lines.last().cloned().unwrap_or_default();
        return Ok((last, rc != 0));
    }
    // 길잡이가 없으면 원격 태그만 본다
    let repo = state.get("setting_repo").and_then(|v| v.as_str()).unwrap_or("");
    let tags = if online && !repo.is_empty() {
        std::process::Command::new("git").args(["ls-remote", "--tags", "--refs", &format!("https://github.com/{repo}.git")]).output().ok()
    } else {
        None
    };
    let Some(tags) = tags.filter(|t| t.status.success()) else {
        return Ok((format!("세팅 v{version} (최신 확인 못 함 — yw-wiki setting install)"), false));
    };
    let mut vers: Vec<String> = wiki::out(&tags)
        .split_whitespace()
        .filter_map(|t| t.rsplit_once("/v").map(|(_, v)| v.to_string()))
        .filter(|v| !v.is_empty() && v.replace('.', "").chars().all(|c| c.is_ascii_digit()))
        .collect();
    vers.sort_by_key(|v| vt(v));
    let latest = vers.last().cloned().unwrap_or(version.clone());
    if latest != version {
        return Ok((format!("세팅 v{version} → 최신 v{latest} — yw-wiki setting install 뒤 갱신해줘"), true));
    }
    Ok((format!("세팅 v{latest} 최신"), false))
}

/// (요약 한 줄, 문제 여부)
pub fn run(root: &Path, force: bool) -> (String, bool) {
    let stamp = root.join(".yw-wiki").join("version-check");
    let fresh = std::fs::metadata(&stamp)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| SystemTime::now().duration_since(t).ok())
        .map(|d| d <= Duration::from_secs(EVERY))
        .unwrap_or(false);
    let online = force || !fresh;
    let mut lines = vec![];
    let mut bad = false;
    let (t, b) = check_git(root, online);
    lines.push(t);
    bad |= b;
    let (t, b) = check_setting(root, online).unwrap_or_else(|e| (format!("확인 실패: {e}"), true));
    lines.push(t);
    bad |= b;
    let (pl, rc) = crate::plugins::run(root, false);
    lines.push(pl.first().cloned().unwrap_or_else(|| "Obsidian 플러그인: ?".into()));
    bad |= rc != 0;
    if online {
        if let Some(p) = stamp.parent() {
            let _ = std::fs::create_dir_all(p);
        }
        let _ = std::fs::write(&stamp, "");
    }
    let name = wiki::tstr(&wiki::config(root), &["wiki", "name"]).unwrap_or_else(|| wiki::name(root));
    (format!("[{name}] {}{}", if bad { "⚠ " } else { "" }, lines.join(" · ")), bad)
}

pub fn main(args: &[String]) -> i32 {
    let (root, rest) = wiki::root_and_rest(args);
    let hook = rest.iter().any(|a| a == "--hook");
    let force = rest.iter().any(|a| a == "--force");
    let (line, bad) = run(&root, force);
    println!("{line}");
    if hook || !bad { 0 } else { 1 }
}

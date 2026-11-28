//! Obsidian 플러그인과 기본 설정을 맞춘다. 목록은 tools/obsidian/plugins.toml (세팅이 관리).
use crate::wiki;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

pub const HELP: &str = "yw-wiki plugins [위키 경로] [--check]
- 플러그인 파일(main.js · manifest.json · styles.css)이 없거나 고정한 sha256과 다르면 GitHub 릴리스에서 받아 확인한 뒤 둔다.
  받은 파일은 ~/.cache/yw-wiki/plugins/<id>/<version>/에 두고 다시 쓴다. sha256이 다르면 설치하지 않는다.
- .obsidian/community-plugins.json에 목록의 플러그인을 켠다 (사람이 더 켠 것은 그대로).
- 기본 설정(플러그인 data.json, graph.json)은 파일이 없을 때만 깐다.
--check: 아무것도 바꾸지 않고 맞지 않는 것만 알린다 (종료 코드 1).";

/// 없을 때만 까는 Obsidian 기본값
const DEFAULTS: &[(&str, &str)] = &[(".obsidian/graph.json", "graph.json")];

fn sha_bytes(b: &[u8]) -> String {
    Sha256::digest(b).iter().map(|x| format!("{x:02x}")).collect()
}

fn sha(p: &Path) -> String {
    sha_bytes(&std::fs::read(p).unwrap_or_default())
}

fn cache() -> PathBuf {
    wiki::home().join(".cache").join("yw-wiki").join("plugins")
}

fn fetch(repo: &str, version: &str, name: &str, want: &str) -> Result<PathBuf, String> {
    if std::env::var_os("YW_NO_NETWORK").is_some() {
        return Err("YW_NO_NETWORK — 받지 않음".into());
    }
    let cached = cache().join(repo.replace('/', "_")).join(version).join(name);
    if !cached.exists() || sha(&cached) != want {
        std::fs::create_dir_all(cached.parent().unwrap()).map_err(|e| e.to_string())?;
        let url = format!("https://github.com/{repo}/releases/download/{version}/{name}");
        let o = std::process::Command::new("curl")
            .args(["-fsSL", "--max-time", "30", &url])
            .output()
            .map_err(|e| format!("curl: {e}"))?;
        if !o.status.success() {
            return Err(format!("{url}: {}", wiki::err(&o).trim()));
        }
        if sha_bytes(&o.stdout) != want {
            return Err(format!("{repo} {version} {name}: sha256이 고정값과 다르다 — 설치하지 않음"));
        }
        std::fs::write(&cached, &o.stdout).map_err(|e| e.to_string())?;
    }
    Ok(cached)
}

fn copy(src: &Path, dest: &Path) -> Result<(), String> {
    if let Some(p) = dest.parent() {
        std::fs::create_dir_all(p).map_err(|e| e.to_string())?;
    }
    std::fs::copy(src, dest).map(|_| ()).map_err(|e| format!("{}: {e}", src.display()))
}

/// (출력 줄, 종료 코드)
pub fn run(root: &Path, check: bool) -> (Vec<String>, i32) {
    let here = root.join("tools").join("obsidian");
    let catalog: toml::Table = wiki::read(&here.join("plugins.toml"))
        .parse::<toml::Table>()
        .ok()
        .and_then(|t| t.get("plugins").and_then(|p| p.as_table()).cloned())
        .unwrap_or_default();
    let obs = root.join(".obsidian");
    let mut problems: Vec<String> = vec![];

    let enabled_path = obs.join("community-plugins.json");
    let enabled: Vec<String> = serde_json::from_str(&wiki::read(&enabled_path)).unwrap_or_default();
    let missing: Vec<String> = catalog.keys().filter(|k| !enabled.contains(k)).cloned().collect();
    if !missing.is_empty() {
        problems.push(format!("켜지지 않은 플러그인: {}", missing.join(", ")));
        if !check {
            let all: Vec<String> = enabled.iter().chain(missing.iter()).cloned().collect();
            wiki::write(&enabled_path, &(serde_json::to_string_pretty(&all).unwrap() + "\n"));
        }
    }

    for (pid, p) in &catalog {
        let dest = obs.join("plugins").join(pid);
        let version = p.get("version").and_then(|v| v.as_str()).unwrap_or("");
        let repo = p.get("repo").and_then(|v| v.as_str()).unwrap_or("");
        if let Some(shas) = p.get("sha256").and_then(|s| s.as_table()) {
            for (name, want) in shas {
                let want = want.as_str().unwrap_or("");
                let f = dest.join(name);
                if f.exists() && sha(&f) == want {
                    continue;
                }
                problems.push(format!("{pid} {version}: {name} {}", if f.exists() { "버전이 다름" } else { "없음" }));
                if !check {
                    match fetch(repo, version, name, want).and_then(|src| copy(&src, &f)) {
                        Ok(()) => {}
                        // 네트워크 없음 · sha256 다름: 깐 것 그대로 두고 알린다
                        Err(e) => problems.push(format!("  설치 못 함: {e}")),
                    }
                }
            }
        }
        if let Some(d) = p.get("defaults").and_then(|v| v.as_str()) {
            if !d.is_empty() && !dest.join("data.json").exists() {
                problems.push(format!("{pid}: 기본 설정 없음"));
                if !check {
                    if let Err(e) = copy(&here.join("defaults").join(d), &dest.join("data.json")) {
                        problems.push(format!("  설치 못 함: {e}"));
                    }
                }
            }
        }
    }
    for (rel, src) in DEFAULTS {
        if !root.join(rel).exists() {
            problems.push(format!("{rel} 없음"));
            if !check {
                if let Err(e) = copy(&here.join("defaults").join(src), &root.join(rel)) {
                    problems.push(format!("  설치 못 함: {e}"));
                }
            }
        }
    }

    if check {
        let rc = if problems.is_empty() { 0 } else { 1 };
        return (problems.iter().map(|l| format!("플러그인  {l}")).collect(), rc);
    }
    let unresolved: Vec<String> = problems.iter().filter(|l| l.starts_with("  설치 못 함")).cloned().collect();
    let head = format!(
        "Obsidian 플러그인: {}{}",
        if unresolved.is_empty() { "맞춤".to_string() } else { format!("설치 못 한 것 {}개", unresolved.len()) },
        if problems.is_empty() { String::new() } else { format!(" (고친 것 {}개)", problems.len() - unresolved.len()) }
    );
    let rc = if unresolved.is_empty() { 0 } else { 1 };
    let mut lines = vec![head];
    lines.extend(unresolved);
    (lines, rc)
}

pub fn main(args: &[String]) -> i32 {
    let (root, rest) = wiki::root_and_rest(args);
    let (lines, rc) = run(&root, rest.iter().any(|a| a == "--check"));
    for l in lines {
        println!("{l}");
    }
    rc
}

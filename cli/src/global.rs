//! yw-wiki setting (= wiki-setting): 이 PC의 LLM 위키 볼트를 관리한다. 볼트 목록은 ~/.config/wiki/vaults.toml.
use crate::setting::{self, vaults_path};
use crate::wiki::{self, expand, resolve, short};
use serde_json::Value;
use std::path::{Path, PathBuf};
use toml::Table;

pub const HELP: &str = "yw-wiki setting <명령>  (예전 이름 wiki-setting으로도 불린다)
  install [--from <세팅 폴더>] [--dry-run]   전역 길잡이 스킬을 Claude Code · Codex에 복사, vaults.toml이 없으면 만든다
  list | path <id>                          등록된 볼트와 경로
  register <폴더> [--id <id>] [--readonly] [--name <이름>] [--alias <별칭>]... [--description <설명>]
                                            볼트 등록 (id는 그 폴더 wiki.toml [wiki] id). 이름 · 별칭 · 설명을 주면 vaults.toml 값이 먼저
  new-vault --profile personal --path <폴더>   위키 생성 + 등록 + 첫 점검
  doctor [--fix]                            전역 스킬 · 목록 · 경로 · 세팅 버전 · 각 볼트 점검 (--fix: 옮겨진 볼트 경로와 그 Obsidian 볼트 고침)
  upgrade [<id>] [--yes]                    세팅 최신 릴리스 태그를 받아 볼트에 반영 (메이저는 --yes)
  check [--vault <폴더>]                    최신 확인: 세팅 사본 → main의 최신 릴리스 태그, 실행 파일 · 전역 스킬 버전,
                                            볼트 세팅 버전(패치 · 마이너는 반영 · 커밋, 메이저는 알림), 볼트 main 기준
  obsidian [--fix] [--prune] [--restart]    Obsidian 볼트 목록 · 폴더 설정을 위키 경로에 맞춘다 (--fix는 Obsidian이 꺼져 있을 때만,
                                            --restart는 사람에게 \"Obsidian을 끌까요?\" 물은 뒤에만)
세팅은 vaults.toml [setting] path의 git 사본을 쓴다. 원격이 있으면 main이 아니라 최신 릴리스 태그(vX.Y.Z)를 받는다.";

const VAULT_META: &[&str] = &["name", "description", "aliases"];
const OBSIDIAN_FOLDERS: &[(&str, &[&str])] = &[
    ("app.json", &["newFileFolderPath", "attachmentFolderPath"]),
    ("templates.json", &["folder"]),
    ("daily-notes.json", &["folder"]),
];

fn skill_dirs() -> Vec<PathBuf> {
    let h = wiki::home();
    vec![h.join(".claude").join("skills").join("wiki-vault"), h.join(".agents").join("skills").join("wiki-vault")]
}

fn search_roots() -> Vec<PathBuf> {
    let h = wiki::home();
    vec![h.join("Desktop").join("wiki"), h.join("vaults"), h.join("Desktop"), h.join("Documents"), h.join("Desktop").join("yw-workspace")]
}

fn load() -> Table {
    let p = vaults_path();
    if p.exists() {
        wiki::read(&p).parse().unwrap_or_else(|e| wiki::fail(&format!("{} 읽기 실패: {e}", p.display())))
    } else {
        let mut t = Table::new();
        t.insert("version".into(), toml::Value::Integer(1));
        t.insert("vaults".into(), toml::Value::Table(Table::new()));
        t
    }
}

fn jstr(s: &str) -> String {
    serde_json::to_string(s).unwrap()
}

/// TOML 키: 안전한 이름은 그대로, 아니면 따옴표
fn key(k: &str) -> String {
    if safe_id(k) { k.to_string() } else { jstr(k) }
}

/// 볼트 id는 영문 · 숫자 · _ · -만
fn safe_id(k: &str) -> bool {
    !k.is_empty() && k.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

fn dump(conf: &Table) -> String {
    let mut lines = vec![
        "# 이 PC의 LLM 위키 볼트 목록. wiki-setting이 관리한다. 이름 · 별칭 · 설명은 각 위키 wiki.toml, 여기 적으면 여기 값이 먼저.".to_string(),
        format!("version = {}", conf.get("version").and_then(|v| v.as_integer()).unwrap_or(1)),
    ];
    if let Some(d) = conf.get("default").and_then(|v| v.as_str()).filter(|s| !s.is_empty()) {
        lines.push(format!("default = {}", jstr(d)));
    }
    if let Some(s) = conf.get("setting").and_then(|v| v.as_table()).filter(|s| !s.is_empty()) {
        lines.push(String::new());
        lines.push("[setting]".into());
        for (k, v) in s {
            lines.push(format!("{} = {}", key(k), jstr(&v.as_str().map(String::from).unwrap_or_else(|| v.to_string()))));
        }
    }
    if let Some(vs) = conf.get("vaults").and_then(|v| v.as_table()) {
        for (vid, v) in vs {
            let v = v.as_table().cloned().unwrap_or_default();
            lines.push(String::new());
            lines.push(format!("[vaults.{}]", key(vid)));
            lines.push(format!("path = {}", jstr(v.get("path").and_then(|x| x.as_str()).unwrap_or(""))));
            if v.get("readonly").and_then(|x| x.as_bool()).unwrap_or(false) {
                lines.push("readonly = true".into());
            }
            // 이 PC의 Obsidian 볼트 id. 위키 폴더가 옮겨져도 같은 볼트를 따라간다
            if let Some(x) = v.get("obsidian_id").and_then(|x| x.as_str()).filter(|s| !s.is_empty()) {
                lines.push(format!("obsidian_id = {}", jstr(x)));
            }
            // Obsidian 볼트가 위키의 몇 단계 위 폴더인가
            if let Some(x) = v.get("obsidian_depth").and_then(|x| x.as_integer()).filter(|d| *d != 0) {
                lines.push(format!("obsidian_depth = {x}"));
            }
            // 경로를 고치기 전 위치. Obsidian 볼트를 맞추면 지운다
            if let Some(x) = v.get("previous_path").and_then(|x| x.as_str()).filter(|s| !s.is_empty()) {
                lines.push(format!("previous_path = {}", jstr(x)));
            }
            for k in VAULT_META {
                match v.get(*k) {
                    Some(toml::Value::Array(a)) if !a.is_empty() => {
                        let items: Vec<String> = a.iter().map(|x| jstr(x.as_str().unwrap_or(""))).collect();
                        lines.push(format!("{k} = [{}]", items.join(", ")));
                    }
                    Some(toml::Value::String(s)) if !s.is_empty() => lines.push(format!("{k} = {}", jstr(s))),
                    _ => {}
                }
            }
        }
    }
    lines.join("\n") + "\n"
}

fn save(conf: &Table, dry: bool) -> String {
    let text = dump(conf);
    if dry {
        return format!("--- {} (미리보기)\n{text}", vaults_path().display());
    }
    wiki::write(&vaults_path(), &text);
    format!("저장: {}", vaults_path().display())
}

fn vaults(conf: &Table) -> Table {
    conf.get("vaults").and_then(|v| v.as_table()).cloned().unwrap_or_default()
}

fn vaults_mut(conf: &mut Table) -> &mut Table {
    if !conf.get("vaults").map(|v| v.is_table()).unwrap_or(false) {
        conf.insert("vaults".into(), toml::Value::Table(Table::new()));
    }
    conf.get_mut("vaults").unwrap().as_table_mut().unwrap()
}

fn vs(v: &Table, k: &str) -> Option<String> {
    v.get(k).and_then(|x| x.as_str()).filter(|s| !s.is_empty()).map(String::from)
}

fn vpath(v: &Table) -> PathBuf {
    resolve(&expand(&vs(v, "path").unwrap_or_default()))
}

fn wiki_info(path: &Path) -> Table {
    let f = path.join("wiki.toml");
    if !f.exists() {
        return Table::new();
    }
    wiki::read(&f).parse::<Table>().ok().and_then(|t| t.get("wiki").and_then(|w| w.as_table()).cloned()).unwrap_or_default()
}

/// wiki.toml [wiki] 위에 vaults.toml 항목의 이름 · 별칭 · 설명을 덮는다.
fn vault_info(v: &Table, path: &Path) -> Table {
    let mut w = wiki_info(path);
    for k in VAULT_META {
        if let Some(x) = v.get(*k) {
            let truthy = match x {
                toml::Value::String(s) => !s.is_empty(),
                toml::Value::Array(a) => !a.is_empty(),
                _ => true,
            };
            if truthy {
                w.insert(k.to_string(), x.clone());
            }
        }
    }
    w
}

fn git(dir: &Path, args: &[&str]) -> std::process::Output {
    wiki::git(dir, args)
}

fn vt(v: &str) -> Vec<u64> {
    fancy_regex::Regex::new(r"\d+").unwrap().find_iter(v).filter_map(|m| m.ok()).take(3).filter_map(|m| m.as_str().parse().ok()).collect()
}

/// 원격이 있으면 태그를 받아 origin/main에 있는 최신 vX.Y.Z. (태그, 메모)
fn latest_tag(sdir: &Path) -> (Option<String>, String) {
    if wiki::out(&git(sdir, &["remote"])).trim().is_empty() {
        return (None, "원격 없음 — 로컬 사본 그대로 씀".into());
    }
    let ok = std::env::var_os("YW_SETTING_CHECKED").is_some()
        || git(sdir, &["fetch", "--tags", "--quiet", "origin", "+refs/heads/main:refs/remotes/origin/main"]).status.success();
    let note = if ok { String::new() } else { "원격에 접속 못 함 — 가진 사본으로 계속".into() };
    let mut tags: Vec<String> = wiki::out(&git(sdir, &["tag", "--merged", "origin/main", "--list", "v*"]))
        .split_whitespace()
        .filter(|t| fancy_regex::Regex::new(r"^v\d+\.\d+\.\d+$").unwrap().is_match(t).unwrap_or(false))
        .map(String::from)
        .collect();
    if tags.is_empty() {
        return (None, if note.is_empty() { "릴리스 태그 없음 — 로컬 사본 그대로 씀".into() } else { note });
    }
    tags.sort_by_key(|t| vt(t));
    (tags.pop(), note)
}

fn obsidian_json() -> PathBuf {
    let h = wiki::home();
    if cfg!(target_os = "macos") {
        return h.join("Library").join("Application Support").join("obsidian").join("obsidian.json");
    }
    std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from).unwrap_or_else(|| h.join(".config")).join("obsidian").join("obsidian.json")
}

/// Obsidian이 켜져 있으면 obsidian.json을 고쳐도 끌 때 덮어쓴다. (시험: YW_OBSIDIAN_RUNNING=0/1)
fn obsidian_running() -> bool {
    match std::env::var("YW_OBSIDIAN_RUNNING").as_deref() {
        Ok("0") => return false,
        Ok("1") => return true,
        _ => {}
    }
    std::process::Command::new("pgrep").args(["-xi", "obsidian"]).output().map(|o| o.status.success()).unwrap_or(false)
}

/// 켜진 Obsidian을 정상 종료하고 꺼질 때까지 기다린다(macOS).
fn obsidian_quit() -> bool {
    if std::env::var("YW_OBSIDIAN_RUNNING").as_deref() == Ok("1") {
        std::env::set_var("YW_OBSIDIAN_RUNNING", "0");
        return true;
    }
    if !cfg!(target_os = "macos") {
        return false;
    }
    let _ = std::process::Command::new("osascript").args(["-e", "quit app \"Obsidian\""]).output();
    for _ in 0..40 {
        if !obsidian_running() {
            return true;
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
    false
}

fn obsidian_open() -> bool {
    if std::env::var_os("YW_OBSIDIAN_RUNNING").is_some() {
        return true;
    }
    cfg!(target_os = "macos") && std::process::Command::new("open").args(["-a", "Obsidian"]).output().map(|o| o.status.success()).unwrap_or(false)
}

fn parts_len(p: &Path) -> usize {
    p.components().count()
}

/// 위키마다 Obsidian 볼트를 찾아 경로를 맞춘다. (메모, 문제 여부, vaults.toml을 고쳤나)
/// 찾는 순서: 기록한 obsidian_id → 같은 경로 → 위키를 품은 상위 폴더 볼트 → 폴더가 없어진 볼트 중 previous_path를 품었거나 이름이 같은 것 하나.
fn obsidian_vaults(conf: &mut Table, fix: bool, prune: bool) -> (Vec<String>, bool, bool) {
    let oj = obsidian_json();
    if !oj.exists() {
        return (vec!["Obsidian 볼트 목록 없음 — 건너뜀".into()], false, false);
    }
    let mut data: Value = serde_json::from_str(&wiki::read(&oj)).unwrap_or(Value::Object(Default::default()));
    if !data.get("vaults").map(|v| v.is_object()).unwrap_or(false) {
        data["vaults"] = Value::Object(Default::default());
    }
    let ovs = data["vaults"].as_object().unwrap().clone();
    let opath = |i: &str| resolve(Path::new(ovs[i]["path"].as_str().unwrap_or("")));
    let (mut notes, mut bad, mut conf_changed) = (vec![], false, false);
    let mut moves: Vec<(String, String, String, String)> = vec![];
    let mut used = std::collections::HashSet::new();
    let ids: Vec<String> = ovs.keys().cloned().collect();
    let vlist = vaults(conf);
    for (vid, v) in &vlist {
        let v = v.as_table().cloned().unwrap_or_default();
        let path = vpath(&v);
        if !path.exists() {
            continue;
        }
        let old = vs(&v, "previous_path").map(|p| resolve(&expand(&p)));
        let mut oid = vs(&v, "obsidian_id").filter(|i| ovs.contains_key(i));
        let mut depth = if oid.is_some() { v.get("obsidian_depth").and_then(|d| d.as_integer()).unwrap_or(0) as usize } else { 0 };
        if oid.is_none() {
            oid = ids.iter().find(|i| opath(i) == path).cloned();
        }
        if oid.is_none() {
            oid = ids.iter().find(|i| path.starts_with(opath(i))).cloned();
            depth = oid.as_ref().map(|i| parts_len(path.strip_prefix(opath(i)).unwrap())).unwrap_or(0);
        }
        if oid.is_none() {
            let mut cands = vec![];
            for i in &ids {
                let op = opath(i);
                if op.exists() {
                    continue;
                }
                if let Some(o) = old.as_ref().filter(|o| o.starts_with(&op)) {
                    cands.push((i.clone(), parts_len(o.strip_prefix(&op).unwrap())));
                } else if wiki::name(&op) == wiki::name(&path) {
                    cands.push((i.clone(), 0));
                }
            }
            if cands.len() != 1 {
                notes.push(format!(
                    "{vid}: Obsidian 볼트 없음{}",
                    if cands.is_empty() {
                        " — Obsidian에서 이 폴더를 볼트로 열면 등록된다".to_string()
                    } else {
                        format!(" — 후보 여럿({}), 사람이 고른다", cands.iter().map(|(i, _)| short(Path::new(ovs[i]["path"].as_str().unwrap_or("")))).collect::<Vec<_>>().join(", "))
                    }
                ));
                continue;
            }
            let (i, d) = cands.remove(0);
            oid = Some(i);
            depth = d;
        }
        let oid = oid.unwrap();
        let opath_s = ovs[&oid]["path"].as_str().unwrap_or("").to_string();
        if depth >= parts_len(&path) {
            notes.push(format!("{vid}: Obsidian 볼트 {}를 이 위키에 맞출 수 없음 — 사람이 고른다", short(Path::new(&opath_s))));
            bad = true;
            continue;
        }
        used.insert(oid.clone());
        let entry = vaults_mut(conf).get_mut(vid).and_then(|x| x.as_table_mut()).unwrap();
        let cur = (vs(entry, "obsidian_id"), entry.get("obsidian_depth").and_then(|d| d.as_integer()).unwrap_or(0) as usize);
        if cur != (Some(oid.clone()), depth) {
            entry.insert("obsidian_id".into(), toml::Value::String(oid.clone()));
            conf_changed = true;
            if depth > 0 {
                entry.insert("obsidian_depth".into(), toml::Value::Integer(depth as i64));
            } else {
                entry.remove("obsidian_depth");
            }
        }
        let target = if depth > 0 { path.ancestors().nth(depth).unwrap().to_path_buf() } else { path.clone() };
        if opath(&oid) != target {
            moves.push((vid.clone(), oid.clone(), opath_s.clone(), target.to_string_lossy().into_owned()));
        } else {
            if depth > 0 {
                notes.push(format!("{vid}: Obsidian은 상위 폴더 볼트 {}로 엶", short(Path::new(&opath_s))));
            }
            if entry.remove("previous_path").map(|x| x.as_str().map(|s| !s.is_empty()).unwrap_or(true)).unwrap_or(false) {
                conf_changed = true;
            }
        }
    }
    let stale: Vec<String> = ids.iter().filter(|i| !used.contains(*i) && !opath(i).exists()).cloned().collect();
    let todo = !moves.is_empty() || (prune && !stale.is_empty());
    if todo && fix && obsidian_running() {
        notes.push("Obsidian이 켜져 있어 볼트 목록을 고치지 않음 — 사람에게 끌지 묻고 --restart로 다시".into());
        return (notes, true, conf_changed);
    }
    for (vid, oid, old_p, new_p) in &moves {
        if fix {
            data["vaults"][oid]["path"] = Value::String(new_p.clone());
            if let Some(e) = vaults_mut(conf).get_mut(vid).and_then(|x| x.as_table_mut()) {
                if e.remove("previous_path").is_some() {
                    conf_changed = true;
                }
            }
        }
        notes.push(format!("{vid}: Obsidian 볼트 {} → {}{}", short(Path::new(old_p)), short(Path::new(new_p)), if fix { "" } else { " (옮길 것 — --fix)" }));
        bad |= !fix;
    }
    for oid in &stale {
        notes.push(format!(
            "폴더가 없는 Obsidian 볼트 {}{}",
            short(Path::new(ovs[oid]["path"].as_str().unwrap_or(""))),
            if fix && prune { " 지움" } else { " (지우려면 --fix --prune)" }
        ));
        bad |= prune && !fix;
        if fix && prune {
            data["vaults"].as_object_mut().unwrap().remove(oid);
        }
    }
    if fix && todo {
        // 고치기 전 사본
        let _ = std::fs::copy(&oj, oj.with_file_name("obsidian.json.yw-bak"));
        wiki::write(&oj, &serde_json::to_string(&data).unwrap());
    }
    (notes, bad, conf_changed)
}

/// 위키의 Obsidian 설정이 없는 폴더를 가리키면 세팅 값으로 맞춘다. (메모, 문제 여부)
fn obsidian_folders(conf: &Table, sdir: &Path, fix: bool) -> (Vec<String>, bool) {
    let (mut notes, mut bad) = (vec![], false);
    for (vid, v) in &vaults(conf) {
        let v = v.as_table().cloned().unwrap_or_default();
        let path = vpath(&v);
        let st = path.join(".wiki-setting.toml");
        if v.get("readonly").and_then(|x| x.as_bool()).unwrap_or(false) || !st.exists() {
            continue;
        }
        let profile = wiki::read(&st).parse::<Table>().ok().and_then(|t| t.get("profile").and_then(|p| p.as_str()).map(String::from)).unwrap_or("personal".into());
        for (name, keys) in OBSIDIAN_FOLDERS {
            let f = path.join(".obsidian").join(name);
            if !f.exists() {
                continue;
            }
            let mut conf_o: Value = serde_json::from_str(&wiki::read(&f)).unwrap_or(Value::Object(Default::default()));
            let obs = sdir.join("core").join("obsidian");
            let src = [obs.join(&profile).join(name), obs.join("common").join(name)].into_iter().find(|p| p.exists());
            let want: Value = src.map(|p| serde_json::from_str(&wiki::read(&p)).unwrap_or(Value::Null)).unwrap_or(Value::Null);
            let mut changed = false;
            for k in *keys {
                let cur = conf_o.get(*k).and_then(|x| x.as_str()).unwrap_or("").to_string();
                if cur.is_empty() || path.join(&cur).is_dir() {
                    continue;
                }
                let new = want.get(*k).and_then(|x| x.as_str()).unwrap_or("").to_string();
                if !new.is_empty() && path.join(&new).is_dir() {
                    notes.push(format!("{vid}: Obsidian {name} {k} {cur} → {new}{}", if fix { "" } else { " (고칠 것 — --fix)" }));
                    if fix {
                        conf_o[*k] = Value::String(new);
                        changed = true;
                    }
                    bad |= !fix;
                } else {
                    notes.push(format!("{vid}: Obsidian {name} {k}가 없는 폴더 {cur}를 가리킴 — 사람이 정한다"));
                    bad = true;
                }
            }
            if changed && obsidian_running() {
                notes.push(format!("{vid}: Obsidian이 켜져 있어 {name}을 고치지 않음 — 사람에게 끌지 묻고 --restart로 다시"));
                bad = true;
            } else if changed {
                wiki::write(&f, &(serde_json::to_string_pretty(&conf_o).unwrap() + "\n"));
            }
        }
    }
    (notes, bad)
}

fn flag(args: &[String], f: &str) -> bool {
    args.iter().any(|a| a == f)
}

fn opt(args: &[String], name: &str) -> Option<String> {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned()
}

fn opts(args: &[String], name: &str) -> Vec<String> {
    args.iter().enumerate().filter(|(_, a)| *a == name).filter_map(|(i, _)| args.get(i + 1).cloned()).collect()
}

/// 옵션 값이 아닌 위치 인자
fn positional(args: &[String], with_value: &[&str]) -> Vec<String> {
    let mut out = vec![];
    let mut i = 0;
    while i < args.len() {
        if with_value.contains(&args[i].as_str()) {
            i += 2;
            continue;
        }
        if !args[i].starts_with("--") {
            out.push(args[i].clone());
        }
        i += 1;
    }
    out
}

fn cmd_obsidian(args: &[String]) -> i32 {
    let (fix, prune, restart) = (flag(args, "--fix"), flag(args, "--prune"), flag(args, "--restart"));
    let mut conf = load();
    let sdir = setting::setting_dir(None);
    let mut reopen = false;
    if fix && restart && obsidian_running() {
        // 고칠 것이 있을 때만 끈다. 끌 때 Obsidian이 쓴 목록을 아래에서 다시 읽는다
        let mut probe = load();
        if obsidian_vaults(&mut probe, false, prune).1 || obsidian_folders(&conf, &sdir, false).1 {
            if !obsidian_quit() {
                println!("⚠ Obsidian을 끄지 못함 — 직접 끈 뒤 다시");
                return 1;
            }
            reopen = true;
        }
    }
    let (notes, bad, conf_changed) = obsidian_vaults(&mut conf, fix, prune);
    let (mut n2, b2) = obsidian_folders(&conf, &sdir, fix);
    if reopen {
        n2.push(if obsidian_open() { "Obsidian을 껐다가 다시 켬".into() } else { "Obsidian을 껐음 — 다시 켜지 못함, 직접 켠다".into() });
    }
    if conf_changed {
        println!("{}", save(&conf, false));
    }
    for n in notes.iter().chain(n2.iter()) {
        println!("{n}");
    }
    println!("{}", if bad || b2 { "⚠ Obsidian 고칠 것 있음" } else { "Obsidian 맞음" });
    if bad || b2 { 1 } else { 0 }
}

fn copy_tree(src: &Path, dest: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dest)?;
    for e in std::fs::read_dir(src)? {
        let e = e?;
        let to = dest.join(e.file_name());
        if e.file_type()?.is_dir() {
            copy_tree(&e.path(), &to)?;
        } else {
            std::fs::copy(e.path(), &to)?;
        }
    }
    Ok(())
}

/// 길잡이 스킬을 d에 다시 깐다 (링크가 아니라 복사: 세팅 폴더가 없어져도 스킬은 남는다)
fn install_skill(src: &Path, d: &Path, version: Option<&str>) {
    if d.exists() {
        let _ = std::fs::remove_dir_all(d);
    }
    copy_tree(src, d).unwrap_or_else(|e| wiki::fail(&format!("{} 복사 실패: {e}", d.display())));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let bin = d.join("bin").join("wiki-setting");
        if bin.exists() {
            let _ = std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o755));
        }
    }
    if let Some(v) = version {
        let _ = std::fs::write(d.join("VERSION"), format!("{v}\n"));
    }
}

fn cmd_install(args: &[String]) -> i32 {
    let dry = flag(args, "--dry-run");
    let mut src = match opt(args, "--from") {
        Some(p) => resolve(&expand(&p)),
        None => setting::setting_dir(None).join("global").join("wiki-vault"),
    };
    if !src.join("SKILL.md").exists() {
        src = src.join("global").join("wiki-vault");
    }
    if !src.join("SKILL.md").exists() {
        wiki::fail(&format!("{}에 wiki-vault 스킬이 없다", src.display()));
    }
    let sdir = src.parent().and_then(|p| p.parent()).map(|p| p.to_path_buf()).unwrap();
    let ver = setting::version(&sdir);
    for d in skill_dirs() {
        println!("{}: {} → {}", if dry { "복사 예정" } else { "복사" }, short(&src), short(&d));
        if !dry {
            install_skill(&src, &d, if ver.is_empty() { None } else { Some(&ver) });
        }
    }
    let mut conf = load();
    if !vaults_path().exists() || dry {
        if !conf.contains_key("setting") {
            let mut s = Table::new();
            s.insert("path".into(), toml::Value::String(short(&sdir)));
            conf.insert("setting".into(), toml::Value::Table(s));
        }
        println!("{}", save(&conf, dry));
    }
    0
}

fn cmd_list() -> i32 {
    let conf = load();
    let vlist = vaults(&conf);
    if vlist.is_empty() {
        println!("등록된 볼트 없음 — yw-wiki setting register <폴더>");
        return 0;
    }
    let default = conf.get("default").and_then(|d| d.as_str()).unwrap_or("");
    for (vid, v) in &vlist {
        let v = v.as_table().cloned().unwrap_or_default();
        let path = vpath(&v);
        let w = vault_info(&v, &path);
        let mark = if path.exists() { "" } else { "  (폴더 없음 — doctor)" };
        let ro = if v.get("readonly").and_then(|x| x.as_bool()).unwrap_or(false) { " · 읽기 전용" } else { "" };
        println!("{vid}{}: {}{ro}{mark}", if default == vid { " (기본)" } else { "" }, short(&path));
        if !w.is_empty() {
            let aliases: Vec<String> = w.get("aliases").and_then(|a| a.as_array()).map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
            println!(
                "  {} — {} · 별칭 {}",
                w.get("name").and_then(|x| x.as_str()).unwrap_or(""),
                w.get("description").and_then(|x| x.as_str()).unwrap_or(""),
                aliases.join(", ")
            );
        }
    }
    0
}

fn cmd_path(args: &[String]) -> i32 {
    let Some(id) = positional(args, &[]).first().cloned() else {
        println!("yw-wiki setting path <id>");
        return 2;
    };
    match vaults(&load()).get(&id).and_then(|v| v.as_table()) {
        Some(v) => {
            println!("{}", vpath(v).display());
            0
        }
        None => wiki::fail(&format!("'{id}' 볼트가 없다 — yw-wiki setting list")),
    }
}

fn register(folder: &str, id: Option<String>, readonly: bool, meta: Vec<(&str, toml::Value)>, dry: bool) -> i32 {
    let path = resolve(&expand(folder));
    let w = wiki_info(&path);
    let Some(vid) = id.or_else(|| w.get("id").and_then(|x| x.as_str()).map(String::from)) else {
        wiki::fail(&format!("{}에 wiki.toml [wiki] id가 없다 — --id로 정한다", short(&path)));
    };
    if !safe_id(&vid) {
        wiki::fail(&format!("볼트 id '{vid}'는 영문 · 숫자 · _ · -만 쓴다"));
    }
    let mut conf = load();
    let prev = vaults(&conf).get(&vid).and_then(|v| v.as_table()).cloned().unwrap_or_default();
    let mut entry = Table::new();
    entry.insert("path".into(), toml::Value::String(short(&path)));
    if readonly {
        entry.insert("readonly".into(), toml::Value::Boolean(true));
    }
    for (k, v) in meta {
        entry.insert(k.into(), v);
    }
    // 다시 등록해도 Obsidian 볼트를 잇는다
    for k in ["obsidian_id", "obsidian_depth"] {
        if let Some(x) = prev.get(k).filter(|x| x.as_integer() != Some(0) && x.as_str() != Some("")) {
            entry.insert(k.into(), x.clone());
        }
    }
    if let Some(pp) = vs(&prev, "path") {
        if resolve(&expand(&pp)) != path {
            entry.insert("previous_path".into(), toml::Value::String(vs(&prev, "previous_path").unwrap_or(pp)));
        }
    }
    vaults_mut(&mut conf).insert(vid.clone(), toml::Value::Table(entry));
    if !conf.contains_key("default") {
        conf.insert("default".into(), toml::Value::String(vid));
    }
    println!("{}", save(&conf, dry));
    0
}

fn register_args(args: &[String], folder: &str) -> i32 {
    let mut meta: Vec<(&str, toml::Value)> = vec![];
    if let Some(n) = opt(args, "--name").filter(|s| !s.is_empty()) {
        meta.push(("name", toml::Value::String(n)));
    }
    if let Some(d) = opt(args, "--description").filter(|s| !s.is_empty()) {
        meta.push(("description", toml::Value::String(d)));
    }
    let aliases = opts(args, "--alias");
    if !aliases.is_empty() {
        meta.push(("aliases", toml::Value::Array(aliases.into_iter().map(toml::Value::String).collect())));
    }
    // 이름 · 설명 · 별칭 순서는 Python판(name, description, aliases)과 같게
    register(folder, opt(args, "--id"), flag(args, "--readonly"), meta, flag(args, "--dry-run"))
}

fn cmd_new_vault(args: &[String]) -> i32 {
    let (Some(profile), Some(path)) = (opt(args, "--profile"), opt(args, "--path")) else {
        println!("yw-wiki setting new-vault --profile personal --path <폴더>");
        return 2;
    };
    let sdir = setting::setting_dir(None);
    let path = resolve(&expand(&path));
    let rc = setting::new_wiki(&sdir, &profile, &path, false, true);
    if rc != 0 && rc != 1 {
        return rc;
    }
    register(&path.to_string_lossy(), None, false, vec![], flag(args, "--dry-run"))
}

fn find_moved(vid: &str) -> Vec<PathBuf> {
    let mut hits = std::collections::BTreeSet::new();
    for root in search_roots() {
        let Ok(rd) = std::fs::read_dir(&root) else { continue };
        for e in rd.flatten() {
            let f = e.path().join("wiki.toml");
            if f.is_file() {
                let t: Table = wiki::read(&f).parse().unwrap_or_default();
                if wiki::tstr(&t, &["wiki", "id"]).as_deref() == Some(vid) {
                    hits.insert(e.path());
                }
            }
        }
    }
    hits.into_iter().collect()
}

fn cmd_doctor(args: &[String]) -> i32 {
    let fix = flag(args, "--fix");
    let mut conf = load();
    let mut bad = 0;
    let mut say = |ok: bool, text: String, fix_hint: &str| {
        if !ok {
            bad += 1;
        }
        println!("{} {text}{}", if ok { "✓" } else { "✗" }, if !fix_hint.is_empty() && !ok { format!("\n  → {fix_hint}") } else { String::new() });
    };
    for d in skill_dirs() {
        say(d.join("SKILL.md").exists(), format!("전역 스킬 {}", short(&d)), "yw-wiki setting install");
    }
    say(vaults_path().exists(), format!("볼트 목록 {}", short(&vaults_path())), "yw-wiki setting install");
    let sdir = setting::setting_dir(None);
    say(sdir.join("VERSION").exists(), format!("세팅 사본 {}", short(&sdir)), "vaults.toml [setting] path를 고친다");
    let sver = if sdir.join("VERSION").exists() { setting::version(&sdir) } else { "?".into() };
    say(crate::VERSION == sver, format!("실행 파일 yw-wiki v{}", crate::VERSION), "yw-wiki setting check (세팅 버전으로 다시 빌드)");
    if sdir.join(".git").exists() {
        let (tag, note) = latest_tag(&sdir);
        println!("  세팅 v{sver}{}{}", tag.map(|t| format!(" · 최신 릴리스 {t}")).unwrap_or_default(), if note.is_empty() { String::new() } else { format!(" · {note}") });
    }
    let mut changed = false;
    for (vid, v) in &vaults(&conf) {
        let v = v.as_table().cloned().unwrap_or_default();
        let mut path = vpath(&v);
        if !path.exists() {
            let hits = find_moved(vid);
            if hits.len() == 1 && fix {
                let e = vaults_mut(&mut conf).get_mut(vid).and_then(|x| x.as_table_mut()).unwrap();
                e.insert("previous_path".into(), v.get("path").cloned().unwrap_or(toml::Value::String(String::new())));
                e.insert("path".into(), toml::Value::String(short(&hits[0])));
                path = hits[0].clone();
                changed = true;
                println!("! {vid}: 경로를 {}로 고침", short(&hits[0]));
            } else {
                let cands = hits.iter().map(|h| short(h)).collect::<Vec<_>>().join(", ");
                let hint = if hits.len() == 1 {
                    format!("후보 {cands} — yw-wiki setting doctor --fix")
                } else if !hits.is_empty() {
                    format!("후보 여럿: {cands} — 사람이 고른다")
                } else {
                    "yw-wiki setting register <폴더>".into()
                };
                say(false, format!("{vid}: {} 없음", short(&path)), &hint);
                continue;
            }
        }
        let w = vault_info(&v, &path);
        say(
            w.get("id").and_then(|x| x.as_str()).unwrap_or(vid) == vid,
            format!("{vid}: {} ({})", short(&path), w.get("name").and_then(|x| x.as_str()).unwrap_or("이름 없음")),
            "wiki.toml [wiki] id가 목록과 다르다",
        );
        if v.get("readonly").and_then(|x| x.as_bool()).unwrap_or(false) {
            continue;
        }
        let st = path.join(".wiki-setting.toml");
        if st.exists() {
            let wv = wiki::read(&st).parse::<Table>().ok().and_then(|t| t.get("version").and_then(|x| x.as_str()).map(String::from)).unwrap_or("?".into());
            say(wv == sver, format!("  세팅 v{wv}"), &format!("yw-wiki setting upgrade {vid}"));
        }
        let (lines, rc) = crate::check::run(&path);
        say(rc == 0, format!("  점검: {}", lines.last().cloned().unwrap_or_default()), &format!("{}에서 '점검해줘'", short(&path)));
    }
    // 옮긴 위키는 Obsidian 볼트도 따라간다
    let (onotes, obad, ochanged) = obsidian_vaults(&mut conf, fix, false);
    for n in &onotes {
        println!("  Obsidian: {n}");
    }
    if obad {
        bad += 1;
    }
    if changed || ochanged {
        println!("{}", save(&conf, false));
    }
    println!("{}", if bad == 0 { "문제 없음".to_string() } else { format!("문제 {bad}개") });
    if bad == 0 { 0 } else { 1 }
}

/// 세팅 사본을 태그로 옮긴다 (고친 파일이 있으면 옮기지 않는다)
fn move_to_tag(sdir: &Path, tag: &str) -> Option<String> {
    let head = wiki::out(&git(sdir, &["describe", "--tags", "--exact-match"])).trim().to_string();
    if head == tag {
        return None;
    }
    if !wiki::out(&git(sdir, &["status", "--porcelain"])).trim().is_empty() {
        return Some(format!("세팅 사본에 고친 파일이 있어 {tag}로 옮기지 않음 ({})", short(sdir)));
    }
    git(sdir, &["checkout", "--quiet", "--detach", tag]);
    Some(format!("세팅 사본 → {tag}"))
}

fn cmd_upgrade(args: &[String]) -> i32 {
    let conf = load();
    let sdir = setting::setting_dir(None);
    if sdir.join(".git").exists() {
        let (tag, note) = latest_tag(&sdir);
        if !note.is_empty() {
            println!("{note}");
        }
        if let Some(t) = tag {
            if let Some(m) = move_to_tag(&sdir, &t) {
                println!("{m}");
            }
        }
    }
    if let Some(n) = ensure_binary(&sdir, &setting::version(&sdir)) {
        println!("{n}");
    }
    let yes = flag(args, "--yes");
    let id = positional(args, &[]).first().cloned();
    let all = vaults(&conf);
    let targets: Vec<(String, Table)> = match &id {
        Some(i) => match all.get(i).and_then(|v| v.as_table()) {
            Some(v) => vec![(i.clone(), v.clone())],
            None => wiki::fail(&format!("'{i}' 볼트가 없다 — yw-wiki setting list")),
        },
        None => all.iter().map(|(k, v)| (k.clone(), v.as_table().cloned().unwrap_or_default())).collect(),
    };
    let mut rc = 0;
    for (vid, v) in targets {
        if v.get("readonly").and_then(|x| x.as_bool()).unwrap_or(false) {
            continue;
        }
        println!("[{vid}]");
        let (lines, r) = setting::upgrade(&sdir, &vpath(&v), false, yes);
        for l in lines {
            println!("{l}");
        }
        if rc == 0 && r != 1 {
            rc = r;
        }
    }
    rc
}

/// 실행 파일(yw-wiki)이 세팅 버전보다 낡았으면 세팅 사본의 cli/로 다시 빌드해 ~/.local/bin에 둔다.
/// 다시 빌드한 실행 파일은 다음 명령부터 쓰인다. (메모)
fn ensure_binary(sdir: &Path, sver: &str) -> Option<String> {
    if sver.is_empty() || vt(crate::VERSION) >= vt(sver) || std::env::var_os("YW_NO_BUILD").is_some() {
        return None;
    }
    let manifest = sdir.join("cli").join("Cargo.toml");
    if !manifest.exists() {
        return Some(format!("실행 파일 v{} — 세팅 사본에 cli/가 없어 다시 빌드 못 함", crate::VERSION));
    }
    let root = wiki::home().join(".local");
    let o = std::process::Command::new("cargo")
        .args(["install", "--quiet", "--locked", "--force", "--path"])
        .arg(sdir.join("cli"))
        .arg("--root")
        .arg(&root)
        .env("CARGO_TARGET_DIR", wiki::home().join(".cache").join("yw-wiki").join("target"))
        .output();
    match o {
        Ok(o) if o.status.success() => Some(format!("실행 파일 yw-wiki v{} → v{sver}", crate::VERSION)),
        Ok(o) => Some(format!("⚠ 실행 파일 다시 빌드 실패: {}", wiki::err(&o).lines().last().unwrap_or(""))),
        Err(_) => Some("⚠ cargo 없음 — brew install rust 뒤 sh <세팅>/install.sh".into()),
    }
}

/// 세팅 사본을 main의 최신 릴리스 태그로 맞춘다. (세팅 폴더, 버전, 메모, 문제 여부)
fn ensure_setting(conf: &Table) -> (PathBuf, String, Vec<String>, bool) {
    let sdir = setting::setting_dir(None);
    let (mut notes, mut bad) = (vec![], false);
    let repo = wiki::tstr(conf, &["setting", "repo"]);
    if !sdir.exists() {
        if let Some(r) = repo {
            let o = std::process::Command::new("git").args(["clone", "--quiet", &format!("https://github.com/{r}.git")]).arg(&sdir).output();
            notes.push(match o {
                Ok(o) if o.status.success() => "세팅 사본을 받음".to_string(),
                Ok(o) => format!("세팅 사본을 못 받음: {}", wiki::err(&o).trim().chars().take(80).collect::<String>()),
                Err(e) => format!("세팅 사본을 못 받음: {e}"),
            });
        }
    }
    if !sdir.join("VERSION").exists() {
        notes.push(format!("세팅 사본 없음 ({})", short(&sdir)));
        return (sdir, "?".into(), notes, true);
    }
    if sdir.join(".git").exists() {
        let (tag, note) = latest_tag(&sdir);
        if !note.is_empty() {
            notes.push(note);
        }
        if let Some(t) = tag {
            if let Some(m) = move_to_tag(&sdir, &t) {
                bad |= m.contains("옮기지 않음");
                notes.push(m);
            }
        }
    }
    let sver = setting::version(&sdir);
    (sdir, sver, notes, bad)
}

/// 최신 확인. (출력 줄, 종료 코드). vault를 주면 그 위키만 본다
pub fn check(vault: Option<&Path>) -> (Vec<String>, i32) {
    let conf = load();
    let (sdir, sver, mut notes, mut bad) = ensure_setting(&conf);
    // 아래 볼트 확인에서 원격을 다시 받지 않는다
    std::env::set_var("YW_SETTING_CHECKED", "1");
    if let Some(n) = ensure_binary(&sdir, &sver) {
        bad |= n.starts_with('⚠');
        notes.push(n);
    }
    // 전역 스킬이 세팅과 같은 버전인지
    let src = sdir.join("global").join("wiki-vault");
    for d in skill_dirs() {
        let have = wiki::read(&d.join("VERSION")).trim().to_string();
        let newer = !have.is_empty() && vt(&have) > vt(&sver);
        if src.exists() && have != sver && !newer {
            install_skill(&src, &d, Some(&sver));
            notes.push(format!("전역 스킬 {} v{} → v{sver}", short(&d), if have.is_empty() { "?" } else { &have }));
        }
    }
    let targets: Vec<(String, PathBuf)> = match vault {
        Some(p) => vec![("(이 위키)".into(), resolve(p))],
        None => vaults(&conf)
            .iter()
            .filter(|(_, v)| !v.get("readonly").and_then(|x| x.as_bool()).unwrap_or(false))
            .map(|(k, v)| (k.clone(), vpath(v.as_table().unwrap())))
            .collect(),
    };
    for (vid, path) in targets {
        let st = path.join(".wiki-setting.toml");
        if !st.exists() {
            notes.push(format!("{vid}: 세팅으로 만든 위키가 아님 ({})", short(&path)));
            bad = true;
            continue;
        }
        let wv = wiki::read(&st).parse::<Table>().ok().and_then(|t| t.get("version").and_then(|x| x.as_str()).map(String::from)).unwrap_or("0.0.0".into());
        if vt(&wv) > vt(&sver) {
            // 위키가 더 새 버전(세팅 개발 중 시험 등) — 절대 내리지 않는다
            notes.push(format!("{vid}: 세팅 v{wv} (세팅 사본 v{sver}보다 새것 — 그대로 둠)"));
        } else if wv != sver {
            if wv.split('.').next() != sver.split('.').next() {
                notes.push(format!("{vid}: 세팅 v{wv} → v{sver}는 메이저 변경 — 확인 뒤 yw-wiki setting upgrade {vid} --yes"));
                bad = true;
            } else {
                let clean = wiki::out(&git(&path, &["status", "--porcelain"])).trim().is_empty();
                let (_, r) = setting::upgrade(&sdir, &path, false, false);
                if r == 0 && clean {
                    git(&path, &["add", "-A"]);
                    let c = git(&path, &["commit", "-q", "-m", &format!("세팅 v{wv} → v{sver} 반영 (wiki-setting check)")]);
                    let mut pushed = String::new();
                    if c.status.success()
                        && !wiki::out(&git(&path, &["remote"])).trim().is_empty()
                        && wiki::out(&git(&path, &["branch", "--show-current"])).trim() == "main"
                    {
                        // 규칙: 끝나면 push (main에서만, force 없음)
                        let pr = git(&path, &["push", "-q", "origin", "main"]);
                        pushed = if pr.status.success() { " · push".into() } else { " · push 못 함(원격이 앞섬 — 갱신해줘)".into() };
                        bad |= !pr.status.success();
                    }
                    notes.push(format!("{vid}: 세팅 v{wv} → v{sver} 반영{}{pushed}", if c.status.success() { "" } else { " (커밋 못 함 — 점검 오류)" }));
                    bad |= !c.status.success();
                } else {
                    notes.push(format!("{vid}: 세팅 v{wv} → v{sver} 반영{}", if r == 0 { " (고친 파일이 있어 커밋은 안 함)" } else { " 실패" }));
                    bad |= r != 0;
                }
            }
        }
        if vault.is_none() {
            // 볼트마다 main 기준 · 플러그인까지
            let (line, _) = crate::version::run(&path, false);
            if !line.is_empty() {
                bad |= line.split_once("] ").map(|x| x.1).unwrap_or(&line).starts_with('⚠');
                notes.push(line);
            }
        }
    }
    let line = format!("{}세팅 v{sver}{}", if bad { "⚠ " } else { "" }, if notes.is_empty() { " · 최신".to_string() } else { format!(" · {}", notes.join(" · ")) });
    (vec![line], if bad { 1 } else { 0 })
}

pub fn main(args: &[String]) -> i32 {
    let Some(cmd) = args.first() else {
        println!("{HELP}");
        return 2;
    };
    let rest = &args[1..];
    match cmd.as_str() {
        "install" => cmd_install(rest),
        "list" => cmd_list(),
        "path" => cmd_path(rest),
        "register" => match positional(rest, &["--id", "--name", "--alias", "--description"]).first() {
            Some(f) => register_args(rest, &f.clone()),
            None => {
                println!("yw-wiki setting register <폴더> [--id <id>] [--readonly] [--name …] [--alias …] [--description …]");
                2
            }
        },
        "new-vault" => cmd_new_vault(rest),
        "doctor" => cmd_doctor(rest),
        "upgrade" => cmd_upgrade(rest),
        "check" => {
            let v = opt(rest, "--vault").map(|p| resolve(&expand(&p)));
            let (lines, rc) = check(v.as_deref());
            for l in lines {
                println!("{l}");
            }
            rc
        }
        "obsidian" => cmd_obsidian(rest),
        "-h" | "--help" | "help" => {
            println!("{HELP}");
            0
        }
        other => {
            eprintln!("모르는 명령: {other}\n{HELP}");
            2
        }
    }
}

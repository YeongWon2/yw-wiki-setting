//! 세팅이 관리하는 파일 목록 · AGENTS.md 합성 · 새 위키 만들기 · 위키에 새 버전 반영.
use crate::wiki::{self, regex};
use std::path::{Path, PathBuf};

pub const PROFILES: &[&str] = &["personal"];
pub const REPO: &str = "YeongWon2/yw-wiki-setting";

pub const NEW_HELP: &str = "yw-wiki new --profile personal --dest <폴더> [--in-place] [--no-git] [--setting <세팅 폴더>]
--in-place: 이미 있는 폴더(기존 저장소)에 뼈대를 깐다. 이미 있는 raw/ · wiki/ 파일과 wiki.toml은 그대로 둔다.
만드는 것: 폴더, 템플릿 · 흐름 · 스킬 · 훅, AGENTS.md(core + profile + local), CLAUDE.md, wiki.toml,
.wiki-setting.toml, wiki/index.md · log.md, review/, .obsidian 설정, .gitignore, git init과 훅 연결, 첫 점검.";

pub const UPGRADE_HELP: &str = "yw-wiki upgrade <위키 경로> [--dry-run] [--yes] [--setting <세팅 폴더>]
세팅 관리 파일(.wiki-setting.toml의 managed, AGENTS.md의 core · profile 구간)만 덮어쓴다.
raw/ · wiki/ · wiki.toml · AGENTS.md의 local 구간 · .obsidian · .gitignore는 건드리지 않는다.
패치 · 마이너는 바로 반영하고, 메이저가 바뀌면 --yes 없이는 멈춘다(종료 코드 2). 바뀐 파일 목록은 wiki/log.md에 남긴다.";

pub fn vaults_path() -> PathBuf {
    wiki::home().join(".config").join("wiki").join("vaults.toml")
}

pub fn default_setting() -> PathBuf {
    wiki::home().join("Desktop").join("wiki").join("yw-wiki-setting")
}

/// 세팅 폴더: --setting → YW_SETTING_DIR → vaults.toml [setting] path → 기본 위치
pub fn setting_dir(explicit: Option<&str>) -> PathBuf {
    if let Some(p) = explicit {
        return wiki::resolve(&wiki::expand(p));
    }
    if let Ok(p) = std::env::var("YW_SETTING_DIR") {
        if !p.is_empty() {
            return wiki::resolve(&wiki::expand(&p));
        }
    }
    let conf: toml::Table = wiki::read(&vaults_path()).parse().unwrap_or_default();
    match wiki::tstr(&conf, &["setting", "path"]) {
        Some(p) => wiki::resolve(&wiki::expand(&p)),
        None => wiki::resolve(&default_setting()),
    }
}

pub fn version(sdir: &Path) -> String {
    wiki::read(&sdir.join("VERSION")).trim().to_string()
}

fn major(v: &str) -> String {
    v.split('.').next().unwrap_or("").to_string()
}

pub fn wiki_conf(sdir: &Path, dest: &Path, profile: &str) -> toml::Table {
    let p = dest.join("wiki.toml");
    let text = if p.exists() { wiki::read(&p) } else { wiki::read(&sdir.join("profiles").join(profile).join("wiki.toml")) };
    text.parse().unwrap_or_default()
}

/// 위키 README (3초 읽기: 첫 줄 = 무엇인지 → 30초: 말하는 법 · 폴더 → 상세: 새 PC에서 시작)
fn readme(conf: &toml::Table) -> String {
    let name = wiki::tstr(conf, &["wiki", "name"]).unwrap_or_default();
    let desc = wiki::tstr(conf, &["wiki", "description"]).unwrap_or_default();
    let raw = "`raw/daily/` 일지 · `raw/notes/` 메모 · `raw/assets/` 이미지 · PDF";
    let extra = "| `wiki/career/` | 성과 기록 (회사 내용은 걸러서) | Claude |\n";
    format!(
        "# {name}

**{desc}를 Claude가 쓰고 유지하는 LLM 위키다. 질문은 `wiki/index.md`부터 읽는다.**

| 말 | 하는 일 |
|---|---|
| 갱신해줘 | 최신 받기 · 세팅 반영 · 할 일 알리기 |
| 업데이트해줘 · 수집해줘 | 원본을 위키에 반영, 충돌 · 중복은 `review/`로 |
| 리뷰해줘 | 검토 대기를 보여 주고 정한 것만 반영 |
| 찾아줘 | 목차부터 읽고 근거 링크로 답함 |
| 점검해줘 | 자동 검사, 판단할 것은 `review/`로 |

| 폴더 | 무엇 | 누가 쓰나 |
|---|---|---|
| `raw/` | 원본: {raw} | 나 (Claude는 읽기만) |
| `wiki/` | 위키: entities · concepts · sources · synthesis · guides · decisions, 목차 `index.md` · 기록 `log.md` | Claude |
{extra}| `review/` | 사람이 정할 검토 대기 (충돌 · 중복은 위키에 쓰지 않는다) | Claude가 쌓고 내가 정함 |

## 새 PC에서 시작하기

1. Rust: `brew install rust`
2. 세팅 받고 CLI · 길잡이 깔기: `git clone https://github.com/YeongWon2/yw-wiki-setting ~/.local/share/wiki/yw-wiki-setting` → `sh ~/.local/share/wiki/yw-wiki-setting/install.sh`
3. 이 위키 받고 등록: `git clone <이 저장소> ~/Desktop/wiki/yw-wiki` → `yw-wiki setting register <그 폴더>`
4. 폴더에서 Claude Code를 열면 세션 시작 때 최신 확인(main · 세팅 · Obsidian 플러그인)이 돈다. Obsidian으로 폴더를 열면 Git 플러그인이 메모를 원격과 맞춘다.

규칙은 `AGENTS.md`, 만든 이유는 세팅의 `docs/철학.md`. 자동화는 모두 `yw-wiki` 명령(Rust CLI, 세팅의 `cli/`)이 한다. 세팅이 관리하는 파일(`_templates/` · `tools/` · `.claude/skills/` · 이 README)은 여기서 고치지 않는다.
"
    )
}

fn sorted_md(dir: &Path) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = std::fs::read_dir(dir)
        .map(|rd| rd.flatten().map(|e| e.path()).filter(|p| wiki::is_md(p) && p.is_file()).collect())
        .unwrap_or_default();
    v.sort();
    v
}

/// 세팅이 관리하는 파일: [(위키 기준 경로, 내용)]. upgrade가 덮어쓴다.
pub fn managed_files(sdir: &Path, profile: &str, name: &str, conf: Option<&toml::Table>) -> Vec<(String, String)> {
    let core = sdir.join("core");
    let mut out = Managed(vec![]);
    if let Some(c) = conf {
        out.push(("README.md".to_string(), readme(c)));
    }
    for src in sorted_md(&core.join("templates")).into_iter().chain(sorted_md(&sdir.join("profiles").join(profile).join("templates"))) {
        out.push((format!("_templates/{}", wiki::name(&src)), wiki::read(&src)));
    }
    let tools = core.join("tools");
    for src in wiki::walk(&tools) {
        if src.components().any(|c| c.as_os_str() == "__pycache__") {
            continue;
        }
        let rel = src.strip_prefix(&tools).unwrap().to_string_lossy().into_owned();
        out.push((format!("tools/{rel}"), wiki::read(&src)));
    }
    // 세션 시작 때 최신 확인
    out.push((".claude/settings.json".into(), wiki::read(&core.join("claude").join("settings.json"))));
    out.push((".githooks/pre-commit".into(), wiki::read(&core.join("githooks").join("pre-commit"))));
    let mut skills: Vec<PathBuf> = std::fs::read_dir(core.join("skills")).map(|rd| rd.flatten().map(|e| e.path()).collect()).unwrap_or_default();
    skills.sort();
    for skill in skills {
        let f = skill.join("SKILL.md");
        if f.exists() {
            out.push((format!(".claude/skills/{}/SKILL.md", wiki::name(&skill)), wiki::read(&f).replace("{{wiki_name}}", name)));
        }
    }
    out.0
}

/// 같은 경로가 또 오면 자리는 처음 그대로, 내용은 나중 것으로 (profile 템플릿이 core 템플릿을 덮는다)
struct Managed(Vec<(String, String)>);

impl Managed {
    fn push(&mut self, item: (String, String)) {
        match self.0.iter_mut().find(|(k, _)| *k == item.0) {
            Some(slot) => slot.1 = item.1,
            None => self.0.push(item),
        }
    }
}

fn executable(rel: &str) -> bool {
    rel.starts_with(".githooks/") || (rel.starts_with("tools/") && !rel[6..].contains('/') && rel.ends_with(".py"))
}

pub fn agents(sdir: &Path, profile: &str, local: Option<String>) -> String {
    let core = wiki::read(&sdir.join("core").join("AGENTS.core.md"));
    let prof = wiki::read(&sdir.join("profiles").join(profile).join("AGENTS.profile.md"));
    let local = local.unwrap_or_else(|| "# 이 위키에만 있는 규칙\n\n(없음. 이 구간은 upgrade가 건드리지 않는다.)".into());
    format!(
        "<!-- core:start v{} — yw-wiki-setting이 관리. 여기서 고치지 않는다 -->\n{}\n<!-- core:end -->\n\n\
         <!-- profile:start {profile} — yw-wiki-setting이 관리 -->\n{}\n<!-- profile:end -->\n\n\
         <!-- local:start -->\n{}\n<!-- local:end -->\n",
        version(sdir),
        core.trim_end(),
        prof.trim_end(),
        local.trim()
    )
}

pub fn local_section(text: &str) -> Option<String> {
    regex!(r"(?s)<!-- (core|profile|local):start[^>]*-->\n(.*?)\n<!-- \1:end -->")
        .captures_iter(text)
        .filter_map(|c| c.ok())
        .find(|c| &c[1] == "local")
        .map(|c| c[2].to_string())
}

fn state_toml(sdir: &Path, profile: &str, managed: &[(String, String)]) -> String {
    let mut paths: Vec<&str> = managed.iter().map(|(p, _)| p.as_str()).collect();
    paths.sort();
    let items = paths.iter().map(|p| format!("  \"{p}\"")).collect::<Vec<_>>().join(",\n");
    format!(
        "# yw-wiki-setting이 쓰는 파일. 손으로 고치지 않는다.\nversion = \"{}\"\nprofile = \"{profile}\"\nsetting_repo = \"{REPO}\"\nmanaged = [\n{items},\n]\n",
        version(sdir)
    )
}

/// .gitignore에 세팅이 정한 규칙이 다 있게 한다(위키가 더한 줄은 그대로). 이미 git에 올라간 무시 대상은 추적만 뺀다(파일은 남김).
fn ensure_gitignore(sdir: &Path, dest: &Path) -> Vec<String> {
    let path = dest.join(".gitignore");
    let have = wiki::read(&path);
    let have_rules: std::collections::HashSet<&str> = have.lines().map(|l| l.trim()).collect();
    let base = wiki::read(&sdir.join("core").join("gitignore"));
    let missing: Vec<&str> = base.lines().filter(|l| !l.trim().is_empty() && !l.starts_with('#')).filter(|l| !have_rules.contains(l.trim())).collect();
    let mut changed = vec![];
    if !missing.is_empty() {
        let block = format!("\n# ── yw-wiki-setting이 더한 규칙 ──\n{}\n", missing.join("\n"));
        let text = if have.is_empty() { base.clone() } else { format!("{}\n{block}", have.trim_end_matches('\n')) };
        wiki::write(&path, &text);
        changed.push(format!(".gitignore 규칙 {}개", missing.len()));
    }
    if dest.join(".git").exists() {
        let tracked: Vec<String> = wiki::out(&wiki::git(dest, &["ls-files", "-ci", "--exclude-standard"])).split('\n').filter(|t| !t.is_empty()).map(String::from).collect();
        if !tracked.is_empty() {
            let mut args = vec!["rm", "-q", "--cached", "--"];
            args.extend(tracked.iter().map(|s| s.as_str()));
            wiki::git(dest, &args);
            changed.push(format!("git 추적에서 뺌: {}", tracked.join(", ")));
        }
    }
    changed
}

fn write(dest: &Path, rel: &str, text: &str) {
    let path = dest.join(rel);
    wiki::write(&path, text);
    #[cfg(unix)]
    if executable(rel) {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755));
    }
}

/// 새 위키를 만든다. 종료 코드는 첫 점검 결과
pub fn new_wiki(sdir: &Path, profile: &str, dest: &Path, in_place: bool, git: bool) -> i32 {
    if !PROFILES.contains(&profile) {
        eprintln!("없는 프로필: {profile} (있는 것: {})", PROFILES.join(", "));
        return 2;
    }
    if dest.exists() && std::fs::read_dir(dest).map(|mut r| r.next().is_some()).unwrap_or(false) && !in_place {
        wiki::fail(&format!("{}가 비어 있지 않다. 기존 폴더에 깔려면 --in-place", dest.display()));
    }
    let _ = std::fs::create_dir_all(dest);
    let prof_dir = sdir.join("profiles").join(profile);
    if !dest.join("wiki.toml").exists() {
        let _ = std::fs::copy(prof_dir.join("wiki.toml"), dest.join("wiki.toml"));
    }
    let conf = wiki::config(dest);
    let name = wiki::tstr(&conf, &["wiki", "name"]).unwrap_or_else(|| wiki::name(dest));

    let mut dirs: Vec<String> = ["wiki/concepts", "wiki/sources", "wiki/synthesis", "wiki/decisions", "review"].iter().map(|s| s.to_string()).collect();
    dirs.extend(wiki::tlist(&conf, &["entities", "kinds"]).iter().map(|k| format!("wiki/entities/{k}")));
    dirs.extend(wiki::tlist(&conf, &["folders", "wiki"]).iter().map(|d| format!("wiki/{d}")));
    dirs.extend(wiki::tlist(&conf, &["folders", "raw"]).iter().map(|d| format!("raw/{d}")));
    for d in &dirs {
        let p = dest.join(d);
        let _ = std::fs::create_dir_all(&p);
        if std::fs::read_dir(&p).map(|mut r| r.next().is_none()).unwrap_or(false) {
            let _ = std::fs::write(p.join(".gitkeep"), "");
        }
    }
    let managed = managed_files(sdir, profile, &name, Some(&conf));
    for (rel, text) in &managed {
        write(dest, rel, text);
    }
    let old = wiki::read(&dest.join("AGENTS.md"));
    write(dest, "AGENTS.md", &agents(sdir, profile, local_section(&old)));
    write(dest, "CLAUDE.md", "@AGENTS.md\n");
    write(dest, ".wiki-setting.toml", &state_toml(sdir, profile, &managed));

    let obs = sdir.join("core").join("obsidian");
    for src in sorted_json(&obs.join("common")).into_iter().chain(sorted_json(&obs.join(profile))) {
        write(dest, &format!(".obsidian/{}", wiki::name(&src)), &wiki::read(&src));
    }
    // 프로필 시드 페이지(입구 지도 등): 없을 때만 한 번 깐다. 이미 있는 위키 페이지는 덮어쓰지 않는다
    let seed = prof_dir.join("seed");
    for src in wiki::walk(&seed).into_iter().filter(|p| wiki::is_md(p)) {
        let rel = src.strip_prefix(&seed).unwrap().to_string_lossy().into_owned();
        if !dest.join(&rel).exists() {
            write(dest, &rel, &wiki::read(&src).replace("{{date}}", &wiki::today()));
        }
    }
    ensure_gitignore(sdir, dest);
    if !dest.join("wiki").join("log.md").exists() {
        crate::log::append(dest, "setup", &format!("yw-wiki-setting v{} · profile {profile}", version(sdir)), &[]);
    }
    crate::index::write(dest);
    // Obsidian 플러그인 · 기본값 (오프라인이면 알리고 계속)
    for l in crate::plugins::run(dest, false).0 {
        println!("{l}");
    }
    if git {
        if !dest.join(".git").exists() {
            std::process::Command::new("git").args(["init", "-q", "-b", "main"]).arg(dest).status().ok();
        }
        wiki::git(dest, &["config", "core.hooksPath", ".githooks"]);
    }
    let (lines, rc) = crate::check::run(dest);
    for l in lines {
        println!("{l}");
    }
    rc
}

fn sorted_json(dir: &Path) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = std::fs::read_dir(dir)
        .map(|rd| rd.flatten().map(|e| e.path()).filter(|p| p.extension().map(|e| e == "json").unwrap_or(false)).collect())
        .unwrap_or_default();
    v.sort();
    v
}

/// 위키에 이 세팅을 반영한다. (출력 줄, 종료 코드)
pub fn upgrade(sdir: &Path, dest: &Path, dry_run: bool, yes: bool) -> (Vec<String>, i32) {
    let state_path = dest.join(".wiki-setting.toml");
    if !state_path.exists() {
        return (vec![format!("{}에 .wiki-setting.toml이 없다 — yw-wiki new로 만든 위키가 아니다", dest.display())], 1);
    }
    let state: toml::Table = wiki::read(&state_path).parse().unwrap_or_default();
    let profile = state.get("profile").and_then(|v| v.as_str()).unwrap_or("personal").to_string();
    let old_v = state.get("version").and_then(|v| v.as_str()).unwrap_or("0.0.0").to_string();
    let new_v = version(sdir);
    if major(&new_v) != major(&old_v) && !yes {
        return (vec![format!("메이저 버전 변경 v{old_v} → v{new_v}. 세팅 README의 마이그레이션 안내를 읽고 --yes로 다시 실행한다.")], 2);
    }
    let conf = wiki_conf(sdir, dest, &profile);
    let name = wiki::tstr(&conf, &["wiki", "name"]).unwrap_or_else(|| wiki::name(dest));
    let managed = managed_files(sdir, &profile, &name, Some(&conf));
    let mut changed: Vec<String> = managed
        .iter()
        .filter(|(rel, text)| !dest.join(rel).exists() || wiki::read(&dest.join(rel)) != *text)
        .map(|(rel, _)| rel.clone())
        .collect();
    let old_managed: Vec<String> = state
        .get("managed")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect())
        .unwrap_or_default();
    let removed: Vec<String> = old_managed.into_iter().filter(|rel| !managed.iter().any(|(m, _)| m == rel) && dest.join(rel).exists()).collect();
    let agents_old = wiki::read(&dest.join("AGENTS.md"));
    let agents_new = agents(sdir, &profile, local_section(&agents_old));
    if agents_new != agents_old {
        changed.push("AGENTS.md (core · profile)".into());
    }
    if !dry_run {
        changed.extend(ensure_gitignore(sdir, dest));
    }
    if changed.is_empty() && removed.is_empty() && old_v == new_v {
        return (vec![format!("최신 (v{new_v})")], 0);
    }
    let mut lines = vec![format!("v{old_v} → v{new_v}")];
    lines.extend(changed.iter().map(|r| format!("  바뀜  {r}")));
    lines.extend(removed.iter().map(|r| format!("  지움  {r}")));
    if dry_run {
        return (lines, 0);
    }
    for (rel, text) in &managed {
        write(dest, rel, text);
    }
    for rel in &removed {
        let p = dest.join(rel);
        let _ = std::fs::remove_file(&p);
        // 비어 버린 스킬 폴더 등
        let mut parent = p.parent().map(|x| x.to_path_buf());
        while let Some(d) = parent {
            if d == dest || !d.exists() || std::fs::read_dir(&d).map(|mut r| r.next().is_some()).unwrap_or(true) {
                break;
            }
            let _ = std::fs::remove_dir(&d);
            parent = d.parent().map(|x| x.to_path_buf());
        }
    }
    write(dest, "AGENTS.md", &agents_new);
    write(dest, ".wiki-setting.toml", &state_toml(sdir, &profile, &managed));
    let details: Vec<String> = changed.iter().map(|r| format!("바뀜 {r}")).chain(removed.iter().map(|r| format!("지움 {r}"))).collect();
    crate::log::append(dest, "upgrade", &format!("yw-wiki-setting v{old_v} → v{new_v}"), &details);
    crate::index::write(dest);
    (lines, 0)
}

fn opt<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).map(|s| s.as_str())
}

pub fn new_main(args: &[String]) -> i32 {
    let (Some(profile), Some(dest)) = (opt(args, "--profile"), opt(args, "--dest")) else {
        println!("{NEW_HELP}");
        return 2;
    };
    let sdir = setting_dir(opt(args, "--setting"));
    let dest = wiki::resolve(&wiki::expand(dest));
    new_wiki(&sdir, profile, &dest, args.iter().any(|a| a == "--in-place"), !args.iter().any(|a| a == "--no-git"))
}

pub fn upgrade_main(args: &[String]) -> i32 {
    let skip: Vec<&str> = vec!["--setting"];
    let mut wiki_arg = None;
    let mut i = 0;
    while i < args.len() {
        if skip.contains(&args[i].as_str()) {
            i += 2;
            continue;
        }
        if !args[i].starts_with('-') && wiki_arg.is_none() {
            wiki_arg = Some(args[i].clone());
        }
        i += 1;
    }
    let Some(w) = wiki_arg else {
        println!("{UPGRADE_HELP}");
        return 2;
    };
    let sdir = setting_dir(opt(args, "--setting"));
    let dest = wiki::resolve(&wiki::expand(&w));
    let (lines, rc) = upgrade(&sdir, &dest, args.iter().any(|a| a == "--dry-run"), args.iter().any(|a| a == "--yes"));
    for l in lines {
        println!("{l}");
    }
    rc
}

//! 작업 흐름 하네스: 스킬의 단계를 하나씩 안내하고, 단계마다 검사를 통과해야 다음으로 넘긴다.
use crate::wiki;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;

pub const HELP: &str = "yw-wiki flow list                 흐름 목록
yw-wiki flow start <흐름>          시작 (update · update-code · query · review · lint)
yw-wiki flow next                 지금 단계 보기
yw-wiki flow done [--note \"…\"]    지금 단계 검사 → 통과하면 다음 단계
yw-wiki flow skip --note \"이유\"    optional 단계만 건너뜀
yw-wiki flow abort                흐름 버림

흐름은 tools/flows/<이름>.toml, 상태는 .yw-wiki/flow.json(git에 안 올림).
검사 종류(gates): note(무엇을 했는지 --note 필수) · run:<명령>(종료 0) · run?:<명령>(보여 주기만) ·
pull(원격이 있으면 git pull --rebase) · changed:<경로>(흐름 시작 뒤 그 아래 파일이 바뀜) ·
log:<작업>(시작 뒤 wiki/log.md에 그 작업 기록) · check(yw-wiki check 오류 0) · pushed(커밋 · push 끝남)
명령의 {bin}은 이 실행 파일(yw-wiki)이다.";

fn git(root: &Path, args: &[&str]) -> std::process::Output {
    let mut a = vec!["-c", "core.quotepath=false"];
    a.extend_from_slice(args);
    wiki::git(root, &a)
}

fn flows_dir(root: &Path) -> PathBuf {
    root.join("tools").join("flows")
}

fn flow_names(root: &Path) -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(flows_dir(root))
        .map(|rd| rd.flatten().map(|e| e.path()).filter(|p| p.extension().map(|e| e == "toml").unwrap_or(false)).map(|p| wiki::stem(&p)).collect())
        .unwrap_or_default();
    v.sort();
    v
}

struct Step {
    id: String,
    doc: String,
    gates: Vec<String>,
    optional: bool,
}

struct Flow {
    title: String,
    steps: Vec<Step>,
}

fn load_flow(root: &Path, name: &str) -> Flow {
    let path = flows_dir(root).join(format!("{name}.toml"));
    if !path.exists() {
        wiki::fail(&format!("없는 흐름: {name} (있는 것: {})", flow_names(root).join(", ")));
    }
    let t: toml::Table = wiki::read(&path).parse().unwrap_or_else(|e| wiki::fail(&format!("{} 읽기 실패: {e}", path.display())));
    let steps = t
        .get("step")
        .and_then(|s| s.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|s| s.as_table())
                .map(|s| Step {
                    id: s.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    doc: s.get("do").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    gates: s.get("gates").and_then(|v| v.as_array()).map(|g| g.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default(),
                    optional: s.get("optional").and_then(|v| v.as_bool()).unwrap_or(false),
                })
                .collect()
        })
        .unwrap_or_default();
    Flow { title: t.get("title").and_then(|v| v.as_str()).unwrap_or(name).to_string(), steps }
}

fn state_path(root: &Path) -> PathBuf {
    root.join(".yw-wiki").join("flow.json")
}

fn load_state(root: &Path) -> Value {
    let p = state_path(root);
    if !p.exists() {
        wiki::fail("진행 중인 흐름이 없다. `yw-wiki flow start <흐름>`으로 시작한다.");
    }
    serde_json::from_str(&wiki::read(&p)).unwrap_or_else(|e| wiki::fail(&format!("흐름 상태를 읽지 못함: {e}")))
}

fn save_state(root: &Path, state: &Value) {
    let mut buf = Vec::new();
    let fmt = serde_json::ser::PrettyFormatter::with_indent(b" ");
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, fmt);
    serde::Serialize::serialize(state, &mut ser).ok();
    wiki::write(&state_path(root), &String::from_utf8_lossy(&buf));
}

fn log_count(root: &Path, action: &str) -> usize {
    wiki::read(&root.join("wiki").join("log.md")).matches(&format!("] {action} |")).count()
}

fn changed_paths(root: &Path, start_head: Option<&str>) -> Vec<String> {
    let mut out: Vec<String> = wiki::out(&git(root, &["status", "--porcelain", "-uall"]))
        .lines()
        .filter(|l| l.len() > 3)
        .map(|l| l[3..].trim_matches('"').rsplit(" -> ").next().unwrap_or("").to_string())
        .collect();
    if let Some(h) = start_head {
        out.extend(wiki::out(&git(root, &["diff", "--name-only", h, "HEAD"])).lines().map(String::from));
    }
    out
}

/// 흐름 파일(세팅 관리)의 명령만 돈다. {bin}은 이 실행 파일. 옛 흐름의 `{py} tools/<x>.py`도 같은 명령으로 돈다.
fn run_cmd(root: &Path, cmd: &str) -> (i32, String) {
    let mut argv = shlex::split(cmd).unwrap_or_default();
    let exe = std::env::current_exe().map(|p| p.to_string_lossy().into_owned()).unwrap_or_else(|_| "yw-wiki".into());
    if argv.len() >= 2 && argv[0] == "{py}" && argv[1].starts_with("tools/") && argv[1].ends_with(".py") {
        let sub = argv[1].trim_start_matches("tools/").trim_end_matches(".py").to_string();
        argv.splice(0..2, [exe.clone(), sub]);
    }
    for a in argv.iter_mut() {
        if a == "{bin}" {
            *a = exe.clone();
        }
    }
    if argv.is_empty() {
        return (2, "빈 명령".into());
    }
    match Command::new(&argv[0]).args(&argv[1..]).current_dir(root).output() {
        Ok(o) => (o.status.code().unwrap_or(1), format!("{}{}", wiki::out(&o), wiki::err(&o)).trim().to_string()),
        Err(e) => (127, format!("{}: {e}", argv[0])),
    }
}

fn tail(text: &str) {
    let lines: Vec<&str> = text.lines().collect();
    for l in &lines[lines.len().saturating_sub(15)..] {
        println!("  │ {l}");
    }
}

fn gate(root: &Path, state: &Value, spec: &str, note: Option<&str>) -> (bool, String) {
    let (kind, arg) = spec.split_once(':').unwrap_or((spec, ""));
    match kind {
        "note" => (note.map(|n| !n.trim().is_empty()).unwrap_or(false), "--note로 무엇을 했는지(또는 사용자 답) 적는다".into()),
        "run" | "run?" => {
            let (rc, text) = run_cmd(root, arg);
            if !text.is_empty() {
                tail(&text);
            }
            (rc == 0 || kind == "run?", format!("`{arg}` 종료 코드 {rc}"))
        }
        "pull" => {
            if wiki::out(&git(root, &["rev-parse", "--abbrev-ref", "@{u}"])).trim().is_empty() {
                return (true, "원격 없음".into());
            }
            (git(root, &["pull", "--rebase"]).status.success(), "git pull --rebase 실패 — 충돌이면 멈추고 사람에게 넘긴다".into())
        }
        "changed" => {
            let hit = changed_paths(root, state["head"].as_str()).iter().any(|p| p.starts_with(arg));
            (hit, format!("흐름 시작 뒤 `{arg}` 아래 바뀐 파일이 없다"))
        }
        "log" => {
            let before = state["logs"][arg].as_u64().unwrap_or(0) as usize;
            (log_count(root, arg) > before, format!("`yw-wiki log {arg} …` 기록이 없다"))
        }
        "check" => {
            let (lines, rc) = crate::check::run(root);
            if rc != 0 {
                tail(&lines.join("\n"));
            }
            (rc == 0, "yw-wiki check 오류를 고친다".into())
        }
        "pushed" => {
            if !wiki::out(&git(root, &["status", "--porcelain"])).trim().is_empty() {
                return (false, "커밋하지 않은 변경이 있다".into());
            }
            if !wiki::out(&git(root, &["rev-parse", "--abbrev-ref", "@{u}"])).trim().is_empty() {
                let ahead = wiki::out(&git(root, &["rev-list", "--count", "@{u}..HEAD"])).trim().to_string();
                return (ahead == "0", format!("push하지 않은 커밋 {ahead}개"));
            }
            (true, "원격 없음".into())
        }
        _ => (false, format!("모르는 검사: {spec}")),
    }
}

fn show(state: &Value, flow: &Flow) {
    let i = state["index"].as_u64().unwrap_or(0) as usize;
    if i >= flow.steps.len() {
        println!("[{}] 모든 단계 끝 ✓", flow.title);
        return;
    }
    let s = &flow.steps[i];
    let opt = if s.optional { " (optional — 해당 없으면 skip --note)" } else { "" };
    println!("[{}] {}/{} {}{opt}", flow.title, i + 1, flow.steps.len(), s.id);
    println!("{}", s.doc.trim());
    let note = if s.gates.iter().any(|g| g == "note") { " --note \"…\"`" } else { "`" };
    println!("검사: {} → `yw-wiki flow done{note}", s.gates.join(", "));
}

pub fn main(args: &[String]) -> i32 {
    let (root, rest) = match args.first().and_then(|a| wiki::wiki_root_arg(a)) {
        Some(r) => (r, args[1..].to_vec()),
        None => (wiki::find_root(), args.to_vec()),
    };
    if rest.is_empty() {
        println!("{HELP}");
        return 2;
    }
    let cmd = rest[0].as_str();
    let rest = &rest[1..];
    let note = rest.iter().position(|a| a == "--note").and_then(|i| rest.get(i + 1)).map(|s| s.as_str());

    match cmd {
        "list" => {
            for n in flow_names(&root) {
                println!("{n}: {}", load_flow(&root, &n).title);
            }
            return 0;
        }
        "start" => {
            let Some(name) = rest.first() else {
                println!("흐름 이름이 필요하다: {}", flow_names(&root).join(", "));
                return 2;
            };
            let flow = load_flow(&root, name);
            if state_path(&root).exists() {
                let old: Value = serde_json::from_str(&wiki::read(&state_path(&root))).unwrap_or(json!({}));
                println!("(진행 중이던 흐름 {}를 버리고 새로 시작)", old["flow"].as_str().unwrap_or("?"));
            }
            let head = wiki::out(&git(&root, &["rev-parse", "HEAD"])).trim().to_string();
            let mut logs = serde_json::Map::new();
            for s in &flow.steps {
                for g in &s.gates {
                    if let Some(a) = g.strip_prefix("log:") {
                        logs.insert(a.to_string(), json!(log_count(&root, a)));
                    }
                }
            }
            let state = json!({
                "flow": name, "index": 0,
                "head": if head.is_empty() || head.contains(' ') { Value::Null } else { json!(head) },
                "started": chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
                "logs": logs, "notes": {},
            });
            save_state(&root, &state);
            show(&state, &flow);
            return 0;
        }
        "abort" => {
            let _ = std::fs::remove_file(state_path(&root));
            println!("흐름을 버렸다");
            return 0;
        }
        _ => {}
    }

    let mut state = load_state(&root);
    let flow = load_flow(&root, state["flow"].as_str().unwrap_or(""));
    if cmd == "next" {
        show(&state, &flow);
        return 0;
    }
    let i = state["index"].as_u64().unwrap_or(0) as usize;
    if i >= flow.steps.len() {
        show(&state, &flow);
        return 0;
    }
    let step = &flow.steps[i];
    match cmd {
        "skip" => {
            if !step.optional {
                println!("{}는 건너뛸 수 없는 단계다", step.id);
                return 1;
            }
            if note.is_none() {
                println!("skip에는 --note로 이유를 적는다");
                return 1;
            }
        }
        "done" => {
            let mut fails = vec![];
            for spec in &step.gates {
                let (ok, why) = gate(&root, &state, spec, note);
                if !ok {
                    fails.push(format!("  ✗ {spec} — {why}"));
                }
            }
            if !fails.is_empty() {
                println!("[{}] {} 검사 실패:", flow.title, step.id);
                println!("{}", fails.join("\n"));
                return 1;
            }
        }
        _ => {
            println!("{HELP}");
            return 2;
        }
    }
    let text = format!("{}{}", if cmd == "skip" { "(건너뜀) " } else { "" }, note.unwrap_or(""));
    state["notes"][step.id.as_str()] = json!(text);
    state["index"] = json!(i + 1);
    save_state(&root, &state);
    println!("✓ {}", step.id);
    show(&state, &flow);
    if i + 1 >= flow.steps.len() {
        let _ = std::fs::remove_file(state_path(&root));
    }
    0
}

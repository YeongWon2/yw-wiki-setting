//! LLM 위키 점검: AGENTS.md 중 기계로 볼 수 있는 것만 본다.
use crate::wiki::{self, mget, mtruthy, regex, Meta, Val};
use chrono::NaiveDate;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::{Path, PathBuf};

pub const HELP: &str = "yw-wiki check [위키 경로]   점검 (오류가 있으면 종료 코드 1)

오류(종료 코드 1)
- wiki/ 페이지에 frontmatter가 없거나 공통 필드가 빠졌다, type · status · confidence · entity_kind · decision_status 값이 틀렸다
- type과 폴더가 맞지 않는다, 템플릿(_templates/<type>.md)의 ## 제목이 빠졌거나 순서가 다르다
- 날짜(created · updated · date · checked)가 YYYY-MM-DD가 아니거나, updated가 created보다 이르거나, 오늘보다 뒤다
- 템플릿 자리표시({{date}} · {{title}})가 남아 있다, 코드 블록(```)이 닫히지 않았다, 노션식 서식이 남아 있다
- 없는 문서로 가는 [[링크]], 없는 파일로 가는 마크다운 링크, 로컬 절대 경로 링크(/Users/… · ~/…)
- 충돌 내용이 위키에 있다 (## Contradictions 칸, wiki/conflicts.md) — 충돌은 review/에 둔다
- review/ 항목의 frontmatter가 틀렸다, raw/ 파일을 지우거나 옮겼다 (커밋에 올린 변경 기준), index.md가 최신이 아니다
- [guard] terms · patterns가 [guard] allow 밖의 파일에 있다
- guide: guide_kind · scope · maturity · owner_checked 값, Rules 번호 목록, Evidence 줄 형식, tested · proven 조건,
  방식 guide의 Evidence · How it was done, 근거, superseded_by, owner_checked_on과 그날 log의 confirm 기록
경고
- raw/ 파일을 고쳤다, 같은 이름의 페이지 · 고아 페이지 · 이름이 거의 같은 페이지 · 별칭이 다른 페이지 이름과 같음
- Key facts 12개 초과, 진행 중인 project · course를 30일 넘게 고치지 않음, source 확인 30일 넘음, proposed 결정 14일 넘음
- review/ 검토 대기 · 14일 넘은 대기, seed guide 90일, 확인하지 않은 guide 수
오늘 날짜는 YW_TODAY(YYYY-MM-DD)로 바꿔 볼 수 있다.";

const REQUIRED: &[&str] = &["title", "type", "status", "confidence", "sources", "related", "created", "updated"];
const STATUSES: &[&str] = &["active", "archived", "superseded"];
const CONFIDENCE: &[&str] = &["high", "low", "medium"];
const ENTITY_KINDS: &[&str] = &["course", "org", "person", "project"];
const DECISION_STATUSES: &[&str] = &["accepted", "proposed", "superseded"];
const DATE_KEYS: &[&str] = &["created", "updated", "date", "checked"];
const MAX_FACTS: usize = 12;
const STALE_DAYS: i64 = 30;
const PROPOSAL_DAYS: i64 = 14;
const REVIEW_DAYS: i64 = 14;
const REVIEW_KINDS: &[&str] = &["conflict", "duplicate", "orphan", "raw-change", "stale"];
const GUIDE_KINDS: &[&str] = &["convention", "playbook", "principle"];
const MATURITY: &[&str] = &["proven", "seed", "tested"];
const SEED_DAYS: i64 = 90;

fn plist(items: &[&str]) -> String {
    wiki::py_list(&items.iter().map(|s| s.to_string()).collect::<Vec<_>>())
}

fn now() -> NaiveDate {
    wiki::parse_date(&wiki::today()).unwrap_or_else(|| chrono::Local::now().date_naive())
}

fn headings(body: &str) -> Vec<String> {
    let body = wiki::sub(regex!(r"(?s)```.*?```"), body, "");
    regex!(r"(?m)^##\s+(.+)$")
        .captures_iter(&body)
        .filter_map(|c| c.ok())
        .map(|c| c.get(1).unwrap().as_str().trim().to_string())
        .collect()
}

fn in_order(needed: &[String], have: &[String]) -> bool {
    let mut it = have.iter();
    needed.iter().all(|h| it.any(|x| x == h))
}

/// `## <name>` 아래 내용 (다음 `## `까지). 없으면 None
fn section_raw(body: &str, name: &str) -> Option<String> {
    let lines: Vec<&str> = body.split('\n').collect();
    let head = regex!(r"^## (.+?)\s*$");
    let start = lines.iter().position(|l| {
        head.captures(l).ok().flatten().map(|c| c.get(1).unwrap().as_str() == name).unwrap_or(false)
    })?;
    let mut out = Vec::new();
    for l in &lines[start + 1..] {
        if l.starts_with("## ") {
            break;
        }
        out.push(*l);
    }
    Some(out.join("\n"))
}

fn section(body: &str, name: &str) -> String {
    section_raw(body, name).map(|s| wiki::sub(regex!(r"(?s)<!--.*?-->"), &s, "")).unwrap_or_default()
}

fn key_facts(body: &str) -> usize {
    section_raw(body, "Key facts")
        .map(|s| s.lines().filter(|l| regex!(r"^- \S").is_match(l).unwrap_or(false)).count())
        .unwrap_or(0)
}

/// log.md에 그날 confirm 기록이 있고 그 기록(제목 · 세부)에 페이지 이름이 있나.
fn confirmed(log_text: &str, day: &str, stem: &str) -> bool {
    let prefix = format!("## [{day}] confirm |");
    let mut blocks: Vec<String> = vec![String::new()];
    for line in log_text.split_inclusive('\n') {
        if line.starts_with("## [") {
            blocks.push(String::new());
        }
        blocks.last_mut().unwrap().push_str(line);
    }
    blocks.iter().any(|b| b.starts_with(&prefix) && b.contains(stem))
}

fn date_of(meta: &Meta, key: &str) -> Option<NaiveDate> {
    match meta.get(key) {
        Some(Val::S(s)) => wiki::parse_date(s),
        _ => None,
    }
}

struct Out {
    errors: Vec<String>,
    warnings: Vec<String>,
}

/// guide 점검. 확인한(또는 대체 · 보관된) guide면 true
fn check_guide(rel: &str, path: &Path, meta: &Meta, body: &str, log_text: &str, today: NaiveDate, o: &mut Out) -> bool {
    let e = &mut o.errors;
    if !GUIDE_KINDS.contains(&mget(meta, "guide_kind").as_str()) {
        e.push(format!("{rel}: guide_kind는 {} 중 하나", plist(GUIDE_KINDS)));
    }
    let scope: Option<&Vec<String>> = match meta.get("scope") {
        Some(Val::L(l)) => Some(l),
        _ => None,
    };
    let scope_re = regex!(r"^(general|(stack|domain|project|area|org):\S.*)$");
    if !matches!(scope, Some(l) if !l.is_empty() && l.iter().all(|s| scope_re.is_match(s).unwrap_or(false))) {
        e.push(format!("{rel}: scope는 [general] · stack:<기술> · domain:<분야> · project:<이름> · area:<생활 영역> · org:<회사> 목록"));
    }
    let maturity = mget(meta, "maturity");
    let owner = mget(meta, "owner_checked").to_lowercase();
    if !MATURITY.contains(&maturity.as_str()) {
        e.push(format!("{rel}: maturity는 {} 중 하나", plist(MATURITY)));
    }
    if owner != "true" && owner != "false" {
        e.push(format!("{rel}: owner_checked는 true · false"));
    }
    let rules_text = section(body, "Rules");
    let rules: Vec<&str> = rules_text.lines().filter(|l| !l.trim().is_empty()).collect();
    if rules.is_empty() || !rules.iter().all(|l| regex!(r"^(\d+\.\s+\S|\s+\S)").is_match(l).unwrap_or(false)) {
        e.push(format!("{rel}: ## Rules는 번호 목록(1. …)으로 쓴다"));
    }
    let mut places = BTreeSet::new();
    for line in section(body, "Evidence").lines() {
        if !line.starts_with("- ") {
            continue;
        }
        let m = regex!(r"^- (\d{4}-\d{2}-\d{2}) · ([^·]+?) · .+ — .+$").captures(line).ok().flatten();
        match m {
            Some(c) if wiki::parse_date(c.get(1).unwrap().as_str()).is_some() => {
                places.insert(c.get(2).unwrap().as_str().trim().to_string());
            }
            _ => {
                let short: String = line.chars().take(40).collect();
                e.push(format!("{rel}: Evidence 줄 형식은 '- YYYY-MM-DD · 어디서 · 결과 — 근거' ({short})"));
            }
        }
    }
    let way = scope.map(|l| l.iter().any(|s| s.starts_with("area:") || s.starts_with("org:") || s.starts_with("project:"))).unwrap_or(false);
    if way && places.is_empty() {
        e.push(format!("{rel}: 내 방식(area:) · 회사 방식(org:) · 프로젝트 방식(project:)은 실제로 한 일을 Evidence에 1줄 이상 적어야 한다"));
    }
    if way && section(body, "How it was done").trim().is_empty() {
        e.push(format!("{rel}: 내 방식 · 회사 방식 · 프로젝트 방식은 ## How it was done(어떻게 작업했나)을 채운다"));
    }
    if places.is_empty() && !mtruthy(meta, "sources") {
        e.push(format!("{rel}: 근거 없는 guide — sources나 Evidence에 근거를 적는다"));
    }
    let stem = wiki::stem(path);
    if mget(meta, "status") == "superseded" {
        let target = wiki::links(&format!("x {}", superseded_by(meta)));
        if target.is_empty() {
            e.push(format!("{rel}: superseded guide에 superseded_by(\"[[남길 페이지]]\")가 없음"));
        } else if target == vec![stem.clone()] {
            e.push(format!("{rel}: superseded_by가 자기 자신을 가리킴"));
        }
    }
    if maturity == "tested" && places.len() < 2 {
        e.push(format!("{rel}: maturity tested는 Evidence의 서로 다른 곳이 2개 이상일 때만 ({}개)", places.len()));
    }
    if maturity == "proven" && owner != "true" {
        e.push(format!("{rel}: maturity proven은 owner_checked: true일 때만"));
    }
    if owner == "true" {
        match date_of(meta, "owner_checked_on") {
            None => e.push(format!("{rel}: owner_checked: true면 owner_checked_on(YYYY-MM-DD)이 있어야 한다")),
            Some(day) => {
                if !confirmed(log_text, &day.to_string(), &stem) {
                    e.push(format!("{rel}: {day} log에 confirm 기록이 없다 — yw-wiki log confirm \"{stem}\""));
                }
            }
        }
    }
    if let Some(updated) = date_of(meta, "updated") {
        if maturity == "seed" && (today - updated).num_days() > SEED_DAYS {
            o.warnings.push(format!("{rel}: guide가 seed인 채 {SEED_DAYS}일 넘게 그대로 (updated {updated})"));
        }
    }
    // 대체 · 보관된 guide는 더 쓰지 않으므로 "확인하지 않은 guide" 수에 넣지 않는다
    let status = mget(meta, "status");
    owner == "true" || status == "superseded" || status == "archived"
}

fn superseded_by(meta: &Meta) -> String {
    match meta.get("superseded_by") {
        Some(v) if v.truthy() => v.text(),
        _ => String::new(),
    }
}

fn norm(name: &str) -> String {
    name.chars().filter(|c| c.is_alphanumeric()).flat_map(|c| c.to_lowercase()).collect()
}

/// 커밋에 올린 raw/ 변경: [(상태, 경로)]. git이 아니면 빈 목록.
fn staged_raw_changes(root: &Path) -> Vec<(char, String)> {
    let o = wiki::git(root, &["-c", "core.quotepath=false", "diff", "--cached", "--name-status", "-M", "--", "raw/"]);
    if !o.status.success() {
        return vec![];
    }
    wiki::out(&o)
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| (l.chars().next().unwrap(), l.rsplit('\t').next().unwrap_or("").to_string()))
        .collect()
}

fn rel_s(rel: &Path) -> String {
    rel.to_string_lossy().into_owned()
}

/// 점검. (출력 줄, 종료 코드)
pub fn run(root: &Path) -> (Vec<String>, i32) {
    let conf = wiki::config(root);
    let profile = wiki::tstr(&conf, &["wiki", "profile"]).unwrap_or_else(|| "personal".into());
    let mut types: HashSet<&str> = ["entity", "concept", "source", "synthesis", "decision"].into_iter().collect();
    if profile == "personal" {
        types.extend(["achievement", "guide"]);
    }
    let mut o = Out { errors: vec![], warnings: vec![] };
    let today = now();

    let files = wiki::all_files(root);
    let mut names: BTreeMap<String, Vec<PathBuf>> = BTreeMap::new();
    let mut name_order: Vec<String> = vec![];
    for (path, rel) in &files {
        let key = if wiki::is_md(path) { wiki::stem(path) } else { wiki::name(path) };
        if !names.contains_key(&key) {
            name_order.push(key.clone());
        }
        names.entry(key).or_default().push(rel.clone());
    }
    let mut targets: HashSet<String> = names.keys().cloned().collect();
    targets.extend(files.iter().map(|(p, _)| wiki::name(p)));

    let mut templates: BTreeMap<String, Vec<String>> = BTreeMap::new();
    if let Ok(rd) = std::fs::read_dir(root.join("_templates")) {
        for e in rd.flatten() {
            let p = e.path();
            if wiki::is_md(&p) {
                templates.insert(wiki::stem(&p), headings(&wiki::frontmatter(&wiki::read(&p)).1));
            }
        }
    }

    let mut incoming: HashSet<String> = HashSet::new();
    let log_text = wiki::read(&root.join("wiki").join("log.md"));
    let mut unchecked = 0;
    let mut stems: BTreeMap<String, Vec<PathBuf>> = BTreeMap::new();
    let mut stem_order: Vec<String> = vec![];
    let mut alias_of: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut alias_order: Vec<String> = vec![];

    let md_link = regex!(r#"(?<!!)\[[^\]]*\]\(<?([^)\s>]+)>?(?:\s+"[^"]*")?\)"#);
    // 링크는 wiki/ · raw/ · 루트 문서 모두에서 본다
    for (path, rel) in &files {
        if !wiki::is_md(path) {
            continue;
        }
        let text = wiki::read(path);
        let r = rel_s(rel);
        let first = wiki::rel_parts(rel)[0].clone();
        let stem = wiki::stem(path);
        let mut seen = HashSet::new();
        for name in wiki::links(&text) {
            if !seen.insert(name.clone()) {
                continue;
            }
            if !targets.contains(&name) {
                o.errors.push(format!("{r}: 없는 문서 링크 [[{name}]]"));
            } else if name != stem && first != "review" {
                incoming.insert(name);
            }
        }
        for c in md_link.captures_iter(&wiki::strip_code(&text)).filter_map(|c| c.ok()) {
            let target = c.get(1).unwrap().as_str();
            if ["http://", "https://", "mailto:", "#", "obsidian:"].iter().any(|p| target.starts_with(p)) {
                continue;
            }
            let local = wiki::unquote(target.split('#').next().unwrap_or(""));
            if local.starts_with('/') || local.starts_with('~') {
                o.errors.push(format!("{r}: 로컬 절대 경로 링크 {target} — 다른 PC에서 깨진다"));
            } else if !local.is_empty() && !path.parent().unwrap().join(&local).exists() {
                o.errors.push(format!("{r}: 없는 파일 링크 {target}"));
            }
        }
        if text.matches("```").count() % 2 == 1 {
            o.errors.push(format!("{r}: 닫히지 않은 코드 블록"));
        }
        if regex!(r"(?m)^## Contradictions\s*$").is_match(&text).unwrap_or(false) && first == "wiki" {
            o.errors.push(format!("{r}: 충돌 칸(## Contradictions)이 있음 — 충돌은 review/에 둔다"));
        }
        if regex!(r"(?m)\[!hero\]|\[!cols\]|yw-wide|^## 🔗 참조").is_match(&text).unwrap_or(false) {
            o.errors.push(format!("{r}: 노션식 서식이 남아 있음 ([!hero] · [!cols] · yw-wide · 🔗 참조)"));
        }
    }

    let placeholder = regex!(r"\{\{\s*(date|title|time)[^}]*\}\}");
    let pages = wiki::pages(root);
    for (path, rel) in &pages {
        let text = wiki::read(path);
        let r = rel_s(rel);
        let (meta, body) = wiki::frontmatter(&text);
        let Some(meta) = meta else {
            o.errors.push(format!("{r}: frontmatter 없음"));
            continue;
        };
        if placeholder.is_match(&wiki::strip_code(&text)).unwrap_or(false) {
            o.errors.push(format!("{r}: 템플릿 자리표시가 남아 있음"));
        }
        let parts = wiki::rel_parts(rel);
        let top = if parts.len() > 2 { parts[1].clone() } else { String::new() };
        for key in REQUIRED {
            let empty = matches!(meta.get(*key), Some(Val::S(s)) if s.is_empty());
            if !meta.contains_key(*key) || (empty && *key != "sources" && *key != "related") {
                o.errors.push(format!("{r}: frontmatter에 {key}가 없음"));
            }
        }
        let kind = mget(&meta, "type");
        if !kind.is_empty() && !types.contains(kind.as_str()) {
            o.errors.push(format!("{r}: type '{kind}'은 이 위키({profile})에 없는 종류"));
        } else if !kind.is_empty() && Some(top.as_str()) != wiki::type_dir(&kind) {
            o.errors.push(format!("{r}: type {kind}은 wiki/{}/에 둔다", wiki::type_dir(&kind).unwrap_or("?")));
        }
        let status = mget(&meta, "status");
        if !status.is_empty() && !STATUSES.contains(&status.as_str()) {
            o.errors.push(format!("{r}: status는 {} 중 하나 (지금 {status})", plist(STATUSES)));
        }
        let conf_v = mget(&meta, "confidence");
        if !conf_v.is_empty() && !CONFIDENCE.contains(&conf_v.as_str()) {
            o.errors.push(format!("{r}: confidence는 {} 중 하나", plist(CONFIDENCE)));
        }
        if let Some(need) = templates.get(&kind) {
            let have = headings(&body);
            if !in_order(need, &have) {
                let missing: Vec<String> = need.iter().filter(|h| !have.contains(h)).cloned().collect();
                o.errors.push(format!(
                    "{r}: 템플릿 제목 {}",
                    if missing.is_empty() { "순서가 다름".to_string() } else { format!("빠짐: {}", missing.join(", ")) }
                ));
            }
        }
        for key in DATE_KEYS {
            if let Some(v) = meta.get(*key) {
                if v.truthy() && !matches!(v, Val::S(s) if wiki::parse_date(s).is_some()) {
                    o.errors.push(format!("{r}: {key} 날짜 형식이 YYYY-MM-DD가 아님 ({})", v.text()));
                }
            }
        }
        let created = date_of(&meta, "created");
        let updated = date_of(&meta, "updated");
        if let (Some(c), Some(u)) = (created, updated) {
            if u < c {
                o.errors.push(format!("{r}: updated가 created보다 이름"));
            }
        }
        for (key, d) in [("created", created), ("updated", updated)] {
            if matches!(d, Some(d) if d > today) {
                o.errors.push(format!("{r}: {key}가 오늘보다 뒤"));
            }
        }
        if mtruthy(&meta, "period") {
            let p = mget(&meta, "period");
            let parts: Vec<&str> = p.split('~').collect();
            let part_re = regex!(r"^\d{4}(-\d{2}(-\d{2})?)?$");
            if parts.len() != 2 || !parts.iter().all(|x| x.trim().is_empty() || part_re.is_match(x.trim()).unwrap_or(false)) {
                o.errors.push(format!("{r}: period는 YYYY-MM-DD~YYYY-MM-DD (끝은 비워도 됨)"));
            }
        }
        let stem = wiki::stem(path);
        match kind.as_str() {
            "entity" => {
                let ek = mget(&meta, "entity_kind");
                if !ENTITY_KINDS.contains(&ek.as_str()) {
                    o.errors.push(format!("{r}: entity_kind는 {} 중 하나", plist(ENTITY_KINDS)));
                }
                if (ek == "project" || ek == "course") && status == "active" {
                    if let Some(u) = updated {
                        if (today - u).num_days() > STALE_DAYS {
                            o.warnings.push(format!("{r}: 진행 중인데 {STALE_DAYS}일 넘게 고치지 않음 (updated {u})"));
                        }
                    }
                }
            }
            "source" => {
                if let Some(c) = date_of(&meta, "checked") {
                    if (today - c).num_days() > STALE_DAYS {
                        o.warnings.push(format!("{r}: 원본 확인이 {STALE_DAYS}일 넘음 (checked {c})"));
                    }
                }
            }
            "decision" => {
                let ds = mget(&meta, "decision_status");
                if !DECISION_STATUSES.contains(&ds.as_str()) {
                    o.errors.push(format!("{r}: decision_status는 {} 중 하나", plist(DECISION_STATUSES)));
                }
                if ds == "superseded" && !mtruthy(&meta, "superseded_by") {
                    o.errors.push(format!("{r}: superseded 결정에 superseded_by가 없음"));
                }
                if wiki::links(&format!("x {}", superseded_by(&meta))) == vec![stem.clone()] {
                    o.errors.push(format!("{r}: superseded_by가 자기 자신을 가리킴"));
                }
                if let Some(d) = date_of(&meta, "date") {
                    if ds == "proposed" && (today - d).num_days() > PROPOSAL_DAYS {
                        o.warnings.push(format!("{r}: proposed가 {PROPOSAL_DAYS}일 넘음 ({d})"));
                    }
                }
            }
            _ => {}
        }
        if kind == "guide" && !check_guide(&r, path, &meta, &body, &log_text, today, &mut o) {
            unchecked += 1;
        }
        let n = norm(&stem);
        if !stems.contains_key(&n) {
            stem_order.push(n.clone());
        }
        stems.entry(n).or_default().push(rel.clone());
        if let Some(Val::L(aliases)) = meta.get("aliases") {
            for a in aliases.iter().filter(|a| !a.is_empty()) {
                let k = norm(a);
                if !alias_of.contains_key(&k) {
                    alias_order.push(k.clone());
                }
                alias_of.entry(k).or_default().insert(stem.clone());
            }
        }
        let facts = key_facts(&body);
        if facts > MAX_FACTS {
            o.warnings.push(format!("{r}: Key facts {facts}개 — {MAX_FACTS}개를 넘으면 나눈다"));
        }
        if !incoming.contains(&stem) {
            o.warnings.push(format!("{r}: 들어오는 링크가 없는 고아 페이지"));
        }
    }

    for key in &name_order {
        let md: Vec<String> = names[key]
            .iter()
            .filter(|r| wiki::is_md(r) && ["wiki", "raw"].contains(&wiki::rel_parts(r)[0].as_str()))
            .map(|r| rel_s(r))
            .collect();
        if md.len() > 1 {
            o.warnings.push(format!("같은 이름 '{key}': {}", md.join(", ")));
        }
    }
    for key in &stem_order {
        let rels = &stems[key];
        let distinct: BTreeSet<String> = rels.iter().map(|r| wiki::stem(r)).collect();
        if rels.len() > 1 && distinct.len() > 1 {
            o.warnings.push(format!("이름이 거의 같은 페이지(중복 후보): {}", rels.iter().map(|r| rel_s(r)).collect::<Vec<_>>().join(", ")));
        }
    }
    for key in &alias_order {
        let owners = &alias_of[key];
        let by_norm: BTreeSet<String> = stems.get(key).map(|v| v.iter().map(|r| wiki::stem(r)).collect()).unwrap_or_default();
        let clash: Vec<String> = by_norm.difference(owners).cloned().collect();
        if !clash.is_empty() {
            let own: Vec<String> = owners.iter().cloned().collect();
            o.warnings.push(format!("별칭이 다른 페이지 이름과 같음(중복 후보): {} ↔ {}", wiki::py_list(&own), wiki::py_list(&clash)));
        }
    }
    if unchecked > 0 {
        o.warnings.push(format!("내가 확인하지 않은 guide {unchecked}개 (owner_checked: false) — yw-wiki-review에서 확인"));
    }

    for (status, rel) in staged_raw_changes(root) {
        if status == 'D' || status == 'R' {
            o.errors.push(format!("{rel}: raw/ 파일을 지우거나 옮김 — raw는 고치지 않는다"));
        } else if status == 'M' {
            o.warnings.push(format!("{rel}: raw/ 파일을 고침 — 사람이 고친 것이 아니면 되돌린다"));
        }
    }

    let terms: Vec<String> = wiki::tlist(&conf, &["guard", "terms"])
        .into_iter()
        .filter(|t| !t.is_empty())
        .map(|t| fancy_regex::escape(&t).into_owned())
        .chain(wiki::tlist(&conf, &["guard", "patterns"]).into_iter().filter(|p| !p.is_empty()))
        .collect();
    if !terms.is_empty() {
        let allow: HashSet<String> = wiki::tlist(&conf, &["guard", "allow"]).into_iter().collect();
        match fancy_regex::Regex::new(&format!("(?i){}", terms.join("|"))) {
            Ok(pattern) => {
                for (path, rel) in &files {
                    if !wiki::is_md(path) || allow.contains(&wiki::stem(path)) {
                        continue;
                    }
                    if let Ok(Some(hit)) = pattern.find(&wiki::read(path)) {
                        o.errors.push(format!("{}: 이 위키에 두지 않는 말 '{}' ([guard] allow 밖)", rel_s(rel), hit.as_str()));
                    }
                }
            }
            Err(e) => o.errors.push(format!("wiki.toml [guard] 정규식 오류: {e}")),
        }
    }

    let mut pending = 0;
    let mut reviews: Vec<PathBuf> = std::fs::read_dir(root.join("review"))
        .map(|rd| rd.flatten().map(|e| e.path()).filter(|p| wiki::is_md(p) && p.is_file()).collect())
        .unwrap_or_default();
    reviews.sort();
    for path in reviews {
        let r = format!("review/{}", wiki::name(&path));
        let text = wiki::read(&path);
        let meta = wiki::frontmatter(&text).0.unwrap_or_default();
        if mget(&meta, "type") != "review" {
            o.errors.push(format!("{r}: review/에는 review 템플릿 문서만 둔다"));
            continue;
        }
        if !REVIEW_KINDS.contains(&mget(&meta, "review_kind").as_str()) {
            o.errors.push(format!("{r}: review_kind는 {} 중 하나", plist(REVIEW_KINDS)));
        }
        let st = mget(&meta, "status");
        if st != "pending" && st != "resolved" {
            o.errors.push(format!("{r}: status는 pending · resolved 중 하나"));
        }
        if placeholder.is_match(&wiki::strip_code(&text)).unwrap_or(false) {
            o.errors.push(format!("{r}: 템플릿 자리표시가 남아 있음"));
        }
        if st == "pending" {
            pending += 1;
            if let Some(c) = date_of(&meta, "created") {
                if (today - c).num_days() > REVIEW_DAYS {
                    o.warnings.push(format!("{r}: 검토 대기 {REVIEW_DAYS}일 넘음 ({c})"));
                }
            }
        }
    }
    if pending > 0 {
        o.warnings.push(format!("review/ 검토 대기 {pending}개 — yw-wiki-review로 정한다"));
    }
    if root.join("wiki").join("conflicts.md").exists() {
        o.errors.push("wiki/conflicts.md가 있음 — 충돌은 review/에 항목 하나씩 둔다".into());
    }
    let index_path = root.join("wiki").join("index.md");
    if !index_path.exists() || wiki::read(&index_path) != crate::index::render(root) {
        o.errors.push("wiki/index.md가 최신이 아님 — yw-wiki index".into());
    }

    let mut lines: Vec<String> = o.errors.iter().map(|l| format!("오류  {l}")).collect();
    lines.extend(o.warnings.iter().map(|l| format!("경고  {l}")));
    lines.push(format!("페이지 {}개 · 오류 {} · 경고 {}", pages.len(), o.errors.len(), o.warnings.len()));
    (lines, if o.errors.is_empty() { 0 } else { 1 })
}

pub fn main(args: &[String]) -> i32 {
    let (root, _) = wiki::root_and_rest(args);
    if !root.join("wiki.toml").is_file() {
        wiki::fail(&format!("{}에 wiki.toml이 없다 — 위키 폴더에서 돌리거나 경로를 준다", root.display()));
    }
    let (lines, rc) = run(&root);
    for l in lines {
        println!("{l}");
    }
    rc
}

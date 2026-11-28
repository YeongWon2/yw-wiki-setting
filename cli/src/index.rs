//! wiki/index.md를 frontmatter로 다시 만든다. 손으로 고치지 않는다.
//! 폴더(type)별로 묶고, 각 묶음 안에서 active를 먼저, archived · superseded를 뒤에 싣는다.
use crate::wiki::{self, mget};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub const HELP: &str = "yw-wiki index [위키 경로] [--check]   wiki/index.md를 다시 만든다 (--check: 최신이 아니면 종료 코드 1)";

const SECTIONS: &[(&str, &str)] = &[
    ("entities", "Entities"), ("concepts", "Concepts"), ("sources", "Sources"), ("synthesis", "Synthesis"),
    ("guides", "Guides"), ("decisions", "Decisions"), ("career", "Career"),
];

fn entry(path: &Path) -> (bool, String) {
    let (meta, body) = wiki::frontmatter(&wiki::read(path));
    let meta = meta.unwrap_or_default();
    let desc = match meta.get("description") {
        Some(v) if v.truthy() => v.text(),
        _ => wiki::summary(&body),
    };
    let status = mget(&meta, "status");
    let active = status.is_empty() || status == "active";
    let tag = if active { String::new() } else { format!(" `{status}`") };
    let tail = if desc.is_empty() { String::new() } else { format!(" — {desc}") };
    (active, format!("- [[{}]]{tag}{tail}", wiki::stem(path)))
}

pub fn render(root: &Path) -> String {
    let conf = wiki::config(root);
    let mut groups: BTreeMap<(String, String), Vec<(bool, String)>> = BTreeMap::new();
    for (path, rel) in wiki::pages(root) {
        let parts = wiki::rel_parts(&rel);
        let top = if parts.len() > 2 { parts[1].clone() } else { "(기타)".to_string() };
        let sub = if top == "entities" && parts.len() > 3 { parts[2].clone() } else { String::new() };
        groups.entry((top, sub)).or_default().push(entry(&path));
    }
    let total: usize = groups.values().map(|v| v.len()).sum();
    let name = wiki::tstr(&conf, &["wiki", "name"]).unwrap_or_else(|| wiki::name(root));
    let desc = wiki::tstr(&conf, &["wiki", "description"]).unwrap_or_default();
    let mut out: Vec<String> = vec![
        "---".into(), "title: index".into(), "type: index".into(), "---".into(), "".into(),
        format!("# {name} 목차"), "".into(),
        format!("{desc}. 페이지 {total}개. `yw-wiki index`가 만든다 — 손으로 고치지 않는다."), "".into(),
        "작업 기록은 [[log]].".into(), "".into(),
    ];
    let kinds = wiki::tlist(&conf, &["entities", "kinds"]);
    let known: BTreeSet<&str> = SECTIONS.iter().map(|(t, _)| *t).collect();
    let mut order: Vec<(String, String)> = SECTIONS.iter().map(|(t, n)| (t.to_string(), n.to_string())).collect();
    let others: BTreeSet<String> = groups.keys().map(|(t, _)| t.clone()).filter(|t| !known.contains(t.as_str())).collect();
    order.extend(others.into_iter().map(|t| (t.clone(), t)));
    for (top, title) in order {
        let mut subs: Vec<String> = groups.keys().filter(|(t, _)| *t == top).map(|(_, s)| s.clone()).collect::<BTreeSet<_>>().into_iter().collect();
        subs.sort_by_key(|s| (kinds.iter().position(|k| k == s).unwrap_or(99), s.clone()));
        for sub in subs {
            let mut items = groups[&(top.clone(), sub.clone())].clone();
            out.push(format!("## {title}") + &if sub.is_empty() { String::new() } else { format!(" — {sub}") });
            out.push(String::new());
            items.sort_by(|a, b| (!a.0, &a.1).cmp(&(!b.0, &b.1)));
            out.extend(items.iter().filter(|x| x.0).map(|x| x.1.clone()));
            let mut done: Vec<String> = items.iter().filter(|x| !x.0).map(|x| x.1.clone()).collect();
            done.sort();
            if !done.is_empty() {
                out.extend(["".to_string(), "보관 · 대체됨:".to_string(), "".to_string()]);
                out.extend(done);
            }
            out.push(String::new());
        }
    }
    out.join("\n").trim_end().to_string() + "\n"
}

/// 다시 만들고 (줄, 종료 코드)
pub fn write(root: &Path) -> String {
    let text = render(root);
    wiki::write(&root.join("wiki").join("index.md"), &text);
    format!("wiki/index.md: {}개 페이지", text.matches("\n- [[").count())
}

pub fn main(args: &[String]) -> i32 {
    let (root, rest) = wiki::root_and_rest(args);
    let target = root.join("wiki").join("index.md");
    if rest.iter().any(|a| a == "--check") {
        if !target.exists() || wiki::read(&target) != render(&root) {
            println!("wiki/index.md가 최신이 아니다 — yw-wiki index");
            return 1;
        }
        return 0;
    }
    println!("{}", write(&root));
    0
}

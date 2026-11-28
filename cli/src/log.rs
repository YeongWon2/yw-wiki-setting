//! wiki/log.md에 작업 기록 한 줄을 더한다. 추가만 하고 지난 기록은 고치지 않는다.
use crate::wiki;
use std::path::Path;

pub const HELP: &str = "yw-wiki log [위키 경로] <작업> <제목> [세부 …]
  예: yw-wiki log update \"2026-10-29 회의\" \"새 source 1\" \"갱신 entity 2\"
작업: sync · update · review · query · lint · upgrade · migrate · career · confirm · merge
merge: 겹치는 guide를 합쳤을 때(사람이 review에서 정한 뒤). 세부에 남긴 쪽 · 없앤 쪽 · 정한 이유.
confirm: 내가 guide를 \"맞다\"고 확인했을 때. 제목에 페이지 이름을 쓴다 (guide의 owner_checked: true 근거)";

const HEADER: &str = "---\ntitle: log\ntype: log\n---\n\n# 작업 기록\n\n추가만 한다. 형식: `## [YYYY-MM-DD] <작업> | <제목>`\n";

pub fn append(root: &Path, action: &str, title: &str, details: &[String]) {
    let path = root.join("wiki").join("log.md");
    let text = if path.exists() { wiki::read(&path) } else { HEADER.to_string() };
    let mut entry = format!("\n## [{}] {action} | {title}\n", wiki::today());
    for d in details {
        entry.push_str(&format!("- {d}\n"));
    }
    wiki::write(&path, &format!("{}\n{entry}", text.trim_end_matches('\n')));
}

pub fn main(args: &[String]) -> i32 {
    // 위키 경로는 첫 인자일 때만 본다 (작업 이름 review가 같은 이름 폴더와 겹쳐도 작업으로 읽는다)
    let (root, rest) = match args.first().and_then(|a| wiki::wiki_root_arg(a)) {
        Some(r) => (r, args[1..].to_vec()),
        None => (wiki::find_root(), args.to_vec()),
    };
    if rest.len() < 2 {
        println!("{HELP}");
        return 2;
    }
    append(&root, &rest[0], &rest[1], &rest[2..]);
    0
}

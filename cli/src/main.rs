//! yw-wiki: 개인 LLM 위키 자동화 CLI. 위키의 점검 · 목차 · 기록 · 흐름 하네스 · 최신 확인 · 플러그인,
//! 세팅의 새 위키 · 반영 · 링크 감사, 이 PC의 볼트 관리(setting = 예전 wiki-setting)를 한 실행 파일로 한다.
mod audit;
mod check;
mod flow;
mod global;
mod index;
mod log;
mod plugins;
mod setting;
mod version;
mod wiki;

pub const VERSION: &str = env!("YW_VERSION");

const HELP: &str = "yw-wiki — 개인 LLM 위키 자동화 CLI

위키에서 (위키 폴더 안에서 돌리거나 첫 인자로 위키 경로)
  check [위키]                 점검 (오류가 있으면 종료 코드 1, 커밋 전 훅이 돌린다)
  index [위키] [--check]       wiki/index.md 다시 만들기
  log [위키] <작업> <제목> [세부 …]   wiki/log.md에 기록 추가
  flow [위키] <list|start|next|done|skip|abort>   작업 흐름 하네스
  version [위키] [--hook] [--force]   최신 확인 (main · 세팅 · 플러그인)
  plugins [위키] [--check]     Obsidian 플러그인 · 기본값 맞추기
세팅
  new --profile personal --dest <폴더>   새 위키
  upgrade <위키> [--yes] [--dry-run]     위키에 세팅 반영
  audit-links <저장소>… [--online]       링크 · 낡은 표현 감사
이 PC
  setting <install|list|path|register|new-vault|doctor|upgrade|check|obsidian>   볼트 관리 (= wiki-setting)

각 명령은 --help로 자세히. 버전: yw-wiki --version";

fn main() {
    let mut args: Vec<String> = std::env::args().collect();
    let prog = args.remove(0);
    // 예전 이름(wiki-setting)으로 불리면 setting 명령으로
    if std::path::Path::new(&prog).file_name().map(|n| n == "wiki-setting").unwrap_or(false) {
        std::process::exit(global::main(&args));
    }
    let Some(cmd) = args.first().cloned() else {
        println!("{HELP}");
        std::process::exit(2);
    };
    let rest = &args[1..];
    let wants_help = rest.iter().any(|a| a == "-h" || a == "--help");
    let (help, run): (&str, fn(&[String]) -> i32) = match cmd.as_str() {
        "check" => (check::HELP, check::main),
        "index" => (index::HELP, index::main),
        "log" => (log::HELP, log::main),
        "flow" => (flow::HELP, flow::main),
        "version" => (version::HELP, version::main),
        "plugins" => (plugins::HELP, plugins::main),
        "new" => (setting::NEW_HELP, setting::new_main),
        "upgrade" => (setting::UPGRADE_HELP, setting::upgrade_main),
        "audit-links" => (audit::HELP, audit::main),
        "setting" => (global::HELP, global::main),
        "--version" | "-V" => {
            println!("yw-wiki {VERSION}");
            std::process::exit(0);
        }
        "-h" | "--help" | "help" => {
            println!("{HELP}");
            std::process::exit(0);
        }
        other => {
            eprintln!("모르는 명령: {other}\n\n{HELP}");
            std::process::exit(2);
        }
    };
    if wants_help && cmd != "log" {
        println!("{help}");
        std::process::exit(0);
    }
    std::process::exit(run(rest));
}

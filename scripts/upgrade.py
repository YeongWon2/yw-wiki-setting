#!/usr/bin/env python3
"""만든 위키에 이 세팅의 변경을 내려보낸다.

  python3 scripts/upgrade.py <위키 경로> [--dry-run] [--yes]

세팅 관리 파일(.wiki-setting.toml의 managed, AGENTS.md의 core · profile 구간)만 덮어쓴다.
raw/ · wiki/ · wiki.toml · AGENTS.md의 local 구간 · .obsidian · .gitignore는 건드리지 않는다.
패치 · 마이너는 바로 반영하고, 메이저가 바뀌면 --yes 없이는 멈춘다(종료 코드 2).
바뀐 파일 목록은 wiki/log.md에 남긴다.
"""
import argparse
import subprocess
import sys
import tomllib
from pathlib import Path

import setting


def main():
    p = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    p.add_argument("wiki")
    p.add_argument("--dry-run", action="store_true")
    p.add_argument("--yes", action="store_true", help="메이저 버전 변경도 반영")
    a = p.parse_args()
    dest = Path(a.wiki).expanduser().resolve()
    state_path = dest / ".wiki-setting.toml"
    if not state_path.exists():
        sys.exit(f"{dest}에 .wiki-setting.toml이 없다 — new_wiki.py로 만든 위키가 아니다")
    state = tomllib.loads(state_path.read_text())
    profile, old_v, new_v = state["profile"], state["version"], setting.version()

    if setting.major(new_v) != setting.major(old_v) and not a.yes:
        print(f"메이저 버전 변경 v{old_v} → v{new_v}. 세팅 README의 마이그레이션 안내를 읽고 --yes로 다시 실행한다.")
        return 2

    managed = setting.managed_files(profile, setting.wiki_name(dest, profile), setting.wiki_conf(dest, profile))
    changed = [rel for rel, text in managed.items()
               if not (dest / rel).exists() or (dest / rel).read_text() != text]
    removed = [rel for rel in state.get("managed", []) if rel not in managed and (dest / rel).exists()]
    agents_old = (dest / "AGENTS.md").read_text() if (dest / "AGENTS.md").exists() else ""
    agents_new = setting.agents(profile, setting.local_section(agents_old))
    if agents_new != agents_old:
        changed.append("AGENTS.md (core · profile)")

    if not changed and not removed and old_v == new_v:
        print(f"최신 (v{new_v})")
        return 0
    print(f"v{old_v} → v{new_v}")
    for rel in changed:
        print(f"  바뀜  {rel}")
    for rel in removed:
        print(f"  지움  {rel}")
    if a.dry_run:
        return 0

    for rel, text in managed.items():
        setting.write(dest, rel, text)
    for rel in removed:
        (dest / rel).unlink()
    setting.write(dest, "AGENTS.md", agents_new)
    setting.write(dest, ".wiki-setting.toml", setting.state_toml(profile, managed))
    details = [f"바뀜 {r}" for r in changed] + [f"지움 {r}" for r in removed]
    subprocess.run([sys.executable, str(dest / "tools" / "log.py"), str(dest), "upgrade",
                    f"yw-wiki-setting v{old_v} → v{new_v}", *details], check=True)
    subprocess.run([sys.executable, str(dest / "tools" / "index.py"), str(dest)], check=True, stdout=subprocess.DEVNULL)
    return 0


if __name__ == "__main__":
    sys.exit(main())

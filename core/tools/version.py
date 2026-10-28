#!/usr/bin/env python3.14
"""이 위키가 최신인지 본다: main 브랜치 · 원격 main · 세팅 최신 릴리스 · Obsidian 플러그인.

  python3.14 tools/version.py [위키 경로] [--hook] [--force]

1. git: 지금 브랜치가 main인지, origin/main보다 뒤처졌는지. 뒤처졌고 고친 파일이 없으면 pull --rebase로 받는다.
2. 세팅: 전역 실행기(wiki-setting)가 있으면 `wiki-setting check --vault <이 위키>`로 넘긴다
   (세팅 사본을 최신 릴리스 태그로 · 전역 스킬 · 이 위키 upgrade). 없으면 원격 태그만 보고 알린다.
3. 플러그인: tools/plugins.py로 맞춘다.
원격 확인은 1시간에 한 번만 한다(.yw-wiki/version-check). --force는 지금 확인한다.
--hook: 세션 시작 훅용. 한 줄 요약만 출력하고 언제나 종료 코드 0.
"""
import os
import subprocess
import sys
import time
import tomllib
from pathlib import Path

import wikilib

RUNNERS = [Path.home() / ".claude" / "skills" / "wiki-vault" / "bin" / "wiki-setting",
           Path.home() / ".agents" / "skills" / "wiki-vault" / "bin" / "wiki-setting"]
EVERY = 3600


def git(root, *args):
    return subprocess.run(["git", "-C", str(root), *args], capture_output=True, text=True)


def check_git(root, online):
    """main 기준인지. (요약 문장, 문제 여부)"""
    if not (root / ".git").exists():
        return "git 저장소 아님", True
    branch = git(root, "branch", "--show-current").stdout.strip()
    notes, bad = (["main"] if branch == "main" else []), False
    if branch != "main":
        notes.append(f"지금 브랜치 '{branch or '(분리됨)'}' — main 기준이 아니다")
        bad = True
    if not git(root, "remote").stdout.strip():
        notes.append("원격 없음")
        return " · ".join(notes) or "main", bad
    if online:
        if git(root, "fetch", "--quiet", "origin", "main").returncode != 0:
            notes.append("원격에 접속 못 함")
    counts = git(root, "rev-list", "--left-right", "--count", "HEAD...origin/main").stdout.split()
    if len(counts) == 2:
        ahead, behind = map(int, counts)
        dirty = bool(git(root, "status", "--porcelain").stdout.strip())
        if behind and branch == "main" and not dirty:
            if git(root, "pull", "--rebase", "--quiet", "origin", "main").returncode == 0:
                notes.append(f"origin/main에서 {behind}개 받음")
                behind = 0
            else:
                git(root, "rebase", "--abort")
                notes.append(f"origin/main보다 {behind}개 뒤 — 받다가 충돌, 사람이 정한다")
                bad = True
        elif behind:
            notes.append(f"origin/main보다 {behind}개 뒤" + (" (고친 파일이 있어 받지 않음)" if dirty else ""))
            bad = True
        if ahead:
            notes.append(f"push 안 한 커밋 {ahead}개")
    if notes == ["main"]:
        notes = ["main 최신"]
    return " · ".join(notes), bad


def check_setting(root, online):
    state = tomllib.loads((root / ".wiki-setting.toml").read_text())
    runner = next((r for r in RUNNERS if r.exists()), None)
    if runner:
        if not online:
            return f"세팅 v{state['version']}", False
        r = subprocess.run([sys.executable, str(runner), "check", "--vault", str(root)], capture_output=True, text=True)
        out = (r.stdout.strip().splitlines() or [r.stderr.strip()])[-1]
        return out, r.returncode != 0
    # 전역 실행기가 없으면 원격 태그만 본다
    repo = state.get("setting_repo", "")
    tags = subprocess.run(["git", "ls-remote", "--tags", "--refs", f"https://github.com/{repo}.git"],
                          capture_output=True, text=True) if online and repo else None
    if not tags or tags.returncode != 0:
        return f"세팅 v{state['version']} (최신 확인 못 함 — wiki-setting install)", False
    vers = [t.rsplit("/v", 1)[-1] for t in tags.stdout.split() if "/v" in t]
    vers = [v for v in vers if v.replace(".", "").isdigit()]
    latest = max(vers, key=lambda v: tuple(map(int, v.split("."))), default=state["version"])
    if latest != state["version"]:
        return f"세팅 v{state['version']} → 최신 v{latest} — wiki-setting install 뒤 갱신해줘", True
    return f"세팅 v{latest} 최신", False


def main():
    root = wikilib.root_from_argv()
    hook, force = "--hook" in sys.argv, "--force" in sys.argv
    stamp = root / ".yw-wiki" / "version-check"
    online = force or not stamp.exists() or time.time() - stamp.stat().st_mtime > EVERY
    lines, bad = [], False
    for fn in (check_git, check_setting):
        try:
            text, b = fn(root, online)
        except Exception as e:  # 확인 실패는 작업을 막지 않고 알린다
            text, b = f"확인 실패: {e}", True
        lines.append(text)
        bad |= b
    p = subprocess.run([sys.executable, str(root / "tools" / "plugins.py"), str(root)], capture_output=True, text=True)
    lines.append((p.stdout.strip().splitlines() or ["Obsidian 플러그인: ?"])[0])
    bad |= p.returncode != 0
    if online:
        stamp.parent.mkdir(parents=True, exist_ok=True)
        stamp.touch()
    summary = ("⚠ " if bad else "") + " · ".join(lines)
    print(f"[{wikilib.config(root).get('wiki', {}).get('name', root.name)}] {summary}")
    return 0 if hook or not bad else 1


if __name__ == "__main__":
    sys.exit(main())

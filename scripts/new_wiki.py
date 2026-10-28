#!/usr/bin/env python3.14
"""LLM 위키를 새로 만든다.

  python3.14 scripts/new_wiki.py --profile personal --dest ../yw-wiki [--in-place] [--no-git]

--in-place: 이미 있는 폴더(기존 저장소)에 뼈대를 깐다. 이미 있는 raw/ · wiki/ 파일과 wiki.toml은 그대로 둔다.
만드는 것: 폴더, 템플릿 · 도구 · 스킬 · 훅, AGENTS.md(core + profile + local), CLAUDE.md, wiki.toml,
.wiki-setting.toml, wiki/index.md · log.md, review/, .obsidian 설정, .gitignore, git init과 훅 연결, 첫 점검.
"""
import argparse
import shutil
import subprocess
import sys
import tomllib
from pathlib import Path

import setting

def create(profile, dest, in_place=False, git=True):
    if dest.exists() and any(dest.iterdir()) and not in_place:
        sys.exit(f"{dest}가 비어 있지 않다. 기존 폴더에 깔려면 --in-place")
    dest.mkdir(parents=True, exist_ok=True)
    prof_dir = setting.SETTING / "profiles" / profile

    if not (dest / "wiki.toml").exists():
        shutil.copy(prof_dir / "wiki.toml", dest / "wiki.toml")
    conf = tomllib.loads((dest / "wiki.toml").read_text())
    name = conf.get("wiki", {}).get("name", dest.name)

    folders = conf.get("folders", {})
    dirs = ["wiki/concepts", "wiki/sources", "wiki/synthesis", "wiki/decisions", "review"]
    dirs += [f"wiki/entities/{k}" for k in conf.get("entities", {}).get("kinds", [])]
    dirs += [f"wiki/{d}" for d in folders.get("wiki", [])] + [f"raw/{d}" for d in folders.get("raw", [])]
    for d in dirs:
        (dest / d).mkdir(parents=True, exist_ok=True)
        if not any((dest / d).iterdir()):
            (dest / d / ".gitkeep").write_text("")

    managed = setting.managed_files(profile, name, conf)
    for rel, text in managed.items():
        setting.write(dest, rel, text)
    old = (dest / "AGENTS.md").read_text() if (dest / "AGENTS.md").exists() else ""
    setting.write(dest, "AGENTS.md", setting.agents(profile, setting.local_section(old)))
    setting.write(dest, "CLAUDE.md", "@AGENTS.md\n")
    setting.write(dest, ".wiki-setting.toml", setting.state_toml(profile, managed))

    once = {".gitignore": (setting.CORE / "gitignore").read_text()}
    for src in sorted((setting.CORE / "obsidian" / "common").glob("*.json")) + \
            sorted((setting.CORE / "obsidian" / profile).glob("*.json")):
        once[f".obsidian/{src.name}"] = src.read_text()
    for rel, text in once.items():
        if not (dest / rel).exists() or rel.startswith(".obsidian/") or rel == ".gitignore":
            setting.write(dest, rel, text)

    tools = dest / "tools"
    if not (dest / "wiki" / "log.md").exists():
        subprocess.run([sys.executable, str(tools / "log.py"), str(dest), "setup",
                        f"yw-wiki-setting v{setting.version()} · profile {profile}"], check=True)
    subprocess.run([sys.executable, str(tools / "index.py"), str(dest)], check=True, stdout=subprocess.DEVNULL)
    subprocess.run([sys.executable, str(tools / "plugins.py"), str(dest)])  # Obsidian 플러그인 · 기본값 (오프라인이면 알리고 계속)

    if git:
        if not (dest / ".git").exists():
            subprocess.run(["git", "init", "-q", "-b", "main", str(dest)], check=True)
        subprocess.run(["git", "-C", str(dest), "config", "core.hooksPath", ".githooks"], check=True)
    return subprocess.run([sys.executable, str(tools / "check.py"), str(dest)]).returncode


def main():
    p = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    p.add_argument("--profile", required=True, choices=setting.PROFILES)
    p.add_argument("--dest", required=True)
    p.add_argument("--in-place", action="store_true")
    p.add_argument("--no-git", action="store_true")
    a = p.parse_args()
    return create(a.profile, Path(a.dest).expanduser().resolve(), a.in_place, not a.no_git)


if __name__ == "__main__":
    sys.exit(main())

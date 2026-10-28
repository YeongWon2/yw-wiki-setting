#!/usr/bin/env python3.14
"""Obsidian 플러그인과 기본 설정을 맞춘다. 목록은 tools/obsidian/plugins.toml (세팅이 관리).

  python3.14 tools/plugins.py [위키 경로] [--check]

- 플러그인 파일(main.js · manifest.json · styles.css)이 없거나 고정한 sha256과 다르면 GitHub 릴리스에서 받아 확인한 뒤 둔다.
  받은 파일은 ~/.cache/yw-wiki/plugins/<id>/<version>/에 두고 다시 쓴다. sha256이 다르면 설치하지 않는다.
- .obsidian/community-plugins.json에 목록의 플러그인을 켠다 (사람이 더 켠 것은 그대로).
- 기본 설정(플러그인 data.json, graph.json)은 파일이 없을 때만 깐다. 열 때마다 바뀌는 파일이라 git에 올리지 않는다.
--check: 아무것도 바꾸지 않고 맞지 않는 것만 알린다 (종료 코드 1).
"""
import hashlib
import json
import os
import shutil
import sys
import tomllib
import urllib.request
from pathlib import Path

import wikilib

CACHE = Path.home() / ".cache" / "yw-wiki" / "plugins"
DEFAULTS = {".obsidian/graph.json": "graph.json"}  # 없을 때만 까는 Obsidian 기본값


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def fetch(repo, version, name, want):
    if os.environ.get("YW_NO_NETWORK"):
        raise RuntimeError("YW_NO_NETWORK — 받지 않음")
    cached = CACHE / repo.replace("/", "_") / version / name
    if not cached.exists() or sha(cached) != want:
        cached.parent.mkdir(parents=True, exist_ok=True)
        url = f"https://github.com/{repo}/releases/download/{version}/{name}"
        with urllib.request.urlopen(url, timeout=30) as r:
            data = r.read()
        if hashlib.sha256(data).hexdigest() != want:
            raise RuntimeError(f"{repo} {version} {name}: sha256이 고정값과 다르다 — 설치하지 않음")
        cached.write_bytes(data)
    return cached


def main():
    root = wikilib.root_from_argv()
    check = "--check" in sys.argv
    here = root / "tools" / "obsidian"
    catalog = tomllib.loads((here / "plugins.toml").read_text()).get("plugins", {})
    obs = root / ".obsidian"
    problems = []

    enabled_path = obs / "community-plugins.json"
    enabled = json.loads(enabled_path.read_text()) if enabled_path.exists() else []
    missing_ids = [pid for pid in catalog if pid not in enabled]
    if missing_ids:
        problems.append(f"켜지지 않은 플러그인: {', '.join(missing_ids)}")
        if not check:
            enabled_path.parent.mkdir(parents=True, exist_ok=True)
            enabled_path.write_text(json.dumps(enabled + missing_ids, ensure_ascii=False, indent=2) + "\n")

    for pid, p in catalog.items():
        dest = obs / "plugins" / pid
        for name, want in p.get("sha256", {}).items():
            f = dest / name
            if f.exists() and sha(f) == want:
                continue
            problems.append(f"{pid} {p['version']}: {name} {'없음' if not f.exists() else '버전이 다름'}")
            if not check:
                try:
                    src = fetch(p["repo"], p["version"], name, want)
                except Exception as e:  # 네트워크 없음 · sha256 다름: 깐 것 그대로 두고 알린다
                    problems.append(f"  설치 못 함: {e}")
                    continue
                dest.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(src, f)
        if p.get("defaults") and not (dest / "data.json").exists():
            problems.append(f"{pid}: 기본 설정 없음")
            if not check:
                dest.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(here / "defaults" / p["defaults"], dest / "data.json")

    for rel, src in DEFAULTS.items():
        if not (root / rel).exists():
            problems.append(f"{rel} 없음")
            if not check:
                (root / rel).parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(here / "defaults" / src, root / rel)

    if check:
        for line in problems:
            print(f"플러그인  {line}")
        return 1 if problems else 0
    unresolved = [l for l in problems if l.startswith("  설치 못 함")]
    print("Obsidian 플러그인: " + ("맞춤" if not unresolved else f"설치 못 한 것 {len(unresolved)}개") +
          (f" (고친 것 {len(problems) - len(unresolved)}개)" if problems else ""))
    for line in unresolved:
        print(line)
    return 1 if unresolved else 0


if __name__ == "__main__":
    sys.exit(main())

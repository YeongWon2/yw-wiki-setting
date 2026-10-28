#!/usr/bin/env python3
"""wiki/log.md에 작업 기록 한 줄을 더한다. 추가만 하고 지난 기록은 고치지 않는다.

  python3 tools/log.py <작업> <제목> [세부 …]
  예: python3 tools/log.py ingest "2026-10-29 회의" "새 source 1" "갱신 entity 2"
작업: ingest · query · lint · upgrade · migrate · career · reconcile
"""
import os
import sys
from datetime import date

import wikilib

HEADER = "---\ntitle: log\ntype: log\n---\n\n# 작업 기록\n\n추가만 한다. 형식: `## [YYYY-MM-DD] <작업> | <제목>`\n"


def append(root, action, title, details=()):
    path = root / "wiki" / "log.md"
    text = path.read_text() if path.exists() else HEADER
    today = os.environ.get("YW_TODAY") or date.today().isoformat()
    entry = f"\n## [{today}] {action} | {title}\n" + "".join(f"- {d}\n" for d in details)
    path.write_text(text.rstrip("\n") + "\n" + entry)


def main():
    args = sys.argv[1:]
    if args and os.path.isdir(os.path.expanduser(args[0])):
        root, args = wikilib.Path(args[0]).expanduser().resolve(), args[1:]
    else:
        root = wikilib.ROOT
    if len(args) < 2:
        print(__doc__)
        return 2
    append(root, args[0], args[1], args[2:])
    return 0


if __name__ == "__main__":
    sys.exit(main())

#!/usr/bin/env python3
"""wiki/index.md를 frontmatter로 다시 만든다. 손으로 고치지 않는다.

폴더(type)별로 묶고, 각 묶음 안에서 active를 먼저, archived · superseded를 뒤에 싣는다.
  python3 tools/index.py [위키 경로]          다시 만든다
  python3 tools/index.py [위키 경로] --check  최신이 아니면 종료 코드 1
"""
import sys

import wikilib

SECTIONS = [("entities", "Entities"), ("concepts", "Concepts"), ("sources", "Sources"),
            ("synthesis", "Synthesis"), ("decisions", "Decisions"), ("career", "Career"), ("drafts", "Drafts")]


def entry(path, rel):
    meta, body = wikilib.frontmatter(path.read_text())
    meta = meta or {}
    text = meta.get("description") or wikilib.summary(body)
    status = meta.get("review") if rel.parts[1] == "drafts" else meta.get("status", "")
    tag = "" if status in ("active", "", None) else f" `{status}`"
    return status in ("active", "draft", "approved", "", None), f"- [[{path.stem}]]{tag}" + (f" — {text}" if text else "")


def render(root):
    conf = wikilib.config(root)
    w = conf.get("wiki", {})
    groups = {}
    for path, rel in wikilib.pages(root):
        top = rel.parts[1] if len(rel.parts) > 2 else "(기타)"
        sub = rel.parts[2] if top == "entities" and len(rel.parts) > 3 else ""
        groups.setdefault((top, sub), []).append(entry(path, rel))
    total = sum(len(v) for v in groups.values())

    out = ["---", "title: index", "type: index", "---", "", f"# {w.get('name', root.name)} 목차", "",
           f"{w.get('description', '')}. 페이지 {total}개. `tools/index.py`가 만든다 — 손으로 고치지 않는다.", "",
           "작업 기록은 [[log]], 사람이 판단할 충돌은 [[conflicts]].", ""]
    kinds = conf.get("entities", {}).get("kinds", [])
    order = [(top, title) for top, title in SECTIONS] + sorted(
        {(t, t) for t, _ in groups if t not in dict(SECTIONS)})
    for top, title in order:
        subs = sorted({s for t, s in groups if t == top}, key=lambda s: (kinds.index(s) if s in kinds else 99, s))
        for sub in subs:
            items = groups[(top, sub)]
            out += [f"## {title}" + (f" — {sub}" if sub else ""), ""]
            out += [line for active, line in sorted(items, key=lambda x: (not x[0], x[1])) if active]
            done = [line for active, line in sorted(items, key=lambda x: x[1]) if not active]
            if done:
                out += ["", "보관 · 대체됨:", ""] + done
            out.append("")
    return "\n".join(out).rstrip() + "\n"


def main():
    root = wikilib.root_from_argv()
    target = root / "wiki" / "index.md"
    text = render(root)
    if "--check" in sys.argv:
        if not target.exists() or target.read_text() != text:
            print("wiki/index.md가 최신이 아니다 — python3 tools/index.py")
            return 1
        return 0
    target.write_text(text)
    print(f"wiki/index.md: {text.count(chr(10) + '- [[')}개 페이지")
    return 0


if __name__ == "__main__":
    sys.exit(main())

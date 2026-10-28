#!/usr/bin/env python3
"""1회용: PARA 볼트(yw-wiki 2026-10 구조)를 개인 위키 · 업무 위키로 나눌 계획을 만든다.

  python3 scripts/migrate_para.py <PARA 볼트> --personal <경로> --work <경로> --dry-run [--report <파일>] [--map <toml>]

--dry-run: 파일을 건드리지 않고 보고서(markdown)만 쓴다.
  문서마다 갈 위키 · 새 경로 · 새 type, 매핑되지 않은 파일, 깨질 링크, 개인 위키에 남는 회사 말, 사람이 정할 것.
규칙은 아래 RULES(폴더 · type 기준)이고, --map TOML의 [override."<기존 경로>"]가 있으면 그것이 먼저다.
  [override."2 Area(영역)/학교/학교.md"]
  action = "merge"            # move(기본) · merge(다른 페이지에 합치고 파일은 없앰) · drop
  into = "인하대학교"          # merge 대상
  dest = "wiki/entities/orgs" # move 대상 폴더
  type = "entity"
  entity_kind = "org"
  note = "왜 이렇게 정했나"
회사 말(개인 위키에 남으면 안 되는 말)은 업무 위키 wiki.toml의 [company] 값과 --guard로 정한다.
"""
import argparse
import re
import sys
import tomllib
from datetime import date
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "core" / "tools"))
import wikilib  # noqa: E402

COMPANY_DIR = "2 Area(영역)/회사/"
SKIP = {"AGENTS.md", "CLAUDE.md", "README.md"}
SKIP_TOP = {".git", ".obsidian", ".claude", ".githooks", "tools", "_templates(템플릿)"}
ACTIVE = {"진행", "예정", "보류", "계획", "유지", "누적", "기록", "필기", "정리됨", "준비"}
ARCHIVED = {"완료", "취소", "보관"}
DECISION = {"확정": "accepted", "제안": "proposed", "대체됨": "superseded"}


def rule(rel, meta, work):
    """(action, dest 폴더, 새 type, entity_kind, 메모). dest는 위키 루트 기준."""
    t = meta.get("type", "")
    s = str(rel)
    if t == "프로젝트":
        course = "/Project(수업)/" in s and rel.parent.name == rel.stem
        if "/Project(수업)/" in s and not course:
            return "move", "wiki/concepts", "concept", "", "수업 폴더 안의 프로젝트 문서 → 수업 산출물 concept (정할 것)"
        return "move", "wiki/entities/projects", "entity", "course" if course else "project", ""
    if t == "결정 기록":
        return "move", "wiki/decisions", "decision", "", "DEC-ID 부여" if work else ""
    if t == "강의 노트":
        return "move", "wiki/concepts", "concept", "", ""
    if t == "회의록":
        return "split", "raw/meetings + wiki/sources", "source", "", "원본은 raw/meetings, 요약은 sources"
    if t == "일지":
        return "move", "raw/worklog" if work else "raw/daily", "raw", "", ""
    if t == "시험" or t == "과제":
        return "move", "wiki/synthesis", "synthesis", "", "수업 entity에 연결"
    if t == "영역":
        return "move", "wiki/entities/orgs", "entity", "org", ""
    if t == "성과 기록":
        return "move", "wiki/entities/orgs", "entity", "org", "경력 회사. 한 일 표를 career/ achievement로 나눌지 정할 것"
    if t == "자료":
        return "move", "wiki/concepts", "concept", "", "자료 → concept 기본 (synthesis일 수 있음)"
    if t == "목록":
        return "drop", "wiki/index.md", "index", "", "index.py가 새로 만든다"
    return "unmapped", "", "", "", f"type '{t}' 규칙 없음"


def collect(vault):
    for path in sorted(vault.rglob("*.md")):
        rel = path.relative_to(vault)
        if rel.parts[0] in SKIP_TOP or (len(rel.parts) == 1 and rel.name in SKIP):
            continue
        yield path, rel


def plan(vault, overrides):
    rows = []
    for path, rel in collect(vault):
        meta, _ = wikilib.frontmatter(path.read_text())
        meta = meta or {}
        work = str(rel).startswith(COMPANY_DIR)
        action, dest, new_type, kind, note = rule(rel, meta, work)
        o = overrides.get(str(rel), {})
        action = o.get("action", action)
        dest = o.get("dest", o.get("into", dest) if action == "merge" else dest)
        new_type, kind = o.get("type", new_type), o.get("entity_kind", kind)
        note = o.get("note", note)
        status = meta.get("status", "")
        new_status = DECISION.get(status, "") if new_type == "decision" else \
            ("archived" if status in ARCHIVED else "active" if new_type not in ("raw", "index") else "")
        rows.append({"rel": rel, "wiki": "work" if work else "personal", "action": action, "dest": dest,
                     "type": meta.get("type", ""), "status": status, "new_type": new_type, "kind": kind,
                     "new_status": new_status, "note": note, "text": path.read_text()})
    return rows


def report(rows, guard, today):
    by_name = {r["rel"].stem: r for r in rows}
    gone = {r["rel"].stem for r in rows if r["action"] in ("merge", "drop")}
    out = [f"# PARA → LLM 위키 dry-run 보고서 ({today})", "",
           "파일은 하나도 바꾸지 않았다. 이 보고서를 보고 정할 것을 정한 뒤 실제 이동을 한다.", ""]
    counts = {}
    for r in rows:
        counts[(r["wiki"], r["action"])] = counts.get((r["wiki"], r["action"]), 0) + 1
    out += ["## 요약", "", f"- 내용 문서 {len(rows)}개"]
    for (w, a), n in sorted(counts.items()):
        out.append(f"- {w} · {a}: {n}")
    out.append("")

    decide = [r for r in rows if r["note"] and ("정할 것" in r["note"] or "있음" in r["note"] or r["action"] == "unmapped")]
    out += ["## 정할 것", ""]
    out += [f"- `{r['rel']}` — {r['note']}" for r in decide] or ["- 없음"]
    out.append("")

    out += ["## 이동 계획", "", "| 위키 | 기존 경로 | 기존 type · status | 처리 | 새 위치 | 새 type · status | 메모 |", "|---|---|---|---|---|---|---|"]
    for r in sorted(rows, key=lambda r: (r["wiki"], r["dest"], str(r["rel"]))):
        nt = r["new_type"] + (f" ({r['kind']})" if r["kind"] else "")
        out.append(f"| {r['wiki']} | {r['rel']} | {r['type']} · {r['status']} | {r['action']} | {r['dest']} | "
                   f"{nt} · {r['new_status']} | {r['note']} |")
    out.append("")

    # 링크: 위키가 갈리거나 문서가 없어지면 깨진다
    broken = []
    for r in rows:
        for name in sorted(set(wikilib.links(r["text"]))):
            t = by_name.get(name)
            if name in gone:
                broken.append((r, name, "합치거나 없애는 문서"))
            elif t and t["wiki"] != r["wiki"] and not (r["wiki"] == "personal" and name in guard.get("allow", [])):
                broken.append((r, name, f"{t['wiki']} 위키로 감"))
            elif not t and name not in {"템플릿 목록"}:
                pass
    out += ["## 깨질 링크", "", f"{len(broken)}개. 개인 → 업무 링크는 지우고 내 말로 바꾼다. 업무 → 개인 링크는 지운다.", "",
            "| 문서(위키) | 링크 | 이유 |", "|---|---|---|"]
    out += [f"| {r['rel'].stem} ({r['wiki']}) | [[{n}]] | {why} |" for r, n, why in broken]
    out.append("")

    terms = [t for t in guard.get("terms", []) if t]
    pat = re.compile("|".join(re.escape(t) for t in terms), re.I) if terms else None
    out += ["## 개인 위키에 남는 회사 말", "", f"검색어: {', '.join(terms)}. 허용 페이지: {', '.join(guard.get('allow', [])) or '없음'}.", ""]
    hits = 0
    for r in rows:
        if r["wiki"] != "personal" or not pat:
            continue
        for i, line in enumerate(r["text"].splitlines(), 1):
            if pat.search(line):
                hits += 1
                ok = " (허용 페이지 — 경력 사실만 남기고 업무 내용은 뺀다)" if r["rel"].stem in guard.get("allow", []) else ""
                out.append(f"- `{r['rel']}:{i}`{ok} — {line.strip()[:160]}")
    if not hits:
        out.append("- 없음")
    out.append("")
    return "\n".join(out) + "\n"


def main():
    p = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    p.add_argument("vault")
    p.add_argument("--personal", required=True)
    p.add_argument("--work", required=True)
    p.add_argument("--dry-run", action="store_true")
    p.add_argument("--report")
    p.add_argument("--map")
    a = p.parse_args()
    if not a.dry_run:
        sys.exit("아직 --dry-run만 된다. 보고서 검토 뒤 실제 이동을 만든다.")
    m = tomllib.loads(Path(a.map).read_text()) if a.map else {}
    rows = plan(Path(a.vault).expanduser().resolve(), m.get("override", {}))
    text = report(rows, m.get("guard", {}), date.today().isoformat())
    if a.report:
        Path(a.report).expanduser().write_text(text)
        print(f"보고서: {a.report} · 문서 {len(rows)}개")
    else:
        print(text)
    return 0


if __name__ == "__main__":
    sys.exit(main())

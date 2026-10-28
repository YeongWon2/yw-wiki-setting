#!/usr/bin/env python3.14
"""1회용: PARA 볼트 → 개인 · 업무 위키 마이그레이션이 끝났는지 검증한다 (계획서 "검증 방법" 표).

  python3.14 scripts/verify_migration.py <옛 볼트(기준 커밋 체크아웃)> --personal <경로> --work <경로> [--map <toml>]

보는 것
- 문서 누락: 옛 내용 문서마다 새 위키 어딘가에 대응 문서가 있다 (이동 · 합침 · 없앰은 매핑대로)
- 내용 보존: 옛 문서의 URL · 날짜(YYYY-MM-DD, YYYY.MM→YYYY-MM) · 6자 이상 따옴표 인용 · % 수치가 대응 문서(들)에 있다
- 회사 분리: 개인 위키에 [guard] 말이 allow 밖에 0건, 업무 위키에 개인 문서 이름 0건
- PARA 흔적: 0~4 번호 폴더 · _templates(템플릿) · [!hero] · [!cols] · yw-wide 0건
- 두 위키 check.py 오류 0
종료 코드: 모두 통과면 0.
"""
import argparse
import re
import subprocess
import sys
import tomllib
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "core" / "tools"))
import migrate_para  # noqa: E402

URL = re.compile(r"https?://[^\s)\]|\"'>]+")
QUOTE = re.compile(r'"([^"\n,][^"\n]{5,})"')
PCT = re.compile(r"\d+(?:\.\d+)?%")
DATE = re.compile(r"\d{4}-\d{2}-\d{2}")
YM = re.compile(r"\b(\d{4})\.(\d{2})\b")
PARA_DIRS = ("0 Inbox(받은함)", "1 Project(프로젝트)", "2 Area(영역)", "3 Resource(자료)", "4 Archive(보관)", "_templates(템플릿)")
LEGACY = re.compile(r"\[!hero\]|\[!cols\]|yw-wide")


def features(text):
    body = re.sub(r"```mermaid.*?```", "", text, flags=re.S)  # 빈 mermaid 예시 날짜는 빼고 본다
    feats = set(URL.findall(text)) | set(QUOTE.findall(text)) | set(PCT.findall(text)) | set(DATE.findall(body))
    feats |= {f"{y}-{m}" for y, m in YM.findall(text)}
    frontmatter_end = text.find("\n---", 4) if text.startswith("---\n") else -1
    head = text[:frontmatter_end] if frontmatter_end > 0 else ""
    return {f for f in feats if f not in head or f in text[frontmatter_end:]}  # frontmatter만의 값(created 등)은 빼지 않는다


def find(root, name):
    hits = [p for p in root.rglob(f"{name}.md") if ".git" not in p.parts]
    return hits


def main():
    p = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    p.add_argument("vault")
    p.add_argument("--personal", required=True)
    p.add_argument("--work", required=True)
    p.add_argument("--map")
    a = p.parse_args()
    vault, personal, work = (Path(x).expanduser().resolve() for x in (a.vault, a.personal, a.work))
    m = tomllib.loads(Path(a.map).read_text()) if a.map else {}
    rows = migrate_para.plan(vault, m.get("override", {}))
    fails, notes = [], []

    for r in rows:
        name = r["rel"].stem
        root = work if r["wiki"] == "work" else personal
        if r["action"] == "drop":
            continue
        target = r["dest"] if r["action"] == "merge" else name
        cands = find(root, target) or [q for q in (root / "wiki" / "decisions").glob(f"DEC-* {name}.md")]
        if r["new_type"] == "raw":
            date = re.search(r"\d{4}-\d{2}-\d{2}", name)
            cands = cands or ([root / "raw" / ("worklog" if r["wiki"] == "work" else "daily") / f"{date.group(0)}.md"] if date else [])
            cands = [c for c in cands if c.exists()]
        if r["action"] == "split":
            cands += list((root / "raw" / "meetings").glob("*.md"))
        if not cands:
            fails.append(f"누락: {r['rel']} → {r['wiki']} 위키에 '{target}' 없음")
            continue
        new_text = "".join(c.read_text() for c in cands)
        if name.startswith(("다닥", "카랑", "닥커넥트", "세이프닥")) or r["type"] == "성과 기록":
            new_text += "".join(c.read_text() for c in (root / "wiki" / "career").glob(f"{name} - *.md"))
        alt = new_text + re.sub(r"\b(\d{4})\.(\d{2})\b", r"\1-\2", new_text)  # 2018.11 표기도 2018-11로 인정
        miss = sorted(f for f in features(r["text"]) if f not in alt)
        if miss:
            fails.append(f"내용: {r['rel']} → 빠짐 {len(miss)}개: {', '.join(miss[:8])}" + (" …" if len(miss) > 8 else ""))

    guard = tomllib.loads((personal / "wiki.toml").read_text()).get("guard", {})
    terms, allow = [t for t in guard.get("terms", []) if t], set(guard.get("allow", []))
    pat = re.compile("|".join(map(re.escape, terms)), re.I) if terms else None
    work_names = {q.stem for q in work.rglob("*.md")}
    personal_names = {r["rel"].stem for r in rows if r["wiki"] == "personal"} - {"QMIT"} - work_names
    for root, label in ((personal, "personal"), (work, "work")):
        for path in root.rglob("*"):
            rel = path.relative_to(root)
            if ".git" in rel.parts or not path.is_file():
                continue
            if rel.parts and rel.parts[0] in PARA_DIRS:
                fails.append(f"PARA 흔적: {label}/{rel}")
                continue
            if path.suffix != ".md" or rel.parts[0] in (".claude", "_templates", ".yw-wiki"):
                continue
            text = path.read_text()
            if LEGACY.search(text):
                fails.append(f"PARA 서식: {label}/{rel}")
            if label == "personal" and pat and path.stem not in allow and (hit := pat.search(text)):
                fails.append(f"회사 분리: personal/{rel}에 '{hit.group(0)}'")
            if label == "work":
                for n in personal_names:
                    if f"[[{n}]]" in text:
                        fails.append(f"개인 내용: work/{rel}에 [[{n}]]")
    for root, label in ((personal, "personal"), (work, "work")):
        out = subprocess.run([sys.executable, str(root / "tools" / "check.py"), str(root)], capture_output=True, text=True)
        last = out.stdout.strip().splitlines()[-1] if out.stdout.strip() else out.stderr.strip()
        (fails if out.returncode else notes).append(f"check {label}: {last}")

    for line in notes:
        print(f"통과  {line}")
    for line in fails:
        print(f"실패  {line}")
    print(f"옛 문서 {len(rows)}개 · 실패 {len(fails)}")
    return 1 if fails else 0


if __name__ == "__main__":
    sys.exit(main())

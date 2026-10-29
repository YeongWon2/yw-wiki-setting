#!/usr/bin/env python3.14
"""LLM 위키 점검: AGENTS.md 중 기계로 볼 수 있는 것만 본다.

오류(종료 코드 1)
- wiki/ 페이지에 frontmatter가 없거나 공통 필드(REQUIRED)가 빠졌다
- type · status · confidence · entity_kind · decision_status가 정해진 값이 아니다
- type과 폴더가 맞지 않는다 (entity → wiki/entities/ …)
- 템플릿(_templates/<type>.md)의 ## 제목이 빠졌거나 순서가 다르다
- 날짜(created · updated · date · checked)가 YYYY-MM-DD가 아니거나, updated가 created보다 이르거나, 오늘보다 뒤다
- 템플릿 자리표시({{date}} · {{title}})가 남아 있다
- 없는 문서로 가는 [[링크]], 없는 파일로 가는 마크다운 링크 [..](경로), 로컬 절대 경로 링크(/Users/… · ~/… — 다른 PC에서 깨짐)
- 코드 블록(```)이 닫히지 않았다
- 노션식 서식이 남아 있다 ([!hero] · [!cols] · yw-wide · ## 🔗 참조)
- 충돌 내용이 위키에 있다 (## Contradictions 칸, wiki/conflicts.md) — 충돌은 review/에 둔다
- review/ 항목의 frontmatter(type: review · review_kind · status)가 틀렸다
- raw/ 파일을 지우거나 옮겼다 (커밋에 올린 변경 기준)
- index.md가 최신이 아니다 (python3.14 tools/index.py)
- [guard] terms(이 위키에 있으면 안 되는 말)가 [guard] allow 밖의 파일에 있다
경고
- raw/ 파일을 고쳤다 (사람이 고친 거면 괜찮다)
- 같은 이름의 페이지가 둘 이상 · 들어오는 링크가 없는 고아 페이지
- Key facts가 12개를 넘는다 (페이지를 나눈다)
- 진행 중인 project · course를 STALE_DAYS일 넘게 고치지 않음 · source 원본 확인(checked)이 STALE_DAYS일 넘음
- proposed 결정이 PROPOSAL_DAYS일 넘음
- review/에 pending 항목이 있다 (yw-wiki-review로 정한다) · REVIEW_DAYS일 넘은 pending
오늘 날짜는 YW_TODAY(YYYY-MM-DD)로 바꿔 볼 수 있다.
  python3.14 tools/check.py [위키 경로]
"""
import os
import re
import subprocess
import sys
from datetime import date, timedelta
import urllib.parse

import index as index_tool
import wikilib

REQUIRED = ("title", "type", "status", "confidence", "sources", "related", "created", "updated")
STATUSES = {"active", "superseded", "archived"}
CONFIDENCE = {"high", "medium", "low"}
ENTITY_KINDS = {"person", "org", "project", "course"}
DECISION_STATUSES = {"proposed", "accepted", "superseded"}
PROFILE_TYPES = {"personal": {"achievement"}}
DATE_KEYS = ("created", "updated", "date", "checked")
PLACEHOLDER = re.compile(r"\{\{\s*(date|title|time)[^}]*\}\}")
LEGACY = re.compile(r"\[!hero\]|\[!cols\]|yw-wide|^## 🔗 참조", re.M)
CONFLICT = re.compile(r"^## Contradictions\s*$", re.M)
MD_LINK = re.compile(r"(?<!!)\[[^\]]*\]\(<?([^)\s>]+)>?(?:\s+\"[^\"]*\")?\)")
PERIOD_PART = re.compile(r"^\d{4}(-\d{2}(-\d{2})?)?$")
MAX_FACTS = 12
STALE_DAYS = 30
PROPOSAL_DAYS = 14
REVIEW_DAYS = 14
REVIEW_KINDS = {"conflict", "stale", "orphan", "duplicate", "raw-change"}


def today():
    return date.fromisoformat(os.environ.get("YW_TODAY") or date.today().isoformat())


def parse_date(value):
    try:
        return date.fromisoformat(str(value))
    except ValueError:
        return None


def headings(body):
    body = re.sub(r"```.*?```", "", body, flags=re.S)
    return [m.strip() for m in re.findall(r"^##\s+(.+)$", body, flags=re.M)]


def in_order(needed, have):
    it = iter(have)
    return all(h in it for h in needed)


def key_facts(body):
    m = re.search(r"^## Key facts\s*$(.*?)(?=^## |\Z)", body, flags=re.M | re.S)
    if not m:
        return 0
    return sum(1 for line in m.group(1).splitlines() if re.match(r"^- \S", line))


def staged_raw_changes(root):
    """커밋에 올린 raw/ 변경: [(상태, 경로)]. git이 아니면 빈 목록."""
    try:
        out = subprocess.run(["git", "-C", str(root), "diff", "--cached", "--name-status", "-M", "--", "raw/"],
                             capture_output=True, text=True, check=True).stdout
    except (subprocess.CalledProcessError, FileNotFoundError):
        return []
    return [(line.split("\t")[0][0], line.split("\t")[-1]) for line in out.splitlines() if line]


def main():
    root = wikilib.root_from_argv()
    conf = wikilib.config(root)
    profile = conf.get("wiki", {}).get("profile", "personal")
    types = set(wikilib.TYPE_DIRS) & ({"entity", "concept", "source", "synthesis", "decision"} | PROFILE_TYPES.get(profile, set()))
    errors, warnings = [], []
    now = today()

    files = list(wikilib.all_files(root))
    names = {}
    for path, rel in files:
        names.setdefault(path.stem if path.suffix == ".md" else path.name, []).append(rel)
    targets = set(names) | {p.name for p, _ in files}

    templates = {}
    for t in (root / "_templates").glob("*.md"):
        templates[t.stem] = headings(wikilib.frontmatter(t.read_text())[1])

    incoming = {}

    # 링크는 wiki/ · raw/ · 루트 문서 모두에서 본다
    for path, rel in files:
        if path.suffix != ".md":
            continue
        text = path.read_text()
        for name in dict.fromkeys(wikilib.links(text)):
            if name not in targets:
                errors.append(f"{rel}: 없는 문서 링크 [[{name}]]")
            elif name != path.stem and rel.parts[0] != "review":
                incoming.setdefault(name, set()).add(rel)
        for target in MD_LINK.findall(wikilib.strip_code(text)):
            if target.startswith(("http://", "https://", "mailto:", "#", "obsidian:")):
                continue
            local = urllib.parse.unquote(target.split("#")[0])
            if local.startswith(("/", "~")):
                errors.append(f"{rel}: 로컬 절대 경로 링크 {target} — 다른 PC에서 깨진다")
            elif local and not (path.parent / local).exists():
                errors.append(f"{rel}: 없는 파일 링크 {target}")
        if text.count("```") % 2:
            errors.append(f"{rel}: 닫히지 않은 코드 블록")
        if CONFLICT.search(text) and rel.parts[0] == "wiki":
            errors.append(f"{rel}: 충돌 칸(## Contradictions)이 있음 — 충돌은 review/에 둔다")
        if LEGACY.search(text):
            errors.append(f"{rel}: 노션식 서식이 남아 있음 ([!hero] · [!cols] · yw-wide · 🔗 참조)")

    for path, rel in wikilib.pages(root):
        text = path.read_text()
        meta, body = wikilib.frontmatter(text)
        if meta is None:
            errors.append(f"{rel}: frontmatter 없음")
            continue
        if PLACEHOLDER.search(wikilib.strip_code(text)):
            errors.append(f"{rel}: 템플릿 자리표시가 남아 있음")
        top = rel.parts[1] if len(rel.parts) > 2 else ""

        for key in REQUIRED:
            if key not in meta or meta[key] == "" and key not in ("sources", "related"):
                errors.append(f"{rel}: frontmatter에 {key}가 없음")
        kind = meta.get("type", "")
        if kind and kind not in types:
            errors.append(f"{rel}: type '{kind}'은 이 위키({profile})에 없는 종류")
        elif kind and top != wikilib.TYPE_DIRS[kind]:
            errors.append(f"{rel}: type {kind}은 wiki/{wikilib.TYPE_DIRS[kind]}/에 둔다")
        if meta.get("status") and meta["status"] not in STATUSES:
            errors.append(f"{rel}: status는 {sorted(STATUSES)} 중 하나 (지금 {meta['status']})")
        if meta.get("confidence") and meta["confidence"] not in CONFIDENCE:
            errors.append(f"{rel}: confidence는 {sorted(CONFIDENCE)} 중 하나")
        if kind in templates and not in_order(templates[kind], headings(body)):
            missing = [h for h in templates[kind] if h not in headings(body)]
            errors.append(f"{rel}: 템플릿 제목 " + (f"빠짐: {', '.join(missing)}" if missing else "순서가 다름"))

        for key in DATE_KEYS:
            if meta.get(key) and parse_date(meta[key]) is None:
                errors.append(f"{rel}: {key} 날짜 형식이 YYYY-MM-DD가 아님 ({meta[key]})")
        created, updated = parse_date(meta.get("created", "")), parse_date(meta.get("updated", ""))
        if created and updated and updated < created:
            errors.append(f"{rel}: updated가 created보다 이름")
        for key, d in (("created", created), ("updated", updated)):
            if d and d > now:
                errors.append(f"{rel}: {key}가 오늘보다 뒤")
        if meta.get("period"):
            parts = str(meta["period"]).split("~")
            if len(parts) != 2 or not all(PERIOD_PART.match(p.strip()) or not p.strip() for p in parts):
                errors.append(f"{rel}: period는 YYYY-MM-DD~YYYY-MM-DD (끝은 비워도 됨)")

        if kind == "entity":
            ek = meta.get("entity_kind", "")
            if ek not in ENTITY_KINDS:
                errors.append(f"{rel}: entity_kind는 {sorted(ENTITY_KINDS)} 중 하나")
            if ek in ("project", "course") and meta.get("status") == "active" and updated \
                    and now - updated > timedelta(days=STALE_DAYS):
                warnings.append(f"{rel}: 진행 중인데 {STALE_DAYS}일 넘게 고치지 않음 (updated {updated})")
        if kind == "source":
            checked = parse_date(meta.get("checked", ""))
            if checked and now - checked > timedelta(days=STALE_DAYS):
                warnings.append(f"{rel}: 원본 확인이 {STALE_DAYS}일 넘음 (checked {checked})")
        if kind == "decision":
            ds = meta.get("decision_status", "")
            if ds not in DECISION_STATUSES:
                errors.append(f"{rel}: decision_status는 {sorted(DECISION_STATUSES)} 중 하나")
            if ds == "superseded" and not meta.get("superseded_by"):
                errors.append(f"{rel}: superseded 결정에 superseded_by가 없음")
            if wikilib.links(f"x {meta.get('superseded_by') or ''}") == [path.stem]:
                errors.append(f"{rel}: superseded_by가 자기 자신을 가리킴")
            d = parse_date(meta.get("date", ""))
            if ds == "proposed" and d and now - d > timedelta(days=PROPOSAL_DAYS):
                warnings.append(f"{rel}: proposed가 {PROPOSAL_DAYS}일 넘음 ({d})")
        if key_facts(body) > MAX_FACTS:
            warnings.append(f"{rel}: Key facts {key_facts(body)}개 — {MAX_FACTS}개를 넘으면 나눈다")
        if path.stem not in incoming:
            warnings.append(f"{rel}: 들어오는 링크가 없는 고아 페이지")

    for name, rels in names.items():
        md = [r for r in rels if r.suffix == ".md" and r.parts[0] in ("wiki", "raw")]
        if len(md) > 1:
            warnings.append(f"같은 이름 '{name}': {', '.join(map(str, md))}")

    for status, rel in staged_raw_changes(root):
        if status in "DR":
            errors.append(f"{rel}: raw/ 파일을 지우거나 옮김 — raw는 고치지 않는다")
        elif status == "M":
            warnings.append(f"{rel}: raw/ 파일을 고침 — 사람이 고친 것이 아니면 되돌린다")

    guard = conf.get("guard", {})
    terms = [t for t in guard.get("terms", []) if t]
    if terms:
        allow = set(guard.get("allow", []))
        pattern = re.compile("|".join(re.escape(t) for t in terms), re.I)
        for path, rel in files:
            if path.suffix != ".md" or path.stem in allow:
                continue
            hit = pattern.search(path.read_text())
            if hit:
                errors.append(f"{rel}: 이 위키에 두지 않는 말 '{hit.group(0)}' ([guard] allow 밖)")

    pending = []
    for path in sorted((root / "review").glob("*.md")):
        rel = path.relative_to(root)
        meta, _ = wikilib.frontmatter(path.read_text())
        meta = meta or {}
        if meta.get("type") != "review":
            errors.append(f"{rel}: review/에는 review 템플릿 문서만 둔다")
            continue
        if meta.get("review_kind") not in REVIEW_KINDS:
            errors.append(f"{rel}: review_kind는 {sorted(REVIEW_KINDS)} 중 하나")
        if meta.get("status") not in ("pending", "resolved"):
            errors.append(f"{rel}: status는 pending · resolved 중 하나")
        if PLACEHOLDER.search(wikilib.strip_code(path.read_text())):
            errors.append(f"{rel}: 템플릿 자리표시가 남아 있음")
        if meta.get("status") == "pending":
            pending.append(rel)
            c = parse_date(meta.get("created", ""))
            if c and now - c > timedelta(days=REVIEW_DAYS):
                warnings.append(f"{rel}: 검토 대기 {REVIEW_DAYS}일 넘음 ({c})")
    if pending:
        warnings.append(f"review/ 검토 대기 {len(pending)}개 — yw-wiki-review로 정한다")

    if (root / "wiki" / "conflicts.md").exists():
        errors.append("wiki/conflicts.md가 있음 — 충돌은 review/에 항목 하나씩 둔다")

    index_text = index_tool.render(root)
    target = root / "wiki" / "index.md"
    if not target.exists() or target.read_text() != index_text:
        errors.append("wiki/index.md가 최신이 아님 — python3.14 tools/index.py")

    for line in errors:
        print(f"오류  {line}")
    for line in warnings:
        print(f"경고  {line}")
    print(f"페이지 {len(wikilib.pages(root))}개 · 오류 {len(errors)} · 경고 {len(warnings)}")
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main())

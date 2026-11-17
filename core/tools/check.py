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
- [guard] terms(이 위키에 있으면 안 되는 말) · [guard] patterns(정규식)가 [guard] allow 밖의 파일에 있다
- guide: guide_kind · scope · maturity · owner_checked 값이 틀렸다, ## Rules가 번호 목록이 아니다,
  ## Evidence 줄이 "- YYYY-MM-DD · 어디서 · 결과 — 근거" 형식이 아니다, tested인데 Evidence의 서로 다른 곳이 2개 미만이다,
  proven인데 owner_checked가 true가 아니다, 내 방식(area:) · 회사 방식(org:)인데 Evidence가 없다,
  sources도 Evidence도 없다, superseded인데 superseded_by가 없다, owner_checked: true인데 owner_checked_on 날짜나 그날 log의 confirm 기록이 없다
경고
- raw/ 파일을 고쳤다 (사람이 고친 거면 괜찮다)
- 같은 이름의 페이지가 둘 이상 · 들어오는 링크가 없는 고아 페이지
- Key facts가 12개를 넘는다 (페이지를 나눈다)
- 진행 중인 project · course를 STALE_DAYS일 넘게 고치지 않음 · source 원본 확인(checked)이 STALE_DAYS일 넘음
- proposed 결정이 PROPOSAL_DAYS일 넘음
- review/에 pending 항목이 있다 (yw-wiki-review로 정한다) · REVIEW_DAYS일 넘은 pending
- 이름이 거의 같은 페이지(공백 · 기호 · 대소문자만 다름) · 별칭이 다른 페이지 이름과 같음 (중복 후보)
- guide가 seed인 채 SEED_DAYS일 넘게 고치지 않음 · 내가 확인하지 않은 guide 수 (owner_checked: false)
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
PROFILE_TYPES = {"personal": {"achievement", "guide"}}
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
GUIDE_KINDS = {"principle", "convention", "playbook"}
MATURITY = {"seed", "tested", "proven"}
SCOPE = re.compile(r"^(general|(stack|domain|project|area|org):\S.*)$")
EVIDENCE = re.compile(r"^- (\d{4}-\d{2}-\d{2}) · ([^·]+?) · .+ — .+$")
SEED_DAYS = 90


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


def section(body, name):
    m = re.search(rf"^## {re.escape(name)}\s*$(.*?)(?=^## |\Z)", body, flags=re.M | re.S)
    return re.sub(r"<!--.*?-->", "", m.group(1), flags=re.S) if m else ""


def confirmed(log_text, day, stem):
    """log.md에 그날 confirm 기록이 있고 그 기록(제목 · 세부)에 페이지 이름이 있나."""
    for block in re.split(r"(?=^## \[)", log_text, flags=re.M):
        if block.startswith(f"## [{day}] confirm |") and stem in block:
            return True
    return False


def check_guide(rel, path, meta, body, log_text, now, errors, warnings):
    if meta.get("guide_kind") not in GUIDE_KINDS:
        errors.append(f"{rel}: guide_kind는 {sorted(GUIDE_KINDS)} 중 하나")
    scope = meta.get("scope")
    if not isinstance(scope, list) or not scope or not all(SCOPE.match(str(s)) for s in scope):
        errors.append(f"{rel}: scope는 [general] · stack:<기술> · domain:<분야> · project:<이름> · area:<생활 영역> · org:<회사> 목록")
    maturity, owner = meta.get("maturity"), str(meta.get("owner_checked", "")).lower()
    if maturity not in MATURITY:
        errors.append(f"{rel}: maturity는 {sorted(MATURITY)} 중 하나")
    if owner not in ("true", "false"):
        errors.append(f"{rel}: owner_checked는 true · false")
    rules = [l for l in section(body, "Rules").splitlines() if l.strip()]
    if not rules or not all(re.match(r"^(\d+\.\s+\S|\s+\S)", l) for l in rules):
        errors.append(f"{rel}: ## Rules는 번호 목록(1. …)으로 쓴다")
    places = set()
    for line in section(body, "Evidence").splitlines():
        if not line.startswith("- "):
            continue
        m = EVIDENCE.match(line)
        if not m or parse_date(m.group(1)) is None:
            errors.append(f"{rel}: Evidence 줄 형식은 '- YYYY-MM-DD · 어디서 · 결과 — 근거' ({line[:40]})")
        else:
            places.add(m.group(2).strip())
    if isinstance(scope, list) and any(str(s).startswith(("area:", "org:")) for s in scope) and not places:
        errors.append(f"{rel}: 내 방식(area:) · 회사 방식(org:)은 실제로 한 일을 Evidence에 1줄 이상 적어야 한다")
    if not places and not meta.get("sources"):
        errors.append(f"{rel}: 근거 없는 guide — sources나 Evidence에 근거를 적는다")
    if meta.get("status") == "superseded":
        target = wikilib.links(f"x {meta.get('superseded_by') or ''}")
        if not target:
            errors.append(f"{rel}: superseded guide에 superseded_by(\"[[남길 페이지]]\")가 없음")
        elif target == [path.stem]:
            errors.append(f"{rel}: superseded_by가 자기 자신을 가리킴")
    if maturity == "tested" and len(places) < 2:
        errors.append(f"{rel}: maturity tested는 Evidence의 서로 다른 곳이 2개 이상일 때만 ({len(places)}개)")
    if maturity == "proven" and owner != "true":
        errors.append(f"{rel}: maturity proven은 owner_checked: true일 때만")
    if owner == "true":
        day = parse_date(meta.get("owner_checked_on", ""))
        if not day:
            errors.append(f"{rel}: owner_checked: true면 owner_checked_on(YYYY-MM-DD)이 있어야 한다")
        elif not confirmed(log_text, day.isoformat(), path.stem):
            errors.append(f"{rel}: {day} log에 confirm 기록이 없다 — python3.14 tools/log.py confirm \"{path.stem}\"")
    updated = parse_date(meta.get("updated", ""))
    if maturity == "seed" and updated and now - updated > timedelta(days=SEED_DAYS):
        warnings.append(f"{rel}: guide가 seed인 채 {SEED_DAYS}일 넘게 그대로 (updated {updated})")
    return owner == "true"


def norm(name):
    return re.sub(r"[\s\W_]+", "", str(name)).lower()


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
    log_path = root / "wiki" / "log.md"
    log_text = log_path.read_text() if log_path.exists() else ""
    unchecked_guides = 0
    stems, alias_of = {}, {}

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
        if kind == "guide" and not check_guide(rel, path, meta, body, log_text, now, errors, warnings):
            unchecked_guides += 1
        stems.setdefault(norm(path.stem), []).append(rel)
        for a in meta.get("aliases") or []:
            if isinstance(a, str) and a:
                alias_of.setdefault(norm(a), set()).add(path.stem)
        if key_facts(body) > MAX_FACTS:
            warnings.append(f"{rel}: Key facts {key_facts(body)}개 — {MAX_FACTS}개를 넘으면 나눈다")
        if path.stem not in incoming:
            warnings.append(f"{rel}: 들어오는 링크가 없는 고아 페이지")

    for name, rels in names.items():
        md = [r for r in rels if r.suffix == ".md" and r.parts[0] in ("wiki", "raw")]
        if len(md) > 1:
            warnings.append(f"같은 이름 '{name}': {', '.join(map(str, md))}")

    for key, rels in stems.items():
        if len(rels) > 1 and len({r.stem for r in rels}) > 1:
            warnings.append(f"이름이 거의 같은 페이지(중복 후보): {', '.join(map(str, rels))}")
    by_norm = {k: {r.stem for r in v} for k, v in stems.items()}
    for key, owners in alias_of.items():
        clash = by_norm.get(key, set()) - owners
        if clash:
            warnings.append(f"별칭이 다른 페이지 이름과 같음(중복 후보): {sorted(owners)} ↔ {sorted(clash)}")
    if unchecked_guides:
        warnings.append(f"내가 확인하지 않은 guide {unchecked_guides}개 (owner_checked: false) — yw-wiki-review에서 확인")

    for status, rel in staged_raw_changes(root):
        if status in "DR":
            errors.append(f"{rel}: raw/ 파일을 지우거나 옮김 — raw는 고치지 않는다")
        elif status == "M":
            warnings.append(f"{rel}: raw/ 파일을 고침 — 사람이 고친 것이 아니면 되돌린다")

    guard = conf.get("guard", {})
    terms = [re.escape(t) for t in guard.get("terms", []) if t] + [p for p in guard.get("patterns", []) if p]
    if terms:
        allow = set(guard.get("allow", []))
        pattern = re.compile("|".join(terms), re.I)
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

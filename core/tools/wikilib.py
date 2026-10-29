"""check.py · index.py · log.py가 같이 쓰는 것: 위키 루트, 설정, frontmatter, 페이지 목록."""
import re
import sys

if sys.version_info < (3, 14):  # 보안 지원 버전 (3.14: 2030-10까지). tomllib은 3.11+
    sys.exit(f"Python 3.14 이상이 필요하다 (지금 {sys.version.split()[0]}). brew install python@3.14 뒤 python3.14로 실행")

import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SKIP_DIRS = {".git", ".obsidian", ".trash", ".claude", ".githooks", ".yw-wiki", "tools", "_templates"}
# wiki/ 안에서 템플릿을 따르지 않는 생성 · 기록 파일
SPECIAL = {"index.md", "log.md"}
# type → 폴더 (wiki/ 기준)
TYPE_DIRS = {"entity": "entities", "concept": "concepts", "source": "sources", "synthesis": "synthesis",
             "decision": "decisions", "achievement": "career"}
LINK = re.compile(r"\[\[([^\]|#^]+)")


def root_from_argv():
    """첫 인자가 경로면 그 위키를, 아니면 이 tools/가 있는 위키를 본다."""
    args = [a for a in sys.argv[1:] if not a.startswith("-")]
    return Path(args[0]).expanduser().resolve() if args and Path(args[0]).is_dir() else ROOT


def config(root):
    return tomllib.loads((root / "wiki.toml").read_text())


def _value(raw):
    raw = raw.strip()
    if raw.startswith(("'", '"')):
        q = raw[0]
        end = raw.find(q, 1)
        return raw[1:end] if end > 0 else raw[1:]
    raw = re.sub(r"\s+#.*$", "", raw).strip()
    if raw.startswith("#"):
        return ""
    if raw.startswith("[") and raw.endswith("]"):
        inner = raw[1:-1].strip()
        if not inner:
            return []
        items = re.findall(r'"[^"]*"|\'[^\']*\'|[^,]+', inner)
        return [i.strip().strip('"\'') for i in items if i.strip()]
    return raw


def frontmatter(text):
    """(meta, body). 스칼라 · 인라인 목록 · 한 단계 블록 목록만 읽는다. frontmatter가 없으면 meta는 None."""
    if not text.startswith("---\n"):
        return None, text
    end = text.find("\n---", 4)
    if end < 0:
        return None, text
    meta, key = {}, None
    for line in text[4:end].splitlines():
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        m = re.match(r"^\s+-\s*(.*)$", line)
        if m and key is not None:
            if not isinstance(meta[key], list):
                meta[key] = []
            meta[key].append(_value(m.group(1)))
            continue
        k, sep, v = line.partition(":")
        if sep and not line.startswith(" "):
            key = k.strip()
            meta[key] = _value(v)
    body = text[end + 4:]
    return meta, body.lstrip("\n")


def strip_code(text):
    """링크 · 자리표시를 볼 때 코드 블록 · 인라인 코드 · HTML 주석은 뺀다."""
    text = re.sub(r"<!--.*?-->", "", text, flags=re.S)
    text = re.sub(r"```.*?```", "", text, flags=re.S)
    return re.sub(r"`[^`\n]*`", "", text)


def links(text):
    if text.startswith("---\n") and (end := text.find("\n---", 4)) > 0:  # frontmatter 주석(# …) 속 예시는 링크가 아니다
        head = "\n".join(re.sub(r"(^|\s)#(?![^\"']*[\"']\s*$).*$", "", l) for l in text[:end].splitlines())
        text = head + text[end:]
    return [m.strip().split("/")[-1].removesuffix(".md") for m in LINK.findall(strip_code(text))]


def pages(root):
    """wiki/ 아래 페이지 (path, rel). index · log는 뺀다."""
    out = []
    for path in sorted((root / "wiki").rglob("*.md")):
        rel = path.relative_to(root)
        if len(rel.parts) == 2 and rel.name in SPECIAL:
            continue
        out.append((path, rel))
    return out


def all_files(root):
    """링크 대상이 될 수 있는 파일 전부 (볼트 안, 숨김 · 세팅 폴더 제외)."""
    for path in sorted(root.rglob("*")):
        rel = path.relative_to(root)
        if path.is_file() and not (set(rel.parts[:-1]) & SKIP_DIRS) and not rel.parts[0].startswith("."):
            yield path, rel


def summary(body):
    """H1 다음 첫 문단 한 줄 (index에 싣는 요약)."""
    body = re.sub(r"<!--.*?-->", "", body, flags=re.S)
    seen_h1 = False
    for line in body.splitlines():
        s = line.strip()
        if s.startswith("# "):
            seen_h1 = True
            continue
        if seen_h1 and s and not s.startswith(("#", "-", ">", "|", "```")):
            if len(s) <= 120:
                return s
            s = s[:119]
            if s.rfind("[[") > s.rfind("]]"):  # 링크 중간에서 자르지 않는다
                s = s[:s.rfind("[[")].rstrip()
            return s + "…"
        if s.startswith("## "):
            break
    return ""

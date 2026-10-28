"""new_wiki.py · upgrade.py가 같이 쓰는 것: 세팅이 관리하는 파일 목록과 AGENTS.md 합성."""
import re
import sys

if sys.version_info < (3, 14):  # 보안 지원 버전 (3.14: 2030-10까지). tomllib은 3.11+
    sys.exit(f"Python 3.14 이상이 필요하다 (지금 {sys.version.split()[0]}). brew install python@3.14 뒤 python3.14로 실행")

import tomllib
from pathlib import Path

SETTING = Path(__file__).resolve().parent.parent
CORE = SETTING / "core"
PROFILES = ("personal", "work")
REPO = "YeongWon2/yw-wiki-setting"


def version():
    return (SETTING / "VERSION").read_text().strip()


def major(v):
    return int(v.split(".")[0])


def wiki_conf(dest, profile):
    path = dest / "wiki.toml"
    return tomllib.loads(path.read_text() if path.exists() else (SETTING / "profiles" / profile / "wiki.toml").read_text())


def wiki_name(dest, profile):
    path = dest / "wiki.toml"
    conf = tomllib.loads(path.read_text()) if path.exists() else \
        tomllib.loads((SETTING / "profiles" / profile / "wiki.toml").read_text())
    return conf.get("wiki", {}).get("name", dest.name)


def readme(conf):
    """위키 README (3초 읽기: 첫 줄에 무엇인지, 그다음 말하는 법)."""
    w = conf.get("wiki", {})
    return (f"# {w.get('name', '')}\n\n**{w.get('description', '')}를 Claude가 쓰고 유지하는 LLM 위키다. 질문은 `wiki/index.md`부터 읽는다.**\n\n"
            "| 말 | 하는 일 |\n|---|---|\n"
            "| 갱신해줘 | 최신 받기 · 세팅 반영 · 할 일 알리기 |\n"
            "| 업데이트해줘 · 수집해줘 | 원본을 위키에 반영, 충돌은 `review/`로 |\n"
            "| 리뷰해줘 | 검토 대기를 보여 주고 정한 것만 반영 |\n"
            "| 찾아줘 | 목차부터 읽고 근거 링크로 답함 |\n"
            "| 점검해줘 | 자동 검사, 판단할 것은 `review/`로 |\n\n"
            "- 원본은 `raw/`(내가 씀, AI는 읽기만), 위키는 `wiki/`(AI가 씀), 검토 대기는 `review/`.\n"
            "- 규칙은 `AGENTS.md`. 세팅(`yw-wiki-setting`)이 관리하는 파일은 여기서 고치지 않는다.\n")


def managed_files(profile, name, conf=None):
    """세팅이 관리하는 파일: {위키 기준 경로: 내용}. upgrade가 덮어쓴다."""
    out = {"README.md": readme(conf)} if conf else {}
    for src in sorted((CORE / "templates").glob("*.md")) + sorted((SETTING / "profiles" / profile / "templates").glob("*.md")):
        out[f"_templates/{src.name}"] = src.read_text()
    for src in sorted((CORE / "tools").glob("*.py")):
        out[f"tools/{src.name}"] = src.read_text()
    out[".githooks/pre-commit"] = (CORE / "githooks" / "pre-commit").read_text()
    for skill in sorted((CORE / "skills").iterdir()):
        if (skill / "SKILL.md").exists():
            out[f".claude/skills/{skill.name}/SKILL.md"] = (skill / "SKILL.md").read_text().replace("{{wiki_name}}", name)
    return out


EXECUTABLE = re.compile(r"^(tools/.*\.py|\.githooks/.*)$")
SECTION = re.compile(r"<!-- (core|profile|local):start[^>]*-->\n(.*?)\n<!-- \1:end -->", re.S)


def agents(profile, local=None):
    core = (CORE / "AGENTS.core.md").read_text().rstrip()
    prof = (SETTING / "profiles" / profile / "AGENTS.profile.md").read_text().rstrip()
    if local is None:
        local = "# 이 위키에만 있는 규칙\n\n(없음. 이 구간은 upgrade가 건드리지 않는다.)"
    return (f"<!-- core:start v{version()} — yw-wiki-setting이 관리. 여기서 고치지 않는다 -->\n{core}\n<!-- core:end -->\n\n"
            f"<!-- profile:start {profile} — yw-wiki-setting이 관리 -->\n{prof}\n<!-- profile:end -->\n\n"
            f"<!-- local:start -->\n{local.strip()}\n<!-- local:end -->\n")


def local_section(text):
    for kind, body in SECTION.findall(text):
        if kind == "local":
            return body
    return None


def state_toml(profile, managed):
    items = ",\n".join(f'  "{p}"' for p in sorted(managed))
    return (f'# yw-wiki-setting이 쓰는 파일. 손으로 고치지 않는다.\nversion = "{version()}"\nprofile = "{profile}"\n'
            f'setting_repo = "{REPO}"\nmanaged = [\n{items},\n]\n')


def write(dest, rel, text):
    path = dest / rel
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text)
    if EXECUTABLE.match(rel):
        path.chmod(0o755)

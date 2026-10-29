"""new_wiki.py · upgrade.py가 같이 쓰는 것: 세팅이 관리하는 파일 목록과 AGENTS.md 합성."""
import re
import sys

if sys.version_info < (3, 14):  # 보안 지원 버전 (3.14: 2030-10까지). tomllib은 3.11+
    sys.exit(f"Python 3.14 이상이 필요하다 (지금 {sys.version.split()[0]}). brew install python@3.14 뒤 python3.14로 실행")

import tomllib
from pathlib import Path

SETTING = Path(__file__).resolve().parent.parent
CORE = SETTING / "core"
PROFILES = ("personal",)
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
    """위키 README (3초 읽기: 첫 줄 = 무엇인지 → 30초: 말하는 법 · 폴더 → 상세: 새 PC에서 시작)."""
    w = conf.get("wiki", {})
    raw = "`raw/daily/` 일지 · `raw/notes/` 메모 · `raw/assets/` 이미지 · PDF"
    extra = "| `wiki/career/` | 성과 기록 (회사 내용은 걸러서) | Claude |\n"
    return f"""# {w.get('name', '')}

**{w.get('description', '')}를 Claude가 쓰고 유지하는 LLM 위키다. 질문은 `wiki/index.md`부터 읽는다.**

| 말 | 하는 일 |
|---|---|
| 갱신해줘 | 최신 받기 · 세팅 반영 · 할 일 알리기 |
| 업데이트해줘 · 수집해줘 | 원본을 위키에 반영, 충돌은 `review/`로 |
| 리뷰해줘 | 검토 대기를 보여 주고 정한 것만 반영 |
| 찾아줘 | 목차부터 읽고 근거 링크로 답함 |
| 점검해줘 | 자동 검사, 판단할 것은 `review/`로 |

| 폴더 | 무엇 | 누가 쓰나 |
|---|---|---|
| `raw/` | 원본: {raw} | 나 (Claude는 읽기만) |
| `wiki/` | 위키: entities · concepts · sources · synthesis · decisions, 목차 `index.md` · 기록 `log.md` | Claude |
{extra}| `review/` | 사람이 정할 검토 대기 (충돌은 위키에 쓰지 않는다) | Claude가 쌓고 내가 정함 |

## 새 PC에서 시작하기

1. Python 3.14: `brew install python@3.14`
2. 세팅 받고 길잡이 깔기: `git clone https://github.com/YeongWon2/yw-wiki-setting ~/.local/share/wiki/yw-wiki-setting` → `python3.14 ~/.local/share/wiki/yw-wiki-setting/global/wiki-vault/bin/wiki-setting install`
3. 이 위키 받고 등록: `git clone <이 저장소> ~/Desktop/wiki/yw-wiki` → `wiki-setting register <그 폴더>`
4. 폴더에서 Claude Code를 열면 세션 시작 때 최신 확인(main · 세팅 · Obsidian 플러그인)이 돈다. Obsidian으로 폴더를 열면 Git 플러그인이 메모를 원격과 맞춘다.

규칙은 `AGENTS.md`, 만든 이유는 세팅의 `docs/철학.md`. 세팅이 관리하는 파일(`_templates/` · `tools/` · `.claude/skills/` · 이 README)은 여기서 고치지 않는다.
"""


def managed_files(profile, name, conf=None):
    """세팅이 관리하는 파일: {위키 기준 경로: 내용}. upgrade가 덮어쓴다."""
    out = {"README.md": readme(conf)} if conf else {}
    for src in sorted((CORE / "templates").glob("*.md")) + sorted((SETTING / "profiles" / profile / "templates").glob("*.md")):
        out[f"_templates/{src.name}"] = src.read_text()
    for src in sorted((CORE / "tools").rglob("*")):
        if src.is_file() and "__pycache__" not in src.parts:
            out[f"tools/{src.relative_to(CORE / 'tools')}"] = src.read_text()
    out[".claude/settings.json"] = (CORE / "claude" / "settings.json").read_text()  # 세션 시작 때 최신 확인
    out[".githooks/pre-commit"] = (CORE / "githooks" / "pre-commit").read_text()
    for skill in sorted((CORE / "skills").iterdir()):
        if (skill / "SKILL.md").exists():
            out[f".claude/skills/{skill.name}/SKILL.md"] = (skill / "SKILL.md").read_text().replace("{{wiki_name}}", name)
    return out


EXECUTABLE = re.compile(r"^(tools/[^/]*\.py|\.githooks/.*)$")
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


def ensure_gitignore(dest):
    """.gitignore에 세팅이 정한 규칙이 다 있게 한다(위키가 더한 줄은 그대로). 이미 git에 올라간 무시 대상은 추적만 뺀다(파일은 남김).
    바꾼 것 목록을 돌려준다."""
    import subprocess
    path = dest / ".gitignore"
    have = path.read_text() if path.exists() else ""
    have_rules = {l.strip() for l in have.splitlines()}
    need = [l for l in (CORE / "gitignore").read_text().splitlines() if l.strip() and not l.startswith("#")]
    missing = [l for l in need if l.strip() not in have_rules]
    changed = []
    if missing:
        block = "\n# ── yw-wiki-setting이 더한 규칙 ──\n" + "\n".join(missing) + "\n"
        path.write_text((have.rstrip("\n") + "\n" + block) if have else (CORE / "gitignore").read_text())
        changed.append(f".gitignore 규칙 {len(missing)}개")
    if (dest / ".git").exists():
        tracked = subprocess.run(["git", "-C", str(dest), "ls-files", "-ci", "--exclude-standard"],
                                 capture_output=True, text=True).stdout.split("\n")
        tracked = [t for t in tracked if t]
        if tracked:
            subprocess.run(["git", "-C", str(dest), "rm", "-q", "--cached", "--", *tracked], check=True)
            changed.append(f"git 추적에서 뺌: {', '.join(tracked)}")
    return changed


def write(dest, rel, text):
    path = dest / rel
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text)
    if EXECUTABLE.match(rel):
        path.chmod(0o755)

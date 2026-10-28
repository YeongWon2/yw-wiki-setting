#!/usr/bin/env python3.14
"""링크 · 낡은 표현 감사: 위키와 세팅 저장소 전체에서 깨진 링크와 옛 구조를 가리키는 말을 찾는다.

  python3.14 scripts/audit_links.py <저장소> [<저장소> …] [--online]

- 마크다운 링크 [..](경로): 없는 파일, 로컬 절대 경로(/Users/… · ~/…  — 다른 PC에서 깨짐)
- --online: github.com/<owner>/<repo> 링크(저장소 · PR · 이슈 · 커밋 · blob)가 실제로 있는지 gh api로 확인
- 낡은 표현: 옛 PARA 경로 · 옛 스킬 · 옛 파일 이름이 "지금"처럼 남았는지 (기록 문맥은 허용 목록으로 뺀다)
종료 코드: 문제가 있으면 1.
"""
import re
import subprocess
import sys
import urllib.parse
from pathlib import Path

MD_LINK = re.compile(r"(?<!!)\[[^\]]*\]\(<?([^)\s>]+)>?(?:\s+\"[^\"]*\")?\)")
GH = re.compile(r"https://github\.com/([\w.-]+)/([\w.-]+)(?:/(pull|issues|commit|blob|tree)/([^\s)#\]\"',`>]+))?")
SKIP_DIRS = {".git", "__pycache__", ".yw-wiki", "node_modules"}
# 지금 구조처럼 쓰이면 안 되는 옛 말. (기록으로 남기는 문장은 아래 HISTORY 단어가 같은 줄에 있으면 허용)
STALE = [r"0 Inbox\(받은함\)", r"1 Project\(프로젝트\)", r"2 Area\(영역\)", r"3 Resource\(자료\)", r"4 Archive\(보관\)",
         r"_templates\(템플릿\)", r"yw-wiki\.toml", r"\[\[홈\]\]", r"홈\.md", r"conflicts\.md", r"yw-wiki-ingest",
         r"## Contradictions", r"(?<![\w.])python3 (tools|scripts)/", r"\[\[템플릿 목록\]\]", r"🔗 참조", r"\[!hero\]", r"\[!cols\]"]
HISTORY = ("옮김", "옛", "이전", "였다", "했다", "었다", "PARA", "aa1ef83", "pre-llm-wiki", "없앤다", "없앰", "제거", "폐기",
           "대체", "바꿨", "바뀜", "지움", "막는다", "잡는다", "오류", "경고", "검사", "감사", "STALE", "check.py", "옛 키")


def md_files(root):
    for p in sorted(root.rglob("*.md")):
        if not set(p.relative_to(root).parts) & SKIP_DIRS:
            yield p


def strip_code(text):
    text = re.sub(r"<!--.*?-->", "", text, flags=re.S)  # 템플릿 안내 주석
    text = re.sub(r"```.*?```", "", text, flags=re.S)
    return re.sub(r"`[^`\n]*`", "", text)


def gh_exists(owner, repo, kind, ref, cache):
    key = (owner, repo, kind, ref)
    if key in cache:
        return cache[key]
    if kind in ("pull", "issues"):
        api = f"repos/{owner}/{repo}/issues/{ref.split('/')[0]}"
    elif kind == "commit":
        api = f"repos/{owner}/{repo}/commits/{ref.split('/')[0]}"
    elif kind in ("blob", "tree"):
        branch, _, path = ref.partition("/")
        api = f"repos/{owner}/{repo}/contents/{urllib.parse.unquote(path)}?ref={branch}"
    else:
        api = f"repos/{owner}/{repo}"
    r = subprocess.run(["gh", "api", api, "--silent"], capture_output=True, text=True)
    cache[key] = r.returncode == 0 or "rate limit" in r.stderr.lower()
    return cache[key]


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    online = "--online" in sys.argv
    problems, cache = [], {}
    for root in map(lambda a: Path(a).expanduser().resolve(), args):
        for p in md_files(root):
            rel = p.relative_to(root)
            text = p.read_text()
            body = strip_code(text)
            for target in MD_LINK.findall(body):
                if target.startswith(("http://", "https://", "mailto:", "#", "obsidian:")):
                    continue
                path = urllib.parse.unquote(target.split("#")[0])
                if path.startswith(("/", "~")):
                    problems.append(f"{root.name}/{rel}: 로컬 절대 경로 링크 {target} — 다른 PC에서 깨진다")
                    continue
                if path and not (p.parent / path).exists():
                    problems.append(f"{root.name}/{rel}: 없는 파일 링크 {target}")
            if online:
                for owner, repo, kind, ref in GH.findall(body):
                    repo = repo.removesuffix(".git")
                    if not gh_exists(owner, repo, kind, ref, cache):
                        problems.append(f"{root.name}/{rel}: GitHub에 없음 https://github.com/{owner}/{repo}" + (f"/{kind}/{ref}" if kind else ""))
            if rel.name == "log.md":  # 작업 기록은 지난 일을 적는 곳이라 옛 말이 있어도 된다
                continue
            for i, line in enumerate(body.splitlines(), 1):
                for pat in STALE:
                    if re.search(pat, line) and not any(h in line for h in HISTORY):
                        problems.append(f"{root.name}/{rel}:{i}: 낡은 표현 /{pat}/ — {line.strip()[:120]}")
    for line in problems:
        print(line)
    print(f"문제 {len(problems)}개")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())

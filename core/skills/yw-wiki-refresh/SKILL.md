---
name: yw-wiki-refresh
description: "[{{wiki_name}}] 위키를 main 기준으로 완전히 최신화한다(완전 최신화 · 브랜치 정리). main으로 돌아와 원격 main을 받고, 세팅 새 버전을 반영하고, main 말고 다른 브랜치(로컬 · 원격)를 지운다. main에 합쳐지지 않은 브랜치는 보여 주고 물은 뒤에만 지운다. 위키 내용은 쓰지 않는다. \"{{wiki_name}} 완전 최신화해줘\", \"main 기준으로 맞춰줘\", \"브랜치 정리해줘\" 요청에 쓴다. 받기와 알리기만 하는 것은 yw-wiki-sync(갱신)."
---

# yw-wiki-refresh — 완전 최신화 ({{wiki_name}})

끝나면 이 위키에는 main 하나만 남고, main은 origin/main · 세팅 최신 릴리스와 같다. 위키 내용(`wiki/`)은 쓰지 않는다.

0. **최신 확인:** `python3.14 tools/version.py --force`. ⚠가 나오면 먼저 알리고 계속한다.
1. **멈출 것 확인:** `git status --porcelain`에 커밋 안 한 변경이 있으면 멈추고 그 목록을 보여 준다. 지우거나 stash하지 않는다. rebase · merge가 진행 중이어도 멈춘다.
2. **main으로:** `git fetch --prune --tags origin` → `git switch main` → `git pull --rebase`. 충돌이 나면 `git rebase --abort`로 되돌리고 멈춘다. main이 origin/main보다 앞서 있으면 `git push`(force push는 하지 않는다).
3. **세팅:** [yw-wiki-sync](../yw-wiki-sync/SKILL.md) 2단계와 같다(`wiki-setting upgrade`. 메이저는 무엇이 바뀌는지 보여 주고 묻는다).
4. **다른 브랜치 지우기:** main · origin/main · origin/HEAD 말고 모두 대상이다.
   - main에 합쳐진 것은 묻지 않고 지운다. 로컬은 `git branch --merged main`에 있는 것을 `git branch -d`, 원격은 `git branch -r --merged origin/main`에 있는 것을 `git push origin --delete <브랜치>`.
   - main에 합쳐지지 않은 것은 지우지 않고 브랜치마다 main에 없는 커밋 수(`git rev-list --count main..<브랜치>`) · 마지막 커밋 · 열린 PR 여부(`gh pr list --head <브랜치>`)를 보여 주고 묻는다. 사람이 지우라고 한 것만 `git branch -D` · `git push origin --delete`. 지우면 되돌릴 수 없다(AGENTS.md §4).
   - 다른 worktree가 쓰고 있는 브랜치는 건너뛰고 그 경로를 알린다. 끝에 `git worktree prune`.
5. **확인:** `git branch -a`에 main · origin/main(· origin/HEAD)만 있는지, `git status -sb` 첫 줄이 `## main...origin/main`인지, `python3.14 tools/version.py`에 ⚠가 없는지, `python3.14 tools/check.py` 오류 수.
6. 세팅 반영으로 파일이 바뀌었으면 `python3.14 tools/log.py sync "완전 최신화 · 세팅 vX → vY"` 뒤 커밋 · push. 브랜치만 지웠으면 커밋하지 않는다.

보고 첫 줄: "main 최신 (세팅 vX) · 지운 브랜치 N · 남긴 브랜치 M". 남긴 브랜치가 있으면 이유(사람이 남기기로 함 · worktree 사용 중)를 적는다.

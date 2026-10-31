---
name: yw-setting-refresh
description: "[yw-wiki-setting] 이 세팅 저장소를 main 기준으로 완전히 최신화한다(완전 최신화 · 브랜치 정리). main으로 돌아와 원격 main과 태그를 받고, main 말고 다른 브랜치(로컬 · 원격)를 지우고, 이 PC의 세팅 사본 · 길잡이 스킬까지 최신 릴리스로 맞춘다. main에 합쳐지지 않은 브랜치는 보여 주고 물은 뒤에만 지운다. \"세팅 완전 최신화해줘\", \"세팅 main 기준으로 맞춰줘\", \"세팅 브랜치 정리해줘\" 요청에 쓴다."
---

# yw-setting-refresh — 완전 최신화 (yw-wiki-setting)

위치는 이 저장소 루트(개발 사본)다. 끝나면 main 하나만 남고, main은 origin/main과 같고, 이 PC의 세팅 사본(`~/.local/share/wiki/yw-wiki-setting`) · 길잡이 스킬은 main의 최신 릴리스 태그다. 각 위키는 그 위키의 `yw-wiki-refresh`가 맡는다.

1. **멈출 것 확인:** `git status --porcelain`에 커밋 안 한 변경이 있으면 멈추고 그 목록을 보여 준다. 지우거나 stash하지 않는다. rebase · merge가 진행 중이어도 멈춘다.
2. **main으로:** `git fetch --prune --tags origin` → `git switch main` → `git pull --rebase`. 충돌이 나면 `git rebase --abort`로 되돌리고 멈춘다. main이 origin/main보다 앞서 있으면 무엇이 앞섰는지 보여 주고 push할지 묻는다(릴리스 전 커밋일 수 있다). force push는 하지 않는다.
3. **다른 브랜치 지우기:** main · origin/main · origin/HEAD 말고 모두 대상이다.
   - main에 합쳐진 것은 묻지 않고 지운다. 로컬은 `git branch --merged main`에 있는 것을 `git branch -d`, 원격은 `git branch -r --merged origin/main`에 있는 것을 `git push origin --delete <브랜치>`.
   - main에 합쳐지지 않은 것은 지우지 않고 브랜치마다 main에 없는 커밋 수(`git rev-list --count main..<브랜치>`) · 마지막 커밋 · 열린 PR 여부(`gh pr list --head <브랜치>`)를 보여 주고 묻는다. 열린 PR의 브랜치는 기여 중인 작업이다. 사람이 지우라고 한 것만 `git branch -D` · `git push origin --delete`. 지우면 되돌릴 수 없다.
   - 다른 worktree가 쓰고 있는 브랜치는 건너뛰고 그 경로를 알린다. 끝에 `git worktree prune`.
   - 원격에 없는 로컬 태그(`git tag` 중 `git ls-remote --tags origin`에 없는 것)는 지우지 않고 알린다.
4. **이 PC 맞추기:** `wiki-setting check` — 세팅 사본을 main의 최신 릴리스 태그로, 낡은 길잡이 스킬은 다시 깐다. ⚠가 나오면 그대로 알린다.
5. **확인:** `git branch -a`에 main · origin/main(· origin/HEAD)만 있는지, `git status -sb` 첫 줄이 `## main...origin/main`인지, `VERSION`과 최신 태그(`git describe --tags --abbrev=0 origin/main`).

보고 첫 줄: "main 최신 (vX) · 지운 브랜치 N · 남긴 브랜치 M". 남긴 브랜치가 있으면 이유(사람이 남기기로 함 · 열린 PR · worktree 사용 중)를 적는다.

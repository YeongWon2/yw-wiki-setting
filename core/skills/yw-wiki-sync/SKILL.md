---
name: yw-wiki-sync
description: "[{{wiki_name}}] 위키를 main 기준으로 완전히 동기화한다(동기화 · 갱신 · sync). main으로 돌아와 원격 main을 받고, 세팅(yw-wiki-setting) 새 버전을 반영하고, main 말고 다른 브랜치(로컬 · 원격)를 지우고, 위키 폴더가 옮겨졌거나 이름이 바뀌었으면 Obsidian 볼트도 따라 옮긴다. main에 합쳐지지 않은 브랜치는 보여 주고 물은 뒤에만 지운다. 끝에 아직 반영 안 된 raw와 검토 대기를 알린다. 위키 내용은 쓰지 않는다. \"{{wiki_name}} 동기화해줘\", \"갱신해줘\", \"sync 해줘\", \"main 기준으로 맞춰줘\", \"브랜치 정리해줘\" 요청에 쓴다."
---

# yw-wiki-sync — 동기화 ({{wiki_name}})

끝나면 이 위키에는 main 하나만 남고, main은 origin/main · 세팅 최신 릴리스와 같고, Obsidian 볼트는 지금 위키 폴더를 가리킨다. 위키 내용(`wiki/`)은 쓰지 않는다. 아래 `wiki-setting`은 `~/.claude/skills/wiki-vault/bin/wiki-setting`이다.

0. **최신 확인:** `python3.14 tools/version.py --force`. ⚠가 나오면 먼저 알리고 계속한다.
1. **멈출 것 확인:** `git status --porcelain`에 커밋 안 한 변경이 있으면 멈추고 그 목록을 보여 준다. 지우거나 stash하지 않는다. rebase · merge가 진행 중이어도 멈춘다.
2. **main으로:** `git fetch --prune --tags origin` → `git switch main` → `git pull --rebase`. 충돌이 나면 `git rebase --abort`로 되돌리고, 어느 파일이 어떻게 다른지 보여 준 뒤 멈춘다. main이 origin/main보다 앞서 있으면 `git push`(force push는 하지 않는다).
3. **세팅:** `.wiki-setting.toml`의 버전과 세팅 최신 릴리스를 비교한다(`wiki-setting upgrade`, 없으면 `python3.14 <yw-wiki-setting>/scripts/upgrade.py .`). 패치 · 마이너는 반영하고, 메이저는 무엇이 바뀌는지 보여 주고 묻는다. 세팅에 접속할 수 없으면 가진 버전으로 계속하고 그렇게 알린다.
4. **다른 브랜치 지우기:** main · origin/main · origin/HEAD 말고 모두 대상이다.
   - main에 합쳐진 것은 묻지 않고 지운다. 로컬은 `git branch --merged main`에 있는 것을 `git branch -d`, 원격은 `git branch -r --merged origin/main`에 있는 것을 `git push origin --delete <브랜치>`.
   - main에 합쳐지지 않은 것은 지우지 않고 브랜치마다 main에 없는 커밋 수(`git rev-list --count main..<브랜치>`) · 마지막 커밋 · 열린 PR 여부(`gh pr list --head <브랜치>`)를 보여 주고 묻는다. 사람이 지우라고 한 것만 `git branch -D` · `git push origin --delete`. 지우면 되돌릴 수 없다(AGENTS.md §4).
   - 다른 worktree가 쓰고 있는 브랜치는 건너뛰고 그 경로를 알린다. 끝에 `git worktree prune`.
5. **폴더 위치 · Obsidian:**
   - `wiki-setting doctor --fix` — 볼트 목록(`~/.config/wiki/vaults.toml`)의 경로가 옮겨졌거나 이름이 바뀐 위키 폴더를 따라가게 고친다. 후보가 여럿이면 사람이 고른다.
   - `wiki-setting obsidian --fix` — Obsidian 볼트 목록이 지금 위키 폴더를 가리키게 옮기고, 볼트 설정(새 노트 · 첨부 · 템플릿 · 일지 폴더)이 없어진 폴더를 가리키면 세팅 값으로 맞춘다.
   - 폴더가 없는 Obsidian 볼트 항목이 나오면 보여 주고 지울지 묻는다. 지우라고 하면 `--prune`을 붙인다.
   - **Obsidian이 켜져 있어 고치지 못하면 사람에게 묻는다: "Obsidian을 잠깐 꺼야 고칠 수 있습니다. 끌까요?"** (질문 도구가 있으면 선택지로). 예면 같은 명령에 `--restart`를 붙인다(정상 종료 → 고침 → 다시 켬). 아니오면 고치지 않은 것을 보고에 남긴다. 묻지 않고 끄지 않는다.
6. **확인:** `git branch -a`에 main · origin/main(· origin/HEAD)만 있는지, `git status -sb` 첫 줄이 `## main...origin/main`인지, `python3.14 tools/version.py`에 ⚠가 없는지, `wiki-setting obsidian`이 "Obsidian 맞음"인지.
7. **알리기:**
   - 반영 안 된 raw: `raw/` 파일 중 어느 source의 `sources`에도 없는 것 → "업데이트할 것 N개" (그 목록).
   - 검토 대기: `review/`의 pending 개수와 가장 오래된 날짜.
   - `python3.14 tools/check.py` 결과 요약 (오류 · 경고 수).
8. 세팅 반영으로 파일이 바뀌었으면 `python3.14 tools/log.py sync "세팅 vX → vY"` 뒤 커밋 · push. 브랜치 · Obsidian만 고쳤으면 커밋하지 않는다(둘 다 위키 저장소 밖).

보고 첫 줄: "동기화 (세팅 vX) · 지운 브랜치 N · 남긴 브랜치 M · Obsidian 맞음 · 업데이트할 raw K · 검토 대기 L". 남긴 브랜치가 있으면 이유(사람이 남기기로 함 · worktree 사용 중)를 적는다. 다음 할 일로 [yw-wiki-update](../yw-wiki-update/SKILL.md) 또는 [yw-wiki-review](../yw-wiki-review/SKILL.md)를 권한다.

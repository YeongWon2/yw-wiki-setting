---
name: yw-wiki-sync
description: "[{{wiki_name}}] 위키를 최신으로 맞춘다(갱신 · sync). 원격 main을 받고, 세팅(yw-wiki-setting) 새 버전이 있으면 반영하고, 아직 위키에 반영되지 않은 raw 파일과 검토 대기(review/)를 알려 준다. 위키 내용은 쓰지 않는다. \"{{wiki_name}} 갱신해줘\", \"sync 해줘\", \"최신으로 맞춰줘\" 요청에 쓴다."
---

# yw-wiki-sync — 갱신 ({{wiki_name}})

위키 내용(`wiki/`)은 쓰지 않는다. 받기와 알리기만 한다.

1. **받기:** `git pull --rebase`. 충돌이 나면 `git rebase --abort`로 되돌리고, 어느 파일이 어떻게 다른지 보여 준 뒤 멈춘다. force push는 하지 않는다.
2. **세팅:** `.wiki-setting.toml`의 버전과 세팅 최신 릴리스를 비교한다(`wiki-setting upgrade`, 없으면 `python3.14 <yw-wiki-setting>/scripts/upgrade.py .`). 패치 · 마이너는 반영하고, 메이저는 무엇이 바뀌는지 보여 주고 묻는다. 세팅에 접속할 수 없으면 가진 버전으로 계속하고 그렇게 알린다.
3. **알리기:**
   - 반영 안 된 raw: `raw/` 파일 중 어느 source의 `sources`에도 없는 것 → "업데이트할 것 N개" (그 목록).
   - 검토 대기: `review/`의 pending 개수와 가장 오래된 날짜.
   - `python3.14 tools/check.py` 결과 요약 (오류 · 경고 수).
4. 세팅 반영으로 파일이 바뀌었으면 `python3.14 tools/log.py sync "세팅 vX → vY"` 뒤 커밋 · push. 바뀐 것이 없으면 커밋하지 않는다.

보고 첫 줄: "최신 (세팅 vX) · 업데이트할 raw N · 검토 대기 M". 다음 할 일로 [yw-wiki-update](../yw-wiki-update/SKILL.md) 또는 [yw-wiki-review](../yw-wiki-review/SKILL.md)를 권한다.

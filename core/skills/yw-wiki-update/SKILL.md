---
name: yw-wiki-update
description: "[{{wiki_name}}] 원본을 위키에 반영한다(업데이트 · ingest). raw/의 새 파일이나 사용자가 준 자료를 읽어 source 요약을 쓰고, 관련 entity · concept · decision과 이웃 페이지의 연결을 갱신한 뒤 index · log를 고친다. work 위키는 회사 출처(Jira · Slack · 노션 · GitHub) 수집도 한다. \"{{wiki_name}} 업데이트해줘\", \"이거 넣어줘\", \"수집해줘\", \"<프로젝트> 수집해줘\" 요청에 쓴다. 원격 최신만 받는 것은 yw-wiki-sync(갱신), 전사 위키의 wiki-update와는 다르다."
---

# yw-wiki-update — 업데이트 ({{wiki_name}})

위치는 `wiki.toml`이 있는 위키 루트다. 규칙은 `AGENTS.md`이고, 이 스킬은 그 순서를 풀어 둔 것이다.

## 0. 시작

`python3.14 tools/version.py` (main · 원격 · 세팅 · 플러그인 최신 확인, ⚠면 먼저 알림) 뒤 `git pull --rebase`. git 충돌이 나면 멈추고 사용자에게 넘긴다.

## 1. 범위

- 사용자가 준 파일 · 링크 · 대화 내용, 또는 `raw/` 중 아직 source가 없는 파일(`git log`로 새로 들어온 것).
- **work 위키의 "수집해줘":** `wiki.toml` `[company.sources]` 전체, "<프로젝트> 수집해줘"는 `[company.projects."<프로젝트>"]`만. 원본 도구는 읽기만 한다. 읽은 것은 `raw/collected/YYYY-MM-DD <출처> <제목>.md`에 원본 URL · ID · 확인 날짜와 필요한 부분만 텍스트로 남긴다. 권한이 없어 못 읽은 곳은 보고에 적는다.
- 회사 업무 자료가 개인 위키로 들어오면 쓰지 않고 업무 위키로 보낼지 묻는다.

## 2. 쓰기

1. **source:** 원본 하나당 `wiki/sources/`에 source 템플릿으로 요약 하나. 이미 있으면 `checked` 이후 바뀐 것만 고치고 `checked`를 오늘로.
2. **갱신:** 원본이 말하는 entity · concept · decision을 찾아 Key facts · Timeline에 근거와 함께 넣는다. 없으면 템플릿으로 새로 만든다. 사용자가 직접 말한 결정은 `accepted`, 추론한 결정은 `proposed` + `confidence: low`. work 결정은 다음 DEC-ID를 붙인다.
3. **이웃:** 새로 만들거나 크게 바뀐 페이지와 연결된 페이지의 Connections와 요약을 함께 고친다.
4. **충돌 (사람 검토 필수):** 새 원본이 기존 사실과 다르거나 원본끼리 다르면 그 항목은 `wiki/`에 쓰지 않고 `review/YYYY-MM-DD <페이지> <요약>.md`에 review 템플릿(`found_by: update`)으로 남긴다. 위키 페이지에 충돌 내용을 적지 않는다.
5. 고친 페이지는 `updated`를 오늘로 바꾼다. `raw/`는 고치지 않는다.

## 3. 마무리

```sh
python3.14 tools/index.py
python3.14 tools/log.py update "<제목>" "source N" "갱신 M" "review K"
python3.14 tools/check.py
```

오류를 고친 뒤 커밋하고 push한다. 보고는 첫 줄에 결론("source N · 갱신 M · 검토 K"), 그 뒤에 새 페이지 · 고친 페이지 · review/에 남긴 것 · 못 읽은 곳. 검토가 생겼으면 이어서 [yw-wiki-review](../yw-wiki-review/SKILL.md)를 권한다.

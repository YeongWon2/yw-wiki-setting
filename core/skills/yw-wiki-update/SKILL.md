---
name: yw-wiki-update
description: "[{{wiki_name}}] 원본을 위키에 반영한다(업데이트 · ingest). raw/의 새 파일이나 사용자가 준 자료를 읽어 source 요약을 쓰고, 관련 entity · concept · guide · decision에 합치기 판정(같음 · 보강 · 범위 다름 · 모순)으로 반영하고 이웃 페이지의 연결을 갱신한 뒤 index · log를 고친다. 다른 프로젝트 세션에서 불리면 그 대화의 원칙 · 규칙 · 절차 · 노하우 후보를 뽑아 고르게 한다. \"{{wiki_name}} 업데이트해줘\", \"이거 넣어줘\", \"이 노하우 위키에 남겨줘\" 요청에 쓴다. 원격 최신만 받는 것은 yw-wiki-sync(갱신), 전사 위키의 wiki-update와는 다르다."
---

# yw-wiki-update — 업데이트 ({{wiki_name}})

위치는 `wiki.toml`이 있는 위키 루트다. 규칙은 `AGENTS.md`이고, 이 스킬은 그 순서를 풀어 둔 것이다.

## 0. 시작

`python3.14 tools/version.py` (main · 원격 · 세팅 · 플러그인 최신 확인, ⚠면 먼저 알림) 뒤 `git pull --rebase`. git 충돌이 나면 멈추고 사용자에게 넘긴다.

## 1. 범위

- 사용자가 준 파일 · 링크 · 대화 내용, 또는 `raw/` 중 아직 source가 없는 파일(`git log`로 새로 들어온 것).
- **다른 프로젝트 세션에서 불렸을 때:** 그 대화에서 다시 쓸 만한 지식 후보(원칙 · 규칙 · 절차 · 파이프라인 · 도메인 개념 · 노하우 · 성과)를 유형별 번호 목록으로 보여 주고, 사용자가 고른 것만 쓴다. 자동으로 넓게 모으지 않는다.
- **회사 자료:** AGENTS.md profile 규칙(회사 내용)의 표를 따른다. 범용 지식은 식별 정보를 걷어 내고 guide · concept로, 한 일은 career/로 나눠 쓴다. 넣지 않는 칸(내부 수치 · 계약 · 보안 · 공개 전 계획)은 쓰지 않는다. 걷어 낸 문장을 쓰기 전에 사용자에게 보여 준다.

## 2. 쓰기

1. **source:** 원본 하나당 `wiki/sources/`에 source 템플릿으로 요약 하나. 이미 있으면 `checked` 이후 바뀐 것만 고치고 `checked`를 오늘로.
2. **내 방식 먼저:** 일지 · 여행 · 작업 기록 · 성과처럼 내가 한 일이 들어오면 먼저 "여기서 드러난 내 방식이 있나"를 본다. 기존 `내 방식 - …` guide와 같으면 그 `## Evidence`에 한 줄 더한다(같음). 처음 보는 방식은 새 페이지를 바로 만들지 말고 후보로 보여 주고, 사용자가 고른 것만 `maturity: seed` · `scope: [area:<영역>]`로 만든다. 새 페이지는 `내 방식 지도`에 링크한다. (AGENTS.md profile 0절)
3. **찾기 먼저:** 지식 하나를 넣기 전에 `wiki/index.md` · 제목 · `aliases` · `tags`로 기존 페이지를 찾는다. 유형은 AGENTS.md profile 1절의 고르는 기준(무엇인가 → concept, 어떻게 하나 · 무엇을 지키나 → guide)을 따른다.
4. **합치기 판정:** AGENTS.md profile의 합치기 판정 표로 하나를 고른다.
   - 같음 → 새 페이지 없이 guide `## Evidence`에 `- YYYY-MM-DD · 어디서 · 결과 — 근거` 한 줄(다른 유형은 Key facts 근거 추가).
   - 보강 → 기존 페이지에 합친다. 범위 다름 → `scope`와 Applies when · Exceptions에 둘 다.
   - Evidence의 서로 다른 곳이 2개가 되면 `maturity: tested`. `owner_checked`는 사용자가 "맞다"고 말한 때만 true(`owner_checked_on` + `log.py confirm "<페이지>"`). `proven`은 그 뒤에만.
   - 없으면 템플릿으로 새로 만든다(guide는 `maturity: seed`, `owner_checked: false`). 사용자가 직접 말한 결정은 `accepted`, 추론한 결정은 `proposed` + `confidence: low`.
5. **이웃:** 새로 만들거나 크게 바뀐 페이지와 연결된 페이지의 Connections와 요약을 함께 고친다.
6. **guide끼리 겹침 · 어긋남 (직접 해결):** AGENTS.md profile 합치기 판정대로 직접 합치고 `log.py merge`로 남긴다. 묻지 않는다.
7. **사실 모순 · guide 아닌 중복 (사람 검토 필수):** 날짜 · 이력 · 수치 같은 사실이 기존과 다르거나 원본끼리 다르면, 또는 guide가 아닌 페이지의 같은 이름 후보면 그 항목은 `wiki/`에 쓰지 않고 `review/YYYY-MM-DD <페이지> <요약>.md`에 review 템플릿(`found_by: update`)으로 남긴다. 위키 페이지에 충돌 내용을 적지 않는다.
8. 고친 페이지는 `updated`를 오늘로 바꾼다. `raw/`는 고치지 않는다.

## 3. 마무리

```sh
python3.14 tools/index.py
python3.14 tools/log.py update "<제목>" "source N" "새 페이지 M" "같음 N" "보강 N" "범위 N" "합침 N" "모순 N"
python3.14 tools/check.py
```

오류를 고친 뒤 커밋하고 push한다. 보고는 첫 줄에 결론("source N · 새 페이지 M · 같음 · 보강 · 범위 · 모순 개수"), 그 뒤에 새 페이지 · 고친 페이지 · review/에 남긴 것 · 못 읽은 곳. 검토가 생겼으면 이어서 [yw-wiki-review](../yw-wiki-review/SKILL.md)를 권한다.

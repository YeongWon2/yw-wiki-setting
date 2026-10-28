---
name: yw-wiki-lint
description: "[{{wiki_name}}] 위키를 점검하고 고친다(lint). tools/check.py로 frontmatter · 템플릿 · 링크 · 날짜 · raw 불변 · index를 보고, 기계적인 것은 바로 고치며, 모순 · 낡은 주장 · 고아 페이지는 review/에 남긴다. work 위키는 DEC-ID와 전사 위키 대조도 한다. \"{{wiki_name}} 점검해줘\", \"위키 lint\", \"위키 정리해줘\" 요청에 쓴다."
---

# yw-wiki-lint — 점검 ({{wiki_name}})

## 1. 점검

`git pull --rebase` 뒤 `python3.14 tools/check.py`. 무엇을 보는지는 그 파일 맨 위 설명에 있다. 커밋할 때 `.githooks/pre-commit`이 같은 점검을 한다.

## 2. 바로 고치는 것 (묻지 않음)

- index가 최신이 아님 → `python3.14 tools/index.py`.
- frontmatter 형식: 빠진 공통 필드 추가, 날짜 형식, `updated`.
- 템플릿 제목이 빠짐 → 템플릿 제목 그대로 빈 칸을 넣는다.
- 깨진 링크 중 대상 후보가 하나뿐인 것(이름 오타, 이름이 바뀐 페이지).
- Key facts가 12개를 넘음 → 주제별로 페이지를 나누고 서로 링크한다.

## 3. review/에 남기는 것

위키 페이지는 고치지 않고 항목마다 `review/`에 review 템플릿(`found_by: lint`)으로 남긴다. 이미 같은 항목이 있으면 새로 만들지 않는다. 정하는 것은 [yw-wiki-review](../yw-wiki-review/SKILL.md)다.

- 페이지끼리 사실이 다름, 원본이 바뀌어 낡은 주장, 오래된 proposed 결정.
- 고아 페이지(어디에 연결할지, 보관할지), 같은 이름 후보, 대상이 둘 이상인 깨진 링크.
- `raw/` 파일을 고치거나 지운 흔적 (되돌릴지).

**묻고 하는 것:** 페이지 삭제, `status` 를 archived로 바꾸기(사용자가 끝났다고 말한 것은 바로), 히스토리 재작성.

## 4. work 위키만

- DEC-ID 누락 · 중복은 오류다. 다음 번호를 붙여 고친다.
- **대조(주 1회):** `[company.company_wiki] path`의 전사 위키를 `git pull`한 뒤, `published`가 있는 결정과 `review: merged` 초안이 가리키는 전사 위키 페이지를 읽어 내용이 다른 것을 review/에 `review_kind: reconcile`로 남긴다. 전사 위키에는 쓰지 않는다.

## 5. 마무리

`python3.14 tools/check.py` 오류 0 확인 → `python3.14 tools/log.py lint "<요약>" "고침 N" "review K"` → 커밋 · push. 보고 첫 줄은 "오류 N → 0, 고침 M, review K".

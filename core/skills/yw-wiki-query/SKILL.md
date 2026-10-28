---
name: yw-wiki-query
description: "[{{wiki_name}}] 위키에 적힌 내용으로 질문에 답한다(query). wiki/index.md에서 출발해 entity · synthesis → 세부 페이지 순으로 읽고 근거 링크와 함께 답하며, 사실 · 추정 · 위키에 없는 것을 구분한다. 다시 쓸 만한 답은 synthesis로 저장한다. \"{{wiki_name}}에서 찾아줘\", \"<프로젝트> 어디까지 했지\", \"<날짜>에 뭐 했지\" 요청에 쓴다. 전사 위키의 wiki-query와 다르다."
---

# yw-wiki-query — 찾기 ({{wiki_name}})

이 위키의 페이지와 그 근거 원본만 쓴다. 다른 위키나 원본 도구는 사용자가 따로 말할 때만 본다.

1. **시작:** `python3.14 tools/version.py` (main · 원격 · 세팅 · 플러그인 최신 확인, ⚠면 먼저 알림) 뒤 `git pull --rebase`.
2. **읽는 순서:** `wiki/index.md` → 질문의 entity(사람 · 조직 · 프로젝트)나 synthesis → Connections를 따라 concept · source · decision. 날짜 질문은 `raw/daily/`(work는 `raw/worklog/`)와 그날의 source도 본다. 필요한 페이지만 연다.
3. **읽는 법:** `status: superseded` 결정은 `superseded_by`를 따라가 현재 결정을 답한다. `confidence: low`와 `[low confidence]`는 추정이라고 밝힌다. `review/`는 근거로 쓰지 않는다. 질문 대상에 pending 검토 항목이 있으면 "검토 중인 내용이 있음"이라고만 알린다.
4. **답하는 법:** 첫 줄에 결론. 그 뒤에 근거 `[[페이지]]`와 페이지에 적힌 원본 링크. 위키에 없으면 "위키에 없음"이라고 말하고 지어내지 않는다.
5. **저장:** 여러 페이지를 묶은 답이고 다시 물을 만하면 `wiki/synthesis/`에 synthesis 템플릿으로 저장하고 묶은 페이지를 `related`에 넣는다. 그다음 `python3.14 tools/index.py`, `python3.14 tools/log.py query "<질문 요약>"`, `python3.14 tools/check.py`, 커밋 · push. 한 번 묻고 끝날 답은 저장하지 않는다.
6. 답하다가 틀리거나 낡은 페이지를 보면 고치지 말고 알린다. 고치는 것은 [yw-wiki-update](../yw-wiki-update/SKILL.md) · [yw-wiki-review](../yw-wiki-review/SKILL.md) · [yw-wiki-lint](../yw-wiki-lint/SKILL.md)로 한다.

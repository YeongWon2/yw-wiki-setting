---
name: yw-wiki-update
description: "[{{wiki_name}}] 원본을 위키에 반영한다(업데이트 · ingest). raw/의 새 파일이나 사용자가 준 자료를 읽어 source 요약을 쓰고, 관련 entity · concept · guide · decision에 합치기 판정(같음 · 보강 · 범위 다름 · 모순)으로 반영하고 이웃 페이지의 연결을 갱신한 뒤 index · log를 고친다. 코드 저장소(사이드 · 회사 프로젝트)를 주면 규칙 · 아키텍처 · 방법론 · 파이프라인 · 작성 방식 · 이력을 조사해 guide로 쌓는다. 다른 프로젝트 세션에서 불리면 그 대화의 원칙 · 규칙 · 절차 · 노하우 후보를 뽑아 고르게 한다. \"{{wiki_name}} 업데이트해줘\", \"이거 넣어줘\", \"이 노하우 위키에 남겨줘\", \"이 저장소로 업데이트해줘\" 요청에 쓴다. 원격 최신만 받는 것은 yw-wiki-sync(갱신), 전사 위키의 wiki-update와는 다르다."
---

# yw-wiki-update — 업데이트 ({{wiki_name}})

원본(자료 · 링크 · 대화 · raw/ 새 파일)을 위키에 반영한다. 규칙은 `AGENTS.md`(합치기 판정 · 회사 내용 · 사실/결정/추정 구분)이고, 이 스킬은 그 순서를 흐름으로 묶은 것이다.

## 흐름 (하네스)

단계는 `tools/flow.py`가 하나씩 안내하고, 검사를 통과해야 다음으로 넘긴다. 단계를 건너뛰거나 순서를 바꾸지 않는다.

```sh
python3.14 tools/flow.py start update
python3.14 tools/flow.py done --note "<이 단계에서 한 일 · 사용자 답>"   # 검사 실패면 고치고 다시
python3.14 tools/flow.py next                                        # 지금 단계 다시 보기
```

각 단계의 할 일과 검사는 `tools/flows/update.toml`에 있다. optional 단계만 `skip --note "이유"`로 넘길 수 있다.

- update-code: `python3.14 tools/flow.py start update-code` — 코드 저장소(사이드 · 회사 프로젝트의 경로나 GitHub 주소)를 주면 이 흐름: 규칙 · 아키텍처 · 방법론 · 파이프라인 · 작성 방식 · 이력을 전부 조사해 `<프로젝트> 방식 - …` guide로 쌓는다. 저장소는 읽기만 하고, 내가 쓴 커밋 · 줄만 근거로 쓴다(작성자로 거른다).

## 지키는 것

- 충돌(사실 모순 · guide 아닌 같은 이름)은 위키에 쓰지 않고 `review/`로. guide끼리 겹치면 직접 합친다.
- `owner_checked`는 사용자가 "맞다"고 한 때만 올린다. `raw/`는 고치지 않는다.

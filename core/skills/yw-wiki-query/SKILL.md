---
name: yw-wiki-query
description: "[{{wiki_name}}] 위키에 적힌 내용으로 질문에 답한다(query). wiki/index.md에서 출발해 entity · synthesis → 세부 페이지 순으로 읽고 근거 링크와 함께 답하며, 사실 · 추정 · 위키에 없는 것을 구분한다. 다시 쓸 만한 답은 synthesis로 저장한다. \"{{wiki_name}}에서 찾아줘\", \"<프로젝트> 어디까지 했지\", \"<날짜>에 뭐 했지\", \"내 원칙 · 컨벤션대로 하면?\", \"내가 하던 방식\", \"이 작업 노하우 있었나\", \"나는 보통 어떻게 해?\", \"내 여행 · 생활 방식\" 요청에 쓴다. 전사 위키의 wiki-query와 다르다."
---

# yw-wiki-query — 찾기 ({{wiki_name}})

이 위키의 페이지와 그 근거 원본만 쓴다. 다른 위키나 원본 도구는 사용자가 따로 말할 때만 본다. 틀리거나 낡은 페이지는 고치지 말고 알린다(고치는 것은 update · review · lint).

## 흐름 (하네스)

단계는 `yw-wiki flow`가 하나씩 안내하고, 검사를 통과해야 다음으로 넘긴다. 단계를 건너뛰거나 순서를 바꾸지 않는다.

```sh
yw-wiki flow start query
yw-wiki flow done --note "<이 단계에서 한 일 · 사용자 답>"   # 검사 실패면 고치고 다시
yw-wiki flow next                                        # 지금 단계 다시 보기
```

각 단계의 할 일과 검사는 `tools/flows/query.toml`에 있다. optional 단계만 `skip --note "이유"`로 넘길 수 있다.


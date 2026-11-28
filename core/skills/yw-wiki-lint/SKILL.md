---
name: yw-wiki-lint
description: "[{{wiki_name}}] 위키를 점검하고 고친다(lint). yw-wiki check로 frontmatter · 템플릿 · 링크 · 날짜 · raw 불변 · index를 보고, 기계적인 것은 바로 고치며, 모순 · 낡은 주장은 review/에 남긴다. \"{{wiki_name}} 점검해줘\", \"위키 lint\", \"위키 정리해줘\" 요청에 쓴다."
---

# yw-wiki-lint — 점검 ({{wiki_name}})

`yw-wiki check`로 보고, 기계적인 것은 묻지 않고 고치며, 판단할 것은 `review/`에 남긴다. `maturity` · `owner_checked`는 lint가 올리거나 내리지 않는다.

## 흐름 (하네스)

단계는 `yw-wiki flow`가 하나씩 안내하고, 검사를 통과해야 다음으로 넘긴다. 단계를 건너뛰거나 순서를 바꾸지 않는다.

```sh
yw-wiki flow start lint
yw-wiki flow done --note "<이 단계에서 한 일 · 사용자 답>"   # 검사 실패면 고치고 다시
yw-wiki flow next                                        # 지금 단계 다시 보기
```

각 단계의 할 일과 검사는 `tools/flows/lint.toml`에 있다. optional 단계만 `skip --note "이유"`로 넘길 수 있다.


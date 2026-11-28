---
name: yw-wiki-review
description: "[{{wiki_name}}] 사람 검토 대기열(review/)을 보여 주고, 사람이 정한 것만 위키에 반영한다(review). 업데이트 · lint가 위키에 쓰지 않고 남긴 충돌 · 낡은 주장 · 같은 이름 후보를 하나씩 정한다. \"{{wiki_name}} 리뷰해줘\", \"검토할 것 보여줘\", \"충돌 정리하자\" 요청에 쓴다."
---

# yw-wiki-review — 사람 검토 ({{wiki_name}})

위키(`wiki/`)에는 충돌 · 미정 내용을 넣지 않는다. 그런 항목은 `review/`에 파일 하나씩 쌓이고, 여기서 사람이 정한 것만 위키에 들어간다. 정하지 않은 것은 그대로 두고, 이미 정한 것은 다시 묻지 않는다.

## 흐름 (하네스)

단계는 `yw-wiki flow`가 하나씩 안내하고, 검사를 통과해야 다음으로 넘긴다. 단계를 건너뛰거나 순서를 바꾸지 않는다.

```sh
yw-wiki flow start review
yw-wiki flow done --note "<이 단계에서 한 일 · 사용자 답>"   # 검사 실패면 고치고 다시
yw-wiki flow next                                        # 지금 단계 다시 보기
```

각 단계의 할 일과 검사는 `tools/flows/review.toml`에 있다. optional 단계만 `skip --note "이유"`로 넘길 수 있다.


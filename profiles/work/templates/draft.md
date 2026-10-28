---
# ── 업무 위키 관리용 (대상 위키로 올릴 때 이 구간은 지운다) ──
draft_of: "{{title}}"
target_wiki:            # 승격 대상 id: wiki.toml [company.wikis.<id>] (예: company, team-dev)
target_path:            # 대상 위키 안 경로 (형식은 대상 위키 규칙. 예: 전사 위키는 concepts/... 영문 소문자 · 하이픈)
decision_id:            # 이 초안의 근거 결정 DEC-YYYY-NNN
review: draft           # draft | approved | merged
published:              # 반영된 커밋 · PR 링크
created: {{date}}
updated: {{date}}
# ── 대상 위키 frontmatter: 대상 위키의 문서 규칙대로 쓴다. 아래는 전사 위키(plco-llm-wiki SCHEMA.md 3절) 예 ──
type:                   # concept | entity | policy (target_path 최상위 폴더와 같게)
title:
description:            # index에 실리는 핵심 한 줄
owner:                  # 승인하는 GitHub 리더 팀 슬러그 (예: product-leads)
sources: []             # 공유 원천 URL만. 개인 공간 · 이 업무 위키는 근거로 쓰지 않는다
related: []             # 전사 위키 슬러그 (예: concepts/training-load)
answers: []
aliases: []
---

<!-- 대상 위키 본문. 형식 · 언어 · 링크 모양은 대상 위키 규칙대로 (전사 위키 예: 링크는 [[concepts/example]]).
     결론 문장 바로 옆에 원본 링크(코드는 커밋 고정 줄 링크).
     넣지 않는 것: 회의 요약, 미해결 질문, 확정되지 않은 설계, 업무 데이터 복사. -->

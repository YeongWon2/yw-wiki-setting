# 업무 위키 규칙 (profile: work)

한 회사의 내 업무 기록이다. 회사는 `wiki.toml` `[company]`에 하나만 둔다. 전사 위키에 올릴 초안도 여기서 만든다.

## 1. 폴더

| 이것은 | 여기에 | 템플릿 |
|---|---|---|
| 동료 · 이해관계자 | `wiki/entities/people/` | entity (person) |
| 회사 · 협업사 | `wiki/entities/orgs/` | entity (org) |
| 회사 프로젝트 | `wiki/entities/projects/` | entity (project) |
| 업무 지식 · 체크리스트 | `wiki/concepts/` | concept |
| 회의 · 수집 자료 요약 | `wiki/sources/` | source |
| 분석 · 비교 | `wiki/synthesis/` | synthesis |
| 의사결정 | `wiki/decisions/` (파일 이름 `DEC-YYYY-NNN 제목.md`) | decision |
| 전사 위키 승격 초안 | `wiki/drafts/` | draft |
| 회의 메모 원본 | `raw/meetings/YYYY-MM-DD 제목.md` | — |
| 업무 일지 | `raw/worklog/YYYY-MM-DD.md` | worklog |
| Jira · Slack · 노션 수집 요약 | `raw/collected/` | — |

## 2. 출처와 수집

- 회사 원본 도구(Jira · Slack · 노션 · GitHub)는 **읽기만** 한다. 댓글 · 수정 · 발송을 하지 않는다.
- 수집 범위는 `wiki.toml` `[company.sources]`와 `[company.projects."<프로젝트>"]`다. "수집해줘"는 회사 전체, "<프로젝트> 수집해줘"는 그 경로만 읽는다.
- 수집한 내용은 `raw/collected/`에 원본 URL · ID · 확인 날짜와 함께 필요한 부분만 텍스트로 남기고, 그다음 ingest로 source 요약을 쓴다.
- 이미 있는 source는 `checked` 이후 원본이 바뀌었는지 대조한다. 바뀐 것은 반영하고 log에 남긴다. 기존 사실과 다르면 충돌이다: 위키에 쓰지 않고 `review/`로 (core 2절).

## 3. 검토할 때 권위 순서

`review/` 항목을 보여 줄 때 근거를 이 순서로 놓고, 위쪽을 기본 제안으로 둔다. 정하는 것은 여전히 사람이다.

1. 전사 위키에 채택된 지식 (`[company.company_wiki]`)
2. 공유 원천: 병합된 PR · 커밋, Jira, Slack, 노션
3. 내 업무 일지 · 회의 메모 (`raw/`)
4. Claude의 추론 (`confidence: low`)

같은 단계끼리 다르면 더 최근 것을 위에 놓되 기본 제안은 두지 않는다.

## 4. 결정 (DEC-ID)

- 결정마다 `decision_id: DEC-YYYY-NNN`을 붙인다. 연도별로 1부터 늘리며 다시 쓰지 않는다.
- `canonical`에 정본 위치(repo / notion / confluence)를 적는다.

## 5. 전사 위키 승격

1. 채택할 만한 결정 · 지식을 `wiki/drafts/`에 draft 템플릿으로 쓴다. 형식은 전사 위키 `SCHEMA.md`를 따른다(근거는 공유 원천 URL만).
2. 사람이 검토해 `review: approved`로 바꾼다.
3. 전사 위키(`[company] company_wiki.path`)에서 브랜치 → PR. **PR 전에 묻는다.**
4. 병합되면 `review: merged`, 결정의 `published`에 PR · 커밋 링크를 적는다.

- 대조: 주 1회 DEC-ID 기준으로 전사 위키와 비교해 다른 것을 `review/`에 `review_kind: reconcile`로 남긴다. 업무 위키와 전사 위키 중 어느 쪽을 고칠지는 사람이 review로 정한다.
- 전사 위키 폴더에는 승격 PR 말고는 쓰지 않는다.

## 6. 경력 추출 (개인 위키로)

- 월 1회, 그리고 프로젝트가 `archived`가 될 때 성과를 추상화해 개인 위키(`[career] personal_wiki`)의 `wiki/career/`에 achievement로 쓴다(`confidence: medium`). 이 작업은 묻지 않고 한다.
- 쓰는 것: 회사 이름, 기간, 내가 한 일, 결과의 성격, 쓴 기술. 쓰지 않는 것: 고객명 · 협업사 내부 정보 · 내부 수치 · 이슈 키 · 내부 링크.
- 걸러지지 않는 항목은 개인 위키에 쓰지 않고 이 위키의 `review/`에 `review_kind: career-filter`로 남긴다.
- 그 밖의 회사 내용을 개인 위키로 복사하지 않는다.

# LLM 위키 공통 규칙

이 위키는 Karpathy의 LLM 위키 패턴을 따른다. 사람은 원본을 넣고 질문하며, LLM이 위키 페이지를 쓰고 유지한다. 왜 이렇게 하는지는 세팅의 `docs/철학.md`에 있다.

**사람이 읽는 글은 3초 읽기로 쓴다** (답 · 보고 · 검토 목록 · README). 첫 줄은 결론이나 요청, 항목은 앞 두 단어에 대상과 판단, 3초 층(첫 줄) → 30초 층(번호 목록 · 표) → 상세 층(근거 링크) 순서. 도구 용어 대신 결과로 말하고, 할 일이 없으면 없다고 한 줄로 쓴다. 위키 페이지는 AI가 읽는 글이라 이 규칙 대신 템플릿을 따른다.

## 1. 3층 구조

| 층 | 위치 | 누가 쓰나 |
|---|---|---|
| 원본 | `raw/` | 사람. LLM은 **읽기만** 한다. 고치거나 지우거나 옮기지 않는다 |
| 위키 | `wiki/` | LLM. 아래 템플릿으로만 쓴다 |
| 규칙 | `AGENTS.md` · `wiki.toml` · `_templates/` · `tools/` · `.claude/skills/` | 세팅 프로젝트(`yw-wiki-setting`). 위키에서 직접 고치지 않는다 (local 구간만 예외) |

```text
wiki/
  index.md        모든 페이지 목록. 질문 때 가장 먼저 읽는다 (tools/index.py가 만든다)
  log.md          작업 기록. 추가만 한다 (tools/log.py)
  entities/       사람(people) · 조직(orgs) · 프로젝트(projects). 종류는 wiki.toml [entities]
  concepts/       개념 · 방법 · 체크리스트
  sources/        원본 하나당 요약 하나
  synthesis/      여러 페이지를 묶은 분석 · 타임라인 · 비교
  decisions/      결정 기록
review/           사람 검토 대기열. 위키에 넣지 않은 충돌 · 미정 항목 하나 = 파일 하나 (review 템플릿)
```

- 보관 폴더는 없다. 끝난 것은 `status: archived`로 두고 자리를 옮기지 않는다.
- 파일 이름은 한국어 제목 그대로 쓴다. 링크는 `[[페이지 이름]]`이다. 같은 이름의 페이지를 두 개 만들지 않는다.

## 2. 페이지 쓰기

1. 템플릿(`_templates/`)으로만 만든다. 템플릿의 `##` 제목과 순서를 지키고, 내용이 없는 칸도 지우지 않는다.
2. 공통 frontmatter: `title` · `type` · `status`(active / superseded / archived) · `confidence`(high / medium / low) · `sources` · `related` · `aliases` · `tags` · `created` · `updated`. type별 추가 필드는 템플릿에 있다.
3. 본문 순서: 제목 → 1~3문장 요약 → (source: Summary / synthesis: Analysis) → Key facts → (entity: Timeline · Open tasks) → Connections → Sources.
4. `##` 제목은 영어로 고정하고 내용은 한국어로 쓴다.

**사실 · 결정 · 추정을 구분한다.**

- 사실: Key facts에 주장 하나당 한 줄, 끝에 근거(원본 경로 · URL · source 페이지). 근거 없는 출처를 만들지 않는다.
- 추정: Claude의 요약 · 추론은 `[low confidence]`를 붙이고, 페이지 전체가 추정이면 `confidence: low`.
- 결정: `decisions/`에 decision 템플릿으로. 사용자가 직접 말한 결정은 `decision_status: accepted`, Claude가 추론한 결정은 `proposed` + `confidence: low`로 쓰고 묻지 않는다.
- 결정이 바뀌면 지우지 않는다. 이전 결정을 `decision_status: superseded` · `status: superseded` · `superseded_by: "[[새 결정]]"`로 두고 새 결정에서 링크한다.
- 원문을 통째로 복사하지 않는다. 요약 + 짧은 인용("따옴표") + 링크 + 확인 날짜.
- Key facts가 12개를 넘으면 페이지를 나눈다.
- **충돌은 위키에 쓰지 않는다.** 새 원본이 기존 사실과 다르거나 원본끼리 다르면, 그 항목은 `wiki/`에 반영하지 않고 `review/`에 review 템플릿으로 남긴다(두 내용 · 각 근거 · 날짜 · 선택지). 한쪽을 임의로 고르지 않는다. 충돌이 아닌 부분은 그대로 반영한다.
- `review/` 항목은 사람이 `yw-wiki-review`로 정해야만 위키에 들어간다. 사람과 대화 중이면 작업 끝에 바로 리뷰를 권한다. query는 `review/`를 근거로 쓰지 않는다.
- 표와 mermaid는 내용을 담을 때만 쓴다. 장식(hero callout · 열 배치 · 이모지 제목 · 전용 CSS)은 쓰지 않는다.

## 3. 작업 5가지 (스킬)

| 작업 | 스킬 | 하는 일 |
|---|---|---|
| 동기화 | `yw-wiki-sync` | main으로 돌아와 원격 main · 세팅 최신으로 맞추고, 다른 브랜치(로컬 · 원격)를 지우고(합쳐지지 않은 것은 물은 뒤에만), 옮긴 위키 폴더에 Obsidian 볼트를 맞춘 뒤 반영 안 된 raw와 검토 대기를 알린다. 위키 내용은 쓰지 않는다 |
| 업데이트 | `yw-wiki-update` | 원본을 읽고 source 요약 → 관련 entity · concept · decision 갱신 → 이웃 페이지의 Connections · 요약 갱신 → index · log |
| 검토 | `yw-wiki-review` | `review/`의 항목을 사람에게 보여 주고 정한 것만 위키에 반영 |
| 찾기 | `yw-wiki-query` | index → 관련 entity · synthesis → 세부 페이지 순으로 읽고 근거 링크로 답한다. 다시 쓸 만한 답은 synthesis로 저장 |
| 점검 | `yw-wiki-lint` | `tools/check.py` 실행. 기계적인 것은 고치고, 모순 · 낡은 주장 · 고아 페이지는 `review/`로 |

모든 작업 뒤 `python3.14 tools/log.py <작업> <제목>`으로 log에 한 줄 남기고, 페이지가 늘거나 줄면 `python3.14 tools/index.py`로 index를 다시 만든다.

## 4. 사람이 개입하는 경우

기본은 자동이다. 아래 세 경우만 사람에게 넘긴다. 셋 다 사람이 정하기 전에는 실행하거나 쓰지 않는다.

| 경우 | 예 | 처리 |
|---|---|---|
| 충돌 | 새 원본이 기존 사실과 다름, 원본끼리 다름, 같은 이름 페이지 후보 둘 | `review/`에 남기고 사람이 `yw-wiki-review`로 정한 것만 반영한다 |
| 되돌릴 수 없음 | 페이지 삭제, 히스토리 재작성, force push, 메이저 세팅 업그레이드 | 실행 전에 묻는다 |
| 밖으로 나감 | 다른 위키에 쓰기, 원본 도구에 쓰기 | 실행 전에 묻는다 |

## 5. git

- **최신 확인:** 모든 스킬은 시작할 때 `python3.14 tools/version.py`를 돌린다. main 브랜치인지 · origin/main보다 뒤처지지 않았는지 · 세팅 최신 릴리스인지 · Obsidian 플러그인이 맞는지 보고, 받을 수 있으면 받는다. 세션을 시작할 때도 훅이 같은 확인을 한다. 경고(⚠)가 나오면 그것부터 사람에게 알린다.
- 갱신: 모든 스킬은 시작할 때 `git pull --rebase`로 원격 main을 받고, 끝나면 커밋하고 push한다. 브랜치와 PR 없이 main에서 작업한다 (구조 변경 같은 큰 작업만 브랜치).
- rebase 충돌이 나면 멈추고 충돌로 넘긴다. force push는 하지 않는다.
- 커밋 전에 `.githooks/pre-commit`이 `tools/check.py`를 돌린다. 오류가 있으면 커밋하지 않는다.

## 6. 하지 않는 것

- `raw/`를 고치거나 지우기.
- 비밀번호 · 토큰 · 주민번호 · 계좌 · 개인 연락처를 어디에든 적기.
- 템플릿 없이 페이지 만들기, 세팅 관리 파일(`_templates/` · `tools/` · `.claude/skills/` · AGENTS.md의 core · profile 구간)을 위키에서 직접 고치기. 바꿀 것은 세팅 프로젝트에 반영하고 `wiki-setting upgrade`로 내려받는다.

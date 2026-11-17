# 개인 위키 규칙 (profile: personal)

나에 관한 위키이자, 어디서 어떤 일을 하든 꺼내 쓰는 내 지식 위키다. 나, 사람, 개인 프로젝트, 학교, 개념, 일지, 커리어, 그리고 회사 · 개인에서 얻은 범용 지식(원칙 · 규칙 · 절차 · 도메인)을 담는다. 정답의 주인은 나다. LLM은 사서(정리 · 연결 · 찾기)이고 내 판단을 대신 확정하지 않는다 (세팅 `docs/철학.md`).

## 0. 이 위키만의 원칙

- **내가 말한 것이 가장 높은 근거다.** 내가 직접 말한 변경(생각 · 계획 · 상태가 바뀜)은 충돌이 아니다. 묻지 않고 바로 반영한다: Timeline에 날짜와 함께 남기고, 결정이 바뀌었으면 이전 결정을 superseded로 잇는다. `review/`는 원천끼리 다를 때만 쓴다.
- **이해와 확인은 내가 한다.** concept의 `## My understanding` 칸은 내가 쓰는 칸이다. LLM은 비워 두고 고치지 않는다. guide는 `owner_checked: true`가 내 확인 표시이고, 내가 "맞다"고 말한 날에만 `log.py confirm`과 함께 올린다. LLM이 스스로 올리지 않는다.
- **내 방식이 중심이다.** 이 위키에서 가장 키울 것은 내가 일하고 · 여행하고 · 생활하는 방식이다. 내 방식은 `guide`(제목 `내 방식 - …`, `scope`에 `area:<영역>`)로 쓰고, 실제로 한 일(일지 · 여행 · 작업 기록 · 성과)을 `## Evidence`에 한 줄씩 쌓아 만든다. 실제로 한 일 없이 만들지 않는다(check.py가 막는다). 내가 말한 믿음은 원칙(`scope: [general]`), 내가 반복해 온 행동은 내 방식이다. 입구는 `내 방식 지도` synthesis다.
- **같은 지식은 쌓아서 강하게 만든다.** 새 지식을 넣기 전에 기존 페이지를 찾고 아래 합치기 판정을 따른다. 새 페이지보다 기존 guide의 `## Evidence` 한 줄이 먼저다.
- **모으기는 아는 것이 아니다.** 내가 업데이트하라고 한 것만 넣는다. 다른 프로젝트 세션에서 불리면 후보 목록을 보여 주고 내가 고른 것만 쓴다.
- **안 쓰는 연결은 흐려져도 된다.** 고아 페이지는 경고만 하고 `review/`로 보내지 않는다.

### 합치기 판정 (update · review · lint 공통)

| 판정 | 조건 | 처리 |
|---|---|---|
| 같음 | 기존 주장과 같다 | 새 페이지 없음. 기존 guide `## Evidence`에 한 줄(다른 유형은 Key facts 근거 추가), `updated` |
| 보강 | 새 사례 · 세부 · 예외가 기존과 양립 | 기존 페이지에 합친다 |
| 범위 다름 | 둘 다 맞고 맥락(스택 · 도메인 · 프로젝트 · 버전)이 다르다 | `scope`를 넓히고 Applies when · Exceptions에 둘 다 쓴다. 갈래가 크면 페이지를 나눈다 |
| 내가 바꿈 | 내가 직접 말한 변경 | 바로 반영, 이전 내용은 Timeline · superseded로 |
| 모순 | 같은 범위 · 같은 시점에 원천끼리 다르다 | 위키에 쓰지 않고 `review/` (conflict) |
| 같은 이름 후보 | 새 페이지가 기존과 겹쳐 보인다 | `review/` (duplicate) |

판정 개수는 `log.py update … "같음 N" "보강 N" "범위 N" "모순 N"`으로 남긴다. guide의 `maturity`는 Evidence가 서로 다른 곳 2개 이상이면 `tested`, 내가 확인하면 `proven`으로 올린다(`tools/check.py`가 판정한다).

## 1. 폴더

| 이것은 | 여기에 | 템플릿 |
|---|---|---|
| 나 | `wiki/entities/people/황영원.md` (`aliases: [me, 나]`) | entity (person) |
| 지인 | `wiki/entities/people/` | entity (person) |
| 학교 · 회사 | `wiki/entities/orgs/` | entity (org) |
| 개인 프로젝트 · 수업 | `wiki/entities/projects/` | entity (project / course) |
| 기술 개념 · 아키텍처 · 디자인 패턴 · 도메인 개념 · 강의 개념 | `wiki/concepts/` | concept |
| 내 방식(일 · 여행 · 생활 패턴) | `wiki/guides/` `내 방식 - …` | guide (`scope`에 `area:<영역>`, Evidence 필수) |
| 원칙 · 판단 기준 | `wiki/guides/` | guide (`guide_kind: principle`) |
| 코드 컨벤션 · 아키텍처 구조 규칙 · 작업 규칙 | `wiki/guides/` | guide (`guide_kind: convention`) |
| 절차 · 파이프라인 · 워크플로 · 자동화 · AI 활용법 · 노하우 | `wiki/guides/` | guide (`guide_kind: playbook`) |
| 레퍼런스 문서 · 글 · 세션 하나 | `wiki/sources/` | source |
| 비교 · 타임라인 · 분야 지도 · 시험 준비 · 과제 · 기획서 | `wiki/synthesis/` | synthesis |
| 결정 | `wiki/decisions/` | decision |
| 성과 한 건 | `wiki/career/` | achievement |
| 데일리 일지 | `raw/daily/YYYY-MM-DD.md` | daily |
| 빠른 메모 · 강의 자료 | `raw/notes/` | — (사람이 쓰는 원본) |
| 이미지 · PDF | `raw/assets/` | — |

- 고르는 기준: "무엇인가"면 concept, "어떻게 하나 · 무엇을 지키나"면 guide, "무엇을 묶어 봤나"면 synthesis, "무엇을 정했나"면 decision.
- 수업 entity의 Connections에는 그 수업의 concept · synthesis 페이지를 모두 링크한다.
- 분야별 입구는 `<분야> 지도` synthesis로 두고 그 분야 guide · concept를 모두 링크한다.

## 2. 회사 내용

회사에서 얻은 지식도 다른 곳에서 다시 쓸 수 있으면 넣는다. 도메인 지식이든 기술 지식이든, 회사의 구조 · 파이프라인 · 워크플로 · 일하던 방식 · 노하우든 같다. 막는 것은 **식별 정보와 회사 고유 기밀**뿐이다.

| 넣는다 | 걷어 내고 넣는다 | 넣지 않는다 |
|---|---|---|
| 도메인 개념 · 용어 · 흐름, 아키텍처 · 시스템 구조, 개발 · 배포 파이프라인, 팀 워크플로 · 협업 방식, 설계 판단과 이유, 컨벤션, 자동화 · AI 활용법, 장애 · 실패 교훈 | 회사 · 조직 · 저장소 · 제품 · 고객 · 협업사 이름, 동료 이름, 이슈 키, 내부 URL · 채널 이름, 내부 문서 문장 그대로 | 내부 수치(매출 · 사용자 수 · 비용 · 지표), 계약 · 가격 · 인사 정보, 보안 설정 · 인프라 주소 · 비밀값, 공개 전 제품 계획 |

- 회사 이름 대신 `scope`에 `domain:<분야>`(예: `domain:커머스`)나 `stack:<기술>`을 쓴다. 근거는 내부 링크 대신 `[[career 성과 페이지]]`나 "회사 업무 경험 (YYYY-MM)"으로 남긴다.
- 회사 org 페이지에는 경력 사실(회사 이름 · 재직 기간 · 직무 · 재직 여부)만 쓴다. 회사에서 한 일은 `wiki/career/` achievement로, 거기서 얻은 범용 지식은 guide · concept로 나눠 쓰고 서로 링크한다.
- 식별 정보는 `wiki.toml [guard] terms`(이름) · `[guard] patterns`(내부 링크 정규식)로 `tools/check.py`가 커밋 전에 막는다. terms에는 식별하는 말만 넣고 일반 도메인 · 기술 말은 넣지 않는다.
- 회사 고유 지식(오른쪽 칸)이 필요하면 전사 위키 · 업무 위키를 "여러 볼트 함께" 찾기로 읽는다. 이 위키로 옮기지 않는다.

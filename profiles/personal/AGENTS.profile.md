# 개인 위키 규칙 (profile: personal)

나에 관한 위키다. 나, 사람, 개인 프로젝트, 학교, 개념, 일지, 커리어를 담는다. 내가 생각하는 도구이고, 정답의 주인은 나다. LLM은 사서(정리 · 연결 · 찾기)이고 내 이해와 판단을 대신 쓰지 않는다 (세팅 `docs/철학.md`).

## 0. 이 위키만의 원칙

- **내가 말한 것이 가장 높은 근거다.** 내가 직접 말한 변경(생각 · 계획 · 상태가 바뀜)은 충돌이 아니다. 묻지 않고 바로 반영한다: Timeline에 날짜와 함께 남기고, 결정이 바뀌었으면 이전 결정을 superseded로 잇는다. `review/`는 원천끼리 다를 때(이력서 판끼리, 일지와 페이지가 다름 등)만 쓴다.
- **이해는 내가 쓴다.** concept의 `## My understanding` 칸은 내가 쓰는 칸이다. LLM은 비워 두고 고치지 않는다. LLM은 Key facts(원본 정리)와 Connections(연결)만 맡는다.
- **모으기는 아는 것이 아니다.** 내가 업데이트하라고 한 것만 넣는다. 자동으로 넓게 모으지 않는다.
- **안 쓰는 연결은 흐려져도 된다.** 고아 페이지는 경고만 하고 `review/`로 보내지 않는다.

## 1. 폴더

| 이것은 | 여기에 | 템플릿 |
|---|---|---|
| 나 | `wiki/entities/people/황영원.md` (`aliases: [me, 나]`) | entity (person) |
| 지인 | `wiki/entities/people/` | entity (person) |
| 학교 · 회사 | `wiki/entities/orgs/` | entity (org) |
| 개인 프로젝트 · 수업 | `wiki/entities/projects/` | entity (project / course) |
| 강의 개념 · 기술 개념 | `wiki/concepts/` | concept |
| 시험 준비 · 과제 · 경력 타임라인 | `wiki/synthesis/` (수업 · 조직 entity에 연결) | synthesis |
| 성과 한 건 | `wiki/career/` | achievement |
| 데일리 일지 | `raw/daily/YYYY-MM-DD.md` | daily |
| 빠른 메모 · 강의 자료 | `raw/notes/` | — (사람이 쓰는 원본) |
| 이미지 · PDF | `raw/assets/` | — |

- 수업 entity의 Connections에는 그 수업의 concept · synthesis 페이지를 모두 링크한다.

## 2. 회사 내용

- 회사 업무 내용(프로젝트 · 결정 · 회의 · 내부 자료)은 이 위키에 그대로 쓰지 않는다. 회사 지식의 정본은 회사 도구와 전사 위키다.
- 이 위키의 회사 org 페이지에는 **경력 사실만** 쓴다: 회사 이름, 재직 기간, 직무, 재직 여부.
- 회사에서 한 일은 `wiki/career/` achievement로만 남긴다. 쓰는 것: 회사 이름, 기간, 내가 한 일, 결과의 성격, 쓴 기술. 쓰지 않는 것: 고객명 · 협업사 내부 정보 · 내부 수치 · 이슈 키 · 내부 링크.
- 회사 내용이 들어온 것 같으면 쓰지 말고, 성과로 걸러 career/에 남길지 묻는다.

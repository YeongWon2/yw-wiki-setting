# yw-wiki-setting

**LLM 위키 두 개(개인 · 개인 업무)를 만들고 같은 규칙으로 맞추는 세팅이다. 위키 내용은 없다.**

> 원격 저장소(`YeongWon2/yw-wiki-setting`)를 지우지 않는다. 위키들이 여기 릴리스 태그를 받아 맞춘다.

## 쓰기

| 하려는 것 | 명령 |
|---|---|
| 새 위키 만들기 | `python3.14 scripts/new_wiki.py --profile personal --dest <경로>` (업무는 `--profile work`) |
| 위키에 새 버전 반영 | `python3.14 scripts/upgrade.py <위키 경로>` (메이저는 `--yes`) |
| 세팅 바꾼 뒤 점검 | `sh check.sh` (통과한 커밋에만 `vX.Y.Z` 태그) |

## 위키 안에서 말하기 (스킬 5개)

| 말 | 스킬 | 하는 일 |
|---|---|---|
| 갱신해줘 | `yw-wiki-sync` | 최신 받기 · 세팅 반영 · 할 일 알리기 |
| 업데이트해줘 · 수집해줘 | `yw-wiki-update` | 원본을 위키에 반영. 충돌은 `review/`로 |
| 리뷰해줘 | `yw-wiki-review` | 검토 대기를 보여 주고 정한 것만 반영 |
| 찾아줘 | `yw-wiki-query` | 목차부터 읽고 근거 링크로 답함 |
| 점검해줘 | `yw-wiki-lint` | 자동 검사, 판단할 것은 `review/`로 |

## 더 보기

- 왜 이렇게 만들었나: [docs/철학.md](docs/철학.md)
- 공통 규칙: `core/AGENTS.core.md` · 개인: `profiles/personal/AGENTS.profile.md` · 업무: `profiles/work/AGENTS.profile.md`
- 템플릿: core 6개(entity · concept · source · synthesis · decision · review), 개인 2개(achievement · daily), 업무 2개(worklog · draft)
- 버전: `VERSION`

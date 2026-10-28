# yw-wiki-setting

> **이 저장소의 원격(`YeongWon2/yw-wiki-setting`)을 지우지 않는다.** 위키들이 여기 릴리스 태그를 받아 규칙 · 템플릿 · 도구를 맞춘다.

LLM 위키를 만들고, 만든 위키들의 규칙 · 템플릿 · 도구 · 스킬을 같은 버전으로 유지하는 세팅 프로젝트다. 위키 내용(raw · wiki)과 회사 값은 갖지 않는다.

| 폴더 | 내용 |
|---|---|
| `core/` | 모든 위키 공통: `AGENTS.core.md`, 템플릿 6개(entity · concept · source · synthesis · decision · review), `tools/`, `skills/`, `obsidian/` |
| `profiles/personal/` | 개인 위키: 규칙, 템플릿 2개(achievement · daily), `wiki.toml` 기본값 |
| `profiles/work/` | 업무 위키: 규칙, 템플릿 2개(worklog · draft), `wiki.toml` 기본값 |
| `scripts/` | `new_wiki.py`(생성) · `upgrade.py`(갱신) · `migrate_para.py`(1회용) |

버전은 `VERSION`. 릴리스 태그 `vX.Y.Z`는 `check.sh`를 통과한 커밋에만 단다.

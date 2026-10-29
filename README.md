# yw-wiki-setting

**LLM 위키 두 개(개인 · 개인 업무)를 만들고 같은 규칙으로 맞추는 세팅이다. 위키 내용은 없다.**

![version](https://img.shields.io/github/v/tag/YeongWon2/yw-wiki-setting?label=version&sort=semver)
![python](https://img.shields.io/badge/python-3.14-3776AB?logo=python&logoColor=white)
![license](https://img.shields.io/badge/license-MIT-blue)

> 원격 저장소(`YeongWon2/yw-wiki-setting`)를 지우지 않는다. 위키들이 여기 릴리스 태그를 받아 맞춘다.

## 3초 요약

1. 위키를 만들 때 이 세팅으로 폴더 구조 · 템플릿 · 규칙(`AGENTS.md`)을 깐다.
2. 위키 안에서는 "업데이트해줘", "찾아줘"처럼 말하면 스킬이 처리한다.
3. 세팅이 바뀌면 `wiki-setting check`가 모든 위키를 최신 릴리스로 맞춘다.

## 새 PC에서 시작하기

```sh
brew install python@3.14
git clone https://github.com/YeongWon2/yw-wiki-setting ~/.local/share/wiki/yw-wiki-setting
python3.14 ~/.local/share/wiki/yw-wiki-setting/global/wiki-vault/bin/wiki-setting install
wiki-setting register <위키 폴더>        # 받아 둔 위키마다
```

새 위키는 `wiki-setting new-vault --profile personal|work --path <폴더>`. 그다음부터 `wiki-setting check`(길잡이 · 세션 시작 · 모든 스킬이 먼저 돌림)가 세팅 · 전역 스킬 · 위키를 main의 최신 릴리스로 맞춘다.

## 쓰기

| 하려는 것 | 명령 |
|---|---|
| 새 위키 만들기 | `python3.14 scripts/new_wiki.py --profile personal --dest <경로>` (업무는 `--profile work`) |
| 위키에 새 버전 반영 | `python3.14 scripts/upgrade.py <위키 경로>` (메이저는 `--yes`) |
| 세팅 바꾼 뒤 점검 | `sh check.sh` (단위 시험) → `sh e2e.sh <개인 위키> <업무 위키>` (실제 위키 복사본으로 사용 시나리오) → `python3.14 scripts/audit_links.py <위키들> . --online` (링크 · 낡은 표현). 모두 통과한 커밋에만 `vX.Y.Z` 태그 |
| 이 PC에 길잡이 스킬 깔기 | `python3.14 global/wiki-vault/bin/wiki-setting install` → `register <위키 폴더>` (볼트 목록 `~/.config/wiki/vaults.toml`) |

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
- 템플릿: core 6개(entity · concept · source · synthesis · decision · review), 개인 3개(achievement · daily · concept에 My understanding 칸을 더한 판), 업무 2개(worklog · draft)
- 버전: `VERSION`

## 기여

버그 · 제안 · 질문은 [이슈](https://github.com/YeongWon2/yw-wiki-setting/issues/new/choose)로 남긴다. 코드는 PR로만 받고, 관리자가 승인해야 main에 들어간다. 라벨과 순서는 [CONTRIBUTING.md](CONTRIBUTING.md)에 있다.

## 라이선스

[MIT](LICENSE)

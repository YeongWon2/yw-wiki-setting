# yw-wiki-setting

**개인 LLM 위키를 만들고 같은 규칙으로 맞추는 세팅이다. 위키 내용은 없다.**

![version](https://img.shields.io/github/v/tag/YeongWon2/yw-wiki-setting?label=version&sort=semver)
![rust](https://img.shields.io/badge/cli-rust-000000?logo=rust&logoColor=white)
![license](https://img.shields.io/badge/license-MIT-blue)

> 원격 저장소(`YeongWon2/yw-wiki-setting`)를 지우지 않는다. 위키들이 여기 릴리스 태그를 받아 맞춘다.

## 3초 요약

1. 위키를 만들 때 이 세팅으로 폴더 구조 · 템플릿 · 규칙(`AGENTS.md`)을 깐다.
2. 위키 안에서는 "업데이트해줘", "찾아줘"처럼 말하면 스킬이 처리한다.
3. 세팅이 바뀌면 `wiki-setting check`가 모든 위키를 최신 릴리스로 맞춘다.

## 새 PC에서 시작하기

```sh
brew install rust
git clone https://github.com/YeongWon2/yw-wiki-setting ~/.local/share/wiki/yw-wiki-setting
sh ~/.local/share/wiki/yw-wiki-setting/install.sh   # yw-wiki를 빌드해 ~/.local/bin에, 길잡이 스킬 · 볼트 목록을 깐다
yw-wiki setting register <위키 폴더>     # 받아 둔 위키마다
```

새 위키는 `yw-wiki setting new-vault --profile personal --path <폴더>`. 그다음부터 `yw-wiki setting check`(길잡이 · 세션 시작 · 모든 스킬이 먼저 돌림)가 세팅 · 실행 파일 · 전역 스킬 · 위키를 main의 최신 릴리스로 맞춘다. 실행 파일이 세팅보다 낡았으면 세팅 사본의 `cli/`로 다시 빌드한다. `wiki-setting`은 `yw-wiki setting`의 예전 이름으로 그대로 쓸 수 있다.

## Python판에서 올리기 (v3, 메이저)

도구가 Python 스크립트에서 Rust CLI(`yw-wiki`)로 바뀌었다. PC마다 한 번:

1. `brew install rust` → `sh ~/.local/share/wiki/yw-wiki-setting/install.sh` (세팅 사본이 새 태그에 있어야 한다)
2. `yw-wiki setting upgrade --yes` — 위키의 `tools/*.py`를 지우고 흐름 · 훅 · 스킬을 `yw-wiki` 명령으로 바꾼다
3. 위키에서 `yw-wiki check` 오류 0을 보고 커밋 · push

옛 길잡이(`bin/wiki-setting`)가 python으로 불려도 `yw-wiki setting`으로 넘어가므로, 1번 전에는 "yw-wiki가 없다"고만 알린다.

## 쓰기

| 하려는 것 | 명령 |
|---|---|
| 새 위키 만들기 | `yw-wiki new --profile personal --dest <경로>` |
| 위키에 새 버전 반영 | `yw-wiki upgrade <위키 경로> --setting .` (메이저는 `--yes`) |
| 세팅 바꾼 뒤 점검 | `sh check.sh` (단위 시험) → `sh e2e.sh <개인 위키>` (실제 위키 복사본으로 사용 시나리오) → `yw-wiki audit-links <위키들> . --online` (링크 · 낡은 표현). 모두 통과한 커밋에만 `vX.Y.Z` 태그 |
| 이 PC에 CLI · 길잡이 깔기 | `sh install.sh` → `yw-wiki setting register <위키 폴더>` (볼트 목록 `~/.config/wiki/vaults.toml`) |
| CLI 고치기 | `cli/src/`(Rust). 명령 목록은 `yw-wiki --help`, 각 명령은 `yw-wiki <명령> --help` |

## 위키 안에서 말하기 (스킬 5개)

| 말 | 스킬 | 하는 일 |
|---|---|---|
| 동기화해줘 | `yw-wiki-sync` | main으로 맞추기 · 세팅 반영 · 다른 브랜치 지우기(합쳐지지 않은 것은 묻고) · 옮긴 폴더에 Obsidian 볼트 맞추기 · 할 일 알리기 |
| 업데이트해줘 | `yw-wiki-update` | 원본을 위키에 반영. 충돌은 `review/`로 |
| 리뷰해줘 | `yw-wiki-review` | 검토 대기를 보여 주고 정한 것만 반영 |
| 찾아줘 | `yw-wiki-query` | 목차부터 읽고 근거 링크로 답함 |
| 점검해줘 | `yw-wiki-lint` | 자동 검사, 판단할 것은 `review/`로 |

## 더 보기

- 왜 이렇게 만들었나: [docs/철학.md](docs/철학.md)
- 공통 규칙: `core/AGENTS.core.md` · 개인: `profiles/personal/AGENTS.profile.md`
- 템플릿: core 6개(entity · concept · source · synthesis · decision · review), 개인 4개(achievement · daily · guide(원칙 · 규칙 · 절차, `wiki/guides/`) · concept에 My understanding 칸을 더한 판), 개인 시드 페이지(`profiles/personal/seed/`: 새 위키에 `내 방식 지도`를 한 번 깐다. 이미 있으면 그대로)
- 버전: `VERSION`

## 기여

버그 · 제안 · 질문은 [이슈](https://github.com/YeongWon2/yw-wiki-setting/issues/new/choose)로 남긴다. 코드는 PR로만 받고, 관리자가 승인해야 main에 들어간다. 라벨과 순서는 [CONTRIBUTING.md](CONTRIBUTING.md)에 있다.

## 라이선스

[MIT](LICENSE)

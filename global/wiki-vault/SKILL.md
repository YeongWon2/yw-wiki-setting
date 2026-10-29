---
name: wiki-vault
description: 내 LLM 위키 길잡이. 이 PC의 볼트 목록(~/.config/wiki/vaults.toml)에서 맞는 위키(개인 위키 · 읽기 전용으로 등록한 전사 위키)를 골라 그 위키의 AGENTS.md와 yw-wiki-* 스킬로 넘긴다. 위키를 직접 쓰지 않는다. "개인 위키에서 찾아줘", "내 위키 업데이트해줘", "내 위키 갱신해줘", "위키 리뷰해줘", "위키 점검해줘", "어떤 위키 있어?", "위키 등록해줘" 같은 요청에 쓴다.
---

# wiki-vault — 위키 길잡이

고르고 넘기기만 한다. 위키 내용은 고른 위키의 규칙과 스킬이 쓴다.

## 0. 최신 확인 (항상 먼저)

`bin/wiki-setting check`를 돌린다. 세팅 사본을 원격 최신 릴리스 태그(main에 있는 태그)로 맞추고, 이 길잡이 스킬이 낡았으면 다시 깔고, 각 볼트가 main 기준 · 최신 세팅인지 본다. ⚠가 나오면 그것부터 알린다. 원격에 접속할 수 없으면 가진 것으로 계속한다.

## 1. 볼트 고르기

1. `bin/wiki-setting list`로 등록된 볼트(id · 경로 · 이름 · 별칭 · 설명)를 본다. 이름 · 별칭 · 설명은 각 위키의 `wiki.toml` `[wiki]`에서 읽는다.
2. 고르는 순서: 사용자가 말한 이름 · 별칭("개인 위키", "내 위키") → 지금 폴더가 볼트 안이면 그 볼트 → 요청 내용과 `description`이 맞는 볼트.
3. **찾기(query)는 여러 볼트를 함께 볼 수 있다.** 답마다 어느 볼트인지 밝힌다. **쓰기는 한 볼트에만** 한다. 애매하면 묻는다.
4. `readonly = true`인 볼트(팀 · 전사 위키)에는 yw-wiki 스킬로 직접 쓰지 않는다. 그 위키에 쓸 일은 **그 위키 자신의 규칙과 스킬**로만 한다.

## 2. 경로 찾기

등록된 경로에 볼트가 없으면 `bin/wiki-setting doctor`가 `~/Desktop/wiki`, `~/vaults`, `~/Desktop`, `~/Documents` 아래에서 같은 `id`의 `wiki.toml`을 찾는다. 하나면 목록을 고치고 알린다. 둘 이상이면 사용자에게 묻는다.

## 3. 넘기기

고른 볼트 루트로 가서 그 볼트의 `AGENTS.md`를 읽고, 요청에 맞는 스킬을 따른다.

| 말 | 스킬 |
|---|---|
| 갱신해줘 · sync | `yw-wiki-sync` |
| 업데이트해줘 · 넣어줘 | `yw-wiki-update` |
| 리뷰해줘 · 검토할 것 | `yw-wiki-review` |
| 찾아줘 · 어디까지 했지 | `yw-wiki-query` |
| 점검해줘 · lint | `yw-wiki-lint` |

## 4. 볼트 관리 (실행기 `bin/wiki-setting`)

| 하려는 것 | 명령 |
|---|---|
| 목록 · 경로 | `wiki-setting list` · `wiki-setting path <id>` |
| 점검 | `wiki-setting doctor` (전역 스킬 · 목록 · 경로 · 세팅 버전 · 각 볼트 점검) |
| 등록 | `wiki-setting register <폴더>` (전사 위키는 `--id company --readonly`) |
| 새 볼트 | `wiki-setting new-vault --profile personal --path <폴더>` |
| 세팅 반영 | `wiki-setting upgrade [<id>]` (패치 · 마이너는 바로, 메이저는 확인) |

목록이나 볼트 파일을 바꾸는 명령(register · new-vault · upgrade)은 무엇이 바뀌는지 먼저 보여 주고 확인받는다.

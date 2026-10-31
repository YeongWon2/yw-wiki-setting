#!/bin/sh
# 세팅 점검: 임시 HOME에서 개인 위키를 새로 만들고 점검 · 업그레이드가 도는지 본다. 릴리스 태그는 이게 통과한 커밋에만 단다.
set -eu
SETTING=$(cd "$(dirname "$0")" && pwd)
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT
export HOME="$TMP/home" YW_TODAY=2026-10-28 YW_NO_NETWORK=1
mkdir -p "$HOME"
git config --global user.name check >/dev/null 2>&1 || true
git config --global user.email check@example.invalid >/dev/null 2>&1 || true
PY=$(command -v python3.15 || command -v python3.14 || command -v python3)
fail() { echo "실패: $1"; exit 1; }

for p in personal; do
  W="$TMP/$p"
  "$PY" "$SETTING/scripts/new_wiki.py" --profile "$p" --dest "$W" >/dev/null || fail "$p 생성 직후 check.py 오류"
  "$PY" "$W/tools/index.py" "$W" --check >/dev/null || fail "$p index 최신 아님"
  [ "$(cat "$W/CLAUDE.md")" = "@AGENTS.md" ] || fail "$p CLAUDE.md"
  grep -q "profile:start $p" "$W/AGENTS.md" || fail "$p AGENTS.md profile 구간"
  grep -rq '{{wiki_name}}' "$W/.claude/skills" && fail "$p 스킬 이름 치환 안 됨"
  [ "$(git -C "$W" config core.hooksPath)" = ".githooks" ] || fail "$p 훅 연결"
  [ -f "$W/.claude/settings.json" ] && grep -q "version.py" "$W/.claude/settings.json" || fail "$p 세션 시작 최신 확인 훅 없음"
  grep -q '"obsidian-git"' "$W/.obsidian/community-plugins.json" || fail "$p 플러그인 목록"
  for f in .obsidian/graph.json .obsidian/workspace.json .obsidian/types.json .obsidian/plugins/obsidian-git/main.js .yw-wiki/version-check; do
    git -C "$W" check-ignore -q "$f" || fail "$p gitignore가 $f를 무시하지 않음"
  done
  git -C "$W" check-ignore -q .obsidian/plugins/obsidian-git/data.json && fail "$p 플러그인 설정 data.json까지 무시함"
  [ -f "$W/.obsidian/graph.json" ] && [ -f "$W/.obsidian/plugins/obsidian-git/data.json" ] || fail "$p Obsidian 기본값 안 깔림"
  "$PY" "$W/tools/version.py" "$W" --hook | grep -q "main" || fail "$p version.py (main 확인)"

  # 템플릿으로 만든 페이지 하나와 그걸 가리키는 페이지 하나가 통과해야 한다
  sed -e 's/{{title}}/시험 페이지/g' -e 's/{{date}}/2026-10-28/g' -e 's/^entity_kind:.*/entity_kind: org/' \
    "$W/_templates/entity.md" > "$W/wiki/entities/orgs/시험 페이지.md"
  sed -e 's/{{title}}/시험 개념/g' -e 's/{{date}}/2026-10-28/g' -e 's/^related: \[\]/related: ["[[시험 페이지]]"]/' \
    "$W/_templates/concept.md" > "$W/wiki/concepts/시험 개념.md"
  printf '%s\n' '- [[시험 개념]] — 시험' >> "$W/wiki/entities/orgs/시험 페이지.md"
  "$PY" "$W/tools/index.py" "$W" >/dev/null
  "$PY" "$W/tools/check.py" "$W" >/dev/null || { "$PY" "$W/tools/check.py" "$W"; fail "$p 템플릿 페이지가 점검을 통과하지 못함"; }

  # 없는 파일 · 로컬 절대 경로 마크다운 링크는 오류
  printf '\n- [없음](../../raw/없는%%20파일.md)\n' >> "$W/wiki/concepts/시험 개념.md"
  "$PY" "$W/tools/check.py" "$W" | grep -q "없는 파일 링크" || fail "$p 없는 파일 링크를 못 잡음"
  sed -i.bak '/없는%20파일.md/d' "$W/wiki/concepts/시험 개념.md"; printf '\n- [로컬](/Users/someone/a.md)\n' >> "$W/wiki/concepts/시험 개념.md"
  "$PY" "$W/tools/check.py" "$W" | grep -q "로컬 절대 경로" || fail "$p 로컬 절대 경로 링크를 못 잡음"
  sed -i.bak '/someone/d' "$W/wiki/concepts/시험 개념.md"; rm -f "$W/wiki/concepts/시험 개념.md.bak"

  # 잘못된 페이지는 오류여야 한다
  printf -- '---\ntitle: 틀림\ntype: entity\n---\n\n# 틀림\n\n[[없는 문서]]\n' > "$W/wiki/concepts/틀림.md"
  "$PY" "$W/tools/index.py" "$W" >/dev/null
  "$PY" "$W/tools/check.py" "$W" >/dev/null && fail "$p 잘못된 페이지를 못 잡음"
  rm "$W/wiki/concepts/틀림.md"; "$PY" "$W/tools/index.py" "$W" >/dev/null

  # 충돌 칸은 위키에 두면 오류, review/ 항목은 통과(경고)
  printf '\n## Contradictions\n' >> "$W/wiki/concepts/시험 개념.md"
  "$PY" "$W/tools/check.py" "$W" >/dev/null && fail "$p 위키 안의 충돌 칸을 못 잡음"
  sed -i.bak '/^## Contradictions$/d' "$W/wiki/concepts/시험 개념.md" && rm "$W/wiki/concepts/시험 개념.md.bak"
  sed -e 's/{{title}}/시험 검토/g' -e 's/{{date}}/2026-10-28/g' -e 's/^review_kind:.*/review_kind: conflict/' \
    "$W/_templates/review.md" > "$W/review/2026-10-28 시험 검토.md"
  "$PY" "$W/tools/check.py" "$W" > "$TMP/out" || { cat "$TMP/out"; fail "$p review 항목이 점검을 통과하지 못함"; }
  grep -q "검토 대기 1개" "$TMP/out" || fail "$p review 대기 경고 없음"
  [ "$(ls "$W/.claude/skills" | wc -l | tr -d ' ')" = 5 ] && [ -f "$W/.claude/skills/yw-wiki-sync/SKILL.md" ] || fail "$p 스킬 5개 아님"

  # 업그레이드: 바뀐 게 없으면 최신, 관리 파일을 망가뜨리면 되돌리고 local 구간은 남긴다
  "$PY" "$SETTING/scripts/upgrade.py" "$W" | grep -q "최신" || fail "$p upgrade 멱등"
  echo "# 망가뜨림" >> "$W/tools/check.py"
  sed -i.bak 's/(없음\. 이 구간은 upgrade가 건드리지 않는다\.)/로컬 규칙 하나/' "$W/AGENTS.md" && rm "$W/AGENTS.md.bak"
  "$PY" "$SETTING/scripts/upgrade.py" "$W" >/dev/null
  grep -q "# 망가뜨림" "$W/tools/check.py" && fail "$p upgrade가 관리 파일을 되돌리지 않음"
  grep -q "로컬 규칙 하나" "$W/AGENTS.md" || fail "$p upgrade가 local 구간을 지움"
  grep -q "upgrade |" "$W/wiki/log.md" || fail "$p upgrade 기록 없음"
  "$PY" "$W/tools/check.py" "$W" >/dev/null || fail "$p upgrade 뒤 점검 오류"

  # 옛 .gitignore에 규칙이 빠지고 무시 대상이 git에 올라가 있어도 upgrade가 맞춘다
  printf '.DS_Store\n' > "$W/.gitignore"; git -C "$W" add -f .gitignore .obsidian/graph.json >/dev/null 2>&1
  "$PY" "$SETTING/scripts/upgrade.py" "$W" >/dev/null
  git -C "$W" check-ignore -q .obsidian/graph.json || fail "$p upgrade가 .gitignore 규칙을 더하지 않음"
  git -C "$W" ls-files --error-unmatch .obsidian/graph.json >/dev/null 2>&1 && fail "$p upgrade가 무시 대상을 추적에서 빼지 않음"
  [ -f "$W/.obsidian/graph.json" ] || fail "$p upgrade가 graph.json 파일까지 지움"

  # raw 삭제는 커밋 점검에서 오류
  echo "원본" > "$W/raw/assets/a.md"
  git -C "$W" add -A >/dev/null && git -C "$W" -c core.hooksPath=/dev/null commit -qm init
  git -C "$W" rm -q "$W/raw/assets/a.md"
  (cd "$W" && "$PY" tools/check.py >/dev/null) && fail "$p raw 삭제를 못 잡음"
  echo "통과: $p"
done
# 전역: 길잡이 스킬 설치 · 볼트 등록 · doctor (임시 HOME)
WS="$SETTING/global/wiki-vault/bin/wiki-setting"
"$PY" "$WS" install >/dev/null || fail "install"
[ -f "$HOME/.claude/skills/wiki-vault/SKILL.md" ] && [ -f "$HOME/.agents/skills/wiki-vault/bin/wiki-setting" ] || fail "전역 스킬 복사 안 됨"
"$PY" "$WS" register "$TMP/personal" >/dev/null || fail "register"
"$PY" "$WS" list | grep -q "^personal.*: " || fail "list"
[ "$("$PY" "$WS" path personal)" = "$(cd "$TMP/personal" && pwd -P)" ] || fail "path"
# 다음 셋은 앞의 raw 삭제 시험이 남긴 상태라 점검이 실패하므로 되돌린 뒤 본다
git -C "$TMP/personal" reset -q --hard
"$PY" "$WS" doctor > "$TMP/doctor" || { cat "$TMP/doctor"; fail "doctor"; }
mv "$TMP/personal" "$TMP/moved-personal"
"$PY" "$WS" doctor >/dev/null && fail "옮긴 볼트를 못 잡음"
mv "$TMP/moved-personal" "$TMP/personal"
# Obsidian: 위키 폴더를 옮기면 doctor --fix가 볼트 목록 · Obsidian 볼트를 따라 옮기고, 없어진 폴더 설정은 세팅 값으로
case "$(uname)" in Darwin) OJ="$HOME/Library/Application Support/obsidian/obsidian.json" ;; *) OJ="$HOME/.config/obsidian/obsidian.json" ;; esac
mkdir -p "$(dirname "$OJ")"
P0=$(cd "$TMP/personal" && pwd -P)
printf '{"vaults":{"aaaa":{"path":"%s","ts":1},"bbbb":{"path":"%s/gone","ts":1}}}' "$P0" "$TMP" > "$OJ"
"$PY" "$WS" obsidian | grep -q "Obsidian 맞음" || fail "obsidian 맞는 상태를 못 봄"
grep -q 'obsidian_id = "aaaa"' "$HOME/.config/wiki/vaults.toml" || fail "obsidian_id 기록 안 함"
P1="$HOME/Desktop/wiki/renamed-personal"  # doctor가 찾는 곳으로 옮기며 이름도 바꾼다
mkdir -p "$(dirname "$P1")" && mv "$TMP/personal" "$P1"
YW_OBSIDIAN_RUNNING=1 "$PY" "$WS" doctor --fix >/dev/null 2>&1 || true  # 켜져 있으면 ⚠라 1
grep -q "$P0" "$OJ" || fail "Obsidian이 켜져 있는데 obsidian.json을 고침"
YW_OBSIDIAN_RUNNING=0 "$PY" "$WS" doctor --fix > "$TMP/doctor" 2>&1 || true  # 결과는 obsidian.json으로 본다
grep -q "renamed-personal" "$OJ" || { cat "$TMP/doctor"; fail "옮긴 위키를 Obsidian 볼트가 따라가지 않음"; }
sed -i.bak 's#"raw/assets"#"첨부"#' "$P1/.obsidian/app.json" && rm "$P1/.obsidian/app.json.bak"
YW_OBSIDIAN_RUNNING=0 "$PY" "$WS" obsidian --fix | grep -q "attachmentFolderPath 첨부 → raw/assets" || fail "없는 폴더 설정을 세팅 값으로 안 고침"
grep -q '"raw/assets"' "$P1/.obsidian/app.json" || fail "app.json 고침이 안 남음"
YW_OBSIDIAN_RUNNING=1 "$PY" "$WS" obsidian --fix --prune >/dev/null && fail "Obsidian이 켜져 있는데 지웠다고 함"
grep -q '"bbbb"' "$OJ" || fail "Obsidian이 켜져 있는데 obsidian.json을 고침"
YW_OBSIDIAN_RUNNING=1 "$PY" "$WS" obsidian --fix --prune --restart > "$TMP/obs" || { cat "$TMP/obs"; fail "--restart 실패"; }
grep -q "껐다가 다시 켬" "$TMP/obs" || fail "--restart가 Obsidian을 끄고 다시 켜지 않음"
grep -q '"bbbb"' "$OJ" && fail "--prune이 폴더 없는 볼트를 안 지움"
mv "$P1" "$TMP/personal" && "$PY" "$WS" register "$TMP/personal" >/dev/null
echo "0.0.1" > "$HOME/.claude/skills/wiki-vault/VERSION"
"$PY" "$WS" check >/dev/null 2>&1 || true  # 오프라인 시험이라 ⚠(1)가 정상
[ "$(cat "$HOME/.claude/skills/wiki-vault/VERSION")" = "$(cat "$SETTING/VERSION")" ] || fail "check가 낡은 전역 스킬을 다시 깔지 않음"
# 버전은 내리지 않는다: 위키가 세팅보다 새것이면 그대로
sed -i.bak 's/^version = .*/version = "99.0.0"/' "$TMP/personal/.wiki-setting.toml" && rm "$TMP/personal/.wiki-setting.toml.bak"
"$PY" "$WS" check --vault "$TMP/personal" >/dev/null 2>&1 || true
grep -q '^version = "99.0.0"' "$TMP/personal/.wiki-setting.toml" || fail "check가 위키 세팅 버전을 내림"
# work 프로필은 없앴다 (2026-10-29): 만들 수 없어야 한다
"$PY" "$SETTING/scripts/new_wiki.py" --profile work --dest "$TMP/work" >/dev/null 2>&1 && fail "없앤 work 프로필로 위키가 만들어짐"
"$PY" "$WS" target company >/dev/null 2>&1 && fail "없앤 target 명령이 돎"
echo "통과: 전역"
echo "check.sh 통과"

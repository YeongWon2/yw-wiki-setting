#!/bin/sh
# 세팅 점검: 임시 HOME에서 두 유형을 새로 만들고 점검 · 업그레이드가 도는지 본다. 릴리스 태그는 이게 통과한 커밋에만 단다.
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

for p in personal work; do
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
  [ -f "$W/.claude/skills/yw-wiki-sync/SKILL.md" ] && [ -f "$W/.claude/skills/yw-wiki-review/SKILL.md" ] || fail "$p 스킬 5개 아님"

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
"$PY" "$WS" register "$TMP/personal" >/dev/null && "$PY" "$WS" register "$TMP/work" >/dev/null || fail "register"
"$PY" "$WS" list | grep -q "work: " || fail "list"
[ "$("$PY" "$WS" path personal)" = "$(cd "$TMP/personal" && pwd -P)" ] || fail "path"
# 다음 셋은 앞의 raw 삭제 시험이 남긴 상태라 점검이 실패하므로 되돌린 뒤 본다
for p in personal work; do git -C "$TMP/$p" reset -q --hard; done
"$PY" "$WS" doctor > "$TMP/doctor" || { cat "$TMP/doctor"; fail "doctor"; }
mv "$TMP/work" "$TMP/moved-work"
"$PY" "$WS" doctor >/dev/null && fail "옮긴 볼트를 못 잡음"
# 승격 대상: wiki.toml에 repo만 적어도 자주 쓰는 폴더에서 찾아 등록한다
mv "$TMP/moved-work" "$TMP/work"
CO="$HOME/Desktop/wiki/co-wiki"; mkdir -p "$CO" && git -C "$CO" init -q && git -C "$CO" remote add origin https://github.com/acme/co-wiki.git && echo "# 규칙" > "$CO/SCHEMA.md"
sed -i.bak 's#^repo = "<조직>/<전사 위키 저장소>"#repo = "acme/co-wiki"#' "$TMP/work/wiki.toml" && rm "$TMP/work/wiki.toml.bak"
"$PY" "$WS" target company | grep -q "SCHEMA.md" || fail "target이 repo로 위치를 못 찾음"
"$PY" "$WS" list | grep -q "company: .*읽기 전용" || fail "target이 읽기 전용으로 등록하지 않음"
"$PY" "$WS" target nope >/dev/null 2>&1 && fail "없는 대상을 통과시킴"
# draft의 target_wiki는 [company.wikis]에 있어야 한다
sed -e 's/{{title}}/승격 초안/g' -e 's/{{date}}/2026-10-28/g' -e 's/^target_wiki:.*/target_wiki: nope/' -e 's/^target_path:.*/target_path: concepts\/x/' \
  "$TMP/work/_templates/draft.md" > "$TMP/work/wiki/drafts/승격 초안.md"
"$PY" "$TMP/work/tools/index.py" "$TMP/work" >/dev/null
"$PY" "$TMP/work/tools/check.py" "$TMP/work" | grep -q "target_wiki 'nope'" || fail "없는 target_wiki를 못 잡음"
sed -i.bak 's/^target_wiki: nope/target_wiki: company/' "$TMP/work/wiki/drafts/승격 초안.md" && rm "$TMP/work/wiki/drafts/승격 초안.md.bak"
"$PY" "$TMP/work/tools/check.py" "$TMP/work" | grep -q "target_wiki" && fail "맞는 target_wiki를 오류로 봄"
echo "0.0.1" > "$HOME/.claude/skills/wiki-vault/VERSION"
"$PY" "$WS" check >/dev/null 2>&1 || true  # 오프라인 시험이라 ⚠(1)가 정상
[ "$(cat "$HOME/.claude/skills/wiki-vault/VERSION")" = "$(cat "$SETTING/VERSION")" ] || fail "check가 낡은 전역 스킬을 다시 깔지 않음"
echo "통과: 전역"
echo "check.sh 통과"

#!/bin/sh
# 세팅 점검: 임시 HOME에서 개인 위키를 새로 만들고 점검 · 업그레이드가 도는지 본다. 릴리스 태그는 이게 통과한 커밋에만 단다.
set -eu
SETTING=$(cd "$(dirname "$0")" && pwd)
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT
# 이 폴더 대신 원격 없는 임시 사본으로 시험한다. 길잡이(wiki-setting)는 세팅 사본을 최신 릴리스 태그로
# checkout하므로, 이 폴더를 그대로 쓰면 작업 중인 브랜치가 옛 태그로 옮겨진다(v2.13.0 · v2.13.1이 그렇게 나갔다).
SRC="$SETTING"
git clone -q "$SRC" "$TMP/setting" && git -C "$TMP/setting" remote remove origin
cp "$SRC"/VERSION "$TMP/setting/VERSION"
for f in $(git -C "$SRC" -c core.quotepath=false diff --name-only --diff-filter=d HEAD; git -C "$SRC" -c core.quotepath=false ls-files --others --exclude-standard); do
  mkdir -p "$TMP/setting/$(dirname "$f")"; cp "$SRC/$f" "$TMP/setting/$f"   # 커밋 전 변경도 시험한다
done
for f in $(git -C "$SRC" -c core.quotepath=false diff --name-only --diff-filter=D HEAD); do rm -f "$TMP/setting/$f"; done   # 커밋 전 삭제도
SETTING="$TMP/setting"
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
  [ -d "$W/wiki/guides" ] && grep -q "^# 내 방식 지도" "$W/wiki/synthesis/내 방식 지도.md" || fail "$p 시드(guides 폴더 · 내 방식 지도) 없음"
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
  # 위키 루트에서 작업 이름이 같은 이름 폴더(review/)와 겹쳐도 log는 wiki/log.md에 쓴다
  (cd "$W" && "$PY" tools/log.py review "폴더 이름 작업" "정함 0" >/dev/null) || fail "$p log.py가 작업 이름 review를 경로로 읽음"
  grep -q "review | 폴더 이름 작업" "$W/wiki/log.md" || fail "$p log.py review 기록 없음"
  "$PY" "$W/tools/version.py" "$W" --hook | grep -q "main" || fail "$p version.py (main 확인)"

  # 템플릿으로 만든 페이지 하나와 그걸 가리키는 페이지 하나가 통과해야 한다
  sed -e 's/{{title}}/시험 페이지/g' -e 's/{{date}}/2026-10-28/g' -e 's/^entity_kind:.*/entity_kind: org/' \
    "$W/_templates/entity.md" > "$W/wiki/entities/orgs/시험 페이지.md"
  sed -e 's/{{title}}/시험 개념/g' -e 's/{{date}}/2026-10-28/g' -e 's/^related: \[\]/related: ["[[시험 페이지]]"]/' \
    "$W/_templates/concept.md" > "$W/wiki/concepts/시험 개념.md"
  printf '%s\n' '- [[시험 개념]] — 시험' >> "$W/wiki/entities/orgs/시험 페이지.md"
  "$PY" "$W/tools/index.py" "$W" >/dev/null
  "$PY" "$W/tools/check.py" "$W" >/dev/null || { "$PY" "$W/tools/check.py" "$W"; fail "$p 템플릿 페이지가 점검을 통과하지 못함"; }

  # guide 하네스: 템플릿 원칙은 통과, 규칙을 어긴 판은 오류 · 경고
  G="$W/wiki/guides/시험 원칙.md"
  EV='- 2026-10-28 · 개인 프로젝트 · 통했다 — [[시험 개념]]'
  EV2='- 2026-10-28 · 학교 과제 · 통했다 — [[시험 개념]]'
  guide() { sed -e 's/{{title}}/시험 원칙/g' -e 's/{{date}}/2026-10-28/g' -e 's/^related: \[\]/related: ["[[시험 개념]]"]/' \
      -e 's/^sources: \[\]/sources: ["[[시험 개념]]"]/' -e 's/^1\. $/1. 시험 규칙을 지킨다./' \
      -e 's/^<!-- 어떻게 작업했나.*$/1. 조사하고 만들고 시험했다./' "$@" "$W/_templates/guide.md" > "$G"; }
  expect() {  # expect <ok|err|warn> <찾을 말> <설명>
    "$PY" "$W/tools/check.py" "$W" > "$TMP/g"; rc=$?
    case $1 in
      ok) [ $rc = 0 ] || { cat "$TMP/g"; fail "$p guide $3"; } ;;
      err) [ $rc = 1 ] && grep -q "$2" "$TMP/g" || { cat "$TMP/g"; fail "$p guide $3"; } ;;
      warn) grep -q "경고.*$2" "$TMP/g" || { cat "$TMP/g"; fail "$p guide $3"; } ;;
    esac; }
  guide; "$PY" "$W/tools/index.py" "$W" >/dev/null
  set +e
  expect ok "" "템플릿 원칙이 점검을 통과하지 못함"
  expect warn "확인하지 않은 guide 1개" "미확인 guide 수 경고 없음"
  guide -e 's/^guide_kind:.*/guide_kind: rule/'; expect err "guide_kind" "틀린 guide_kind를 못 잡음"
  guide -e 's/^scope:.*/scope: [회사]/'; expect err "scope" "틀린 scope를 못 잡음"
  guide -e 's/^1\. 시험 규칙을 지킨다\./규칙 한 줄/'; expect err "번호 목록" "번호 없는 Rules를 못 잡음"
  guide -e 's/^<!-- 적용 기록.*$/- 어제 해 봄/'; expect err "Evidence 줄 형식" "틀린 Evidence 줄을 못 잡음"
  guide -e "s/^<!-- 적용 기록.*\$/$EV/" -e 's/^maturity:.*/maturity: tested/'; expect err "tested는" "곳 하나로 tested를 못 막음"
  guide -e "s/^<!-- 적용 기록.*\$/$EV\\
$EV2/" -e 's/^maturity:.*/maturity: tested/'; expect ok "" "곳 둘인 tested가 통과하지 못함"
  guide -e 's/^maturity:.*/maturity: proven/'; expect err "proven은" "확인 없는 proven을 못 막음"
  guide -e 's/^scope:.*/scope: [area:여행]/'; expect err "내 방식" "Evidence 없는 내 방식을 못 막음"
  guide -e 's/^scope:.*/scope: [area:여행]/' -e "s/^<!-- 적용 기록.*\$/$EV/"; expect ok "" "Evidence 있는 내 방식이 통과하지 못함"
  guide -e 's/^scope:.*/scope: [org:시험회사]/'; expect err "회사 방식" "Evidence 없는 회사 방식을 못 막음"
  guide -e 's/^scope:.*/scope: [project:시험]/'; expect err "프로젝트 방식" "Evidence 없는 프로젝트 방식을 못 막음"
  guide -e 's/^scope:.*/scope: [area:일]/' -e "s/^<!-- 적용 기록.*\$/$EV/" -e 's/^1\. 조사하고 만들고 시험했다\.$//'; expect err "How it was done" "빈 작업 과정을 못 막음"
  guide -e 's/^sources: .*/sources: []/'; expect err "근거 없는 guide" "근거 없는 guide를 못 막음"
  guide -e 's/^status:.*/status: superseded/'; expect err "superseded_by" "superseded_by 없는 guide를 못 막음"
  guide -e 's/^owner_checked:.*/owner_checked: true/' -e 's/^owner_checked_on:.*/owner_checked_on: 2026-10-28/'
  expect err "confirm 기록이 없다" "log 없는 owner_checked를 못 막음"
  "$PY" "$W/tools/log.py" "$W" confirm "시험 원칙" "맞다" >/dev/null; expect ok "" "confirm 뒤 owner_checked가 통과하지 못함"
  guide -e 's#^- $#- https://x.atlassian.net/browse/A-1#'; expect err "두지 않는 말" "[guard] patterns(내부 링크)를 못 잡음"
  guide; cp "$G" "$W/wiki/guides/시험원칙.md"; sed -i.bak 's/^title: .*/title: "시험원칙"/' "$W/wiki/guides/시험원칙.md"
  "$PY" "$W/tools/index.py" "$W" >/dev/null; expect warn "중복 후보" "이름이 거의 같은 페이지를 못 잡음"
  set -e
  rm -f "$W/wiki/guides/시험원칙.md" "$W/wiki/guides/시험원칙.md.bak"; "$PY" "$W/tools/index.py" "$W" >/dev/null

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

  # 규칙 문서는 하나당 200줄 이하
  for f in "$W/AGENTS.md" "$W"/.claude/skills/*/SKILL.md "$W"/tools/flows/*.toml; do
    [ "$(wc -l < "$f")" -le 200 ] || fail "$p 규칙 문서 200줄 넘음: $f"
  done
  # 흐름 하네스: 검사를 통과해야 다음 단계로, optional만 건너뛴다
  flow() { "$PY" "$W/tools/flow.py" "$W" "$@" > "$TMP/flow" 2>&1; }
  [ "$(ls "$W/tools/flows" | wc -l | tr -d ' ')" = 5 ] || fail "$p 흐름 5개 아님"
  flow next && fail "$p 시작 전 next가 통과"
  flow start lint && flow done || { cat "$TMP/flow"; fail "$p 흐름 start 단계"; }
  flow done && fail "$p note 없이 다음 단계로 감"
  flow done --note "오류 0" && flow done --note "없음" && flow done --note "없음" && flow done || { cat "$TMP/flow"; fail "$p lint 흐름 중간"; }
  flow done && fail "$p log 없이 log 단계 통과"
  (cd "$W" && "$PY" tools/log.py lint "흐름 시험" "고침 0" >/dev/null)
  flow done || { cat "$TMP/flow"; fail "$p log 뒤에도 log 단계 실패"; }
  flow done && fail "$p 커밋 안 했는데 push 단계 통과"
  flow skip --note "x" && fail "$p optional 아닌 단계를 건너뜀"
  flow start query && flow done && flow done --note "index" && flow done --note "페이지" && flow done --note "답" || { cat "$TMP/flow"; fail "$p query 흐름"; }
  flow done && fail "$p synthesis를 안 바꿨는데 저장 단계 통과"
  flow skip --note "한 번 묻고 끝" || fail "$p optional 단계 skip"
  grep -q "push" "$TMP/flow" || fail "$p skip 뒤 다음 단계 안내 없음"
  flow abort && [ ! -f "$W/.yw-wiki/flow.json" ] || fail "$p abort"
  git -C "$W" check-ignore -q .yw-wiki/flow.json || fail "$p 흐름 상태가 git에 올라감"

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
# 상위 폴더 볼트: Obsidian은 위키를 품은 폴더를 열고, 그 폴더와 위키 이름이 함께 바뀌어도 따라간다
mkdir -p "$TMP/hub" && mv "$TMP/personal" "$TMP/hub/w" && "$PY" "$WS" register "$TMP/hub/w" >/dev/null
printf '{"vaults":{"cccc":{"path":"%s/hub","ts":1}}}' "$TMP" > "$OJ"
"$PY" "$WS" obsidian | grep -q "상위 폴더 볼트" || fail "상위 폴더 볼트를 못 봄"
grep -q 'obsidian_depth = 1' "$HOME/.config/wiki/vaults.toml" || fail "상위 폴더 볼트 단계를 기록 안 함"
mv "$TMP/hub" "$TMP/hub2" && mv "$TMP/hub2/w" "$TMP/hub2/w2" && "$PY" "$WS" register "$TMP/hub2/w2" >/dev/null
grep -q 'previous_path' "$HOME/.config/wiki/vaults.toml" || fail "다시 등록할 때 이전 경로를 안 남김"
YW_OBSIDIAN_RUNNING=0 "$PY" "$WS" obsidian --fix > "$TMP/obs" || { cat "$TMP/obs"; fail "상위 폴더 볼트 옮기기 실패"; }
grep -q 'hub2"' "$OJ" || { cat "$TMP/obs"; fail "상위 폴더 볼트가 바뀐 폴더를 따라가지 않음"; }
grep -q 'previous_path' "$HOME/.config/wiki/vaults.toml" && fail "옮긴 뒤 이전 경로가 남음"
mv "$TMP/hub2/w2" "$TMP/personal" && "$PY" "$WS" register "$TMP/personal" >/dev/null
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

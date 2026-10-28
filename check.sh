#!/bin/sh
# 세팅 점검: 임시 HOME에서 두 유형을 새로 만들고 점검 · 업그레이드가 도는지 본다. 릴리스 태그는 이게 통과한 커밋에만 단다.
set -eu
SETTING=$(cd "$(dirname "$0")" && pwd)
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT
export HOME="$TMP/home" YW_TODAY=2026-10-28
mkdir -p "$HOME"
git config --global user.name check >/dev/null 2>&1 || true
git config --global user.email check@example.invalid >/dev/null 2>&1 || true
fail() { echo "실패: $1"; exit 1; }

for p in personal work; do
  W="$TMP/$p"
  python3 "$SETTING/scripts/new_wiki.py" --profile "$p" --dest "$W" >/dev/null || fail "$p 생성 직후 check.py 오류"
  python3 "$W/tools/index.py" "$W" --check >/dev/null || fail "$p index 최신 아님"
  [ "$(cat "$W/CLAUDE.md")" = "@AGENTS.md" ] || fail "$p CLAUDE.md"
  grep -q "profile:start $p" "$W/AGENTS.md" || fail "$p AGENTS.md profile 구간"
  grep -rq '{{wiki_name}}' "$W/.claude/skills" && fail "$p 스킬 이름 치환 안 됨"
  [ "$(git -C "$W" config core.hooksPath)" = ".githooks" ] || fail "$p 훅 연결"

  # 템플릿으로 만든 페이지 하나와 그걸 가리키는 페이지 하나가 통과해야 한다
  sed -e 's/{{title}}/시험 페이지/g' -e 's/{{date}}/2026-10-28/g' -e 's/^entity_kind:.*/entity_kind: org/' \
    "$W/_templates/entity.md" > "$W/wiki/entities/orgs/시험 페이지.md"
  sed -e 's/{{title}}/시험 개념/g' -e 's/{{date}}/2026-10-28/g' -e 's/^related: \[\]/related: ["[[시험 페이지]]"]/' \
    "$W/_templates/concept.md" > "$W/wiki/concepts/시험 개념.md"
  printf '%s\n' '- [[시험 개념]] — 시험' >> "$W/wiki/entities/orgs/시험 페이지.md"
  python3 "$W/tools/index.py" "$W" >/dev/null
  python3 "$W/tools/check.py" "$W" >/dev/null || { python3 "$W/tools/check.py" "$W"; fail "$p 템플릿 페이지가 점검을 통과하지 못함"; }

  # 잘못된 페이지는 오류여야 한다
  printf -- '---\ntitle: 틀림\ntype: entity\n---\n\n# 틀림\n\n[[없는 문서]]\n' > "$W/wiki/concepts/틀림.md"
  python3 "$W/tools/index.py" "$W" >/dev/null
  python3 "$W/tools/check.py" "$W" >/dev/null && fail "$p 잘못된 페이지를 못 잡음"
  rm "$W/wiki/concepts/틀림.md"; python3 "$W/tools/index.py" "$W" >/dev/null

  # 충돌 칸은 위키에 두면 오류, review/ 항목은 통과(경고)
  printf '\n## Contradictions\n' >> "$W/wiki/concepts/시험 개념.md"
  python3 "$W/tools/check.py" "$W" >/dev/null && fail "$p 위키 안의 충돌 칸을 못 잡음"
  sed -i.bak '/^## Contradictions$/d' "$W/wiki/concepts/시험 개념.md" && rm "$W/wiki/concepts/시험 개념.md.bak"
  sed -e 's/{{title}}/시험 검토/g' -e 's/{{date}}/2026-10-28/g' -e 's/^review_kind:.*/review_kind: conflict/' \
    "$W/_templates/review.md" > "$W/review/2026-10-28 시험 검토.md"
  python3 "$W/tools/check.py" "$W" > "$TMP/out" || { cat "$TMP/out"; fail "$p review 항목이 점검을 통과하지 못함"; }
  grep -q "검토 대기 1개" "$TMP/out" || fail "$p review 대기 경고 없음"
  [ -f "$W/.claude/skills/yw-wiki-sync/SKILL.md" ] && [ -f "$W/.claude/skills/yw-wiki-review/SKILL.md" ] || fail "$p 스킬 5개 아님"

  # 업그레이드: 바뀐 게 없으면 최신, 관리 파일을 망가뜨리면 되돌리고 local 구간은 남긴다
  python3 "$SETTING/scripts/upgrade.py" "$W" | grep -q "최신" || fail "$p upgrade 멱등"
  echo "# 망가뜨림" >> "$W/tools/check.py"
  sed -i.bak 's/(없음\. 이 구간은 upgrade가 건드리지 않는다\.)/로컬 규칙 하나/' "$W/AGENTS.md" && rm "$W/AGENTS.md.bak"
  python3 "$SETTING/scripts/upgrade.py" "$W" >/dev/null
  grep -q "# 망가뜨림" "$W/tools/check.py" && fail "$p upgrade가 관리 파일을 되돌리지 않음"
  grep -q "로컬 규칙 하나" "$W/AGENTS.md" || fail "$p upgrade가 local 구간을 지움"
  grep -q "upgrade |" "$W/wiki/log.md" || fail "$p upgrade 기록 없음"
  python3 "$W/tools/check.py" "$W" >/dev/null || fail "$p upgrade 뒤 점검 오류"

  # raw 삭제는 커밋 점검에서 오류
  echo "원본" > "$W/raw/assets/a.md"
  git -C "$W" add -A >/dev/null && git -C "$W" -c core.hooksPath=/dev/null commit -qm init
  git -C "$W" rm -q "$W/raw/assets/a.md"
  (cd "$W" && python3 tools/check.py >/dev/null) && fail "$p raw 삭제를 못 잡음"
  echo "통과: $p"
done
echo "check.sh 통과"

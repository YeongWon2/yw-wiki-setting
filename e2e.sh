#!/bin/sh
# 시나리오 시험: 실제 위키의 임시 복사본으로 사용 흐름을 처음부터 끝까지 돌린다. 실제 위키 · 원격은 건드리지 않는다.
#   sh e2e.sh <개인 위키> <업무 위키>
# 보는 것: 세션 훅 → 갱신 → 업데이트(새 원본 → source · index · log → 커밋 훅) → 충돌 → review → 리뷰 반영 →
#          lint(망가뜨린 것을 잡음) → 짧은 PATH(Obsidian)에서 커밋 → 원격이 앞섰을 때 자동으로 받기 → main이 아니면 경고
set -eu
PERSONAL=$(cd "$1" && pwd); WORK=$(cd "$2" && pwd)
SETTING=$(cd "$(dirname "$0")" && pwd)
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT
export YW_TODAY=$(date +%F)
PY=$(command -v python3.15 || command -v python3.14 || echo /opt/homebrew/bin/python3.14)
fail() { echo "실패: $1"; exit 1; }
ok() { echo "통과: $1"; }

for src in "$PERSONAL" "$WORK"; do
  name=$(basename "$src")
  # 원격 흉내: bare 저장소 하나와 복사본 둘(A: 내 PC, B: 다른 기기)
  git clone -q --bare "$src" "$TMP/$name.git"
  # 원격 main = 지금 보는 커밋 (병합 전 브랜치도 병합 뒤 모습으로 시험)
  git -C "$TMP/$name.git" branch -f main "$(git -C "$src" rev-parse HEAD)"
  git -C "$TMP/$name.git" symbolic-ref HEAD refs/heads/main
  git clone -q "$TMP/$name.git" "$TMP/$name-A"; git clone -q "$TMP/$name.git" "$TMP/$name-B"
  A="$TMP/$name-A"; B="$TMP/$name-B"
  for W in "$A" "$B"; do
    git -C "$W" checkout -q main 2>/dev/null || git -C "$W" checkout -q -b main
    git -C "$W" config core.hooksPath .githooks; git -C "$W" config user.name e2e; git -C "$W" config user.email e2e@example.invalid
    cp -R "$src/.obsidian/plugins" "$W/.obsidian/" 2>/dev/null || true
  done

  # 1. 세션 시작 훅 (.claude/settings.json의 명령 그대로)
  CMD=$("$PY" -c "import json,sys;print(json.load(open(sys.argv[1]))['hooks']['SessionStart'][0]['hooks'][0]['command'])" "$A/.claude/settings.json")
  OUT=$(CLAUDE_PROJECT_DIR="$A" sh -c "$CMD")
  echo "$OUT" | grep -q "main" || fail "$name 세션 훅: $OUT"
  ok "$name 세션 훅 — $OUT"

  # 2. 업데이트: 새 원본 → source 페이지 → index · log → 커밋(훅이 점검)
  if grep -q 'profile = "work"' "$A/wiki.toml"; then RAW=raw/meetings; else RAW=raw/notes; fi
  mkdir -p "$A/$RAW"
  printf -- '---\ntype: note\ndate: %s\n---\n\n# e2e 시험 메모\n\n시험 사실: 시나리오 시험이 돈다.\n' "$YW_TODAY" > "$A/$RAW/$YW_TODAY e2e 시험.md"
  sed -e "s/{{title}}/e2e 시험/g" -e "s/{{date}}/$YW_TODAY/g" -e 's/^source_kind:.*/source_kind: doc/' \
      -e "s#^sources: \[\]#sources: [\"$RAW/$YW_TODAY e2e 시험.md\"]#" -e 's/^related: \[\]/related: ["[[황영원]]"]/' \
      "$A/_templates/source.md" > "$A/wiki/sources/e2e 시험.md"
  grep -q 'profile = "work"' "$A/wiki.toml" && sed -i.bak 's/\[\[황영원\]\]/[[QMIT]]/' "$A/wiki/sources/e2e 시험.md" && rm "$A/wiki/sources/e2e 시험.md.bak"
  "$PY" "$A/tools/index.py" "$A" >/dev/null
  "$PY" "$A/tools/log.py" "$A" update "e2e 시험" "source 1" >/dev/null
  git -C "$A" add -A
  git -C "$A" commit -qm "e2e: 업데이트" || fail "$name 업데이트 커밋이 점검 훅에 막힘"
  grep -q "\[\[e2e 시험\]\]" "$A/wiki/index.md" || fail "$name index에 새 source 없음"
  ok "$name 업데이트 → source · index · log · 커밋 훅"

  # 3. 충돌은 위키에 못 쓴다 → review/로 → 리뷰로 정하고 반영 → 항목 지움
  printf '\n## Contradictions\n\n- 다른 내용\n' >> "$A/wiki/sources/e2e 시험.md"
  git -C "$A" add -A
  git -C "$A" commit -qm "e2e: 충돌을 위키에" 2>/dev/null && fail "$name 위키 안 충돌 칸이 커밋됨"
  git -C "$A" reset -q --hard HEAD
  sed -e "s/{{title}}/e2e 충돌/g" -e "s/{{date}}/$YW_TODAY/g" -e 's/^review_kind:.*/review_kind: conflict/' \
      -e 's/^page:.*/page: "[[e2e 시험]]"/' -e 's/^found_by:.*/found_by: update/' "$A/_templates/review.md" > "$A/review/$YW_TODAY e2e 충돌.md"
  git -C "$A" add -A && git -C "$A" commit -qm "e2e: review 항목" || fail "$name review 항목 커밋 실패"
  "$PY" "$A/tools/check.py" "$A" | grep -q "검토 대기" || fail "$name review 대기 경고 없음"
  sed -i.bak 's/^- 시험 사실.*//' "$A/wiki/sources/e2e 시험.md" && rm "$A/wiki/sources/e2e 시험.md.bak"
  git -C "$A" rm -q "review/$YW_TODAY e2e 충돌.md"
  "$PY" "$A/tools/log.py" "$A" review "e2e 충돌" "정함 1" >/dev/null
  git -C "$A" add -A && git -C "$A" commit -qm "e2e: 리뷰 반영" || fail "$name 리뷰 반영 커밋 실패"
  ok "$name 충돌 → review/ → 리뷰 반영"

  # 4. lint: 깨진 링크 · 낡은 index는 커밋을 막는다
  printf '\n- [[없는 페이지]]\n' >> "$A/wiki/sources/e2e 시험.md"
  git -C "$A" add -A
  git -C "$A" commit -qm "e2e: 깨진 링크" 2>/dev/null && fail "$name 깨진 링크가 커밋됨"
  git -C "$A" reset -q --hard HEAD
  ok "$name lint가 깨진 링크를 막음"

  # 5. Obsidian처럼 PATH가 짧은 곳에서 커밋 (Obsidian Git이 하는 커밋)
  printf -- '---\ntype: note\n---\n\n# 짧은 PATH 메모\n' > "$A/raw/짧은 PATH.md" 2>/dev/null || { mkdir -p "$A/raw"; printf -- '# 짧은 PATH 메모\n' > "$A/raw/짧은 PATH.md"; }
  git -C "$A" add -A
  env -i HOME="$HOME" PATH=/usr/bin:/bin git -C "$A" -c user.name=obs -c user.email=o@example.invalid commit -qm "obsidian: 메모" \
    || fail "$name 짧은 PATH에서 커밋 실패 (훅이 python3.14를 못 찾음)"
  ok "$name 짧은 PATH(Obsidian) 커밋"
  git -C "$A" push -q origin main

  # 6. 다른 기기(B): 원격이 앞서 있으면 version.py가 받는다
  "$PY" "$B/tools/version.py" "$B" --force > "$TMP/v" || true
  grep -q "origin/main에서 .*개 받음" "$TMP/v" || { cat "$TMP/v"; fail "$name 다른 기기에서 원격 앞선 것을 받지 않음"; }
  [ -f "$B/wiki/sources/e2e 시험.md" ] || fail "$name 받은 뒤 파일 없음"
  ok "$name 다른 기기 자동 받기 — $(cat "$TMP/v")"

  # 7. main이 아니면 경고
  git -C "$B" checkout -q -b 다른-브랜치
  "$PY" "$B/tools/version.py" "$B" --force | grep -q "main 기준이 아니다" || fail "$name main 아님 경고 없음"
  ok "$name main 아닌 브랜치 경고"
done

# 8. 길잡이: 목록 · 경로 · 확인 · doctor (실제 PC 설정을 읽기만)
WS="$HOME/.claude/skills/wiki-vault/bin/wiki-setting"
[ -x "$WS" ] || fail "전역 스킬 없음 — wiki-setting install"
"$PY" "$WS" list | grep -q "work:" || fail "길잡이 목록"
[ -d "$("$PY" "$WS" path work)" ] || fail "길잡이 경로"
ok "길잡이 목록 · 경로 — $("$PY" "$WS" list | head -1)"
echo "e2e 통과"

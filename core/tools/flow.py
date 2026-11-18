#!/usr/bin/env python3.14
"""작업 흐름 하네스: 스킬의 단계를 하나씩 안내하고, 단계마다 검사를 통과해야 다음으로 넘긴다.

  python3.14 tools/flow.py list                 흐름 목록
  python3.14 tools/flow.py start <흐름>          시작 (update · update-code · query · review · lint)
  python3.14 tools/flow.py next                 지금 단계 보기
  python3.14 tools/flow.py done [--note "…"]    지금 단계 검사 → 통과하면 다음 단계
  python3.14 tools/flow.py skip --note "이유"    optional 단계만 건너뜀
  python3.14 tools/flow.py abort                흐름 버림

흐름은 tools/flows/<이름>.toml, 상태는 .yw-wiki/flow.json(git에 안 올림).
검사 종류(gates): note(무엇을 했는지 --note 필수) · run:<명령>(종료 0) · run?:<명령>(보여 주기만) ·
pull(원격이 있으면 git pull --rebase) · changed:<경로>(흐름 시작 뒤 그 아래 파일이 바뀜) ·
log:<작업>(시작 뒤 wiki/log.md에 그 작업 기록) · check(tools/check.py 오류 0) · pushed(커밋 · push 끝남)
"""
import json
import shlex
import subprocess
import sys
import tomllib
from datetime import datetime

import wikilib

FLOWS = wikilib.ROOT / "tools" / "flows"


def git(root, *args):
    return subprocess.run(["git", "-C", str(root), "-c", "core.quotepath=false", *args], capture_output=True, text=True)


def load_flow(name):
    path = FLOWS / f"{name}.toml"
    if not path.exists():
        sys.exit(f"없는 흐름: {name} (있는 것: {', '.join(flow_names())})")
    return tomllib.loads(path.read_text())


def flow_names():
    return sorted(p.stem for p in FLOWS.glob("*.toml"))


def state_path(root):
    return root / ".yw-wiki" / "flow.json"


def load_state(root):
    path = state_path(root)
    if not path.exists():
        sys.exit("진행 중인 흐름이 없다. `python3.14 tools/flow.py start <흐름>`으로 시작한다.")
    return json.loads(path.read_text())


def save_state(root, state):
    path = state_path(root)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(state, ensure_ascii=False, indent=1))


def log_count(root, action):
    path = root / "wiki" / "log.md"
    text = path.read_text() if path.exists() else ""
    return text.count(f"] {action} |")


def changed_paths(root, start_head):
    out = set()
    for line in git(root, "status", "--porcelain", "-uall").stdout.splitlines():
        out.add(line[3:].strip('"').split(" -> ")[-1])
    if start_head:
        out.update(git(root, "diff", "--name-only", start_head, "HEAD").stdout.splitlines())
    return out


def run_cmd(root, cmd):
    argv = [sys.executable if a == "{py}" else a for a in shlex.split(cmd)]  # 흐름 파일(세팅 관리)의 명령만 돈다
    r = subprocess.run(argv, cwd=root, capture_output=True, text=True)
    text = (r.stdout + r.stderr).strip()
    return r.returncode, text


def gate(root, state, spec, note):
    """(통과?, 설명) 하나를 돌려준다."""
    kind, _, arg = spec.partition(":")
    if kind == "note":
        return bool(note and note.strip()), "--note로 무엇을 했는지(또는 사용자 답) 적는다"
    if kind in ("run", "run?"):
        rc, text = run_cmd(root, arg)
        if text:
            print("\n".join("  │ " + l for l in text.splitlines()[-15:]))
        return (rc == 0 or kind == "run?"), f"`{arg}` 종료 코드 {rc}"
    if kind == "pull":
        if not git(root, "rev-parse", "--abbrev-ref", "@{u}").stdout.strip():
            return True, "원격 없음"
        r = git(root, "pull", "--rebase")
        return r.returncode == 0, "git pull --rebase 실패 — 충돌이면 멈추고 사람에게 넘긴다"
    if kind == "changed":
        hit = [p for p in changed_paths(root, state.get("head")) if p.startswith(arg)]
        return bool(hit), f"흐름 시작 뒤 `{arg}` 아래 바뀐 파일이 없다"
    if kind == "log":
        return log_count(root, arg) > state["logs"].get(arg, 0), f"`python3.14 tools/log.py {arg} …` 기록이 없다"
    if kind == "check":
        rc, text = run_cmd(root, "{py} tools/check.py")
        if rc:
            print("\n".join("  │ " + l for l in text.splitlines()[-15:]))
        return rc == 0, "tools/check.py 오류를 고친다"
    if kind == "pushed":
        if git(root, "status", "--porcelain").stdout.strip():
            return False, "커밋하지 않은 변경이 있다"
        if git(root, "rev-parse", "--abbrev-ref", "@{u}").stdout.strip():
            ahead = git(root, "rev-list", "--count", "@{u}..HEAD").stdout.strip()
            return ahead == "0", f"push하지 않은 커밋 {ahead}개"
        return True, "원격 없음"
    return False, f"모르는 검사: {spec}"


def show(state, flow):
    steps = flow["step"]
    i = state["index"]
    if i >= len(steps):
        print(f"[{flow['title']}] 모든 단계 끝 ✓")
        return
    s = steps[i]
    opt = " (optional — 해당 없으면 skip --note)" if s.get("optional") else ""
    print(f"[{flow['title']}] {i + 1}/{len(steps)} {s['id']}{opt}")
    print(s["do"].strip())
    print("검사: " + ", ".join(s.get("gates", [])) + " → `python3.14 tools/flow.py done" +
          (' --note "…"`' if "note" in s.get("gates", []) else "`"))


def main():
    args = sys.argv[1:]
    root = wikilib.wiki_root_arg(args[0]) if args else None
    if root:
        args = args[1:]
    else:
        root = wikilib.ROOT
    if not args:
        print(__doc__)
        return 2
    cmd, rest = args[0], args[1:]
    note = rest[rest.index("--note") + 1] if "--note" in rest and rest.index("--note") + 1 < len(rest) else None

    if cmd == "list":
        for n in flow_names():
            print(f"{n}: {load_flow(n)['title']}")
        return 0
    if cmd == "start":
        if not rest:
            print(f"흐름 이름이 필요하다: {', '.join(flow_names())}")
            return 2
        name = rest[0]
        flow = load_flow(name)
        if state_path(root).exists():
            old = json.loads(state_path(root).read_text())
            print(f"(진행 중이던 흐름 {old['flow']}를 버리고 새로 시작)")
        head = git(root, "rev-parse", "HEAD").stdout.strip() or None
        actions = {g.partition(":")[2] for s in flow["step"] for g in s.get("gates", []) if g.startswith("log:")}
        state = {"flow": name, "index": 0, "head": head, "started": datetime.now().isoformat(timespec="seconds"),
                 "logs": {a: log_count(root, a) for a in actions}, "notes": {}}
        save_state(root, state)
        show(state, flow)
        return 0
    if cmd == "abort":
        state_path(root).unlink(missing_ok=True)
        print("흐름을 버렸다")
        return 0

    state = load_state(root)
    flow = load_flow(state["flow"])
    steps = flow["step"]
    if cmd == "next":
        show(state, flow)
        return 0
    if state["index"] >= len(steps):
        show(state, flow)
        return 0
    step = steps[state["index"]]
    if cmd == "skip":
        if not step.get("optional"):
            print(f"{step['id']}는 건너뛸 수 없는 단계다")
            return 1
        if not note:
            print("skip에는 --note로 이유를 적는다")
            return 1
    elif cmd == "done":
        fails = []
        for spec in step.get("gates", []):
            ok, why = gate(root, state, spec, note)
            if not ok:
                fails.append(f"  ✗ {spec} — {why}")
        if fails:
            print(f"[{flow['title']}] {step['id']} 검사 실패:")
            print("\n".join(fails))
            return 1
    else:
        print(__doc__)
        return 2
    state["notes"][step["id"]] = ("(건너뜀) " if cmd == "skip" else "") + (note or "")
    state["index"] += 1
    save_state(root, state)
    print(f"✓ {step['id']}")
    show(state, flow)
    if state["index"] >= len(steps):
        state_path(root).unlink(missing_ok=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())

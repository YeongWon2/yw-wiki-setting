#!/bin/sh
# 이 PC에 yw-wiki CLI(Rust)를 빌드해 ~/.local/bin에 두고, 길잡이 스킬 · 볼트 목록(~/.config/wiki/vaults.toml)을 깐다.
#   sh install.sh        (세팅 폴더 어디서든)
set -eu
S=$(cd "$(dirname "$0")" && pwd)
command -v cargo >/dev/null 2>&1 || { echo "cargo가 없다 — brew install rust"; exit 1; }
CARGO_TARGET_DIR="$HOME/.cache/yw-wiki/target" cargo install --quiet --locked --force --path "$S/cli" --root "$HOME/.local"
"$HOME/.local/bin/yw-wiki" setting install --from "$S"
case ":$PATH:" in
  *":$HOME/.local/bin:"*) ;;
  *) echo "PATH에 ~/.local/bin이 없다 — ~/.zshrc에 export PATH=\"\$HOME/.local/bin:\$PATH\"를 더한다" ;;
esac
"$HOME/.local/bin/yw-wiki" --version

#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN="$ROOT/.output/RustSharp.Server.dll"

[[ -f "$BIN" ]] || bash "$ROOT/build.sh"

export PORT="${PORT:-8099}"
exec dotnet "$BIN"

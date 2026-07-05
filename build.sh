#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUT="$ROOT/.output"

echo "Building Rust library..."
cargo build --release --manifest-path "$ROOT/rust/Cargo.toml"

echo "Publishing C# server..."
dotnet publish "$ROOT/csharp/RustSharp.Server" -c Release -o "$OUT" --nologo

echo "Copying librustsharp.so..."
cp "$ROOT/rust/target/release/librustsharp.so" "$OUT/"

echo "Done. Run: bash rustsharp/run.sh"

#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

echo "Building WASM package with wasm-pack…"
wasm-pack build --target web --out-dir web/pkg

echo ""
echo "Done. To run:"
echo "  cd web && python3 -m http.server 8080"
echo "  open http://localhost:8080"

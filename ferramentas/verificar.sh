#!/usr/bin/env bash
# Executa todas as verificações exigidas antes de um commit (mesmas do CI).
# Uso: ferramentas/verificar.sh   — sai com código ≠ 0 se qualquer verificação falhar.
set -euo pipefail
cd "$(dirname "$0")/.."
export PATH="$HOME/.cargo/bin:$PATH"
etapa() { printf '\n== %s\n' "$1"; }
etapa "formatação";            cargo fmt --all --check
etapa "clippy";                cargo clippy --workspace --all-targets --quiet -- -D warnings
etapa "testes";                cargo test --workspace --quiet 2>&1 | grep -E 'test result: ok. [1-9]|FAILED|panicked' || true
                               cargo test --workspace --quiet >/dev/null 2>&1
etapa "licenças, fontes e avisos"; cargo deny check licenses bans sources advisories 2>&1 | tail -1
                               cargo deny check licenses bans sources advisories >/dev/null 2>&1
etapa "documentos gerados";    python3 -I ferramentas/gerar_docs.py --verificar
printf '\nTodas as verificações passaram.\n'

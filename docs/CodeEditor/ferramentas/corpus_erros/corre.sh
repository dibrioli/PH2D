#!/usr/bin/env bash
# Corpus de erros (CodeEditor I0c): gera os mutantes, corre as duas vias e refaz as tabelas.
# Uso (da raiz da worktree):  bash docs/CodeEditor/ferramentas/corpus_erros/corre.sh
# Os mutantes vivem FORA do repo (o `.luaurc` da raiz poria o luau-lsp em modo estrito).
set -euo pipefail
raiz="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
aqui="$raiz/docs/CodeEditor/ferramentas/corpus_erros"
export PH2D_CORPUS_DIR="${PH2D_CORPUS_DIR:-$HOME/.cache/ph2d/corpus_erros}"
export PH2D_CORPUS_SAIDA="$aqui/runtime.jsonl"
cd "$raiz"
python3 -I "$aqui/corpus_erros.py" gera
python3 -I "$aqui/corpus_erros.py" estatico
bash scripts/ph2d-run.sh cargo test -p ph2d-script --test it corpus_de_erros -- --ignored --nocapture
python3 -I "$aqui/corpus_erros.py" tabelas

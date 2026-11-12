#!/bin/bash
# uso: build.sh <sha>...   constrói cada ponto em sequência na worktree de bissecção
SP=/tmp/claude-1000/-home-enio-Documentos-Projetos-PH2D/fb53f5ce-2f42-4773-b6f1-219f6399f510/scratchpad
W=/home/enio/Documentos/Projetos/.ph2d-bis-4k; export CARGO_TARGET_DIR=/home/enio/Documentos/Projetos/.ph2d-bis-4k-target
cd $W
for s in "$@"; do
  [ -x $SP/bin_$s ] && continue
  git checkout -q --detach $s || exit 1
  t0=$(date +%s)
  bash scripts/ph2d-run.sh cargo build -q -p ph2d-host-desktop --profile smoke > $SP/build_$s.log 2>&1 || { echo "$s FALHOU"; continue; }
  cp $CARGO_TARGET_DIR/smoke/ph2d-host-desktop $SP/bin_$s
  mkdir -p $SP/arv_$s; git archive $s | tar -x -C $SP/arv_$s --exclude='docs/*' ; git archive $s docs/Painter/ferramentas | tar -x -C $SP/arv_$s
  echo "$(date +%T) $s ok $(( $(date +%s)-t0 ))s"
done
echo "[build] feito"

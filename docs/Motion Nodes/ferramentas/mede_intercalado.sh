#!/bin/bash
# mede_intercalado.sh — a régua do doc 121 §9.16: a sonda INTERCALADA (`sonda_intercalada`, todas as variantes e
# cenas num só processo, em blocos rodados, relógio da placa) nas duas placas, com UMA compilação.
#
# Substitui `mede_sonda_das_estrelas.sh` para comparar variantes: aquela abria um processo por célula e
# esperava a máquina calma antes de cada um (a rodada do §9.15: 4–5 h); esta mede 9 variantes × 6 cenas
# em ~11 s (iGPU) e ~28 s (RTX), sem esperar calma — a carga alheia cai em todas as variantes por igual e
# o resumo é o MÍNIMO das rodadas. Medido em 05/10 com load 20–29: mínimo e mediana iguais ao 0,01.
#
# uso (dentro da worktree): bash "docs/Motion Nodes/ferramentas/mede_intercalado.sh" <dir-de-saida>
# Ambiente (passa à sonda): PH2D_SONDA_VARIANTES · PH2D_SONDA_CENAS · PH2D_SONDA_BLOCO · PH2D_SONDA_RODADAS ·
#           PH2D_SONDA_VELLO=0 · PLACAS="igpu rtx"
set -u
OUT="${1:?dir-de-saida}"
RAIZ="$(cd "$(dirname "$0")/../../.." && pwd)"
PLACAS="${PLACAS:-igpu rtx}"
mkdir -p "$OUT"
t0=$(date +%s)
# UMA compilação, no perfil `smoke` (otimizado e incremental): o relógio é o da PLACA, e o nível de
# otimização do Rust só mexe no lado da CPU.
saida=$(bash "$RAIZ/scripts/ph2d-run.sh" cargo test -p ph2d-app-motion --lib --profile smoke --no-run 2>&1) \
  || { echo "$saida" | tail -20; echo "a compilação falhou" >&2; exit 2; }
bin=$(echo "$saida" | grep -o 'target/smoke/deps/ph2d_app_motion-[0-9a-f]*' | tail -1)
[ -x "$RAIZ/$bin" ] || { echo "binário da sonda não encontrado" >&2; exit 2; }
cp "$RAIZ/$bin" "$OUT/sonda"
echo "compilou em $(( $(date +%s) - t0 )) s"
for placa in $PLACAS; do
  kv=()
  [ "$placa" = igpu ] && kv+=(VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/radeon_icd.json)
  # ⛔ 05/10: na RTX o processo da sonda morria com SIGSEGV DEPOIS do `test result: ok` — uma thread do
  # driver ainda a gravar a cache de shaders (54 passes × ~20 pipelines) quando a biblioteca já fora
  # descarregada (core: salto para 0x170 numa thread que não a principal). Com a cache desligada sai
  # limpo. A cache só acelera o ARRANQUE seguinte; o relógio da placa não a vê.
  kv+=(__GL_SHADER_DISK_CACHE=0)
  t=$(date +%s)
  PH2D_GPU=1 PH2D_GPU_ESPERA=1500 PH2D_PRAZO=900 bash "$RAIZ/scripts/ph2d-run.sh" \
    env "${kv[@]}" "$OUT/sonda" --ignored --nocapture --test-threads=1 sonda_intercalada > "$OUT/$placa.txt" 2>&1
  rc=$?
  echo "$placa: exit $rc em $(( $(date +%s) - t )) s"
  # ⛔ 05/10 (doc 121 §9.17): com 13 variantes a RTX morreu outra vez DEPOIS do `test result: ok`, mesmo sem a
  # cache. A tabela já saiu inteira: aceita-se, e diz-se alto ao lado da tabela.
  if [ "$rc" -ne 0 ] && grep -q '^test result: ok' "$OUT/$placa.txt"; then
    echo "⚠️  $placa: exit $rc DEPOIS do 'test result: ok' (a saída do driver; os dados estão completos)" | tee -a "$OUT/avisos.txt"
  elif [ "$rc" -ne 0 ]; then
    tail -5 "$OUT/$placa.txt"; exit 3
  fi
done
for placa in $PLACAS; do
  echo "== $placa"
  grep '^INTERCALADA' "$OUT/$placa.txt" | sed 's/^INTERCALADA //'
done > "$OUT/tabela.txt"
[ -f "$OUT/avisos.txt" ] && cat "$OUT/avisos.txt" >> "$OUT/tabela.txt"
echo "tudo em $(( $(date +%s) - t0 )) s · tabela: $OUT/tabela.txt"

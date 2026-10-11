#!/usr/bin/env bash
# Corre o oráculo Godot da Luz 2D (W5e, Bloco C) e grava a fixture com cabeçalho.
#   bash docs/Luz2D/oraculo_godot/corre.sh [saida.txt] [-- --rodadas=3 --quadros=60 ...]
# Precisa de janela real (GPU): passa pela porta da placa (PH2D_GPU=1, exit 75 = espere).
# Método: docs/_ComoInvestigarApps/00_o_metodo.md — o Godot (MIT) é CORRIDO, nunca lido.
set -uo pipefail
AQUI="$(cd "$(dirname "$0")" && pwd)"
RAIZ="$(git -C "$AQUI" rev-parse --show-toplevel)"
SAIDA="${1:-$AQUI/medida_godot_w5e.txt}"
shift || true
[ "${1:-}" = "--" ] && shift
CRUA="$AQUI/saida_crua.txt"
CMD=(godot --path "$AQUI" --rendering-driver vulkan --rendering-method forward_plus
     --script res://oraculo.gd -- "$@")

carga_antes="$(cat /proc/loadavg)"
gpu_antes="$(nvidia-smi --query-gpu=name,driver_version,utilization.gpu,clocks.gr,temperature.gpu --format=csv,noheader 2>/dev/null || echo '?')"
cd "$RAIZ" || exit 1
while :; do
  PH2D_GPU=1 PH2D_PRAZO=1200 bash scripts/ph2d-run.sh "${CMD[@]}" >"$CRUA" 2>"$CRUA.err"
  rc=$?
  [ "$rc" = "75" ] && { echo "placa ocupada (75) — nova tentativa em 30 s" >&2; sleep 30; continue; }
  break
done
carga_depois="$(cat /proc/loadavg)"
gpu_depois="$(nvidia-smi --query-gpu=utilization.gpu,clocks.gr,temperature.gpu --format=csv,noheader 2>/dev/null || echo '?')"

{
  echo "# Oráculo Godot — Luz 2D W5e (Bloco C). Fixture gerada por docs/Luz2D/oraculo_godot/corre.sh"
  echo "# data: $(date -Iseconds)"
  echo "# godot: $(godot --version 2>/dev/null | tail -1) · pacote $(LC_ALL=C pacman -Q godot 2>/dev/null) · licença MIT"
  echo "# gpu (nvidia-smi, antes): $gpu_antes"
  echo "# gpu (nvidia-smi, depois: util, clock, temp): $gpu_depois"
  echo "# loadavg antes:  $carga_antes"
  echo "# loadavg depois: $carga_depois"
  echo "#   ⚠️ acima de load ~5 o relógio da CPU não vale (CLAUDE.md §5.0); o tempo de GPU sofre menos, mas registe-se."
  echo "# comando: cd $RAIZ && PH2D_GPU=1 PH2D_PRAZO=1200 bash scripts/ph2d-run.sh ${CMD[*]}"
  echo "# exit: $rc"
  echo "# Cenas (oraculo.gd, semente 20261011, tudo numa SubViewport UPDATE_ALWAYS; 1920x1080 salvo S7):"
  echo "#   S1 retângulo de tela cheia + sprite 128 px (com oclusor) + 1 PointLight2D raio 512 px"
  echo "#   S2 N sprites emissivos (unshaded) + N PointLight2D raio 48 px, posições uniformes, sem oclusores"
  echo "#   S3 1 PointLight2D com sombra (cobre a tela) + K oclusores retangulares 8–32 px rodados"
  echo "#   S6 S2 a N=100 com Camera2D em pan subpixel (+0.37, +0.23) px/quadro — parede_media é o quadro médio"
  echo "#   S7 S2 a N=100 em 720p/1080p/1440p/2160p (size_2d_override_stretch: a mesma cena escalada)"
  echo "# Tempo: viewport_get_measured_render_time_gpu/cpu da SubViewport, ms; MÍNIMO sobre rodadas×quadros"
  echo "#   (rodadas intercaladas com início rodado); gpu_min_por_rodada mostra a banda."
  echo "# PROVA: grelha 100×100 de células de 40 px, N luzes raio 16 px, uma por célula; 'acesas' = células"
  echo "#   cujo pixel central sobe acima do fundo; 'sombra_efectiva' = acesas cujo pixel a +10 px, atrás de um muro"
  echo "#   (oclusor 2×12 px a +5 px), fica no fundo. Fundo = um retângulo só, ou um retângulo por célula."
  echo "#"
  grep -E '^(META|COLUNAS|PROVA|MEDIDA|FIM)\|' "$CRUA"
  echo "COLUNAS|ERROS|chave|linhas_ERROR_do_godot_no_stderr (somadas nas rodadas; >0 = a config NÃO rendeu o que se pediu)"
  awk '/^MARCA\|/{split($0,f,"|"); t[f[2]]+=n; if(!(f[2] in o)){o[f[2]]=++k; ord[k]=f[2]} n=0; next}
       /^ERROR:/{n++}
       END{for(i=1;i<=k;i++) if(t[ord[i]]>0) printf "ERROS|%s|%d\n", ord[i], t[ord[i]]; if(n>0) printf "ERROS|depois_da_ultima_marca|%d\n", n}' "$CRUA.err"
  if ! grep -q '^FIM|ok' "$CRUA"; then
    echo "# ⛔ a corrida NÃO terminou — últimas linhas de stderr:"
    tail -20 "$CRUA.err" | sed 's/^/#   /'
  fi
} >"$SAIDA"
echo "fixture: $SAIDA (exit $rc)"
exit "$rc"

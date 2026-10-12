#!/usr/bin/env bash
# Mede a memória (placa + RSS) e o tempo do «Send to board…» de um Flip (`pass::raster::current_frame`)
# e o PNG que o quadro faz dele. Um PROCESSO por variante (o pico de placa não desce entre corridas).
# Toca na placa ⇒ corra SEMPRE pela porta:
#   PH2D_GPU=1 bash scripts/ph2d-run.sh bash docs/MiroClone/ferramentas/memoria_flip/corre.sh
# Saída: docs/MiroClone/ferramentas/memoria_flip/saida_<data>.txt (cabeçalho: GPU, driver, loadavg).
set -uo pipefail
raiz="$(cd "$(dirname "$0")/../../../.." && pwd)"
cd "$raiz"
aqui="docs/MiroClone/ferramentas/memoria_flip"
saida="$aqui/saida_$(date +%F).txt"
bin_de() { # o executável dos testes da lib de uma crate, perfil smoke
  cargo test -p "$1" --lib --profile smoke --no-run --message-format=json 2>/dev/null \
    | python3 -c 'import sys,json
for l in sys.stdin:
    m=json.loads(l)
    if m.get("reason")=="compiler-artifact" and m.get("executable") and m["target"]["kind"]==["lib"]: print(m["executable"])' | tail -1
}
flip="$(bin_de ph2d-app-flip)"; board="$(bin_de ph2d-app-board)"
[ -x "$flip" ] && [ -x "$board" ] || { echo "não compilou" >&2; exit 2; }
{
  echo "# memória do «Send to board…» de um Flip — $(date '+%F %T %z')"
  echo "# commit $(git rev-parse --short HEAD) ($(git branch --show-current)) · perfil smoke (optimizado, sem LTO)"
  echo "# GPU: $(nvidia-smi --query-gpu=name,driver_version,memory.used,memory.total --format=csv,noheader)"
  echo "# CPU: $(nproc) núcleos · loadavg no início: $(cat /proc/loadavg)"
  echo "# cena: N camadas, cada uma a cobrir 10×10 unidades (10 traços de raio 0,5); ecrã simulado 2560×1440"
  echo "# ⚠️ tempos sob carga valem pouco (loadavg ao lado de cada variante); a memória é o que importa"
  for lado in 2048 8192; do
    for n in 1 4 8; do
      echo
      echo "## camadas=$n lado=$lado · loadavg $(cut -d' ' -f1-3 /proc/loadavg)"
      PH2D_MEDE_CAMADAS=$n PH2D_MEDE_LADO=$lado timeout 300 "$flip" --ignored --nocapture --test-threads=1 \
        mede_memoria_do_quadro_actual_do_flip 2>&1 | grep -v '^running\|^test result\|^$'
      echo "exit=${PIPESTATUS[0]}"
    done
  done
  for lado in 2048 8192; do
    echo
    echo "## PNG lado=$lado · loadavg $(cut -d' ' -f1-3 /proc/loadavg)"
    PH2D_MEDE_LADO=$lado timeout 300 "$board" --ignored --nocapture --test-threads=1 \
      mede_png_do_quadro_do_flip 2>&1 | grep -v '^running\|^test result\|^$'
  done
  echo
  echo "# loadavg no fim: $(cat /proc/loadavg)"
} | tee "$saida"

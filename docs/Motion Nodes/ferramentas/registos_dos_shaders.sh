#!/bin/bash
# registos_dos_shaders.sh — os REGISTOS e a OCUPAÇÃO de cada shader do passe de formas na iGPU (RADV),
# sem relógio: `RADV_DEBUG=shaderstats` com os caches desligados (com cache o driver não recompila e
# não imprime nada). Doc 121 §9.10: o ramo do tracejado inline levou o fragmento de `56` a `128` VGPRs
# (`18 → 8` ondas por SIMD) e nenhum gate de pixel o via — a imagem era a mesma.
#
# uso (dentro da worktree):
#   B=$(cargo test -p ph2d-app-motion --lib --release --no-run 2>&1 \
#         | grep -o 'target/release/deps/ph2d_app_motion-[0-9a-f]*' | tail -1)
#   cp "$B" <copia>
#   bash "docs/Motion Nodes/ferramentas/registos_dos_shaders.sh" <copia> [<outra-copia>…]
# Imprime, por binário, cada shader: estágio · VGPRs · ondas por SIMD · bytes de código · VGPRs em
# scratch. A ordem é a da COMPILAÇÃO (o Vello compila em paralelo e embaralha-se; os nossos vêm
# primeiro, e cada variante do `override TRACEJADO` é um shader à parte).
set -u
RAIZ="$(cd "$(dirname "$0")/../../.." && pwd)"
FILTRO=sonda_relogio_das_estrelas_grandes
[ "$#" -ge 1 ] || { echo "uso: $0 <binario> [<binario>…]" >&2; exit 2; }
for b in "$@"; do
  [ -x "$b" ] || { echo "binario ausente: $b" >&2; exit 2; }
  "$b" --list --ignored 2>/dev/null | grep -q "$FILTRO" \
    || { echo "o filtro '$FILTRO' nao casa nenhum teste de $b" >&2; exit 2; }
  saida=$(mktemp)
  PH2D_GPU=1 PH2D_PRAZO=300 bash "$RAIZ/scripts/ph2d-run.sh" env \
    VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/radeon_icd.json \
    RADV_DEBUG=shaderstats,nocache MESA_SHADER_CACHE_DISABLE=1 \
    "$b" --ignored --nocapture --test-threads=1 "$FILTRO" > "$saida" 2>&1
  echo "== $(basename "$b")"
  # Controlo positivo: sem nenhum bloco de estatísticas, o driver não as imprimiu (outro ICD, cache).
  grep -q '\*\*\* SHADER STATS' "$saida" || { echo "nenhuma estatistica — o RADV nao as imprimiu" >&2; rm -f "$saida"; exit 3; }
  awk '
    /Shader:$/ { estagio = $1 }
    /^\*\*\* SHADER STATS/ { n++; e[n] = estagio }
    /^VGPRs: / { v[n] = $2 }
    /^Subgroups per SIMD: / { o[n] = $4 }
    /^Code size: / { c[n] = $3 }
    /^Spilled VGPRs: / { s[n] = $3 }
    END { for (i = 1; i <= n; i++) printf "%3d %-8s VGPRs %4s · ondas %3s · codigo %6s B · scratch %s\n", i - 1, e[i], v[i], o[i], c[i], s[i] }
  ' "$saida"
  grep 'copias: placa' "$saida"
  rm -f "$saida"
done

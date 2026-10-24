#!/usr/bin/env bash
# Corre um comando SÓ na Radeon (RADV/radeonsi), sem pedir a vez da NVIDIA.
# ⚠️ A pasta `vez` falsa de antes (um `pronto` e um porteiro `sleep`) pendurava o cliente desde
# a W2 da velocidade (`9e7a70cd61`): o cliente espera o SEU `vez.<pid>`, que ninguém criava.
# Agora: sem `PH2D_GPU_VEZ` (o cliente não pede nada) e a fila da integrada segura-se aqui,
# pelo cadeado da fila de testes do `ph2d-run.sh` (exclusivo com os testes que a pedem).
set -euo pipefail
export VK_DRIVER_FILES=/usr/share/vulkan/icd.d/radeon_icd.json
export VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/radeon_icd.json
export __EGL_VENDOR_LIBRARY_FILENAMES=/usr/share/glvnd/egl_vendor.d/50_mesa.json
export __GLX_VENDOR_LIBRARY_NAME=mesa
unset PH2D_GPU_VEZ
fecho="${PH2D_GPU_FECHADURA_TESTES:-${XDG_RUNTIME_DIR:-/tmp}/ph2d-gpu-testes.lock}"
exec 9>"$fecho"
flock -w "${PH2D_GPU_ESPERA:-1200}" 9 || { echo "radeon.sh: a fila da integrada não chegou" >&2; exit 75; }
"$@" 9>&-   # o cadeado é do shell; nenhum filho (o `sccache`) o herda

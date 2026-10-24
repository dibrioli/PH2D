#!/usr/bin/env bash
# Corre um comando SÓ na Radeon (RADV/radeonsi), sem pedir a vez da NVIDIA:
# a pasta `vez` falsa só existe junto com as variáveis que escondem a NVIDIA.
set -euo pipefail
export VK_DRIVER_FILES=/usr/share/vulkan/icd.d/radeon_icd.json
export VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/radeon_icd.json
export __EGL_VENDOR_LIBRARY_FILENAMES=/usr/share/glvnd/egl_vendor.d/50_mesa.json
export __GLX_VENDOR_LIBRARY_NAME=mesa
vez="$(mktemp -d "${XDG_RUNTIME_DIR:-/tmp}/ph2d-vez-radeon.XXXXXX")"
mkfifo "$vez/porta"
sleep 86400 & guarda=$!
echo "$guarda" > "$vez/porteiro"
: > "$vez/pronto"
trap 'kill $guarda 2>/dev/null; rm -rf "$vez"' EXIT
export PH2D_GPU_VEZ="$vez"
"$@"

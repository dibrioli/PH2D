#!/bin/bash
# F38: UMA rodada — (1) a pegada na Radeon onboard, F36 × F37 × F38, cenas 1 e 41, placa livre e cheia;
# (2) os blocos do alocador (PH2D_GPU_BLOCOS=uso × desempenho, o MESMO binário) com o rabisco automático
# do Composite, nas duas placas, e a sonda `blocos_do_alocador` (texturas 4K e lotes de dabs).
# uso: PH2D_GPU=1 bash scripts/ph2d-run.sh bash <este> <bin_dir com f36/ f37/ f38/> <saida_dir> [prova]
set -euo pipefail
BIN="${1:?bin}"
OUT="${2:?saida}"
MODO="${3:-tudo}"
RAIZ=/home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX
mkdir -p "$OUT/cfg" "$OUT/home"
CACHE="${XDG_CACHE_HOME:-$HOME/.cache}/ph2d-fotografa"
mkdir -p "$CACHE"
# A Radeon: o carregador do Vulkan só vê o ICD dela, e o EGL só o Mesa.
RADEON="VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/radeon_icd.json VK_DRIVER_FILES=/usr/share/vulkan/icd.d/radeon_icd.json __EGL_VENDOR_LIBRARY_FILENAMES=/usr/share/glvnd/egl_vendor.d/50_mesa.json"

# A fila de testes vive na Radeon: a rodada pede-lhe a vez (a das outras árvores espera), além da principal.
if [ -n "${PH2D_GPU_VEZ:-}" ] && [ -p "$PH2D_GPU_VEZ/porta-testes" ]; then
  echo "$$ rodada-F38" > "$PH2D_GPU_VEZ/porta-testes"
  until [ -e "$PH2D_GPU_VEZ/vez-testes.$$" ]; do sleep 0.5; done
  echo "vez da fila de testes (Radeon): sim" > "$OUT/log"
else
  echo "vez da fila de testes (Radeon): SEM porteiro — medição exposta a testes de outras árvores" > "$OUT/log"
fi

# (2b) A sonda dos blocos, fora da sessão gráfica: NVIDIA e Radeon.
if [ "$MODO" = tudo ]; then
  "$RAIZ/target/smoke/examples/blocos_do_alocador" 12 > "$OUT/sonda_nvidia.log" 2>&1 || true
  env $RADEON "$RAIZ/target/smoke/examples/blocos_do_alocador" 12 > "$OUT/sonda_radeon.log" 2>&1 || true
fi

cat > "$OUT/cfg/kwinrulesrc" <<'EOF'
[General]
count=1
rules=r1

[r1]
Description=maximiza tudo
wmclassmatch=0
types=1
maximizehoriz=true
maximizehorizrule=3
maximizevert=true
maximizevertrule=3
EOF
cat > "$OUT/sessao.sh" <<EOF
#!/bin/bash
if [ -z "\$DISPLAY" ] || [ "\$DISPLAY" = ":0" ] || [ "\$DISPLAY" = "${DISPLAY:-:0}" ]; then
  echo "RECUSA: nao e' a tela virtual" >> "$OUT/log"; exit 3
fi
cd "$RAIZ"
janela() {
  for w in \$(xprop -display "\$DISPLAY" -root _NET_CLIENT_LIST | grep -o '0x[0-9a-f]*'); do
    xprop -display "\$DISPLAY" -id "\$w" _NET_WM_PID | grep -qx "_NET_WM_PID(CARDINAL) = \$1" && echo "\$w"
  done | tail -1
}
# MiB do processo na Radeon (amdgpu fdinfo, um cliente por drm-client-id) e na NVIDIA.
mede_amd() {
  for f in /proc/\$1/fdinfo/*; do
    awk '/^drm-pdev:/{p=\$2} /^drm-client-id:/{c=\$2} /^drm-memory-vram:/{v=\$2} /^drm-memory-gtt:/{g=\$2} /^amd-evicted-vram:/{e=\$2}
         END{if (p=="0000:0e:00.0") print c, v, g, e}' "\$f" 2>/dev/null
  done | sort -u | awk '{v+=\$2; g+=\$3; e+=\$4} END{printf "vram=%d gtt=%d evicted=%d", v/1024, g/1024, e/1024}'
}
mede_nv() {
  nvidia-smi --query-compute-apps=pid,used_memory --format=csv,noheader,nounits \
    | awk -F', *' -v p="\$1" '\$1 == p { print \$2 }'
}
vram_livre() { echo \$(( (\$(cat /sys/class/drm/card0/device/mem_info_vram_total) - \$(cat /sys/class/drm/card0/device/mem_info_vram_used)) >> 20 )); }
# corre <placa nv|amd> <bin> <cena> <tag> <segundos> [env extra...]
corre() {
  local placa=\$1 v=\$2 c=\$3 tag=\$4 s=\$5; shift 5
  local pe="" cena=""; [ "\$placa" = amd ] && pe="$RADEON"; [ "\$c" != 0 ] && cena="PH2D_OBJECT_MODE_SMOKE=\$c"
  env -u WAYLAND_DISPLAY HOME="$OUT/home" __GL_SHADER_DISK_CACHE_SIZE=17179869184 \
    __GL_SHADER_DISK_CACHE_PATH="$CACHE" MESA_SHADER_CACHE_DIR="$CACHE/mesa" \$pe \
    \$cena PH2D_GPU_ERROS_TODOS=1 PH2D_GPU_PEGADA=12 PH2D_EXIT_AFTER_FRAMES=100000 "\$@" \
    "$BIN/\$v/ph2d-host-desktop" > "$OUT/app.\$tag.log" 2>&1 &
  APP=\$!
  sleep 12; local a12=\$(mede_amd \$APP) n12=\$(mede_nv \$APP)
  sleep \$((s - 12)); local a=\$(mede_amd \$APP) n=\$(mede_nv \$APP)
  local W=\$(janela \$APP)
  [ -n "\$W" ] && import -display "\$DISPLAY" -window "\$W" "$OUT/foto.\$tag.png" 2>/dev/null
  echo "\$tag | 12s amd[\$a12] nv=\$n12 | \${s}s amd[\$a] nv=\$n | livre=\$(vram_livre) | erros=\$(grep -c '^\[gpu-erro\] #' "$OUT/app.\$tag.log") oom=\$(grep -ci 'outofmemory\|out of memory' "$OUT/app.\$tag.log") | adaptador=\$(grep -m1 -o 'AMD[^,)]*\|NVIDIA[^,)]*' "$OUT/app.\$tag.log")" >> "$OUT/log"
  kill -9 \$APP 2>/dev/null; wait \$APP 2>/dev/null
  sleep 2
}
echo "livre antes: \$(vram_livre) MiB" >> "$OUT/log"
if [ "$MODO" = prova ]; then
  corre amd f38 1 prova.amd.f38.c1 15
  exit 0
fi
if [ "\$PARTE" = amd ]; then
# (1a) Radeon, placa livre: as três versões, cenas 41 e 1, intercaladas e rodadas.
for p in f36:41 f37:41 f38:41 f38:1 f37:1 f36:1 f37:41 f38:41 f36:41 f36:1 f38:1 f37:1; do
  corre amd \${p%%:*} \${p##*:} livre.amd.\${p%%:*}.c\${p##*:}.\$SECONDS 20
done
# (1b) Radeon cheia: a ~950 MiB livres (o caso do dono na NVIDIA) e a ~300 (menos do que a cena 41 pede).
for alvo in 950 300; do
  env $RADEON "$RAIZ/target/smoke/examples/ocupa_a_placa" \$alvo 200 > "$OUT/ocupa.\$alvo.log" 2>&1 &
  OC=\$!
  until grep -q seguro "$OUT/ocupa.\$alvo.log" || ! kill -0 \$OC 2>/dev/null; do sleep 1; done
  echo "ocupa \$alvo: \$(tail -1 "$OUT/ocupa.\$alvo.log")" >> "$OUT/log"
  for p in f36:41 f37:41 f38:41 f38:1; do
    corre amd \${p%%:*} \${p##*:} cheia\$alvo.amd.\${p%%:*}.c\${p##*:}.\$SECONDS 25
  done
  kill \$OC 2>/dev/null; wait \$OC 2>/dev/null
  sleep 2
done
fi
# (2a) Blocos × rabisco do Composite, o MESMO binário (F38), na placa desta sessão, intercalado.
for b in uso desempenho desempenho uso uso desempenho; do
  pl=\$PARTE
  corre \$pl f38 0 rabisco.\$pl.\$b.\$SECONDS 30 PH2D_COMPOSITE_SMOKE=1 PH2D_COMPOSITE_RABISCO=0.6 \
    PH2D_PAINT_PERF=1 PH2D_FLUID_PROFILE=1 PH2D_GPU_BLOCOS=\$b
done
EOF
chmod +x "$OUT/sessao.sh"
bash "$RAIZ/scripts/vez-da-placa.sh" || true
# Uma sessão por placa: a Radeon só apresenta numa tela virtual que renderiza NELA (medido: na do kwin da
# NVIDIA o `Surface::configure` dá «Invalid surface» e o app cai), e vice-versa.
for PARTE in amd nv; do
  [ "$MODO" = prova ] && [ "$PARTE" = nv ] && break
  KW=""; [ "$PARTE" = amd ] && KW="$RADEON DRI_PRIME=pci-0000_0e_00_0 KWIN_DRM_DEVICES=/dev/dri/card0"
  env $KW PARTE=$PARTE XDG_CONFIG_HOME="$OUT/cfg" timeout 1500 kwin_wayland --virtual --width 1930 --height 1040 \
    --xwayland --exit-with-session "$OUT/sessao.sh" > "$OUT/kwin.$PARTE.log" 2>&1 || true
done
cat "$OUT/log"

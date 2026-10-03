#!/usr/bin/env bash
# Prova de mutação das CAMADAS EM TONS DE ECRÃ — ADR-0177 / doc Painter 45 (P0–P2): o decode e o
# encode do compositor de referência, a fronteira luz↔ecrã dos ajustes, o des-premultiplicar da
# aquarela, e o gémeo na placa (a tabela do decode, o encode final, a fronteira por píxel e a do
# grafo de vizinhança).
#
# Corre-se pela fatia da linha e com a placa (a corrida `placa` pede-a):
#   PH2D_GPU=1 bash scripts/ph2d-run.sh bash docs/Painter/ferramentas/muta_as_camadas_em_ecra.sh
# Pré-voo das âncoras (zero testes): MUTA_SO_ANCORAS=1 bash <este ficheiro>
#
# ⚠️ O arnês CONTROLA-SE A SI MESMO (o idioma dos irmãos): a âncora casa EXACTAMENTE uma vez, a
# mutação COMPILA, e a corrida corre N > 0 testes (`passed + failed`). Cada corrida tem o seu
# CONTROLO verde antes.
set -u
SO_ANCORAS="${MUTA_SO_ANCORAS:-}"
SRCS=(crates/ph2d-tool-painter/src crates/ph2d-render/src)
BK=$(mktemp -d)
for s in "${SRCS[@]}"; do mkdir -p "$BK/$s"; cp -r "$s/." "$BK/$s/"; done
restore() {
  for s in "${SRCS[@]}"; do rm -rf "$s"; mkdir -p "$s"; cp -r "$BK/$s/." "$s/"; find "$s" -type f -exec touch {} +; done
}
# ⚠️ Também no INT/TERM: a fatia da linha mata por prazo, e um arnês morto a meio deixaria o src MUTADO.
trap restore EXIT
trap 'exit 130' INT TERM

corre() {
  case "$1" in
    cpu) cargo test -p ph2d-tool-painter --lib -- compositor:: o_traco_numa_camada_nova watercolor_ground_is_the_real_backdrop 2>&1 ;;
    tabela) cargo test -p ph2d-render --lib -- decode_lut_is_the_cpu_decode 2>&1 ;;
    placa) cargo test -p ph2d-render --test it -- --ignored --test-threads=1 layer_compositor_gpu 2>&1 ;;
  esac
}
contados() { grep -oP 'test result: \w+\. \K[0-9]+(?= passed)|[0-9]+(?= failed)' | awk '{s+=$1}END{print s+0}'; }

if [ -z "$SO_ANCORAS" ]; then
  for c in cpu tabela placa; do
    out=$(corre "$c"); rc=$?
    n=$(echo "$out" | contados)
    echo "CONTROLO [$c]: rc=$rc, $n testes"
    [ "$rc" -eq 0 ] && [ "$n" -gt 0 ] || { echo "ABORTO: o controlo [$c] não está verde"; exit 2; }
  done
fi

sangram=0; total=0
muta() { # ficheiro  agulha  substituto  nome  corrida
  local f="$1" agulha="$2" subst="$3" nome="$4" c="$5"
  total=$((total+1))
  local n; n=$(python3 -c 'import sys;print(open(sys.argv[1]).read().count(sys.argv[2]))' "$f" "$agulha")
  if [ -n "$SO_ANCORAS" ]; then
    if [ "$n" -ne 1 ]; then echo "  ✗ ANCORA [$nome]: casou $n vezes (esperado 1)"; else sangram=$((sangram+1)); fi
    return
  fi
  if [ "$n" -ne 1 ]; then echo "  ABORTO [$nome]: a âncora casou $n vezes (esperado 1)"; return; fi
  python3 - "$f" "$agulha" "$subst" <<'PY'
import sys
p,a,b = sys.argv[1], sys.argv[2], sys.argv[3]
s = open(p).read()
assert s.count(a) == 1, (p, s.count(a))
open(p,'w').write(s.replace(a, b, 1))
PY
  touch "$f"
  local out rc corridos
  out=$(corre "$c"); rc=$?
  if echo "$out" | grep -q '^error\[\|^error: could not compile'; then
    echo "  ABORTO [$nome]: a mutação não compila"
  else
    corridos=$(echo "$out" | contados)
    if [ "$corridos" -eq 0 ]; then echo "  ABORTO [$nome]: zero testes correram"
    elif [ $rc -ne 0 ]; then echo "  SANGRA  [$nome]"; sangram=$((sangram+1))
    else echo "  SOBREVIVE [$nome]  <<<<"; fi
  fi
  restore
}

P=crates/ph2d-tool-painter/src
R=crates/ph2d-render/src

# ── CPU, o compositor de referência ──
muta $P/compositor/mod.rs '    b as f32 / 255.0' '    ph2d_color::srgb::srgb_to_linear_byte(b)' \
  'C1 o decode volta à luz' cpu
muta $P/compositor/mod.rs '(v.clamp(0.0, 1.0) * 255.0).round() as u8' '(v.clamp(0.0, 1.0) * 255.0) as u8' \
  'C2 o encode trunca' cpu
muta $P/compositor/compose.rs '    [l(p[0]), l(p[1]), l(p[2]), p[3]]' '    let _ = l; p' \
  'C3 o ajuste recebe o acumulador codificado como se fosse luz' cpu
muta $P/compositor/compose.rs '    [e(p[0]), e(p[1]), e(p[2]), p[3]]' '    let _ = e; p' \
  'C4 o ajuste devolve luz ao acumulador codificado' cpu
muta $P/tool/paint/watercolor_render.rs 'let l = (app - ground_enc[c] * (1.0 - out_a)) * inv_a;' 'let l = app;' \
  'C5 a aquarela numa camada não se des-premultiplica' cpu
muta $P/tool/paint/watercolor_render.rs 'let ground_enc = ground_enc.map(|g| g / 255.0);' 'let ground_enc = ground_enc.map(|g| g / 255.0 * 0.0 + ph2d_color::srgb::srgb_to_linear_unit(g / 255.0));' \
  'C6 a aquarela des-premultiplica sobre o chão em luz' cpu

# ── a placa ──
muta $R/layer_compositor/mod.rs 'core::array::from_fn(|b| b as f32 / 255.0)' 'core::array::from_fn(|b| ph2d_color::srgb::srgb_to_linear_byte(b as u8))' \
  'G1 a tabela do decode volta à luz' tabela
muta $R/shaders/layer_composite.wgsl 'vec4<f32>(1.0)) * 255.0 + 0.5) / 255.0;' 'vec4<f32>(1.0)) * 255.0) / 255.0;' \
  'G2 o encode final trunca' placa
muta $R/shaders/layer_composite.wgsl '    let lin = em_luz(acc);' '    let lin = acc;' \
  'G3 o ajuste por píxel recebe o codificado como luz' placa
muta $R/shaders/layer_composite.wgsl '    let adj_rgb = em_tons_de_ecra(vec4<f32>(apply_adjustment(ap, lin.rgb, coord), acc.a)).rgb;' '    let adj_rgb = apply_adjustment(ap, lin.rgb, coord);' \
  'G4 o ajuste por píxel devolve luz' placa
muta $R/shaders/layer_composite.wgsl '        let l = em_luz(t);' '        let l = t;' \
  'G5 o 1.º passe do blur lê o codificado como luz' placa
muta $R/shaders/layer_composite.wgsl '    var result = em_tons_de_ecra(adj);' '    var result = adj;' \
  'G6 o combine devolve luz' placa
muta $R/shaders/layer_composite.wgsl '    let acc = em_luz(acc_enc); // the kernel'"'"'s space' '    let acc = acc_enc;' \
  'G7 o combine calcula o kernel sobre o codificado' placa
muta $R/shaders/layer_composite.wgsl '    let tr = em_luz(textureLoad(chroma_src, sr, 0));' '    let tr = textureLoad(chroma_src, sr, 0);' \
  'G8 o chroma lê o vermelho codificado como luz' placa
muta $R/shaders/layer_composite.wgsl '    let base = em_luz(textureLoad(bloom_base, p, 0)); // straight linear RGBA' '    let base = textureLoad(bloom_base, p, 0);' \
  'G9 o bright-pass do bloom lê o codificado como luz' placa
muta $R/shaders/layer_composite.wgsl '    let base = em_luz(textureLoad(sh_base, p, 0));' '    let base = textureLoad(sh_base, p, 0);' \
  'G10 a luma do S/H lê o codificado como luz' placa
muta $R/shaders/layer_composite.wgsl '    var result_rgb = em_tons_de_ecra(vec4<f32>(corrected, base.a)).rgb;' '    var result_rgb = corrected;' \
  'G11 o combine do S/H devolve luz' placa

echo "RESULTADO: $sangram de $total sangraram${SO_ANCORAS:+ (pré-voo: âncoras únicas)}"

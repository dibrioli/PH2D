#!/usr/bin/env bash
# Prova de mutação dos EFEITOS DE VIZINHANÇA NA PEÇA — W6, `docs/3D/30` §14: as quatro portas
# novas — a RETÍCULA (`ph2d_mesh_colors::difusao`, o calor), o GANCHO (`Neighbourhood` na crate de
# efeitos e no compositor do Painter), a PLACA (`ph2d-render` surface + `surface_heat.wgsl`, e o
# `composto_na_placa`) e a UI (as unidades do raio, o menu da peça, a ponte da ferramenta).
#
# Corre-se pela fatia da linha e com a placa (a corrida `placa` pede-a), com prazo longo:
#   PH2D_GPU=1 PH2D_PRAZO=7200 bash scripts/ph2d-run.sh bash docs/3D/ferramentas/muta_os_efeitos_de_vizinhanca.sh
# Pré-voo das âncoras (zero testes): MUTA_SO_ANCORAS=1 bash <este ficheiro>
#
# ⚠️ O arnês CONTROLA-SE A SI MESMO (o idioma dos irmãos): a âncora casa EXACTAMENTE uma vez, a
# mutação COMPILA, e a corrida corre N > 0 testes (`passed + failed`). Cada corrida tem o seu
# CONTROLO verde antes. ⚠️ Ele RESTAURA o src das crates dele no fim e em INT/TERM: não se edita
# código nem se correm gates enquanto ele corre (docs pode).
set -u
SO_ANCORAS="${MUTA_SO_ANCORAS:-}"
SRCS=(crates/ph2d-mesh-colors/src crates/ph2d-painter-effects/src crates/ph2d-tool-painter/src
  crates/ph2d-render/src crates/ph2d-app-sculpt3d/src crates/ph2d-panel-painter-layers/src)
BK=$(mktemp -d)
for s in "${SRCS[@]}"; do mkdir -p "$BK/$s"; cp -r "$s/." "$BK/$s/"; done
restore() {
  for s in "${SRCS[@]}"; do rm -rf "$s"; mkdir -p "$s"; cp -r "$BK/$s/." "$s/"; find "$s" -type f -exec touch {} +; done
}
trap restore EXIT
trap 'exit 130' INT TERM

# As corridas, por nome.
corre() {
  case "$1" in
    reticula) cargo test -p ph2d-mesh-colors --profile smoke --lib -- difusao 2>&1 ;;
    peca) cargo test -p ph2d-app-sculpt3d --profile smoke --lib -- vizinhanca_da_peca scenes_vizinhanca 2>&1 ;;
    placa) cargo test -p ph2d-app-sculpt3d --profile smoke --lib -- --ignored --test-threads=1 placa_vizinhanca painel::o_desfoque --skip diag_ 2>&1 ;;
    efeitos) cargo test -p ph2d-painter-effects --profile smoke 2>&1 ;;
    painel) cargo test -p ph2d-panel-painter-layers --profile smoke --test it -- seam_peca 2>&1 ;;
    render) cargo test -p ph2d-render --profile smoke --test it -- spatial_weights_parity 2>&1 ;;
  esac
}
contados() { grep -oP 'test result: \w+\. \K[0-9]+(?= passed)|[0-9]+(?= failed)' | awk '{s+=$1}END{print s+0}'; }

if [ -z "$SO_ANCORAS" ]; then
  for c in reticula peca placa efeitos painel render; do
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

M=crates/ph2d-mesh-colors/src
E=crates/ph2d-painter-effects/src/adjustments
T=crates/ph2d-tool-painter/src
R=crates/ph2d-render/src
A=crates/ph2d-app-sculpt3d/src
P=crates/ph2d-panel-painter-layers/src

# ── A RETÍCULA: o laplaciano e o polinómio ──────────────────────────────────────────────────────
muta $M/difusao.rs '*w = w.max(0.0);' '*w = *w;' \
  'V1 o peso negativo de uma face obtusa entra' reticula
muta $M/difusao.rs 'massa[i as usize] += dupla / 6.0;' 'massa[i as usize] += dupla / 4.0;' \
  'V2 a massa não é um terço do triângulo' reticula
muta $M/difusao.rs 'soma(ip, iq, 0.5 * cot_r);' 'soma(ip, iq, cot_r);' \
  'V3 um lado leva a cotangente inteira' reticula
muta $M/difusao.rs 'dot(ac, ac) <= dot(bd, bd)' 'dot(ac, ac) >= dot(bd, bd)' \
  'V4 a célula parte-se pela diagonal mais LONGA' peca
muta $M/difusao.rs 'triangulo(a, cc, d);' 'triangulo(b, cc, d);' \
  'V5 a triangulação não é a do grafo' reticula
muta $M/difusao.rs '2.0 * rigidez * self.inv_massa[a]' 'rigidez * self.inv_massa[a]' \
  'V6 o tecto do espectro a metade (o polinómio fora do intervalo)' reticula
muta $M/difusao.rs 'd[0] *= 0.5;' '' \
  'V7 o 1.º coeficiente de Chebyshev inteiro' reticula
muta $M/difusao.rs 'const ERRO_DO_POLINOMIO: f64 = 1e-6;' 'const ERRO_DO_POLINOMIO: f64 = 1e-2;' \
  'V8 o polinómio cortado cedo' reticula
muta $M/difusao.rs 'y.soma(-d[0], *t0);' 'y.soma(d[0], *t0);' \
  'V9 o 1.º termo com o sinal trocado' reticula
muta $M/difusao.rs 'v.soma(2.0, x);' 'v.soma(1.0, x);' \
  'V10 a recorrência de Chebyshev sem o 2' reticula
muta $M/difusao.rs 'T::menos(za, z[self.viz[e] as usize])' 'T::menos(z[self.viz[e] as usize], za)' \
  'V11 o laplaciano com o sinal trocado' reticula

# ── A PEÇA: a vizinhança, a pilha, as portas ────────────────────────────────────────────────────
muta $A/vizinhanca_da_peca.rs $'fn blur4(&self, radius: f32, buf: &mut [[f32; 4]]) {\n        self.difusao.desfoca(gaussian_sigma(radius), buf);' $'fn blur4(&self, radius: f32, buf: &mut [[f32; 4]]) {\n        self.difusao.desfoca(radius, buf);' \
  'P1 σ = o raio (sem o /3)' peca
muta $A/pilha_da_peca_vizinhanca.rs 'let todo = self.com_vizinhos(|nb| composite_over(&self.pilha, self, nb));' 'let todo = ph2d_tool_painter::composite(&self.pilha, self, dobra(self.amostras).0, dobra(self.amostras).1);' \
  'P2 a peça borra pela ordem das amostras (a grelha da dobra)' peca
muta $A/pilha_da_peca_vizinhanca.rs 'Some(LayerKind::Adjustment(a)) if a.kind.reads_the_image_layout()' 'Some(LayerKind::Adjustment(a)) if false && a.kind.reads_the_image_layout()' \
  'P3 a pilha nunca lê vizinhos' peca
muta $A/pilha_da_peca_vizinhanca.rs '.is_some_and(|v| v.serve(&impressao))' '.is_some_and(|_| true)' \
  'P4 a vizinhança nunca se refaz' peca
muta $A/vizinhanca_da_peca.rs 'for p in mesh.positions() {' 'for p in mesh.positions().iter().take(0) {' \
  'P5 a impressão esquece as posições' peca
muta $A/pilha_da_peca.rs 'if kind.reads_the_image_plane() {' 'if kind.reads_the_image_layout() {' \
  'P6 a porta recusa os de vizinhança (a W3)' peca
muta $A/pilha_da_peca.rs 'rescale_spatial_params(&mut a.params, SpatialUnits::Pixels, unidades);' '' \
  'P7 o raio novo nasce em px' peca
muta $A/vizinhanca_da_peca.rs '(kind.reads_the_image_layout() && nivel > NIVEL_MAX_DA_VIZINHANCA)' '(kind.reads_the_image_layout() && nivel >= NIVEL_MAX_DA_VIZINHANCA)' \
  'P8 a recusa já a 64x' peca
muta $A/tinta_da_peca_pilha.rs $'pilha.atrasa(peca);\n        *compor_na_placa = true;' 'pilha.atrasa(peca);' \
  'P9 o traço por baixo de um desfoque não pede a placa' placa
muta $A/tinta_da_peca_pilha.rs $'pilha.em_dia(peca, stack.mesh());\n    if pilha.le_a_vizinhanca() && concorda_com(peca, stack.mesh()) {' $'pilha.em_dia(peca, stack.mesh());\n    if false && pilha.le_a_vizinhanca() && concorda_com(peca, stack.mesh()) {' \
  'P10 o em_dia deixa a cor por vértice velha' peca
muta $A/tinta_da_peca_pilha.rs '&& !p.atrasada()' '&& p.atrasada()' \
  'P11 abrir o ficheiro não acerta a cor por vértice' peca
muta $A/vizinhanca_da_peca.rs 'let size = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();' 'let size = 1.0f32;' \
  'P12 o tamanho da peça não é a diagonal' peca
muta $A/composto_na_placa.rs 'if !pilha.le_a_vizinhanca() {' 'if true {' \
  'P13 a placa nunca instala a vizinhança' placa
muta $A/painter_na_malha_camadas.rs 'painter.sync_piece_units(crate::vizinhanca_da_peca::unidades(o.stack.mesh()));' '' \
  'P14 a ferramenta não recebe as unidades da peça' placa

# ── O GANCHO: a grelha ao bit e a superfície no compositor ─────────────────────────────────────
muta $E/spatial_neighbourhood.rs 'separable_blur_premul(radius, buf, *self);' '' \
  'H1 o passa-baixo da grelha é a identidade' efeitos
muta $E/spatial_neighbourhood.rs $'premultiply(acc);\n    nb.blur4(p.radius, acc);\n    unpremultiply(acc);' 'nb.blur4(p.radius, acc);' \
  'H2 o gaussiano borra a cor DIREITA' efeitos
muta $T/compositor/compose.rs $'Some(n) => {\n                        debug_assert_eq!(n.window(), win, "a surface is composed whole");\n                        n\n                    }' 'Some(_) => &win,' \
  'H3 o compositor ignora a superfície' peca
muta $T/compositor/compose.rs 'if !has_spatial_adjustment(stack, stack.root()) {' 'if true {' \
  'H4 composite_over salta para a grelha' peca
muta $E/spatial_tonal.rs '*o = bilinear(&small, sw, sh, fx, fy);' '*o = small[0];' \
  'H5 a pirâmide do brilho da grelha não interpola' efeitos
muta $E/spatial_tonal.rs 'nb.blur1(p.shadows_radius, &mut local_lo);' 'nb.blur1(p.highlights_radius, &mut local_lo);' \
  'H6 as sombras usam o raio dos realces' placa
muta $E/gpu_codes.rs 'Self::MotionBlur | Self::ChromaticAberration | Self::Halftone' 'Self::MotionBlur | Self::ChromaticAberration' \
  'H7 o Halftone deixa de ler o plano da imagem' peca

# ── A UI: as unidades, o menu, a ponte ──────────────────────────────────────────────────────────
muta $P/peca.rs '!on_piece() || !kind.reads_the_image_plane()' '!on_piece() || !kind.reads_the_image_layout()' \
  'U1 o menu da peça apaga os de vizinhança (a W3)' painel
muta $E/compute/units.rs 'Self::Surface { size } => px_max / SPATIAL_PX_MAX * SURFACE_RADIUS_MAX * size,' 'Self::Surface { size: _ } => px_max / SPATIAL_PX_MAX * SURFACE_RADIUS_MAX,' \
  'U2 o curso da superfície esquece o tamanho' efeitos
muta $E/compute/units.rs 'Self::Surface { .. } => px_max / SPATIAL_PX_MAX * SURFACE_RADIUS_MAX * 100.0,' 'Self::Surface { .. } => px_max,' \
  'U3 o número da superfície em px' efeitos
muta $E/compute/units.rs 'AdjustmentParams::ShadowsHighlights(_) => &[2, 5],' 'AdjustmentParams::ShadowsHighlights(_) => &[2],' \
  'U4 a lista dos slots esquece o raio dos realces' efeitos
muta $T/tool/piece_layers.rs 'apply_param_edit(&mut adj.params, &p, self.piece_units);' 'apply_param_edit(&mut adj.params, &p, ph2d_painter_effects::adjustments::SpatialUnits::Pixels);' \
  'U5 o slider da peça escreve px' placa
muta $T/tool/piece_layers.rs $'            self.piece_units\n        } else {' $'            ph2d_painter_effects::adjustments::SpatialUnits::Pixels\n        } else {' \
  'U6 o painel da peça lê px' placa
muta $P/paint_adjust.rs 'let units = crate::peca::spatial_units();' 'let units = ph2d_tool_painter::SpatialUnits::Pixels;' \
  'U7 o painel pinta o número em px' painel

# ── A PLACA: o grafo, o polinómio em WGSL, os estágios ─────────────────────────────────────────
muta $R/shaders/surface_heat.wgsl 'let x = hp.escala * a - t_cur[i];' 'let x = hp.escala * a + t_cur[i];' \
  'G1 X = escala·A + I' placa
muta $R/shaders/surface_heat.wgsl 'v = 2.0 * x - t_prev[i];' 'v = 2.0 * x + t_prev[i];' \
  'G2 a recorrência soma o anterior' placa
muta $R/shaders/surface_heat.wgsl 'c = vec4<f32>(c.rgb * c.a, c.a);' 'c = c;' \
  'G3 a placa não pré-multiplica ao ler' placa
muta $R/layer_compositor/surface.rs 'k => step_bg(ts[(k - 1) % 3], ts[(k - 2) % 3], ts[k % 3]),' 'k => step_bg(ts[(k - 2) % 3], ts[(k - 1) % 3], ts[k % 3]),' \
  'G4 a rotação dos T troca o actual com o anterior' placa
muta $R/layer_compositor/surface.rs 'uniforms.push(pass(k.min(2) as u32, d));' 'uniforms.push(pass(k.min(1) as u32, d));' \
  'G5 sem a recorrência (todo termo é X·T)' placa
muta $R/layer_compositor/surface.rs 'escala: 2.0 / graph.lambda_sup,' 'escala: 1.0 / graph.lambda_sup,' \
  'G6 o intervalo de Chebyshev errado' placa
muta $R/layer_compositor/compositor/dispatch.rs 'BlurStage::Separable if on_surface => {' 'BlurStage::Separable if false && on_surface => {' \
  'G7 a placa borra a peça pela grelha da dobra' placa
muta $R/layer_compositor/compositor/dispatch.rs 'self.run_bloom_bright(gpu, cur, *threshold, *falloff, work);' '' \
  'G8 o brilho da superfície sem o passo claro' placa
muta $R/layer_compositor/compositor/effects.rs 'self.run_surface_heat(gpu, (WorkSel::Sh(0), WorkSel::Sh(1)), lo, false);' 'self.run_surface_heat(gpu, (WorkSel::Sh(0), WorkSel::Sh(1)), hi, false);' \
  'G9 as sombras da superfície com o raio dos realces' placa
muta $R/layer_compositor/ops.rs 'radius.max(0.0) / 3.0' 'radius.max(0.0) / 2.0' \
  'G10 o σ da placa não é o da CPU' render

if [ -n "$SO_ANCORAS" ]; then
  echo "PRE-VOO: $sangram de $total ancoras casam exactamente uma vez (ZERO testes corridos)"
  [ "$sangram" -eq "$total" ]
else
  echo "RESULTADO: $sangram de $total sangram"
fi

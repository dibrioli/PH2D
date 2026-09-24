#!/usr/bin/env bash
# Prova de mutação do PAINTER NA PEÇA (2026-09-24, etapa 1) — a lei que pousa
# a tela na peça, as quatro guardas do Painter e a costura família+shell.
#
# ⭐ A POPULAÇÃO é POR MUTAÇÃO, e é de quem OBSERVA, nunca de quem contém:
#   · a lei (`ph2d-sculpt3d/tela_na_malha.rs`, `tinta_fina.rs`, `stroke_freeze.rs`)
#     observa-se pelos gates `tela_na_malha` da própria crate;
#   · as guardas do Painter observam-se pelos `screen_canvas` da `ph2d-tool-painter`;
#   · a costura (`painter_na_malha.rs`, `cursor.rs`) e os TRÊS elos da SHELL
#     observam-se pelo censo `painter_fiacao` da família — ele lê a shell por
#     `include_str!`, logo mutar a shell recompila a família e a mutação sangra
#     ali sem uma crate a mais na corrida.
#
# ⚠️ O arnês CONTROLA-SE A SI MESMO nos quatro pontos que já mentiram nesta casa:
#   (a) a âncora casa EXACTAMENTE UMA vez (pré-voo: `MUTA_SO_ANCORAS=1`);
#   (b) a mutação COMPILA — um erro de compilação lê-se como sangrar;
#   (c) a corrida corre N > 0 testes, `passed + failed` do `test result:`
#       (o `running N tests` CONTA os `#[ignore]`);
#   (d) as TRÊS corridas LIMPAS estão verdes antes da primeira mutação.
set -u
SO_ANCORAS="${MUTA_SO_ANCORAS:-}"
# `MUTA_FILTRO=<regex>` corre só as mutações cujo NOME casa — e o sumário DIZ que é parcial:
# *um placar parcial lido como completo é a forma mais barata de um arnês mentir.*
FILTRO="${MUTA_FILTRO:-}"
ROOT=$(git rev-parse --show-toplevel)
cd "$ROOT" || exit 2
BK=$(mktemp -d)
FICHEIROS=(
  crates/ph2d-sculpt3d/src/tela_na_malha.rs
  crates/ph2d-sculpt3d/src/tinta_fina.rs
  crates/ph2d-sculpt3d/src/stroke_freeze.rs
  crates/ph2d-tool-painter/src/tool/runtime.rs
  crates/ph2d-tool-painter/src/tool/layers/preview.rs
  crates/ph2d-tool-painter/src/tool/trait_impls_raster.rs
  crates/ph2d-tool-painter/src/tool/documents.rs
  crates/ph2d-tool-painter/src/tool/screen_canvas.rs
  crates/ph2d-app-sculpt3d/src/painter_na_malha.rs
  crates/ph2d-app-sculpt3d/src/cursor.rs
  shells/desktop/src/sculpt3d_host.rs
  shells/desktop/src/input_dispatch/painter_canvas_input.rs
  shells/desktop/src/render_loop/fase_painter_dispatch.rs
)
for f in "${FICHEIROS[@]}"; do mkdir -p "$BK/$(dirname "$f")"; cp "$f" "$BK/$f"; done
restore() {
  for f in "${FICHEIROS[@]}"; do
    if ! cmp -s "$BK/$f" "$f"; then cp "$BK/$f" "$f"; fi
    # ⚠️ `touch` SEMPRE: um `cp` devolve o conteúdo e o cargo pode guardar o
    #    build da mutação (memória: restaurar por mv/cp deixa o mtime antigo).
    touch "$f"
  done
}
trap restore EXIT

LEI=(cargo test -p ph2d-sculpt3d --lib tela_na_malha)
PINTOR=(cargo test -p ph2d-tool-painter --lib screen_canvas)
COSTURA=(cargo test -p ph2d-app-sculpt3d --lib painter_fiacao)

corridos() { grep -oP 'test result: \w+\. \K[0-9]+(?= passed)|[0-9]+(?= failed)' | awk '{s+=$1}END{print s+0}'; }

if [ -z "$SO_ANCORAS" ]; then
  for pop in LEI PINTOR COSTURA; do
    declare -n cmd="$pop"
    out=$("${cmd[@]}" 2>&1); rc=$?
    n=$(echo "$out" | corridos)
    echo "VERDE antes [$pop]: rc=$rc, $n testes correram"
    [ "$rc" -eq 0 ] && [ "$n" -gt 0 ] || { echo "ABORTO: a corrida limpa [$pop] não está verde"; exit 2; }
  done
fi

sangram=0; total=0; controlos=0; ncontrolos=0
muta() { # populacao  ficheiro  agulha  substituto  nome  [controlo]
  local pop="$1" f="$2" agulha="$3" subst="$4" nome="$5" controlo="${6:-}"
  if [ -n "$FILTRO" ] && ! [[ "$nome" =~ $FILTRO ]]; then return; fi
  total=$((total+1))
  [ -n "$controlo" ] && ncontrolos=$((ncontrolos+1))
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
  declare -n cmd="$pop"
  local out rc corr
  out=$("${cmd[@]}" 2>&1); rc=$?
  if echo "$out" | grep -q '^error\[\|^error: could not compile'; then
    echo "  ABORTO [$nome]: a mutação não compila"
    echo "$out" | grep -m3 '^error' | sed 's/^/      /'
  else
    corr=$(echo "$out" | corridos)
    if [ "$corr" -eq 0 ]; then echo "  ABORTO [$nome]: zero testes correram"
    elif [ -n "$controlo" ]; then
      if [ $rc -eq 0 ]; then echo "  CONTROLO sobrevive, como tem de ser [$nome]"; controlos=$((controlos+1))
      else echo "  ✗ CONTROLO SANGROU [$nome] — o arnês mede outra coisa"; fi
    elif [ $rc -ne 0 ]; then echo "  SANGRA  [$nome]"; sangram=$((sangram+1))
    else echo "  SOBREVIVE [$nome]  <<<<"; fi
  fi
  restore
}

# ── A LEI ──────────────────────────────────────────────────────────────────
muta LEI crates/ph2d-sculpt3d/src/tela_na_malha.rs \
  '.is_none_or(|h| h.t >= dist * (1.0 - FOLGA_DA_OCLUSAO))' \
  '.is_none_or(|_h| true)' \
  'L1 a oclusão deixa de tapar o que está atrás'
muta LEI crates/ph2d-sculpt3d/src/tela_na_malha.rs \
  '    ponto(n, sub(olho, p(0))) > 0.0' \
  '    ponto(n, sub(olho, p(0))) > 0.0 || true' \
  'L2 a face de costas também é pintada'
muta LEI crates/ph2d-sculpt3d/src/tela_na_malha.rs \
  '    let fica = 1.0 - a * k;' \
  '    let fica = 0.0 * a * k;' \
  'L3 a tinta substitui a base em vez de a cobrir'
muta LEI crates/ph2d-sculpt3d/src/tela_na_malha.rs \
  '                    let k = keep_da_amostra(w, &m[..n]);' \
  '                    let k = { let _ = (w, &m[..n]); 1.0 };' \
  'L4 a máscara deixa de proteger a tinta fina'
muta LEI crates/ph2d-sculpt3d/src/tela_na_malha.rs \
  '                    let k = keep_da_amostra(&[1.0], &m[c..=c]);' \
  '                    let k = { let _ = &m[c..=c]; 1.0 };' \
  'L5 a máscara deixa de proteger os vértices'
muta LEI crates/ph2d-sculpt3d/src/tela_na_malha.rs \
  '        if *v == 0 {
            self.raios += 1;' \
  '        if true {
            self.raios += 1;' \
  'L6 a visibilidade deixa de ser lembrada: um raio por quadro'
muta LEI crates/ph2d-sculpt3d/src/tinta_fina.rs \
  '        let nova = cor(self.base[s]);' \
  '        let nova = cor(self.tinta.amostras()[idx as usize]);' \
  'L7 a amostra fina mistura sobre a CORRENTE: pousar duas vezes escurece'
muta LEI crates/ph2d-sculpt3d/src/stroke_freeze.rs \
  '        let nova = cor(self.base_color[self.slot[v as usize] as usize]);' \
  '        let nova = cor(mesh.colors_mut()[v as usize]);' \
  'L8 o vértice mistura sobre a CORRENTE: pousar duas vezes escurece'
muta LEI crates/ph2d-sculpt3d/src/tela_na_malha.rs \
  '            s[0] >= caixa[0] && s[0] <= caixa[2] && s[1] >= caixa[1] && s[1] <= caixa[3]' \
  '            s[0] >= caixa[0] && s[1] >= caixa[1]' \
  'L9 o rectângulo sujo deixa de limitar a pousada'

# ── AS GUARDAS DO PAINTER ─────────────────────────────────────────────────
muta PINTOR crates/ph2d-tool-painter/src/tool/runtime.rs \
  '        if self.on_screen_canvas() {
            return None;
        }' \
  '' \
  'T1 a ponte da sprite drena a tela da vista'
muta PINTOR crates/ph2d-tool-painter/src/tool/layers/preview.rs \
  'if self.canvas_rgba.is_empty() || self.on_screen_canvas() {' \
  'if self.canvas_rgba.is_empty() {' \
  'T2 a pista GPU da sprite vê a tela da vista'
muta PINTOR crates/ph2d-tool-painter/src/tool/trait_impls_raster.rs \
  'std::mem::take(&mut self.pending_commit) && !self.on_screen_canvas()' \
  'std::mem::take(&mut self.pending_commit)' \
  'T3 o Apply assa a vista numa sprite'
muta PINTOR crates/ph2d-tool-painter/src/tool/documents.rs \
  '        if self.on_screen_canvas() {
            return false;
        }' \
  '' \
  'T4 a sprite escolhida desloca a tela a meio do traço'
muta PINTOR crates/ph2d-tool-painter/src/tool/screen_canvas.rs \
  '            let (w, h) = self.source_size;
            self.set_source(transparente(w, h), w, h);' \
  '            let _ = self.source_size;' \
  'T5 a tela não se limpa'
muta PINTOR crates/ph2d-tool-painter/src/tool/screen_canvas.rs \
  '        self.bind_document(
            SCREEN_CANVAS_DOC,' \
  '        self.bound_doc = Some(SCREEN_CANVAS_DOC);
        self.set_source(
            transparente(largura, altura),
            largura,
            altura,
        );
        self.bind_document(
            SCREEN_CANVAS_DOC,' \
  'T6 prender a tela achata a sprite de várias camadas'

# ── A COSTURA, na família e na shell ──────────────────────────────────────
muta COSTURA shells/desktop/src/sculpt3d_host.rs \
  '.is_some_and(|p| p.on_screen_canvas())' \
  '.is_some_and(|_p| false)' \
  'P1 a escultura rouba o botão esquerdo'
muta COSTURA shells/desktop/src/input_dispatch/painter_canvas_input.rs \
  'ph2d_app_sculpt3d::painter_na_malha::entrega(' \
  'ph2d_app_sculpt3d::painter_na_malha::entrega_morta(' \
  'P2 o ponteiro nunca chega à peça (mutação de TEXTO: o censo não compila a shell)'
muta COSTURA shells/desktop/src/render_loop/fase_painter_dispatch.rs \
  'ph2d_app_sculpt3d::painter_na_malha::quadro(' \
  'ph2d_app_sculpt3d::painter_na_malha::quadro_morto(' \
  'P3 a tela nunca é presa (mutação de TEXTO)'
muta COSTURA crates/ph2d-app-sculpt3d/src/painter_na_malha.rs \
  '                s.painter_pousa(&f);' \
  '                let _ = &f;' \
  'P4 o traço só aparece quando o dedo sobe'
muta COSTURA crates/ph2d-app-sculpt3d/src/painter_na_malha.rs \
  '        scene.painter_fecha();' \
  '' \
  'P5 o traço nunca fecha'
muta COSTURA crates/ph2d-app-sculpt3d/src/painter_na_malha.rs \
  '        painter.clear_screen_canvas();' \
  '' \
  'P6 a tela não se limpa depois do traço'
muta COSTURA crates/ph2d-app-sculpt3d/src/painter_na_malha.rs \
  '        None => painter.release_screen_canvas(),' \
  '        None => {}' \
  'P7 sem cena a tela fica presa'
muta COSTURA crates/ph2d-app-sculpt3d/src/cursor.rs \
  '        if let Some(raio) = self.painter_raio_px {' \
  '        if let Some(raio) = None::<f32> {' \
  'P8 o anel do Painter volta a ser o do pincel de escultura'

# ── O CONTROLO — tem de SOBREVIVER ──────────────────────────────────────────
muta COSTURA crates/ph2d-app-sculpt3d/src/painter_na_malha.rs \
  '/// **Pousa na peça o que a tela mudou.**' \
  '/// **Pousa na peça o que a tela mudou** (controlo).' \
  'C1 um comentário muda' controlo

echo
if [ -n "$SO_ANCORAS" ]; then
  echo "PRE-VOO: $sangram de $total ancoras casam exactamente uma vez (ZERO testes corridos)"
  [ "$sangram" -eq "$total" ]
  exit $?
fi
[ -n "$FILTRO" ] && echo "⚠️ PARCIAL — só as mutações que casam /$FILTRO/"
echo "MUTACAO: $sangram de $((total-ncontrolos)) sangram · controlo a sobreviver: $controlos de $ncontrolos"
[ "$sangram" -eq $((total-ncontrolos)) ] && [ "$controlos" -eq "$ncontrolos" ]

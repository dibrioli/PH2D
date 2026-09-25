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
#   (d) as corridas LIMPAS estão verdes antes da primeira mutação;
#   (e) a população de PRODUTO (a aquarela molhada, 24/09) é um gate de PLACA:
#       sem adaptador o `gpu_or_skip!` devolve cedo e o teste PASSA — lido como
#       «a mutação sobreviveu». Ela só corre com `MUTA_PRODUTO=1` (e com
#       `PH2D_GPU=1` na porta), e o arnês ABORTA se a saída disser que não há
#       placa.
set -u
SO_ANCORAS="${MUTA_SO_ANCORAS:-}"
# `MUTA_FILTRO=<regex>` corre só as mutações cujo NOME casa — e o sumário DIZ que é parcial:
# *um placar parcial lido como completo é a forma mais barata de um arnês mentir.*
FILTRO="${MUTA_FILTRO:-}"
COM_PRODUTO="${MUTA_PRODUTO:-}"
ROOT=$(git rev-parse --show-toplevel)
cd "$ROOT" || exit 2
BK=$(mktemp -d)
FICHEIROS=(
  crates/ph2d-sculpt3d/src/tela_na_malha.rs
  crates/ph2d-sculpt3d/src/tela_na_malha_pousa.rs
  crates/ph2d-sculpt3d/src/tela_semente.rs
  crates/ph2d-tool-painter/src/tool/paint/mode_switch.rs
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

LEI=(cargo test -p ph2d-sculpt3d --lib tela_)
PINTOR=(cargo test -p ph2d-tool-painter --lib screen_canvas)
COSTURA=(cargo test -p ph2d-app-sculpt3d --lib painter_fiacao)
PRODUTO=(cargo test -p ph2d-app-sculpt3d --lib
  tinta_no_produto_tests::painter::a_aquarela -- --ignored --nocapture)
SEM_PLACA='no GPU adapter'

corridos() { grep -oP 'test result: \w+\. \K[0-9]+(?= passed)|[0-9]+(?= failed)' | awk '{s+=$1}END{print s+0}'; }

if [ -z "$SO_ANCORAS" ]; then
  pops=(LEI PINTOR COSTURA)
  [ -n "$COM_PRODUTO" ] && pops+=(PRODUTO)
  for pop in "${pops[@]}"; do
    declare -n cmd="$pop"
    out=$("${cmd[@]}" 2>&1); rc=$?
    if echo "$out" | grep -q "$SEM_PLACA"; then echo "ABORTO: [$pop] correu sem placa"; exit 2; fi
    n=$(echo "$out" | corridos)
    echo "VERDE antes [$pop]: rc=$rc, $n testes correram"
    [ "$rc" -eq 0 ] && [ "$n" -gt 0 ] || { echo "ABORTO: a corrida limpa [$pop] não está verde"; exit 2; }
  done
fi

sangram=0; total=0; controlos=0; ncontrolos=0
muta() { # populacao  ficheiro  agulha  substituto  nome  [controlo]
  local pop="$1" f="$2" agulha="$3" subst="$4" nome="$5" controlo="${6:-}"
  if [ -n "$FILTRO" ] && ! [[ "$nome" =~ $FILTRO ]]; then return; fi
  if [ "$pop" = PRODUTO ] && [ -z "$COM_PRODUTO" ] && [ -z "$SO_ANCORAS" ]; then return; fi
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
  if echo "$out" | grep -q "$SEM_PLACA"; then
    echo "  ABORTO [$nome]: correu sem placa — um gate que salta lê-se como sobreviver"
  elif echo "$out" | grep -q '^error\[\|^error: could not compile'; then
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
muta LEI crates/ph2d-sculpt3d/src/tela_na_malha_pousa.rs \
  '            let fica = 1.0 - a * k;' \
  '            let fica = 0.0 * a * k;' \
  'L3 a tinta substitui a base em vez de a cobrir'
muta LEI crates/ph2d-sculpt3d/src/tela_na_malha_pousa.rs \
  '    let k = keep_da_amostra(w, m);' \
  '    let k = { let _ = (w, m); 1.0 };' \
  'L4 a máscara deixa de proteger a tinta fina'
muta LEI crates/ph2d-sculpt3d/src/tela_na_malha_pousa.rs \
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
muta LEI crates/ph2d-sculpt3d/src/tela_na_malha_pousa.rs \
  '    s[0] >= caixa[0] && s[0] <= caixa[2] && s[1] >= caixa[1] && s[1] <= caixa[3]' \
  '    s[0] >= caixa[0] && s[1] >= caixa[1]' \
  'L9 o rectângulo sujo deixa de limitar a pousada'

# ── O PREÇO a 256x (24/09): a oclusão por (face, píxel) e os blocos da retícula.
#    Os dois atalhos pintam o MESMO que o caminho sem eles; o gate dos blocos e o
#    do contador existem porque, sem eles, desligar os blocos era INOBSERVÁVEL.
muta LEI crates/ph2d-sculpt3d/src/tela_na_malha_pousa.rs \
  'b[2] >= caixa[0]' \
  'b[0] >= caixa[0]' \
  'L13 a caixa do bloco mede o INÍCIO e deixa amostras por pintar'
muta LEI crates/ph2d-sculpt3d/src/tela_na_malha.rs \
  '.then(|| y as usize * w + x as usize);' \
  '.then(|| y as usize * w + x as usize).filter(|_| false);' \
  'L14 a oclusão volta a ser um raio por AMOSTRA'
muta LEI crates/ph2d-sculpt3d/src/tela_na_malha.rs \
  'let chave = face.wrapping_add(1) & !VEREDITO;' \
  'let chave = 1u32;' \
  'L15 a chave do píxel esquece a FACE: a placa decide pelo que está atrás'
muta LEI crates/ph2d-sculpt3d/src/tela_na_malha_pousa.rs \
  '.is_none_or(|b| {' \
  '.is_none_or(|b| true || {' \
  'L16 nenhum bloco é cortado: um traço pequeno projecta a face inteira'
muta LEI crates/ph2d-sculpt3d/src/tela_na_malha_pousa.rs \
  'sessao.projetadas += 1;' \
  'sessao.projetadas += 0;' \
  'L17 o contador de projecção fica mudo'

# ── ETAPA 2: o retrato e a lei da DIFERENÇA ──────────────────────────────
muta LEI crates/ph2d-sculpt3d/src/tela_na_malha_pousa.rs \
  '            (base[0] + d[0] * k).clamp(0.0, 1.0),' \
  '            base[0].clamp(0.0, 1.0),' \
  'L10 a diferença deixa de chegar ao canal vermelho'
muta LEI crates/ph2d-sculpt3d/src/tela_na_malha.rs \
  '        (Mistura::Diferenca(d), d == [0.0; 3])' \
  '        (Mistura::Diferenca(d), false)' \
  'L11 o intacto deixa de sair cedo: um raio por amostra da peça'
muta LEI crates/ph2d-sculpt3d/src/tela_na_malha.rs \
  '        if a <= COBERTURA_MINIMA || sa <= COBERTURA_MINIMA {' \
  '        if false {' \
  'L12 um píxel apagado divide por zero'
muta LEI crates/ph2d-sculpt3d/src/tela_semente.rs \
  '            if inv <= perto[o] {' \
  '            if false {' \
  'S1 o retrato mostra a última face desenhada e não a mais perto'
muta LEI crates/ph2d-sculpt3d/src/tela_semente.rs \
  '        if !de_frente(pos, cantos, vista.olho()) {' \
  '        if false {' \
  'S2 as costas entram no retrato'
muta LEI crates/ph2d-sculpt3d/src/tela_semente.rs \
  '                l[0] * ps[0].1 / inv,' \
  '                l[0],' \
  'S3 o retrato interpola no ecrã e não com perspectiva'
muta PINTOR crates/ph2d-tool-painter/src/tool/paint/mode_switch.rs \
  '        !simples' \
  '        false' \
  'T7 nenhum modo lê a peça: o borrão borra o vazio'
muta PINTOR crates/ph2d-tool-painter/src/tool/screen_canvas.rs \
  '        self.set_source(rgba, w, h);
        true' \
  '        let _ = rgba;
        true' \
  'T8 semear diz que semeou e a tela fica como estava'

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
  '            _ => {
                painter.clear_screen_canvas();
            }' \
  '            _ => {}' \
  'P6 a tela não se limpa depois do traço'
muta COSTURA crates/ph2d-app-sculpt3d/src/painter_na_malha.rs \
  '        None => painter.release_screen_canvas(),' \
  '        None => {}' \
  'P7 sem cena a tela fica presa'
muta COSTURA crates/ph2d-app-sculpt3d/src/cursor.rs \
  '        if let Some(raio) = self.painter_raio_px {' \
  '        if let Some(raio) = None::<f32> {' \
  'P8 o anel do Painter volta a ser o do pincel de escultura'

muta COSTURA crates/ph2d-app-sculpt3d/src/painter_na_malha.rs \
  '        if painter.screen_canvas_reads_the_piece() {' \
  '        if false {' \
  'P9 a costura nunca semeia a tela'
muta COSTURA crates/ph2d-app-sculpt3d/src/painter_na_malha.rs \
  '            sessao.com_semente(retrato.clone());' \
  '            let _ = retrato.clone();' \
  'P10 a sessão não recebe o retrato: a lei continua o «over»'
muta COSTURA crates/ph2d-app-sculpt3d/src/painter_na_malha.rs \
  '            let _ = painter.take_screen_canvas();
            sessao.com_semente(retrato.clone());' \
  '            sessao.com_semente(retrato.clone());' \
  'P11 o retrato é pousado como mudança'
muta COSTURA shells/desktop/src/input_dispatch/painter_canvas_input.rs \
  '            super::painter_canvas_mods::forward(painter, shift, ctrl, alt);
            return ph2d_app_sculpt3d' \
  '            return ph2d_app_sculpt3d' \
  'P12 os modificadores não chegam ao Painter sobre a peça'

# ── O ANEL DO LIQUIFY E A AQUARELA MOLHADA (report do dono, 24/09) ─────────
muta PINTOR crates/ph2d-tool-painter/src/tool/screen_canvas.rs \
  '        if self.is_deform_mode() {' \
  '        if false {' \
  'T9 o anel do Liquify fica no tamanho do pincel de pintura'
muta PINTOR crates/ph2d-tool-painter/src/tool/screen_canvas.rs \
  'self.on_screen_canvas() && self.wet_session_continues()' \
  'self.on_screen_canvas()' \
  'T10 o papel diz-se molhado depois de um traço Digital'
muta COSTURA crates/ph2d-app-sculpt3d/src/painter_na_malha.rs \
  's.painter_raio_px = Some(painter.screen_canvas_ring_px());' \
  's.painter_raio_px = Some(painter.dab_footprint_px());' \
  'P13 a costura lê o raio do pincel de pintura para o anel'
muta COSTURA crates/ph2d-app-sculpt3d/src/painter_na_malha.rs \
  'if semeado && painter.screen_canvas_is_wet()' \
  'if semeado' \
  'P14 a tela fica depois de TODO traço semeado, molhado ou não'
muta COSTURA crates/ph2d-app-sculpt3d/src/painter_na_malha.rs \
  '                scene.painter_guarda(vista, retrato);' \
  '                let _ = (vista, retrato);' \
  'P15 a tela molhada nunca é guardada'
muta COSTURA crates/ph2d-app-sculpt3d/src/painter_na_malha.rs \
  '        self.painter_ultima = Some(Arc::clone(&f.rgba));' \
  '' \
  'P16 a semente guardada não é o que a peça recebeu'
muta COSTURA crates/ph2d-app-sculpt3d/src/painter_na_malha.rs \
  '                s.painter_molhada = None;' \
  '' \
  'P17 uma tela que renasce fica tomada pela molhada'
muta COSTURA crates/ph2d-app-sculpt3d/src/painter_na_malha.rs \
  '            s.painter_molhada = None;
            painter.release_screen_canvas();' \
  '            painter.release_screen_canvas();' \
  'P18 uma tela solta fica tomada pela molhada'
muta COSTURA crates/ph2d-app-sculpt3d/src/painter_na_malha.rs \
  '            painter.clear_screen_canvas();
            let _ = painter.take_screen_canvas();
        }' \
  '        }' \
  'P19 a pintura simples começa por cima da tela molhada'
muta PRODUTO crates/ph2d-app-sculpt3d/src/painter_na_malha.rs \
  'if semeado && painter.screen_canvas_is_wet()' \
  'if false && semeado && painter.screen_canvas_is_wet()' \
  'W1 a tela limpa-se sempre: a aquarela seca a cada traço'
muta PRODUTO crates/ph2d-app-sculpt3d/src/painter_na_malha.rs \
  '&& g.serve(sessao.vista(), objeto, edits)' \
  '&& true' \
  'W2 a tela molhada é reaproveitada sem conferir a chave'
muta PRODUTO crates/ph2d-app-sculpt3d/src/painter_na_malha.rs \
  'self.vista == *vista && self.objeto == objeto && self.edits == edits' \
  'self.vista == *vista && self.objeto == objeto' \
  'W3 a chave não vê a peça mudar (Ctrl+Z, outro pincel)'
muta PRODUTO crates/ph2d-app-sculpt3d/src/painter_na_malha.rs \
  'self.vista == *vista && self.objeto == objeto && self.edits == edits' \
  'self.objeto == objeto && self.edits == edits' \
  'W4 a chave não vê a vista rodar'

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

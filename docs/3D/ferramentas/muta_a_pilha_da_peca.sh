#!/usr/bin/env bash
# Prova de mutação das CAMADAS na peça — W1 (a pilha, o documento v6), W2
# (pintar na camada activa) e W3 (o painel de Layers sobre a pilha). `docs/3D/30` §10–§12.
#
# Corre-se pela fatia da linha e com a placa (as corridas `gpu` pedem-na):
#   PH2D_GPU=1 bash scripts/ph2d-run.sh bash docs/3D/ferramentas/muta_a_pilha_da_peca.sh
# Pré-voo das âncoras (zero testes): MUTA_SO_ANCORAS=1 bash <este ficheiro>
#
# ⚠️ O arnês CONTROLA-SE A SI MESMO (o idioma dos irmãos): a âncora casa
# EXACTAMENTE uma vez, a mutação COMPILA, e a corrida corre N > 0 testes
# (`passed + failed`). Cada corrida tem o seu CONTROLO verde antes.
set -u
SO_ANCORAS="${MUTA_SO_ANCORAS:-}"
SRCS=(crates/ph2d-app-sculpt3d/src crates/ph2d-sculpt3d/src crates/ph2d-mesh-colors/src
      crates/ph2d-tool-painter/src crates/ph2d-panel-painter-layers/src)
BK=$(mktemp -d)
for s in "${SRCS[@]}"; do mkdir -p "$BK/$s"; cp -r "$s/." "$BK/$s/"; done
restore() {
  for s in "${SRCS[@]}"; do rm -rf "$s"; mkdir -p "$s"; cp -r "$BK/$s/." "$s/"; find "$s" -name '*.rs' -exec touch {} +; done
}
# ⚠️ Também no INT/TERM: a fatia da linha (`ph2d-run.sh`) mata por prazo, e um
#    arnês morto a meio deixaria o src MUTADO.
trap restore EXIT
trap 'exit 130' INT TERM

# As corridas, por nome.
corre() {
  case "$1" in
    core) cargo test -p ph2d-sculpt3d --lib -- tinta_fina_camada alfa_tests o_retrato_de_uma_camada 2>&1 ;;
    app) cargo test -p ph2d-app-sculpt3d --lib -- tinta_da_peca::pilha pilha_da_peca doc:: 2>&1 ;;
    gpu) cargo test -p ph2d-app-sculpt3d --lib -- --ignored camadas:: tinta_no_produto_tests::painter::um_traco_do_painter tinta_no_produto_tests::fill 2>&1 ;;
    tool) cargo test -p ph2d-tool-painter --lib -- piece_layers 2>&1 ;;
    painel) cargo test -p ph2d-panel-painter-layers --test it -- seam_peca 2>&1 ;;
    w3gpu) cargo test -p ph2d-app-sculpt3d --lib -- --ignored tinta_no_produto_tests::painel:: 2>&1 ;;
  esac
}
contados() { grep -oP 'test result: \w+\. \K[0-9]+(?= passed)|[0-9]+(?= failed)' | awk '{s+=$1}END{print s+0}'; }

if [ -z "$SO_ANCORAS" ]; then
  for c in core app gpu tool painel w3gpu; do
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

A=crates/ph2d-app-sculpt3d/src
S=crates/ph2d-sculpt3d/src

# ── W1: a pilha ─────────────────────────────────────────────────────────────
muta $A/pilha_da_peca.rs '                    x: x as u32,
                    y: y as u32,
                    w: w as u32,
                    h: 1,' '                    x: (x as u32).saturating_sub(1),
                    y: y as u32,
                    w: w as u32,
                    h: 1,' 'M1 compor_faixa lê a faixa deslocada de uma amostra' app
muta $A/pilha_da_peca.rs '    let altura = n.div_ceil(largura).max(1);' '    let altura = (n / largura).max(1);' \
  'M2 a dobra perde a última linha parcial' app
muta $A/pilha_da_peca.rs '            .partition(|(k, _)| vivas.contains(k));' '            .partition(|_| true);' \
  'M3 apagar uma camada deixa o plano órfão' app
muta $A/pilha_da_peca.rs '        if px[3] == 255 {' '        if px[3] == 254 {' \
  'M4 o opaco deixa de ser byte/255 exacto' app
muta $A/pilha_da_peca.rs '        tinta.com_relevo(self.relevo_composto());' '        tinta.com_relevo(None);' \
  'M5 o relevo da pilha não chega à peça' app
muta $A/pilha_da_peca.rs '        let fundo = if precisa_de_fundo(&composto) {
            fundo()
        } else {
            Vec::new()
        };' '        let fundo: Vec<[f32; 3]> = {
            let _ = fundo;
            Vec::new()
        };' 'M6 o transparente assenta no branco e não na semente' app
muta $A/pilha_da_peca.rs '        plano.rgba8[..self.amostras * 4].fill(255);' '        plano.rgba8.fill(255);' \
  'M7 a máscara nova pinta a cauda da dobra' app
muta $A/pilha_da_peca.rs '    (c.clamp(0.0, 1.0) * 255.0).round() as u8' '    (c.clamp(0.0, 1.0) * 255.0) as u8' \
  'M8 o byte trunca em vez de arredondar' app
# (W6, `docs/3D/30` §14: os de vizinhança borram na retícula — o que fica de fora é o PLANO da imagem.)
muta $A/pilha_da_peca.rs '                    Some(LayerKind::Adjustment(a)) => !a.kind.reads_the_image_plane(),' \
  '                    Some(LayerKind::Adjustment(_)) => true,' 'M9 a pilha aceita um ajuste que lê o plano da imagem' app
muta $A/doc_camadas.rs '            if planos.insert(p.id, plano).is_some() {' '            if planos.insert(p.id, plano).is_some() && false {' \
  'M10 o leitor aceita um plano repetido' app

# ── W2: pintar na camada activa ─────────────────────────────────────────────
muta $S/tinta_fina.rs '            if fina.tinta.tem_alfa() {' '            if false {' \
  'W1 a pintura não escreve a opacidade' core
muta $S/tinta_fina_anel.rs '            if wa > 0.0 {
                for k in 0..4 {' '            if wa > 0.0 {
                for k in 0..3 {' 'W2 o anel não mistura a opacidade' core
muta $S/tela_na_malha_pousa.rs '                alfa * fica + a * k,' '                alfa,' \
  'W3 a tela do Painter não pousa opacidade' core
muta $S/preenche.rs '            let nova_a = a * (1.0 - keep) + keep;' '            let nova_a = a;' \
  'W4 o balde não pousa opacidade' core
muta $S/tela_semente.rs '                px[3] = (a.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;' '                px[3] = 255;' \
  'W5 a semente do Painter perde a opacidade da camada' core
muta $S/tinta_fina.rs '        if mudou_a {' '        if false {' \
  'W6 repintar não escreve a opacidade' core
muta $A/tinta_da_peca_pilha.rs '    p.compoe_amostras(sujas, peca, || p.fundo_semeado(mesh, k));' \
  '    let _ = (mesh, k, p);' 'W7 o quadro desce à camada e não recompõe a peça' gpu
muta $A/pilha_da_peca_traco.rs '        if let Some(r) = w.relevo()' '        if let Some(r) = None::<&[[f32; 2]]>' \
  'W8 o relevo do traço não desce à base' app
muta $A/pilha_da_peca_traco.rs '            Some((_, fim)) if *fim == i => *fim += 1,' '            Some((_, fim)) if *fim == i => *fim += 0,' \
  'W9 as corridas da recomposição perdem amostras' app
muta $A/pilha_da_peca_traco.rs '    if ab == 0 {
        return [0; 4];
    }' '' 'W10 um quase-transparente explode ao branco' app
muta $A/history_tinta_fina.rs '        j.camada = Some((id, rgba));' '        let _: Vec<[u8; 4]> = rgba;
        let _ = id;' 'W11 o desfazer de um traço numa camada troca o plano da peça' gpu
muta $A/tinta_da_peca_pilha.rs '            p.pinta_tinta(t, Vec::new);' '            let _ = &t;' \
  'W12 a pilha nasce e o plano não é recomposto dela' app
muta $A/tinta_da_peca_pilha.rs '        if d.origem == Origem::Parque {' '        if false {' \
  'W13 a pilha não volta do estacionamento' app
muta $A/doc_camadas.rs '                    camadas: match pilha.filter(|p| p.amostras() == t.amostras().len()) {' \
  '                    camadas: match None::<&PilhaDaPeca> {' 'W14 o escritor grava UMA camada em vez da pilha' app

# ── W3: o painel de Layers sobre a pilha da peça ────────────────────────────
T=crates/ph2d-tool-painter/src/tool
P=crates/ph2d-panel-painter-layers/src
muta $A/painter_na_malha.rs '            s.camadas_do_painel(painter);' '' \
  'P1 o quadro não drena os pedidos do painel' w3gpu
muta $A/painter_na_malha_camadas.rs '        if !continua {' '        if true {' \
  'P2 cada passo de um arrasto é um desfazer' w3gpu
muta $A/painter_na_malha_camadas.rs '            crate::tinta_da_peca::pilha::recompoe(o);' '            let _ = o;' \
  'P3 os pedidos mudam a pilha e a peça não se recompõe' w3gpu
muta $A/history_tinta_fina.rs '        crate::tinta_da_peca::pilha::recompoe(obj);' '' \
  'P4 o desfazer do painel não recompõe a peça' w3gpu
muta $A/pilha_da_peca_porta.rs '        inversa.extend(sem_camada);' '        drop(sem_camada);' \
  'P5 a troca perde o plano que fica sem camada' app
muta $A/pilha_da_peca.rs '        Ok(mortos)' '        Ok({ drop(mortos); BTreeMap::new() })' \
  'P6 apagar não devolve os planos ao desfazer' app
muta $A/pilha_da_peca_porta.rs '        if !mesma_estrutura(&self.pilha, &nova) {' '        if false {' \
  'P7 o metadado muda a estrutura' app
muta $A/pilha_da_peca_porta.rs '        if nova.root().last() != self.pilha.root().last() {' '        if false {' \
  'P8 a base sai do fundo pelo metadado' app
muta $A/pilha_da_peca_porta.rs '        if self.em_traco.is_some() {' '        if false {' \
  'P9 a porta mexe na pilha com um traço aberto' app
muta $A/pilha_da_peca.rs $'        if let Some(p) = self.planos.get(&id).cloned() {\n            self.planos.insert(copia, p);' $'        if let Some(mut p) = self.planos.get(&id).cloned() {\n            p.relevo = None;\n            self.planos.insert(copia, p);' \
  'P10 a cópia perde o relevo (W4: leva-o)' app
muta $A/pilha_da_peca.rs '            self.pilha.set_active(a);' '            let _ = a;' \
  'P11 o ajuste novo rouba a activa' app
muta $T/trait_impls.rs '        if self.route_piece_layer_event(&event) {' '        if false && self.route_piece_layer_event(&event) {' \
  'P12 com a tela presa os gestos caem na pilha da tela' tool
muta $T/piece_layers.rs '        self.piece_layers = Some(m.clone());' '' \
  'P13 o espelho não segue o pedido' tool
muta $T/piece_layers.rs '            PanelEvent::SetValue(id, _) => Some(*id),' '            PanelEvent::SetValue(..) => None,' \
  'P14 o arrasto perde o id do controlo' tool
muta $T/piece_layers.rs '        if self.panel_shows_the_piece() {
            self.piece_layers.as_ref()' '        if true {
            self.piece_layers.as_ref()' 'P15 o painel mostra a pilha da peça fora dela' tool
muta $A/painter_na_malha.rs '        if let Some(r) = scene.a_activa_recusa_o_traco() {' \
  '        if let Some(r) = None::<crate::pilha_da_peca::RecusaDaPilha> {' 'P16 o traço não recusa a activa que não é de pintura' w3gpu
muta $P/peca.rs '    let Some(s) = stack else {
        return false;
    };' '    let Some(s) = stack else {
        return false;
    };
    let _ = s;
    return true;
    #[allow(unreachable_code)]' 'P17 o painel oferece na peça o que ela não tem' painel
muta $P/dropdown_popover.rs '        if opt.disabled {' '        if false {' \
  'P18 a opção apagada continua clicável' painel
muta $P/adjust_menu.rs '            .disabled(!crate::peca::adjustment_offered(*kind))' '' \
  'P19 o menu da peça não apaga os de vizinhança' painel

if [ -n "$SO_ANCORAS" ]; then
  echo "PRE-VOO: $sangram de $total ancoras casam exactamente uma vez (ZERO testes corridos)"
  [ "$sangram" -eq "$total" ]
else
  echo "RESULTADO: $sangram de $total sangram"
fi

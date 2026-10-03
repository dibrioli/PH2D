#!/usr/bin/env bash
# Prova de mutação das CAMADAS na peça — W1 (a pilha, o documento v6) e W2
# (pintar na camada activa). `docs/3D/30` §10–§11.
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
SRCS=(crates/ph2d-app-sculpt3d/src crates/ph2d-sculpt3d/src crates/ph2d-mesh-colors/src)
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
  esac
}
contados() { grep -oP 'test result: \w+\. \K[0-9]+(?= passed)|[0-9]+(?= failed)' | awk '{s+=$1}END{print s+0}'; }

if [ -z "$SO_ANCORAS" ]; then
  for c in core app gpu; do
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
muta $A/pilha_da_peca.rs '(n as u64).div_ceil(u64::from(LARGURA_DA_DOBRA))' '((n as u64) / u64::from(LARGURA_DA_DOBRA))' \
  'M2 a dobra perde a última linha parcial' app
muta $A/pilha_da_peca.rs '        self.planos.retain(|k, _| vivas.contains(k));' '' \
  'M3 apagar uma camada deixa o plano órfão' app
muta $A/pilha_da_peca.rs '        if px[3] == 255 {' '        if px[3] == 254 {' \
  'M4 o opaco deixa de ser byte/255 exacto' app
muta $A/pilha_da_peca.rs '        self.planos.get(&base)?.relevo.clone()' '        None' \
  'M5 o relevo da base não chega à peça' app
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
muta $A/pilha_da_peca.rs '                    Some(LayerKind::Adjustment(a)) => !a.kind.reads_the_image_layout(),' \
  '                    Some(LayerKind::Adjustment(_)) => true,' 'M9 a pilha aceita um ajuste que lê a vizinhança' app
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
muta $A/tinta_da_peca_pilha.rs '    pilha.compoe_amostras(sujas, peca, || semente(mesh, k).amostras().to_vec());' \
  '    let _ = (mesh, k);' 'W7 o quadro desce à camada e não recompõe a peça' gpu
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

if [ -n "$SO_ANCORAS" ]; then
  echo "PRE-VOO: $sangram de $total ancoras casam exactamente uma vez (ZERO testes corridos)"
  [ "$sangram" -eq "$total" ]
else
  echo "RESULTADO: $sangram de $total sangram"
fi

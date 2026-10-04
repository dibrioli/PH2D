#!/usr/bin/env bash
# Prova de mutação da PILHA NA PLACA — W1b (a pilha da peça composta na GPU),
# `docs/3D/30` §13: o `composto_na_placa`, o `tinta_achata` (Rust + WGSL), o gatilho
# das `slots`, a semente do fundo e o v7.
#
# Corre-se pela fatia da linha e com a placa (a corrida `placa` pede-a):
#   PH2D_GPU=1 bash scripts/ph2d-run.sh bash docs/3D/ferramentas/muta_a_pilha_na_placa.sh
# Pré-voo das âncoras (zero testes): MUTA_SO_ANCORAS=1 bash <este ficheiro>
#
# ⚠️ O arnês CONTROLA-SE A SI MESMO (o idioma dos irmãos): a âncora casa
# EXACTAMENTE uma vez, a mutação COMPILA, e a corrida corre N > 0 testes
# (`passed + failed`). Cada corrida tem o seu CONTROLO verde antes.
set -u
SO_ANCORAS="${MUTA_SO_ANCORAS:-}"
SRCS=(crates/ph2d-app-sculpt3d/src crates/ph2d-mesh-render/src)
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
    cpu) cargo test -p ph2d-app-sculpt3d --profile smoke --lib -- composto_na_placa uma_pilha_translucida_recomposta_fica um_v6_abre doc:: 2>&1 ;;
    placa) cargo test -p ph2d-app-sculpt3d --profile smoke --lib -- --ignored --test-threads=1 tinta_no_produto_tests::placa:: tinta_no_produto_tests::fill tinta_no_produto_tests::camadas:: tinta_no_produto_tests::painel:: --skip diag_ 2>&1 ;;
    render) cargo test -p ph2d-mesh-render --profile smoke --lib -- tinta_achata 2>&1 ;;
  esac
}
contados() { grep -oP 'test result: \w+\. \K[0-9]+(?= passed)|[0-9]+(?= failed)' | awk '{s+=$1}END{print s+0}'; }

if [ -z "$SO_ANCORAS" ]; then
  for c in cpu placa render; do
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
R=crates/ph2d-mesh-render/src

muta $A/composto_na_placa.rs 'version: p.na_placa.versao,' 'version: 1,' \
  'P1 a GPU compõe a camada velha' placa
muta $A/composto_na_placa.rs 'h: b - a + 1,' 'h: 1,' \
  'P2 só a primeira linha suja volta a subir' placa
muta $A/pilha_da_peca.rs 'Some((x, y)) => (x.min(a), y.max(b)),' 'Some(_) => (a, b),' \
  'P3 a união das linhas sujas perde' cpu
muta $A/pilha_da_peca_traco.rs 'plano.mudou_amostra(i);' '' \
  'P4 o traço não marca a camada' cpu
muta $A/pilha_da_peca_traco.rs 'plano.mudou_toda();' '' \
  'P5 a troca de plano não marca' cpu
muta $A/pilha_da_peca.rs 'self.mudou_toda();' '' \
  'P6 escrever o plano não marca' cpu
muta $R/shaders/tinta_achata.wgsl 'cor = vec3<f32>(tabela[r], tabela[g], tabela[b]);' 'cor = vec3<f32>(tabela[256u + r], tabela[256u + g], tabela[256u + b]);' \
  'P7 o ramo opaco lê a tabela errada' placa
muta $R/shaders/tinta_achata.wgsl '+ f * (1.0 - al);' '+ f * al;' \
  'P8 o mix translúcido invertido' placa
muta $A/slots.rs '|| (subiu_inteiro && atrasada)' '|| false' \
  'P9 o plano velho subido inteiro não recompõe' placa
# (W6: há um 2.º `compor_na_placa` — o do traço por baixo de um desfoque, no `muta_os_efeitos_de_vizinhanca.sh`.)
muta $A/tinta_da_peca_pilha.rs $'    *relevo_sujo |= pilha.redobra_o_relevo(peca);\n    *compor_na_placa = true;' $'    *relevo_sujo |= pilha.redobra_o_relevo(peca);' \
  'P10 recompor o plano não pede a placa' placa
muta $A/pilha_da_peca_fundo.rs 'tinta.amostras_mut()[..v.len()].copy_from_slice(&v);' '' \
  'P11 o prefixo de vértices não refresca' placa
muta $A/pilha_da_peca_fundo.rs 'Tinta::semeada(&self.fundo, faces(), nivel)' 'Tinta::semeada(mesh.colors().unwrap_or(&self.fundo), faces(), nivel)' \
  'P12 o fundo volta à cor viva (a deriva)' cpu
muta $A/tinta_da_peca_pilha.rs 'Some(p) if p.atrasada() && p.amostras() == plano.amostras().len() =>' 'Some(p) if false && p.atrasada() && p.amostras() == plano.amostras().len() =>' \
  'P13 para_ler não usa o plano atrasado' placa
# (W6: a recusa da placa passa pela porta `em_dia`, que também acerta a cor por vértice.)
muta $A/slots.rs 'crate::tinta_da_peca::pilha::em_dia(&mut self.objects[i]);' '' \
  'P14 a recusa sobe o plano velho' placa
muta $A/doc.rs 'doc.camadas.fundo.clone()' 'vec![ph2d_mesh_colors::BRANCO; mesh.vert_count()]' \
  'P15 o v7 ignora o fundo gravado' cpu
muta $A/doc_migracao.rs 'fundo: Vec::new(),' 'fundo: vec![[0.5; 3]; 1],' \
  'P16 de_v6 inventa um fundo' cpu
muta $R/tinta_achata.rs '[gx, grupos.div_ceil(gx), gx * GRUPO]' '[gx, grupos.div_ceil(gx), gx]' \
  'P17 o passo_y da grade do despacho' render
muta $A/pilha_da_peca.rs '    while n.div_ceil(largura) > teto && largura < teto {' '    while n.div_ceil(largura) >= teto && largura < teto {' \
  'P20 a dobra alarga antes de precisar' cpu
# (P18 saiu: «outra dobra, outro compositor» era uma 2.ª resposta — o `ensure_array` do compositor
#  já reconstrói as fatias e esquece o cache quando a tela muda de tamanho; a mutação era equivalente.)
muta $A/history_tinta_fina.rs 'crate::tinta_da_peca::pilha::recompoe_o_plano(obj);' 'crate::tinta_da_peca::pilha::recompoe(obj);' \
  'P19 o Ctrl+Z do balde não recompõe o plano' placa

if [ -n "$SO_ANCORAS" ]; then
  echo "PRE-VOO: $sangram de $total ancoras casam exactamente uma vez (ZERO testes corridos)"
  [ "$sangram" -eq "$total" ]
else
  echo "RESULTADO: $sangram de $total sangram"
fi

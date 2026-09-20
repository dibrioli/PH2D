#!/usr/bin/env bash
# Provas de mutação da LEI DOS OSSOS (report do dono, 2026-09-19, com duas fotos):
#   «Funciona como desejado se coloque pivot offset x em -2 mas o pivot fica na ponta dos ossos.
#    assim skeleton:bend coloca a base de um osso na ponta do outro. Contudo o mais correto seria
#    se tivesse o mesmo resultado colocando na base do osso.»
#
# ⭐⭐⭐ **A causa não estava na FORMA: estava no que a corrente ENTREGA.** O elemento que um
#    `rig.*` publica é uma JUNTA, e o `len`/`rot` que ele carrega são os do osso que CHEGA a ele
#    ⇒ carimbar a forma na junta desenha-a uma junta à frente. O `rig.bones` devolve o quadro do
#    próprio osso: a CABEÇA (a posição do pai, que é o ponto em torno do qual ele roda).
#
# ⚠️ `touch` no fim de cada restauro · CONTROLO sobre o próprio FILTRO · `muta` aborta na âncora.
# ⛔⛔ NUNCA duas corridas deste script ao mesmo tempo (ele escreve na árvore de verdade).
#
# Corra-o pela porta de recursos:
#   bash scripts/ph2d-run.sh bash "docs/Motion Nodes/ferramentas/mutacao_a_corrente_entrega_ossos_2026-09-19.sh"
set -u
cd "$(dirname "$0")/../../.." || exit 1

BON="crates/ph2d-node-rig-bones/src/lib.rs"
DEM="crates/ph2d-app-motion/src/motion_state_osso_demo.rs"

TMP="$(mktemp -d)"
FALHAS=0
TOTAL=0

guarda()   { cp "$1" "$TMP/$(basename "$1").orig"; }
restaura() { cp "$TMP/$(basename "$1").orig" "$1"; touch "$1"; }

muta() { # ficheiro vezes antigo novo
  python3 - "$1" "$2" "$3" "$4" <<'PY'
import sys
p, n, old, new = sys.argv[1], int(sys.argv[2]), sys.argv[3], sys.argv[4]
s = open(p).read()
c = s.count(old)
if c != n:
    sys.exit(f"  ⛔ ANCORA: {old!r} aparece {c} vezes em {p} (esperado {n})")
open(p, "w").write(s.replace(old, new))
PY
}

prova() { # nome pacote filtro
  TOTAL=$((TOTAL+1))
  echo "── $1"
  local out rc corridos
  out=$(cargo test -p "$2" --lib -- "$3" 2>&1); rc=$?
  corridos=$(printf '%s' "$out" | grep -oE 'running [0-9]+ tests?' | grep -oE '[0-9]+' \
             | awk '{s+=$1} END {print s+0}')
  if [ "$corridos" -lt 1 ]; then
    echo "  ⛔ FILTRO VAZIO — '$3' nao casou teste nenhum (ou nao compilou):"
    printf '%s\n' "$out" | grep -E '^error' | head -3
    FALHAS=$((FALHAS+1)); return
  fi
  if [ "$rc" = 0 ]; then
    echo "  ⛔⛔ SOBREVIVEU — os $corridos teste(s) de '$3' passaram sobre o produto MUTADO"
    FALHAS=$((FALHAS+1))
  else
    echo "  ✅ sangrou (de $corridos teste(s) corridos)"
  fi
}

bloco() { # nome pacote filtro ficheiro vezes antigo novo
  guarda "$4"
  if muta "$4" "$5" "$6" "$7"; then prova "$1" "$2" "$3"; else FALHAS=$((FALHAS+1)); fi
  restaura "$4"
}

echo "== O QUADRO DO OSSO: a CABECA e' a posicao do PAI =="

# 1. O coracao da wave: a peca volta a pousar na JUNTA em vez de na cabeca do osso dela.
#    ⚠️ E' EXACTAMENTE o estado que o dono fotografou, e que so' o `Pivot Offset X = -2` curava.
bloco "a peca volta a pousar na JUNTA" ph2d-node-rig-bones o_osso_vai_de_uma_junta \
  "$BON" 1 \
  "Column::Vec2(ossos.iter().map(|&(_, j)| pos[j]).collect())," \
  "Column::Vec2(ossos.iter().map(|&(i, j)| { let _ = j; pos[i] }).collect()),"

# 2. As outras colunas sao do FILHO (quem carrega o `rot` e o `len` daquele osso), nao do pai.
bloco "as colunas passam a ser colhidas pelo PAI" ph2d-node-rig-bones as_outras_colunas \
  "$BON" 1 \
  "ossos.iter().map(|&(i, _)| v[i].clone()).collect()" \
  "ossos.iter().map(|&(_, j)| v[j].clone()).collect()"

echo
echo "== A POPULACAO: uma corrente de n juntas veste n-r ossos =="

# 3. A raiz volta a levar peca — era ela a peca pendurada para fora da corrente nas fotos.
bloco "a raiz volta a levar uma peca" ph2d-node-rig-bones a_raiz_sai_e_a_contagem \
  "$BON" 1 \
  "        if pi >= 0.0 && pi.is_finite() && j < i {
            ossos.push((i, j));
        }" \
  "        if pi >= 0.0 && pi.is_finite() && j < i {
            ossos.push((i, j));
        } else {
            ossos.push((i, i));
        }"

# 4. A leitura ingenua («sem pais ⇒ sem ossos») esvazia um `motion.grid`.
bloco 'sem parent ele esvazia em vez de ser a identidade' ph2d-node-rig-bones nao_e_um_rig \
  "$BON" 1 \
  "        return input.clone();" \
  "        return Stream::new(0);"

echo
echo "== O QUE VIAJA, E O QUE NAO PODE VIAJAR =="

# 5. ⛔ A pior das tres: com o `lrot` presente, um `rig.fk` a jusante reescreve `rot` com o
#    angulo LOCAL e a cadeia sai desenhada com os relativos, calada.
bloco 'o lrot volta a viajar' ph2d-node-rig-bones as_colunas_da_corrente \
  "$BON" 1 \
  "if name == PARENT || name == LROT || name == WROT {" \
  "if name == PARENT || name == WROT {"

# 6. E o `parent`: com ele, um `rig.fk` a jusante re-resolve uma arvore cujos indices ja' nao
#    existem.
bloco 'o parent volta a viajar' ph2d-node-rig-bones as_colunas_da_corrente \
  "$BON" 1 \
  "if name == PARENT || name == LROT || name == WROT {" \
  "if name == LROT || name == WROT {"

echo
echo "== A POPULACAO MUDOU DE NOME: Index/Count sao os dos OSSOS =="

# 7. Um `Count` a dizer `n` sobre `n-1` linhas faz toda a normalizacao a jusante (falloff por
#    indice, `motion.stagger`) enderecar o elemento errado.
bloco 'o Count continua a ser o das JUNTAS' ph2d-node-rig-bones o_index_e_o_count \
  "$BON" 1 \
  "        out.set(COUNT, Column::Scalar(vec![m as f32; m]));" \
  "        out.set(COUNT, colhe(input.get(COUNT).expect(\"ha\"), &ossos));"

# 8. E o `Index` que salta o zero.
bloco 'o Index continua a ser o das JUNTAS' ph2d-node-rig-bones o_index_e_o_count \
  "$BON" 1 \
  "            Column::Scalar((0..m).map(|i| i as f32).collect::<Vec<_>>()),
        );" \
  "            colhe(input.get(INDEX).expect(\"ha\"), &ossos),
        );"

echo
echo "== A FIACAO E O ROTEIRO DA CENA =125 =="

# 9. Um motor com a lei certa e a cena a nao o ligar le-se como um motor sem a lei.
bloco 'a cena =125 deixa de ligar o rig.bones' ph2d-app-motion a_cadeia_de_ossos_ladrilha \
  "$DEM" 1 \
  "        let ossos = no(g, \"rig.bones\", 340.0, y);
        g.connect(Edge {
            from: (corpo, 0),
            to: (ossos, 0),
            delayed: false,
        })
        .ok()?;
        corpo = ossos;
" \
  ""

# 10. O roteiro nomeia um cartao que esta' na tela — e o nome vem do REGISTO, nao da memoria.
# ⛔⛔ **TRES ocorrencias, e nao uma:** a 1.a redaccao mutou so' a do passo (1b) e a prova
#     SOBREVIVEU — as outras duas mantinham o gate verde. *Uma mutacao que nao muta tudo o que a
#     lei le' e' um no-op, e um no-op le-se exactamente como uma sobrevivencia.*
bloco 'o roteiro passa a chamar o cartao por outro nome' ph2d-app-motion o_roteiro_nomeia \
  "$DEM" 3 \
  'Bones' \
  'Ossos'

echo
if [ "$FALHAS" = 0 ]; then
  echo "✅ $TOTAL de $TOTAL mutacoes sangraram."
else
  echo "⛔ $FALHAS de $TOTAL falharam."
fi
# ⛔⛔ **A prova do restauro compara-se com o BACKUP, nunca com o `HEAD`** — a linha esta' por
# commitar, logo o `HEAD` difere por construcao.
for f in "$BON" "$DEM"; do
  cmp -s "$f" "$TMP/$(basename "$f").orig" || {
    echo "⛔⛔ $f NAO foi restaurado — reponha de $TMP a mao."; exit 1; }
done
exit "$FALHAS"

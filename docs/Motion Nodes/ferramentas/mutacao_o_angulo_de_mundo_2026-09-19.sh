#!/usr/bin/env bash
# Provas de mutação da LEI DO ÂNGULO DE MUNDO no rig (report do dono, 2026-09-19, com foto:
# «por que em skeleton o último objeto tem direção diferente?»).
#
# A corrente tem TRÊS elos, e cada bloco mata um:
#
#   o AUTOR escreve o local (`lrot`)      →  `rig.skeleton::build`, e os 3 nós que POSAM
#   o RESOLVEDOR acumula                  →  `fk::resolve` + a porta `fk::local`
#   e publica o MUNDO em `rot`            →  a coluna que o desenho lê
#
# ⛔⛔ **As duas metades falham em silêncio, e é por isso que este script existe:** escrever no
#    `rot` de uma corrente já resolvida é IGNORADO (o `local` prefere o `lrot`), e ler o `rot`
#    como local devolve zeros numa cadeia recta. Nenhuma das duas dá erro.
# ⚠️ `touch` no fim de cada restauro · CONTROLO sobre o próprio FILTRO · `muta` aborta na âncora.
# ⛔⛔ NUNCA duas corridas deste script ao mesmo tempo (ver o irmão `mutacao_a_lei_o_gizmo…`).
#
# Corra-o pela porta de recursos:
#   bash scripts/ph2d-run.sh bash "docs/Motion Nodes/ferramentas/mutacao_o_angulo_de_mundo_2026-09-19.sh"
set -u
cd "$(dirname "$0")/../../.." || exit 1


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

# ⚠️ O irmão para os gates que vivem em `tests/it/`: um `--lib` ali casa ZERO testes, e o
# controlo do filtro leria isso como «filtro vazio» — certo, e inútil.
prova_it() { # nome pacote filtro
  TOTAL=$((TOTAL+1))
  echo "── $1"
  local out rc corridos
  out=$(cargo test -p "$2" --test it -- "$3" 2>&1); rc=$?
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

# ⭐⭐⭐ A forma MAIS FORTE de sangrar: a mutação nem chega a compilar.
#
# ⚠️ **Ela precisa de bloco próprio porque o `prova` a leria como «filtro vazio»** — e um
# «filtro vazio» é um alarme sobre o ARNÊS, não sobre o produto. Aqui o que se afirma é que o
# COMPILADOR guarda a lei, que é o que um `const _: () = assert!(…)` compra.
prova_nao_compila() { # nome pacote
  TOTAL=$((TOTAL+1))
  echo "── $1"
  if cargo check -p "$2" --all-targets >/dev/null 2>&1; then
    echo "  ⛔⛔ SOBREVIVEU — o produto MUTADO compila; a lei nao esta' no compilador"
    FALHAS=$((FALHAS+1))
  else
    echo "  ✅ sangrou no COMPILADOR (a forma mais forte)"
  fi
}

bloco_nao_compila() { # nome pacote ficheiro vezes antigo novo
  guarda "$3"
  if muta "$3" "$4" "$5" "$6"; then prova_nao_compila "$1" "$2"; else FALHAS=$((FALHAS+1)); fi
  restaura "$3"
}

bloco_it() { # nome pacote filtro ficheiro vezes antigo novo
  guarda "$4"
  if muta "$4" "$5" "$6" "$7"; then prova_it "$1" "$2" "$3"; else FALHAS=$((FALHAS+1)); fi
  restaura "$4"
}


SK=crates/ph2d-node-rig-skeleton/src/lib.rs
FKS=crates/ph2d-node-rig-skeleton/src/fk.rs
IK=crates/ph2d-node-rig-ik-2bone/src/lib.rs
FKI=crates/ph2d-node-rig-ik-2bone/src/fk.rs
POSE=crates/ph2d-node-rig-ik-2bone/src/pose.rs

echo "=== ELO 1 — o AUTOR escreve o LOCAL ==="

# ⛔⛔⛔ **NO-OP MEDIDO — esta mutação NÃO sangra, e a razão é a lei a funcionar.**
#
# Trocar aqui o `fk::LROT` pelo `fk::ROT` deixa o `o_esqueleto_entrega_o_angulo_de_mundo` VERDE,
# e não é um buraco do gate: a porta `fk::local` tem DOIS degraus (`lrot`, senão `rot`), e numa
# corrente **acabada de construir** — que é o que o `build` devolve — não existe `lrot` nenhum,
# logo os dois nomes levam ao mesmo sítio. *O segundo degrau existe de propósito: é por ele que
# uma corrente autorada à mão (um documento, um MCP, uma fixtura) continua a posar.*
#
# ⚠️ O que a escolha do `LROT` no `build` compra é UNIFORMIDADE — a mesma coluna em toda a
# família —, e o que a defende é o bloco «a porta ignora o ROT» abaixo, que mata o degrau.
# *Uma linha que a mutação não consegue matar é normalmente comentário com sintaxe de código;
# esta é a excepção que se nomeia, com a medição ao lado.*

# ⛔⛔ **NO-OP MEDIDO — e ele é a escada NOVA a ser tolerante, não um buraco.**
#
# Trocar aqui o `fk::LROT` pelo `fk::ROT` já não sangra: com o degrau 2 (*«o `rot` reescrito
# ganha»*), o solve entra pelos dois caminhos. ⚠️ Ele NÃO era um no-op antes desse degrau existir
# — nessa altura esta mutação punha a mão da IK a aterrar em `[3.0, 0.0]` em vez de `[1.5, 1.5]`,
# e foi ela que provou que a cura tinha de ir além do resolvedor.
#
# ⭐ O que fica a defender a escolha é o degrau em si (o bloco «o rot REESCRITO deixa de ganhar»):
# sem ele, um solve escrito no `rot` de uma corrente já resolvida voltaria a ser deitado fora.

echo
echo "=== ELO 2 — o RESOLVEDOR le' o local pela PORTA ==="

bloco "a porta ignora o LROT (o solve da resolucao anterior ganha)" \
  ph2d-node-rig-ik-2bone "" "$FKI" 1 \
  "    scalars(input, LROT, 0.0, n)" \
  "    scalars(input, ROT, 0.0, n)"

bloco "a porta ignora o ROT AUTORADO (uma corrente a' mao deixa de posar)" \
  ph2d-node-rig-skeleton "" "$FKS" 1 \
  "        return rot; // autorada à mão: o \`rot\` é o local, como sempre foi" \
  "        return vec![0.0; n];"

# ⛔⛔ **O degrau 2 e' o que o `motion_rig_probe` comprou**, e a mutacao dele e' a mais
#    importante deste script: sem ele, ligar uma caneta ao canal \`rot\` — que e' como o artista
#    dobra um esqueleto — deixa de posar, EM SILENCIO.
bloco "o rot REESCRITO deixa de ganhar (a caneta do artista fica muda)" \
  ph2d-node-rig-skeleton "as_duas_portas_posam" "$FKS" 1 \
  "    if rot != w {
        return rot;
    }" \
  "    let _ = &w;"

echo
echo "=== ELO 3 — o `rot` que SAI e' o MUNDO ==="

# ⚠️ A 1.ª redacção desta mutação NÃO COMPILAVA (o `rot` já foi movido para o `LROT` acima), e
#    o arnês leu-a como «FILTRO VAZIO» — que é um alarme sobre o ARNÊS e não sobre o produto.
#    *Uma mutação que não compila não afirma nada.* Aqui ela recompõe o local pela porta.
bloco "o rot volta a levar o LOCAL (a foto do dono, seta a seta)" \
  ph2d-node-rig-skeleton "o_rot_que_sai" "$FKS" 1 \
  "    out.set(ROT, Column::Scalar(w.clone()));" \
  "    out.set(ROT, Column::Scalar(local(input, n)));"

bloco "o LROT deixa de ser preservado (a 2.a resolucao soma o mundo ao mundo)" \
  ph2d-node-rig-skeleton "as_duas_portas_posam" "$FKS" 1 \
  "    out.set(LROT, Column::Scalar(rot));" \
  "    let _ = &rot;"

bloco "o pose.rs volta a ler o local do ROT (ele passa a ler o MUNDO)" \
  ph2d-node-rig-ik-2bone "o_relocal_devolve" "$POSE" 2 \
  "fk::local(input, n)" \
  "fk::scalars(input, fk::ROT, 0.0, n)"

echo
echo "── TOTAL: $TOTAL provas · $FALHAS falha(s)"
[ "$FALHAS" = 0 ]

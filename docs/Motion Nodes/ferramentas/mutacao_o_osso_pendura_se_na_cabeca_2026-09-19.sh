#!/usr/bin/env bash
# Provas de mutação da LEI DO OSSO PENDURADO (report do dono, 2026-09-19, com foto):
#   «Shape Bone deveria ser gerado por padrão a 180 graus de rotação do atual pois está
#    invertido. Outra coisa: em skeleton deveríamos ter um offset do centro para os bones;
#    os ossos estão rotacionando a partir do centro e não da cabeça dos ossos.»
#
# ⭐⭐⭐ **As duas frases são UMA lei em DOIS ficheiros, e nenhuma metade serve sozinha:**
#
#   a SILHUETA diz qual ponta é a cabeça   →  `symbols_rig::bone` (o quadrilátero + o olho)
#   a CAIXA diz onde a cabeça FICA         →  `motion_shape_gen::vec_recipe` (`rig_box`)
#
# ⛔ Com a forma CENTRADA na junta, as duas orientações lêem-se uma como a outra — foi por isso
#    que o report da manhã («ficou 180 graus rodado») foi obedecido virando só a silhueta, e o
#    dono teve de o mandar virar outra vez à tarde. *Um pivô ao meio não declara cabeça nenhuma.*
#
# ⚠️ `touch` no fim de cada restauro · CONTROLO sobre o próprio FILTRO · `muta` aborta na âncora.
# ⛔⛔ NUNCA duas corridas deste script ao mesmo tempo (ele escreve na árvore de verdade).
#
# Corra-o pela porta de recursos:
#   bash scripts/ph2d-run.sh bash "docs/Motion Nodes/ferramentas/mutacao_o_osso_pendura_se_na_cabeca_2026-09-19.sh"
set -u
cd "$(dirname "$0")/../../.." || exit 1

SIM="crates/ph2d-vec-scene/src/symbols_rig.rs"
GEN="crates/ph2d-app-motion/src/motion_shape_gen.rs"

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

echo "== A SILHUETA: qual ponta e' a cabeca =="

# 1. A volta de 180° que o dono mandou dar — o estado EXACTO que ele reprovou à tarde.
bloco "a junta volta a +X (o que o dono reprovou)" ph2d-vec-scene o_osso_e_afilado \
  "$SIM" 1 \
  "poly(&u, &[(0.0, 0.5), (OMBRO, 0.0), (1.0, 0.5), (OMBRO, 1.0)])" \
  "poly(&u, &[(1.0, 0.5), (1.0 - OMBRO, 0.0), (0.0, 0.5), (1.0 - OMBRO, 1.0)])"

# 2. O olho tem de acompanhar a silhueta: virar um sem o outro fura a aresta.
bloco "o olho fica do lado da PONTA" ph2d-vec-scene o_olho_cabe_dentro \
  "$SIM" 1 \
  "u.arc((OMBRO, 0.5), r, r, 0.0, -360.0)" \
  "u.arc((1.0 - OMBRO, 0.5), r, r, 0.0, -360.0)"

echo
echo "== A CAIXA: onde a cabeca fica =="

# 3. O osso volta a ser cortado CENTRADO — a metade que o report da tarde nomeia.
bloco "os simbolos de rig voltam a ser centrados" ph2d-app-motion a_cabeca_do_osso \
  "$GEN" 1 \
  "VecKind::Bone | VecKind::RopeSegment => rig_box," \
  "VecKind::Bone | VecKind::RopeSegment => box_,"

# 4. O CONTROLO do gate: toda a forma passa a pendurar-se, e a circunferência acusa.
#    ⚠️ Sem esta prova, o `lo ≈ 0` ficaria verde sobre uma normalização partida da casa inteira.
bloco "TODA forma passa a pendurar-se (o controlo)" ph2d-app-motion a_cabeca_do_osso \
  "$GEN" 1 \
  "                _ => box_,
            };" \
  "                _ => rig_box,
            };"

# 5. O COMPRIMENTO é metade da promessa: pendurar sem manter os `2s` encolhe o osso.
bloco "a caixa pendura e ENCOLHE o osso" ph2d-app-motion a_cabeca_do_osso \
  "$GEN" 1 \
  "let rig_box = ([0.0, -ry], [2.0 * s, ry]);" \
  "let rig_box = ([0.0, -ry], [s, ry]);"

echo
echo "== A POSE: a consequencia no produto =="

# 6. O sentido da rotação — a metade que prova que o gate ve' PARA ONDE o osso aponta.
bloco "o seno do basis inverte" ph2d-app-motion com_rot_de_90 \
  "$GEN" 1 \
  "                f64::from(b1 * sx)," \
  "                f64::from(-b1 * sx),"

echo
if [ "$FALHAS" = 0 ]; then
  echo "✅ $TOTAL de $TOTAL mutacoes sangraram."
else
  echo "⛔ $FALHAS de $TOTAL falharam."
fi
# ⛔⛔ **A prova do restauro compara-se com o BACKUP, nunca com o `HEAD`.** A 1.ª redacção usava
# `git diff --quiet` e acusou «a arvore ficou suja» sobre um restauro perfeito: a linha estava por
# commitar, logo o `HEAD` difere por construção. *Um arnês que verifica o próprio restauro contra o
# `HEAD` mede se a LINHA está commitada, não se ele repôs o que mutou.*
for f in "$SIM" "$GEN"; do
  cmp -s "$f" "$TMP/$(basename "$f").orig" || {
    echo "⛔⛔ $f NAO foi restaurado — reponha de $TMP a mao."; exit 1; }
done
exit "$FALHAS"

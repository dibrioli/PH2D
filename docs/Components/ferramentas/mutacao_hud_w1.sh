#!/usr/bin/env bash
# Provas de mutação do TOP-20 #20 (o HUD) — W1, A LEI DO CANVAS (`ph2d-hud`).
#
# Cada bloco: MUTA o produto → corre O GATE QUE DEVE MORRER → restaura.
# ⚠️ `touch` no fim de cada restauro (o `cp` devolve mtime antigo e o cargo serve o build MUTADO).
# ⚠️⚠️ **CONTROLO sobre o próprio FILTRO** — um filtro que casa ZERO testes sai VERDE, e isso
#      lê-se exactamente como «sobreviveu» (lição paga pela wave do projéctil, #14).
# ⚠️ **Toda troca é por `muta`, que ABORTA se o texto não aparecer o número esperado de vezes.**
#    o sangue que elas procuram.
set -u
cd "$(dirname "$0")/../../.." || exit 1

TMP="$(mktemp -d)"
FALHAS=0
TOTAL=0

guarda()   { cp "$1" "$TMP/$(basename "$1").$2"; }
restaura() { cp "$TMP/$(basename "$1").$2" "$1"; touch "$1"; }

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

prova() { # nome crate filtro [timeout_s]
  TOTAL=$((TOTAL+1))
  echo "── $1"
  local out rc corridos
  out=$(timeout "${4:-900}" cargo test -p "$2" --all-targets -- "$3" 2>&1); rc=$?
  if [ "$rc" = 124 ]; then
    echo "  ✅ sangrou (o teste nao acabou em ${4}s — o prazo e' o que o fazia acabar)"
    return
  fi
  corridos=$(printf '%s' "$out" | grep -oE 'running [0-9]+ tests?' | grep -oE '[0-9]+' \
             | awk '{s+=$1} END {print s+0}')
  if [ "$corridos" -lt 1 ]; then
    echo "  ⛔ FILTRO VAZIO — '$3' nao casou teste nenhum (ou nao compilou):"
    printf '%s\n' "$out" | grep -E '^error' | head -3
    FALHAS=$((FALHAS+1)); return
  fi
  if [ "$rc" = 0 ]; then
    echo "  ⛔⛔ SOBREVIVEU — os $corridos teste(s) de '$3' passaram sobre o produto mutado"
    FALHAS=$((FALHAS+1))
  else
    echo "  ✅ sangrou (de $corridos teste(s) corridos)"
  fi
}

# um bloco = guarda, muta, prova, restaura. Se a âncora falhar, restaura e conta como falha.
bloco() { # nome crate filtro ficheiro vezes antigo novo [timeout]
  guarda "$4" b
  if muta "$4" "$5" "$6" "$7"; then
    prova "$1" "$2" "$3" "${8:-900}"
  else
    TOTAL=$((TOTAL+1)); FALHAS=$((FALHAS+1))
  fi
  restaura "$4" b
}


LEI=crates/ph2d-hud/src/lib.rs

echo "════ W1 — a lei do canvas ════"

# (1) o `keep` é o MÍNIMO dos dois factores — com o máximo, a caixa transborda a vista.
bloco "keep: min -> max" ph2d-hud a_lei_do_keep "$LEI" 1 \
  "let s = fx.min(fy);" "let s = fx.max(fy);"

# (2) o `stretch` é um factor POR EIXO — repetir o x é o keep disfarçado.
bloco "stretch: um eixo para os dois" ph2d-hud o_stretch_distorce "$LEI" 1 \
  "Fit::Stretch => [fx, fy]," "Fit::Stretch => [fx, fx],"

# (3) a translação é o CENTRO da vista — na origem, o HUD fica onde a cena começou.
bloco "translacao: centro -> origem" ph2d-hud a_translacao_e_sempre "$LEI" 1 \
  "translate: view.center," "translate: [0.0, 0.0],"

# (4) a BANDA é METADE do que sobra (é de cada lado) — sem o meio, ela mede o vão inteiro.
bloco "banda: metade -> inteiro" ph2d-hud a_lei_do_keep "$LEI" 2 \
  ") / 2.0," ") / 1.0,"

# (5) a recusa de uma caixa impossível é a LEI — sem ela, um lado zero dá escala infinita.
bloco "recusa: aceita o lado zero" ph2d-hud uma_caixa_impossivel "$LEI" 1 \
  "ref_w > 0.0 && ref_h > 0.0" "ref_w >= 0.0 && ref_h >= 0.0"

# (6) ⭐ a DIVERGÊNCIA declarada: arredondar a banda como o alvo faz TEM de reprovar o gate que a
#     exige — é o que impede o tecto de virar licença.
bloco "divergencia: arredondar como o alvo" ph2d-hud onde_o_alvo_arredonda "$LEI" 2 \
  ") / 2.0," ").round() / 2.0,"

echo
echo "════ $((TOTAL-FALHAS))/$TOTAL sangraram ════"
[ "$FALHAS" = 0 ] || exit 1

#!/usr/bin/env bash
# Provas de mutação do TOP-20 #19 (a CUTSCENE) — W2, a fase que a faz correr.
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


FASE=shells/desktop/src/render_loop/fase_sequences.rs
LEI=crates/ph2d-ecs/src/sequence.rs

echo "════ W2 — a cutscene corre ════"

# (1) O RELÓGIO PARADO tem de calar a cutscene. Sem a guarda, um objecto com o componente toca
#     sempre — e o artista não teria como a PARAR.
bloco "parado deixa de calar" ph2d-ecs um_relogio_parado_nao_toca_nada "$LEI" 1 \
  "if !estado.running {" "if false {"

# (2) O relógio é o PRIMEIRO. Com «o primeiro a CORRER», a cutscene troca de relógio quando o
#     artista arranca outro timer para outra coisa — em silêncio.
bloco "o relogio deixa de ser o primeiro" ph2d-ecs o_relogio_e_o_primeiro "$LEI" 1 \
  "let Some(estado) = relogios.0.first() else {" \
  "let Some(estado) = relogios.0.iter().find(|s| s.running) else {"

# (3) Um nome que não resolve NÃO cai no container 0 — tocar a cutscene errada lê-se como um
#     defeito do motor.
bloco "nome ausente cai no container 0" ph2d-ecs um_nome_que_nao_resolve "$LEI" 1 \
  "let Some(container) = seq.resolve(nomes.iter().copied()) else {
            continue;
        };" \
  "let container = seq.resolve(nomes.iter().copied()).unwrap_or(0);"

# (4) O LEDGER é o que impede «a cutscene moveu o herói» de virar um passo de undo por quadro.
bloco "sem declarar ao ledger" ph2d-host-desktop o_que_a_cutscene_escreve_passa_pelo_ledger "$FASE" 1 \
  "crate::timeline_preview::declare_timeline_writes(world, &antes, drive);" \
  "let _ = (&antes, &mut *drive);"

# (5) O container é o DO OBJECTO, e não o primeiro — a mutação que troca o índice no apply.
bloco "o apply toca sempre o container 0" ph2d-host-desktop uma_cutscene_a_correr_move "$FASE" 1 \
  "ph2d_timeline::apply_container(world, doc, c.container, c.t, skip);" \
  "ph2d_timeline::apply_container(world, doc, 0, c.t, skip);"

echo
if [ "$FALHAS" = 0 ]; then
  echo "✅ $TOTAL de $TOTAL mutações sangraram"
else
  echo "⛔ $FALHAS de $TOTAL NÃO sangraram"
fi
exit "$FALHAS"

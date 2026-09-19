#!/usr/bin/env bash
# Provas de mutação da cura do report «conflito com atalho Home» (dono, 2026-09-19).
#
# Cada bloco: MUTA → corre O GATE QUE DEVE MORRER → restaura.
# ⚠️ `touch` no fim de cada restauro (o `cp` devolve mtime antigo e o cargo serve o build MUTADO).
# ⚠️⚠️ CONTROLO sobre o próprio FILTRO — um filtro que casa ZERO testes sai VERDE e lê-se como
#      «sobreviveu» (lição paga pela wave do projéctil, #14).
# ⚠️ Toda troca é por `muta`, que ABORTA se o texto não aparecer o número esperado de vezes.
#
# uso:  bash scripts/ph2d-run.sh bash docs/Components/ferramentas/mutacao_tecla_do_roteiro_2026-09-19.sh
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

prova() { # nome crate filtro [alvos]
  TOTAL=$((TOTAL+1))
  echo "── $1"
  local out rc corridos
  # shellcheck disable=SC2086
  out=$(timeout 900 cargo test -p "$2" ${4:---all-targets} -- "$3" 2>&1); rc=$?
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

bloco() { # nome crate filtro ficheiro vezes antigo novo [alvos]
  guarda "$4" b
  if muta "$4" "$5" "$6" "$7"; then
    prova "$1" "$2" "$3" "${8:-}"
  else
    TOTAL=$((TOTAL+1)); FALHAS=$((FALHAS+1))
  fi
  restaura "$4" b
}

CENA=crates/ph2d-app-components/src/dano_smoke.rs
GATILHO=crates/ph2d-app-components/src/trigger_smoke.rs
CENSO=shells/desktop/tests/it/um_roteiro_nunca_rouba_uma_tecla_do_editor.rs
ALVO=--test\ it

echo "════ O DEFEITO DO REPORT volta ════"

# (1) A frase exacta que o dono encontrou. Se o censo não a apanhar, ele não mede nada.
bloco "roteiro do golpe: volta a mandar carregar no Home" ph2d-host-desktop um_roteiro_nunca_rouba \
  "$CENA" 1 \
  'na barra de CIMA carregue em `Pause`: o {TECLA_NOME} deixa de disparar. `Reset`, ao' \
  'carregue em STOP na regua de baixo: o {TECLA_NOME} deixa de disparar. `Home` rebobina, ao' \
  "--test it"

# (2) O irmão: a mesma frase na cena do GATILHO, que a shipou primeiro e ninguém viu.
bloco "roteiro do gatilho: volta a mandar carregar no Home" ph2d-host-desktop um_roteiro_nunca_rouba \
  "$GATILHO" 1 \
  'na barra de CIMA carregue em `Pause`: o {TECLA_NOME} deixa de disparar e volta a ser' \
  'carregue no `Home`: o {TECLA_NOME} deixa de disparar e volta a ser' \
  "--test it"

echo "════ A RÉGUA deixa de medir ════"

# (3) A extracção passa a aceitar braços COM GUARDA — o controlo negativo do `ArrowUp` morre, e sem
#     ele a régua acusaria o passo (4) do golpe, que é CERTO.
bloco "extraccao: aceita bracos com guarda" ph2d-host-desktop um_roteiro_nunca_rouba \
  "$CENSO" 1 \
  '        if t.contains(" if ") {
            continue;
        }' \
  '        if false {
            continue;
        }' \
  "--test it"

# (4) A varredura passa a ler o FICHEIRO em vez do texto impresso — e a própria cura de 19/09 cita
#     o `Home` no comentário, de propósito.
bloco "varredura: le o ficheiro inteiro" ph2d-host-desktop um_roteiro_nunca_rouba \
  "$CENSO" 1 \
  '        .filter(|l| !l.trim_start().starts_with("//"))' \
  '        .filter(|_l| true)' \
  "--test it"

# (5) `roubos` devolve sempre vazio: a asserção principal fica verde para sempre, e só o CONTROLO
#     POSITIVO no fim do gate a apanha.
bloco "roubos: devolve sempre vazio" ph2d-host-desktop um_roteiro_nunca_rouba \
  "$CENSO" 1 \
  '        .filter(|k| texto.contains(&format!("`{k}`")))' \
  '        .filter(|k| texto.contains(&format!("`{k}`")) && false)' \
  "--test it"

# (6) O censo passa a varrer ZERO ficheiros — o modo de falha MUDO que o piso de população existe
#     para impedir (§5.0: um censo por prefixo de nome varre zero e fica verde).
bloco "censo: varre zero cenas" ph2d-host-desktop um_roteiro_nunca_rouba \
  "$CENSO" 1 \
  'if !nome.ends_with("_smoke.rs") {' \
  'if !nome.ends_with("_nao_existe.rs") {' \
  "--test it"

echo "════ O ROTEIRO deixa de nomear o que EXISTE ════"

# (7) O passo (5) deixa de nomear o chip do transporte: o gate irmão dos rótulos sangra.
bloco "roteiro: perde o chip `Reset`" ph2d-app-components o_roteiro_nomeia_rotulos \
  "$CENA" 1 \
  'disparar. `Reset`, ao' \
  'disparar. `Rebobinar`, ao' \
  "--lib"

# (8) E o chip `Pause`, pela outra ponta da mesma frase.
bloco "roteiro: perde o chip `Pause`" ph2d-app-components o_roteiro_nomeia_rotulos \
  "$CENA" 1 \
  'na barra de CIMA carregue em `Pause`' \
  'na barra de CIMA carregue em `Parar`' \
  "--lib"

echo
if [ "$FALHAS" = 0 ]; then
  echo "✅ $TOTAL de $TOTAL mutacoes sangraram"
else
  echo "⛔ $FALHAS de $TOTAL mutacoes NAO sangraram"
fi
exit "$FALHAS"

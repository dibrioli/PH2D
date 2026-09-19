#!/usr/bin/env bash
# Provas de mutação da W8 do SUPLENTE #22 — O CICLO (o *ping-pong*).
#
# Report do dono, 2026-09-19: *«e onde estão as opções úteis como ping-pong?»*.
#
# Arnês IDÊNTICO ao das waves anteriores — controlo sobre o próprio FILTRO (um filtro que casa ZERO
# testes sai verde e lê-se como «sobreviveu») e `muta` a ABORTAR quando a âncora não aparece o
# número esperado de vezes (*uma mutação que não entra lê-se exactamente como uma que sobreviveu*).
#
# uso:  bash scripts/ph2d-run.sh bash docs/Components/ferramentas/mutacao_tween_w8_2026-09-19.sh
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

LEI=crates/ph2d-tween/src/lib.rs
PAINEL=crates/ph2d-app-components/src/tween_inspector.rs
CENA=crates/ph2d-app-components/src/tween_smoke.rs
POPULATE=crates/ph2d-panel-inspector/src/populate_tween.rs
EVENTO=crates/ph2d-panel-inspector/src/event_tween.rs

echo "════ A LEI — a dobra, e o caminho de omissão ════"

# (1) ⭐⭐ O TRIÂNGULO ao contrário: ele passa a começar em `para`, ir a `de` e voltar — o mesmo
#     movimento com as pontas trocadas, que é o defeito mais difícil de ver numa foto.
bloco "lei: o triangulo invertido" ph2d-tween a_dobra_do_pingpong "$LEI" 1 \
  "Ciclo::PingPong => 1.0 - (2.0 * u - 1.0).abs()," \
  "Ciclo::PingPong => (2.0 * u - 1.0).abs(),"

# (2) ⭐⭐⭐ O caminho de OMISSÃO deixa de ser byte-idêntico: TODA cena gravada passa a correr em
#     ping-pong, e nenhuma delas o pediu. ⚠️ É a metade que protege o produto que já shipava.
bloco "lei: o Reinicia a dobrar tambem" ph2d-tween a_dobra_do_pingpong "$LEI" 1 \
  "            Ciclo::Reinicia => u," \
  "            Ciclo::Reinicia => 1.0 - (2.0 * u - 1.0).abs(),"

# (3) ⭐⭐ A DOBRA depois da CURVA: a ida e a volta passam a ter formas diferentes em toda curva que
#     não seja simétrica — e num *ease-in* a volta fica abrupta.
bloco "lei: a dobra depois da curva" ph2d-tween a_dobra_vem_antes_da_curva "$LEI" 1 \
  "Some(mistura(t, t.easing.eval(t.ciclo.dobra(u))))" \
  "Some(mistura(t, t.ciclo.dobra(t.easing.eval(u))))"

echo "════ O PAINEL — o chip escreve, e o instantâneo lê ════"

# (4) O dreno do ciclo cala-se: o chip acende, o barro não muda, e o artista lê «o botão não faz
#     nada» — a espécie que o §5.0 nomeia como controlo MORTO.
bloco "painel: o dreno do ciclo apagado" ph2d-app-components o_ciclo_chega_ao_componente "$PAINEL" 1 \
  "            t.ciclo = novo;
            true" \
  "            let _ = novo;
            false"

# (5) O instantâneo lê sempre `Reinicia`: o chip aceso é o errado depois de trocar de objecto, que
#     é a lei que a §11 pagou com um report.
bloco "painel: o instantaneo cravado" ph2d-app-components o_ciclo_chega_ao_componente "$PAINEL" 1 \
  "                ciclo: t.ciclo.tag()," \
  "                ciclo: 0,"

echo "════ A COSTURA — o chip está vivo sob o dedo ════"

# (6) ⭐⭐⭐ O array do ciclo sai do `populate`: os dois chips ficam PINTADOS, hit-registados e
#     MORTOS SOB O DEDO — o defeito que este repo já pagou sete vezes, e que nenhum dos gates
#     anteriores desta secção podia ver (eles entram abaixo do store).
bloco "costura: o ciclo fora do populate" ph2d-panel-inspector seam_tween "$POPULATE" 1 \
  "        .chain(ids::INSP_TWEEN_CICLO.iter())
" \
  ""

# (7) O braço do ciclo sai da tabela do despacho: o clique chega e não produz edição nenhuma.
bloco "costura: o braco do ciclo fora do despacho" ph2d-panel-inspector seam_tween "$EVENTO" 1 \
  "            (&crate::ids::INSP_TWEEN_CICLO[..], &|i, n| {
                TweenFieldEdit::Ciclo(i, n)
            }),
" \
  ""

# (8) ⚠️ Todo chip passa a escrever a PRIMEIRA opção: eles ficam todos vivos, e cinco das seis
#     famílias passam a ler-se como «o botão não faz nada».
bloco "costura: a tag trocada pelo zero" ph2d-panel-inspector seam_tween "$EVENTO" 1 \
  "                push(host, bits, faz(i, n));" \
  "                push(host, bits, faz(i, 0));"

echo "════ A CENA — ela MOSTRA o que o chip faz ════"

# (9) A coluna volta a `Reinicia`: o ciclo existe, tem lei, tem gates — e a cena não o mostra, que
#     é o estado exacto em que o dono perguntou por ele.
bloco "cena: a coluna sem pingpong" ph2d-app-components uma_coluna_demonstra_o_pingpong "$CENA" 1 \
  "            ciclo: ph2d_tween::Ciclo::PingPong," \
  "            ciclo: ph2d_tween::Ciclo::Reinicia,"

echo
if [ "$FALHAS" = 0 ]; then
  echo "✅ $TOTAL de $TOTAL mutacoes sangraram"
else
  echo "⛔ $FALHAS de $TOTAL mutacoes NAO sangraram"
fi
exit "$FALHAS"

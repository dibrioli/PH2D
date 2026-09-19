#!/usr/bin/env bash
# Provas de mutação da W7 do SUPLENTE #22 — O PAINEL DIZ ONDE MORA O TEMPO.
#
# Report do dono, 2026-09-19, no smoke da `PH2D_TWEEN_SMOKE=1`: *«onde selecciono o tempo?»*.
#
# Arnês IDÊNTICO ao do `mutacao_tween_2026-09-19.sh` — com o controlo sobre o próprio FILTRO (um
# filtro que casa ZERO testes sai verde e lê-se como «sobreviveu») e com a `muta` a ABORTAR quando
# a âncora não aparece o número esperado de vezes (*uma mutação que não entra lê-se exactamente
# como uma que sobreviveu*).
#
# uso:  bash scripts/ph2d-run.sh bash docs/Components/ferramentas/mutacao_tween_w7_2026-09-19.sh
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

PAINEL=crates/ph2d-app-components/src/tween_inspector.rs
MODELO=crates/ph2d-editor-core/src/tween_edits.rs
EDITOR=crates/ph2d-panel-inspector/src/sections/tween_editor.rs
I18N=crates/ph2d-i18n/src/inspector_game.rs
SECCOES=crates/ph2d-editor-core/src/ids/live_sections.rs

echo "════ O INSTANTÂNEO — a duração vem do relógio do MESMO índice ════"

# (1) ⭐⭐ A duração passa a ser a do PRIMEIRO relógio: com dois tweens, o painel diz ao artista que
#     os dois duram o mesmo, e mandá-lo afinar o segundo não muda nada do que ele lê.
# ⚠️ É a armadilha que a wave anterior pagou com uma mutação SOBREVIVENTE: *uma fixtura com um
#    elemento não pode testar um índice*. A fixtura deste gate tem DOIS de cada, com durações
#    diferentes, e é isso que a torna capaz de a matar.
bloco "instantaneo: o indice trocado pelo zero" ph2d-app-components a_duracao_que_o_painel_mostra \
  "$PAINEL" 1 \
  ".and_then(|ts| ts.0.get(i))" \
  ".and_then(|ts| ts.0.first())"

# (2) A coluna desaparece: o painel volta a não saber o número, e a resposta ao dono some com ela.
bloco "instantaneo: a duracao por ler" ph2d-app-components a_duracao_que_o_painel_mostra \
  "$PAINEL" 1 \
  "duracao_us: relogios
                    .as_ref()
                    .and_then(|ts| ts.0.get(i))
                    .map(|t| t.duration_us)," \
  "duracao_us: None,"

echo "════ A QUEIXA — um relógio a ZERO não é a ausência de relógio ════"

# (3) ⭐⭐⭐ As duas queixas colapsam numa: quem tem um relógio parado no zero passa a ler *«não há
#     relógio neste índice»* e vai anexar um SEGUNDO timer — ficando com dois, e um deles mudo.
#     *As duas curas ficam em sítios diferentes, e é por isso que não podem partilhar a frase.*
bloco "modelo: o zero lido como ausencia" ph2d-app-components um_relogio_a_zero_queixa_se \
  "$MODELO" 1 \
  "        if duracao_us == 0 {
            return Some(TweenQueixa::RelogioSemDuracao);
        }" \
  "        let _ = duracao_us;"

echo "════ O EDITOR — a frase é PINTADA, e depois dos presets ════"

# (4) O readout evapora-se: o painel volta a saber o número e a não o dizer, que é o estado exacto
#     em que o dono fez a pergunta.
bloco "editor: o readout apagado" ph2d-panel-inspector o_editor_le_a_duracao \
  "$EDITOR" 1 \
  "    if let Some(us) = row.duracao_us {" \
  "    if let Some(us) = None::<u64> {"

# (5) ⚠️ A frase cresce e passa a ser CORTADA pela coluna — *uma resposta cortada a meio é a mesma
#     pergunta outra vez*. A régua é o sistema de texto REAL, à largura de omissão do painel.
bloco "i18n: a frase que nao cabe" ph2d-panel-inspector a_frase_do_tempo_cabe \
  "$I18N" 1 \
  '            "Duration {s} s — set in Timer {n}, above."' \
  '            "Duration {s} s — set it in the Timers section, timer {n}, above."'

echo "════ A POSIÇÃO — o «above» é uma afirmação, não uma suposição ════"

# (6) ⭐ A entrada dos TIMERS sai da tabela que o painel pinta (a lista tem comprimento fixo, logo
#     uma troca a sério pede duas edições — esta é a forma de UMA que produz o mesmo defeito): a
#     frase continua a dizer *«above»* sobre uma secção cuja posição a tabela já não garante.
# ⚠️ *Uma palavra de POSIÇÃO envelhece sozinha* — e sem este gate ela envelheceria em silêncio.
bloco "seccoes: os relogios abaixo do tween" ph2d-panel-inspector a_seccao_dos_relogios_esta_mesmo \
  "$SECCOES" 1 \
  "    (INSP_LIVE_TIMER_SECTION, INSP_LIVE_TIMER_COLOR)," \
  "    (INSP_LIVE_TWEEN_SECTION, INSP_LIVE_TWEEN_COLOR),"

echo
if [ "$FALHAS" = 0 ]; then
  echo "✅ $TOTAL de $TOTAL mutacoes sangraram"
else
  echo "⛔ $FALHAS de $TOTAL mutacoes NAO sangraram"
fi
exit "$FALHAS"

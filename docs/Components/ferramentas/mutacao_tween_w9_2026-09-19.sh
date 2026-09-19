#!/usr/bin/env bash
# Provas de mutação da W9 do SUPLENTE #22 — O RELÓGIO DENTRO DA SECÇÃO.
#
# Report do dono, 2026-09-19: *«porque usar timer para isso? por que não embutir na própria
# secção?»*.
#
# Arnês IDÊNTICO ao das waves anteriores — controlo sobre o próprio FILTRO e `muta` a ABORTAR
# quando a âncora não aparece o número esperado de vezes.
#
# uso:  bash scripts/ph2d-run.sh bash docs/Components/ferramentas/mutacao_tween_w9_2026-09-19.sh
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
EVENTO=crates/ph2d-panel-inspector/src/event_tween.rs
SEMENTE=crates/ph2d-panel-inspector/src/sync_tween.rs
EDITOR=crates/ph2d-panel-inspector/src/sections/tween_editor.rs

echo "════ O TWEEN NASCE COM RELÓGIO ════"

# (1) ⭐⭐ O acrescimo cala-se: o SEGUNDO tween de um objecto nasce inerte, e a cura vive noutra
#     seccao — que e' o estado exacto em que o dono fez a pergunta.
bloco "add: o relogio por nascer" ph2d-app-components um_tween_nasce_com_relogio "$PAINEL" 1 \
  "                ts.0.push(ph2d_ecs::Timer::default());" \
  "                let _ = &mut ts;"

# (2) ⛔ Ele passa a escrever POR CIMA do relogio 0: uma recarga, uma cutscene ou um sinal que
#     dependessem daquele timer sao reescritos em silencio ao acrescentar um tween.
bloco "add: o relogio 0 esmagado" ph2d-app-components um_tween_nasce_com_relogio "$PAINEL" 1 \
  "                ts.0.push(ph2d_ecs::Timer::default());" \
  "                ts.0[0] = ph2d_ecs::Timer::default();"

echo "════ O INSTANTÂNEO — as duas colunas novas, no índice CERTO ════"

# (3) As caixas passam a ler o PRIMEIRO relogio: com dois tweens, as duas linhas mostram o mesmo
#     estado e mexer numa delas parece nao fazer nada.
bloco "instantaneo: o repeat do primeiro" ph2d-app-components as_colunas_do_relogio_vem_do_indice \
  "$PAINEL" 1 \
  "                repeat: relogio(relogios, i).is_some_and(|t| t.repeat)," \
  "                repeat: relogio(relogios, 0).is_some_and(|t| t.repeat),"

echo "════ A COSTURA — os três controlos chegam à porta dos TIMERS ════"

# (4) ⭐⭐⭐ As duas caixas voltam ao ramo do CLIQUE, que e' o evento ERRADO: elas ficam pintadas,
#     vivas sob o dedo, e **inalcancaveis** — e foi assim que a 1.a redaccao desta wave as escreveu.
bloco "costura: a caixa no evento errado" ph2d-panel-inspector seam_tween "$EVENTO" 1 \
  "    if let WidgetEvent::Toggled(id) = ev
        && let Some(r) = info.rows.get(aberto)" \
  "    if let WidgetEvent::Click(id) = ev
        && let Some(r) = info.rows.get(aberto)"

# (5) A DURACAO sai do despacho: o campo aceita o numero e ele nao chega ao relogio.
bloco "costura: a duracao fora do despacho" ph2d-panel-inspector seam_tween "$EVENTO" 1 \
  "        if id == crate::ids::INSP_TWEEN_DURACAO {
            relogio(host, bits, TimerFieldEdit::DurationSecs(i, f));
            return true;
        }
" \
  ""

# (6) ⚠️ O indice trocado pelo ZERO: mexer no relogio do segundo tween escreve no do primeiro, e as
#     duas linhas passam a partilhar um relogio que so' uma delas usa.
# ⛔⛔ **Ela SOBREVIVEU a` 1.a corrida, e a culpa era da FIXTURA do gate:** ele tinha UMA linha, logo
#     `i` era `0` e a mutacao era um no-op. *Uma fixtura com um elemento nao pode testar um indice*
#     — a MESMA licao que a W6 desta linha ja' tinha pago, paga outra vez. O gate passou a abrir a
#     SEGUNDA linha, pelo clique REAL nela.
bloco "costura: o indice trocado pelo zero" ph2d-panel-inspector seam_tween "$EVENTO" 1 \
  "            relogio(host, bits, TimerFieldEdit::Repeat(i, !r.repeat));" \
  "            relogio(host, bits, TimerFieldEdit::Repeat(0, !r.repeat));"

# (7) ⛔ O estado de partida da caixa passa a vir do STORE: o primeiro clique depois de trocar de
#     objecto manda o valor do objecto ANTERIOR — a lei que a §11 pagou com um report.
bloco "costura: a caixa lida do snapshot invertida" ph2d-panel-inspector seam_tween "$EVENTO" 1 \
  "            relogio(host, bits, TimerFieldEdit::Autostart(i, !r.autostart));" \
  "            relogio(host, bits, TimerFieldEdit::Autostart(i, r.autostart));"

echo "════ A SEMENTE — o campo mostra o objecto, não a fábrica ════"

# (8) ⭐⭐ A semente cala-se: o campo mostra `1 s` (o default do `populate`) sobre um relogio de
#     `0,4 s`. ⚠️ *Numeros plausiveis sao a pior forma deste defeito.*
bloco "semente: a duracao por semear" ph2d-panel-inspector seam_tween "$SEMENTE" 1 \
  "            host.store_mut().set_number_value(id, us as f64 / 1e6);" \
  "            let _ = us;"

echo "════ O PINTOR — o bloco existe, e vem depois dos presets ════"

# (9) O bloco do relogio desaparece do editor: o painel volta a saber o numero e a nao o mostrar.
bloco "pintor: o bloco do relogio apagado" ph2d-panel-inspector o_editor_pinta_o_relogio "$EDITOR" 1 \
  "    if row.duracao_us.is_some() {" \
  "    if false {"

echo
if [ "$FALHAS" = 0 ]; then
  echo "✅ $TOTAL de $TOTAL mutacoes sangraram"
else
  echo "⛔ $FALHAS de $TOTAL mutacoes NAO sangraram"
fi
exit "$FALHAS"

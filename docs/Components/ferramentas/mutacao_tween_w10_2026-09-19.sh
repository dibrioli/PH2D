#!/usr/bin/env bash
# Provas de mutação da W10 do SUPLENTE #22 — A COR É UMA COR, E NENHUM CHIP MENTE.
#
# Report do dono, 2026-09-19 (com foto): *«por que usar cores em números se temos caixas
# selectoras?»* — e a mesma foto mostrava `Silhoue...` · `Positio...` · `Positio...`.
#
# Arnês IDÊNTICO ao das waves anteriores — controlo sobre o próprio FILTRO e `muta` a ABORTAR
# quando a âncora não aparece o número esperado de vezes.
#
# uso:  bash scripts/ph2d-run.sh bash docs/Components/ferramentas/mutacao_tween_w10_2026-09-19.sh
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

CANAL=crates/ph2d-tween/src/canal.rs
PAINEL=crates/ph2d-app-components/src/tween_inspector.rs
EDITOR=crates/ph2d-panel-inspector/src/sections/tween_editor.rs
EVENTO=crates/ph2d-panel-inspector/src/event_tween.rs
SEMENTE=crates/ph2d-panel-inspector/src/sync_tween.rs
POPULATE=crates/ph2d-panel-inspector/src/populate_tween.rs

echo "════ OS CHIPS — nenhum sai cortado ════"

# (1) ⭐⭐⭐ Volta a constante `4`: o grupo dos CANAIS corta `Silhouette`, `Position X` e
#     `Position Y`, e os dois ultimos passam a ler-se «Positio...» OS DOIS — indistinguiveis sob o
#     dedo. E' a foto do dono, a` letra.
bloco "chips: a constante de volta" ph2d-panel-inspector o_chip_pintado_cabe_o_rotulo "$EDITOR" 1 \
  "    let por_fileira = cabem_por_fileira(text_system, w, labels);" \
  "    let por_fileira = CHIPS_POR_FILEIRA;"

# ⛔⛔ **DUAS MUTACOES FORAM RETIRADAS, e o motivo e' o proprio desenho** (medidas 19/09):
#
#   * **a PORTA do botao trocada para `Sm`** — ela move o produto E a regua ao mesmo tempo, porque
#     os dois a leem. O texto fica mais pequeno e os chips encolhem com ele: *a lei continua
#     verdadeira*. **E' exactamente isso que a porta existe para fazer**, e uma mutacao que nao
#     quebra a lei nao tem de sangrar.
#
#   * **a regua interna trocada para `Sm`** — com os rotulos deste corpus ela ainda devolve `3` por
#     fileira (o `mais_largo` encolhe mas a folga nao), logo a saida nao muda. *Uma mutacao que este
#     corpus nao discrimina fica NOMEADA aqui em vez de fingida* — quem lhe quiser um gate precisa
#     de uma familia cujos rotulos caiam exactamente entre as duas fontes.

echo "════ A COR — a LEI e o DRENO ════"

# (4) ⭐⭐ O `e_cor` deixa de ser derivado e responde sempre NAO: o `Tint` volta aos quatro campos
#     numericos, que e' o estado exacto em que o dono fez a pergunta.
bloco "lei: nenhum canal e' cor" ph2d-app-components e_cor_e_derivado_da_aridade "$CANAL" 1 \
  "        self.aridade() == 4" \
  "        false"

# (5) O dreno da cor cala-se: a amostra abre, o artista escolhe, e o barro nao muda.
bloco "painel: o dreno da cor apagado" ph2d-app-components uma_cor_chega_ao_componente "$PAINEL" 1 \
  "            *alvo = *rgba;
            true" \
  "            let _ = rgba;
            false"

# (6) ⛔ A cor escrita no extremo ERRADO: escolher o `de` reescreve o `para`, e o tween passa a
#     nao ir de lado nenhum para lado nenhum.
bloco "painel: a cor no extremo errado" ph2d-app-components uma_cor_chega_ao_componente "$PAINEL" 1 \
  "            let alvo = if *fim { &mut t.para } else { &mut t.de };" \
  "            let alvo = if *fim { &mut t.de } else { &mut t.para };"

echo "════ A COSTURA — a amostra abre, e a escolha CHEGA ════"

# (7) ⭐⭐⭐ As duas amostras saem do `populate`: elas ficam PINTADAS e hit-registadas, e o clique
#     morre no `is_focusable` — **sem o selector abrir**. O defeito que este repo ja' pagou sete
#     vezes, e que nenhum gate que nao faca o gesto REAL consegue ver.
bloco "costura: as amostras fora do populate" ph2d-panel-inspector seam_tween "$POPULATE" 1 \
  "    for id in [ids::INSP_TWEEN_COR_DE, ids::INSP_TWEEN_COR_PARA] {
        store.register(id, InteractiveState::Plain);
    }
" \
  ""

# (8) O selector abre com a cor ERRADA (a do outro extremo): o artista comeca a escolher a partir
#     de um sitio que nao e' o dele.
bloco "costura: o selector semeado ao contrario" ph2d-panel-inspector seam_tween "$EVENTO" 1 \
  "            let atual = if fim { r.para } else { r.de };" \
  "            let atual = if fim { r.de } else { r.para };"

# (9) ⛔⛔ O bloco da cor volta para DEPOIS da saida antecipada da semente: enquanto o artista
#     escolhe, o documento nao mudou, logo a assinatura e' a mesma e a cor **morre no
#     `widget_color`**. *O fio fica completo ate' ao ultimo passo.*
bloco "semente: a cor depois da saida antecipada" ph2d-panel-inspector a_cor_escolhida_chega \
  "$SEMENTE" 1 \
  "    let picker = host.store().picker_target();" \
  "    let picker: Option<ph2d_a11y::NodeId> = None;"

echo "════ O PINTOR — os dois caminhos são EXCLUSIVOS ════"

# (10) O ramo da cor deixa de existir: um `Tint` volta a pintar oito numeros.
bloco "pintor: o ramo da cor apagado" ph2d-panel-inspector um_canal_de_cor_pinta_amostras \
  "$EDITOR" 1 \
  "    if canal.e_cor() {" \
  "    if false {"

echo
if [ "$FALHAS" = 0 ]; then
  echo "✅ $TOTAL de $TOTAL mutacoes sangraram"
else
  echo "⛔ $FALHAS de $TOTAL mutacoes NAO sangraram"
fi
exit "$FALHAS"

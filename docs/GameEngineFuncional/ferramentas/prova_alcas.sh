#!/bin/bash
# A prova ponta a ponta das alças da forma: o App real, pela mão de `PH2D_GAME_SMOKE_FOTO=fisica_forma_*` (botão, ponteiro
# e Esc pelas portas reais), e o que `[forma-mao]` diz em cada instante. Reprova com exit 1. Precisa de placa: a `App`
# segura uma surface de janela, sem isso não corre em `cargo test` (ficha f1, onda 4).
#   bash docs/GameEngineFuncional/ferramentas/prova_alcas.sh
cd "$(dirname "$0")/../../.." || exit 2
FOTOS="$(mktemp -d)"
falhas=0
# caso | instante | o que a linha desse instante tem de conter (grep -E)
while IFS='|' read -r caso k quer; do
  [ -n "$caso" ] || continue
  if [ ! -f "$FOTOS/$caso.log" ]; then
    cena=PH2D_GAME_SMOKE=4; alt=1400
    case "$caso" in
      forma_ponta) cena=PH2D_PHYSICS_SMOKE=120; export PH2D_PHYSICS_SMOKE_SELECT="Loose Shape" ;;
      # O Fit fica abaixo do cartão da animação: a janela alta.
      forma_fit) cena=PH2D_PHYSICS_SMOKE=120; alt=2700; export PH2D_PHYSICS_SMOKE_SELECT="Player" ;;
      *) unset PH2D_PHYSICS_SMOKE_SELECT ;;
    esac
    FOTO_GUARDA_TMP=1 PH2D_GAME_SMOKE_FOTO="fisica_$caso" bash docs/Components/ferramentas/fotografa_cena.sh \
      "$cena" 1930 "$alt" "$FOTOS/$caso.png" 7 > "$FOTOS/$caso.saida" 2>&1
    sessao="$(grep -o 'sessao guardada em .*' "$FOTOS/$caso.saida" | awk '{print $4}')"
    grep '\[forma-mao\]' "$sessao/app.log" > "$FOTOS/$caso.log" 2>/dev/null
    rm -rf "$sessao"
  fi
  linha="$(grep "k=$k " "$FOTOS/$caso.log")"
  if echo "$linha" | grep -Eq "$quer"; then
    echo "ok    $caso k=$k: $quer"
  else
    echo "FALHA $caso k=$k: queria «$quer», veio: ${linha:-nada}"
    falhas=$((falhas + 1))
  fi
done <<'CASOS'
forma_arrasto|8|erro_px=0\.[0-4]
forma_chave|8|erro_px=0\.[0-4].*curvas=[1-9]
forma_esc_chave|8|curvas=[1-9]
forma_esc_chave|20|half_x: 0\.5, half_y: 0\.5 \} offset=\[0\.2, 0\.1\].*editando=true · curvas=0 · passos=0
forma_esc_chave|36|editando=false · curvas=0 · passos=0
forma_desfaz|36|half_x: 0\.5, half_y: 0\.5 \} offset=\[0\.2, 0\.1\].*editando=true
forma_chave_desfaz|36|half_x: 0\.5, half_y: 0\.5 \} offset=\[0\.2, 0\.1\].*editando=true · curvas=0
forma_clique|8|editando=true
forma_clique|20|editando=false
forma_ponta|8|half_x: 4\.0, half_y: 0\.25.*alvo=Loose Shape editando=true
forma_ponta|20|alvo=Updraft Zone editando=false
forma_fit|antes|Capsule \{ half_height: 0\.15, radius: 0\.2 \}.*passos=0
forma_fit|8|Capsule \{ half_height: 0\.0, radius: 0\.5 \}.*passos=1
forma_fit|36|Capsule \{ half_height: 0\.15, radius: 0\.2 \}.*passos=0
CASOS
rm -rf "$FOTOS"
[ "$falhas" = 0 ] && echo "prova_alcas: verde" || { echo "prova_alcas: $falhas falha(s)"; exit 1; }

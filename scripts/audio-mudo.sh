#!/usr/bin/env bash
# audio-mudo.sh — «o mixer conta vozes vivas e eu NÃO OUÇO»: as 4 causas, medidas em 1 s.
#
# WHY this exists (TRÊS ocorrências: 2026-07-08, 2026-07-15, 2026-09-09). O app produz
# sinal — o medidor do master mexe, o `[audio-2d]` conta vozes vivas — e não sai som.
# ⛔ **Nenhuma das três foi defeito de código.** É o PipeWire/WirePlumber, e são causas
# DISTINTAS que dão exactamente o mesmo sintoma, o que faz cada uma custar uma jornada:
#
#   1. O WirePlumber guarda mute/volume POR-APP em `~/.local/state/wireplumber/
#      stream-properties` e RE-APLICA a cada abertura. Um `"mute":true` ali cala o app
#      para sempre — com volume 1.0, medidor vivo e o sink do sistema perfeito.
#      (2026-07-08 e 2026-09-09; nas duas o `ph2d-host-desktop` era a entrada mute:true)
#   2. O sink default está BAIXO demais: 24% = −37 dB, e com conteúdo a −15 dB o total
#      (~−52 dB) é inaudível. (2026-07-15)
#   3. O sink default está MUTADO.
#   4. O sink default é a saída ERRADA — este DAC tem três (Speakers / Front Headphones
#      / S/PDIF) e todo app nativo (cpal → pipewire-alsa) vai para o DEFAULT.
#
# ⚠️⚠️ **O app não consegue medir nenhuma delas.** O mute vive no nó do PipeWire, a
# JUSANTE do cpal: do lado de cá o `write_out` entrega o buffer e devolve sucesso. É por
# isso que o instrumento é um script e não uma linha de log — e é por isso que a linha
# `[audio-2d] … pico do master` do app só responde METADE da pergunta (ela diz que o som
# CHEGA à saída, nunca que a saída o ENTREGA).
#
# ⛔ A causa 4 não se cura sozinha: qual saída tem as colunas ligadas é o único facto
# aqui que a máquina não sabe. Ela é MEDIDA e apresentada; escolher é do Enio.
#
# USAGE
#   bash scripts/audio-mudo.sh           # mede e imprime o comando que cura
#   bash scripts/audio-mudo.sh --curar   # aplica a cura das causas 1..3
#
# EXIT  0 = nada a apontar · 1 = achou (ou curou) alguma causa

set -uo pipefail

CURAR=0
[ "${1:-}" = "--curar" ] && CURAR=1

STATE="$HOME/.local/state/wireplumber/stream-properties"
ACHADOS=0
CURAS=()

say()  { printf '%s\n' "$*"; }
bad()  { printf '  \033[31m✗\033[0m %s\n' "$*"; ACHADOS=$((ACHADOS + 1)); }
good() { printf '  \033[32m✓\033[0m %s\n' "$*"; }

say ""
say "audio-mudo.sh — porque é que o som do PH2D não chega aos ouvidos"
say "════════════════════════════════════════════════════════════════"

# ── 1) O WirePlumber tem o APP mutado no ficheiro que ele re-aplica ──────────────
say ""
say "1) mute do APP guardado pelo WirePlumber"
if [ ! -f "$STATE" ]; then
  good "não há ficheiro de estado ($STATE) — nada guardado"
else
  MUDAS=$(grep -ci 'ph2d.*"mute":true' "$STATE" 2>/dev/null || true)
  if [ "${MUDAS:-0}" -gt 0 ]; then
    bad "$MUDAS entrada(s) do ph2d com \"mute\":true — o app abre MUDO, sempre"
    grep -in 'ph2d.*"mute":true' "$STATE" | sed 's/^/      /'
    CURAS+=("systemctl --user stop wireplumber && sed -i '/ph2d/s/\"mute\":true/\"mute\":false/g' \"$STATE\" && systemctl --user start wireplumber")
    if [ "$CURAR" = 1 ]; then
      systemctl --user stop wireplumber
      sed -i '/ph2d/s/"mute":true/"mute":false/g' "$STATE"
      RESTAM=$(grep -ci 'ph2d.*"mute":true' "$STATE" 2>/dev/null || true)
      systemctl --user start wireplumber
      sleep 1
      # ⚠️ O `sed -i` num padrão que não casa é um NO-OP SILENCIOSO: confira a conta.
      if [ "${RESTAM:-0}" = 0 ]; then
        good "CURADO: as $MUDAS entradas do ph2d passaram a \"mute\":false"
      else
        bad "a cura NÃO pegou — restam $RESTAM (formato do ficheiro mudou?)"
      fi
    fi
  else
    good "nenhuma entrada do ph2d está mutada"
  fi
fi

# ── 2 e 3) O sink default: volume e mute ────────────────────────────────────────
say ""
say "2) o sink default (para onde todo app nativo vai)"
if ! command -v wpctl >/dev/null 2>&1; then
  bad "wpctl não existe nesta máquina — não sei medir o sink"
else
  VOLLINE=$(wpctl get-volume @DEFAULT_AUDIO_SINK@ 2>/dev/null || true)
  DEFAULT=$(wpctl status 2>/dev/null | sed -n '/Sinks:/,/Sources:/p' | grep -F '*' | head -1 | sed 's/[│├─]//g; s/^ *//; s/ *$//')
  say "      $DEFAULT"
  VOL=$(printf '%s' "$VOLLINE" | awk '{print $2}')
  if printf '%s' "$VOLLINE" | grep -q 'MUTED'; then
    bad "o sink default está MUTADO"
    CURAS+=("wpctl set-mute @DEFAULT_AUDIO_SINK@ 0")
    if [ "$CURAR" = 1 ]; then wpctl set-mute @DEFAULT_AUDIO_SINK@ 0 && good "CURADO: sink desmutado"; fi
  elif [ -n "${VOL:-}" ] && awk -v v="$VOL" 'BEGIN{exit !(v < 0.30)}'; then
    bad "volume do sink em $VOL — abaixo de 0,30 um som a −15 dB é inaudível (2026-07-15)"
    CURAS+=("wpctl set-volume @DEFAULT_AUDIO_SINK@ 0.8")
    if [ "$CURAR" = 1 ]; then wpctl set-volume @DEFAULT_AUDIO_SINK@ 0.8 && good "CURADO: volume a 0.8"; fi
  else
    good "volume $VOL, não mutado"
  fi

  # ── 4) É a saída CERTA? Medida e apresentada — a escolha é do Enio ────────────
  say ""
  say "3) as outras saídas desta máquina (se as colunas estão NOUTRA, é aqui)"
  wpctl status 2>/dev/null | sed -n '/Sinks:/,/Sources:/p' | grep -E '[0-9]+\.' | sed 's/^/      /'
  say "      ⇒ para trocar:  wpctl set-default <número>"
  say "      ⇒ para provar qual é:  wpctl set-default <número> && paplay /usr/share/sounds/freedesktop/stereo/complete.oga"
fi

# ── 5) O stream VIVO, se o app estiver aberto ───────────────────────────────────
say ""
say "4) o stream vivo do app (só existe com o PH2D aberto)"
if ! command -v pactl >/dev/null 2>&1; then
  say "      pactl não existe — saltado"
else
  IDX=$(pactl list sink-inputs 2>/dev/null | awk '/^Sink Input #/{i=substr($3,2)} /application\.name/{if ($0 ~ /ph2d/) print i}' | head -1)
  if [ -z "${IDX:-}" ]; then
    say "      o PH2D não está a tocar agora — nada a medir aqui"
  else
    MUTE=$(pactl list sink-inputs 2>/dev/null | awk -v n="$IDX" '/^Sink Input #/{c=(substr($3,2)==n)} c && /Mute:/{print $2; exit}')
    if [ "$MUTE" = "yes" ]; then
      bad "o stream vivo do app (#$IDX) está MUTADO"
      CURAS+=("pactl set-sink-input-mute $IDX 0")
      if [ "$CURAR" = 1 ]; then pactl set-sink-input-mute "$IDX" 0 && good "CURADO: stream #$IDX desmutado"; fi
    else
      good "stream #$IDX não está mutado"
    fi
  fi
fi

say ""
say "════════════════════════════════════════════════════════════════"
if [ "$ACHADOS" = 0 ]; then
  say "NADA A APONTAR do lado do sistema."
  say "⇒ Se mesmo assim não ouve, a pergunta seguinte é se o som CHEGA à saída:"
  say "  a linha [audio-2d] do app diz 'pico do master L… R…'. A zero, o defeito é do app."
  exit 0
fi
if [ "$CURAR" = 1 ]; then
  say "$ACHADOS causa(s) tratada(s). Abra o app e ouça."
else
  say "$ACHADOS causa(s). Para curar de uma vez:"
  say ""
  say "  bash scripts/audio-mudo.sh --curar"
  say ""
  say "ou, uma a uma:"
  for c in "${CURAS[@]}"; do say "  $c"; done
fi
exit 1

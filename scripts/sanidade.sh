#!/usr/bin/env bash
# sanidade.sh — o vigia da máquina. Cala-se quando está tudo bem.
#
# ============================================================================
# POR QUE ISTO NÃO É "MAIS UM SCRIPT DE DIAGNÓSTICO"
# ============================================================================
#
# MEDIDO em 2026-09-01 sobre 239 209 comandos reais de 83 sessões: as sondas de
# saúde deste repo foram invocadas **177 vezes = 0,07%**. O caso decisivo é o
# `ph2d-check-memoria`, escrito depois dos travamentos de 08/08 para exatamente
# este fim: **1 invocação na vida**, contra 510 `free`/`df` digitados à mão.
# O `git-stage-guard.sh` tem 5 docs a apontá-lo e **zero**.
#
# ⇒ Uma ferramenta que é preciso LEMBRAR de correr não é corrida. Este script
# existe para ser chamado por um TIMER, não por uma pessoa. O modo humano
# (`--agora`) é a exceção, não o desenho.
#
# ⚠️ E o segundo defeito da família antiga: **nenhuma daquelas sondas media a
# grandeza que de facto matou a máquina.** O `btrfs-health` olha disco, o
# `ph2d-check-memoria` olha marcas d'água — e os dois travamentos de 30/08
# foram por **memória contígua**, que ninguém media. Um painel de números certos
# sobre coisas erradas dá confiança falsa.
#
# ============================================================================
# CADA VERIFICAÇÃO VEM DE UM INCIDENTE REAL DESTA MÁQUINA
# ============================================================================
#
#   contiguidade .......... 30/08 11:09 e 18:35. `0*2048kB 0*4096kB` com 68 GB
#                           livres: kcompactd 495 s preso, e o driver da NVIDIA
#                           a levar -ENOMEM num pedido de 2 MB e a não o tratar.
#   páginas fincadas ...... a MESMA causa, vista pelo lado que a nomeia:
#                           `Unevictable` acompanhava o uso de uma tmpfs
#                           `noswap` ao kB. É a assinatura da doença a voltar.
#   swap só-zram .......... 22/08. tmpfs swappável + zram como ÚNICO swap = o
#                           excedente comprime para dentro da própria RAM e o
#                           swap chega a 100% com 61 GiB de RAM livre.
#   RAM disponível ........ 08/08. Livelock de recuperação com 32 núcleos presos
#                           em direct reclaim; sem OOM, ninguém morre, tudo para.
#   disco cheio ........... disco cheio corrompe `.o` e o mold rebenta com
#                           SIGBUS — falha que se lê como bug de compilador.
#   metadata do btrfs ..... delegada ao `btrfs-health.sh`, que é o dono dela.
#                           ⛔ Não se reimplementa aqui: duas respostas para a
#                           mesma pergunta e a que envelhece é a que se lê.
#
# ⚠️⚠️ A RÉGUA DA CONTIGUIDADE ESTEVE ERRADA **DUAS** VEZES, e as duas correcções
# vieram de a correr contra a máquina real, não de a pensar:
#
#   1ª — CONTAVA os blocos de 2 MB, e uma máquina saudável mostra `4`. O buddy
#        allocator FUNDE dois blocos de 2 MB vizinhos num de 4 MB, e cada bloco
#        de 4 MB serve um pedido de 2 MB partindo-se ao meio. A leitura ingénua
#        gritava "quase a travar" sobre **86 GB** livres.
#   2ª — passou a somar o total contíguo (≥ ordem 9) e **deu alarme falso na 1ª
#        corrida a sério**: `1,2 GB`, com a máquina perfeitamente sã. Medido:
#
#          agora       livre = 4,2 GB   contíguo = 0,18 GB   → 4,4% do livre
#          travamento  livre = 68,0 GB  contíguo = 0,00 GB   → 0,0% do livre
#
#        Os dois lêem-se iguais em GB absolutos e são **opostos**: hoje só há
#        4 GB livres porque 101 GB estão em **cache** — recuperável e movível, e
#        o kernel fabrica um bloco de 2 MB assim que alguém pedir. No travamento
#        havia **68 GB LIVRES que ele não conseguia juntar**. Memória em cache é
#        memória a trabalhar; memória livre e inutilizável é a doença.
#
# ⇒ A grandeza honesta é a **FRAÇÃO do livre que está em pedaço grande**, e ela
#   só faz sentido quando há livre que chegue para a pergunta ter sentido (daí o
#   piso de 8 GB: abaixo disso a resposta certa é "a RAM está em uso", que é
#   outra verificação, a da memória disponível).
#   ⛔ Não volte a alarmar sobre os GB contíguos em absoluto: é a 3ª versão desta
#   régua, e as duas primeiras liam a máquina sã como moribunda.
#
# ⏳ OS LIMITES SÃO PROVISÓRIOS, E ISSO ESTÁ DECLARADO. Há dois pontos medidos —
# **0,0 GB = máquina morta** (30/08) e **86,1 GB = saudável** (01/09) — e a curva
# entre eles não. Por isso o vigia GRAVA sempre (`~/.ph2d/sanidade.log`): ao fim
# de duas semanas o limite deixa de ser palpite e passa a sair de `--historico`.
#
# ============================================================================
# USO
# ============================================================================
#   bash scripts/sanidade.sh              # quadro completo, para uma pessoa
#   bash scripts/sanidade.sh --vigia      # modo timer: calado se estiver bem
#   bash scripts/sanidade.sh --historico  # a curva gravada até agora
#   bash scripts/sanidade.sh --instalar   # arma o timer de 15 min (systemd user)
#   bash scripts/sanidade.sh --desinstalar
#
# Sai com  0 = bem  ·  1 = aviso  ·  2 = crítico.
# ============================================================================
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIR="$HOME/.ph2d"; LOG="$DIR/sanidade.log"; ESTADO="$DIR/sanidade.estado"
mkdir -p "$DIR"

MODO="${1:---agora}"
AVISOS=(); CRITICOS=(); ACOES=()

# ⚠️ O formato do registo é DECLARADO e o `--historico` recusa linhas que não o
# cumpram. Acrescentar uma coluna sem mexer aqui faz o passado inteiro ler-se
# deslocado — aconteceu na 1ª hora deste script.
COLUNAS=11
CABECALHO='# quando,contiguo_GB,livre_GB,fracao_pct,arrumador_nucleos,fincado_kB,disponivel_GB,zram_pct,swap_disco,disco_pct,nivel'
if [ ! -s "$LOG" ] || [ "$(head -1 "$LOG")" != "$CABECALHO" ]; then
  # formato novo: arquiva o que houver em vez de o misturar
  [ -s "$LOG" ] && mv "$LOG" "$LOG.$(date +%Y%m%d-%H%M%S).antigo"
  echo "$CABECALHO" > "$LOG"
fi

aviso()   { AVISOS+=("$1");   ACOES+=("$2"); }
critico() { CRITICOS+=("$1"); ACOES+=("$2"); }

# ---------------------------------------------------------------- medições --
# ⚠️ soma TODAS as linhas de zona (uma máquina com 2 nós NUMA tem várias) — ler
# só a primeira daria metade da resposta e ela pareceria pior do que é.
eval "$(awk '/Normal/{
  for (i=5; i<=NF; i++) livre += $i * 4096 * 2^(i-5);   # $5..$NF = ordens 0..10
  contig += ($(NF-1)*2 + $NF*4) * 1048576;              # ordens 9 e 10, em bytes
} END{
  printf "CONTIG=%.1f\nLIVRE_GB=%.1f\nFRACAO=%.1f\n", contig/2^30, livre/2^30,
         (livre>0 ? 100*contig/livre : 100)
}' /proc/buddyinfo)"
UNEV_KB=$(awk '/^Unevictable/{print $2}' /proc/meminfo)
AVAIL_GB=$(awk '/^MemAvailable/{printf "%.1f", $2/1048576}' /proc/meminfo)
SWAP_DISCO=$(swapon --show=NAME --noheadings 2>/dev/null | grep -cv zram || true)
ZRAM_PCT=$(awk '/zram/{ if ($3+0>0) printf "%d", 100*$4/$3; else print 0 }' /proc/swaps 2>/dev/null | head -1)
ZRAM_PCT=${ZRAM_PCT:-0}
NOSWAP_TMPFS=$(awk '$3=="tmpfs" && $4 ~ /noswap/ {print $2}' /proc/mounts | grep -v '^/run/credentials' || true)

# ⭐ O INDICADOR QUE ANTECEDE, e o único que estava a faltar: quanto CPU o
# `kcompactd` está a queimar. É ele que fabrica os blocos grandes, e em 30/08
# ficou **495 s preso num núcleo** (18 `soft lockup` seguidos) antes de a máquina
# morrer. Os outros números descrevem o estado; este descreve o ESFORÇO — e sobe
# muito antes de o estado ficar mau. Lê-se sem root em /proc/<pid>/stat.
KC_PID=$(pgrep -x kcompactd0 2>/dev/null | head -1)
KC_CPU=0
if [ -n "${KC_PID:-}" ] && [ -r "/proc/$KC_PID/stat" ]; then
  KC_CPU=$(awk -v hz="$(getconf CLK_TCK)" '{printf "%.2f", ($14+$15)/hz}' "/proc/$KC_PID/stat" 2>/dev/null || echo 0)
fi
AGORA_S=$(date +%s)
# taxa em núcleos: 0 = parado · 1,0 = um núcleo inteiro ocupado sem parar
KC_TAXA=""
if [ -f "$ESTADO" ]; then
  KC_ANT=$(awk -F= '/^KC_CPU=/{print $2}' "$ESTADO" 2>/dev/null)
  KC_T_ANT=$(awk -F= '/^KC_T=/{print $2}' "$ESTADO" 2>/dev/null)
  if [ -n "${KC_ANT:-}" ] && [ -n "${KC_T_ANT:-}" ] && [ "$AGORA_S" -gt "${KC_T_ANT:-0}" ]; then
    KC_TAXA=$(awk -v a="$KC_CPU" -v b="$KC_ANT" -v dt="$(( AGORA_S - KC_T_ANT ))" \
      'BEGIN{ d=a-b; if(d<0)d=0; printf "%.2f", d/dt }')
  fi
fi
DISCO_PIOR=$(df --output=pcent,target / "$ROOT" 2>/dev/null | tail -n +2 | tr -d ' %' | sort -rn | head -1)
DISCO_PCT=${DISCO_PIOR%%/*}; DISCO_PCT=${DISCO_PCT:-0}

# ------------------------------------------------------------- veredictos --
# contiguidade — a que matou a máquina 2× em 30/08.
# ⚠️ Só se pergunta quando há livre que chegue: com pouco livre a resposta certa
# é "a RAM está em uso", e quem a dá é a verificação da memória disponível.
# (a) ESFORÇO — o indicador que antecede. 495 s presos num núcleo em 30/08.
if [ -n "$KC_TAXA" ]; then
  if   awk "BEGIN{exit !($KC_TAXA > 0.90)}"; then
    critico "O arrumador de memória do sistema está preso num núcleo (${KC_TAXA} de 1,00)." \
            "É o padrão exato do travamento de 30/08. Pare metade dos agentes AGORA."
  elif awk "BEGIN{exit !($KC_TAXA > 0.50)}"; then
    aviso   "O arrumador de memória está a lutar (${KC_TAXA} de um núcleo, contínuo)." \
            "Pare 2 agentes e feche o VS Code. Não abra frentes novas."
  fi
fi

# (b) ESTADO — a assinatura exata do travamento, e só ela.
# ⚠️ O piso é ALTO de propósito (25 GB) e a fração BAIXA (2%): uma máquina
# ocupada e SÃ mantém pouco livre — o resto é cache a trabalhar — e esse pouco é
# naturalmente fragmentado. Medido em 01/09: 1,4% de 9,4 GB livres, com a máquina
# perfeitamente boa e 94 GB recuperáveis em cache. Um piso de 8 GB dava alarme
# falso em todas as corridas. O travamento tinha **68 GB livres com 0,0%**.
if awk "BEGIN{exit !($LIVRE_GB >= 25 && $FRACAO < 2)}"; then
  critico "Há ${LIVRE_GB} GB de memória livre e o sistema não consegue juntar quase nada dela (${FRACAO}%)." \
          "Feche o VS Code e pare metade dos agentes AGORA. Se não melhorar em 2 min, reinicie."
fi

# páginas fincadas — a assinatura da doença de 30/08 a voltar
if [ -n "$NOSWAP_TMPFS" ]; then
  critico "Um disco em RAM voltou a ficar 'noswap': $(echo $NOSWAP_TMPFS | tr '\n' ' ')" \
          "É a configuração que travou a máquina em 30/08. Corra: sudo bash scripts/fix-travamentos.sh"
elif [ "${UNEV_KB:-0}" -gt 1048576 ]; then
  critico "$(( UNEV_KB/1024 )) MB de memória estão fincados e o sistema não os pode mover." \
          "Corra 'bash scripts/ram-build.sh --status' e veja se o disco em RAM voltou a ficar preso."
fi

# swap — a armadilha de 22/08
if [ "${SWAP_DISCO:-0}" -eq 0 ]; then
  aviso "Não há swap em disco: o único swap é o zram, que vive dentro da RAM." \
        "Corra: sudo bash scripts/fix-travamentos.sh (ele cria o swap em disco)."
fi
if [ "${ZRAM_PCT:-0}" -ge 80 ]; then
  aviso "O zram está a ${ZRAM_PCT}% — perto de encher, que foi o travamento de 22/08." \
        "Pare alguns agentes; se houver swap em disco, ele absorve o resto."
fi

# RAM — o livelock de 08/08
if   awk "BEGIN{exit !($AVAIL_GB < 4)}"; then
  critico "Só ${AVAIL_GB} GB de memória disponível." \
          "Pare agentes já. Abaixo disto a máquina deixa de responder em vez de matar alguém."
elif awk "BEGIN{exit !($AVAIL_GB < 10)}"; then
  aviso   "Memória disponível a ${AVAIL_GB} GB." "Evite abrir mais uma frente de compilação."
fi

# disco — o mold SIGBUS
if [ "${DISCO_PCT:-0}" -ge 95 ]; then
  critico "Disco a ${DISCO_PCT}% cheio." \
          "Disco cheio corrompe ficheiros de compilação e dá erros que parecem bug de compilador. Limpe os target/."
elif [ "${DISCO_PCT:-0}" -ge 90 ]; then
  aviso   "Disco a ${DISCO_PCT}% cheio." "Corra: bash scripts/target-to-disk.sh ou limpe target/ de linhas já integradas."
fi

# metadata do btrfs — DELEGADA ao dono da pergunta
if [ -x "$ROOT/scripts/btrfs-health.sh" ]; then
  if ! bash "$ROOT/scripts/btrfs-health.sh" >/dev/null 2>&1; then
    aviso "O disco pede atenção (metadata do btrfs)." \
          "Corra: bash scripts/btrfs-health.sh — ele diz o que fazer. 'Disco cheio' com espaço livre é isto."
  fi
fi

# ------------------------------------------------------------------ estado --
NIVEL=0; [ ${#AVISOS[@]}   -gt 0 ] && NIVEL=1
         [ ${#CRITICOS[@]} -gt 0 ] && NIVEL=2

# grava SEMPRE — é isto que transforma os limites de palpite em medição
printf '%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s\n' "$(date +%FT%T)" "$CONTIG" "$LIVRE_GB" "$FRACAO" \
  "${KC_TAXA:-}" "$UNEV_KB" "$AVAIL_GB" "$ZRAM_PCT" "$SWAP_DISCO" "$DISCO_PCT" "$NIVEL" >> "$LOG"

# ⚠️ a linha de base do kcompactd grava-se em TODOS os modos: se só o `--vigia` a
# gravasse, uma corrida humana pelo meio partiria o intervalo e a taxa seguinte
# mediria outro período que não o dela.
NIVEL_ANT=$(awk -F= '/^NIVEL=/{print $2}' "$ESTADO" 2>/dev/null); NIVEL_ANT=${NIVEL_ANT:-0}
NOTIF_T=$(awk -F= '/^NOTIF_T=/{print $2}' "$ESTADO" 2>/dev/null);  NOTIF_T=${NOTIF_T:-0}
gravar_estado() {
  printf 'KC_CPU=%s\nKC_T=%s\nNIVEL=%s\nNOTIF_T=%s\n' \
    "$KC_CPU" "$AGORA_S" "$NIVEL" "$NOTIF_T" > "$ESTADO"
}

# ------------------------------------------------------------------ saídas --
case "$MODO" in
  --historico)
    # ⚠️ SALTA linhas com outro número de colunas, e a razão é medida: acrescentei
    # uma coluna a meio da sessão e o `--historico` passou a ler TODAS as linhas
    # antigas deslocadas — mostrou "CRÍTICO" e "RAM 2524 G" sobre amostras sãs.
    # Um registo cujo formato muda em silêncio mente sobre todo o seu passado.
    echo "  quando               contíg  livre  frac  arrum  fincado    RAM  zram swapD disco  nível"
    tail -40 "$LOG" | awk -F, -v n="$COLUNAS" '/^#/{next} NF==n{
      printf "  %-19s %6s %6s %5s%% %6s %5d MB %6s G %4s%% %4s %4s%%  %s\n",
        $1,$2,$3,$4,($5==""?"—":$5),$6/1024,$7,$8,$9,$10,
        ($11==0?"ok":($11==1?"AVISO":"CRÍTICO"))}'
    n_ok=$(awk -F, -v n="$COLUNAS" '!/^#/ && NF==n' "$LOG" | wc -l)
    n_tot=$(awk '!/^#/' "$LOG" | wc -l)
    echo; echo "  amostras legíveis: $n_ok de $n_tot   ·   ficheiro: $LOG"
    [ "$n_ok" -lt "$n_tot" ] && echo "  ⚠️ $(( n_tot - n_ok )) linha(s) de um formato antigo foram SALTADAS (não convertidas)."
    echo "  ⏳ o limite (fração <5% do livre, com ≥8 GB livres) é PROVISÓRIO: há dois"
    echo "     pontos medidos — 0,0% com 68 GB livres = máquina morta (30/08) — e a"
    echo "     curva entre eles é o que estas amostras estão a construir."
    exit 0 ;;

  --instalar)
    U="$HOME/.config/systemd/user"; mkdir -p "$U"
    cat > "$U/ph2d-sanidade.service" <<EOF
[Unit]
Description=PH2D — vigia de sanidade da máquina
[Service]
Type=oneshot
ExecStart=/usr/bin/env bash $ROOT/scripts/sanidade.sh --vigia
EOF
    cat > "$U/ph2d-sanidade.timer" <<'EOF'
[Unit]
Description=PH2D — vigia de sanidade a cada 15 min
[Timer]
OnBootSec=3min
OnUnitActiveSec=15min
AccuracySec=30s
Persistent=true
[Install]
WantedBy=timers.target
EOF
    systemctl --user daemon-reload
    systemctl --user enable --now ph2d-sanidade.timer
    echo "✓ vigia armado. Confere a cada 15 min e só fala se houver problema."
    systemctl --user list-timers ph2d-sanidade.timer --no-pager | head -3
    exit 0 ;;

  --desinstalar)
    systemctl --user disable --now ph2d-sanidade.timer 2>/dev/null
    rm -f "$HOME/.config/systemd/user/ph2d-sanidade."{service,timer}
    systemctl --user daemon-reload
    echo "✓ vigia desarmado."; exit 0 ;;

  --vigia)
    # ⚠️ Anti-spam: uma notificação a cada 15 min sobre o mesmo problema deixa de
    # ser lida em uma hora, e aí o vigia está pior que ausente. Fala quando o
    # estado PIORA, e depois no máximo de 2 em 2 h enquanto durar.
    FALAR=0
    [ "$NIVEL" -gt "$NIVEL_ANT" ] && FALAR=1
    [ "$NIVEL" -gt 0 ] && [ $(( AGORA_S - NOTIF_T )) -gt 7200 ] && FALAR=1
    [ "$FALAR" = 1 ] && NOTIF_T=$AGORA_S
    gravar_estado
    if [ "$FALAR" = 1 ]; then
      if [ "$NIVEL" -ge 2 ]; then T="⛔ A máquina está em risco de travar"; U=critical
      else                        T="⚠️ A máquina precisa de atenção";      U=normal; fi
      CORPO=""
      for i in "${!ACOES[@]}"; do
        P=("${CRITICOS[@]}" "${AVISOS[@]}")
        CORPO+="• ${P[$i]:-}"$'\n'"   → ${ACOES[$i]}"$'\n'
      done
      notify-send -u "$U" -a "PH2D" -t 0 "$T" "$CORPO" 2>/dev/null || true
    fi
    exit "$NIVEL" ;;
esac

# ------------------------------------------------------- modo humano (--agora)
# ⚠️ um só sítio decide o ícone; escrever a regra ao lado de cada linha foi como
# a 2ª régua errada sobreviveu a uma revisão.
ic() { # ic <valor> <limite_aviso> <limite_critico>  (menor = pior)
  awk -v v="$1" -v a="$2" -v c="$3" 'BEGIN{ print (v<c) ? "⛔" : ((v<a) ? "⚠️" : "✓") }'
}
echo
echo "  ===================== SANIDADE DA MÁQUINA ====================="
if awk "BEGIN{exit !($LIVRE_GB >= 8)}"; then
  printf '  %s  %-34s %s\n' "$(ic "$FRACAO" 5 1)" "memória livre em pedaço grande" \
    "${FRACAO}%   de ${LIVRE_GB} GB livres   (0% = trava)"
else
  printf '  %s  %-34s %s\n' "✓" "memória livre em pedaço grande" \
    "n/a — só ${LIVRE_GB} GB livres (o resto está em cache, a trabalhar)"
fi
printf '  %s  %-34s %s\n' "$([ "${UNEV_KB:-0}" -gt 1048576 ] && echo ⛔ || echo ✓)" \
  "memória fincada" "$(( UNEV_KB/1024 )) MB   (tem de ficar em ~3 MB)"
printf '  %s  %-34s %s\n' \
  "$([ -z "$KC_TAXA" ] && echo "·" || ic "$(awk "BEGIN{print 1-$KC_TAXA}")" 0.50 0.10)" \
  "arrumador de memória" \
  "$([ -z "$KC_TAXA" ] && echo 'primeira leitura — a taxa sai na próxima' || echo "${KC_TAXA} de 1,00 núcleo   (1,00 = o padrão do travamento)")"
printf '  %s  %-34s %s\n' "$(ic "$AVAIL_GB" 10 4)" "memória disponível" "${AVAIL_GB} GB"
printf '  %s  %-34s %s\n' "$([ "${SWAP_DISCO:-0}" -eq 0 ] && echo ⚠️ || echo ✓)" \
  "swap em disco" "$([ "${SWAP_DISCO:-0}" -eq 0 ] && echo 'NÃO — só zram (armadilha de 22/08)' || echo 'sim')"
printf '  %s  %-34s %s\n' "$([ "${ZRAM_PCT:-0}" -ge 80 ] && echo ⚠️ || echo ✓)" "zram" "${ZRAM_PCT}%"
printf '  %s  %-34s %s\n' "$([ "${DISCO_PCT:-0}" -ge 90 ] && echo ⚠️ || echo ✓)" "disco mais cheio" "${DISCO_PCT}%"
printf '  %s  %-34s %s\n' "$([ -n "$NOSWAP_TMPFS" ] && echo ⛔ || echo ✓)" \
  "disco em RAM preso ('noswap')" "$([ -n "$NOSWAP_TMPFS" ] && echo "$NOSWAP_TMPFS" || echo 'não')"
echo "  ==============================================================="
if [ "$NIVEL" = 0 ]; then
  echo "  ✓ Está tudo bem. O vigia avisa sozinho se mudar."
else
  echo
  P=("${CRITICOS[@]}" "${AVISOS[@]}")
  for i in "${!ACOES[@]}"; do echo "  • ${P[$i]:-}"; echo "     → ${ACOES[$i]}"; done
fi
echo
gravar_estado
exit "$NIVEL"

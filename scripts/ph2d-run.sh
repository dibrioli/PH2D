#!/usr/bin/env bash
# ph2d-run — a PORTA por onde passa todo comando pesado de uma linha.
#
# Ela põe o comando numa FATIA (cgroup) que é da LINHA, não do comando, com
# quatro tectos: CPU, memória, prazo de parede e — quando pedido — a exclusão da
# placa. Runbook com todas as medições: `docs/DevOps/TETOS_DE_RECURSO_POR_LINHA.md`.
#
# ## POR QUE A FATIA É DA LINHA E NÃO DO COMANDO (medido 2026-09-15)
#
# Um tecto por COMANDO não compõe — duas corridas somam dois tectos:
#
#   fatia SEM tecto,        2 comandos ....... 22,32 núcleos de 32   (controlo)
#   tecto no COMANDO,       2 comandos ....... 30,05                 ⛔ soma
#   tecto na FATIA da linha, 2 comandos ...... 16,03
#   tecto na FATIA da linha, 4 comandos ...... 16,01                 ⭐ compõe
#
# ⚠️ *«nunca mais de 50 %» é uma afirmação sobre a LINHA, e só um cgroup que a
# linha inteira partilha a pode fazer.* Um `nice`, um `-j`, um `--test-threads`
# são todos por-processo, e por isso nenhum deles exprime esta frase.
#
# ## POR QUE O PRAZO É DO SCOPE E NÃO UM `timeout` (medido 2026-09-15)
#
# Um binário de teste reparenta-se ao `systemd --user` e sobrevive a quem o
# lançou (CLAUDE.md §5 / cargo-test-narrow.sh). Com um neto `setsid` a fingir
# esse órfão, e uma régua com controlo (ela VÊ 1 neto vivo antes do ensaio):
#
#   scope com RuntimeMaxSec=4s ....... 0 netos vivos depois   ⭐ o cgroup alcança
#   `timeout --kill-after` sozinho ... 1 neto vivo depois     ⛔ não alcança
#
# ## POR QUE A PLACA NÃO TEM PERCENTAGEM (medido 2026-09-15)
#
# ⛔ **50 % da GPU não é exprimível nesta máquina, e isso é uma propriedade do
# hardware, não uma folga minha.** As quatro portas foram perguntadas:
#   · MIG (partição de placa) .............. `[N/A]` — é uma placa de consumidor
#   · controlador cgroup `dmem` ............ existe no kernel e NÃO está delegado
#                                            ao utilizador; e o driver proprietário
#                                            da NVIDIA não implementa contabilidade
#                                            DRM por cgroup de qualquer maneira
#   · compute mode EXCLUSIVE_PROCESS ....... só cobre contextos CUDA; este repo
#                                            desenha por Vulkan/wgpu
#   · algum ficheiro de quota em sysfs ..... nenhum
# ⇒ a placa é um recurso de EXCLUSÃO, não de fatia: a regra honesta é *uma linha
# de cada vez, e com prazo*. E isso é mais forte do que 50 % para o defeito que
# de facto aconteceu — uma sonda pendurada segurou a placa **56 minutos**, e uma
# quota de 50 % não teria ajudado em nada, porque o problema não era partilhar,
# era SEGURAR. Provado: com um detentor vivo, um segundo é recusado; um detentor
# pendurado é morto pelo prazo (exit 124) e a fechadura liberta-se sozinha.
#
# ## USO
#   bash scripts/ph2d-run.sh cargo test -p ph2d-timeline
#   PH2D_GPU=1 bash scripts/ph2d-run.sh bash ronda.sh   # a vez da placa JÁ à partida (medir)
#   PH2D_PRAZO=5400 bash scripts/ph2d-run.sh cargo nextest run --workspace
#
# ## SUBIR UM TECTO (§0.0: quem sobe, MEDE e escreve o número)
#   PH2D_CPU_PCT=75   — percentagem dos núcleos desta linha (default 50)
#   PH2D_MEM_MAX=48G  — memória da linha inteira (default 24G)
#   PH2D_PRAZO=5400   — prazo de parede em segundos (default 1800); `0` desliga
#   PH2D_GPU_ESPERA=600 — quanto esperar pela placa antes de desistir (300 com PH2D_GPU=1; sem
#                         ele, a vez pedida a meio espera até o prazo)
set -uo pipefail

if [ $# -eq 0 ]; then
  sed -n '2,4p;49,59p' "$0" | sed 's/^# \?//'
  exit 64
fi

# ── quem é esta linha ─────────────────────────────────────────────────────────
# O nome sai da árvore de trabalho, que é o registo de posse do Modo L
# (CLAUDE.md §5: «Modo L: `git worktree list`»). Sanitizado porque um `-` num
# nome de fatia do systemd cria um nível de aninhamento.
raiz="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
linha="$(basename "$raiz" | tr '[:upper:]' '[:lower:]' | tr -c 'a-z0-9' '_' | sed 's/_*$//')"
fatia="ph2d-${linha}.slice"

nucleos="$(nproc)"
pct="${PH2D_CPU_PCT:-50}"
quota="$(( nucleos * pct ))%"
mem="${PH2D_MEM_MAX:-24G}"
# ⛔ No PRIMÁRIO o `target/debug` é um `tmpfs` cujas páginas ficam cobradas a ESTA fatia
# (TETOS_DE_RECURSO_POR_LINHA §1.1): um tecto de 24G que já começa com ~36G gastos mata o teste calado.
# A cura lá era a regra manual «leve PH2D_MEM_MAX=96G» — e ninguém a seguia (17/10: o portão do
# CLAUDE.md morreu 2× assim no primário). Sem `PH2D_MEM_MAX` explícito, o default soma o que o disco
# em RAM já ocupa: a linha recebe os mesmos 24G de trabalho que qualquer outra.
if [ -z "${PH2D_MEM_MAX:-}" ]; then
  _ram="$(readlink -f "$raiz/target/debug" 2>/dev/null)"
  if [ -n "$_ram" ] && [ "$(stat -f -c %T "$_ram" 2>/dev/null)" = "tmpfs" ]; then
    _usado_g="$(df --output=used -BG "$_ram" 2>/dev/null | tail -1 | tr -dc '0-9')"
    [ -n "$_usado_g" ] && mem="$(( 24 + _usado_g ))G"
  fi
fi
prazo="${PH2D_PRAZO:-1800}"

# ── a fatia da linha, idempotente ─────────────────────────────────────────────
unidade="${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user/${fatia}"
corpo="[Unit]
Description=PH2D — tectos de recurso da linha ${linha}
Documentation=file://${raiz}/docs/DevOps/TETOS_DE_RECURSO_POR_LINHA.md

[Slice]
# ${pct} % dos ${nucleos} núcleos, para a linha INTEIRA somada.
CPUQuota=${quota}
# Peso baixo = esta linha CEDE quando o dono está a fazer um smoke. Medido: com
# uma linha a 32 queimadores, o smoke passou de 6,8 s para 7,0 s (+3 %) e a linha
# ainda usou 17,7 núcleos do que sobrava. ⚠️ Um PESO não é um tecto: ele não
# desperdiça máquina ociosa, e é por isso que os dois coexistem aqui.
CPUWeight=20
MemoryMax=${mem}
# Metade da protecção, não um detalhe: sem isto o excesso vai para o zram e o que
# morre é a capacidade de reclaim da máquina inteira (ver o histórico de 14/08).
MemorySwapMax=0"

if command -v systemd-run >/dev/null 2>&1 && [ -d "${XDG_RUNTIME_DIR:-/run/user/$(id -u)}/systemd" ]; then
  if [ ! -f "$unidade" ] || [ "$(cat "$unidade")" != "$corpo" ]; then
    mkdir -p "$(dirname "$unidade")"
    printf '%s\n' "$corpo" > "$unidade"
    systemctl --user daemon-reload 2>/dev/null || true
  fi
  temos_systemd=1
else
  # CI, contentor, macOS: a porta continua a funcionar, mas diz o que perdeu.
  echo "⚠️  ph2d-run: sem systemd de utilizador — SEM tecto de CPU/memória (só o prazo)." >&2
  temos_systemd=0
fi

# ── a placa: a VEZ (docs/DevOps/A_VEZ_DA_PLACA.md) ────────────────────────────
# ⭐ Toda árvore que passa por esta porta tem um PORTEIRO, e ninguém precisa de se lembrar de nada:
# ele pede a placa ao cadeado partilhado (`ph2d-gpu.lock`, exclusivo entre árvores) na PRIMEIRA vez
# que alguém da árvore a pede, e segura-a até o comando acabar. Pedem-na: o `GpuContext` do
# `ph2d-gpu` (antes do adaptador — é ele que põe o processo na placa), o `vez-da-placa.sh` (o que não
# é Rust: o `kwin --virtual` das fotos e das medições) e o `PH2D_GPU=1` (já à partida, para medir).
# ⛔ Exclusiva de propósito: duas linhas na placa ao mesmo tempo penduraram o driver em 14/09.
# Medido 18/10 (20/20, também a load 58): o `flock` do Linux dá a vez por ORDEM DE CHEGADA, e quem já
# espera ganha de quem a larga e volta a pedir no mesmo instante — uma rodada de medição e um fluxo
# de testes alternam, nenhum mata o outro de fome.
#   · uma árvore DENTRO de outra (um portão que re-entra pela porta) usa o porteiro de fora: pedir
#     a vez de novo seria esperar pelo próprio pai;
#   · fora da porta não há porteiro e nada espera: o produto, o app aberto à mão pelo dono, a CI.
# ⛔⛔ **NENHUM FILHO SEGURA A PLACA** (29/09 e 01/10): um `flock` é do DESCRITOR, e um daemon que o
# herde (o servidor do `sccache`) segura a placa depois de o comando acabar — 21 min presas em 29/09.
# ⇒ quem tem o descritor é SÓ o porteiro, e ele não lança nada que fique vivo.
fechadura="${PH2D_GPU_FECHADURA:-${XDG_RUNTIME_DIR:-/tmp}/ph2d-gpu.lock}"
vez_viva() { # a árvore de fora tem porteiro vivo?
  [ -n "${PH2D_GPU_VEZ:-}" ] && [ -p "$PH2D_GPU_VEZ/porta" ] \
    && kill -0 "$(cat "$PH2D_GPU_VEZ/porteiro" 2>/dev/null || echo x)" 2>/dev/null
}
porteiro() { # $1 = o pid desta porta (o dono da árvore) · $2 = o tempo de desistir (vazio = nunca)
  local dono="$1" espera="$2" pedido="" fpid="" tenho=""
  trap 'exit 0' TERM INT HUP
  trap '[ -n "$fpid" ] && kill "$fpid" 2>/dev/null; [ -n "$tenho" ] && rm -f "${fechadura}.dono"' EXIT
  # 1. o 1.º pedido, sem gastar nada (um `read` no FIFO), de olho no dono (um `kill -9` não corre traps)
  until read -r -t 5 -u 3 pedido; do kill -0 "$dono" 2>/dev/null || exit 0; done
  local t0=$SECONDS
  exec 9>"$fechadura"
  if ! flock -n 9; then
    { echo "⏳ a placa está com outra árvore — à espera da vez (por ordem de chegada):"
      sed 's/^/    /' "${fechadura}.dono" 2>/dev/null; } >&2
    # Em fundo + `wait`: o `wait` interrompe-se pelo TERM do dono; um `flock` em primeiro plano não.
    flock ${espera:+-w "$espera"} 9 &
    fpid=$!
    if ! wait "$fpid"; then
      fpid=""
      { echo "a placa está com outra árvore há mais de ${espera}s — quem a segura:"
        sed 's/^/    /' "${fechadura}.dono" 2>/dev/null
        # Sem `.dono` a fechadura está com quem a HERDOU e não a pediu (um daemon de uma worktree velha).
        [ -s "${fechadura}.dono" ] || fuser -v "$fechadura" 2>&1 | sed 's/^/    /'
      } > "$vez/recusa.tmp"
      mv "$vez/recusa.tmp" "$vez/recusa"
      exit 0
    fi
    fpid=""
    echo "▸ a vez da placa chegou ao fim de $((SECONDS - t0)) s (pedida por ${pedido})" >&2
  fi
  tenho=1
  printf 'linha %s · pid %s · desde %s · %s · pedida por %s\n' \
    "$linha" "$dono" "$(date +%T)" "$cmd_txt" "$pedido" > "${fechadura}.dono"
  : > "$vez/pronto"
  # 2. segura até o dono acabar (o `trap` dele mata-me; se ele morrer sem `trap`, saio em ≤ 5 s)
  while kill -0 "$dono" 2>/dev/null; do read -r -t 5 -u 3 _; done
  exit 0
}
cmd_txt="$*"
porteiro_pid=""
if ! vez_viva && command -v flock >/dev/null 2>&1; then
  espera_vez="${PH2D_GPU_ESPERA:-}"
  [ "${PH2D_GPU:-0}" = "1" ] && espera_vez="${PH2D_GPU_ESPERA:-300}"
  vez="$(mktemp -d "${XDG_RUNTIME_DIR:-/tmp}/ph2d-vez.XXXXXX")"
  mkfifo "$vez/porta"
  # A porta abre-se ANTES do porteiro nascer (O_RDWR num FIFO não bloqueia): um pedido que chegue
  # antes de ele ler nunca se perde. O comando não a herda (`3>&-` logo a seguir).
  exec 3<>"$vez/porta"
  porteiro "$$" "$espera_vez" >/dev/null &
  porteiro_pid=$!
  exec 3>&-
  echo "$porteiro_pid" > "$vez/porteiro"
  export PH2D_GPU_VEZ="$vez"
  trap 'kill "$porteiro_pid" 2>/dev/null; wait "$porteiro_pid" 2>/dev/null; rm -rf "$vez"' EXIT
fi

usa_fd=""
if [ "${PH2D_GPU:-0}" = "1" ]; then
  # O servidor do `sccache` nasce FORA da fatia: nascido dentro do `systemd-run --scope` ele morreria
  # com ela (o prazo, o fim do comando), levando o servidor partilhado por todas as linhas.
  if command -v sccache >/dev/null 2>&1; then
    sccache --start-server >/dev/null 2>&1 || true
  fi
  bash "$(dirname "$0")/vez-da-placa.sh" || exit $?   # 75 = EX_TEMPFAIL, tente outra vez
  [ -n "$porteiro_pid" ] || vez_viva || echo "⚠️  ph2d-run: sem \`flock\` — SEM exclusão da placa." >&2
  usa_fd=1
fi

# ── correr ────────────────────────────────────────────────────────────────────
printf '▸ linha %s · CPU ≤ %s de %s núcleos · mem ≤ %s · prazo %ss%s\n' \
  "$linha" "$quota" "$nucleos" "$mem" "$prazo" \
  "$([ -n "$usa_fd" ] && echo ' · COM A PLACA')" >&2

# ⚠️ Marca a travessia: os portões do repo (`ship.sh`, `nextest-impacted.sh`,
# `cargo-test-narrow.sh`) re-invocam-se por aqui, e sem esta marca fá-lo-iam para
# sempre. Ela é EXPORTADA, logo vale para tudo o que nascer dentro do comando.
export PH2D_NA_PORTA=1

# ⛔⛔ `9>&-` nos dois ramos: o comando NÃO herda o cadeado da placa (o bloco da placa, acima: quem o
# segura é SÓ o porteiro). Este bash já não o abre; a cerca fica para um `9>` que alguém lhe junte.
if [ "$temos_systemd" = "1" ]; then
  props=( -p MemoryMax="$mem" -p MemorySwapMax=0 )
  [ "$prazo" != "0" ] && props+=( -p RuntimeMaxSec="${prazo}s" )
  systemd-run --user --scope --quiet --collect \
    --slice="$fatia" \
    --description="ph2d-run[$linha]: $*" \
    "${props[@]}" -- "$@" 9>&-
else
  if [ "$prazo" != "0" ]; then timeout --kill-after=30s "${prazo}s" "$@" 9>&-; else "$@" 9>&-; fi
fi
saida=$?

if [ "$saida" = "143" ] || [ "$saida" = "124" ]; then
  echo "✗ PENDUROU — bateu no prazo de ${prazo}s e foi morto com a árvore inteira." >&2
  echo "  ⚠️ Antes de subir o prazo, MEÇA se ele está a trabalhar ou bloqueado: leia" >&2
  echo "     utime+stime de /proc/<pid>/stat DUAS vezes. O \`ps\` mostra a média da VIDA" >&2
  echo "     do processo, e um processo bloqueado a segurar a placa lê-se ali como 95 %." >&2
fi
exit $saida

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
#   PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-field-gpu -- --ignored
#   PH2D_PRAZO=5400 bash scripts/ph2d-run.sh cargo nextest run --workspace
#
# ## SUBIR UM TECTO (§0.0: quem sobe, MEDE e escreve o número)
#   PH2D_CPU_PCT=75   — percentagem dos núcleos desta linha (default 50)
#   PH2D_MEM_MAX=48G  — memória da linha inteira (default 24G)
#   PH2D_PRAZO=5400   — prazo de parede em segundos (default 1800); `0` desliga
#   PH2D_GPU_ESPERA=600 — quanto esperar pela placa antes de desistir (default 300)
set -uo pipefail

if [ $# -eq 0 ]; then
  sed -n '2,4p;44,52p' "$0" | sed 's/^# \?//'
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

# ── a placa, quando o comando lhe toca ────────────────────────────────────────
fechadura="${XDG_RUNTIME_DIR:-/tmp}/ph2d-gpu.lock"
usa_fd=""
if [ "${PH2D_GPU:-0}" = "1" ]; then
  espera="${PH2D_GPU_ESPERA:-300}"
  exec 9>"$fechadura"
  if ! flock -w "$espera" 9; then
    echo "✗ a placa está com outra linha há mais de ${espera}s — quem a segura:" >&2
    cat "${fechadura}.dono" 2>/dev/null | sed 's/^/    /' >&2
    echo "  (⛔ NÃO force: duas linhas na placa ao mesmo tempo foi o que pendurou o driver em 14/09)" >&2
    exit 75   # EX_TEMPFAIL — tente outra vez, não é erro do seu código
  fi
  printf 'linha %s · pid %s · desde %s · %s\n' "$linha" "$$" "$(date +%T)" "$*" > "${fechadura}.dono"
  trap 'rm -f "${fechadura}.dono"' EXIT
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

if [ "$temos_systemd" = "1" ]; then
  props=( -p MemoryMax="$mem" -p MemorySwapMax=0 )
  [ "$prazo" != "0" ] && props+=( -p RuntimeMaxSec="${prazo}s" )
  systemd-run --user --scope --quiet --collect \
    --slice="$fatia" \
    --description="ph2d-run[$linha]: $*" \
    "${props[@]}" -- "$@"
else
  if [ "$prazo" != "0" ]; then timeout --kill-after=30s "${prazo}s" "$@"; else "$@"; fi
fi
saida=$?

if [ "$saida" = "143" ] || [ "$saida" = "124" ]; then
  echo "✗ PENDUROU — bateu no prazo de ${prazo}s e foi morto com a árvore inteira." >&2
  echo "  ⚠️ Antes de subir o prazo, MEÇA se ele está a trabalhar ou bloqueado: leia" >&2
  echo "     utime+stime de /proc/<pid>/stat DUAS vezes. O \`ps\` mostra a média da VIDA" >&2
  echo "     do processo, e um processo bloqueado a segurar a placa lê-se ali como 95 %." >&2
fi
exit $saida

#!/usr/bin/env bash
# vez-da-placa — pede a VEZ da placa ao porteiro desta árvore e espera por ela.
#
# Para o que toca na placa sem passar pelo `GpuContext` do `ph2d-gpu` (que pede a vez sozinho): o
# `kwin_wayland --virtual` das fotos e das medições, um oráculo. Chame-o ANTES de lançar esse programa.
# O porteiro nasce no `scripts/ph2d-run.sh`; o protocolo e as medições: docs/DevOps/A_VEZ_DA_PLACA.md.
#
#   bash scripts/vez-da-placa.sh           — a placa principal (o kwin, as medições, os oráculos)
#   bash scripts/vez-da-placa.sh testes    — a fila de testes, se a árvore a abriu (senão a principal)
#
#   exit 0  — a vez é de quem chamou, o `$PPID`, até ele acabar (ou não há porteiro: fora da porta
#             ninguém espera)
#   exit 75 — o porteiro desistiu (`PH2D_GPU_ESPERA`); EX_TEMPFAIL, tente outra vez
#
# ⚠️ O protocolo tem um segundo cliente, em Rust (`crates/ph2d-gpu/src/vez.rs`): `porta` (FIFO onde se
# escreve «pid nome»), `vez.<pid>` (a vez chegou a esse pid), `recusa` (o porteiro desistiu),
# `porteiro` (o pid dele) — e, na fila de testes, os mesmos com o sufixo `-testes`.
set -u
v="${PH2D_GPU_VEZ:-}"
[ -n "$v" ] || exit 0
suf=""
[ "${1:-}" = testes ] && [ -p "$v/porta-testes" ] && suf="-testes"
vivo() { kill -0 "$(cat "$v/porteiro$suf" 2>/dev/null || echo x)" 2>/dev/null; }
if [ ! -p "$v/porta$suf" ] || ! vivo; then
  echo "⚠️  vez-da-placa: o porteiro de $v já não existe — sigo SEM a vez" >&2
  exit 0
fi
# O_RDWR: num FIFO o `open` não bloqueia — um porteiro que morra agora não pendura este pedido.
exec 4<>"$v/porta$suf"
printf '%s %s\n' "$PPID" "$(cat "/proc/$PPID/comm" 2>/dev/null || echo '?')" >&4
exec 4>&-
while [ ! -e "$v/vez$suf.$PPID" ]; do
  if [ -e "$v/recusa$suf" ]; then
    { echo "✗ $(cat "$v/recusa$suf")"
      echo "  (⛔ NÃO force: duas linhas na placa ao mesmo tempo foi o que pendurou o driver em 14/09)"; } >&2
    exit 75
  fi
  if ! vivo; then
    echo "⚠️  vez-da-placa: o porteiro morreu à espera — sigo SEM a vez" >&2
    exit 0
  fi
  sleep 0.1
done
exit 0

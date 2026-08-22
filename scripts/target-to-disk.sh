#!/usr/bin/env bash
# target-to-disk.sh — tira o target/ do PRIMÁRIO do tmpfs (/dev/shm) e põe-no no
# disco com nodatacow (+C), PRESERVANDO o build quente. É o inverso do
# `target-on-tmpfs.sh`, e o substitui no tier workstation.
#
# WHY (medido 2026-08-22): o target em tmpfs tinha **33 GB**, dos quais 3 GB
# residentes e ~30 GB SWAPADOS para o zram — que é RAM (12,1 GiB comprimidos).
# Swap 32/32 GiB cheio com 61 GiB de RAM livre; no próximo pico o earlyoom não
# fecha o AND e o OOM do kernel derruba a janela inteira (OOMPolicy=stop,
# project-memory 14/08). Em disco, com +C, o artefato descartável sai do caminho
# CoW+zstd, o page cache serve o que está quente, e um reboot NÃO evapora o
# build. O «grande ganho de link/IO» do tmpfs nunca teve tabela ao lado
# (CLAUDE.md §0.0); o preço acima tem.
#
# PORTÕES (refuse-by-default, como a DIRETIVA_FIM_DE_DIA — TODOS passam, ou nada):
#   1. target/ do primário é um symlink para /dev/shm/… (senão não há o que migrar)
#   2. ninguém CONSTRÓI no primário (cargo/rustc/mold/… com cwd na raiz, fora de
#      Worktrees/) nem EXECUTA de dentro do target (/proc/*/exe dentro do tmpfs)
#   3. o btrfs tem de onde receber ~N GB de arquivos novos: `btrfs-health.sh`
#      sem vermelho em não-alocado/metadata — senão a cópia morre em ENOSPC a
#      meio e fica um target pela metade
#
# USAGE
#   bash scripts/target-to-disk.sh             # migra, preservando o build (cp)
#   bash scripts/target-to-disk.sh --cold      # não copia: target novo vazio (+C); apaga o tmpfs
#   bash scripts/target-to-disk.sh --dry-run   # só os portões e o plano, nada muda
set -euo pipefail

opt="${1:-}"
root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
link="$root/target"
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# A MESMA lista da DIRETIVA_FIM_DE_DIA §4. O rust-analyzer fica de FORA de
# propósito: ele vive com cwd no primário enquanto a janela existir (nunca
# «termina»), e o que ele constrói é um `cargo` filho — que ESTA lista pega.
BUILDERS=(cargo rustc mold cc1 ld rustdoc)

fail() { echo "✗ $1"; echo "  ABORTADO — nada mudou."; exit 1; }

# ---- portão 1: é um symlink para tmpfs? -------------------------------------
[ -L "$link" ] || fail "$link não é symlink — nada a migrar (já está em disco?)."
shm="$(readlink -f "$link")"
case "$shm" in /dev/shm/*|/tmp/*) ;; *) fail "$link aponta para $shm, que não é tmpfs." ;; esac
if [ ! -d "$shm" ]; then
  # Depois de um reboot o tmpfs evaporou: sem a regra tmpfiles.d o dir nem existe.
  # --cold não precisa dele (não há o que copiar); sem --cold, não há o que preservar.
  [ "$opt" = "--cold" ] || fail "$shm não existe (tmpfs evaporou no reboot?). Não há build a preservar: use --cold."
fi

# ---- portão 2: ninguém constrói no primário nem executa de dentro do target --
_self=$(basename "$(readlink -f /proc/$$/exe 2>/dev/null)" 2>/dev/null)
{ [ -n "$_self" ] && pgrep -x "$_self" >/dev/null; } || fail "controle positivo: pgrep não vê o próprio shell — o portão 2 seria cego."
for p in "${BUILDERS[@]}"; do
  for pid in $(pgrep -x "$p" 2>/dev/null || true); do
    cwd="$(readlink -f "/proc/$pid/cwd" 2>/dev/null || true)"
    case "$cwd" in
      "$root"/Worktrees/*) ;;                       # outra linha, outro target — não é nosso
      "$root"|"$root"/*) fail "'$p' (pid $pid) constrói no primário (cwd=$cwd). Espere-o terminar." ;;
    esac
  done
done
for pid in $(ls /proc 2>/dev/null | grep -E '^[0-9]+$'); do
  exe="$(readlink -f "/proc/$pid/exe" 2>/dev/null)" || continue
  case "$exe" in "$shm"/*) fail "pid $pid executa de dentro do target ($exe) — um smoke? Feche-o primeiro." ;; esac
done

# ---- portão 3: o disco aguenta a cópia? ------------------------------------
size=0; nfiles=0
if [ -d "$shm" ]; then
  size=$(du -sb "$shm" 2>/dev/null | cut -f1 || echo 0)
  nfiles=$(find "$shm" -type f 2>/dev/null | wc -l)
fi
eval "$(bash "$here/btrfs-health.sh" --env 2>/dev/null || true)"
if [ "${PH2D_FSTYPE:-}" = "btrfs" ] && [ "$opt" != "--cold" ]; then
  need_gib=$(awk -v b="$size" 'BEGIN{printf "%d", b/1073741824 + 8}')
  un="${PH2D_BTRFS_UNALLOC_GIB:-0}"; mf="${PH2D_BTRFS_META_FREE_GIB:-0}"
  awk -v u="$un" -v n="$need_gib" 'BEGIN{exit !(u >= n)}' \
    || fail "não-alocado ${un} GiB < ${need_gib} GiB necessários para copiar $(awk -v b="$size" 'BEGIN{printf "%.1f", b/1073741824}') GiB com folga. Rode ANTES: sudo btrfs balance start -dusage=10 / (docs/DevOps/BTRFS_METADATA_E_SWAP.md §1)"
  awk -v m="$mf" 'BEGIN{exit !(m >= 1.0)}' \
    || fail "metadata livre ${mf} GiB < 1 GiB — $nfiles arquivos novos não cabem. Balance primeiro."
fi

swap_before=$(awk '/^SwapFree:/{print $2}' /proc/meminfo)
echo "plano: $link -> $shm ($(awk -v b="$size" 'BEGIN{printf "%.1f", b/1073741824}') GiB, $nfiles arquivos) → $link (dir real, +C)"
[ "$opt" = "--cold" ] && echo "       --cold: sem cópia; o tmpfs é apagado e o próximo build é frio (sccache serve os deps)."
[ "$opt" = "--dry-run" ] && { echo "portões: todos passaram. (--dry-run: nada mudou)"; exit 0; }

# ---- migra -------------------------------------------------------------------
disk="$root/target.disk.$$"
mkdir -p "$disk"
chattr +C "$disk" 2>/dev/null || echo "! chattr +C falhou (FS sem nodatacow?) — segue sem"
if [ "$opt" != "--cold" ]; then
  echo "copiando (tmpfs → disco, sem reflink)…"
  cp -a --reflink=never "$shm/." "$disk/"
  got=$(find "$disk" -type f | wc -l)
  [ "$got" -eq "$nfiles" ] || { rm -rf "$disk"; fail "cópia incompleta: $got de $nfiles arquivos. Disco cheio a meio? Nada mudou (cópia removida)."; }
fi
rm "$link"                                   # só o symlink — NUNCA rm -rf num symlink (DIRETIVA §1.3)
mv "$disk" "$link"
[ -d "$shm" ] && rm -rf "$shm"               # a cópia foi verificada; isto devolve o swap
swap_after=$(awk '/^SwapFree:/{print $2}' /proc/meminfo)

echo "✓ $link é dir real no disco, nodatacow: $(lsattr -d "$link" | cut -d' ' -f1)"
echo "  swap livre: $(( swap_before / 1024 )) MiB → $(( swap_after / 1024 )) MiB"
echo "  (a regra /etc/tmpfiles.d/ph2d-ramtarget.conf, se existir, só recria um dir vazio no boot — inofensiva;"
echo "   para retirar: sudo rm /etc/tmpfiles.d/ph2d-ramtarget.conf)"

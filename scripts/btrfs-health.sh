#!/usr/bin/env bash
# btrfs-health.sh — o disco desta máquina está DOENTE? Mede SEM root, em 1 s.
#
# WHY this exists (medido 2026-08-22): "disco cheio" com **526 GB livres**. O btrfs
# tinha distribuído 937,85 GiB dos 950 em blocos de DADOS (410 usados) e sobrava
# ZERO byte sem dono; a metadata (6 GiB alocados, 5,46 usados) não tinha de onde
# crescer → ENOSPC a meio de um build → artefato truncado → `mold` em SIGBUS, em
# laço, ~10 min por tentativa. `df -h` dizia 45% e NÃO vê nada disso: ele soma o
# que está livre DENTRO dos blocos de dados, e a metadata não mora lá.
#
# As perguntas certas são «quanto espaço NÃO-ALOCADO sobra?» (é de onde a
# metadata cresce) e «quanta metadata livre?» — e ambas se respondem sem root por
# `btrfs filesystem df --raw` + sysfs. Junto vão as três vizinhas que a mesma
# jornada pagou: swap (zram) cheio pelo target em tmpfs, corrupção de checksum
# (contador do device + journal desta boot) e targets de worktree sem nodatacow.
#
# Cada medição sai com VEREDITO e com o comando que cura. A cura de metadata é
# `btrfs balance` e exige root — este script mede e entrega o comando; quem roda
# é o Enio. Runbook: docs/DevOps/BTRFS_METADATA_E_SWAP.md.
#
# USAGE
#   bash scripts/btrfs-health.sh          # banner humano · exit 0 verde, 1 vermelho
#   bash scripts/btrfs-health.sh --env    # KEY=VALUE para scripts (target-to-disk.sh usa)
#   bash scripts/btrfs-health.sh --scan <dir>...   # lista os arquivos CORROMPIDOS sob <dir>
#         (lê cada arquivo; checksum errado ⇒ EIO ⇒ o caminho sai no stdout). É a única
#         lista COMPLETA: o scrub imprime ~10 caminhos a cada 5 s e suprime o resto
#         (medido 22/08: 228 erros, 10 caminhos no journal). Arquivos +C (nodatacow)
#         não têm checksum e nunca aparecem aqui. Apague os de target/ e sccache
#         (regeneram); os de fonte restauram-se do git.
#
# Portável: num FS que não é btrfs imprime só swap/tmpfs e sai 0 (Mac/Windows não
# têm o problema; o script existe para o tier workstation).
set -euo pipefail

mode="${1:-}"
if [ "$mode" = "--scan" ]; then
  shift; [ "$#" -gt 0 ] || { echo "uso: btrfs-health.sh --scan <dir>..." >&2; exit 2; }
  find "$@" -type f -print0 2>/dev/null \
    | nice -n 19 ionice -c3 xargs -0 -n 64 -P 8 sh -c 'for f; do cat "$f" >/dev/null 2>&1 || echo "$f"; done' _
  exit 0
fi
root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
mnt="$(findmnt -no TARGET --target "$root" 2>/dev/null || echo /)"
fstype="$(findmnt -no FSTYPE --target "$root" 2>/dev/null || echo unknown)"
uuid="$(findmnt -no UUID --target "$root" 2>/dev/null || echo "")"

GIB=1073741824
gib() { awk -v b="${1:-0}" 'BEGIN{printf "%.2f", b/1073741824}'; }
red=0; yellow=0
lines=()
say() { lines+=("$1"); }

# ---- 1+2. não-alocado e folga de metadata (btrfs) --------------------------
unalloc=0; meta_free=0; meta_total=0; meta_used=0; data_total=0; data_used=0; dev_size=0; alloc_phys=0
corrupt=0; csum_boot=0
if [ "$fstype" = "btrfs" ] && command -v btrfs >/dev/null; then
  while IFS= read -r l; do
    # "Metadata, DUP: total=6442450944, used=5279055872"
    kind="${l%%,*}"; rest="${l#*, }"; prof="${rest%%:*}"
    total="$(printf '%s' "$l" | sed -n 's/.*total=\([0-9]*\).*/\1/p')"
    used="$(printf '%s' "$l" | sed -n 's/.*used=\([0-9]*\).*/\1/p')"
    [ -n "$total" ] || continue
    case "$prof" in
      DUP|RAID1|RAID10) n=2 ;; RAID1C3) n=3 ;; RAID1C4) n=4 ;; *) n=1 ;;
    esac
    case "$kind" in
      Data)     data_total=$total; data_used=$used; alloc_phys=$(( alloc_phys + total * n )) ;;
      Metadata) meta_total=$total; meta_used=$used; alloc_phys=$(( alloc_phys + total * n )) ;;
      System)   alloc_phys=$(( alloc_phys + total * n )) ;;
    esac
  done < <(btrfs filesystem df --raw "$mnt" 2>/dev/null || true)
  for d in /sys/fs/btrfs/"$uuid"/devices/*/size; do
    [ -r "$d" ] && dev_size=$(( dev_size + $(cat "$d") * 512 ))
  done
  unalloc=$(( dev_size - alloc_phys )); [ "$unalloc" -lt 0 ] && unalloc=0
  meta_free=$(( meta_total - meta_used ))

  # ---- 4. corrupção: contador persistente do device + journal desta boot ----
  corrupt=$(cat /sys/fs/btrfs/"$uuid"/devinfo/*/error_stats 2>/dev/null \
            | awk '/corruption_errs/{s+=$2} END{print s+0}')
  csum_boot=$(journalctl -k -b 0 --no-pager 2>/dev/null | grep -c 'csum failed' || true)
fi

# ---- 3. swap (zram) e target do primário em tmpfs --------------------------
swap_total=$(awk '/^SwapTotal:/{print $2*1024}' /proc/meminfo 2>/dev/null || echo 0)
swap_free=$(awk '/^SwapFree:/{print $2*1024}' /proc/meminfo 2>/dev/null || echo 0)
swap_used=$(( swap_total - swap_free ))
swap_pct=0; [ "$swap_total" -gt 0 ] && swap_pct=$(( swap_used * 100 / swap_total ))
zram_line="$(zramctl --noheadings --output NAME,DISKSIZE,DATA,COMPR 2>/dev/null | head -1 || true)"

tmpfs_target=0; tmpfs_path=""
if [ -L "$root/target" ]; then
  tmpfs_path="$(readlink -f "$root/target" 2>/dev/null || true)"
  case "$tmpfs_path" in /dev/shm/*|/tmp/*)
    tmpfs_target=$(du -sb "$tmpfs_path" 2>/dev/null | cut -f1 || echo 0) ;;
  esac
fi

# ---- 5. targets de worktree sem nodatacow ---------------------------------
nocow_missing=()
if command -v lsattr >/dev/null; then
  while read -r wt; do
    [ "$wt" = "$root" ] && continue
    t="$wt/target"
    [ -d "$t" ] && [ ! -L "$t" ] || continue
    attrs="$(lsattr -d "$t" 2>/dev/null | cut -d' ' -f1 || true)"
    case "$attrs" in *C*) ;; *) nocow_missing+=("$t") ;; esac
  done < <(git worktree list --porcelain 2>/dev/null | awk '/^worktree /{print $2}')
fi

# ---- vereditos ---------------------------------------------------------------
if [ "$fstype" = "btrfs" ]; then
  if   [ "$unalloc" -lt $(( 8 * GIB )) ];  then red=1;    say "✗ não-alocado $(gib "$unalloc") GiB — a metadata NÃO tem de onde crescer (dados: $(gib "$data_used") usados em $(gib "$data_total") GiB alocados)"
  elif [ "$unalloc" -lt $(( 32 * GIB )) ]; then yellow=1; say "! não-alocado $(gib "$unalloc") GiB — pouco; um dia de jornada come isto"
  else                                                    say "✓ não-alocado $(gib "$unalloc") GiB"; fi
  if   [ "$meta_free" -lt $(( 1 * GIB )) ]; then red=1;    say "✗ metadata livre $(gib "$meta_free") GiB de $(gib "$meta_total") — ENOSPC iminente (o df não mostra)"
  elif [ "$meta_free" -lt $(( 2 * GIB )) ]; then yellow=1; say "! metadata livre $(gib "$meta_free") GiB de $(gib "$meta_total") — apertado"
  else                                                     say "✓ metadata livre $(gib "$meta_free") GiB de $(gib "$meta_total")"; fi

  state_dir="${XDG_CACHE_HOME:-$HOME/.cache}/ph2d"; mkdir -p "$state_dir" 2>/dev/null || true
  last_corrupt=$(cat "$state_dir/btrfs-health.corrupt" 2>/dev/null || echo "$corrupt")
  printf '%s\n' "$corrupt" > "$state_dir/btrfs-health.corrupt" 2>/dev/null || true
  delta=$(( corrupt - last_corrupt ))
  if [ "$csum_boot" -gt 0 ] || [ "$delta" -gt 0 ]; then red=1
    say "✗ corrupção de checksum: $csum_boot linha(s) NESTA boot, contador do device $corrupt (+$delta desde a última medição) — cada uma é um artefato que volta errado do disco → mold SIGBUS / rustc SIGSEGV"
  else say "✓ checksum: 0 nesta boot (contador do device: $corrupt, estável)"; fi
fi

if   [ "$swap_pct" -ge 90 ]; then red=1;    say "✗ swap ${swap_pct}% ($(gib "$swap_used") de $(gib "$swap_total") GiB) — sem folga para o próximo pico; ${zram_line:-zram?}"
elif [ "$swap_pct" -ge 60 ]; then yellow=1; say "! swap ${swap_pct}% ($(gib "$swap_used") GiB)"
else                                        say "✓ swap ${swap_pct}%"; fi

if [ "$tmpfs_target" -gt 0 ]; then
  if [ "$swap_pct" -ge 60 ]; then red=1; else yellow=1; fi
  say "! target/ do primário em tmpfs: $(gib "$tmpfs_target") GiB em $tmpfs_path — é RAM, e o que não cabe vai para o zram (que também é RAM); o swap acima é em boa parte ISTO"
fi

if [ "${#nocow_missing[@]}" -gt 0 ]; then yellow=1
  say "! ${#nocow_missing[@]} target(s) de worktree SEM nodatacow (+C): ${nocow_missing[*]}"
fi

# ---- emit ------------------------------------------------------------------
if [ "$mode" = "--env" ]; then
  cat <<EOF
PH2D_FSTYPE=$fstype
PH2D_BTRFS_UNALLOC_GIB=$(gib "$unalloc")
PH2D_BTRFS_META_FREE_GIB=$(gib "$meta_free")
PH2D_BTRFS_DATA_SLACK_GIB=$(gib $(( data_total - data_used )))
PH2D_BTRFS_CORRUPT=$corrupt
PH2D_BTRFS_CSUM_THIS_BOOT=$csum_boot
PH2D_SWAP_USED_PCT=$swap_pct
PH2D_TMPFS_TARGET_GIB=$(gib "$tmpfs_target")
PH2D_NOCOW_MISSING=${#nocow_missing[@]}
PH2D_DISK_RED=$red
PH2D_DISK_YELLOW=$yellow
EOF
  exit "$red"
fi

bold=""; dim=""; rst=""
if [ -t 1 ]; then bold="\033[1m"; dim="\033[2m"; rst="\033[0m"; fi
printf "${bold}PH2D disk health${rst}  →  %s em %s (%s)  kernel %s\n" "$fstype" "$mnt" "$(df -h "$mnt" | awk 'NR==2{print $3" de "$2", "$5}')" "$(uname -r)"
echo   "─────────────────────────────────────────────────────────────"
for l in "${lines[@]}"; do echo "  $l"; done
echo   "─────────────────────────────────────────────────────────────"
if [ "$red" = 1 ]; then
  echo "  VERMELHO — a cura, na ordem (detalhe: docs/DevOps/BTRFS_METADATA_E_SWAP.md):"
  [ "$fstype" = "btrfs" ] && [ "$unalloc" -lt $(( 32 * GIB )) ] && cat <<'EOF'
    1. (root) devolver blocos meio-vazios ao «não-alocado» — progressivo, pode parar com Ctrl+C:
         sudo btrfs balance start -dusage=10 /   &&   sudo btrfs balance start -dusage=30 /
       confira:  sudo btrfs filesystem usage / | grep -E 'unallocated|Metadata'
       ⛔ SÓ num boot SEM «csum failed» (o balance REESCREVE dados; num kernel que corrompe
          escritas ele arrisca o que está bom — e aborta em EIO no 1º extent ruim). Se a linha
          «checksum» acima está vermelha: reboot no kernel LTS → limpar os targets corrompidos
          (fim-de-dia) → só então o balance. O timer ph2d-btrfs-balance já tem essa trava.
EOF
  [ "$tmpfs_target" -gt 0 ] && [ "$swap_pct" -ge 60 ] && cat <<'EOF'
    2. tirar o target/ do primário da RAM (preserva o build; só depois do passo 1):
         bash scripts/target-to-disk.sh
EOF
  { [ "$csum_boot" -gt 0 ] || [ "$delta" -gt 0 ]; } 2>/dev/null && cat <<'EOF'
    3. corrupção: A/B de kernel (LTS já instalado) + scrub — ver runbook §3.
         sudo btrfs scrub start -B /      # lê tudo e imprime o CAMINHO de cada arquivo corrompido
EOF
  [ "${#nocow_missing[@]}" -gt 0 ] && printf '    4. nodatacow nos targets listados:  find <target> -type d -exec chattr +C {} +\n'
  exit 1
elif [ "$yellow" = 1 ]; then
  echo "  AMARELO — nada quebrado; os «!» acima dizem o que vai apertar primeiro."
else
  echo "  VERDE."
fi
exit 0

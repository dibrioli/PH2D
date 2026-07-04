#!/usr/bin/env bash
# target-on-tmpfs.sh — put cargo's target/ on a RAM disk (tmpfs) for workstation
# tier machines with RAM to spare. Big win on link/IO; incremental state lives in
# RAM. The dep cache survives reboot via sccache (see ~/.cargo/config.toml), so a
# post-reboot cold build is served from the sccache disk cache.
#
# Only acts on Linux with /dev/shm (a tmpfs). No-ops on macOS/Windows and on
# constrained machines — safe to run anywhere. See scripts/hw-profile.sh.
#
#   bash scripts/target-on-tmpfs.sh          # arm target/ -> tmpfs (idempotent)
#   bash scripts/target-on-tmpfs.sh --off    # revert: put target/ back on disk
#
# REBOOT SAFETY: a symlink target -> /dev/shm/<dir> DANGLES after reboot (tmpfs is
# wiped), and cargo's create_dir_all does NOT heal a dangling symlink (mkdir -p
# returns EEXIST). So this prints the one-time systemd-tmpfiles.d rule that
# recreates the dir at boot. Install it once (needs sudo) and reboots are safe.
set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
shm_dir="/dev/shm/ph2d-target"
link="$root/target"

if [ "$(uname -s)" != "Linux" ] || [ ! -d /dev/shm ]; then
  echo "[tmpfs] not Linux+/dev/shm — no-op (target/ stays on disk)."; exit 0
fi

if [ "${1:-}" = "--off" ]; then
  if [ -L "$link" ]; then rm -f "$link"; mkdir -p "$link"
    echo "[tmpfs] reverted: $link is a real dir again (tmpfs copy left in $shm_dir)."
  else echo "[tmpfs] $link is not a symlink — nothing to revert."; fi
  exit 0
fi

mkdir -p "$shm_dir"
if [ -L "$link" ]; then
  :                                   # already a symlink — leave it
elif [ -d "$link" ]; then
  # migrate existing on-disk target into tmpfs to preserve the warm build
  echo "[tmpfs] migrating existing target/ into $shm_dir …"
  cp -a "$link/." "$shm_dir/" 2>/dev/null || true
  rm -rf "$link"; ln -s "$shm_dir" "$link"
else
  ln -s "$shm_dir" "$link"
fi

# keep git status clean (target-as-symlink escapes the `target/` dir rule)
excl="$root/.git/info/exclude"
grep -qx "/target" "$excl" 2>/dev/null || echo "/target" >> "$excl"

echo "[tmpfs] armed: $link -> $shm_dir"
echo "[tmpfs] ONE-TIME reboot-safety (run once, needs sudo):"
echo "    echo 'd $shm_dir 0755 $USER $USER -' | sudo tee /etc/tmpfiles.d/ph2d-ramtarget.conf"
echo "    sudo systemd-tmpfiles --create /etc/tmpfiles.d/ph2d-ramtarget.conf"

#!/usr/bin/env bash
# hw-profile.sh — detect this machine's hardware TIER and print the speed knobs.
#
# WHY this exists: the whole speed strategy (DIRETRIZ §6.6) was hardcoded to the
# 8 GiB Apple Silicon Mac (≤3 cargos, rust-analyzer BLOCKED, CoW slots mandatory).
# PH2D is now developed across machines of very different power (weak Mac / weak
# Windows / a 128 GB Linux workstation). A fixed strategy wastes the strong box
# and would crash the weak one. This script makes the strategy a FUNCTION of the
# hardware instead of a constant — no per-machine values are hardcoded; each
# machine self-classifies at runtime.
#
# TIERS
#   constrained  — ≤12 GiB RAM  (e.g. the 8 GiB Mac): the original §6.6 baseline.
#   standard     — mid box      (e.g. the Windows machine): measured middle ground.
#   workstation  — ≥48 GiB RAM AND ≥12 cores (the Linux desktop): unlocked.
#
# USAGE
#   bash scripts/hw-profile.sh          # human banner + knob table (default)
#   bash scripts/hw-profile.sh --env    # sourceable KEY=VALUE only (for scripts)
#   eval "$(bash scripts/hw-profile.sh --env)"   # load PH2D_* into the shell
#
# An agent should run this FIRST and read the rules for its tier. The rules in
# DIRETRIZ §6.6 are the `constrained` baseline; the printed OVERRIDES apply on
# top for the current tier.
set -euo pipefail

# ---- detect OS -------------------------------------------------------------
uname_s="$(uname -s 2>/dev/null || echo unknown)"
case "$uname_s" in
  Darwin)                 os="macos" ;;
  Linux)                  os="linux" ;;
  MINGW*|MSYS*|CYGWIN*)   os="windows" ;;
  *)                      os="unknown" ;;
esac

# ---- detect RAM (GiB, integer) --------------------------------------------
ram_gib=0
case "$os" in
  linux)
    if [ -r /proc/meminfo ]; then
      kb=$(awk '/^MemTotal:/{print $2}' /proc/meminfo)
      ram_gib=$(( kb / 1024 / 1024 ))
    fi ;;
  macos)
    bytes=$(sysctl -n hw.memsize 2>/dev/null || echo 0)
    ram_gib=$(( bytes / 1024 / 1024 / 1024 )) ;;
  windows)
    # MSYS2 exposes /proc/meminfo; git-bash may not — fall back to wmic.
    if [ -r /proc/meminfo ]; then
      kb=$(awk '/^MemTotal:/{print $2}' /proc/meminfo)
      ram_gib=$(( kb / 1024 / 1024 ))
    else
      bytes=$(wmic ComputerSystem get TotalPhysicalMemory 2>/dev/null | tr -dc '0-9' || echo 0)
      [ -n "$bytes" ] && ram_gib=$(( bytes / 1024 / 1024 / 1024 ))
    fi ;;
esac

# ---- detect logical cores --------------------------------------------------
cores=1
case "$os" in
  linux)   cores=$(nproc 2>/dev/null || echo 1) ;;
  macos)   cores=$(sysctl -n hw.ncpu 2>/dev/null || echo 1) ;;
  windows) cores=${NUMBER_OF_PROCESSORS:-$(nproc 2>/dev/null || echo 1)} ;;
esac

# ---- detect filesystem reflink capability (cheap CoW slots) ---------------
fstype="$(findmnt -no FSTYPE --target . 2>/dev/null || echo unknown)"
reflink="no"
case "$os:$fstype" in
  macos:apfs)      reflink="yes" ;;     # APFS clonefile
  linux:btrfs)     reflink="yes" ;;
  linux:xfs)       reflink="yes" ;;     # reflink=1 (default on modern XFS)
  linux:zfs)       reflink="yes" ;;     # zfs block cloning
esac

# ---- classify --------------------------------------------------------------
if   [ "$ram_gib" -le 12 ];                              then tier="constrained"
elif [ "$ram_gib" -ge 48 ] && [ "$cores" -ge 12 ];       then tier="workstation"
else                                                          tier="standard"
fi

# ---- knobs per tier --------------------------------------------------------
# max_cargos_build : parallel FULL cargo build/test processes (each saturates
#                    many cores) — the ceiling the operator schedules against.
# max_cargos_check : parallel `cargo check -p` (light, short) — much higher.
# rust_analyzer    : off | on | full  (full = RA-as-oracle, was RAM-blocked)
# slots            : required | optional
# nextest_jobs     : test concurrency for the module gate
# linker           : the fast linker for this OS
case "$tier" in
  constrained)
    max_cargos_build=3;  max_cargos_check=3
    rust_analyzer="off"; slots="required"; nextest_jobs=4 ;;
  standard)
    max_cargos_build=$(( cores/4>1 ? cores/4 : 2 ));  max_cargos_check=$(( cores/2>2 ? cores/2 : 3 ))
    rust_analyzer="on";  slots="optional"; nextest_jobs=$(( cores/2>2 ? cores/2 : 2 )) ;;
  workstation)
    max_cargos_build=$(( cores/6>4 ? cores/6 : 4 ));  max_cargos_check=$(( cores/3>8 ? cores/3 : 8 ))
    rust_analyzer="full"; slots="optional"; nextest_jobs="$cores" ;;
esac

case "$os" in
  macos)   linker="ld64.lld (Mach-O)"; linker_note="mold is ELF-only, INCOMPATIBLE with macOS" ;;
  linux)   linker="mold";              linker_note="set globally in ~/.cargo/config.toml" ;;
  windows) linker="rust-lld";          linker_note="MSVC target uses rust-lld (repo .cargo/config.toml)" ;;
  *)       linker="default";           linker_note="" ;;
esac

# ---- emit ------------------------------------------------------------------
if [ "${1:-}" = "--env" ]; then
  cat <<EOF
PH2D_HW_TIER=$tier
PH2D_OS=$os
PH2D_RAM_GIB=$ram_gib
PH2D_CORES=$cores
PH2D_FSTYPE=$fstype
PH2D_REFLINK=$reflink
PH2D_MAX_CARGOS_BUILD=$max_cargos_build
PH2D_MAX_CARGOS_CHECK=$max_cargos_check
PH2D_RUST_ANALYZER=$rust_analyzer
PH2D_SLOTS=$slots
PH2D_NEXTEST_JOBS=$nextest_jobs
EOF
  exit 0
fi

bold=""; dim=""; rst=""
if [ -t 1 ]; then bold="\033[1m"; dim="\033[2m"; rst="\033[0m"; fi
printf "${bold}PH2D hardware profile${rst}  →  tier = ${bold}%s${rst}\n" "$tier"
printf "${dim}%s · %s GiB RAM · %s cores · %s (reflink: %s)${rst}\n" "$os" "$ram_gib" "$cores" "$fstype" "$reflink"
echo   "─────────────────────────────────────────────────────────────"
printf "  parallel cargo (build)   %s\n" "$max_cargos_build"
printf "  parallel cargo (check)   %s\n" "$max_cargos_check"
printf "  rust-analyzer            %s\n" "$rust_analyzer"
printf "  CoW slots                %s\n" "$slots"
printf "  nextest jobs             %s\n" "$nextest_jobs"
printf "  linker                   %s  ${dim}(%s)${rst}\n" "$linker" "$linker_note"
echo   "─────────────────────────────────────────────────────────────"
case "$tier" in
  constrained) echo "  → Follow DIRETRIZ §6.6 verbatim (this is the baseline)." ;;
  standard)    echo "  → §6.6 baseline + measured overrides above. Pilot heavy levers before mandating." ;;
  workstation) echo "  → §6.6 OVERRIDDEN: RA-as-oracle, high parallelism, slots optional." ;;
esac

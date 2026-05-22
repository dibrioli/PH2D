#!/usr/bin/env bash
# sync-llm-memory.sh — versioned backup of the LLM's persisted memory.
#
# WHY IT EXISTS: the LLM memory (~/.claude/projects/<slug>/memory/) is the only
# irreplaceable artifact NOT in the project's GitHub repo — it lives on the
# local disk and holds profile / feedback / accumulated state. This script
# mirrors it to a SEPARATE PRIVATE repo (dibrioli/PH2D-llm-memory) and commits +
# pushes only when something changed. It runs unattended via a daily launchd
# agent (com.ph2d.llm-memory-backup) — no one has to remember. It can also be
# run by hand: `bash scripts/sync-llm-memory.sh`.
#
# The PH2D repo is PUBLIC, so the memory goes to a PRIVATE repo, never here.
set -euo pipefail

MEMORY_DIR="$HOME/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory"
BACKUP_DIR="$HOME/.ph2d-llm-memory-backup"
REPO_URL="https://github.com/dibrioli/PH2D-llm-memory.git"

log() { echo "[sync-llm-memory $(date '+%H:%M:%S')] $*"; }

if [[ ! -d "$MEMORY_DIR" ]]; then
    log "memory dir not found at $MEMORY_DIR — nothing to do."
    exit 0
fi

# Persistent (incremental) clone of the backup repo. First run: clone it; if the
# repo is still empty the clone may warn/fail, so init it locally instead.
if [[ ! -d "$BACKUP_DIR/.git" ]]; then
    log "first run — cloning $REPO_URL"
    git clone "$REPO_URL" "$BACKUP_DIR" 2>/dev/null || {
        mkdir -p "$BACKUP_DIR"
        git -C "$BACKUP_DIR" init -q
        git -C "$BACKUP_DIR" remote add origin "$REPO_URL" 2>/dev/null || true
        git -C "$BACKUP_DIR" branch -M main
    }
fi

# README of the backup repo (written once).
if [[ ! -f "$BACKUP_DIR/README.md" ]]; then
    cat > "$BACKUP_DIR/README.md" <<'EOF'
# PH2D — LLM persisted-memory backup (PRIVATE)

Versioned mirror of `~/.claude/projects/<slug>/memory/` for the PH2D project.
Updated automatically by `scripts/sync-llm-memory.sh` (in the public PH2D repo)
via a daily launchd agent. **Do not hand-edit** — the source of truth is the
local memory; this repo is only the off-site backup.
EOF
fi

# Mirror the memory (--delete propagates removals) so the snapshot stays exact.
mkdir -p "$BACKUP_DIR/memory"
rsync -a --delete "$MEMORY_DIR/" "$BACKUP_DIR/memory/"

cd "$BACKUP_DIR"
git add -A
if git diff --cached --quiet; then
    log "memory unchanged — nothing to commit."
    exit 0
fi
git commit -q -m "memory snapshot $(date '+%Y-%m-%d %H:%M:%S')"
git push -q origin main
log "backup updated and pushed."

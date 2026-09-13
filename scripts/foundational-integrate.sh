#!/usr/bin/env bash
# foundational-integrate.sh — Modo L tested integration gate (ADR-0107, Camada 2).
#
# `git merge --ff-only` proves "nobody landed between my rebase and my merge";
# it does NOT prove the COMBINED tree compiles. For a drop-crate line that's
# fine (physical isolation guarantees it). For a line that touched foundational
# it is exactly where a silent semantic/build conflict hides (agent A changes a
# signature, agent B adds a caller — each compiles alone, together they break).
#
# This script closes that hole: rebase → re-sync → BUILD+TEST the rebased tip
# (== the future main) → only then ff-merge. Because --ff-only makes the tip
# BECOME main, a green rebased tip is a green main. If another line lands in the
# window, the ff-merge fails → rerun (rebases onto it, re-tests). No line ever
# merges an untested combination. (This is the Zuul/Bors gate, one slot.)
#
# Run from INSIDE your line worktree, module gate already batched-green.
# Usage:  bash scripts/foundational-integrate.sh
set -euo pipefail

# ---------- preconditions ----------
BRANCH="$(git rev-parse --abbrev-ref HEAD)"
PRIMARY="$(dirname "$(git rev-parse --path-format=absolute --git-common-dir)")"
HERE="$(git rev-parse --show-toplevel)"

if [[ "$HERE" == "$PRIMARY" ]]; then
    echo "✗ You are in the PRIMARY checkout, not a line worktree." >&2
    echo "  Integrate FROM a line worktree (Worktrees/line-<módulo>/)." >&2
    exit 1
fi
if [[ "$BRANCH" != line/* && "$BRANCH" != integ/* ]]; then
    echo "✗ HEAD is '$BRANCH', not a line/* or integ/* branch — refusing to integrate." >&2
    exit 1
fi
# Primary must be clean and on main (--ff-only lands there).
# ⚠️ `project-memory/` fica FORA desta pergunta: `~/.claude/projects/<key>/memory` é symlink para
# ela, logo TODA sessão viva suja o primário ao aprender algo (medido 13/09: 22 entradas, todas
# ali). Contá-la fazia o script recusar SEMPRE, a acusar «alguém a codar no main» — falso. Sujo
# FORA dela continua a ser a violação do §1.5.1.
PRIMARY_DIRTY="$(git -C "$PRIMARY" status --porcelain -- . ':(exclude)project-memory')"
if [[ -n "$PRIMARY_DIRTY" ]]; then
    echo "✗ Primary checkout ($PRIMARY) is dirty outside project-memory/ — refusing." >&2
    echo "$PRIMARY_DIRTY" | sed 's/^/    /' >&2
    echo "  Code changes in the primary mean someone is working in main (DIRETRIZ §1.5.1 violation)." >&2
    exit 1
fi
if [[ "$(git -C "$PRIMARY" symbolic-ref --short HEAD)" != "main" ]]; then
    echo "✗ Primary checkout is not on 'main' — refusing." >&2
    exit 1
fi
# Your own work must be committed before we rebase.
if [[ -n "$(git status --porcelain)" ]]; then
    echo "✗ This worktree has uncommitted changes — commit them first (§1.5.2.2)." >&2
    exit 1
fi

echo "▸ Integrating $BRANCH → main (primary: $PRIMARY)"

# ---------- 1. rebase onto the current main ----------
echo "▸ [1/5] git rebase main"
if ! git rebase main; then
    cat >&2 <<'EOF'
✗ rebase hit a conflict. The ONLY legitimate conflicts (DIRETRIZ §1.5.5):
    • Cargo.lock            → NEVER by hand: `git checkout main -- Cargo.lock`
                              then `cargo check -p <sua-crate>`, `git add Cargo.lock`
    • *-registry-init/      → NEVER by hand: accept either side, re-run the sync
    • icons.rs (IconId)     → keep BOTH variants, alphabetical
Everything else is the integrator's to resolve — by the index STAGES (`git show :1:<f>` base ·
`:2:` ours · `:3:` theirs), never by the markers. Numbered lists / ratchets: `merge=text` in
"$(git rev-parse --git-common-dir)/info/attributes" while integrating (Mergiraf can DROP one
side's deletion in a list and still print «Solved» — 13/09), with a count assert.
Same-symbol DESIGN collision (two lines rewrote the same core fn) → STOP, report to Enio (ADR-0107).
⚠️ A rebase WITHOUT conflicts is not proof either: compare every rebased commit with its original
(per-file multiset of +/- lines, positive control) — DIRETRIZ §1.5.5.
Resolve, `git rebase --continue`, then re-run this script.
EOF
    exit 1
fi

# ---------- 2. re-sync codegen, auto-commit ONLY generated targets ----------
echo "▸ [2/5] re-sync codegen (tool/node/app registries)"
cargo run -q -p ph2d-tool-sync
cargo run -q -p ph2d-node-sync
# ⚠️ O TERCEIRO gerador nasceu na W2/L0 (2026-09-11) e este passo não o conhecia: o
# `ph2d-app-registry-init` lista as FAMÍLIAS da shell (crates/ph2d-app-*), e o bloco dele é
# append-only por construção — cinco linhas vão acrescentar a família delas. Sem esta linha,
# TODA integração de família reprovava no passo 3 com «o corpo de `register_all_app_families`
# não descreve a árvore», que é o gate de staleness a funcionar sobre um passo que faltava.
# ⛔ Um gerador novo que não entre aqui transforma o gate dele num bloqueio de integração em vez
# de uma rede: a regeneração é trabalho mecânico do integrador, como os outros dois.
cargo run -q -p ph2d-app-sync
GEN_GLOBS=('crates/ph2d-tool-registry-init' 'crates/ph2d-node-registry-init'
           'crates/ph2d-panel-registry-init' 'crates/ph2d-app-registry-init'
           'crates/ph2d-editor-core/src/icons.rs'
           'Cargo.lock')
git add -- "${GEN_GLOBS[@]}" 2>/dev/null || true
# Anything dirty OUTSIDE the generated targets after a sync is unexpected.
if [[ -n "$(git status --porcelain --untracked-files=no | grep -v '^[MA] ' || true)" ]]; then
    echo "✗ Post-sync working tree dirty outside generated targets — inspect manually." >&2
    git status --short >&2
    exit 1
fi
if [[ -n "$(git diff --cached --name-only)" ]]; then
    git commit --no-verify -q -m "chore(sync): regenerate registries post-rebase"
    echo "    committed registry regen"
fi

# ---------- 3. staleness gate (generated surfaces match the crate glob) ----------
echo "▸ [3/5] registry staleness"
cargo test -q -p ph2d-tool-registry-init -p ph2d-node-registry-init -p ph2d-app-registry-init

# ---------- 4. combined-tree BUILD gate (the ADR-0107 core) ----------
# Foundational touched anywhere? → the whole workspace must still compile.
# Only docs/ is exempt. ⚠️ As drop-crates deixaram de estar isoladas de quem as LÊ (medido 13/09):
# desde a W2, 7 famílias `ph2d-app-*` dependem de `ph2d-tool-*`, 8 de `ph2d-panel-*` e 1 de
# `ph2d-node-*` — um `check -p` na crate mudada não compila os leitores, e é neles que a quebra aparece.
CHANGED="$(git diff --name-only main...HEAD)"
FOUNDATIONAL_TOUCH="$(echo "$CHANGED" | grep -vE '^docs/' || true)"
echo "▸ [4/5] combined-tree build gate"
if [[ -n "$FOUNDATIONAL_TOUCH" ]]; then
    echo "    code touched → CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets"
    echo "$FOUNDATIONAL_TOUCH" | sed 's/^/      · /'
    # `--all-targets` + avisos como ERRO (a paridade do ship.sh/CI): um `dead_code` que só existe na
    # árvore COMBINADA — cada linha apagou os usos DELA e o último morreu na fusão — passava por
    # aqui verde e só o ship.sh o via (13/09, dois casos).
    CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets
else
    echo "    docs only → nothing to compile"
    CRATES="$(echo "$CHANGED" | sed -n 's#^crates/\([^/]*\)/.*#-p \1#p' | sort -u | tr '\n' ' ')"
    # shellcheck disable=SC2086
    [[ -n "$CRATES" ]] && cargo check $CRATES || echo "    (no crate changes to check)"
fi

# ---------- 5. impacted tests, then land ----------
echo "▸ [5/5] impacted tests"
BASE=main bash scripts/nextest-impacted.sh

echo "▸ merge --ff-only into main"
if git -C "$PRIMARY" merge --ff-only "$BRANCH"; then
    echo "✓ $BRANCH integrated into main. Green combined tree landed."
    echo "  Ship/push: SÓ por ordem EXPLÍCITA do Enio (CLAUDE.md §0.7) — nunca como passo seguinte automático."
else
    echo "✗ --ff-only failed: another line landed after your rebase." >&2
    echo "  That is the serialization working — just re-run this script (rebase+retest)." >&2
    exit 1
fi

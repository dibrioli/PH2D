---
name: project_integration_prefork_lines_ship_drift
description: Linhas forkadas antes de um cutover grande carregam ship-blockers latentes que o foundational-integrate NÃO pega — só o ship.sh final
metadata:
  type: project
---

Jornada 2026-07-06/07: integrei 3 linhas (imageio, audio, Painter) ao main **depois** do cutover do Vector (ADR-0108). Todas forkaram PRÉ-cutover. Lições da integração (o integrador da última linha faz ship+push+babysit, §1.5.4):

**O que `foundational-integrate.sh` pega vs. o que só o `ship.sh` pega:**
- `foundational-integrate.sh` roda: rebase → tool/node sync → staleness → `cargo check --workspace` → **nextest-impacted**. Ele **NÃO roda** fmt, clippy `--all-targets`, typos, nem panel-sync/widget-sync/chrome-sync (rode esses à mão pós-rebase; no meu caso sync deu no-op).
- Como `nextest-impacted` roda os arch-tests, ele **pegou** os gates de arch da linha audio (`no_tofu_glyphs` = `→` U+2192 num `println!`; `arch_safe_clamp_only` = `.clamp()` cru em widget → trocar por `crate::math::safe_clamp`). Falhou ANTES do ff-merge (gate funcionando).
- **fmt e typos NÃO são nextest** → passam despercebidos pela integração e só vermelham no `ship.sh`/CI. Uma linha pré-cutover carrega **drift de fmt latente**: o `.rustfmt` do main é `style_edition = "2024"` + `max_width=100`; código fmt'd sob config antigo (single-line calls) vira não-canônico → `cargo fmt` reescreve multiline. Peguei isso em `level_meter.rs` (na integração) e depois em `ph2d-audio`/`ph2d-panel-audio-mixer` inteiros (no ship). Fix: `cargo fmt -p <crates>` na worktree isolada, commit, re-ff-merge.

**Procedimento que funcionou:** integrar do mais seguro (drop-crate ortogonal) ao mais foundational; rebase de cada linha foi **limpo** (Mergiraf + pastas disjuntas — zero conflito mesmo pós-cutover de 30 crates). Painter tinha 18 arquivos UNCOMMITTED → commitar na worktree ANTES do rebase. No fim: `ship.sh` 1× sobre o main combinado + os 3 gates CI-only das [[feedback_ship_parity_gaps_ci_only]] (advisory-db `pull --ff-only`, `cargo deny --all-features`, `ph2d-bindgen --check`). ship 7/7 + CI matrix verde (run 28834404504).

**Regra:** ao integrar uma linha que forkou antes de um cutover grande, **assuma drift de fmt/typos latente** e rode o `ship.sh` completo no fechamento — o gate da árvore combinada da integração é necessário mas NÃO suficiente. Ver também [[feedback_cargo_fmt_p_reformats_foreign_wip]] (aqui foi seguro: worktree isolada, sem WIP alheio).

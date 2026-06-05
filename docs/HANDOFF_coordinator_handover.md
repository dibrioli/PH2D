═══════════════════════════════════════════════════════════════════
HANDOFF → PRÓXIMO COORDENADOR · estado pós-ship + threads abertas + lições
Autor: Coordenador (jornada 2026-06-04→05, pós-ship Vector W1-W5 + SDF + Painter W4)
═══════════════════════════════════════════════════════════════════

## §0 — TL;DR (4 coisas)
1. **MARCO GRANDE SHIPADO E VERDE.** `origin/main = 1156441`. CI run
   [26992155064](https://github.com/dibrioli/PH2D/actions/runs/26992155064) **success**
   (3-OS ubuntu/macOS/windows + replay-hash + ECS/physics + MSRV). 78 commits de TODOS
   os agentes (Vector + Painter + Coord) pushados de uma vez.
2. **HEAD local = `b096c0d`** · 1 commit ahead (só o registro de ship em SESSION_ACTIVE,
   **docs-only, NÃO pushado** de propósito — não dispare CI 3-OS de ~22min por um doc).
3. **Working tree:** só `.vscode/settings.json` + docs `?? `alheios/inconsequentes +
   `test_strip` + `docs/UI_Fonts/`. **Zero `.rs` uncommitted.** Baseline limpa.
4. **Arranque:** DIRETRIZ §0 (sanity) + leia §2-§4 abaixo + `SESSION_ACTIVE` (mapa de posse
   vivo). Pegue a próxima ordem do Enio OU as threads abertas (§2).

## §1 — O QUE SHIPOU NESTA JORNADA
- **Vector W1·W2·W3·W4·W5** — TODOS fechados + auditados (T3.5/T4.13/T5.3, relatórios
  `docs/AUDIT_vector_w{3,4,5}_session_2026-06-0{4,5}.md`).
- **SDF Hybrid (ADR-0065)** — CPU core + GPU compute + draft/reconcile no `vector_graph_bridge`
  + gate `vector_sdf_real_time` (64-path boolean @ 5.33ms < 8.33ms/120FPS). 100% fechado.
- **Painter W4** — Curves/Levels/B&W/Selective Color/Gradient Map adjustment panels + tokens
  cromáticos `curve-r/g/b`.
- **Variable-width stroke (W5 T5.1)** — `ph2d_vector::draw_variable_width_stroke` (render-time,
  per-sample) + `WidthProfile` na StrokeStyle (paramétrico/persistido) + pressão→largura no Pencil.
- **11 geometry nodes** (W4 fan-out) + offset + boolean.

## §2 — THREADS ABERTAS (próximo trabalho, por dono)
- **[COORD] Painter spatial GPU multipass infra** — ACEITO, é tua arquitetura (`ph2d-render`).
  Compositor é single-pass per-pixel (zero vizinho); filtros espaciais (Gaussian/Bloom/Sharpen/
  Motion/ChromaticAberration/ShadowsHighlights) precisam de pass-graph segmentado + ping-pong +
  materialize-below + dirty-rect⊕raio + `LayerOp::SpatialAdjustment`. **Faseamento: Gaussian spike
  primeiro.** O impl entrega a ref CPU + pesos do kernel; tu fazes o mecanismo. Briefing +
  minha RESPOSTA: [`HANDOFF_painter_w4_spatial_multipass_gpu_coord.md`](HANDOFF_painter_w4_spatial_multipass_gpu_coord.md).
- **[IMPL Painter] Noise + Halftone** (DESBLOQUEADO — per-pixel, switch escalar via `gpu_code()`,
  não espera a infra) + a **ref CPU do Gaussian**. + **débito documentado** (não-bloqueante):
  tokenizar os `1.5px` ring-outlines (hoje `// LITERAL-PX-OK` em `paint_adjust.rs`) + split
  `paint_adjust.rs` (829 LOC) + `event.rs::apply_event_impl` (299) — estão em OVERAGE_OK no gate.
- **[IMPL Vector] W6** (procedural fill / shader graph, plano §9) quando o Enio liberar.
  `pattern-along-path` (12º geometry node) → **W8** (binário + painter-brush).
- **Deferidos (não-bloqueantes):** smoke visual do variable-width (precisa device de pressão —
  Enio não tem) · ativação SDF asset/tool-mode (gated na UI de editor de grafo) · Piece C node bulge.

## §3 — AGENTES ATIVOS + POSSE (RAM ≤3 cargos)
- **Painter impl: ATIVO.** Posse: `ph2d-panel-painter-layers`, `ph2d-tool-painter`,
  `ph2d-painter-brush`, + dispatch do curve-editor em `editor-core` (CurvePoint).
- **Vector impl: livre p/ W6.** Posse: `ph2d-node-vector-*` (novos), `ph2d-vector-doc/{crdt,spiro}`,
  tool-bridges `vector_*_bridge`.
- **Coord:** `ph2d-render` (spatial infra), `ph2d-vector-sdf`, `vector_graph_bridge`, foundational
  (`ph2d-tokens`/`editor-core` widget+chrome/`ph2d-vector`/`ph2d-vector-doc` contrato), `mod.rs`
  (CONTENDED — coordena), ship.

## §4 — DISCIPLINA / LIÇÕES DESTA JORNADA (leia antes do próximo ship)
- **`--no-verify` dos agentes ESCONDE gate-drift.** Este ship pegou **9 blockers** que os hooks
  pulados deixaram passar: fmt-drift, Cargo.lock stale, clippy, typos, panel-LOC, tofu, hex,
  14×magic. **Ship-prep = rodar `ship.sh` + consertar TODO `✗` antes do push.** O Coord absorve
  isso (inclusive UI-canônico de crate alheio — anota literais objetivamente-corretos: sRGB=math,
  gradient=data; tokenização fica como follow-up do dono).
- **`nextest` cancela no 1º fail** → use **`cargo nextest run --workspace --cargo-profile ci-test
  --no-fail-fast`** pra enumerar TODAS as falhas de uma vez (senão é 1-ciclo-de-ship.sh por falha).
- **fmt 1.95 move comentário pós-`{` pra dentro do bloco** → um `// LITERAL-PX-OK` numa linha de
  condição (`if x <= 0.003 { // ...`) some. Fix: extrai o literal pra um `let` (statement line,
  trailing comment é fmt-estável). E `// LITERAL-{COLOR,PX}-OK` tem que ser **MESMA-LINHA** do
  literal/match; call multi-linha longa → colapsa pra single-line curta.
- **Índice git COMPARTILHADO entre agentes** → SEMPRE `git commit --no-verify -m "msg" -- <só teus
  paths>` (scoped). NUNCA `git commit` sem pathspec (agarra o staged alheio). `git status` antes.
- **NÃO rode 2 `ship.sh` concorrentes** (eu errei: 2× background no mesmo `slot-coord` + mesmo log →
  garbled + colisão de target lock). Um de cada vez.
- **Ship com WIP `.rs` alheio:** valide o committed HEAD via `git worktree add --detach HEAD`
  (não rode ship.sh sobre tree-suja — mascara drift). Neste ship NÃO havia `.rs` WIP (só docs/lock)
  → shipei do main direto.
- **CI 3-OS ~22min.** Babysit: `gh run watch <id> --exit-status` em background (notifica no fim).
  Docs-only NÃO pusha sozinho (CI à toa).

## §5 — REFERÊNCIA RÁPIDA
- Estado por-módulo: `CLAUDE.md §5`. Contratos congelados: `§6`. Posse viva: `SESSION_ACTIVE`.
  Memória/perfil/lições: `MEMORY.md`. Slot: `target-slots/slot-coord` (`scripts/slot-seed.sh coord`).
- Ship: `./scripts/ship.sh` (paridade CI, roda TODOS os gates + sumariza ✓/✗ no fim — NÃO é
  fail-fast no script, mas o nextest interno é). Link: `gh run list --workflow=spike.yml --limit=1`.
- Handoffs vivos: spatial-infra (Painter §2), `HANDOFF_vector_w5_integration_impl.md`, os AUDIT_*.
═══════════════════════════════════════════════════════════════════

---
name: canonical-files-and-paths
description: "Onde estão os docs canônicos vigentes (DIRETRIZ v6.1, SKILL v2.14, CLAUDE, ADRs 0027-0029, design system). Snapshot 2026-05-19 noite — verificar git log antes de citar SHA específico."
metadata: 
  node_type: memory
  type: reference
  originSessionId: fe59209c-4f42-43aa-a540-0a60c10ff373
---

Diretório raiz: `/Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/`.

## Docs canônicos vivos (ordem de leitura recomendada)

### Operacional / processo

1. **[`docs/IntegracaoMultiAgente/DIRETRIZ.md`](file:///Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/docs/IntegracaoMultiAgente/DIRETRIZ.md)** — Diretriz de Implementação Universal v6.1 (~833 LOC).
   - §0 sanity check
   - §1 modelo 2 papéis (Coord + Implementador) + fluxo invertido + 3 obrigações
   - §2 comunicação Coord ↔ Implementador (Enio relay mecânico, briefing template)
   - §3 receitas canônicas (tool / painel / widget / chrome action / modificar / foundational / **cross-cutting §3.7**)
   - §4 UI canonical (tokens.json → ph2d-tokens → resolve(theme); 11 arch gates listados)
   - §5 codificação rápida (cargo check -p; LOC threshold 1200; **§5.4 T2 escopado vs workspace; §5.6 como NÃO escrever test slow**)
   - §6 disciplina git (stage→commit atômico; **§6.4 armadilhas — typos pt-BR, cargo lock**)
   - §7 smoke + push + babysit CI (Coord absorve PRCI)
   - §9 cheat-sheet (HRs + paths físicos + comandos)

   **Substituiu em 2026-05-19** 8 docs anteriores (DIRETRIZ v5.0, 01-04 docs operacionais, STATE.md, DIRETRIZ_CODIFICACAO_RAPIDA, PARALLEL_AGENTS_PROBLEM) — todos arquivados em `docs/archive/multi-agente-pre-v6.0/`.

2. **[`CLAUDE.md`](file:///Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/CLAUDE.md)** — workflow operacional curto (~80 LOC). CI section + Cadência de validação atualizadas pra v6.0+. Aponta pra DIRETRIZ §5 e §7.

### Stack / arquitetura técnica

3. **[`SKILL_Stack_PH2D_Definitiva.md`](file:///Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/SKILL_Stack_PH2D_Definitiva.md)** — fonte de verdade técnica (~1050 LOC, v2.14 — Wave 9 cravada).
   - §HR-1..HR-18+ Hard Rules (citáveis por ID; gated por arch tests)
   - Stack canônico (wgpu, vello, kurbo, parley, harfrust, skrifa, rapier, bevy_ecs, mlua, etc.) com versões pinadas
   - Arquitetura 1 core + 4 shells (PC/Mac/iPad/Web)
   - Subsistemas (rendering, vetorial+SDF, text, físcs, fluidos, scripting, MCP, a11y, i18n)
   - Tiebreakers (perf hot path > determinismo > segurança > a11y > UX iPad > APIs estáveis > LLM-friendly)
   - Linha 12 aponta pra DIRETRIZ.md + archive narrativa multi-agente.

4. **[`docs/architecture/decisions/`](file:///Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/docs/architecture/decisions/)** — ADRs (snapshots históricos de decisão):
   - 0003 ecs-choice (bevy_ecs 0.18)
   - 0019-0026 spike output + GPU lifecycle + sim/presentation boundary + sprite strategies + UI baseline + editor input + GameObject model
   - **0027 convention-by-discovery** (Wave 1) — tool-as-crate
   - **0028 wave-2-codegen-design-canonical** (Wave 2 + amendments Wave 5/8/9) — tokens.json codegen, design canonical
   - **0029 trait-driven-panel-host** (Wave 8 / ADR-0029, fechada) — Panel<State> trait + panel registry

   ADRs **não são editadas retroativamente** — são snapshots do contrato no momento da decisão.

### Design system

5. **[`docs/design/`](file:///Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/docs/design/)** — fonte canonical de UI:
   - `tokens.json` — fonte raw de 4 temas OKLCH + spacing/radius/typography/stroke/chrome
   - `styles/tokens.css` — aliases CSS pra mockups (var(--*))
   - `screens/*.html` — mockups (gated por `mockup_tokens_exist` test)
   - `tools/*.toml` — design canonical TOML per tool (gated por `tool_manifest_design_sync`)
   - `icons/*.svg` — Lucide-style 24×24 currentColor
   - `PROMPT_CLAUDE_DESIGN.md` — brief pra Claude Design gerar tokens + mockups + icons + specs

## Caminhos físicos canônicos (extensão multi-agente)

| O que | Onde |
|-------|------|
| Tool nova | `crates/ph2d-tool-<slug>/` (1 crate isolado per tool) |
| Painel novo | `crates/ph2d-panel-<slug>/` (1 crate per panel, feature-gated) |
| Widget primitive | `crates/ph2d-editor-core/src/widget/<slug>.rs` (≤500 LOC cap) |
| Chrome handler (TopBar action) | `crates/ph2d-editor-core/src/screens/hero/chrome/<slug>.rs` (1 handler per file) |
| Tool registry init | `crates/ph2d-tool-registry-init/src/lib.rs::register_all` (alfabético, gated) |
| Panel registry init | `crates/ph2d-panel-registry-init/src/lib.rs::register_all_panels` |
| Widget showcase | `crates/ph2d-editor-core/src/widget/showcase/` |
| Arch tests editor | `crates/ph2d-editor-core/tests/` (11 gates ativos) |
| Arch tests tokens | `crates/ph2d-tokens/tests/mockup_tokens_exist.rs` |

## Memória persistente

`/Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/MEMORY.md` (este index) — auto-loaded.

## Arquivos arquivados em `docs/archive/` (NÃO referenciar como canônico)

Limpeza 2026-05-22 (`git mv`, histórico preservado):
- `docs/archive/multi-agente-pre-v6.0/` — 8 docs operacionais antigos (01-04 papéis, STATE.md, DIRETRIZ_CODIFICACAO_RAPIDA, PARALLEL_AGENTS_PROBLEM). Substituídos pela DIRETRIZ unificada.
- `docs/archive/handoffs-completed/` — `HANDOFF.md` (bootstrap), `HANDOFF_M13_UI.md`, `HANDOFF_WAVE_8_PHASE_C{,2,3,4,_CI}.md`.
- `docs/archive/migracao-waves-completed/` — wave-2-5-deferred-splits, wave-3-2-remaining-shell-decomp, wave-3-deferred-state, wave-8-phase-b-completed.
- `docs/archive/plans-completed/` — color-picker-fix, editor-hero-screen, hero-deep-polish, ui-components.
- `docs/archive/README.md` explica tudo.

## NÃO arquivados (ainda ativos / referenciados por ADR ou código) — caminho ATUAL

- `docs/HANDOFF_node_system.md` — tracker vivo do sistema de nós.
- `docs/Migracao/2026-05-{node-centric-architecture,foundational-parallelism-three-bottlenecks}.md` — design node-centric ativo.
- `docs/Migracao/2026-05-{convention-by-discovery,wave-2-eliminating-all-collisions}.md` — citados por ADR-0027/0028 (mantidos em Migracao/).
- `docs/plans/2026-05-{node-waves,post-spike,editor-input-pipeline}.md` — node-waves ativo, post-spike canônico (CLAUDE), input-pipeline citado por ADR-0024.
- `docs/spike/` — citado por ADR-0003 + ADR-0019 (mantido).
- `docs/scripting/` — referenciado por código (`tests/spike/.../c8_llm_gen.rs` lê o path; `ph2d-mcp/catalog.rs` cita c6-prompts) + SKILL + ADR-0019 (mantido).
- `docs/perf/mouse-stutter.md` — conhecimento de bug, fix #2 pendente (mantido).

Vide também [[project-multi-agent-v6-2026-05-19]], [[project-perf-audit-2026-05-19]], [[project-wave-9-completed]].

# Deform — sistema de transformação & deformação do Painter

> **Tracker único** deste módulo (DIRETIVA §1). O resto é histórico/detalhe nos
> companheiros. Uma ferramenta unificada de **Transform + Liquify + Puppet/Mesh**,
> projetada para ser **superior ao Procreate** evitando redundância, complexidade de
> uso e dificuldade de implementação.

**Status:** 📐 DESENHO FECHADO · implementação NÃO iniciada (bloqueada por posse — ver §Coordenação).
**Autor do plano:** Coordenador (sessão 2026-07-02).
**Origem:** pedido do Enio — botão de deformação na rail esquerda do Painter, UI no inspector direito, superior ao Procreate, cards, belo.

## Índice dos docs
| Doc | Conteúdo |
|---|---|
| [`01_research.md`](01_research.md) | Procreate (Transform 4 modos + Liquify 7 modos) + estado-da-arte além do Procreate + referências publicadas portáveis |
| [`02_design_and_architecture.md`](02_design_and_architecture.md) | Kernel único (inverse-warp), IA da ferramenta, o que nos torna **superiores**, decisões e trade-offs |
| [`03_ui_ux_panel_spec.md`](03_ui_ux_panel_spec.md) | Painel do inspector — cards, widgets (ancorado na Widget Gallery), mockup, tokens |
| [`04_implementation_plan.md`](04_implementation_plan.md) | Plano por waves, **sites file:line exatos**, DoD/gates/kill-criteria, isolamento |

## TRIAGEM (DIRETRIZ §2)
- **Tarefa:** ferramenta de transformação/deformação no Painter (rail esquerda + UI no inspector), superior ao Procreate.
- **Caminho:** **misto** — Wave 1 = **(D) modificar features existentes** (`ph2d-tool-painter`, `ph2d-panel-painter-layers`) **+ (C) Coord-only** para 1 botão na rail (foundational `ph2d-editor-core`). Waves 2-3 (gizmo interativo) = **(C)/(B) Coord** (dispatch InteractiveState em editor-core). Wave 4 (warps paramétricos) = **(A) fan-out drop-crate** (`ph2d-node-*`).
- **Toca contrato congelado?** **Não.** `PaintMode` (enum interno), `BrushSettings` (snapshot), canal genérico `PanelEvent::{SelectOption,SetValue,Click}` — nada gateado. `CanvasPaintTool` (cap=1, ADR-0040-am-3) é **reusado sem mudança**. Sem ADR para Wave 1.
- **Razão:** o paradigma canvas-pointer + snapshot-flag + Apply/undo estrutural já existe e absorve toda a Wave 1 dentro do painter; só o botão da rail cruza foundational.

## Coordenação / isolamento (LEIA ANTES DE IMPLEMENTAR)
⚠️ **Bloqueio de posse.** No momento do desenho (2026-07-02) há um agente editando
`crates/ph2d-tool-painter/src/tool/paint.rs`, `.../paint/canvas_pointer.rs`,
`.../paint/selection.rs`, `.../paint/state_default.rs`, `.../paint/tests.rs`
(sistema de **Selection**, ADR-0103). A **Wave 1 do Deform toca os MESMOS arquivos**
(`paint_mode.rs`, `canvas_pointer.rs` ladder, `brush_settings.rs`/`snapshot.rs`,
`paint.rs`, e `ph2d-panel-painter-layers/*`). Portanto:

1. **Não iniciar Wave 1 até o agente de Selection landar** (commit + posse liberada no SESSION_ACTIVE).
2. A Wave 1 **reusa** a infraestrutura de Selection (freeze/protect via `selection_mask`,
   `restore_deselected_region`, `selection_coverage_at`) — então herdar, não duplicar.
3. O botão da rail (foundational `ph2d-editor-core`) é **Coord-only** e é o único ponto
   que cruza para fora do painter — pode ser preparado em paralelo (arquivos disjuntos da
   Selection), mas o `PaintMode::Deform` no painter espera a liberação.

**Sem commit nesta sessão** (pedido do Enio; agentes em paralelo).

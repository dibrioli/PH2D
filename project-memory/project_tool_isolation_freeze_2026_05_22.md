---
name: project-tool-isolation-freeze-2026-05-22
description: ADR-0040 fechado integralmente em 2026-05-22 (TG-A..TG-E). Tool = crate satélite drop-in; contrato Tool/ImageEditTool/PanelEvent congelado por arch-gate.
metadata: 
  node_type: memory
  type: project
  originSessionId: d6a21f34-c141-4992-af37-b8a238fc4645
---

ADR-0040 (Tool como crate de feature isolado) **fechou em 2026-05-22** em uma jornada de 5 fases (TG-A..TG-E). A engine agora trata tool igual node-crate: pasta exclusiva + `cargo run -p ph2d-tool-sync` regenera o wiring central, zero edit central por feature, contrato `Tool`/`ImageEditTool`/`PanelEvent` congelado por arch-gate.

**Why:** o projeto vai ter muitas tools (bgremoval foi só a primeira pesada). O norte é o mesmo do sistema de nós (ADR-0030/0031): crescer por adição de crate isolado para escalar com multi-agente sem colisão. Pré-ADR-0040, `editor-core/src/tools/` carregava ~512 LOC de vocab + 4 variants per-tool em `EditorAction`, e adicionar uma tool exigia 3 edits centrais.

**How to apply:**
- **Tool nova** = §3.9 da DIRETRIZ (não §3.1 — essa virou referência histórica). Três passos: largar crate em `crates/ph2d-tool-<slug>/` → `cargo run -p ph2d-tool-sync` → verificar 3 gates (`tool-registry-init` staleness + `architecture_tool_contract_surface` cap + crate test). Briefing pronto-pra-colar disponível.
- **Exemplos canônicos a copiar:** one-shot stateless = `ph2d-tool-trim-transparency` ou `make-square`; stateful leve = `ph2d-tool-padding`; stateful completo = `ph2d-tool-bgremoval` (preview cap + ImageEditTool + protect-mask).
- **Mudar o contrato `Tool`/`ImageEditTool`/`PanelEvent`** = evento raro Coordenador-only com amendment de ADR-0040: bump dos caps em [`architecture_tool_contract_surface.rs`](file:///Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/crates/ph2d-editor-core/tests/architecture_tool_contract_surface.rs). Cap atual: `Tool=10` métodos, `ImageEditTool=4`, `PanelEvent=4` variants.
- **EditorAction não ganha variant per-tool.** Os 4 genéricos cobrem tudo: `ActivateTool { tool_id }`, `OneShotImageOp { tool_id, entity_bits }`, `ToolPanelEvent(PanelEvent)`, `CancelActiveTool`. Se precisar de um quinto, é um Coordenador-only + ADR.
- **Edge panel→tool é permitida** (panel-bgremoval depende de tool-bgremoval; panel-padding depende de tool-padding). Codificado em `panel_crates_depend_only_on_editor_core`, que agora auto-descobre `crates/ph2d-panel-*` e bane apenas (a) dep em `ph2d-editor` shim, (b) cross-panel dep.

**Commits locais (não-pushados; ship é do Enio):**
- `5be7541` TG-A — generic ActivateTool + OneShotImageOp (7 variants per-tool removidos).
- `42438be` TG-A close — register pure-push + activate_default + register_all_tools codegen.
- `7676793` TG-B — bgremoval ToolPanelEvent + take_params_dirty + vocab migra.
- `4a15d9b` TG-C — padding (espelho mecânico de TG-B).
- `c4063b7` TG-D — `editor-core/src/tools/` deletado.
- `<sha-tg-e>` TG-E — FREEZE + arch-gate contract surface + DIRETRIZ §3.9 + SKILL_Stack reescrita + ADR fechado.

Vide [[project-tool-isolation-tclose-2026-05-22]] para o estado pré-TG-A. Histórico completo de execução está em `docs/architecture/decisions/0040-tool-as-isolated-feature-crate.md` §7.

**Stutter de mouse VSync** ([[project-stutter-text-shaping-2026-05-20]]) segue sendo trade-off documentado em `docs/perf/mouse-stutter.md` — não regredido por ADR-0040, não user-visible em fluxos do dia-a-dia. Não confundir com regressão.

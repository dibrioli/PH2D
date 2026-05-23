# Handoff — fechamento do ADR-0040 (canal genérico + FREEZE)

**Data:** 2026-05-22
**Para:** a LLM que vai fechar os ~30% restantes do ADR-0040 (canal de ação genérico de UI-edit + cleanup + FREEZE).
**Pré-leitura obrigatória:**
- [`docs/architecture/decisions/0040-tool-as-isolated-feature-crate.md`](architecture/decisions/0040-tool-as-isolated-feature-crate.md) (o contrato).
- [`docs/plans/2026-05-tool-isolation-waves.md`](plans/2026-05-tool-isolation-waves.md) (tabela de status).
- [`docs/perf/mouse-stutter.md`](perf/mouse-stutter.md) (NÃO regressar — o stutter VSync é trade-off escolhido pelo Enio, NÃO bug).
- Este handoff.

---

## 0. Mandato (lê antes de qualquer código)

> Padrão-ouro. Sem economias, sem gambiarras. Sem medo.

Cada fase: implementação → auto-verifica verde (`cargo test -p <crate>` + `cargo clippy --all-targets -- -D warnings` + `cargo fmt -- --check`) → auditoria adversarial (≥2 agentes em paralelo, lentes distintas, instruídos a serem DUROS) → corrige TODOS os achados Crít/Alto/Médio (Baixos doc-only podem virar follow-up registrado) → re-audita até erro-zero → commit local (`--no-verify`, em background, **NUNCA `--amend`, NUNCA push** — ship é do Enio).

**Smoke do Enio entre fases** quando interativo. Acumular commits locais; ele dispara `./scripts/ship.sh` + push no fim.

---

## 1. Estado atual (HEAD = `cf79e5a`, 14 commits locais à frente de origin/main)

**ADR-0040 estruturalmente fechado.** Tudo o que segue está commitado + auditado:

- Contrato `Tool` + `ImageEditTool` + `Tool::is_default` em editor-core.
- `ToolRegistry::register` é **pure-push** (não auto-ativa). Boot tool via `tools.activate_default()` (data-driven, `Tool::is_default = true` no Brush).
- **TODOS os 10 tools em crates `ph2d-tool-*`** isolados (bgremoval, brush, grid-snap, make-square, move, padding, real-size, trim-transparency + registry + registry-init).
- `ph2d-tool-sync` (codegen) gera **3 regiões marcadas** em `ph2d-tool-registry-init`: `register_all` (manifests), `register_all_tools` (Box<dyn Tool>), Cargo deps. Staleness gate cobre as 3.
- Gate `architecture_cycle_prevention::editor_core_has_no_concrete_tool_deps` ativo.
- `shells/desktop/src/init.rs`: registro manual MORTO; substituído por `ph2d_tool_registry_init::register_all_tools(&mut tools) + tools.activate_default()`.
- **TG-A done:** action bus tem `EditorAction::ActivateTool { tool_id }` + `OneShotImageOp { tool_id, entity_bits }` genéricos. Eliminou 7 dos 11 variants per-tool (`Trim`/`MakeSquare`/`RealSize`/`ActivateBgRemoval`/`ActivatePadding`/`Bgremoval{}`/`Padding{}` dead-code).
- Workspace 183 binários de teste verdes. Smoke do Enio confirmou 4 image tools + palette. Stutter VSync = trade-off documentado, não regressão.

**Restante (este handoff):** os 4 variants per-tool `BgremovalUiEdit`, `BgremovalCancel`, `PaddingUiEdit`, `PaddingCancel` + os 512 LOC de vocab em `editor-core/src/tools/{bgremoval,padding}/params.rs` + FREEZE.

---

## 2. O trabalho — fases TG-B / TG-C / TG-D / TG-E

Forma: incremental e behavior-preserving. Cada fase verifica + commita. Smoke entre fases recomendado (toca o caminho interativo do bgremoval/padding).

### TG-B — bgremoval: vocab migra + canal `ToolPanelEvent`

**Por que:** o vocabulário (`BgRemovalUiEdit`/`UiSnapshot`/`BrushFalloff`/`BgRemovalParams` + consts) é referenciado pelo `action_bus.rs` (variant `BgremovalUiEdit(...)`) e pelo `ph2d-panel-bgremoval`. Pra movê-lo pro crate, o action bus precisa parar de carregá-lo — o que exige um canal genérico (`ToolPanelEvent(PanelEvent)`) que rotea PanelEvent → `Tool::handle_panel_event`. A semantic mapping (slider id → `BgRemovalUiEdit::Tolerance(v)`) que hoje vive em [`crates/ph2d-panel-bgremoval/src/event.rs`](../crates/ph2d-panel-bgremoval/src/event.rs) migra pra `BgRemovalTool::handle_panel_event` (a tool já tem o método; só estende com arms novos para os NodeIds do panel docado).

**Ordem importa** (cada substep deixa o build verde):

#### TG-B.1 — Adicionar os 2 novos variants ao `EditorAction`
Em [`crates/ph2d-editor-core/src/action_bus.rs`](../crates/ph2d-editor-core/src/action_bus.rs), adicionar **junto com** os antigos (coexistem):

```rust
/// ADR-0040 TG-B. Panel converte WidgetEvent → PanelEvent (sem semantic
/// mapping) e empurra aqui; shell drena chamando
/// tools.active_mut().map(|t| t.handle_panel_event(ev)). A semantic
/// mapping vive no tool (handle_panel_event matches NodeIds + calls
/// apply_ui_edit), não no panel.
ToolPanelEvent(crate::tool::PanelEvent),

/// Generic cancel: shell drena chamando tools.activate_default().
CancelActiveTool,
```

(Os antigos `BgremovalUiEdit`/`Cancel`/`PaddingUiEdit`/`Cancel` ficam até suas migrações; remoção em TG-B.7 / TG-C.7.)

#### TG-B.2 — Estender `BgRemovalTool::handle_panel_event` com os NodeIds do panel docado
Em [`crates/ph2d-tool-bgremoval/src/tool.rs`](../crates/ph2d-tool-bgremoval/src/tool.rs) (~linha 860), o `handle_panel_event` atual cobre só os NodeIds da `FloatingPanel` (TOLERANCE_NODE, etc., locais do tool). Adicionar arms para os `BGR_*` NodeIds do panel docado (importar de `ph2d_editor_core::ids`):

```rust
// Sliders / number chips do panel docado → apply_ui_edit
PanelEvent::SetValue(id, v)
    if id == ph2d_editor_core::ids::BGR_TOLERANCE
        || id == ph2d_editor_core::ids::BGR_TOLERANCE_NUM =>
{
    self.apply_ui_edit(BgRemovalUiEdit::Tolerance(v as f32));
}
// ... espelhar p/ Feather, Refine, Grow, BrushSize
// Clicks (falloff, apply, eyedropper, protect, clear-protect, show-mask) →
// apply_ui_edit(BgRemovalUiEdit::X)
// Espelhar EXATAMENTE o mapping atual de panel-bgremoval/event.rs (linhas 27-186).
```

**Cuidado:** `apply_ui_edit` já existe e tem a lógica semântica (clamps, projeções full-scale). NÃO duplicar — só mapear PanelEvent → BgRemovalUiEdit → chamar.

#### TG-B.3 — Adicionar `BgRemovalTool::take_params_dirty` (substitui o `!ui_edits.is_empty()` do preview)
Em [`crates/ph2d-tool-bgremoval/src/tool.rs`](../crates/ph2d-tool-bgremoval/src/tool.rs):

```rust
// Campo novo na struct:
params_dirty: bool,

// Em apply_ui_edit, no final (depois de qualquer mutação de params):
self.params_dirty = true;

// Método novo (mirror de take_pending_apply):
pub fn take_params_dirty(&mut self) -> bool {
    std::mem::take(&mut self.params_dirty)
}
```

#### TG-B.4 — Shell: drain ToolPanelEvent/CancelActiveTool + parar de coletar bgremoval_ui_edits/cancel
Em [`shells/desktop/src/render_loop/mod.rs`](../shells/desktop/src/render_loop/mod.rs) (~linha 354-369):

- Remover `bgremoval_ui_edits` + `bgremoval_cancel` da collection (substituídos pelo roteamento direto).
- Remover arms `EditorAction::BgremovalUiEdit(edit) => ...` e `EditorAction::BgremovalCancel => ...`.
- Adicionar:
  ```rust
  EditorAction::ToolPanelEvent(ev) => {
      if let Some(t) = tools.active_mut() {
          t.handle_panel_event(ev);
      }
  }
  EditorAction::CancelActiveTool => {
      // Mesma lógica que o atual bgremoval_cancel fazia:
      // tools.activate_default() + limpar last_bgremoval_pushed_entity +
      // self.bgremoval_preview = None + self.title_dirty = true.
      if let Some(default_id) = tools.default_tool_id()
          && tools.set_active(&default_id)
      {
          self.last_bgremoval_pushed_entity = None;
          self.bgremoval_preview = None;
          self.title_dirty = true;
      }
  }
  ```
- Linhas 491-498 (`if bgremoval_cancel ...`): remover o bloco (a lógica migrou pra `CancelActiveTool` arm acima).
- O `padding_cancel` segue o mesmo destino em TG-C.

#### TG-B.5 — Shell: bgremoval_preview lê `take_params_dirty` em vez de `bgremoval_ui_edits`
Em [`shells/desktop/src/render_loop/bgremoval_preview.rs`](../shells/desktop/src/render_loop/bgremoval_preview.rs):

- Remover o parâmetro `bgremoval_ui_edits` (linha 47) — o caller (`render_loop/mod.rs:586`) também para de passar.
- Linha 88: `let params_changed = !bgremoval_ui_edits.is_empty();` → `let params_changed = bg.take_params_dirty();` (precisa do `bg` na escala — pode ser `tool.as_any_mut().downcast_mut::<BgRemovalTool>().map(|bg| bg.take_params_dirty()).unwrap_or(false)`).
- Linhas 102-103: remover o loop `for edit in bgremoval_ui_edits.drain(..) { bg.apply_ui_edit(edit); }` — os edits já foram aplicados pelo `handle_panel_event` que TG-B.4 dispara antes.

#### TG-B.6 — Panel-bgremoval: pushea ToolPanelEvent/CancelActiveTool em vez de variants tipados
Em [`crates/ph2d-panel-bgremoval/src/event.rs`](../crates/ph2d-panel-bgremoval/src/event.rs), reescrever (rascunho já desenhado neste handoff §6):

- WidgetEvent::ValueChanged(id) para sliders/number chips → ler valor de host.store() → push `EditorAction::ToolPanelEvent(PanelEvent::SetValue(id, value as f64))`.
- WidgetEvent::Click(id) para botões/falloff → reset button state + push `EditorAction::ToolPanelEvent(PanelEvent::Click(id))`.
- WidgetEvent::Click(BGR_CANCEL) → push `EditorAction::CancelActiveTool`.
- Remover imports de `BgRemovalUiEdit`/`BrushFalloff` (não precisa mais do mapping semântico — moveu pra tool).
- Remover `use ph2d_editor_core::tools::bgremoval` por `use ph2d_editor_core::tool::PanelEvent`.

#### TG-B.7 — Remover `BgremovalUiEdit` + `BgremovalCancel` da `EditorAction`
Agora seguro (zero produtores, zero drains).

#### TG-B.8 — Mover `params.rs` pro crate
- `git mv crates/ph2d-editor-core/src/tools/bgremoval/params.rs crates/ph2d-tool-bgremoval/src/params.rs`.
- `git rm crates/ph2d-editor-core/src/tools/bgremoval/mod.rs` (vazio agora).
- Em [`crates/ph2d-tool-bgremoval/src/lib.rs`](../crates/ph2d-tool-bgremoval/src/lib.rs): trocar `pub use ph2d_editor_core::tools::bgremoval::params;` por `pub mod params;` + `pub use params::{BgRemovalParams, BgRemovalUiEdit, BgRemovalUiSnapshot, BrushFalloff, ChromaParams, GuidedFilterParams, MAX_EXTRA_BG_COLORS};`.
- Em [`crates/ph2d-panel-bgremoval/Cargo.toml`](../crates/ph2d-panel-bgremoval/Cargo.toml): adicionar `ph2d-tool-bgremoval = { path = "../ph2d-tool-bgremoval" }`.
- Em [`crates/ph2d-panel-bgremoval/src/{state,paint,populate}.rs`](../crates/ph2d-panel-bgremoval/src/): trocar imports `ph2d_editor_core::tools::bgremoval::{BgRemovalUiSnapshot, BrushFalloff, ...}` por `ph2d_tool_bgremoval::params::{...}` (ou `ph2d_tool_bgremoval::{...}` se preferir o re-export do lib).
- Verificar: gate `editor_core_has_no_concrete_tool_deps` continua verde (editor-core não ganhou dep no crate).

#### TG-B — Validação
- `cargo test -p ph2d-tool-bgremoval -p ph2d-editor-core -p ph2d-panel-bgremoval -p ph2d-host-desktop`.
- `cargo clippy --all-targets -- -D warnings` nos mesmos crates.
- **Smoke do Enio**: bg removal interativo idêntico — sliders movem params, eyedropper amostra cor, protect-brush pinta, Apply baqueia, Cancel volta pra Brush.

### TG-C — padding: espelho exato de TG-B

Mesma estrutura, símbolos espelhados:
- `PaddingTool::handle_panel_event` ganha arms para `PAD_*` NodeIds (de `ph2d_editor_core::ids`) → chama `apply_ui_edit(PaddingUiEdit::Top(px))` etc.
- `PaddingTool::take_params_dirty()`.
- Shell drain: ToolPanelEvent já cobre (mesmo arm); CancelActiveTool já cobre. Só remover `padding_ui_edits`/`padding_cancel` da collection + arms antigos.
- Padding bridge se ajusta a `take_params_dirty` se relevante (provavelmente já lê do tool direto).
- Panel-padding: pushea genérico.
- Remover `PaddingUiEdit` + `PaddingCancel` da EditorAction.
- Mover `crates/ph2d-editor-core/src/tools/padding/params.rs` → `crates/ph2d-tool-padding/src/params.rs`.
- Atualizar imports nos consumidores.

Smoke do Enio: padding panel interativo idêntico.

### TG-D — `editor-core/src/tools/` deletado

Após TG-B + TG-C: `editor-core/src/tools/` só tem `mod.rs` vazio (ou só comentário). Deletar:
- `crates/ph2d-editor-core/src/tools/mod.rs`.
- Remover `pub mod tools;` de `crates/ph2d-editor-core/src/lib.rs`.
- Atualizar o doc do lib.rs (linha 20 menciona `tools`).
- `cargo test --workspace --exclude ph2d-asset` verde.

### TG-E — FREEZE + DIRETRIZ + final report

1. **Arch-gate da superfície congelada.** Mirror de `architecture_contract_surface.rs` dos nós. Adicionar `crates/ph2d-editor-core/tests/architecture_tool_contract_surface.rs` que conta métodos públicos de `Tool` (cap 8 atualmente — id/label/icon_slug/build_panel/on_activate/on_deactivate/handle_panel_event/as_any_mut/as_image_edit_mut/is_default = 10 itens; ajustar cap) e `ImageEditTool` (4: set_source/preview/take_pending_commit/run_full). Marcadores `🔒` no `tool.rs` doc-comment.
2. **DIRETRIZ.md ganha balde "tool fan-out"** (irmão do §3.8 de nós) — briefing pronto-pra-colar, garantia sem-colisão (codegen scan + glob members), checklist do revisor. Atualizar §3.1 "Tool nova" pra deletar o forward-note de migração (TG-* concluído) e apontar pro novo balde.
3. **SKILL_Stack §"Adicionar uma tool"** reescrito pro fluxo codegen: largar crate (manifest +/ou make()) → `cargo run -p ph2d-tool-sync` → fim. Sem passos manuais.
4. **ADR-0040 status:** trocar "Accepted (implementação pendente)" por "Accepted (implementado, FREEZE ratificado)". Adicionar §"Histórico de execução" no fim listando commits.
5. **Memória:** salvar `project_tool_isolation_freeze_2026_05_XX.md` (data do fechamento).
6. **Relatório final pro Enio:** o que fechou, métricas (LOC saída de editor-core, contagem de variants antes/depois, commits, gates ativos), ship checklist.

---

## 3. Riscos vivos

| Risco | Mitigação |
|---|---|
| Mapping semântico em `BgRemovalTool::handle_panel_event` perde uma slider/click | Auditar 1:1 contra `event.rs` atual (linhas 27-186); cobrir cada `if id == ...` lá com arm equivalente aqui |
| `take_params_dirty` não dispara em algum caminho que mudava params (ex.: `add_extra_color`, `paint_protect_at_uv`, `clear_protect_mask`) | Setar `params_dirty = true` em TODOS os mutators que mudam state que o preview reflete, não só em `apply_ui_edit` |
| Panel-bgremoval depende do crate (panel→tool, nova direção) | Confirmar gate `architecture_cycle_prevention` permite (gate só proíbe editor-core→panel-* e panel-*→shim; panel-*→tool-* não é listado, então OK; documentar) |
| Stutter de mouse volta (smoke) | NÃO é regressão TG-B/C esperar — `docs/perf/mouse-stutter.md` é o playbook; toggle Config→Display→Immediate primeiro |
| Cancel em algum tool tinha lógica específica perdida no genérico | TG-B.4 codifica o "cancel = activate_default + limpar preview + dirty title"; padding tem variante similar — confirmar antes de remover |

---

## 4. Acceptance (definição de "feito")

- [ ] EditorAction tem **zero variants per-tool** (só `ActivateTool`, `OneShotImageOp`, `ToolPanelEvent`, `CancelActiveTool`, e os não-tool).
- [ ] `editor-core/src/tools/` **não existe** (módulo deletado, `pub mod tools` removido do lib.rs).
- [ ] `architecture_tool_contract_surface` arch-gate ativo, capas apertadas, 🔒 nos lib.rs.
- [ ] DIRETRIZ tem balde "tool fan-out" + SKILL_Stack reescrito.
- [ ] ADR-0040 status: "implementado, FREEZE ratificado".
- [ ] Workspace `cargo test --workspace --exclude ph2d-asset` verde.
- [ ] Smoke do Enio: bg removal + padding + image tools + palette idênticos.
- [ ] Ship: `./scripts/ship.sh` verde → push origin main → babysit CI verde.

---

## 5. Files map (touch points)

**editor-core (perde):**
- `src/tools/bgremoval/` → deletado em TG-B.8
- `src/tools/padding/` → deletado em TG-C
- `src/tools/mod.rs` → deletado em TG-D
- `src/lib.rs` linha `pub mod tools;` → removido em TG-D
- `src/action_bus.rs` → 4 variants per-tool removidos em TG-B.7/TG-C.7; 2 genéricos adicionados em TG-B.1

**editor-core (ganha):**
- `tests/architecture_tool_contract_surface.rs` → criado em TG-E
- `src/tool.rs` → 🔒 markers em TG-E

**ph2d-tool-bgremoval (ganha):**
- `src/params.rs` (movido de editor-core em TG-B.8)
- `src/tool.rs`: handle_panel_event estendido + take_params_dirty (TG-B.2/B.3)

**ph2d-tool-padding (ganha):**
- `src/params.rs` (movido em TG-C)
- `src/tool.rs`: idem TG-B mas para padding

**ph2d-panel-bgremoval (muda):**
- `Cargo.toml` ganha dep `ph2d-tool-bgremoval` (TG-B.8)
- `src/event.rs` reescrito (TG-B.6) — passa de mapping semântico pra thin WidgetEvent→PanelEvent
- `src/{state,paint,populate,ids}.rs` imports trocam `ph2d_editor_core::tools::bgremoval` → `ph2d_tool_bgremoval` (TG-B.8)

**ph2d-panel-padding (muda):** espelho de panel-bgremoval em TG-C.

**shells/desktop (muda):**
- `src/render_loop/mod.rs`: drains ToolPanelEvent/CancelActiveTool adicionados, antigos removidos (TG-B.4/B.7)
- `src/render_loop/bgremoval_preview.rs`: usa take_params_dirty (TG-B.5)
- `src/render_loop/padding_bridge.rs`: espelho de B.5 para padding (TG-C)

**docs (muda):**
- `docs/IntegracaoMultiAgente/DIRETRIZ.md`: novo balde §3.X "tool fan-out" (TG-E)
- `SKILL_Stack_PH2D_Definitiva.md`: §"Adicionar uma tool" reescrita (TG-E)
- `docs/architecture/decisions/0040-tool-as-isolated-feature-crate.md`: status atualizado (TG-E)
- `docs/plans/2026-05-tool-isolation-waves.md`: tabela de status finalizada (TG-E)

---

## 6. Apêndice — esboço do novo `panel-bgremoval/src/event.rs` (TG-B.6)

```rust
//! Background-Removal panel apply_event — converts WidgetEvents into
//! generic PanelEvents and forwards via EditorAction::ToolPanelEvent
//! (ADR-0040 TG-B). Semantic mapping lives in BgRemovalTool::handle_panel_event.

use crate::ids;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal};
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::widget::ButtonState;

use crate::state::BgRemovalPanelState;

pub(crate) fn apply_event(
    _state: &mut BgRemovalPanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    EventOutcome::from_bool(apply_event_impl(host, ev))
}

fn apply_event_impl(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    if let WidgetEvent::ValueChanged(id) = ev && is_bgr_slider(id) {
        let value = host.store().slider(id).map(|(_, v)| v).unwrap_or(0.0);
        host.bus_mut().push(EditorAction::ToolPanelEvent(
            PanelEvent::SetValue(id, value as f64),
        ));
        return true;
    }
    if let WidgetEvent::ValueChanged(id) = ev && is_bgr_number(id) {
        let value = host.store().number_value(id).unwrap_or(0.0);
        host.bus_mut().push(EditorAction::ToolPanelEvent(
            PanelEvent::SetValue(id, value),
        ));
        return true;
    }
    if let WidgetEvent::Click(id) = ev && id == ids::BGR_CANCEL {
        reset_button(host, id);
        host.bus_mut().push(EditorAction::CancelActiveTool);
        return true;
    }
    if let WidgetEvent::Click(id) = ev && is_bgr_click(id) {
        reset_button(host, id);
        host.bus_mut()
            .push(EditorAction::ToolPanelEvent(PanelEvent::Click(id)));
        return true;
    }
    false
}

fn is_bgr_slider(id: ph2d_a11y::NodeId) -> bool { /* BGR_TOLERANCE / FEATHER / REFINE / GROW / BRUSH_SIZE */ }
fn is_bgr_number(id: ph2d_a11y::NodeId) -> bool { /* *_NUM versions */ }
fn is_bgr_click(id: ph2d_a11y::NodeId) -> bool { /* FALLOFF_* / SHOW_MASK / APPLY / EYEDROPPER / PROTECT / PROTECT_CLEAR */ }
fn reset_button(host: &mut dyn PanelHostInternal, id: ph2d_a11y::NodeId) {
    if let Some(InteractiveState::Button { state }) = host.store_mut().get_mut(id) {
        *state = ButtonState::Normal;
    }
}
```

---

## 7. Estimativa de esforço (calibrada com TG-A e T-close)

- TG-B: ~3-5h focado (12-15 file edits + 1 audit + smoke).
- TG-C: ~2h (mecânico, espelho de TG-B).
- TG-D: ~30min (deleção + verifica).
- TG-E: ~2h (arch-gate + DIRETRIZ + SKILL + ADR + relatório).
- **Total ~7-10h** numa sessão dedicada.

Razão sugerida: TG-B + smoke; TG-C + smoke; TG-D + TG-E + smoke final; ship.

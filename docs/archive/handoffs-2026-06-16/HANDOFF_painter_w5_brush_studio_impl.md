# HANDOFF — Painter W5 Brush Studio (impl)

**Status:** scaffold planejado, motor pronto. Fecha a W5 (última peça pós pigmento + grão).
**Diretriz de UI do Enio (2026-06-06):** a **fonte da verdade da UI é o `ph2d-panel-widget-gallery`
(canon dos widgets) + o `ph2d-panel-inspector` (padrão de painel-com-seções)**. NÃO improvisar
chrome/controles — espelhar esses dois.

## Objetivo (smoke W5)
"Abre Brush Studio, edita Round Hard mudando spacing/jitter, vê live preview; salva como Round
Soft; troca grão/scale/depth." 3 seções: **Stroke Path / Shape / Rendering** + live preview.

## Template = Inspector (estudado)
`ph2d-panel-inspector` é o molde exato (painel com seções de params, scroll, row-builders):
- `sections/*.rs` — uma seção por arquivo; helpers `check_row`, `number_row`, slider rows.
- `paint.rs` — sequencia seções com `paint_section_separator` + `push_section_top_y` + scroll
  (`store.panel_scroll(INSP_PANEL)`), macro `live_section!`.
- `state.rs` — thread_local snapshot + `set_current_*_snapshot` publicado pelo shell.
- Widgets canônicos vêm de `ph2d-editor-core/src/widget/*` (Checkbox/Slider/NumberInput/Dropdown),
  showcased no `ph2d-panel-widget-gallery`.

## Arquitetura decidida
- **Painel separado** `ph2d-panel-brush-studio` (NÃO empilhar no sidebar: `PainterUiSnapshot` está
  no **cap de 18 campos** — gate `painter_ui_snapshot_field_count_is_capped`). O Brush Studio tem
  **snapshot próprio rico** (`BrushStudioSnapshot` em `ph2d-tool-painter`, sem cap) com todos os
  params de `Brush` (stroke_path/shape/rendering/grain/dynamics…).
- **Geometria:** reusa o slot do right-dock da sidebar (`ctx.layout.painter_sidebar`) — ocupa o
  MESMO espaço quando aberto (sidebar escondida). NODE_ID já criado:
  `PAINTER_BRUSH_STUDIO_PANEL` (chrome.rs). **Não precisa de slot de layout novo.**
- **Abrir/fechar:** flag `show_brush_studio: bool` no `PainterTool`; `PainterUiEdit::OpenBrushStudio`
  (variante já existe, hoje no-op) flipa. Botão "Brush Studio" no sidebar abre; X no painel fecha.
- **Workaround do cap p/ sliders:** valores store-driven (sem campo no snapshot) — **provado** com
  o slider de Grain Depth (`f11b54b`). Cada param: `PainterUiEdit::Set*` + handler em `lifecycle.rs`
  + rota `SetValue` em `trait_impls.rs::handle_panel_event` + slider id em `event.rs`.

## Passos do scaffold (ordem)
1. **chrome.rs:** NODE_ID `PAINTER_BRUSH_STUDIO_PANEL` — ✅ FEITO. + ids dos widgets das seções.
2. **Crate `crates/ph2d-panel-brush-studio/`** (workspace é glob `crates/*`, auto-membro):
   Cargo.toml (deps: editor-core, tool-painter, a11y, tokens — espelhar sidebar) + `lib.rs`
   (`impl Panel`, ID `"painter_brush_studio"`, NODE_ID acima) + `state.rs` (snapshot static) +
   `paint.rs` (chrome surface+title "Brush Studio" + seções estilo inspector + scroll) +
   `populate.rs` + `event.rs`.
3. **`cargo run -p ph2d-panel-sync`** → regenera os markers do `ph2d-panel-registry-init`
   (deps + features `panel-brush-studio` + push block). Gate de staleness em
   `ph2d-panel-registry-init/tests/staleness.rs`.
4. **`shells/desktop/Cargo.toml`** (hand-edit, host não é synced): `dep:ph2d-panel-brush-studio` +
   feature `panel-brush-studio = ["ph2d-panel-registry-init/panel-brush-studio", "dep:…"]` + add à
   lista de features default de painéis (≈ linha 270).
5. **`ph2d-tool-painter`:** `BrushStudioSnapshot` + `brush_studio_snapshot()` + `show_brush_studio`
   + handler `OpenBrushStudio` (flip) + `PainterUiEdit::Set{Spacing,ShapeCount,Scatter,Rotation,
   GrainScale,…}` + handlers (escrevem em `brush.*`, `cached_brush_hash=None`).
6. **`painter_bridge.rs`:** `panel_visibility["painter_brush_studio"] = active && show_brush_studio`;
   sidebar visível só quando `!show_brush_studio && !shows_layers`; publicar o snapshot via
   `ph2d_panel_brush_studio::set_current_brush_studio_snapshot(...)`. Bump z-order (igual layers).
7. **Sidebar:** botão "Brush Studio" → `OpenBrushStudio`.

## Seções (params de `Brush` a expor — mirror inspector rows)
- **Stroke Path:** spacing, jitter, streamline (StrokePathParams/StabilizationParams).
- **Shape:** shape_count, shape_scatter, shape_rotation_follow, flip_x/y (ShapeParams).
- **Rendering:** pigment (checkbox), accumulate (checkbox), grain (dropdown 4 tipos +
  scale + depth sliders), rendering_mode (dropdown 6). Hoje os toggles + grain depth já vivem no
  sidebar — migrar/espelhar aqui.

## Live preview
Um stroke preview re-renderizado via `cpu_render` num buffer pequeno quando params mudam (reusa
`apply_stamps`). Ou, MVP: o canvas é o preview (pinta e vê). Decidir no impl.

## Motor pronto (esta sessão)
Pigmento 7-curvas (`1970740`) + grão 4-tipos CPU+WGSL (`de43c00`/`5c6992d`) + UI: checkboxes
Pigment/Accumulate (`4edfc9a` empilhados) + cycler de grão (`4ac54cc`) + slider Grain Depth
(`f11b54b`). O Brush Studio só precisa **expor** os params restantes seguindo o padrão acima.

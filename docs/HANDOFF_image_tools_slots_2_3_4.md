# HANDOFF — Image Tools slots 2/3/4 (shell wiring + Widget Gallery migration)

**Status:** ABERTO 2026-05-23 · Coord pré-trabalho concluído · 1 sessão Implementador.
**Branch:** `main` (HEAD = `71505dd` na escrita deste handoff). 33 commits locais acima de `origin/main`.

## 0. TL;DR

Os 4 crates de Image Tools nasceram em paralelo
(`HANDOFF_image_tools_4.md`, run de 2026-05-23). **Slot 1 (Color
Equalization) está fechado end-to-end** e funcionou como sonda — descobriu
4 erros arquiteturais que viraram regras escritas. Agora restam **3 tools
pra linkar** seguindo o padrão fechado da slot 1, **sem repetir os bugs
que já queimaram**.

| Slot | Tool                  | Crate tool                              | Crate panel                              | Sabor    | Status                                       |
|------|-----------------------|------------------------------------------|------------------------------------------|----------|----------------------------------------------|
| 1    | Color Equalization    | `ph2d-tool-color-equalization` ✅       | `ph2d-panel-color-equalization` ✅      | (3)      | **FECHADO** (commits `b54c865`..`71505dd`)   |
| 2    | Equalize Sizes        | `ph2d-tool-equalize-sizes` ✅           | `ph2d-panel-equalize-sizes` ✅          | (3)      | **WIRING PENDENTE**                          |
| 3    | Rasterize             | `ph2d-tool-rasterize` ✅                | (sem panel — sabor 1)                    | (1)      | **WIRING PENDENTE** (mais simples dos 3)     |
| 4    | Upscale               | `ph2d-tool-upscale` ✅                  | `ph2d-panel-upscale` ✅                 | (3)      | **WIRING PENDENTE**                          |

Tool + panel crates JÁ existem com algoritmo + UI implementados. **O que
falta é o END-TO-END no shell** — botão na TopBar dispara a tool, painel
abre, sliders mexem o canvas em tempo real, Apply faz bake multi-sprite.

## 1. LEIA ANTES DE CODAR (não-negociável)

Cada item abaixo introduz uma regra/gate criada na slot 1. Pular qualquer
um vai virar bug conhecido (vide §11.4 UI_Bugs):

1. **[`docs/IntegracaoMultiAgente/DIRETRIZ.md §4.2`](IntegracaoMultiAgente/DIRETRIZ.md#42-widget-gallery-é-a-fonte-de-verdade--copie-não-reinvente)** — Widget Gallery é a fonte de verdade. Quatro regras numeradas, cada uma já queimou.
2. **[`docs/UI_Bugs/README.md §11`](UI_Bugs/README.md#11-slot-1--color-equalization-2026-05-23)** — Os 4 bugs da slot 1 (preview ausente, chip drag absoluto, phantom stepper, divergência do Gallery) com causa + fix + lição estrutural.
3. **Reference vivo:** [`crates/ph2d-editor-core/src/screens/hero/pre_populate.rs:212-231`](../crates/ph2d-editor-core/src/screens/hero/pre_populate.rs) (seed do Speed slider/chip do Widget Gallery — é EXATAMENTE esse setup que se copia).
4. **Template do shell:** rastreie tudo que a slot 1 mexeu com `git log --name-only b54c865..71505dd` e use como mapa. Os 4 fixes da slot 1 (commits `903d63c`, `7b5f7c1`, `2f58b73`, `3bf8806`) mostram em código o ANTES e o DEPOIS de cada erro.
5. **Arch-gate ativo:** [`architecture_panel_chip_pill_no_stepper`](../crates/ph2d-editor-core/tests/architecture_panel_chip_pill_no_stepper.rs) — se você mexer no populate de um painel e ele perder o `link_slider_number`/`mark_chip_no_stepper`, CI vermelho.

## 2. Padrão de wiring — fechado na slot 1

A slot 1 cobriu 7 superfícies. Cada tool sabor (3) repete as 7; sabor (1)
repete só 3 (sem panel, sem bridge).

### 2.1 Touchpoints sabor (3) — Equalize Sizes + Upscale

| #  | Arquivo                                                                                                   | O que entra                                                                                            | Reference slot 1                |
|----|-----------------------------------------------------------------------------------------------------------|--------------------------------------------------------------------------------------------------------|----------------------------------|
| 1  | [`crates/ph2d-editor-core/src/ids.rs`](../crates/ph2d-editor-core/src/ids.rs)                             | `pub const <SLUG>_PANEL: NodeId = hash_node_id("panel.<slug>")` + comentário cross-ref ao panel crate | `CEQ_PANEL` linha ~123          |
| 2  | [`crates/ph2d-editor-core/src/screens/hero.rs`](../crates/ph2d-editor-core/src/screens/hero.rs)            | Adicionar `ids::<SLUG>_PANEL` ao z_order fallback list (~linha 810)                                    | `ids::CEQ_PANEL`                |
| 3  | [`crates/ph2d-editor-core/src/screens/hero/topbar/mod.rs`](../crates/ph2d-editor-core/src/screens/hero/topbar/mod.rs) | Botão no topbar com tooltip + icon                                                                | grep `color_equalization`       |
| 4  | [`crates/ph2d-editor-core/src/screens/hero/chrome/image_actions.rs`](../crates/ph2d-editor-core/src/screens/hero/chrome/image_actions.rs) | `stateful_tool_for(<slug>)` mapping pro `image_actions` dispatcher              | grep `color_equalization`       |
| 5  | [`crates/ph2d-i18n/src/lib.rs`](../crates/ph2d-i18n/src/lib.rs)                                            | `tool.<slug>.label` + `.tooltip` strings (pt-BR + en-US)                                              | `tool.color_equalization.*`     |
| 6  | [`shells/desktop/Cargo.toml`](../shells/desktop/Cargo.toml)                                                | `ph2d-tool-<slug>` + `ph2d-panel-<slug>` deps + `panel-<slug>` feature                                | grep `color-equalization`       |
| 7  | [`shells/desktop/src/app_state.rs`](../shells/desktop/src/app_state.rs)                                    | `<Slug>Preview` struct + `<slug>_preview: Option<<Slug>Preview>` + `last_<slug>_pushed_entity: Option<u64>` fields | `ColorEqualizationPreview` |
| 8  | [`shells/desktop/src/main.rs`](../shells/desktop/src/main.rs)                                              | Init dos 2 novos fields em `App::new` (None)                                                          | grep `color_equalization`       |
| 9  | [`shells/desktop/src/render_loop/mod.rs`](../shells/desktop/src/render_loop/mod.rs)                       | `activate_<slug>` flag local + drain match arm em `EditorAction` + activate handler + bridge dispatch call + Apply teardown | grep `color_equalization` |
| 10 | [`shells/desktop/src/render_loop/<slug>_bridge.rs`](../shells/desktop/src/render_loop/color_equalization_bridge.rs) | **Novo arquivo.** Espelha `bgremoval_preview.rs` / `color_equalization_bridge.rs` 1:1. Refresh em `take_params_dirty()` + overlay via `draw_image_rgba`. | `color_equalization_bridge.rs` |
| 11 | [`shells/desktop/src/render_loop/image_edit.rs`](../shells/desktop/src/render_loop/image_edit.rs)         | `<slug>_apply: Option<Vec<u64>>` param + multi-sprite bake loop                                       | `color_equalization_apply`      |
| 12 | [`shells/desktop/src/hero_intents/image_edit.rs`](../shells/desktop/src/hero_intents/image_edit.rs)       | `drain_<slug>(...)` function (espelho de `drain_color_equalization`)                                  | `drain_color_equalization`      |
| 13 | [`shells/desktop/src/hero_intents/mod.rs`](../shells/desktop/src/hero_intents/mod.rs)                     | Re-export da drain function                                                                            | `drain_color_equalization`      |
| 14 | `crates/ph2d-panel-<slug>/src/populate.rs`                                                                | **MIGRAÇÃO §4.2** — chip storage em `0..1` (não unidade natural); `link_slider_number(slider, chip)` para cada pair (auto-marca `mark_chip_no_stepper`); buffer = `format_number(track)` | CEQ populate.rs `link_slider_number` loop |
| 15 | `crates/ph2d-panel-<slug>/src/event.rs`                                                                   | **SIMPLIFICAÇÃO §4.2** — virar forwarder thin (ler slider track, emitir `PanelEvent::SetValue(slider_id, track)`). DROPAR mirror manual.                          | CEQ event.rs `apply_event_impl` |
| 16 | `crates/ph2d-panel-<slug>/src/paint.rs`                                                                   | `display_override` agora pega o track (read from store.slider) + projeta natural unit via `params::slider_to_*`. Storage do chip não é mais consultado pra display. | CEQ paint.rs `chip_display`      |

### 2.2 Touchpoints sabor (1) — Rasterize

Bem mais simples — só 4 superfícies. Sem panel, sem bridge, sem preview
live. É um botão da TopBar que dispara `OneShotImageOp` na seleção.

| # | Arquivo                                                                                                   | O que entra                                                                                            | Reference                       |
|---|-----------------------------------------------------------------------------------------------------------|--------------------------------------------------------------------------------------------------------|----------------------------------|
| 1 | [`crates/ph2d-editor-core/src/screens/hero/topbar/mod.rs`](../crates/ph2d-editor-core/src/screens/hero/topbar/mod.rs) | Botão no topbar                                                                                | template: `trim_transparency`   |
| 2 | [`crates/ph2d-editor-core/src/screens/hero/chrome/image_actions.rs`](../crates/ph2d-editor-core/src/screens/hero/chrome/image_actions.rs) | Mapping `<slug>` → `OneShotImageOp` no `image_actions` palette                            | template: `trim_transparency`   |
| 3 | [`crates/ph2d-i18n/src/lib.rs`](../crates/ph2d-i18n/src/lib.rs)                                            | `tool.rasterize.label` + `.tooltip`                                                                   | template: `tool.trim_transparency.*` |
| 4 | [`shells/desktop/Cargo.toml`](../shells/desktop/Cargo.toml)                                                | `ph2d-tool-rasterize` dep                                                                              | template: `ph2d-tool-trim-transparency` |
| 5 | [`shells/desktop/src/render_loop/mod.rs`](../shells/desktop/src/render_loop/mod.rs)                       | Drain match arm em `EditorAction::OneShotImageOp` pra `rasterize` slug                                | grep `trim_transparency`        |
| 6 | [`shells/desktop/src/render_loop/image_edit.rs`](../shells/desktop/src/render_loop/image_edit.rs)         | Bake function nova                                                                                    | template: `trim_transparency`    |

Template canônico de sabor (1):
[`crates/ph2d-tool-trim-transparency/`](../crates/ph2d-tool-trim-transparency/)
+ grep `trim_transparency` no shell pra ver TODAS as superfícies.

## 3. Regras herdadas da slot 1 — REPITA EXATO (não invente)

### 3.1 Panel populate (sabor 3): COPY do Speed slider do Widget Gallery

A slot 1 tentou natural-unit storage no chip (clip_limit em 1..4 etc.). Queimou:
mirror manual em event.rs ficou descalado, clamp não engatou, phantom stepper.
**Solução canônica** (commit `3bf8806`):

```rust
// populate.rs — para cada slider+chip pair:
let track = <slug>_params::<NAME>_to_slider(<NAME>_DEFAULT);
store.register(slider_id, InteractiveState::Slider {
    state: SliderState::Normal,
    value: track,
    orientation: SliderOrientation::Horizontal,
});
store.register(chip_id, InteractiveState::NumberInput {
    state: TextInputState::Normal,
    value: track as f64,                       // ← 0..1, NÃO natural unit
    buffer: format_number(track as f64),       // ← formatado a partir de track
    caret: 0,
    last_committed: track as f64,
    selection_anchor: None,
});
store.link_slider_number(slider_id, chip_id);
// (link_slider_number AUTO-CHAMA mark_chip_no_stepper — não precisa repetir)
```

> **Atenção (Equalize Sizes):** os chips standalone (`EQS_FIXED_W`,
> `EQS_FIXED_H`) NÃO têm slider pareado. Para eles, mantenha
> `mark_chip_no_stepper` explícito (já está) e o storage pode ficar em
> pixels (natural unit). A regra `link_slider_number` só vale para
> pairs slider+chip. O chip `EQS_GRID_UNIT_NUM` (pareado com
> `EQS_GRID_UNIT` slider) PRECISA migrar pra o padrão Speed acima.

### 3.2 Panel event.rs: forwarder thin, NÃO mirror manual

Com `link_slider_number` no populate, o dispatch sincroniza slider↔chip
automaticamente. `event.rs` só precisa ouvir `ValueChanged(slider_id ou chip_id)`,
ler o slider track, emitir `PanelEvent::SetValue(slider_id, track)`:

```rust
WidgetEvent::ValueChanged(id) if slider_for_widget(id).is_some() => {
    let slider_id = slider_for_widget(id).unwrap();
    let track = host.store().slider(slider_id).map(|(_, v)| v).unwrap_or(0.0);
    host.bus_mut().push(EditorAction::ToolPanelEvent(
        PanelEvent::SetValue(slider_id, track as f64),
    ));
    true
}
```

Compare com `ph2d-panel-color-equalization/src/event.rs` linha-por-linha.

### 3.3 Panel paint.rs: `display_override` lê o slider, NÃO o chip

Chip value agora vive em 0..1; pra mostrar natural unit ("256 px",
"4× scale"), pegue o slider track e projete via `slider_to_<param>()`:

```rust
let track = store.slider(slider_id).map(|(_, v)| v).unwrap_or(default_track);
let display = format!("{:.1}", slider_to_scale(track));   // natural unit
paint_slider_with_chip_layout(
    rect, label, track,
    track as f64,         // chip_value = track (não consultado quando display_override is Some)
    Some(&display),       // display_override
    slider_id, chip_id,
    LABEL_COL_W, chip_w,
    store, hit_index, scene, text_system, theme,
);
```

### 3.4 Bridge: refresh em `take_params_dirty()`, overlay `draw_image_rgba`

**Espelhe** `shells/desktop/src/render_loop/color_equalization_bridge.rs`
(menos custoso) ou `bgremoval_preview.rs` (mais completo). 5 passos:

1. Active/inactive: liga panel visibility; se inactive, zera cache + return None.
2. Source push: quando `hero.gizmo.selection` mudar, lê textura via `texture_edit::read_sprite_source` e empurra pro tool via `set_source_snapshot` (downcast `as_any_mut`).
3. Snapshot publish: `set_current_snapshot(Some(tool.ui_snapshot()))` (feature-gated).
4. Preview refresh: quando `tool.take_params_dirty()`, chama `tool.preview_rgba()` e cacheia como `Arc<Vec<u8>>` em `<Slug>Preview { entity_bits, rgba, width, height }`.
5. Overlay paint: ao final, se cache existe, `vector_scene.draw_image_rgba(&rgba, w, h, world_bbox, quality)` sobre o footprint da sprite.

### 3.5 Chip drag = incremental delta (já está infra-wide)

Commit `7b5f7c1` migrou o dispatch de drag pra modelo incremental. **NÃO
precisa fazer nada** — a infra cobre toda chip. Mas se você ver código
seu computando `event.x - start_x` no Move handler, é regressão.

## 4. Estado vivo dos 3 painéis

### Equalize Sizes
- Tool crate: completo (`tool.rs`, `params.rs`, `algorithm.rs`).
- Panel crate: **migração §4.2 pendente.** Hoje:
  - `populate.rs` registra chips standalone OK (FIXED_W, FIXED_H — com `mark_chip_no_stepper` explícito já adicionado em `fd37ca8`).
  - `populate.rs` **NÃO** linka `EQS_GRID_UNIT` + `EQS_GRID_UNIT_NUM` — precisa `link_slider_number` + storage em 0..1.
  - `event.rs` tem mirror manual — precisa virar forwarder thin (só para o pair grid_unit).
- Shell wiring: **0% feito.**

### Rasterize
- Tool crate: completo (sabor 1 — só `MANIFEST` + `register` + algorithm).
- Sem panel (sabor 1 não tem).
- Shell wiring: **0% feito.** Use `trim_transparency` como template — é o sabor (1) mais simples.

### Upscale
- Tool crate: completo (`tool.rs`, `params.rs`, `algorithm.rs` com Lanczos3/Nearest/xBR).
- Panel crate: **migração §4.2 pendente.** Hoje:
  - `populate.rs` registra slider+chip mas **NÃO** chama `link_slider_number`. `mark_chip_no_stepper` foi adicionado em `fd37ca8` mas é band-aid.
  - `populate.rs` storage chip em `DEFAULT_SCALE_FACTOR as f64` (natural unit) — precisa migrar pra `scale_to_slider(...)`.
  - `event.rs` tem mirror manual — precisa virar forwarder thin.
- Shell wiring: **0% feito.**

## 5. Sequência sugerida

Faça os 3 em ordem de complexidade crescente — pega o ritmo no fácil
antes dos sabor (3) com migração de populate:

1. **Rasterize** (sabor 1, ~6 superfícies) — confirma que você sabe drenar `OneShotImageOp` multi-sprite. Smoke: clica botão, sprite vira pixel raster.
2. **Upscale** (sabor 3, 1 slider só) — migração §4.2 mínima (1 pair grid_unit equivalente). Smoke: arrasta slider de scale, canvas mostra preview em tempo real, Apply faz bake.
3. **Equalize Sizes** (sabor 3, slider + chips standalone + 3 modos exclusivos) — mais widgets, mais wiring. Mesma migração §4.2 (1 pair grid_unit), chips standalone ficam como estão.

## 6. Smoke + checklist final (Implementador → Coord)

Antes de declarar "fechei a wiring":

- [ ] `cargo run -p ph2d-tool-sync` rodado; `cargo test -p ph2d-tool-registry-init` verde (staleness).
- [ ] `cargo test -p ph2d-editor-core --test architecture_panel_chip_pill_no_stepper` verde (não retrocede gate).
- [ ] `cargo test -p ph2d-editor-core --test architecture_tool_contract_surface` verde (contrato congelado intocado).
- [ ] `cargo test -p ph2d-host-desktop` compila clean.
- [ ] Pra cada tool: smoke manual no app:
  - Botão da TopBar dispara o tool ✓
  - Painel abre (sabor 3) ou ação aplica direto (sabor 1) ✓
  - Slider mexe canvas EM TEMPO REAL (sabor 3) ✓
  - Chip drag scrub funciona, reversão reverte imediato ✓
  - Chip click no canto direito NÃO incrementa sozinho (regressão phantom stepper) ✓
  - Multi-seleção: Apply baka em TODAS as sprites selecionadas ✓
  - Cancel fecha painel sem alterar pixels ✓
- [ ] Relate ao Coord: lista de SHAs locais + comando smoke + resultado.

NÃO faça `git push`. Coord absorve o push + babysit CI ao final (vide
DIRETRIZ §7.3).

## 7. Anti-patterns conhecidos (NÃO repita)

Cada um destes JÁ queimou na slot 1 (vide UI_Bugs §11):

- ❌ Storage do chip em unidade natural quando há slider pareado.
      Sempre 0..1; natural unit só em paint via `display_override`.
- ❌ Mirror manual slider↔chip em `event.rs`. Use `link_slider_number`.
- ❌ Bridge sem `take_params_dirty()` refresh. Canvas fica congelado.
- ❌ Esquecer `<SLUG>_PANEL` no z_order fallback de `hero.rs`. Painel
      registra mas nunca pinta.
- ❌ Não ler `hero.gizmo.iter_selected()` no drain. Tool single-sprite-only.
- ❌ `dx_total = event.x - start_x` no chip drag (use `event.x - last_x`).
      A infra já está correta, mas se você duplicar código, vai recriar.
- ❌ Esquecer `mark_chip_no_stepper` para chip standalone (sem slider
      pareado). O `link_slider_number` auto-marca; sem link, marca à mão.

## 8. Onde olhar quando estiver perdido

- Como CEQ ficou no final: `git show 3bf8806` (panel canônico) + `git show 903d63c` (bridge canônico) + `git show 71505dd` (docs).
- Range completo da slot 1: `git log --oneline b54c865..71505dd -- shells/desktop/ crates/ph2d-panel-color-equalization/ crates/ph2d-editor-core/src/ids.rs crates/ph2d-i18n/`.
- Discussão das decisões: `docs/UI_Bugs/README.md §11` (PT-BR, com tabela de divergência → bug em §11.4).
- Receita estrutural: `docs/IntegracaoMultiAgente/DIRETRIZ.md §4.2`.

---

**Coord ao Implementador:** se travar em algo que o handoff não cobre,
PERGUNTE ao Enio antes de improvisar. A slot 1 mostrou que "improvisar"
custa 4 ciclos de smoke e 5 commits de correção pelo Coord. Bounce
qualquer dúvida sobre Widget Gallery / contrato.

**Coord ao Coord futuro:** ao fechar slot 2-4, registre em
[`memory/MEMORY.md`](../../.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/MEMORY.md)
um project memory novo `project_image_tools_slots_fechadas_<data>.md`
seguindo o template das memórias slot 1.

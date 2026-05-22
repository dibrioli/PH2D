# Trim Transparency — Plano de Integração

**Status:** Ilha isolada pronta — aguardando janela de integração.
**Agente Implementador:** worktree `agent-trim-transparency` / branch `feature/trim-transparency`.
**Audiência:** próximo agente Integrador.

## 1. O que esta ilha entrega (já implementado)

- Algoritmo puro `trim_transparency(rgba, width, height, alpha_threshold) -> TrimResult` em
  [`algorithm.rs`](algorithm.rs). Pure-Rust, sem deps externas, edge-scanning O(W+H) no caso comum.
- Ícone `crop_bezpath()` em [`icon.rs`](icon.rs) — port do Lucide `crop` (2 paths) para `kurbo::BezPath`,
  desenhado num espaço 24×24 (igual ao viewBox do SVG original) com `transform` aplicado pelo caller.
- Tipos públicos `TrimResult` e `Bounds` em [`mod.rs`](mod.rs).
- Testes em [`../../../tests/trim_transparency_algorithm.rs`](../../../tests/trim_transparency_algorithm.rs).

Esta ilha **não** toca:
- `tools/mod.rs` (re-exports — você adiciona).
- Qualquer arquivo do chrome do editor (top-bar, zonas, etc — você cria do zero).
- Asset / sprite data model (você decide como invocar `apply`).

## 2. UX-alvo (do mockup do Enio, 2026-05-12)

Top-bar evolui em 3 fases discretas. Hoje (estado atual em `main`) ela praticamente não existe — o demo só renderiza no título da janela. A integração introduz a barra inteira E o modo Image Tools.

**Fase A — Top-bar default ("Modo Edição de Cena"):**

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ [PH2D▼]  [💾 Save] [📂 Open] [🔲 ImageTools]  [Level_01▼]  [▶][⏸][⟲]  [🗂][🖼][📜]  [⚙ Config] │
└──────────────────────────────────────────────────────────────────────────────┘
                                  ▲
                             novo botão
```

- `[🔲 Image Tools]` é o botão novo, à direita de `[📂 Open]`.
- `[⚙ Config]` se move da posição atual (próximo a Open) para o **final da barra**, à direita
  dos painéis de visualização.
- Demais elementos ficam onde estão hoje (project picker à esquerda, Level dropdown ao
  centro, play/pause/reset, panel-toggles).

**Fase B — Modo Image Tools (após click em `[🔲 Image Tools]`):**

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ [PH2D▼]  [💾 Save] [📂 Open] [🔲 ImageTools*]  [✂ Trim]  [...future image actions...] │
└──────────────────────────────────────────────────────────────────────────────┘
                                       ▲                ▲
                                  destacado         primeira ação
                                  (ativo)           (este PR)
```

- Tudo à direita de `[🔲 Image Tools]` em modo default é **ocultado** (Hide), e a fileira
  de ações de imagem aparece no espaço liberado.
- `[✂ Trim]` é a primeira ação. Click → invoca `trim_transparency()` na seleção atual.
- Click novamente em `[🔲 Image Tools]` (ou ESC) volta pra Fase A.

**Fase C (futuro, fora deste PR):** outras ações de imagem (BG Removal, Equalize, Make
Square, Padding, Upscale, etc — vide engine legada como referência).

## 3. Modelo arquitetural sugerido para o Integrador

### 3.1 Top-bar como widget próprio

Hoje não existe `TopBar` no editor. Sugestão de criação (fora do escopo desta ilha):

- Novo módulo `crates/ph2d-editor/src/top_bar/`.
- Tipos:
  ```rust
  pub enum TopBarMode { Edit, ImageTools }
  pub struct TopBar { pub mode: TopBarMode, pub items: Vec<TopBarItem> }
  pub enum TopBarItem {
      ProjectPicker, Save, Open, ImageToolsToggle,
      LevelDropdown, PlayPauseReset, PanelToggles, Config,
      // image-mode-only:
      ImageAction { id: ImageActionId, icon: BezPath, label: String, enabled: bool },
  }
  ```
- Reflete `mode` em qual lista de itens é montada por frame. Filtragem central — não usar
  `.hide()` em widgets individuais.
- Cliques emitem eventos via mesmo `PanelEvent` ou um novo `TopBarEvent`. Reaproveita
  a paint pass do `widget::Button`.

### 3.2 Wiring de `Trim Transparency` à Image Tools row

Quando `TopBarMode::ImageTools` está ativo, o slot `[✂ Trim]` é um `TopBarItem::ImageAction`
com:

- `icon`: invocar [`icon::crop_bezpath()`](icon.rs) e aplicar `Affine::scale(chip_size / 24.0)`
  para renderizar dentro do chip (o BezPath está em coordenadas 24×24, igual ao viewBox
  do Lucide).
- `label`: chave i18n `tool.trim_transparency.label` (HR-15) — registrar em
  `crates/ph2d-editor/locales/*.ftl` quando a infra Fluent chegar; até lá, fallback "Trim".
- A11y: `AccessKit::Button` com label da chave acima, role `Role::Button`, action
  `Action::Default`.
- On click: chama o handler abaixo.

### 3.3 Handler do click

Pseudocódigo do que o Integrador implementa (não pertence a esta ilha — depende do
modelo de seleção/asset que ele decidir, ainda não existe em `ph2d-editor`):

```rust
fn on_trim_transparency_clicked(editor: &mut Editor) {
    let selection = editor.selected_sprites();  // a definir
    if selection.is_empty() { return; }
    for sprite in selection {
        let rgba = sprite.image_data();       // &[u8] RGBA8 cru
        let (w, h) = sprite.dimensions();
        let result = trim_transparency(rgba, w, h, /* alpha_threshold = */ 0);
        if !result.trimmed { continue; }

        // 1. Replace pixels.
        sprite.set_image(result.pixels, result.width, result.height);

        // 2. Reproject pivot to preserve world position.
        //    new_px = (old_w * old_px - bounds.x) / new_w,  clamped [0, 1]
        let new_pivot_x = ((sprite.w as f32 * sprite.pivot_x) - result.bounds.x as f32)
                          / result.width as f32;
        sprite.pivot_x = new_pivot_x.clamp(0.0, 1.0);
        // (same for Y)

        // 3. Emit events for render + history (HR equivalent: undo).
        editor.events.push(SpriteImageChanged { id: sprite.id });
        editor.events.push(TransformUpdated { id: sprite.id });
    }
    editor.history.push("Trim transparency");
    editor.request_redraw();
}
```

**Importante:** a reprojeção de pivô NÃO está no algoritmo. O algoritmo só devolve
`bounds`. A matemática de pivô depende do modelo `Sprite` que o Integrador definir.
A fórmula `(old_w * old_pivot - bounds.x) / new_w` está validada pelo port legacy
(vide `apps/editor/src/EditorToolDispatcher.ts:122-123` em `Referencias/Game-Engine-Legada`).

### 3.4 Por que NÃO impl `Tool` trait

`Trim Transparency` é **ação one-shot sem estado**, não tool stateful com painel. A trait
`Tool` exige `build_panel() -> FloatingPanel`, `on_activate`/`on_deactivate`, e
`handle_panel_event` — todos vazios pra esta ação. Implementar a trait inflaria a
superfície e geraria um painel-zumbi que nunca aparece (a ação dispara direto do botão
top-bar, sem ativar tool).

Esta ilha entrega a ação como módulo puro. A trait `Tool` continua válida pra ferramentas
stateful (Brush, Move, futuras Painter/Eraser/etc).

## 4. Threshold

Hardcoded `alpha_threshold = 0` (paridade exata com a engine legada).

Se no futuro o produto pedir "trim quase-transparente" (ex: limpar JPEG artifacts com
alpha residual), expor um slider via Settings panel ou tornar a Image Tools row sticky
com sub-controles. **Não está no escopo deste PR** e nem mostrado no mockup do Enio.

## 5. Checklist do Integrador

Quando montar a top-bar e o modo Image Tools:

- [ ] Add `pub mod trim_transparency;` em `crates/ph2d-editor/src/tools/mod.rs`.
- [ ] Re-export `TrimResult`, `Bounds`, `trim_transparency`, `crop_bezpath` no `lib.rs` (se
      pertinente para shells/desktop consumir).
- [ ] Top-bar widget novo, com 2 modos (Edit / ImageTools) e toggle controlado pelo botão
      `[🔲 Image Tools]`.
- [ ] Botão `[⚙ Config]` migrado pro fim da barra (já não está adjacente a `[📂 Open]`).
- [ ] `[✂ Trim]` adicionado à Image Tools row, renderizando `crop_bezpath()` com escala
      apropriada.
- [ ] Handler do click conforme §3.3.
- [ ] A11y nó AccessKit no botão Trim (HR-12).
- [ ] String "Trim" via `t!()` (HR-15) quando bundle Fluent existir.
- [ ] Undo: empilhar entry "Trim transparency" no history.
- [ ] Smoke test cobrindo o fluxo top-bar → click → asset modificado → render redrawed.

## 6. Caso de teste manual após integração

1. Carregar sprite PNG com bordas transparentes (ex: 256×256 com objeto centralizado 64×64).
2. Selecionar sprite no Hierarchy.
3. Click `[🔲 Image Tools]` → barra muda pra Fase B.
4. Click `[✂ Trim]` → sprite passa pra 64×64, pivô mantém posição visual no canvas, undo
   disponível com label "Trim transparency".
5. ESC ou re-click em `[🔲 Image Tools]` → volta pra Fase A.
6. Ctrl+Z → sprite restaurada pra 256×256, pivô original.

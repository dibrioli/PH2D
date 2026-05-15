# Make Square — Plano de Integração

**Status:** Ilha isolada pronta — aguardando integração pelo Coordenador.
**Slot:** #3 — slug `make-square` (commit `87c176d`).
**Audiência:** Coordenador (Agente Central).

## 1. O que esta ilha entrega (já implementado)

- Algoritmo puro `make_square(rgba, width, height) -> MakeSquareResult`
  em [`algorithm.rs`](algorithm.rs). Pure-Rust (`std`-only), O(W·H) com
  short-circuit zero-copy quando entrada já é quadrada.
- Ícone `square_bezpath()` em [`icon.rs`](icon.rs) — port do Lucide
  `square` (1 rounded-rect, MIT/ISC-compat) para `kurbo::BezPath` num
  design space 24×24 (mesmo viewBox do SVG original), com `Affine` e
  `Stroke` aplicados pelo caller.
- Tipo público `MakeSquareResult` em [`mod.rs`](mod.rs).
- Testes unitários inline em `algorithm.rs` e `icon.rs`, mais teste
  de integração via `#[path]` em
  [`../../../tests/make_square_algorithm.rs`](../../../tests/make_square_algorithm.rs).

Esta ilha **não** toca:
- `tools/mod.rs` (re-export — Coordenador adiciona).
- Qualquer arquivo do chrome do editor (TopBar, cluster Image Tools,
  `ids.rs`, `fixture.rs`).
- Asset / sprite data model (Coordenador decide como invocar `apply`
  conforme modelo de seleção que esteja vigente no editor).

## 2. UX-alvo (Enio, 2026-05-15)

Botão aparece dentro do **modo Image Tools**, lado a lado com o `Trim
Transparency` já planejado. Espera-se a evolução da Fase B do mockup
do Trim:

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ [PH2D▼] [💾 Save] [📂 Open] [🔲 ImageTools*]  [✂ Trim]  [▢ MakeSquare]  ... │
└──────────────────────────────────────────────────────────────────────────────┘
                                       ▲              ▲              ▲
                                  destacado       já wirado      este PR
                                  (ativo)         (trim)
```

- Click em `[▢ MakeSquare]` → invoca `make_square()` no(s) sprite(s)
  selecionado(s).
- Se sprite já é quadrado (`!result.made_square`), pular sem entry de
  undo (mesma política do Trim quando `!trimmed`).

## 3. Modelo arquitetural — handler do click

A trait `Tool` **não** se aplica (mesma justificativa de Trim — vide
§3.4 do INTEGRATION.md de `trim_transparency`: ação one-shot sem
estado, sem painel; implementar `Tool` infla a superfície e gera
painel-zumbi).

> **Atualização pós-integração (Coord, 2026-05-15):** a versão original
> deste §3 apresentava um pseudocódigo com `sprite.pivot_x/pivot_y` e
> `sprite.set_image()` — APIs **que não existem** no modelo
> `ph2d_render::Sprite` atual. O modelo real é **center-anchored**: a
> sprite é renderizada com seu centro na `Transform.translation`; não
> há campo de pivot explícito. O texto abaixo reflete o drainer
> efetivamente implementado em `shells/desktop/src/main.rs` (search:
> `pending_make_square`).

### Drainer real (host `shells/desktop/src/main.rs`)

```rust
// 1. Click handler em HeroScreen::apply_event raises pending_make_square
//    com o gizmo_selection atual (entity_bits).
// 2. Render loop drena pending_make_square uma vez por frame:

if let Some(entity_bits) = hero.pending_make_square.take() {
    // Snapshot do sprite + Transform.translation + texture_id antigo
    // (se source = Individual — usado pra release pós-swap, evita
    // texture leak no IndividualTextureStore).
    let snapshot = read_sprite_pixels(sim, entity, asset_db, renderer);

    let result = make_square(&pixels, width, height);
    if !result.made_square { /* toast info */ }
    else if result.size > renderer.max_texture_dimension_2d() {
        // M1: cap GPU pré-acquire para não cair em device-loss
        // silencioso no primeiro render. Toast preventivo.
    }
    else {
        let texture_id = renderer.acquire_individual(result.size, result.size, &result.pixels)?;

        // M2: sub-pixel recenter via image_edit::recenter_after_pad.
        // Para diff par, é no-op (offset = diff/2 → delta=0). Para diff
        // ímpar, ajusta translation em 0.5/ppm no eixo do diff ímpar
        // para preservar o centro VISUAL do conteúdo em world space.
        let new_translation = recenter_after_pad(
            old_translation,
            [size_world, size_world],
            [result.size, result.size],
            PixelBounds { x: offset_x, y: offset_y, width, height },
        );

        // Apply: substitui source + size + translation. C1: libera
        // texture_id antigo (refcount-aware via individual_mut().release).
        sprite.source = Individual { texture_id };
        sprite.size = [size_world, size_world];
        transform.translation = new_translation;
        if let Some(old_id) = old_individual { renderer.individual_mut().release(old_id); }
    }
}
```

### Por que centered-padding + sub-pixel recenter preserva world position

O modelo PH2D usa center-anchor (`translation` = centro geométrico do
sprite render). `make_square` faz padding **centrado** com `floor(diff/2)`
no leading edge:

- **Diff par (e.g., 64×32 → 64×64):** offset = 16 = diff/2 exato. O
  centro pixel do conteúdo original mapeia para o centro pixel do novo
  canvas. Translation não precisa mudar.
- **Diff ímpar (e.g., 65×32 → 65×65):** offset = floor(33/2) = 16,
  leaving 17 no trailing edge. Centro pixel do conteúdo (32) está
  0.5 px do centro pixel do canvas (32.5). Sem correção, o conteúdo
  visualmente desliza 0.5/ppm em world coords após cada Make Square em
  dim ímpar — driftando ao longo do ciclo Trim↔Square↔Trim. O
  `recenter_after_pad` compensa exatamente esse meio-pixel.

A garantia "Trim → MakeSquare → Trim preserva pixels" é coberta pelo
teste `round_trip_trim_then_make_square_then_trim_preserves_bbox_and_pixels`
em `tests/make_square_algorithm.rs` (audit fix N1).

## 4. Wiring esperado

- Variant nova em `IconId` — sugestão: `IconId::MakeSquare`.
  SVG path (Lucide `square.svg`):
  ```svg
  <rect width="18" height="18" x="3" y="3" rx="2" />
  ```
  Equivalente Rust: `square_bezpath()` (já implementado neste módulo).
- Cluster Image Tools em `topbar_clusters()` (fixture.rs ou top_bar/
  novo): adicionar item `MakeSquare` no slot imediatamente à direita
  do `Trim`. NodeId no range TopBar 100..199 — sugestão `109` se
  Trim ocupar `108`, ou o próximo livre.
- Re-export em `tools/mod.rs`:
  ```rust
  pub mod make_square;
  ```
- Click handler em `shells/desktop/src/main.rs` (ou onde o handler
  do Trim ficar), invocando `on_make_square_clicked`.

## 5. Checklist do Coordenador

Estado pós-integração (commits `49dfcb8` + `e3e1671` + audit fixes):

- [x] `pub mod make_square;` em `tools/mod.rs` + re-exports em `lib.rs`.
- [x] `IconId::MakeSquare` em `icons.rs` (via `IconCmd::Rect`, paralelo
      Trim — `square_bezpath()` segue exportada para consumidores diretos).
- [x] Item `MakeSquare` na `ACTIONS` slice do `paint_image_action_row`
      em `screens/hero/topbar.rs`, lado a lado com Trim.
- [x] `IMAGE_ACTION_MAKE_SQUARE = NodeId(118)` em `screens/hero/ids.rs`.
- [x] `pending_make_square: Option<u64>` em `HeroScreen` + click handler
      em `apply_event` (mirror Trim).
- [x] Drainer em `shells/desktop/src/main.rs` com C1 (texture leak fix
      conjunto Trim+MS) + M1 (cap pré-render) + M2 (sub-pixel recenter
      via `recenter_after_pad`).
- [x] Tooltip "Make Square" registrado em `topbar.rs::populate()`.
- [ ] **Pendente — follow-up Trim+MS conjunto:** Label i18n
      `tool.make_square.label` (HR-15 — bundle Fluent não existe ainda;
      Trim tem mesma pendência).
- [ ] **Pendente — follow-up Trim+MS conjunto:** A11y `AccessKit::Button`
      com label + role + action `Default` (HR-12 — Trim idem).
- [ ] **Pendente — follow-up Trim+MS conjunto:** Undo entry
      `"Make square"` via `EditorCommandQueue` (drainer atual muta
      `Sprite` + `Transform` direto; Trim idem; refactor pra
      `EditorCommand::SetComponent` cobre os dois drainers de uma vez).
- [x] Smoke test e2e (validado manualmente pelo Enio post-`e3e1671`).

## 6. Caso de teste manual após integração

1. Carregar sprite PNG não-quadrado (ex: 64×32 com objeto centralizado).
2. Selecionar o sprite na Hierarchy.
3. Click `[🔲 Image Tools]` → barra muda pra Fase B.
4. Click `[▢ MakeSquare]` → sprite passa pra 64×64 com 16 px de
   banda transparente em cima e em baixo, pivô mantém posição visual
   no canvas, undo disponível com label "Make square".
5. Click novamente em sprite já-quadrado → no-op, sem nova entry de
   undo.
6. Ctrl+Z após operação válida → sprite restaurada pra 64×32, pivô
   original.

## 7. Notas de design

- **Sem deformação, sem crop:** todo pixel da fonte aparece no destino
  byte-por-byte. A operação é puramente aditiva (pixels novos são
  sempre `(0,0,0,0)`), tornando `make_square` idempotente após a
  primeira aplicação — teste cobre essa propriedade
  (`idempotent_after_first_application`).
- **Centralização (floor na borda inicial):** banda transparente
  dividida o mais equitativamente possível, com `floor(diff/2)` na
  borda inicial (top ou left) e o resto na final. Diff par →
  simétrico; diff ímpar → 1 px a mais na borda inferior/direita.
  Consistente com ImageMagick `-gravity Center -extent` e CSS
  centered-background.
- **Sentinela degenerada:** entradas com `width == 0` ou `height == 0`
  retornam 1×1 RGBA fully-transparent com `made_square = true`,
  mesmo contrato de Trim (asset stores não precisam de branch
  separado pra zero-area).
- **Pivot reprojection vive no chrome:** algoritmo só emite offsets;
  handler do Coordenador faz a matemática usando o modelo `Sprite`
  específico do editor.

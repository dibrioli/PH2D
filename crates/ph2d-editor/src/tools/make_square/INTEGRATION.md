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

Pseudocódigo do handler que o Coordenador implementa (paralelo direto
ao `on_trim_transparency_clicked`):

```rust
fn on_make_square_clicked(editor: &mut Editor) {
    let selection = editor.selected_sprites();
    if selection.is_empty() { return; }
    let mut any_changed = false;
    for sprite in selection {
        let rgba = sprite.image_data();
        let (w, h) = sprite.dimensions();
        let result = make_square(rgba, w, h);
        if !result.made_square { continue; }

        // 1. Replace pixels.
        sprite.set_image(result.pixels, result.size, result.size);

        // 2. Reproject pivot to preserve world position.
        //    new_px = (old_w * old_px + offset_x) / new_size
        let new_pivot_x = ((sprite.w as f32 * sprite.pivot_x)
                           + result.offset_x as f32)
                          / result.size as f32;
        sprite.pivot_x = new_pivot_x.clamp(0.0, 1.0);
        let new_pivot_y = ((sprite.h as f32 * sprite.pivot_y)
                           + result.offset_y as f32)
                          / result.size as f32;
        sprite.pivot_y = new_pivot_y.clamp(0.0, 1.0);

        // 3. Events.
        editor.events.push(SpriteImageChanged { id: sprite.id });
        editor.events.push(TransformUpdated { id: sprite.id });
        any_changed = true;
    }
    if any_changed {
        editor.history.push("Make square");
        editor.request_redraw();
    }
}
```

**Importante (paralelo direto com Trim Transparency):** a reprojeção
de pivô NÃO está no algoritmo. O algoritmo só devolve `offset_x` e
`offset_y` (quanto a imagem original ficou deslocada dentro do novo
canvas). A matemática de pivô depende do modelo `Sprite` que o
Coordenador definir.

A fórmula `(old_dim * old_pivot + offset) / new_size` é o **inverso
exato** da usada pelo Trim — onde Trim *subtrai* `bounds.{x,y}` e
divide por dimensão reduzida, Make Square *adiciona* `offset_{x,y}` e
divide por dimensão expandida. Isso garante que um sprite que passe
por Trim → Make Square mantém o pivô na mesma posição absoluta em
world space ao longo das duas operações.

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

- [ ] Add `pub mod make_square;` em
      `crates/ph2d-editor/src/tools/mod.rs` (ao lado do
      `pub mod trim_transparency;`).
- [ ] Re-export `MakeSquareResult`, `make_square`, `square_bezpath` em
      `lib.rs` se pertinente (mesmo critério do trim).
- [ ] `IconId::MakeSquare` em `icons.rs` com o glyph acima.
- [ ] Item `MakeSquare` no cluster Image Tools, lado a lado com
      `Trim`.
- [ ] Render do ícone: `square_bezpath()` com
      `Affine::scale(chip_px / 24.0)` + `Stroke::new(2.0)` +
      `Stroke::with_caps(Round, Round)`.
- [ ] Label: chave i18n `tool.make_square.label` (HR-15) — fallback
      `"Make Square"` até bundle Fluent existir.
- [ ] A11y: `AccessKit::Button` com label da chave acima, role
      `Role::Button`, action `Action::Default` (HR-12).
- [ ] On-click handler conforme §3.
- [ ] Undo: empilhar entry `"Make square"` no history.
- [ ] Smoke test cobrindo image-tools → click → asset modificado →
      render redrawed.

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

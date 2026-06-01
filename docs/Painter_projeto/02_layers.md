# 02 — Camadas (Layers)

## 2.1 Tipos de camada

Seis tipos, todos representados como variants do enum `LayerKind`:

| Kind | Descrição | Editável diretamente? |
|------|-----------|-----------------------|
| **Raster** | Camada bitmap padrão (RGBA8 ou RGBA16F dependendo de canvas profile). Default ao criar. | Sim — pincel, eraser, fill, transform. |
| **Mask** | Bitmap grayscale 8-bit; multiplica alpha de uma camada parent. Bound a 1 raster layer. | Sim — pincel pinta na mask, white = visível, black = oculto. |
| **Clipping Mask** | Limita a área de pintura à pintura da layer imediatamente abaixo. Não tem bitmap próprio — é um modifier da camada raster que ela contém. | Sim — pincel só "marca" onde a layer-abaixo tem alpha > 0. |
| **Reference** | Marca uma layer raster como **reference para ColorDrop** e operações de fill (define geometry sem precisar estar ativa). Não-destrutivo. | Toggle on/off via menu da camada. |
| **Group** | Container que agrupa N layers; pode ser collapsed; aplica blend mode/opacity ao stack inteiro. Hierarquia ilimitada (mas LOC cap §2.6). | Não é pintável — só organiza. |
| **Alpha-locked** | Modifier de uma camada raster: novas pinturas só aparecem onde já há alpha. Toggle não-destrutivo. | Sim — pincel pinta restringido ao alpha existente. |

**Não suportado** (decisão consciente, vide [12_fora_de_escopo.md](12_fora_de_escopo.md) §12.2):
- **Adjustment Layers** estilo Photoshop. Adjustments são destrutivas (aplicam a uma layer/selection). Razão: workflow Procreate lean; manter coerência com o sabor.
- **Smart Object Layers** / instâncias linkadas. Razão: paradigma alternativo é o node-graph (W12+ integration).
- **Text Layers** com edição. Razão: texto é outra tool (Text Tool), com parley, em layer raster commitada.

## 2.2 Blend modes — lista canônica (22 modos)

Lista enxuta vs Procreate (29). Tirados por redundância prática:
- "Darker Color" / "Lighter Color" — substituídos por Darken/Lighten que cobrem 95% dos casos.
- "Pin Light" / "Hard Mix" — raramente usados, fora.
- "Subtract" / "Divide" — fora; sub-modos de Math expression em adjustments cobrem se preciso.
- "Shade" (Procreate-specific PSD compat) — fora; usar Darken.

Ordem oficial no popover (5 grupos):

### Normal
1. `Normal` — Porter-Duff over.

### Darken
2. `Multiply`
3. `Darken`
4. `Color Burn`
5. `Linear Burn`

### Lighten
6. `Lighten`
7. `Screen`
8. `Color Dodge`
9. `Add` (Linear Dodge)

### Contrast
10. `Overlay`
11. `Soft Light`
12. `Hard Light`
13. `Vivid Light`
14. `Linear Light`

### Difference
15. `Difference`
16. `Exclusion`

### Color (HSL)
17. `Hue`
18. `Saturation`
19. `Color`
20. `Luminosity`

### Especiais
21. `Behind` — pinta apenas onde alpha == 0 (Photoshop "Behind"; útil pra colorir sob line art sem mask).
22. `Clear` — destrutivo, zera alpha (mas mais usável que Photoshop's; equivale a "erase mode" no compositor).

Implementação: cada blend mode é uma função `(dst_rgba, src_rgba) → new_dst_rgba` no compositor (CPU para low-level operations + GPU para layer composition em real-time). Lista canônica em `crates/ph2d-painter-brush/src/blend.rs::BlendMode`.

**Compositor:** dado `N` layers, compositor recompõe top-down em real-time durante render. Cache por layer (texture); invalidação por dirty rect quando o user pinta. Bench de 50 layers @ 4K em §08.

## 2.3 Operações na camada

Acesso: tap (single) no thumb da camada abre o menu lateral; long-press abre Properties pop-up. Layout do menu:

```
┌─────────────────────────────┐
│  Rename                     │
│  Select                     │  ← gera selection do conteúdo opaco
│  Copy                       │
│  Fill                       │  ← com active color
│  Clear                      │
│ ─────────────────────────── │
│  Alpha Lock          ◯/●    │  ← toggle
│  Mask                       │  ← cria layer de mask filha
│  Clipping Mask       ◯/●    │  ← toggle
│  Reference           ◯/●    │  ← toggle
│ ─────────────────────────── │
│  Flip Horizontal            │
│  Flip Vertical              │
│ ─────────────────────────── │
│  Merge Down                 │
│  Combine Down (Group)       │  ← agrupa este + abaixo
│  Duplicate                  │
│  Delete                     │
│  Lock                ◯/●    │  ← toggle
└─────────────────────────────┘
```

### 2.3.1 Atalhos de teclado (desktop)

| Atalho | Ação |
|--------|------|
| `Ctrl+J` | Duplicate layer |
| `Ctrl+G` | Group layers (selected) |
| `Shift+Ctrl+G` | Ungroup |
| `Ctrl+E` | Merge Down |
| `Shift+Ctrl+E` | Flatten Visible |
| `Ctrl+Shift+N` | New layer |
| `Ctrl+/` | Lock/unlock active |
| `Ctrl+,` | Alpha lock toggle |
| `Ctrl+Alt+G` | Clipping mask toggle |

## 2.4 Layer panel — gestures

Espelhamento dos gestos de Procreate, com extensões desktop.

| Gesto | Ação | Plataforma |
|-------|------|------------|
| **Tap** | Selecionar layer (primary selection) | Todas |
| **Swipe right (1-finger)** | Selecionar layer (secondary selection — multi-select pra mover/transform como conjunto) | Touch |
| **Shift+Click** | Multi-select range (primary + tap) | Desktop |
| **Ctrl+Click** | Multi-select discreto | Desktop |
| **Long press** | Abrir Properties pop-up | Todas |
| **2-finger tap no thumb** | Slider de opacity inline aparece | Touch |
| **2-finger pinch (touchpad)** | Idem | Desktop trackpad |
| **2-finger swipe right** | Toggle Alpha Lock | Touch |
| **2-finger hold no thumb** | Gera selection do conteúdo opaco da layer (= Select operation) | Touch |
| **Pinch dois layers** | Merge Down (combina os dois layers num só) | Touch |
| **Drag up/down** | Reordenar layer no stack | Touch + Desktop |
| **Right-click** | Menu de operações (igual ao menu do thumb) | Desktop |

## 2.5 Layer limit dinâmico

Layer count máximo = `f(canvas_dimensions, color_format, MemoryBudget)`. Cálculo:

```
bytes_per_layer = width × height × bytes_per_pixel
  bytes_per_pixel = 4 (RGBA8) ou 8 (RGBA16F, P3 wide gamut)

max_layers_raw = (vram_budget_mb * 1024 * 1024) / bytes_per_layer

max_layers = min(max_layers_raw, HARD_CAP_999)  // espelha Procreate
```

**Exibido na Canvas creation dialog** em tempo real conforme o usuário ajusta dimensions/DPI/profile. Pra Painter v1.0, HARD_CAP é **999**.

### 2.5.1 Tabela aproximada (Apple M2 desktop, vram_budget 1200 MB)

| Canvas | RGBA8 (sRGB) | RGBA16F (P3) |
|--------|--------------|--------------|
| 1024×1024 (square) | 300 layers (cap 999 não bate; fica 300) | 150 |
| 2048×2048 | 75 | 38 |
| 4096×4096 | 19 | 9 |
| 4K (3840×2160) | 38 | 19 |
| 8K (7680×4320) | 9 | 5 |
| Max canvas iPad-like (16384×8192) | 2 | 1 |

> Os números variam com `MemoryBudget::Painter.vram_mb` per platform — pra iPad 350 MB → metade aproximada; pra web 200 MB → 1/6.

Quando o usuário atinge o limite e tenta criar +1 layer, dialog: *"Layer limit atingido. Considere flatten algumas layers ou criar um canvas menor."* — não popup de erro, popup explicativo.

## 2.6 Group nesting cap

Limite de hierarquia: **8 níveis de profundidade**. Razão arbitrária (Procreate é unbounded; nós cap para sanidade de UI + bench). Limite enforced no app — tentativa de criar grupo nesteado nível 9 cliente UI silenciosamente para depth 8 (fold automático).

## 2.7 Mask layer — detalhes

Mask é grayscale 8-bit (`R8Unorm`), 1 byte/pixel. Pintar com pincel em mask = pintar em valores de luminance (white→255, black→0). Tudo que está em white = visível na parent layer; black = invisível.

- Mask **inicia branca** (full visible).
- Pintar com `Color::WHITE` (active color) = revelar.
- Pintar com `Color::BLACK` = esconder.
- Active color é automaticamente convertida para luminance (`L = 0.299*R + 0.587*G + 0.114*B`).
- Brush continua o mesmo (Shape × Grain × tudo); só a interpretação muda.

**Invert mask** disponível no menu da mask layer. **Apply mask** (destrutivo, "bake" mask into parent) também disponível.

Layout no panel: mask aparece como sub-layer do parent, indentada à direita, com ícone distintivo (square com furos).

## 2.8 Clipping mask — detalhes

A camada raster vira clipping mask de **toda a layer imediatamente abaixo** (não da camada-base do grupo; do vizinho direto). Pintar nela só "marca" onde a abaixo tem alpha > 0.

- Toggle não-destrutivo — pode liberar a qualquer momento sem perder pintura.
- N clipping masks podem encadear consecutivamente (todas clippadas à mesma base).
- Hierarquia visual: clipping masks aparecem indentadas com seta apontando pra baixo.

Útil para colorir line art: line art em layer 1, paint colorido em layer 2 clipped a layer 1 → cor só aparece dentro das linhas.

## 2.9 Reference layer — detalhes

Marca uma layer raster como "geometria referência" para o ColorDrop e outras operações de fill. Quando ativa:
- ColorDrop (§03) usa a geometria da reference layer para definir o flood-fill region, **não** a layer ativa.
- Permite pintar embaixo de line art numa layer separada (Color), com flood preso à geometria da line art (Reference), mantendo color e line art separadas.

Apenas **1 reference layer por canvas** simultaneamente. Toggle outra reference layer auto-desativa a anterior.

Visual: layer marcada com badge "Reference" pequeno no canto do thumb.

## 2.10 Alpha lock — detalhes

Modifier toggle por layer. Quando ativo: novas pinturas só escrevem em pixels onde alpha > 0. Não-destrutivo.

Implementação no compute shader: depois de calcular `new_color`, se `alpha_lock_enabled && layer.alpha[pixel] == 0`, não escreve.

Visual: thumbnail ganha checkerboard pattern por trás (indica que o alpha é "locked").

## 2.11 Compositor (composição final)

Top-down recursive compositor. Algoritmo simplificado:

```rust
fn composite_stack(layers: &[Layer], canvas_size: Size) -> Texture {
    let mut result = Texture::transparent(canvas_size);
    for layer in layers.iter().rev() {  // bottom-to-top
        if !layer.visible { continue; }
        let layer_tex = match &layer.kind {
            Group(children) => composite_stack(children, canvas_size),
            Raster(tex) => tex.clone(),
            _ => continue,
        };
        let layer_tex = apply_mask_if_any(layer_tex, layer.mask);
        let layer_tex = apply_clipping_if_any(layer_tex, layer.clipped_by);
        let alpha = layer.opacity * layer.visibility;
        composite_with_blend_mode(&mut result, &layer_tex, layer.blend_mode, alpha);
    }
    result
}
```

**Dirty rect tracking:** quando user pinta numa layer, marca dirty rect; compositor apenas re-composita do menor `bounding_box(dirty_rect, ancestor_clipping_box)` para cima. Reduz custo drasticamente em workloads típicos (pinta numa zona pequena, mantém 95% do canvas cached).

Cache de layers em `LayerCache` (HashMap<LayerId, Texture> — Note: `EntityHashMap`-like em sim crates, mas Painter está em `PresentWorld`, então HashMap regular OK por HR-5 escopo).

## 2.12 Performance gates

> **Nota (W3 Block 2, 2026-05-31):** os gates abaixo foram **realizados em
> `ph2d-render`** (compositor GPU) com nomes concretos. O nome histórico
> `layers_composite_50_4k_under_5ms` foi **dividido em dois** porque o
> recompose 4K-cheio × 50 layers lê 1.66 GB → é **bandwidth-bound** (~23ms numa
> GPU de ~70 GB/s, ~4ms em ≥330 GB/s); o budget de 5ms vale no caminho
> INTERATIVO (dirty-rect). `layers_blend_mode_golden` (SSIM vs Photoshop) foi
> substituído por bit-paridade CPU↔GPU (`shader_blend_modes_bit_identical` +
> readback ≤1 byte) — mais forte e sem assets externos (ADR-rationale).

| Gate (implementado) | Crate | Valida |
|------|-------|--------|
| `gpu_composite_50_layers_dirty_rect_under_5ms` | `ph2d-render` | 50 layers, dirty-rect 512² (caminho interativo real) ≤ 5ms |
| `gpu_composite_full_4k_scales_linearly` | `ph2d-render` | recompose 4K cheio escala ~linear (50 vs 10 layers < 6×) — sem cliff de ocupação |
| `gpu_dirty_rect_matches_full` | `ph2d-render` | recompositar sub-região == crop do full composite (bit-idêntico) |
| `shader_blend_modes_bit_identical_with_rust` + GPU readback | `ph2d-render` | literais dos 22 modos pinados a Rust + output GPU≈CPU ≤1 byte |
| `max_layers_for_budget` (`layers_max_count_per_budget`) | `ph2d-render` | `max_layers` por budget bate o documentado; `TooManyLayers` recusa no cap |
| `layers_no_alloc_hot_compose` | `ph2d-render` | flatten do op-list é alloc-free (HR-3) |
| `layers_no_alloc_hot_compose` | idem | HR-3: composite path zero-alloc com layer stack pré-alocado |

## 2.13 Memory model — layer storage

**RAM:** stack de layers + metadata (`LayerId`, `name`, `blend_mode`, `opacity`, `mask_handle`, `clipped_by_handle`, etc.). Custo: ~256 bytes/layer.

**VRAM:** texture por layer (bitmap RGBA8 ou RGBA16F). Mask = R8Unorm separada.

**LayerCache evictioin:** se VRAM budget estoura, evicta cache de layers não-visíveis (offload pra disk swap em `tmp/painter_layer_cache/<canvas_uuid>/<layer_uuid>.bin`, postcard). Re-load lazy. Quando o sistema operacional sinaliza low memory (HR + `host_low_memory`), evicta agressivamente.

Stroke history: ring buffer de 250 frames undo, separado das layers. Detalhe em §08.

**Continua em:** [03_color.md](03_color.md) — sistema de cor e palettes.

# 06 — Mask & Clip — `ClipChildren` + `MaskInteraction`

## 6.1 Dois conceitos independentes

| Conceito | Component | Função | Inspirado em |
|---|---|---|---|
| **ClipChildren** | `ClipChildren(Mode)` no node-pai | Recorta DESCENDENTES pela silhueta do node-pai | Godot |
| **MaskInteraction** | `MaskInteraction { mode, alpha_cutoff }` no node-mask-responder | Sprite responde a um Mask2D irmão na hierarquia | Unity SpriteMask |

Os dois podem coexistir num mesmo node. ClipChildren é "este node mascara seus filhos". MaskInteraction é "este node é mascarado por um Mask2D externo".

## 6.2 ClipChildren — 3 modos

```rust
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ClipMode {
    /// Default. Sem clip; render normal.
    Disabled,
    /// Filhos recortados pela silhueta do node, mas node NÃO desenha.
    /// Útil pra "molde": progress bar de forma arbitrária onde o sprite-molde
    /// é só o template, a barra é o filho.
    ClipOnly,
    /// Node desenha + filhos recortados pela silhueta do node desenhado.
    /// Útil pra avatar circular (sprite circular + foto retangular dentro).
    ClipAndDraw,
}
```

### Casos de uso canônicos

| Caso | Modo | Setup |
|---|---|---|
| Avatar circular | `ClipAndDraw` | sprite-pai circular; foto-filho retangular dentro |
| Progress bar com forma arbitrária | `ClipOnly` | sprite-pai = molde da barra; filho = barra colorida que cresce |
| Sprite com furos | `ClipOnly` | sprite-pai com alpha-cutout; filho = textura visível só onde pai opaco |
| UI com bordas redondas | `ClipAndDraw` | NinePatch arredondado + conteúdo dentro |
| Botão com ripple effect | `ClipAndDraw` | sprite-pai = forma do botão; filho = ripple animation contida |

### Implementação (back-buffer based)

ClipChildren usa **backbuffer copy**: o sprite-pai é desenhado num render-target; filhos depois desenham nele; alpha do pai vira mask binário (com alpha_cutoff configurável). Pass extra fullscreen.

**Custo:** 1 render target extra por hierarquia com ClipChildren ativo. Aceitável quando usado conscientemente (Inspector mostra warning quando >5 sprites num frame usam ClipChildren — performance hint).

**Incompatibilidade documentada:** `ClipChildren` + `CanvasGroup` (composite-then-blend) **não podem coexistir no mesmo node** — ambos usam backbuffer. PH2D **não terá CanvasGroup separado** (Godot tem), porque ClipChildren cobre o caso útil.

## 6.3 ClipChildren — gate de regressão obrigatório

Godot teve 5 issues abertos sucessivos:
- [#79885 Sprite2D clip not working](https://github.com/godotengine/godot/issues/79885)
- [#102190 Clip children no longer works (regression)](https://github.com/godotengine/godot/issues/102190)
- [#102224 Clip children no longer masks by alpha (Control)](https://github.com/godotengine/godot/issues/102224)
- #91068, #90793, #98882

**PH2D adiciona gate específico** ([tests/clip_children_regression.rs](../../crates/ph2d-render/tests/) — a criar em W3):

```rust
#[test]
fn clip_only_mode_renders_children_within_parent_silhouette() {
    let parent = create_sprite_circle(64x64);
    let child = create_sprite_rect(64x64);
    parent.add_component(ClipChildren(ClipMode::ClipOnly));
    parent.add_child(child);

    let render = render_scene_to_image(...);
    
    // Parent sprite NÃO deve aparecer.
    assert_eq!(render.pixel_at(center_of_circle), TRANSPARENT);  // failed in #79885
    
    // Child deve aparecer SÓ onde parent era opaco.
    assert_eq!(render.pixel_at(corner_of_rect_outside_circle), TRANSPARENT);  // failed in #102190
    assert_eq!(render.pixel_at(center_of_circle), child_color);  // passes
}

#[test]
fn clip_and_draw_renders_parent_plus_clipped_child() {
    // ... idem com parent visível ...
}

#[test]
fn disabled_mode_renders_normally() {
    // ... idem sem clip ...
}
```

Smoke do Enio em cada wave que toca a feature: visual checklist 5 cenários.

## 6.4 MaskInteraction

Sprite com Component `MaskInteraction { mode, alpha_cutoff }` responde a um Mask2D (entidade separada com Component `Mask2D` — futuro).

### Modos
- **None** — default; sprite ignora masks.
- **VisibleInside** — sprite renderiza onde mask é "dentro" (alpha > cutoff).
- **VisibleOutside** — sprite renderiza onde mask é "fora" (alpha ≤ cutoff). Inverso.

### Casos de uso
- HP bar: mask preenche o "fill", sprite-fill aparece com `VisibleInside`.
- Fog of war: mask é o "olho" do player; foglayer sprite com `VisibleOutside` aparece fora do olho.
- Spotlight reveal: portrait window mask; portrait sprite com `VisibleInside` aparece só dentro da window.

### Alpha Cutoff
Threshold de alpha que conta como "dentro" da mask. Default 0.5 (intuitivo). Slider 0..1 no Inspector.

### Custom Range (sub-seção avançada)
Component `MaskCustomRange { front_sorting_layer, back_sorting_layer }` (opcional). Quando presente, mask afeta APENAS sprites cuja SortingLayer está no intervalo `[back, front]`. Permite múltiplas masks coexistindo sem brigar.

Inspirado em Unity Custom Range — não é caso comum, vai pra sub-seção colapsável.

## 6.5 Inspector layout

```
▼ Visibility
  ☐ Visible
  Visibility Layer: [bitmask grid 4x8]
  
  Clip Children:
    Mode:  [Disabled ▾ | Clip Only | Clip And Draw]
  
  Mask Interaction:
    Mode:  [None ▾ | Visible Inside | Visible Outside]
    Alpha Cutoff: [▒▒▒▒▒░░░░░] 0.50            (só visível se Mode != None)
    ▸ Custom Range (Sorting Layer ranges)       (colapsado por default)
  
  ▸ On-Screen Enabler
```

## 6.6 Mask2D (entidade separada, fora do escopo Inspector v2)

O **Mask2D** propriamente (entidade que serve de mask) é uma feature de outro spec (módulo Mask futuro). Inspector do Sprite v2 só expõe `MaskInteraction` no sprite que RESPONDE à mask.

Razão de separação: Mask2D tem seu próprio Inspector (source: SpriteRef vs SupportedRenderer; sprite_sort_point; etc.). Não cabe no escopo do Sprite Inspector.

## 6.7 Incompatibilidades documentadas

| Combinação | Resultado |
|---|---|
| `ClipChildren != Disabled` + `MaskInteraction != None` no MESMO node | **Aceito**, mas custo dobrado (2 backbuffer passes). Inspector mostra warning. |
| `ClipChildren` + `ParentMaterial` (filhos compartilham material do pai) | **Aceito**. ClipChildren afeta render-target; ParentMaterial é metadata de batching. |
| `MaskInteraction` + `BlendMode::Add` | **Aceito**. Mask afeta alpha; blend mode afeta RGB composition. Ortogonais. |
| `ClipChildren` + sprite com `Material` shader custom que escreve via fragment | **Aceito**, mas shader custom precisa respeitar `discard;` em fragments fora da silhueta. Inspector mostra hint. |

## 6.8 Caps gateados

| Cap | Valor | Razão |
|---|---|---|
| `ClipMode` variants | 3 (Disabled/ClipOnly/ClipAndDraw) | Coberta 100% dos casos |
| `MaskInteraction.mode` variants | 3 (None/VisibleInside/VisibleOutside) | Unity acerta com esses 3 |
| `alpha_cutoff` range | `[0.0, 1.0]` | clamp |

## 6.9 Anti-padrões evitados

1. **Cliip Children como flag boolean simples** ❌ — Godot legacy v3 só tinha bool. PH2D 3 modos cobrem template/molde/avatar.
2. **MaskInteraction implícito via SortingLayer apenas** ❌ — Phaser confunde os dois conceitos. PH2D separa: ClipChildren = hierarchical, MaskInteraction = layer-based.
3. **Mask2D dentro do Sprite Inspector** ❌ — feature complexa, merece spec próprio.
4. **Shader custom como única forma de mask** ❌ — barreira alta pra artistas. ClipChildren + MaskInteraction cobrem 99% via Inspector.
5. **Backbuffer pass sem warning de custo** ❌ — Inspector mostra "Render passes: 2" quando ClipChildren ativo (performance hint).

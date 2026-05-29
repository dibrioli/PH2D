# ADR-0071 — Tint channels — matemática multiplicativa canônica (4 canais)

**Status:** Accepted (2026-05-28) — ratificado pelo Enio pós 5 lentes adversariais.
**Decisor(es):** Enio + Claude (Coord-A sessão paralela docs-only, Sprite Inspector W0).
**Pré-requisitos:** [ADR-0069 — decisão-mãe](0069-sprite-inspector-v2.md), [ADR-0070 — schema v4](0070-sprite-schema-v4.md).
**Spec normativa:** [`docs/Sprite_projeto/04_color_tint_canais.md`](../../Sprite_projeto/04_color_tint_canais.md).
**Tags:** sprite, color, tint, blend, canon

---

## 1. Contexto

Pesquisa multi-engine identificou que **nenhum engine expõe os 4 canais de tint que cobrem todos os casos práticos**:

| Caso de uso | Canal necessário | Quem tem hoje |
|---|---|---|
| Fade-out de cena inteira (hierarchy-cascade) | **Tint herdável** (modulate) | Godot ✓ |
| Hurt-flash de char inteiro (cascade) | **Tint herdável** | Godot ✓ |
| Piscar arma sem piscar char (local) | **Self Tint** (self_modulate) | Godot ✓ (único) |
| Gradient céu-chão num sprite | **Per-corner tint** (4 cantos) | Phaser ✓ (único) |
| Damage flash = silhueta colorida | **Tint Fill** (substitui RGB) | Phaser setTintFill ✓ (único) |
| Animar opacity preservando RGB | **Opacity** separado de tint.a | Universal |

**Combinação dos 4 canais = padrão-ouro absoluto.** Sem isso, cada caso vira hack (shader custom, sprite duplicado, etc.).

Da pesquisa também: **ordem de multiplicação ambígua** é fonte de bugs (Unity teve shaders com Add em vez de Mul; Godot misturou em alguns paths). Esta ADR **canoniza ordem multiplicativa pura**.

---

## 2. Decisão

### 2.1 Quatro canais independentes

Vivem no `Sprite` struct v4 ([ADR-0070](0070-sprite-schema-v4.md)):

| Canal | Tipo | Herda? | Default |
|---|---|---|---|
| `tint` | `[f32; 4]` RGBA | **SIM** (cascateia hierarquicamente) | WHITE |
| `self_tint` | `[f32; 4]` RGBA | NÃO (aplica só neste sprite) | WHITE |
| `per_corner_tint` | `[[f32; 4]; 4]` (TL/TR/BL/BR) | NÃO (vertex color, bilinear interp) | [WHITE; 4] |
| `opacity` | `f32` | NÃO (multiplicador final local) | 1.0 |

### 2.2 Ordem de multiplicação canônica

```
final_rgb = sample_rgb [SE NOT tint_fill, senão WHITE]
          × per_corner_tint_rgb (bilinear interpolated do vertex)
          × self_tint_rgb
          × Π(modulate_ancestors_rgb)
          × premultiply_factor [SE blend != PremultAlpha]

final_alpha = sample.a
            × per_corner_tint.a (bilinear interpolated)
            × self_tint.a
            × Π(modulate_ancestors.a)
            × opacity
```

**Critical:** TODOS os multiplicativos. Sem mistura (lerp), sem additive, sem max/min. Composição commutative (modulo precisão FP) — facilita batching e collapsing CPU-side.

### 2.3 Cascade collapse CPU-side (extract phase) — top-down traversal canônica

**Ordem traversal canônica (Lens C H3):** `ancestors` chega em ordem **top-down** (`ancestors[0]` = root; `ancestors[N-1]` = pai direto). Razão: FP multiplicação **NÃO é associative** em precisão finita; `(a×b)×c ≠ a×(b×c)` em ULP boundary. Sem ordem fixa, mesmo sample idêntico cross-OS pode produzir cascade divergente.

```rust
fn extract_sprite_tint(sprite: &Sprite, ancestors: &[GlobalSpriteState]) -> [f32; 4] {
    // ⚠️ ancestors DEVE estar em ordem top-down (root primeiro).
    // Gate `extract_sprite_tint_traversal_order_canonical` enforce.
    let mut cascade = sprite.self_tint;
    cascade[0] *= sprite.tint[0];
    cascade[1] *= sprite.tint[1];
    cascade[2] *= sprite.tint[2];
    cascade[3] *= sprite.tint[3];
    for a in ancestors {                   // root → pai direto
        cascade[0] *= a.tint[0];           // só `tint` cascateia (não `self_tint` ancestral)
        cascade[1] *= a.tint[1];
        cascade[2] *= a.tint[2];
        cascade[3] *= a.tint[3];
    }
    cascade
}
```

`RenderInstance.tint` recebe `cascade` collapsed. `RenderInstance.per_corner_tint` recebe cópia direta de `sprite.per_corner_tint`. `RenderInstance.opacity` recebe `sprite.opacity`.

**Insight:** apenas `tint` ancestral é multiplicado. `self_tint` do ancestral NÃO afeta filho — esse é o ponto de Self Tint independente.

**Gate `extract_sprite_tint_traversal_order_canonical`** (W2 cria): test asserta `ancestors[0].depth == 0`, `ancestors[N-1].depth == sprite.depth - 1`. Falha imediatamente se Bevy ECS mudar ordem de iteração de hierarquia (proteção contra silent regression).

### 2.4 Tint Fill semantica

Quando `Sprite.tint_fill = true`:
- `sample.rgb` é **IGNORADO**.
- `final_rgb` usa apenas `per_corner_tint × cascade`.
- `final_alpha` continua usando `sample.a` (silhueta preservada).

Sprite vira silhueta colorida. Damage flash de 1 toggle.

### 2.5 PremultAlpha gotcha

`Sprite.premultiplied = true` indica sample já em RGB × alpha pré-multiplicado (BG-Removal Apply path). Fragment shader pula o multiplique pré-blend final:

```wgsl
let premul_rgb = if (instance.premultiplied != 0.0) {
    rgb        // sample já era premultiplicado
} else {
    rgb * alpha   // standard premultiply para Mix blend
};
```

**Gate:** test `premultiplied_tint_correctness.rs` compara render de sprite premultiplicado + tint amarelo contra fixture golden 4-pixel. Sem isso, fica RGB×alpha² → fringe escura (bug clássico).

### 2.6 Per-corner tint — interpolação bilinear via varying interpolation natural

Per-corner tint **chega como 4 instance vertex attrs** em `@location(9..12)` (1 vec4 por canto). Vertex stage seleciona qual canto está sendo processado **pelo quad_uv da vertex atual** (o quad é triangle-strip com 4 vertices; cada vertex tem UV nos cantos do unit quad):

```wgsl
// Em ph2d-render/shaders/sprite.wgsl (vertex stage).
// Pseudo-código canônico — impl real W2.

struct InstanceInput {
    // ... outros attrs ...
    @location(9)  per_corner_tl: vec4<f32>,
    @location(10) per_corner_tr: vec4<f32>,
    @location(11) per_corner_bl: vec4<f32>,
    @location(12) per_corner_br: vec4<f32>,
}

struct VertexInput {
    @location(0) quad_pos: vec2<f32>,  // [-0.5, 0.5]²
    @location(1) quad_uv: vec2<f32>,   // [0, 1]², TL=(0,0), TR=(1,0), BL=(0,1), BR=(1,1)
}

@vertex
fn vs_main(v: VertexInput, i: InstanceInput) -> VertexOutput {
    // ... transform vertex pos ...
    
    // Bilinear interpolation dos 4 cantos via UV do vertex atual.
    let top = mix(i.per_corner_tl, i.per_corner_tr, v.quad_uv.x);
    let bot = mix(i.per_corner_bl, i.per_corner_br, v.quad_uv.x);
    let corner_tint = mix(top, bot, v.quad_uv.y);
    
    out.corner_tint = corner_tint;
    out.uv = v.quad_uv;
    return out;
}
```

Fragment recebe `corner_tint: vec4<f32>` **automaticamente interpolado** pelo rasterizer (varying interpolation natural). Cada fragment vê a cor bilinearmente blendada dos 4 cantos — **sem código adicional no fragment shader**.

Resultado: gradient suave sem branch ou shader custom. Quatro cantos = quatro cores; rasterizer interpola.

### 2.7 Animatable property paths

Cada canal é animável via timeline (módulo Animation futuro). Paths canônicos:

- `sprite.tint`, `sprite.tint:r`, `sprite.tint:g`, `sprite.tint:b`, `sprite.tint:a`
- `sprite.self_tint` + sub-properties
- `sprite.per_corner_tint[0..3]` (per-canto)
- `sprite.opacity`
- `sprite.tint_fill` (boolean toggleable)

Padrão bate com Godot (`modulate:a`, `material:shader_parameter/foo`).

### 2.8 OKLCH ColorPicker (UI canônica)

Color picker do Inspector usa **OKLCH** (perceptually uniform). Web tem desde 2023 (Chrome 111+ / Safari 15.4+ / Firefox 113+ / Edge 111+); engines de jogo não. PH2D = primeira:
- L = lightness perceptual constante.
- C = chroma constante (cor satura sem virar).
- H = hue uniforme.

Implementação em `crates/ph2d-editor-core/src/widget/color_picker_oklch.rs` (W2.T7 cria). Conversão OKLCH ↔ RGB linear via [`ph2d-color`](../../../crates/ph2d-color/) (crate existente; tipo canônico `OklchColor` em [`src/oklch.rs`](../../../crates/ph2d-color/src/oklch.rs)). Vide [ADR-0051 Color profile pipeline](0051-color-profile-pipeline.md) para contexto color management completo.

### 2.9 Caps congelados

Arch-gate `tint_math_multiplicative_canonical` em `crates/ph2d-render/tests/`:

| Cap | Valor |
|---|---|
| Canais de tint independentes | **4 FROZEN** (tint + self_tint + per_corner + opacity) |
| Per-corner corners | **4 FROZEN** (TL/TR/BL/BR) |
| Range de opacity | `[0.0, 1.0]` (clamp) |
| Range de tint RGB | `[0.0, +∞)` (HDR-OK) |
| Range de tint alpha | `[0.0, 1.0]` (clamp) |

Bump → ADR-0071-amendment.

---

## 3. Consequências

### 3.1 Positivas

- **4 canais cobrem 99% dos casos** sem shader custom — fade hierárquico, hurt-flash local, gradient, damage silhueta, opacity animation independente.
- **Ordem multiplicativa canônica** elimina ambiguidade — testes golden cross-OS são bit-identical (modulo FP epsilon).
- **Cascade collapse O(depth)** em extract phase é trivial — sem overhead em hot path.
- **OKLCH picker** é diferencial UX raro — designers acostumados ao OKLCH em CSS finalmente têm engine equivalente.
- **Tint Fill** = damage flash de 1 toggle (sem hack).

### 3.2 Negativas

- **`Sprite` carrega 4 canais ALWAYS** (mesmo quando todos = WHITE identity) — overhead memory ~80B (4×4×4 floats). Aceito: aparência intrínseca.
- **Per-corner tint ABI** adiciona 64B ao `RenderInstance` (4 attrs × 4 floats). Mitigação dual-buffer documentada em ADR-0070.

### 3.3 Neutras

- **Animatable property paths** segue convenção Godot — sem surpresa pra LLMs.
- **PremultAlpha gotcha** já existe em v3; v4 não muda semântica, só amplia campos.

---

## 4. Alternativas consideradas

### 4.1 1 canal de tint (sem self_tint) — rejeitada

Manter apenas `tint` herdável (status v3). **Por que rejeitada:** "Piscar arma sem piscar char" exige hack (mexer modulate do char inteiro reseta filhos). Self Tint é caso real, comum.

### 4.2 Per-corner como Component opcional — rejeitada

Vide ADR-0070 §4.3. Component duplica estado de attachment vs vertex format ABI.

### 4.3 Tint Fill via flag em material — rejeitada

Exigir material override pra silhueta colorida. **Por que rejeitada:** damage flash é caso muito comum; 1 toggle no Sprite struct é estritamente mais ergonômico. Material override é heavyweight (clone material, novo draw call em alguns paths).

### 4.4 Opacity dentro de tint.a — rejeitada

Mantém `tint.a` como único alpha channel. **Por que rejeitada:** animar opacity preserving RGB exige patch path `tint:a` mas RGB animation continua usando `tint:rgb` — campo separado `opacity` é estritamente mais expressivo. Phaser, Unity SpriteRenderer também separam.

### 4.5 Mistura de modos (alguns multiplicativo, alguns additive) — rejeitada

Exemplo: `tint` multiplicativo, `glow` aditivo. **Por que rejeitada:** ambiguidade fonte de bugs (Unity teve esse problema em shaders custom). PH2D fixa multiplicativo em TODOS. Sem exceções. Sem flag "blend tint as add". Add-style efeitos vão pro FX chain (módulo Shader FX futuro).

---

## 5. Implementação (Wave 1 + Wave 2)

W1 expand Sprite v4 com os 4 canais + extract phase collapse + shader v4. Vide [ADR-0070](0070-sprite-schema-v4.md).

W2 adiciona seção 6 do Inspector (Color & Tint) com OKLCH ColorPicker. Vide [`docs/Sprite_projeto/15_plano_de_implementacao.md §15.3`](../../Sprite_projeto/15_plano_de_implementacao.md).

Gates ativos:
- `tint_math_multiplicative_canonical` (W1.T1.11 cria).
- `premultiplied_tint_correctness` (regression).
- `tint_per_corner_bilinear_interpolation` (vertex stage correctness).
- Smoke do Enio W2: 5 visual checks (vide ADR-0069 §2.4 + plano §15.3).

---

## 6. Open questions

| Q | Resposta |
|---|----------|
| Per-corner tint keyframable por-canto via timeline? | **W2 NÃO** (v1 simples: tween entire `per_corner_tint` array). Keyframe por-canto isolado = future amendment. |
| OKLCH picker fallback se browser/device não suporta? | OKLCH é cálculo puro (matemática); funciona em qualquer device. Sem fallback necessário. |
| Order of cascade: top-down OR bottom-up? | **Top-down** (ancestor first → descendant) — standard hierarchy traversal, matches Godot. |

---

## 7. Referências

- Spec normativa: [`docs/Sprite_projeto/04_color_tint_canais.md`](../../Sprite_projeto/04_color_tint_canais.md).
- ADR pais: [ADR-0069](0069-sprite-inspector-v2.md), [ADR-0070](0070-sprite-schema-v4.md).
- ADR Color profile pipeline (precedente OKLCH): [ADR-0051](0051-color-profile-pipeline.md).
- Godot self_modulate: <https://docs.godotengine.org/en/stable/classes/class_canvasitem.html#property-self-modulate>.
- Phaser per-vertex tint: <https://docs.phaser.io/api-documentation/namespace/gameobjects-components-tint>.
- OKLCH spec (W3C CSS Color 4): <https://www.w3.org/TR/css-color-4/#ok-lab>.

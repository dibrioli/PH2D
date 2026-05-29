# 04 — Color & Tint — 4 canais multiplicativos canônicos

## 4.1 Os 4 canais independentes

| Canal | Onde mora | Herda pra filhos? | Cascateia? | Default |
|---|---|---|---|---|
| **Tint** (`Sprite.tint`) | campo do struct | ✅ SIM (multiplicativo) | Cascade na hierarquia | WHITE |
| **Self Tint** (`Sprite.self_tint`) | campo do struct | ❌ NÃO | Aplicado SÓ neste sprite | WHITE |
| **Per-corner Tint** (`Sprite.per_corner_tint[4]`) | campo do struct | ❌ NÃO (vertex color) | Aplicado SÓ neste sprite, por-canto | [WHITE; 4] |
| **Opacity** (`Sprite.opacity`) | campo do struct | ❌ NÃO | Multiplicador FINAL local | 1.0 |

**Razão de TER os 4 (não 1):**

| Cenário | Sem 4 canais separados | Com 4 canais separados |
|---|---|---|
| Fade-out de cena inteira | Tint preciso varrer todas entidades | Tint do nó-pai cascateia automaticamente |
| Hurt-flash do personagem inteiro | idem | Self Tint do char raiz não afeta filhos (arma, sombra ficam intactas) |
| Piscar arma sem piscar char | Mexer modulate do char inteiro reseta filhos | Self Tint da arma muda só a arma |
| Gradient céu-chão num sprite background | Shader custom | Per-corner [TopWHITE, TopWHITE, BotORANGE, BotORANGE] |
| Animar opacity sem mexer cor | `tint.a` muda alpha + tem que preservar RGB | Opacity é multiplier final separado |

Sem os 4, cada caso vira hack ou shader custom. Com os 4, cada caso é trivial e composável.

## 4.2 Matemática multiplicativa canônica

A cor RGBA final que o fragment shader emite por-fragmento (em pré-multiplicação para o blend Mix padrão):

```wgsl
// Pseudo-código WGSL (real impl em ph2d-render/shaders/sprite.wgsl).
// `instance` = RenderInstance (CPU collapsed do Sprite + ancestors).
// `uv` = uv interpolated do quad [0..1, 0..1].

fn sample_sprite(instance: RenderInstance, uv: vec2<f32>) -> vec4<f32> {
    // 1. Sample texel from atlas/individual texture.
    let sample = textureSample(diffuse, sampler, uv);

    // 2. Apply per-corner tint via interpolation in vertex shader (bilinear).
    //    `corner_tint` chega pre-interpolated por vertex stage.
    let corner_tint = instance.corner_tint_interpolated;

    // 3. Apply self_tint × tint_cascade (collapsed CPU-side em instance.tint).
    //    instance.tint = self_tint * Π(modulate_ancestors).
    let cascade_tint = instance.tint;

    // 4. Tint Fill: se ativo, IGNORE sample.rgb, use só per-corner * cascade.
    let rgb = if (instance.tint_fill != 0.0) {
        corner_tint.rgb * cascade_tint.rgb
    } else {
        sample.rgb * corner_tint.rgb * cascade_tint.rgb
    };

    // 5. Alpha = sample.a × corner_tint.a × cascade_tint.a × opacity.
    let alpha = sample.a * corner_tint.a * cascade_tint.a * instance.opacity;

    // 6. Premultiply para blend correto (Mix). PremultAlpha branch documentado.
    let premul_rgb = if (instance.premultiplied != 0.0) {
        rgb              // sample já era premultiplicado (BG-Removal apply)
    } else {
        rgb * alpha      // standard premultiply para Mix blend
    };

    return vec4<f32>(premul_rgb, alpha);
}
```

### Ordem canônica explícita

```
final_rgb = (sample_rgb [SE NOT tint_fill, senão WHITE])
          × corner_tint_rgb
          × self_tint_rgb
          × Π(modulate_ancestors)
          × premultiply_factor

final_alpha = sample.a
            × corner_tint.a
            × self_tint.a
            × Π(modulate_ancestors.a)
            × opacity
```

**Critical:** todos os multiplicativos. Sem mistura (lerp), sem additive, sem max/min. Composição commutative (qualquer ordem dá mesmo resultado modulo precisão FP) — facilita batching e collapsing CPU-side.

## 4.3 CPU collapsing (extract phase)

O CPU collapse pre-computa multiplicações estáveis ao longo da hierarquia:

```rust
// crates/ph2d-render/src/extract.rs (ou onde fica o sistema de extract).
fn extract_sprite_tint(sprite: &Sprite, ancestors: &[GlobalSpriteState]) -> [f32; 4] {
    let mut cascade = sprite.self_tint;
    // self_tint vezes tint próprio.
    cascade[0] *= sprite.tint[0];
    cascade[1] *= sprite.tint[1];
    cascade[2] *= sprite.tint[2];
    cascade[3] *= sprite.tint[3];

    // Multiplica modulate de cada ancestral (não self_modulate dos ancestrais).
    for a in ancestors {
        cascade[0] *= a.modulate[0];
        cascade[1] *= a.modulate[1];
        cascade[2] *= a.modulate[2];
        cascade[3] *= a.modulate[3];
    }
    cascade
}
```

`RenderInstance.tint` (existente, [sprite.rs:179](../../crates/ph2d-render/src/sprite.rs#L179)) recebe `cascade`. Per-corner tint vai como attributes separados @location(9..12) interpolados em vertex stage.

**Performance:** cascade collapse é O(altura_da_hierarquia) por sprite — irrelevante. Per-corner = 4 vec4 attrs adicionais, ~64 bytes/instance, ~ +30% bandwidth de upload em cena com 10k sprites = ainda muito sob budget HR-4.

## 4.4 PremultAlpha gotcha

PH2D já suporta `Sprite.premultiplied: bool` (existente, runtime hint do BG-Removal Apply). Quando true:
- Sample já está em RGB×alpha (premultiplicado pelo cooker).
- Fragment shader pula o multiplique pré-blend.
- Math: `rgb_out = sample_rgb_premultiplied × corner_tint × cascade` (sem multiplicar por alpha porque sample já tem alpha embutido).

**Atenção (gate em testes):** quando `premultiplied=true`, **NÃO** aplicar a etapa "premultiply" do passo 6 acima. Senão fica RGB×alpha² → fringe escura.

Fixture de regressão: [tests/premultiplied_tint_correctness.rs](../../crates/ph2d-render/tests/) — compara render de sprite premultiplicado + tint=YELLOW contra fixture golden 4-pixel.

## 4.5 Tint Fill detalhado

Quando `Sprite.tint_fill = true`:
- Sample texel.rgb é **ignorado**.
- `final_rgb = corner_tint × cascade × self_tint × Π(ancestors)`.
- `final_alpha` continua usando `sample.a` (silhueta preservada).

Resultado: sprite vira **silhueta colorida** com a cor combinada. Damage flash de 1 toggle:

```rust
// Game code, no input handler "damaged":
sprite.tint_fill = true;
sprite.self_tint = [1.0, 1.0, 1.0, 1.0]; // ou outro override
// ... 50ms depois:
sprite.tint_fill = false;
```

Sem `tint_fill`, damage flash exige shader custom OU duplicar sprite. Com, é 2 linhas.

## 4.6 Per-corner tint — caso animação + GPU varying note

Per-corner é attribute em `RenderInstance`. Animável via SpriteAnimator keyframing por-canto? **Decisão:** v1 NÃO suporta keyframe por-canto (cada corner é separado), mas SUPORTA tween per-instance via `tween_property(sprite, "per_corner_tint[0]", ...)`.

Animação canônica de per-corner: mexer os 4 cantos juntos (gradient direction shift) via interpolação CPU-side antes do extract. Adicionar timeline track por-canto seria over-engineering em v1.

**Nota Lens C M4 — GPU varying interpolation NÃO é deterministic cross-driver:** A bilinear interpolation dos 4 cantos é controlada pelo rasterizer + driver de cada backend wgpu (Vulkan/Metal/DX12 podem ULP-diverge em fragments mid-quad). Por isso `RenderInstance.per_corner_tint` vive em **PresentWorld (HR-5 exempt)**, NÃO em SimWorld. Tests automatizados de tint correctness usam **`compute_final_color` CPU mock pure** (não GPU readback); GPU readback testing é smoke do Enio em PR review, não gate automatizado cross-OS.

## 4.7 Bulk-edit em multi-select

Quando N sprites selecionados:
- **Tint** mostra valor se idênticos; "—" Mixed se divergem.
- **Self Tint** idem.
- **Per-corner** idem por canto.
- **Tint Fill** mostra `?` se misto.
- **Opacity** idem.
- Edit em qualquer campo aplica a TODOS selecionados.

Botão **"Reset All Tints"** (resta para WHITE × WHITE × [WHITE;4]; tint_fill=false; opacity=1.0) é destruidor — confirmação obrigatória (HR-11 spirit).

## 4.8 OKLCH color picker — estender `BlenderColorPicker` existente (D9)

Color picker do PH2D usa **OKLCH** (perceptually uniform). Vantagens documentadas:
- L = lightness perceptual constante (slide L sem mudar percepção de cor).
- C = chroma constante (cor satura sem virar).
- H = hue uniforme (cores espaçadas regularmente são perceptualmente espaçadas).

Diferencial: nenhum engine (Unity, Godot, Unreal, Phaser, etc.) tem OKLCH picker. Web tem desde 2023 (Chrome/Edge/Safari/Firefox). PH2D = primeira engine.

**Implementação (corrigido pós-Lens-D D9):** **estender `BlenderColorPicker` existente** ([crates/ph2d-editor-core/src/widget/blender_color_picker/](../../crates/ph2d-editor-core/src/widget/blender_color_picker/) — 1790 LOC em 10 sub-files; `state.rs:9` já comenta "OKLCH-perceptually uniform (Blender's default)"). **Math OKLCH já existe** em [crates/ph2d-color/src/oklab.rs](../../crates/ph2d-color/src/oklab.rs) + [`src/oklch.rs`](../../crates/ph2d-color/src/oklch.rs) (RGB → OKLab → OKLCH conversion completa).

**W6.T6.1 escopo correto:**
1. Expor OKLCH como **modo selecionável** diretamente em `BlenderColorPicker.mode` enum (não só RGB→OKLCH internal).
2. Alterar `state.rs` + `wheel.rs` para emitir `ph2d_color::OklchColor` como output canônico via `ph2d_tokens::ColorValue`.
3. Adicionar input field "L · C · H · A" no chrome do picker.
4. AccessKit role: `ColorWell` com label dinâmico "OKLCH picker, L X.XX, C Y.YY, H Z°".

**NÃO reinventar:** spec original dizia "Implementação em `crates/ph2d-editor-core/src/widget/color_picker.rs` (a criar em W6)" — incorreto. Reinventar = drift do widget consolidado + 1790 LOC desperdiçados + risk de divergir math de tokens.

## 4.9 Animatable property paths

Todos os 4 canais são animáveis via timeline (módulo Animation futuro). Property paths canônicos:

- `sprite.tint`
- `sprite.tint:r`, `sprite.tint:g`, `sprite.tint:b`, `sprite.tint:a` (sub-properties)
- `sprite.self_tint` + sub-properties
- `sprite.per_corner_tint[0..3]` (per-canto)
- `sprite.opacity`
- `sprite.tint_fill` (boolean toggleable)

Padrão de path bate com Godot (`modulate:a`, `material:shader_parameter/foo`). Permite fade-out só de alpha sem mexer RGB:

```
animate sprite.opacity 1.0 → 0.0 over 2s
```

## 4.10 Caps gateados

Arch-gate `architecture_sprite_inspector_surface`:

| Cap | Valor |
|---|---|
| Canais de tint independentes no `Sprite` struct | 4 (tint + self_tint + per_corner + opacity) |
| Componentes de per_corner_tint | 4 cantos × 4 floats = 16 floats |
| Range de opacity | `[0.0, 1.0]` (clamp) — razão: opacity é "visibility multiplier" semântico (não HDR exposure). Para HDR boost transitório (bloom), usar `tint.rgb > 1.0` (range `[0.0, +∞)` aceita HDR). Separação semântica preservada — usuário escolhe o canal correto pelo intent. |

Bump exige ADR-0071-amendment-N.

## 4.11 Anti-padrões evitados

1. **Tint como aditivo (Add) por default** ❌ — Unity teve esse bug histórico em alguns shaders custom. PH2D enforce multiplicativo.
2. **Self Tint como propriedade de Component opcional** ❌ — Component opcional força ramificação semântica ("Sprite com self_tint=WHITE explícito" ≠ "Sprite sem Component"). Como WHITE é identidade multiplicativa, campo sempre presente é simpler e zero overhead.
3. **Per-corner via 4 sprites filhos com tint flat** ❌ — 4 draw calls; resolução pixel-aware diferente; PH2D usa vertex color (1 draw call).
4. **Opacity dentro de tint.a** ❌ — animar opacity preserving RGB exige patch path tint:a; campo separado é estritamente mais expressivo.
5. **Ordem de multiplicação ambígua (sometimes additive)** ❌ — PH2D fixa multiplicativo em TODOS os canais. Sem exceções. Sem flag "blend tint as add".

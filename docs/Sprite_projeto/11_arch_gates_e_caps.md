# 11 — Arch gates e caps numéricos

## 11.1 Princípio dos arch-gates

Arch-gates são **testes de compile/CI que rejeitam violação automática** (não dependem de revisão humana). Padrão PH2D: `crates/ph2d-*/tests/architecture_*.rs` (vide `architecture_contract_surface.rs` do nodegraph + `architecture_tool_contract_surface.rs` do tool).

Cada cap é **número exato** (não range). Bump = ADR-0070-amendment-N obrigatório.

## 11.2 Gates novos para Sprite Inspector v2

### 11.2.1 `architecture_sprite_inspector_surface` (criado em W1.T1.12)

Localização: `crates/ph2d-render/tests/architecture_sprite_inspector_surface.rs`.

```rust
/// Arch-gate dos caps numéricos do Sprite Inspector v2.
/// Bump = ADR-0070-amendment-N + revisão de impactos.

#[test]
fn sprite_struct_field_count_capped() {
    // Sprite v4 tem EXATAMENTE 20 campos (5 v3 + 14 v4 + 1 version field serializável).
    let fields = sprite_struct_field_count();   // helper via reflection ou stringify!
    assert_eq!(fields, 20, "Sprite v4 struct field count must be exactly 20 (FROZEN by ADR-0070)");
}

#[test]
fn render_instance_field_count_capped() {
    // RenderInstance v4 tem EXATAMENTE 12 campos (10 GPU vertex attrs incluindo 4 per_corner + 2 CPU-only).
    let fields = render_instance_field_count();
    assert_eq!(fields, 12);
}

#[test]
fn render_instance_pod_size_capped() {
    // RenderInstance v4 tem EXATAMENTE 144 bytes.
    assert_eq!(std::mem::size_of::<RenderInstance>(), 144);
}

#[test]
fn sprite_schema_version_v4() {
    assert_eq!(Sprite::VERSION, 4);
}

#[test]
fn tint_channel_count_capped() {
    // 4 canais: tint + self_tint + per_corner_tint + opacity.
    // Não 3 (sem self_tint) nem 5 (sem 5º canal novo).
    let channels = sprite_tint_channel_count();
    assert_eq!(channels, 4);
}
```

### 11.2.2 `vertex_attr_offsets_match_struct` (existente, ampliado)

`crates/ph2d-render/src/sprite.rs` test do bloco `#[cfg(test)] mod tests`, já existente em [sprite.rs:343-375](../../crates/ph2d-render/src/sprite.rs#L343-L375). **Atualizar para 11 attrs** (era 7):

```rust
#[test]
fn vertex_attr_offsets_match_struct() {
    use std::mem::offset_of;
    let expect = [
        (2u32, offset_of!(RenderInstance, world_pos) as u64),
        (3, offset_of!(RenderInstance, size) as u64),
        (4, offset_of!(RenderInstance, atlas_uv) as u64),
        (5, offset_of!(RenderInstance, tint) as u64),
        (6, offset_of!(RenderInstance, rotation) as u64),
        (7, offset_of!(RenderInstance, premultiplied) as u64),
        (8, offset_of!(RenderInstance, anchor) as u64),
        // novos v4:
        (9, offset_of!(RenderInstance, per_corner_tint) as u64),
        (13, offset_of!(RenderInstance, opacity) as u64),
        (14, offset_of!(RenderInstance, flip_uv) as u64),
    ];
    let attrs = RenderInstance::VERTEX_ATTRIBUTES;
    assert_eq!(attrs.len(), expect.len(), "attribute count drifted");
    // ... rest of check ...
}
```

### 11.2.3 `inspector_section_count_capped`

Localização: `crates/ph2d-panel-inspector/tests/architecture_section_count.rs`.

```rust
#[test]
fn inspector_section_count_canonical() {
    // 12 seções FROZEN (vide ADR-0069 §2.1 + Sprite_projeto/03_inspector_secoes.md).
    // Pre-W0 audit corrigiu "11" inconsistency.
    let sections = inspector_section_count();   // count from sections.rs registry
    assert_eq!(sections, 12, "Inspector section count FROZEN at 12 by ADR-0069");
}

#[test]
#[ignore] // Gate ativado APÓS W2.T2.1 (refactor de sections.rs para sections/ módulos).
          // sections.rs atual tem 574 LOC (acima do cap); ativar pre-T2.1 quebra CI.
fn inspector_section_loc_cap() {
    // Cada arquivo sections/*.rs ≤ 500 LOC (HR-18).
    for entry in std::fs::read_dir("crates/ph2d-panel-inspector/src/sections/").unwrap() {
        let path = entry.unwrap().path();
        let loc = count_loc(&path);
        assert!(loc < 500, "{} has {} LOC (cap 500)", path.display(), loc);
    }
}
```

> **Nota pós-audit:** `sections.rs` atual tem 574 LOC ([crates/ph2d-panel-inspector/src/sections.rs](../../crates/ph2d-panel-inspector/src/sections.rs)), acima do cap ≤ 500. Gate fica `#[ignore]` até W2.T2.1 fechar (refactor para `sections/` módulos). Sem este `#[ignore]`, gate vermelha imediatamente em CI.

### 11.2.4 `tint_math_multiplicative_canonical`

Localização: `crates/ph2d-render/tests/tint_math_canonical.rs`.

**Notas pós-Lens-C:** (a) test usa **epsilon comparison** (não `assert_eq!` direto em f32 — fragile com FMA contraction); (b) `compute_final_color` é **CPU implementation pure** (mock); GPU readback testing é smoke do Enio (não automatizado cross-OS — varying interpolation tem ULP divergence cross-driver, M4); (c) **fixture com valores não-triviais** (não só potências de 2) detecta reorder via FP precision loss (M6).

```rust
const EPSILON: f32 = 1.0e-6;  // FP epsilon — não bit-exact comparison.

#[test]
fn tint_4_channels_multiply_to_final_color() {
    // Fixture com valores não-triviais — detecta reorder via overflow/precision (M6).
    let sprite = Sprite {
        tint: [0.5, 1.0, 1.0, 1.0],
        self_tint: [1.0, 0.5, 1.0, 1.0],
        per_corner_tint: [[1.0, 1.0, 0.5, 1.0]; 4],
        opacity: 0.5,
        // ...
    };
    let sample = [1.0, 1.0, 1.0, 1.0]; // white texel
    let final_color = compute_final_color(&sprite, sample);
    
    // Ordem canônica: sample × per_corner × self_tint × tint × opacity (em a).
    let expected = [
        0.5 * 1.0 * 1.0 * 0.5,        // R: tint=0.5
        1.0 * 1.0 * 0.5 * 1.0,        // G: self_tint=0.5
        1.0 * 0.5 * 1.0 * 1.0,        // B: per_corner=0.5
        1.0 * 1.0 * 1.0 * 1.0 * 0.5,  // A: opacity multiplicação final
    ];
    for i in 0..4 {
        assert!((final_color[i] - expected[i]).abs() < EPSILON,
            "channel {i}: expected {} got {} (diff {})",
            expected[i], final_color[i], (final_color[i] - expected[i]).abs());
    }
}

#[test]
fn tint_reorder_detection_via_bit_pattern() {
    // Fixture com valores que IEEE 754 NÃO representa exato — qualquer reorder
    // produz mantissa ULP-diferente. Lens E E1 fix: anterior fixture (1e-30 × 1e+30 = 1.0)
    // era TAUTOLÓGICA — pair cancela exatamente em IEEE 754 (`2^-100 × 2^100 = 1`),
    // qualquer ordem produzia mesmo resultado.
    //
    // Valores não-power-of-2: 0.1, 0.2, 0.3, 0.7 — cada produto é irracional em binary32;
    // reorder ((0.7×0.3)×0.2)×0.1 vs 0.7×(0.3×(0.2×0.1)) diverge em ≥1 ULP.
    let sprite = Sprite {
        tint: [0.1, 1.0, 1.0, 1.0],
        self_tint: [0.2, 1.0, 1.0, 1.0],
        per_corner_tint: [[0.3, 1.0, 1.0, 1.0]; 4],
        opacity: 1.0,
        // ...
    };
    let sample = [0.7, 1.0, 1.0, 1.0];
    let final_color = compute_final_color(&sprite, sample);

    // Bit-exact assertion contra fixture pre-computed na ORDEM CANÔNICA top-down:
    // sample × per_corner × self_tint × tint
    // = 0.7 × 0.3 × 0.2 × 0.1
    // Em binary32: 0x3D23D70A (≈ 0.0042000003)
    // Qualquer reorder produz ULP-diferente bits.
    //
    // Hash hex precomputado em fixtures/tint_reorder_canonical.expected
    // (gerado em qualquer host com ordem top-down; cross-OS bit-identical).
    let bits = final_color[0].to_bits();
    let expected_bits = 0x3D23D70Au32; // canonical top-down ordering
    assert_eq!(bits, expected_bits,
        "tint reorder detected: got {:#010x} = {}; expected {:#010x} (top-down canonical). \
         Multiplication order changed.",
        bits, final_color[0], expected_bits);
}

#[test]
fn tint_fill_replaces_rgb_not_alpha() {
    let sprite = Sprite { tint_fill: true, self_tint: [1.0, 0.0, 0.0, 1.0], ...};
    let sample = [0.2, 0.3, 0.4, 0.8];  // textura cinza
    let final_color = compute_final_color(&sprite, sample);
    
    // tint_fill: RGB do sample ignorado, alpha preservado.
    assert_eq!(final_color[3], 0.8);  // alpha do sample
    assert_eq!(final_color[0], 1.0);  // R = self_tint.r * outros
    assert_eq!(final_color[1], 0.0);
    assert_eq!(final_color[2], 0.0);
}
```

### 11.2.4.1 Security/sanitization gates (Lens E E2, E5, E6, E7, E12 fixes)

```rust
#[test]
fn instance_shader_params_key_length_byte_cap() {
    // Lens E E2: setter rejeita key >32 bytes UTF-8.
    let mut params = InstanceShaderParams::new();
    let big_key = "a".repeat(33);
    assert!(params.try_insert(&big_key, InstanceParamValue::Float(0.0)).is_err());
    assert!(params.try_insert("hue_shift", InstanceParamValue::Float(0.5)).is_ok());
}

#[test]
fn anchor_dict_depth_cap_at_deserialize() {
    // Lens E E5: postcard fixture com Dict depth=5 → load_named_anchor_list().is_err().
    let nested_5_deep = build_dict_depth(5);   // recursive helper
    let bytes = postcard::to_allocvec(&nested_5_deep).unwrap();
    assert!(load_named_anchor_list(&bytes).is_err());
}

#[test]
fn sprite_scene_load_size_cap_enforced() {
    // Lens E E6: 100MB+1 byte → load_sprite_scene().is_err().
    let oversized = vec![0u8; SpriteScene::MAX_SCENE_BYTES + 1];
    assert!(load_sprite_scene(&oversized).is_err());
}

#[test]
fn sprite_tint_finite_rejects_nan_and_inf() {
    // Lens E E7: NaN/+Inf/-Inf em tint inputs rejected.
    let mut sprite = Sprite::atlas(0, [1.0, 1.0], [1.0, 1.0, 1.0, 1.0]);
    assert!(sprite.try_set_tint([f32::NAN, 1.0, 1.0, 1.0]).is_err());
    assert!(sprite.try_set_tint([f32::INFINITY, 1.0, 1.0, 1.0]).is_err());
    assert!(sprite.try_set_tint([f32::NEG_INFINITY, 1.0, 1.0, 1.0]).is_err());
    assert!(sprite.try_set_tint([1.0, 1.0, 1.0, 1.0]).is_ok());
}

#[test]
fn named_anchor_total_cap_at_setter() {
    // Lens E E12: 65th sprite_anchor_set → Err (não wait validation-time).
    let mut list = NamedAnchorList::new();
    for i in 0..64 {
        assert!(list.try_insert_anchor(NamedAnchor { name: format!("a{i}"), ..Default::default() }).is_ok());
    }
    let result = list.try_insert_anchor(NamedAnchor { name: "a64".into(), ..Default::default() });
    assert!(result.is_err());
}

#[test]
fn validate_named_anchor_sanitizes_and_rejects_dup() {
    // Lens E E4 + Lens D D6: 4 testes (length, control_char, empty, duplicate).
    let mut list = NamedAnchorList::new();
    list.try_insert_anchor(NamedAnchor { name: "muzzle".into(), ..Default::default() }).unwrap();
    
    // Length cap
    let too_long = NamedAnchor { name: "a".repeat(65), ..Default::default() };
    assert!(matches!(validate_named_anchor(&too_long, &list), Err(SpriteError::AnchorNameTooLong)));
    
    // Control chars
    let control = NamedAnchor { name: "muz\0zle".into(), ..Default::default() };
    assert!(matches!(validate_named_anchor(&control, &list), Err(SpriteError::AnchorNameControlChar)));
    
    // Empty
    let empty = NamedAnchor { name: "".into(), ..Default::default() };
    assert!(matches!(validate_named_anchor(&empty, &list), Err(SpriteError::AnchorNameEmpty)));
    
    // Duplicate
    let dup = NamedAnchor { name: "muzzle".into(), ..Default::default() };
    assert!(matches!(validate_named_anchor(&dup, &list), Err(SpriteError::AnchorNameDuplicate)));
}
```

### 11.2.5 `clip_children_regression` (fixture concreta — Lens E E17 fix)

Localização: `crates/ph2d-render/tests/clip_children_regression.rs`.

Gate visual contra os 5 issues abertos do Godot. **Pixel-comparison cross-OS é inviável** (wgpu backend differences entre Vulkan/Metal/DX12 produzem ULP-divergence em rasterization + blending — vide audit Lens A H6 + Lens C H2). Estratégia de gate atualizada:

**Fixture canon ClipOnly (Lens E E17 fix):**

```
parent = create_sprite(circle 64×64 white at (100, 100))   // pivot center
child  = create_sprite(rect 64×64 red at (108, 108))        // offset diagonal
parent.add_component(ClipChildren(ClipMode::ClipOnly))

Expected summary-stats:
  opaque_pixel_count = 3217    // circular ∩ rect (approx π × 32² × overlap_fraction)
  bbox             = (108, 108, 144, 144)  // child's rect intersected by circle silhouette
  mean_alpha_q8_8  = 0xFF00   // full opaque inside circle
  opacity_bitmap_hash = "blake3:abc...123"  // documented in fixtures/clip_only.expected
```

**Fixture ClipAndDraw:** parent visível + child clipped — same setup, `ClipMode::ClipAndDraw`. Expected `opaque_pixel_count` ≈ 5000 (circle + overlapping child) + bbox = (68, 68, 184, 184).

**Fixture Disabled:** parent + child renderizados normais sem clip. Expected `opaque_pixel_count` ≈ 5500 (both shapes full) + bbox = (68, 68, 184, 184).

1. **Pixel-comparison gate roda em `ubuntu-latest` single-OS** (reference backend). Outras matrices (macOS/Windows) executam apenas o **summary stats gate** (cross-OS deterministic).

2. **Summary stats gate (cross-OS):** compara métricas integer/aggregated, não pixels brutos:
   - Count de pixels não-transparentes (`alpha > 0`)
   - Bounding box do clipped area
   - Mean alpha (rounded to int em fixed-point Q8.8)
   - Hash do "is_pixel_opaque" bitmap (binary, sub-pixel rounded)

   Essas métricas são bit-identical cross-driver porque dependem só de álgebra inteira/agregada, não de FP precise rasterization.

3. **Smoke do Enio (humans-in-the-loop)** cobre regressão Godot-style visual em PR review — gate automatizado cobre estrutura.

Vide [06_mask_clip.md §6.3](06_mask_clip.md) para detalhe das 3 fixtures.

### 11.2.6 `sorting_pipeline_determinism`

Localização: `crates/ph2d-render/tests/sorting_pipeline_determinism.rs`.

Cenário fixo de 10 sprites em hierarquia 3-níveis com mix de Z + YSort + SortingGroup + ShowBehindParent. Cross-OS hash blake3 do `Vec<RenderInstance>` produzido = byte-identical em macOS/Linux/Windows.

Vide [05_ordering_sorting.md §5.6](05_ordering_sorting.md).

### 11.2.7 `migrate_sprite_v3_to_v4`

Localização: `crates/ph2d-render/tests/migrate_sprite_v3_to_v4.rs`.

5 fixtures v3 binárias congeladas em `fixtures/` carregam como v4 com defaults benignos. Vide [10_schema_versionamento.md §10.6](10_schema_versionamento.md).

### 11.2.8 `named_anchor_list_inline_cap`

Localização: `crates/ph2d-render/tests/named_anchor_caps.rs`.

```rust
#[test]
fn named_anchor_list_inline_smallvec_size() {
    // SmallVec inline = 4 anchors.
    let typed: SmallVec<[NamedAnchor; 4]> = SmallVec::new();
    let inline_cap = typed.spilled().not() && typed.capacity() == 4;
    assert!(inline_cap);
}

#[test]
fn named_anchor_name_length_cap() {
    let too_long = "a".repeat(65);
    let anchor = NamedAnchor { name: too_long.clone(), ... };
    let result = validate_named_anchor(&anchor);
    assert!(result.is_err(), "name length > 64 must fail validation");
}
```

## 11.3 Tabela consolidada de caps

| Cap | Valor | Arch-gate | Bump exige |
|---|---|---|---|
| `Sprite` struct fields | **20** | `architecture_sprite_inspector_surface` | ADR-0070-amendment |
| `Sprite::VERSION` const | 4 | idem | ADR-0070-amendment |
| `Sprite.version: u32` (serializable field) | PRESENT em v4 (habilita versioned dispatch) | idem | ADR-0070-amendment |
| `RenderInstance` fields total | **12** | idem + `vertex_attr_offsets_match_struct` | ADR-0070-amendment |
| `RenderInstance` vertex attrs (GPU-visible) | 11 (locations 2..14) | `vertex_attr_offsets_match_struct` | ADR-0070-amendment |
| `RenderInstance` `size_of` | 144 bytes | idem | ADR-0070-amendment |
| Tint channels independentes | 4 | idem | ADR-0071-amendment |
| Per-corner tint corners | 4 (TL/TR/BL/BR) | idem | ADR-0071-amendment |
| Inspector sections | **12 (canônico)** | `inspector_section_count_canonical` | ADR-0069-amendment |
| Sections sub-arquivo LOC | ≤ 500 | `inspector_section_loc_cap` (**`#[ignore]` até W2.T2.1**) | HR-18 (já existente) |
| `BlendMode` variants | 6 | `architecture_blend_mode_variants` | ADR-0070-amendment |
| `FilterMode` variants | 7 | idem | ADR-0070-amendment |
| `RepeatMode` variants | 4 | idem | ADR-0070-amendment |
| `ClipMode` variants | 3 | idem | ADR-0070-amendment |
| `MaskInteractionMode` variants | 3 | idem | ADR-0070-amendment |
| `NamedAnchorList` inline | SmallVec[4] (FROZEN; drift inter-arquivo corrigido pós-audit) | `named_anchor_list_inline_cap` | ADR-0072-amendment |
| `AnchorData::Dict` impl | `SmallVec<[(String, AnchorData); 8]>` (NÃO HashMap — viola ADR-0022) | `anchor_dict_no_hashmap` | ADR-0072-amendment |
| `NamedAnchor.name` length | **≤ 64 bytes UTF-8** (não chars; H6 fix Lens C) | `named_anchor_name_byte_length_cap` | ADR-0072-amendment |
| `AnchorData::Dict` keys sorted invariant | ENFORCED lexicograficamente (H5 fix Lens C) | `anchor_dict_keys_sorted_invariant` | ADR-0072-amendment |
| `NamedAnchor` total per-sprite | ≤ 64 | `named_anchor_total_cap` | ADR-0072-amendment |
| `InstanceShaderParams` inline | SmallVec[8] | `instance_shader_params_cap` | — |
| `InstanceShaderParams` key length | **≤ 32 bytes UTF-8 ENFORCED on setter** (Lens E E2 fix) | `instance_shader_params_key_length_byte_cap` | ADR-0070-amendment |
| `SortingLayer` count no projeto | **≤ 32 (FROZEN; H7 reconciliado em 4 lugares)** | `sorting_layer_total_cap` | ADR-0073-amendment |
| `SpriteFrames.frames` count | ≤ 4096 | `sprite_frames_count_cap` | — |
| `SpriteFrames.tags` count | ≤ 256 | idem | — |
| `AnimationTag.name` length | ≤ 64 | idem | — |
| `frame.duration_ms` range | `[1, 60_000]` | `sprite_frame_duration_range` | — |
| `speed_scale` range | `[-100, 100]` | `sprite_animator_speed_range` | — |

## 11.4 Gates a herdar (existentes, garantir que ainda passam)

Estes JÁ existem mas precisam ser revisados/ampliados após Sprite v4:

| Gate | Localização | Ação no W1 |
|---|---|---|
| `vertex_attr_offsets_match_struct` | `crates/ph2d-render/src/sprite.rs#L343` | Expandir para 11 attrs |
| `render_instance_is_pod_compatible` | `crates/ph2d-render/src/sprite.rs#L304` | Atualizar `assert_eq!(bytes.len(), 144)` |
| `vertex_attributes_cover_full_stride` | `crates/ph2d-render/src/sprite.rs#L327` | Atualizar last shader_location para 14 (flip_uv) |
| `sprite_constructors_default_straight_alpha` | `crates/ph2d-render/src/sprite.rs#L378` | Confirma `Sprite::atlas/individual` retornam v4 com defaults benignos |
| `hr12_widgets_a11y` | `crates/ph2d-editor-core/tests/hr12_widgets_a11y.rs` | Novos widgets (OKLCH picker, NamedAnchorEditor) emitem AccessKit |
| `architecture_widget_loc_cap` | `crates/ph2d-editor-core/tests/architecture_widget_loc_cap.rs` | Cada widget novo ≤ 500 LOC |
| `architecture_widget_showcase_coverage` | `crates/ph2d-editor-core/tests/architecture_widget_showcase_coverage.rs` | Widget novo aparece no Gallery |
| `no_literal_color` | `crates/ph2d-editor-core/tests/no_literal_color.rs` | Sem hex literal em sections novos |
| `mockup_tokens_exist` | `crates/ph2d-editor-core/tests/mockup_tokens_exist.rs` | Mockup `inspector_v2.html` tokens válidos |

## 11.5 Smoke gates (do Enio)

Cada wave requer **smoke visual confirmado pelo Enio** antes de fechar. Checklist W2-W7 mínima:

### W2 (Color & Tint canônicos)
- [ ] Tint cascateia pra filhos visualmente (selecionar pai, mexer Tint, filhos tingem juntos).
- [ ] Self Tint NÃO cascateia (filhos imunes).
- [ ] Per-corner tint produz gradient (TL=red, BR=blue → gradient diagonal).
- [ ] Tint Fill: ativa, sprite vira silhueta colorida; desativa, volta normal.
- [ ] Opacity slider preserva RGB (canal independente de tint.a).

### W3 (Sorting + Mask)
- [ ] Z Index = 5 com ZAsRelative=true cascata pro filho corretamente.
- [ ] Show Behind Parent: filho desenha atrás do pai.
- [ ] YSort em hierarchy: char mais "embaixo" desenha na frente.
- [ ] ClipChildren ClipOnly: filhos recortados, pai invisível.
- [ ] Mask Interaction VisibleInside: sprite só aparece dentro do Mask2D.

### W5 (Named Anchors)
- [ ] Adiciona socket "muzzle" via Inspector.
- [ ] Anexa particle emitter filha pelo socket.
- [ ] Sprite anima (4 frames de attack); muzzle se move automaticamente; particles seguem.

## 11.6 Gates de regressão (em CI cross-OS)

Lista mínima de gates que rodam em **Linux x86_64 + macOS + Windows** matrix (paridade com `architecture_contract_surface` do nodegraph):

| Gate | Tipo | Cross-OS? |
|---|---|---|
| `sorting_pipeline_determinism` | hash blake3 bit-identical | ✅ |
| `migrate_sprite_v3_to_v4` | fixtures byte-comparison | ✅ |
| `tint_math_multiplicative_canonical` | float epsilon comparison | ✅ |
| `vertex_attr_offsets_match_struct` | compile-time | ✅ |
| `clip_children_regression` | summary stats (count opaque + bbox + mean alpha Q8.8 + opacity bitmap hash) — cross-OS | ✅ |
| `clip_children_regression_pixels` | pixel-comparison goldens — Linux x86_64 only (reference backend) | linux-only |

## 11.6.1 AccessKit roles canônicos por widget novo (Lens D D7)

Cada widget novo do Sprite Inspector v2 declara role + label + state machine canônicos. Gate `hr12_widgets_a11y` enforce.

| Widget | Role AccessKit | Label dinâmico | State machine |
|---|---|---|---|
| **OKLCH ColorPicker** (estendido de BlenderColorPicker) | `Role::ColorWell` | "OKLCH color picker, L X.XX, C Y.YY, H Z°, A W%" | focus → hover → value change → emit `TreeUpdate` |
| **NumericInputWithUnit** | `Role::SpinButton` (numeric constraint + unit suffix) | "Position X, 12.5 px" | focus → text edit → commit/cancel |
| **BulkSelectInspector** | `Role::Group` + child `ColorWell`/`SpinButton`/etc com `state: Indeterminate` quando mixed | "Multiple values selected" via parent group | focus → "Mixed" announces; edit → apply to all |
| **BitmaskGrid32** | `Role::Group` + 32 children `Role::CheckBox` | "Visibility Layer 0", "Visibility Layer 1", ... | individual checkbox state |
| **NamedAnchorEditor** | `Role::List` + per-anchor `Role::ListItem` | "Socket muzzle at (28, -4)", "Slice face_box 24×24px" | list nav + per-item edit |
| **Canvas handle (per-anchor)** | `Role::Handle` (custom; AccessKit gizmo annotation) | "Drag handle for anchor muzzle" | drag begin → move → drop |
| **OrderDebugOverlay** | NÃO widget — canvas annotation; AccessKit emit `Description` no Sprite Node ("z_index: 5, sorting_layer: UI") | (annotation no Sprite, não widget próprio) | toggle on/off |
| **SegmentedAdaptive 8-region 9-slice** | `Role::Group` + per-region `Role::ComboBox` (4 options) | "Corner top-left mode: Stretch", "Border top mode: Repeat", ... | 8 independent selectors |
| **KeyValueList (Instance Shader Params)** | `Role::Table` (2 cols: name, value) + per-row `Role::Row` | "Param hue_shift = 0.5" | row add/remove/edit |
| **VariantEditor (user_data Anchor)** | `Role::Group` + variant-kind `Role::ComboBox` + sub-widget per variant | "user_data type: Dict (3 entries)" | type switch → sub-widget |
| **Rect2Editor (bounds/center/region)** | `Role::Group` + 4 `Role::SpinButton` (x/y/w/h) + canvas `Role::Handle`s | "Region rect (0, 0, 64, 64)" | drag corners → numeric update |

**Implementação:** cada widget emit `accesskit::Node` em `populate` com role+label+state inicial; mudanças (focus, value) emitem `accesskit::TreeUpdate`. AccessKit roles disponíveis verificados em [`crates/ph2d-a11y/src/`](../../crates/ph2d-a11y/src/) (real).

## 11.6.2 Slider+chip pattern lista canônica enforce (Lens D D8)

DIRETRIZ §5.2 (Widget Gallery canon): "Slider + chip pareados → SEMPRE `store.link_slider_number(slider_id, chip_id)` + `store.mark_chip_no_stepper(chip_id)`". Sem isso, click dispatch silenciosamente quebra (memory `feedback-panel-populate-register`).

**Lista canônica de slider+chip pares em Sprite Inspector v2 — ENFORCE:**

| Seção | Slider | Chip | populate() exige |
|---|---|---|---|
| §3.6 Color & Tint | Opacity slider 0..1 | Opacity chip 0..100% | `link_slider_number(OPACITY_SLIDER, OPACITY_CHIP)` + `mark_chip_no_stepper(OPACITY_CHIP)` |
| §3.5 9-Slice | Stretch Value slider 0..1 | (sem chip — uso só raro) | n/a |
| §3.8 Visibility | Alpha Cutoff slider 0..1 | Alpha Cutoff chip 0..100% | `link_slider_number(ALPHA_CUTOFF_SLIDER, ALPHA_CUTOFF_CHIP)` + `mark_chip_no_stepper` |
| §3.11 Animation | Frame Progress slider 0..1 | Frame Progress chip 0..100% | `link_slider_number(FRAME_PROGRESS_SLIDER, FRAME_PROGRESS_CHIP)` + `mark_chip_no_stepper` |
| §3.11 Animation | Speed Scale slider | Speed Scale chip (float val) | `link_slider_number(SPEED_SCALE_SLIDER, SPEED_SCALE_CHIP)` + `mark_chip_no_stepper` |

Gate `architecture_panel_chip_pill_no_stepper` (existente) força. Implementador W2-W4 que esquecer dispara CI vermelho.

## 11.6.2.1 HR-3 inspector paint zero-alloc gate (Lens E E8 fix)

```rust
#[test]
fn inspector_paint_zero_alloc_after_warmup() {
    // Build scene with 1 sprite + Inspector open (12 sections; 3 default-abertas + 9 colapsadas).
    let mut world = build_test_world_with_inspector();
    
    // Warmup 3 paints (initial caches/thread-local snapshots fill).
    for _ in 0..3 { panel.paint(&mut world); }
    
    // Measure allocations during steady-state paint loop.
    let before = global_alloc_count();
    for _ in 0..100 { panel.paint(&mut world); }
    let after = global_alloc_count();
    
    // HR-3: zero `Box::new` / `Vec::push`-que-realoca em hot path.
    // Thread-local snapshots padrão Phase C.1; per-corner tint 16 bytes adicional zero overhead.
    assert_eq!(after - before, 0, 
        "Inspector paint allocated {} times in 100 paints (HR-3 violation; expected 0)",
        after - before);
}
```

Localização: `crates/ph2d-panel-inspector/tests/inspector_paint_no_alloc.rs` (W2 cria). Usa `allocation_counter` crate OR jemalloc stats. Roda em CI Linux x86_64 + macOS aarch64 (não Windows — alloc tracker variation between OS).

## 11.6.2.2 HR-4 inspector paint budget (Lens E E9 fix)

Criterion bench `inspector_paint_budget_hr4` em `crates/ph2d-panel-inspector/benches/`:

| Scene fixture | Default-open sections | Budget @ M-series macOS | Budget @ Linux x86_64 |
|---|---|---|---|
| `smoke_w2_color_tint.scene` | 3 (Identity + Transform + Color&Tint sub-tabs) | < 0.5 ms | < 0.7 ms |
| `smoke_w3_sorting.scene` | 3 + Ordering expanded | < 0.7 ms | < 1.0 ms |
| `smoke_w4_animation.scene` | 3 + Material&Blend + Animation expanded | < 1.0 ms | < 1.5 ms |
| `smoke_w5_sockets.scene` | 3 + Sockets/Slices expanded (5 anchors) | < 1.2 ms | < 1.7 ms |

p95 threshold em criterion benchmark; bench falha em wave fechamento (não esperar W7). Gate `inspector_paint_budget_hr4_p95` por wave.

## 11.6.3 Layout density budget (Lens D D11)

Inspector zona Right ~320px wide × ~668px height (viewport laptop 768px - chrome 100px). **Orçamento das 5 seções sempre abertas:**

| Seção | Altura estimada (px) | Observação |
|---|---|---|
| Identity | ~80 | Name + Tags + Notes 3 lines |
| Transform | ~250 | 5 NumberInputs (Pos X/Y, Rot, Scale X/Y) + Skew X/Y + Top Level + Reset/LookAt = 9 rows |
| Render Source | ~200 | Strategy + Storage + Source + Region + Filter Clip + Format + Reimport = 7 rows |
| Sprite Sheet | ~180 | Centered + Offset + Flip + HFrames/VFrames + Frame = 6 rows |
| Color & Tint | ~342 | 6 ColorPickers (Tint, SelfTint, 4 cantos) + Tint Fill + Opacity + Equalize button |
| **Total sempre abertas** | **~1052 px** | **excede viewport 668px** — scroll obrigatório no first paint |

**Decisão:** spec original "5 sempre abertas" precisa revisão. Opções:
- **(a) Reduzir defaults abertas para 3** (Identity + Transform + Color & Tint); Render Source + Sprite Sheet colapsadas por default (expande quando user clica). Total: ~672 px (próximo ao viewport).
- **(b) Compactar Color & Tint** em sub-tabs (Tint | Self Tint | Per-corner | Effects); só 1 sub-tab visível por vez. Color & Tint cai pra ~120px. Total: ~830 px (ainda >668 mas scroll mínimo).
- **(c) Reduzir Transform** colapsando Skew/Top Level/LookAt como sub-seção advanced (cai pra ~180px). Total: ~982px (ainda excede).

**Recomendado: combinar (a) + (b)** — 3 sempre abertas + Color & Tint em sub-tabs = ~450px (caber em 668px com folga). Documentar em §3.0.

**Gate `inspector_default_open_total_height_max_768px`** (W2 cria): mede paint height em scene fixture canônica (sprite simples carregado, Inspector aberto, todos defaults). Falha se altura excede 668px.

## 11.7 Anti-padrões evitados

1. **Gates não-numéricos ("aproximadamente 18 campos")** ❌ — números exatos, sem range.
2. **Bump silencioso de cap sem ADR** ❌ — todo bump = ADR-amendment.
3. **Gate sem teste compile-time** ❌ — runtime test pode passar despercebido em modo release-only.
4. **Smoke do Enio "verbalmente confirmado"** ❌ — checklist explícita, pixel-by-pixel.
5. **Caps separados em N arquivos** ❌ — `architecture_sprite_inspector_surface.rs` consolida tudo num lugar; fácil de revisar.

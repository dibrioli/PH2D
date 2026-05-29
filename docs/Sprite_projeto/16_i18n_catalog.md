# 16 — i18n strings catalog (HR-15 Fluent)

> **Lens D D3 fix:** spec original deixava W7.T7.1 "Fluent strings i18n para todas as labels (sprite.section.*)" sem catalog explícito → Implementador improvisaria bundle layout, naming, e drift entre en-US/pt-BR.

## 16.1 Bundle path canônico

**Foundational widgets** (OKLCH ColorPicker, NumericInputWithUnit, BulkSelectInspector, NamedAnchorEditor, OrderDebugOverlay, SegmentedAdaptive, KeyValueList, VariantEditor, Rect2Editor) → genérico em `editor-core`:

```
crates/ph2d-editor-core/locales/{en-US,pt-BR}.ftl
```

**Sprite Inspector-specific** (section labels, field labels, tooltips, error messages, button labels) → no panel crate:

```
crates/ph2d-panel-inspector/locales/{en-US,pt-BR}.ftl
```

Bundles loaded via `ph2d_i18n::Bundle::load_panel("inspector")` em runtime (`ph2d-i18n` é stub atualmente — vide [feedback-no-industrial-claims-without-verification]; quando Fluent runtime amadurece, gates ativam).

## 16.2 Naming convention canônica

```
sprite.section.<slug>         section labels
sprite.field.<slug>           field labels
sprite.tooltip.<slug>         tooltips
sprite.error.<slug>           error messages
sprite.button.<slug>          button labels
sprite.placeholder.<slug>     placeholders ("Mixed", "—", etc.)
sprite.unit.<slug>            unit suffixes (px, m, deg, rad, %)
sprite.option.<enum>.<value>  enum/dropdown options
```

## 16.3 Strings catalog completo

### Section labels (12)

| Key | en-US | pt-BR |
|---|---|---|
| `sprite.section.identity` | "Identity" | "Identidade" |
| `sprite.section.transform` | "Transform" | "Transform" |
| `sprite.section.render_source` | "Render Source" | "Fonte de Render" |
| `sprite.section.sprite_sheet` | "Sprite Sheet" | "Sprite Sheet" |
| `sprite.section.nine_slice` | "9-Slice" | "9-Slice" |
| `sprite.section.color_tint` | "Color & Tint" | "Cor & Tint" |
| `sprite.section.ordering` | "Ordering / Sorting" | "Ordenação" |
| `sprite.section.visibility` | "Visibility" | "Visibilidade" |
| `sprite.section.sampling` | "Sampling" | "Amostragem" |
| `sprite.section.material_blend` | "Material & Blend" | "Material & Blend" |
| `sprite.section.animation` | "Animation" | "Animação" |
| `sprite.section.sockets_slices` | "Sockets / Slices" | "Sockets / Slices" |

### Field labels Transform (9)

| Key | en-US | pt-BR |
|---|---|---|
| `sprite.field.position_x` | "Position X" | "Posição X" |
| `sprite.field.position_y` | "Position Y" | "Posição Y" |
| `sprite.field.rotation` | "Rotation" | "Rotação" |
| `sprite.field.scale_x` | "Scale X" | "Escala X" |
| `sprite.field.scale_y` | "Scale Y" | "Escala Y" |
| `sprite.field.skew_x` | "Skew X" | "Skew X" |
| `sprite.field.skew_y` | "Skew Y" | "Skew Y" |
| `sprite.field.top_level` | "Top Level" | "Top Level" |
| `sprite.button.reset_transform` | "Reset" | "Resetar" |
| `sprite.button.look_at` | "Look At..." | "Olhar Para..." |

### Field labels Render Source (7)

| `sprite.field.strategy`, `sprite.field.storage_detail`, `sprite.field.source_size`, `sprite.field.region_enabled`, `sprite.field.region_rect`, `sprite.field.region_filter_clip`, `sprite.field.pixel_format`, `sprite.button.reimport` |

### Field labels Sprite Sheet (6)

| `sprite.field.centered`, `sprite.field.offset`, `sprite.field.flip_h`, `sprite.field.flip_v`, `sprite.field.hframes`, `sprite.field.vframes`, `sprite.field.frame`, `sprite.field.frame_coords` |

### Field labels 9-Slice (8)

| `sprite.field.draw_mode`, `sprite.field.slice_borders_l/t/r/b`, `sprite.field.slice_size`, `sprite.field.tile_mode`, `sprite.field.stretch_value`, `sprite.field.fill_center`, `sprite.field.per_region_tile_mode` |

### Field labels Color & Tint (8)

| `sprite.field.tint`, `sprite.field.self_tint`, `sprite.field.per_corner_tl/tr/bl/br`, `sprite.field.tint_fill`, `sprite.field.opacity`, `sprite.button.equalize_corners`, `sprite.button.reset_all_tints` |

### Field labels Ordering / Sorting (11)

| `sprite.field.z_index`, `sprite.field.z_as_relative`, `sprite.field.show_behind_parent`, `sprite.field.sorting_layer`, `sprite.field.order_in_layer`, `sprite.field.y_sort_enabled`, `sprite.field.y_sort_point`, `sprite.field.y_sort_custom_axis`, `sprite.field.sorting_group`, `sprite.field.sort_at_root`, `sprite.field.translucency_priority`, `sprite.field.translucency_distance_offset`, `sprite.field.order_debug_overlay` |

### Field labels Visibility (6)

| `sprite.field.visible`, `sprite.field.visibility_layer`, `sprite.field.clip_children_mode`, `sprite.field.mask_interaction`, `sprite.field.alpha_cutoff`, `sprite.field.on_screen_enabler` |

### Field labels Sampling (3)

| `sprite.field.texture_filter`, `sprite.field.texture_repeat`, `sprite.field.anti_halo` |

### Field labels Material & Blend (4)

| `sprite.field.material`, `sprite.field.use_parent_material`, `sprite.field.instance_shader_params`, `sprite.field.blend_mode` |

### Field labels Animation (11)

| `sprite.field.sprite_frames`, `sprite.field.current_animation`, `sprite.field.frame_anim`, `sprite.field.frame_progress`, `sprite.field.speed_scale`, `sprite.field.playing`, `sprite.field.autoplay`, `sprite.field.direction_override`, `sprite.field.loop_override`, `sprite.field.hold_ms`, `sprite.field.repeat_delay_ms`, `sprite.button.open_in_timeline`, `sprite.button.add_animator` |

### Field labels Sockets / Slices (8)

| `sprite.field.anchor_name`, `sprite.field.anchor_transform`, `sprite.field.anchor_bounds`, `sprite.field.anchor_center`, `sprite.field.anchor_user_data`, `sprite.field.drive_by_frame`, `sprite.button.add_anchor`, `sprite.button.add_slice`, `sprite.button.add_9slice_region`, `sprite.button.remove_slice`, `sprite.button.remove_region` |

### Tooltips para os 8 itens "pequenos com impacto desproporcional" (8)

| Key | en-US (1 frase + 1 exemplo) |
|---|---|
| `sprite.tooltip.self_tint` | "Local tint that doesn't cascade to children. Use for hurt-flash on a single body part without recoloring the whole hierarchy." |
| `sprite.tooltip.z_as_relative` | "When ON (default), Z Index is added to the parent's effective Z. Turn OFF for absolute Z that ignores hierarchy." |
| `sprite.tooltip.show_behind_parent` | "Renders this child BEFORE its parent, behind it visually. Useful for shadows kept as children but drawn underneath." |
| `sprite.tooltip.top_level` | "Breaks the cascade of transform and modulate from parent. Use for floating UI numbers in world-space that shouldn't rotate with the parent." |
| `sprite.tooltip.use_parent_material` | "Children share the parent's material instance. Massive batching win (10k children = 1 material = 1 draw call)." |
| `sprite.tooltip.region_filter_clip` | "Sampler clamps to the region rect, preventing texture bleed from atlas neighbors. Default ON for Atlas, OFF for Individual textures." |
| `sprite.tooltip.tint_fill` | "When ON, RGB of the sprite texture is IGNORED — only tint colors show through (silhouette mode). Damage flash in 1 toggle." |
| `sprite.tooltip.centered` | "Sprite origin at center (default). When OFF, origin is top-left + offset applies. Useful for foot-anchored sprites in topdown games." |

### Tooltips secundários (~10)

| `sprite.tooltip.clip_children_mode`, `sprite.tooltip.mask_interaction`, `sprite.tooltip.y_sort_enabled`, `sprite.tooltip.sorting_group`, `sprite.tooltip.opacity`, `sprite.tooltip.per_corner_tint`, `sprite.tooltip.blend_mode`, `sprite.tooltip.instance_shader_params`, `sprite.tooltip.order_debug_overlay`, `sprite.tooltip.skew_xy` |

### Enum options (~30)

| `sprite.option.clip_mode.disabled / clip_only / clip_and_draw` (3)
| `sprite.option.mask_interaction.none / visible_inside / visible_outside` (3)
| `sprite.option.draw_mode.simple / sliced / tiled` (3)
| `sprite.option.tile_mode.continuous / adaptive` (2)
| `sprite.option.per_region_tile.stretch / repeat / mirror / blank_repeat` (4)
| `sprite.option.texture_filter.inherit / nearest / linear / nearest_mipmap / linear_mipmap / nearest_aniso / linear_aniso` (7)
| `sprite.option.texture_repeat.inherit / disabled / enabled / mirror` (4)
| `sprite.option.blend_mode.mix / add / sub / mul / screen / premult_alpha` (6)
| `sprite.option.direction.forward / reverse / pingpong / pingpong_reverse` (4)
| `sprite.option.sort_point.center / pivot / custom` (3)
| `sprite.option.strategy.atlas / individual / handpacked` (3)
| `sprite.option.pixel_format.rgba8 / rgba16` (2)

### Error messages (~10)

| `sprite.error.name_too_long` | "Name exceeds 64 bytes UTF-8" |
| `sprite.error.anchor_name_duplicate` | "Anchor name '{$name}' already exists in this sprite" |
| `sprite.error.dict_depth_exceeded` | "Dict depth exceeds 4 levels (anti-DoS limit)" |
| `sprite.error.dict_keys_too_many` | "Dict has more than 32 keys per level" |
| `sprite.error.frame_index_out_of_range` | "Frame index {$idx} out of range [0, {$max})" |
| `sprite.error.region_rect_invalid` | "Region rect must have w > 0 and h > 0" |
| `sprite.error.opacity_out_of_range` | "Opacity must be in [0.0, 1.0]" |
| `sprite.error.cap_exceeded` | "Cannot add more — limit {$cap} reached" |
| `sprite.error.material_param_unknown` | "Shader does not declare uniform '{$name}'" |
| `sprite.error.sprite_frames_full` | "SpriteFrames has reached 4096 frames cap" |

### Placeholders (~5)

| `sprite.placeholder.mixed` | "—" (BulkSelect Mixed value) |
| `sprite.placeholder.empty_name` | "(unnamed)" |
| `sprite.placeholder.no_anchors` | "(no anchors)" |
| `sprite.placeholder.coming_soon` | "Coming soon (W{$wave})" |
| `sprite.placeholder.notes_hint` | "Notes about this sprite (purpose, level role, etc.)" |

### Unit suffixes (5)

| `sprite.unit.px` / `sprite.unit.m` / `sprite.unit.deg` / `sprite.unit.rad` / `sprite.unit.percent` |

## 16.4 Total estimado

- 12 section labels
- ~75 field labels
- ~18 tooltips (8 críticos + ~10 secundários)
- ~30 enum options
- ~10 error messages
- ~5 placeholders
- 5 unit suffixes
- **Total: ~155 keys** (estimativa para implementação W7).

Bundle size estimado: ~6KB per language em UTF-8 Fluent compactado.

## 16.5 Gate `sprite_inspector_i18n_keys_present` + content validation (Lens E E16 fix)

**W7.T7.1 cria** (Lens D D3 + Lens E E16 fix obrigatórios):

```rust
#[test]
fn sprite_inspector_all_i18n_keys_present_and_consistent() {
    let en = load_bundle("en-US");
    let pt = load_bundle("pt-BR");
    let canonical_keys = include_str!("../canonical_keys.txt").lines();
    
    // (a) Presença em ambos bundles.
    for key in canonical_keys {
        assert!(en.has_message(key), "key '{key}' missing in en-US");
        assert!(pt.has_message(key), "key '{key}' missing in pt-BR");
    }
    
    // (b) Content validation cross-bundle (Lens E E16):
    for key in canonical_keys {
        let en_msg = en.get_message(key).unwrap();
        let pt_msg = pt.get_message(key).unwrap();
        
        // Non-empty
        assert!(!en_msg.value().is_empty(), "key '{key}' has empty value in en-US");
        assert!(!pt_msg.value().is_empty(), "key '{key}' has empty value in pt-BR");
        
        // Argument interpolation `{$arg}` consistent (mesma lista de args em ambas languages).
        let en_args = extract_fluent_args(en_msg.value());
        let pt_args = extract_fluent_args(pt_msg.value());
        assert_eq!(en_args, pt_args,
            "key '{key}' args drift: en-US uses {en_args:?} but pt-BR uses {pt_args:?}");
        
        // Plural forms consistent (se key tem `{ $count -> ... }`, ambos têm).
        let en_has_plural = en_msg.value().contains("->");
        let pt_has_plural = pt_msg.value().contains("->");
        assert_eq!(en_has_plural, pt_has_plural,
            "key '{key}' plural form drift");
    }
}
```

`canonical_keys.txt` é arquivo single-source-of-truth gerado deste catalog. PR W7 que adiciona key nova adiciona em ambos bundles + canonical list (gate falha senão). Content validation pega: empty string, args drift, plural form mismatch — bug class i18n comum.

## 16.6 Plural rules + arg interpolation

Fluent suporta plurals + args via `{ $arg }`. Casos no Sprite Inspector v2:

- `sprite.error.frame_index_out_of_range` usa `$idx` + `$max`.
- `sprite.error.anchor_name_duplicate` usa `$name`.
- `sprite.placeholder.coming_soon` usa `$wave`.
- `sprite.error.cap_exceeded` usa `$cap`.

Plural exemplo (não usado em v1; reservado pra futuras counts):

```fluent
sprite-frames-count = { $count ->
    [one] {$count} frame
   *[other] {$count} frames
}
```

## 16.7 Estado `ph2d-i18n` runtime

`crates/ph2d-i18n/` é **stub atualmente** (verificado em [editor-core/Cargo.toml](../../crates/ph2d-editor-core/Cargo.toml) comentário "M13 deferred Fluent"). Gate `hr15_no_hardcoded_ui_strings` ([crates/ph2d-editor-core/tests/hr15_no_hardcoded_ui_strings.rs](../../crates/ph2d-editor-core/tests/hr15_no_hardcoded_ui_strings.rs)) atualmente usa string-table workaround.

**W7.T7.1 entrega:**
1. Bundles `.ftl` físicos em paths canônicos (§16.1).
2. Lista canonical keys (`canonical_keys.txt`).
3. Gate `sprite_inspector_i18n_keys_present` ativo (mesmo se Fluent runtime stub).
4. Implementação use `ph2d_i18n::tr("sprite.section.color_tint")` — quando runtime amadurecer, substring lookup ativa transparentemente.

**Pre-requisito da ratificação:** Lens D D3 fix exige catalog (este arquivo) Accepted como parte de W0; lista pode crescer em W7 mas estrutura + path + naming convention são FROZEN agora.

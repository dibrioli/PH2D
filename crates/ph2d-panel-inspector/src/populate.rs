//! Inspector panel `populate` — pre-allocates Inspector-only widget
//! state slots in the `WidgetStore`. Called once at host boot via
//! `Panel::populate`.
//!
//! Other widgets historically registered in the same module (gallery
//! showcase samples, blender color picker, hierarchy chrome handles,
//! global context-menu items, scrollbars) remain in
//! `ph2d_editor_core::screens::hero::pre_populate` because they are
//! shared across panels / chrome layers and are not Inspector-specific.

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore, format_number};
use ph2d_editor_core::widget::{
    ButtonState, CheckboxState, CheckboxValue, DropdownState, SliderOrientation, SliderState,
    TextInputState,
};

pub fn populate(store: &mut WidgetStore) {
    populate_transform_editor(store);
    populate_visibility_editor(store);
    populate_render_strategy(store);
    populate_region(store);
    populate_sprite_flip(store);
    populate_color_tint(store);
    populate_sprite_sheet(store);
    populate_name_editor(store);
    populate_ordering(store);
    populate_sampling(store);
    populate_slice(store);
    populate_anchors(store);
    populate_visibility_section(store);
    populate_blend(store);
    super::populate_physics::populate_physics(store);
    super::populate_physics::populate_joint(store);
    super::populate_physics::populate_wheel(store);
    super::populate_physics::populate_player(store);
}

/// W3 §8 Visibility section: register the segmented + bitmask + toggle ids
/// as `Button`s (is_focusable) and the cutoff/rect NumberInputs. Live
/// values come from the snapshot; defaults match the optional-component
/// "absent" state (cutoff 0.5, rect zero).
fn populate_visibility_section(store: &mut WidgetStore) {
    register_button_ids(store, &ids::INSP_VIS_CLIP);
    register_button_ids(store, &ids::INSP_VIS_MASK);
    register_button_ids(store, &ids::INSP_VIS_LAYER_BIT);
    register_button_ids(store, &[ids::INSP_VIS_MASK_SOURCE, ids::INSP_VIS_ON_SCREEN]);
    for (id, value) in [
        (ids::INSP_VIS_ALPHA_CUTOFF, 0.5_f64),
        (ids::INSP_VIS_RECT_X, 0.0_f64),
        (ids::INSP_VIS_RECT_Y, 0.0_f64),
        (ids::INSP_VIS_RECT_W, 0.0_f64),
        (ids::INSP_VIS_RECT_H, 0.0_f64),
    ] {
        store.register(
            id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value,
                buffer: format_number(value),
                caret: 0,
                last_committed: value,
                selection_anchor: None,
            },
        );
    }
    // Alpha Cutoff is a hard `0..1` mask threshold — drag-scrub spans the whole range (coherent with
    // its limits, like the texture number boxes; Enio 2026-06-26). The Rect fields are pixel extents
    // with no natural ceiling, so they keep the unbounded step-rate (no artificial clamp).
    store.set_number_range(ids::INSP_VIS_ALPHA_CUTOFF, 0.0, 1.0, 0.01); // LITERAL-PX-OK: alpha-cutoff chip 0..1 track step (non-design behaviour value)
}

/// Register the W3 segmented-tab + dropdown-option ids as `Button`s so
/// the pointer dispatcher routes their clicks (an unregistered hit id is
/// rejected by `is_focusable` and never emits `Click`). The selected
/// visual is snapshot-driven via `Tabs::selected`; these states exist
/// purely so the click reaches the event handler. §7 Sort Point tabs,
/// §7 Sorting Layer dropdown options, and the §9 Sampling tabs.
pub(crate) fn register_button_ids(store: &mut WidgetStore, ids: &[ph2d_a11y::NodeId]) {
    for &id in ids {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
}

/// §10 Material & Blend: register the 6 blend-mode segmented ids as
/// `Button`s (is_focusable → clicks route). Selection is snapshot-driven.
fn populate_blend(store: &mut WidgetStore) {
    register_button_ids(store, &ids::INSP_SAMPLE_BLEND);
}

/// **§5 9-Slice** (spec Sprite 03 §3.5) — construída em 2026-08-21.
///
/// ⚠️ **Cada id aqui é o que torna o widget FOCÁVEL**, e é a ponta que este repositório já perdeu
/// seis vezes: um controlo pintado e não registado é indistinguível de um partido — o ponteiro
/// nunca lhe chega e todo gate de compilação continua verde (DIRETIVA §2). O gate
/// `every_painted_id_is_reachable` cobra esta metade.
/// **§12 Sockets / Named Anchors** (ADR-0072) — construída em 2026-08-21.
fn populate_anchors(store: &mut WidgetStore) {
    // As 64 linhas da lista + os dois botões. ⚠️ TODAS as 64 se registam, mesmo que a maioria
    // dos sprites tenha 3 âncoras: um id só registado "quando aparece" nunca aparece, porque o
    // registo acontece uma vez, no arranque, e a lista cresce depois.
    register_button_ids(store, &ids::INSP_ANCHOR_ROW);
    register_button_ids(store, &[ids::INSP_ANCHOR_ADD, ids::INSP_ANCHOR_REMOVE]);
    store.register(
        ids::INSP_ANCHOR_NAME,
        InteractiveState::TextInput {
            state: TextInputState::Normal,
            text: String::new(),
            caret: 0,
            selection_anchor: None,
        },
    );
    for id in [ids::INSP_ANCHOR_BOUNDS_ON, ids::INSP_ANCHOR_CENTER_ON] {
        store.register(
            id,
            InteractiveState::Checkbox {
                state: CheckboxState::Normal,
                value: CheckboxValue::Unchecked,
            },
        );
    }
    for id in ids::INSP_ANCHOR_POS
        .iter()
        .chain(std::iter::once(&ids::INSP_ANCHOR_ROT))
        .chain(ids::INSP_ANCHOR_BOUNDS.iter())
        .chain(ids::INSP_ANCHOR_CENTER.iter())
        .copied()
    {
        store.register(
            id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: 0.0,
                buffer: format_number(0.0),
                caret: 0,
                last_committed: 0.0,
                selection_anchor: None,
            },
        );
    }
}

fn populate_slice(store: &mut WidgetStore) {
    register_button_ids(store, &ids::INSP_SLICE_MODE);
    register_button_ids(store, &ids::INSP_SLICE_TILE_MODE);
    // As oito células da grelha 3×3 são BOTÕES que ciclam — não segmentos.
    register_button_ids(store, &ids::INSP_SLICE_REGION);
    register_button_ids(store, &[ids::INSP_SLICE_ADD, ids::INSP_SLICE_REMOVE]);
    store.register(
        ids::INSP_SLICE_FILL_CENTER,
        InteractiveState::Checkbox {
            state: CheckboxState::Normal,
            value: CheckboxValue::Checked,
        },
    );
    // Bordas (px da fonte) e tamanho alvo (m). O `0.0` de partida é o do componente inerte.
    for id in ids::INSP_SLICE_BORDER
        .iter()
        .chain(ids::INSP_SLICE_SIZE.iter())
        .copied()
    {
        store.register(
            id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: 0.0,
                buffer: format_number(0.0),
                caret: 0,
                last_committed: 0.0,
                selection_anchor: None,
            },
        );
    }
    // O par slider+chip do `Stretch`. ⚠️ Ambos vivem em `0..1` — ao contrário do Emissive, aqui
    // **não há mapeamento**: o valor É uma fração de ladrilho, e mostrá-lo noutra escala seria
    // inventar uma unidade que o modelo não tem.
    store.register(
        ids::INSP_SLICE_STRETCH,
        InteractiveState::Slider {
            state: SliderState::Normal,
            value: 0.5,
            orientation: SliderOrientation::Horizontal,
        },
    );
    store.register(
        ids::INSP_SLICE_STRETCH_CHIP,
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value: 0.5,
            buffer: format_number(0.5),
            caret: 0,
            last_committed: 0.5,
            selection_anchor: None,
        },
    );
    store.link_slider_number(ids::INSP_SLICE_STRETCH, ids::INSP_SLICE_STRETCH_CHIP);
}

fn populate_sampling(store: &mut WidgetStore) {
    register_button_ids(store, &ids::INSP_SAMPLE_FILTER);
    register_button_ids(store, &ids::INSP_SAMPLE_REPEAT);
    register_button_ids(
        store,
        &[
            ids::INSP_ORDER_SP_CENTER,
            ids::INSP_ORDER_SP_PIVOT,
            ids::INSP_ORDER_SP_CUSTOM,
        ],
    );
    register_button_ids(store, &ids::INSP_ORDER_LAYER_OPT);
    // UV tiling/scroll NumberInputs (scale default 1.0, offset 0.0).
    for (id, value) in [
        (ids::INSP_SAMPLE_UV_SCALE_X, 1.0_f64),
        (ids::INSP_SAMPLE_UV_SCALE_Y, 1.0_f64),
        (ids::INSP_SAMPLE_UV_OFFSET_X, 0.0_f64),
        (ids::INSP_SAMPLE_UV_OFFSET_Y, 0.0_f64),
    ] {
        store.register(
            id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value,
                buffer: format_number(value),
                caret: 0,
                last_committed: value,
                selection_anchor: None,
            },
        );
    }
}

/// W3 Sprite Inspector v2 §7 Ordering / Sorting: 7 toggles + 2 integer
/// NumberInputs. Defaults match the optional-component "absent" state
/// (everything off, Z as Relative on per Godot). Live values sync from
/// the snapshot.
fn populate_ordering(store: &mut WidgetStore) {
    for (id, on) in [
        (ids::INSP_ORDER_Z_RELATIVE, true),
        (ids::INSP_ORDER_SHOW_BEHIND, false),
        (ids::INSP_ORDER_YSORT_ENABLED, false),
        (ids::INSP_ORDER_SORTING_GROUP, false),
        (ids::INSP_ORDER_SORT_AT_ROOT, false),
        (ids::INSP_ORDER_TOP_LEVEL, false),
    ] {
        store.register(
            id,
            InteractiveState::Checkbox {
                state: CheckboxState::Normal,
                value: if on {
                    CheckboxValue::Checked
                } else {
                    CheckboxValue::Unchecked
                },
            },
        );
    }
    for id in [
        ids::INSP_ORDER_Z_INDEX,
        ids::INSP_ORDER_ORDER_IN_LAYER,
        ids::INSP_ORDER_AXIS_X,
        ids::INSP_ORDER_AXIS_Y,
    ] {
        store.register(
            id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: 0.0,
                buffer: format!("{:.0}", 0.0),
                caret: 0,
                last_committed: 0.0,
                selection_anchor: None,
            },
        );
    }
    // Sorting Layer dropdown (default = "Default" layer index 2).
    store.register(
        ids::INSP_ORDER_SORTING_LAYER,
        InteractiveState::Dropdown {
            state: DropdownState::Normal,
            open: false,
            selected_index: Some(2),
        },
    );
}

/// W2 Sprite Inspector v2 Sprite Sheet grid: Centered toggle (default
/// on) + Offset X/Y (default 0) + HFrames / VFrames (default 1) + Frame
/// (default 0). Live values sync from the snapshot.
fn populate_sprite_sheet(store: &mut WidgetStore) {
    store.register(
        ids::INSP_SPRITE_CENTERED,
        InteractiveState::Checkbox {
            state: CheckboxState::Normal,
            value: CheckboxValue::Checked,
        },
    );
    for id in [ids::INSP_SPRITE_OFFSET_X, ids::INSP_SPRITE_OFFSET_Y] {
        store.register(
            id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: 0.0,
                buffer: format!("{:.0}", 0.0),
                caret: 0,
                last_committed: 0.0,
                selection_anchor: None,
            },
        );
    }
    for (id, value) in [
        (ids::INSP_SPRITE_HFRAMES, 1.0_f64),
        (ids::INSP_SPRITE_VFRAMES, 1.0_f64),
        (ids::INSP_SPRITE_FRAME, 0.0_f64),
    ] {
        store.register(
            id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value,
                buffer: format!("{value:.0}"),
                caret: 0,
                last_committed: value,
                selection_anchor: None,
            },
        );
    }
}

/// W2 Sprite Inspector v2 Color & Tint controls: Opacity Slider (0..1
/// storage, default 1.0) with a linked percent chip (0..100), + Tint Fill
/// checkbox (default off). Live values sync from the snapshot.
fn populate_color_tint(store: &mut WidgetStore) {
    // Opacity Slider 0..1 + linked chip showing 0..100 % (spec §3.6).
    store.register(
        ids::INSP_SPRITE_OPACITY,
        InteractiveState::Slider {
            state: SliderState::Normal,
            value: 1.0,
            orientation: SliderOrientation::Horizontal,
        },
    );
    store.register(
        ids::INSP_SPRITE_OPACITY_CHIP,
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value: 100.0, // LITERAL-PX-OK: opacity percent scale (1.0 → 100 %), not a design token
            buffer: format_number(100.0), // LITERAL-PX-OK: opacity percent scale
            caret: 0,
            last_committed: 100.0, // LITERAL-PX-OK: opacity percent scale
            selection_anchor: None,
        },
    );
    // chip_display = slider_storage * 100 (+0); integer-snapped so the
    // chip is whole percents while the slider track stays continuous.
    store.link_slider_number_mapped_integer(
        ids::INSP_SPRITE_OPACITY,
        ids::INSP_SPRITE_OPACITY_CHIP,
        100.0, // LITERAL-PX-OK: opacity percent scale (slider 0..1 → chip 0..100)
        0.0,
    );
    // Opacity is a hard `0..100 %` — drag-scrub on the chip spans the whole range proportionally
    // (coherent with its limits, like the texture number boxes; Enio 2026-06-26).
    store.set_number_range(ids::INSP_SPRITE_OPACITY_CHIP, 0.0, 100.0, 1.0); // LITERAL-PX-OK: opacity percent scale

    // **EMISSIVE** — a sprite como fonte de luz (plano `docs/Sprite_projeto/18` W8).
    //
    // ⚠️ **O slider guarda `0..1` normalizado; a chip mostra a intensidade REAL** (`0..EMISSIVE_MAX`).
    // Mesmo par que a Opacidade, e pela mesma razão: um slider cujo curso fosse `0..64` daria ao
    // artista 63/64 do percurso para valores que ele nunca usa. O mapeamento vive AQUI, num sítio só
    // — se ele se duplicasse, o número que o artista lê e o que o motor aplica divergiriam.
    store.register(
        ids::INSP_SPRITE_EMISSIVE,
        InteractiveState::Slider {
            state: SliderState::Normal,
            value: 0.0,
            orientation: SliderOrientation::Horizontal,
        },
    );
    store.register(
        ids::INSP_SPRITE_EMISSIVE_CHIP,
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value: 0.0,
            buffer: format_number(0.0),
            caret: 0,
            last_committed: 0.0,
            selection_anchor: None,
        },
    );
    // ⚠️ **A escala vem de `ph2d_ecs::EMISSIVE_MAX`, não de um literal.** Ela é o tecto da
    // REPRESENTAÇÃO (o meio-float do `GameRt`), documentado ao lado da constante; escrevê-lo aqui
    // outra vez faria a UI e o motor discordarem no dia em que alguém remedisse o tecto.
    store.link_slider_number_mapped(
        ids::INSP_SPRITE_EMISSIVE,
        ids::INSP_SPRITE_EMISSIVE_CHIP,
        ph2d_editor_core::EMISSIVE_MAX_UI,
        0.0,
    );
    store.set_number_range(
        ids::INSP_SPRITE_EMISSIVE_CHIP,
        0.0,
        f64::from(ph2d_editor_core::EMISSIVE_MAX_UI),
        0.1, // LITERAL-PX-OK: passo de scrub da intensidade, não um token de desenho
    );
    store.register(
        ids::INSP_SPRITE_TINT_FILL,
        InteractiveState::Checkbox {
            state: CheckboxState::Normal,
            value: CheckboxValue::Unchecked,
        },
    );
    // Tint / Self Tint + 4 per-corner color swatches. Registered as
    // `Plain` (like the section color-dots in `pre_populate` and
    // grid-snap's swatch) so `is_focusable` is true and the pointer
    // dispatch arms `active` on Down → emits `Click` on Up. Without this
    // leg the click is silently dropped and the picker never opens (the
    // swatch carries no value of its own — its color lives in the
    // `widget_colors` side-table).
    for id in [
        ids::INSP_SPRITE_TINT_SWATCH,
        ids::INSP_SPRITE_SELF_TINT_SWATCH,
        ids::INSP_SPRITE_CORNER_TL,
        ids::INSP_SPRITE_CORNER_TR,
        ids::INSP_SPRITE_CORNER_BL,
        ids::INSP_SPRITE_CORNER_BR,
    ] {
        store.register(id, InteractiveState::Plain);
    }
    // "Equalize corners" button (copies TL → the other three).
    store.register(
        ids::INSP_SPRITE_CORNER_EQUALIZE,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
    // (The Color & Tint sub-tabs were retired 2026-05-31 — the section
    // now stacks every control visible at once, so no tab Button group.)
}

/// W2 Sprite Inspector v2: Flip H / Flip V checkboxes. Default
/// Unchecked (the Sprite default `flip_x = flip_y = false`); the live
/// value is synced from the snapshot each frame in `sync.rs`.
fn populate_sprite_flip(store: &mut WidgetStore) {
    for id in [ids::INSP_SPRITE_FLIP_X, ids::INSP_SPRITE_FLIP_Y] {
        store.register(
            id,
            InteractiveState::Checkbox {
                state: CheckboxState::Normal,
                value: CheckboxValue::Unchecked,
            },
        );
    }
}

/// W2 Sprite Inspector v2 — Region sampling (Render Source section,
/// spec §3.3): enable toggle (default off) + 4 px NumberInputs (x/y/w/h,
/// default 0) + filter-clip toggle (default ON, the Atlas anti-bleed
/// default). Live values sync from the snapshot.
fn populate_region(store: &mut WidgetStore) {
    store.register(
        ids::INSP_REGION_ENABLED,
        InteractiveState::Checkbox {
            state: CheckboxState::Normal,
            value: CheckboxValue::Unchecked,
        },
    );
    for id in [
        ids::INSP_REGION_X,
        ids::INSP_REGION_Y,
        ids::INSP_REGION_W,
        ids::INSP_REGION_H,
    ] {
        store.register(
            id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: 0.0,
                buffer: format!("{:.0}", 0.0),
                caret: 0,
                last_committed: 0.0,
                selection_anchor: None,
            },
        );
    }
    store.register(
        ids::INSP_REGION_FILTER_CLIP,
        InteractiveState::Checkbox {
            state: CheckboxState::Normal,
            value: CheckboxValue::Checked,
        },
    );
}

fn populate_name_editor(store: &mut WidgetStore) {
    store.register(
        ids::INSP_ENTITY_NAME,
        InteractiveState::TextInput {
            state: TextInputState::Normal,
            text: String::new(),
            caret: 0,
            selection_anchor: None,
        },
    );
}

fn populate_render_strategy(store: &mut WidgetStore) {
    for id in [
        ids::INSP_RENDER_STRATEGY_ATLAS,
        ids::INSP_RENDER_STRATEGY_INDIVIDUAL,
        ids::INSP_RENDER_STRATEGY_HANDPACKED,
    ] {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    // ⚠️ **`INSP_RENDER_FORMAT_RGBA8`/`_RGBA16` VOLTARAM** (plano `docs/Sprite_projeto/18` W5), e o
    // registo aqui é a metade que os matava.
    //
    // Eles saíram em 2026-08-19 (plano 17 §5) por serem **registados e focáveis com o clique a cair
    // no chão** — sem arm de dispatch em lado nenhum, nem um toast, e com o aceso vindo de um
    // literal. O que mudou não foi a opinião sobre o botão: foi existir modelo por trás dele
    // (`Asset::ImageRgba16`, `FORMAT_16`, `PixelPayload`, e a conversão nos dois sentidos).
    //
    // ⛔ Sem este `register` o botão nasce **sem `InteractiveState`**: não é focável, o Down não o
    // arma, o Up nunca emite `Click`. Pintado, hit-registered e morto sob o rato — a mesma falha
    // que apagou oito pills da barra em 2026-08-19.
    for id in [
        ids::INSP_RENDER_FORMAT_RGBA8,
        ids::INSP_RENDER_FORMAT_RGBA16,
    ] {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    store.register(
        ids::INSP_RENDER_SOURCE_REIMPORT,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
}

fn populate_visibility_editor(store: &mut WidgetStore) {
    store.register(
        ids::INSP_VISIBILITY_CHECK,
        InteractiveState::Checkbox {
            state: CheckboxState::Normal,
            value: CheckboxValue::Checked,
        },
    );
}

fn populate_transform_editor(store: &mut WidgetStore) {
    let identity_pairs = [
        (ids::INSP_TRANSFORM_POS_X, 0.0_f64),
        (ids::INSP_TRANSFORM_POS_Y, 0.0_f64),
        (ids::INSP_TRANSFORM_ROT, 0.0_f64),
        (ids::INSP_TRANSFORM_SCALE_X, 1.0_f64),
        (ids::INSP_TRANSFORM_SCALE_Y, 1.0_f64),
        (ids::INSP_TRANSFORM_SKEW_X, 0.0_f64),
        (ids::INSP_TRANSFORM_SKEW_Y, 0.0_f64),
    ];
    for (id, value) in identity_pairs {
        let buffer = format!("{value}");
        store.register(
            id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value,
                buffer,
                caret: 0,
                last_committed: value,
                selection_anchor: None,
            },
        );
    }
    store.register(
        ids::INSP_TRANSFORM_RESET,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
}

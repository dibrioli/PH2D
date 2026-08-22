//! Inspector panel `apply_event` — ADR-0029 Phase C.1 port.
//!
//! Migrated from `ph2d_editor_core::screens::hero::inspector::{mod,
//! apply_event_full}` to the panel crate. The signature changed from
//! `(hero: &mut HeroScreen, ev: WidgetEvent)` to
//! `(state: &mut InspectorState, host: &mut dyn PanelHostInternal,
//! ev: WidgetEvent)`. All `hero.<field>` accesses route through
//! [`PanelHostInternal`] trait methods.
//! ✅ **O braço do ponto de cor deixou de ENUMERAR os seus leitores** (2026-08-21) — a cura que
//! esta nota nomeava está feita: **uma** tabela [`ids::LIVE_SECTIONS`] de pares `(seção, cor)`, e
//! `pre_populate` (dobra + registo) e este braço (despacho) são projeções dela.
//!
//! ⚠️ **A nota anterior estava certa no mecanismo e errada no número** — dizia que ORDERING /
//! SAMPLING / BLEND estavam «em nenhum dos dois sítios»; a auditoria de 7 lentes mediu **sete**
//! pontos mortos (mais Pulley Wheel e Platform Player) e **três** cabeçalhos que pintavam o chevron
//! e não dobravam. *Uma nota de dívida também envelhece — foi por isso que a cura virou tabela e
//! não uma sexta entrada na lista.*
//!
//! A varredura que impede a recaída é `tests/every_painted_id_is_reachable.rs`: ela pinta o
//! Inspector inteiro e exige que **todo id registado no índice de acerto** passe na pergunta do
//! `is_focusable`. Ela não tem lista nenhuma dentro — a lista é o que o painel pinta
//! ([[feedback_a_condition_that_enumerates_its_readers_rots]]).
//!

use crate::state;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal};
use ph2d_editor_core::screens::hero::{
    InspectorNameInfo, InspectorTransformInfo, InspectorVisibilityInfo, SpriteFieldEdit,
};
use ph2d_editor_core::widget::{ButtonState, CheckboxValue};

pub(crate) fn apply_event(
    _state: &mut state::InspectorState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    EventOutcome::from_bool(apply_event_impl(host, ev))
}

fn apply_event_impl(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    if section_color_click(host, ev) {
        return true;
    }

    // W2 Color & Tint — Tint / Self Tint swatch click opens the shared
    // BlenderColorPicker (OKLCH) seeded from the sprite's CURRENT channel
    // (not the generic per-widget accent the section dot uses). The
    // chosen color round-trips via `widget_color(<swatch>)` — mirrored
    // each frame from the picker in `hero.rs` — and `sync.rs` dispatches
    // it as `SpriteFieldEdit::Tint` / `SelfTint` while the picker targets
    // this swatch.
    if let WidgetEvent::Click(id) = ev
        && matches!(
            id,
            ids::INSP_SPRITE_TINT_SWATCH | ids::INSP_SPRITE_SELF_TINT_SWATCH
        )
        && let Some(info) = state::current_inspector_sprite()
    {
        let chan = if id == ids::INSP_SPRITE_TINT_SWATCH {
            info.tint
        } else {
            info.self_tint
        };
        let seed = state::tint_f32_to_u8(chan);
        host.store_mut().set_widget_color(id, seed);
        host.store_mut().set_picker_target(Some(id));
        host.store_mut().set_blender_value(
            ids::INSP_BLENDER_PICKER,
            ph2d_tokens::ColorValue::from_rgba8(seed[0], seed[1], seed[2], seed[3]),
        );
        return true;
    }

    // (Color & Tint sub-tabs retired 2026-05-31 — the section stacks all
    // controls visible at once, so there's no tab selection to pin.)

    // W2 Color & Tint — per-corner swatch click opens the picker seeded
    // from the sprite's CURRENT corner color (TL=0, TR=1, BL=2, BR=3).
    // `sync.rs` replaces that one corner of the [[f32;4];4] array and
    // dispatches the whole `SpriteFieldEdit::PerCornerTint`.
    if let WidgetEvent::Click(id) = ev
        && let Some(corner) = match id {
            ids::INSP_SPRITE_CORNER_TL => Some(0usize),
            ids::INSP_SPRITE_CORNER_TR => Some(1),
            ids::INSP_SPRITE_CORNER_BL => Some(2),
            ids::INSP_SPRITE_CORNER_BR => Some(3),
            _ => None,
        }
        && let Some(info) = state::current_inspector_sprite()
    {
        let seed = state::tint_f32_to_u8(info.per_corner_tint[corner]);
        host.store_mut().set_widget_color(id, seed);
        host.store_mut().set_picker_target(Some(id));
        host.store_mut().set_blender_value(
            ids::INSP_BLENDER_PICKER,
            ph2d_tokens::ColorValue::from_rgba8(seed[0], seed[1], seed[2], seed[3]),
        );
        return true;
    }

    // W2 Color & Tint — "Equalize Corners" copies the top-left corner to
    // the other three (spec §3.6), dispatched as one PerCornerTint edit.
    if let WidgetEvent::Click(id) = ev
        && id == ids::INSP_SPRITE_CORNER_EQUALIZE
        && let Some(info) = state::current_inspector_sprite()
    {
        host.bus_mut().push(EditorAction::InspectorSpriteEdit {
            entity_bits: info.entity_bits,
            edit: SpriteFieldEdit::EqualizeCorners,
        });
        // Momentary button — demote the visual back to Normal so it
        // doesn't stick Pressed after the click.
        if let Some(InteractiveState::Button { state }) = host.store_mut().get_mut(id) {
            *state = ButtonState::Normal;
        }
        return true;
    }

    // Close (X) — hide the Inspector. Same effect as toggling the
    // left-rail Inspector pill (vide `chrome/rail_panels.rs`). UI canon
    // post-2026-05-24: every floating panel except Hierarchy has X.
    //
    // Sync the left-rail RAIL_SHOW_INSPECTOR button state so its
    // Pressed/Normal visual tracks the panel's actual visibility —
    // without this, hiding via X leaves the rail toggle stuck
    // Pressed (bug reported 2026-05-24).
    if let WidgetEvent::Click(id) = ev
        && id == ids::INSP_CLOSE
    {
        let next = !host.panel_visible("inspector");
        host.set_panel_visible("inspector", next);
        if let Some(InteractiveState::Button { state }) =
            host.store_mut().get_mut(ids::RAIL_SHOW_INSPECTOR)
        {
            *state = if next {
                ButtonState::Pressed
            } else {
                ButtonState::Normal
            };
        }
        return true;
    }
    // M14.5 inspector phase (6.4) — Reimport button.
    if let WidgetEvent::Click(id) = ev
        && id == ids::INSP_RENDER_SOURCE_REIMPORT
        && let Some(info) = state::current_inspector_sprite()
        && info.can_reimport
    {
        host.bus_mut().push(EditorAction::Reimport {
            entity_bits: info.entity_bits,
        });
        return true;
    }
    // M14.A — Transform editor commits.
    if let WidgetEvent::ValueChanged(id) = ev
        && matches!(
            id,
            ids::INSP_TRANSFORM_POS_X
                | ids::INSP_TRANSFORM_POS_Y
                | ids::INSP_TRANSFORM_ROT
                | ids::INSP_TRANSFORM_SCALE_X
                | ids::INSP_TRANSFORM_SCALE_Y
                | ids::INSP_TRANSFORM_SKEW_X
                | ids::INSP_TRANSFORM_SKEW_Y,
        )
        && let Some(info) = state::current_inspector_transform()
    {
        let unit = host.project().display_unit;
        let ppm = host.project().pixels_per_meter;
        let x_disp =
            host.store()
                .number_value(ids::INSP_TRANSFORM_POS_X)
                .unwrap_or(unit.from_meters(info.translation[0], ppm) as f64) as f32;
        let y_disp =
            host.store()
                .number_value(ids::INSP_TRANSFORM_POS_Y)
                .unwrap_or(unit.from_meters(info.translation[1], ppm) as f64) as f32;
        let x = unit.to_meters(x_disp, ppm);
        let y = unit.to_meters(y_disp, ppm);
        let rot_deg = host
            .store()
            .number_value(ids::INSP_TRANSFORM_ROT)
            .unwrap_or((info.rotation_rad as f64).to_degrees()) as f32;
        let sx = host
            .store()
            .number_value(ids::INSP_TRANSFORM_SCALE_X)
            .unwrap_or(info.scale[0] as f64) as f32;
        let sy = host
            .store()
            .number_value(ids::INSP_TRANSFORM_SCALE_Y)
            .unwrap_or(info.scale[1] as f64) as f32;
        // Skew authored in degrees for UX parity with Rotation; the
        // ECS-commit boundary converts to radians and clamps to
        // Transform::SKEW_LIMIT (ADR-0025-amendment-1 §2.5).
        let skew_x_deg = host
            .store()
            .number_value(ids::INSP_TRANSFORM_SKEW_X)
            .unwrap_or((info.skew_rad[0] as f64).to_degrees()) as f32;
        let skew_y_deg = host
            .store()
            .number_value(ids::INSP_TRANSFORM_SKEW_Y)
            .unwrap_or((info.skew_rad[1] as f64).to_degrees()) as f32;
        host.bus_mut().push(EditorAction::InspectorTransformEdit(
            InspectorTransformInfo {
                entity_bits: info.entity_bits,
                translation: [x, y],
                rotation_rad: rot_deg.to_radians(),
                scale: [sx, sy],
                skew_rad: [skew_x_deg.to_radians(), skew_y_deg.to_radians()],
            },
        ));
        return true;
    }
    if let WidgetEvent::Click(id) = ev
        && id == ids::INSP_TRANSFORM_RESET
        && let Some(info) = state::current_inspector_transform()
    {
        host.bus_mut().push(EditorAction::InspectorTransformEdit(
            InspectorTransformInfo {
                entity_bits: info.entity_bits,
                translation: [0.0, 0.0],
                rotation_rad: 0.0,
                scale: [1.0, 1.0],
                skew_rad: [0.0, 0.0],
            },
        ));
        return true;
    }
    if visibility_toggle(host, ev) {
        return true;
    }
    // W2 Sprite Inspector v2 — logical Flip H / Flip V toggled.
    if let WidgetEvent::Toggled(id) = ev
        && matches!(id, ids::INSP_SPRITE_FLIP_X | ids::INSP_SPRITE_FLIP_Y)
        && let Some(info) = state::current_inspector_sprite()
    {
        let checked = matches!(
            host.store().checkbox(id).map(|(_, v)| v),
            Some(CheckboxValue::Checked)
        );
        let edit = if id == ids::INSP_SPRITE_FLIP_X {
            SpriteFieldEdit::FlipX(checked)
        } else {
            SpriteFieldEdit::FlipY(checked)
        };
        host.bus_mut().push(EditorAction::InspectorSpriteEdit {
            entity_bits: info.entity_bits,
            edit,
        });
        return true;
    }
    // W2 Color & Tint — Tint Fill (silhouette) toggled.
    if let WidgetEvent::Toggled(id) = ev
        && id == ids::INSP_SPRITE_TINT_FILL
        && let Some(info) = state::current_inspector_sprite()
    {
        let checked = matches!(
            host.store().checkbox(id).map(|(_, v)| v),
            Some(CheckboxValue::Checked)
        );
        host.bus_mut().push(EditorAction::InspectorSpriteEdit {
            entity_bits: info.entity_bits,
            edit: SpriteFieldEdit::TintFill(checked),
        });
        return true;
    }
    // **Os dois sliders-com-chip da sprite** — Opacidade e Emissive, no irmão
    // [`crate::event_sprite_value`]. Saíram juntos em 2026-08-21 quando a linha `Emissive` (plano
    // `docs/Sprite_projeto/18` W8) empurrou este despachante para 433 contra uma tolerância de 410:
    // levar só o novo devolveria o número a 410 e não desceria nada, e *a catraca só desce*.
    if crate::event_sprite_value::apply_sprite_slider_event(host, ev) {
        return true;
    }
    // W3 §7 Ordering — all ordering widget events (sibling module, LOC).
    if crate::event_ordering::apply_ordering_event(host, ev) {
        return true;
    }
    // W2 Sprite Sheet — HFrames / VFrames / Frame committed. Integer
    // fields; rounded from the NumberInput's f64. Clamps (>=1, in-grid)
    // land at the commit boundary (apply_sprite_field).
    if let WidgetEvent::ValueChanged(id) = ev
        && matches!(
            id,
            ids::INSP_SPRITE_HFRAMES | ids::INSP_SPRITE_VFRAMES | ids::INSP_SPRITE_FRAME
        )
        && let Some(info) = state::current_inspector_sprite()
    {
        let raw = host.store().number_value(id).unwrap_or(0.0);
        let n = raw.round().max(0.0) as u32;
        let edit = if id == ids::INSP_SPRITE_HFRAMES {
            SpriteFieldEdit::Hframes(n)
        } else if id == ids::INSP_SPRITE_VFRAMES {
            SpriteFieldEdit::Vframes(n)
        } else {
            SpriteFieldEdit::Frame(n)
        };
        host.bus_mut().push(EditorAction::InspectorSpriteEdit {
            entity_bits: info.entity_bits,
            edit,
        });
        return true;
    }
    // W2 Region (spec §3.3) — enable / filter-clip toggles.
    if let WidgetEvent::Toggled(id) = ev
        && matches!(id, ids::INSP_REGION_ENABLED | ids::INSP_REGION_FILTER_CLIP)
        && let Some(info) = state::current_inspector_sprite()
    {
        let checked = matches!(
            host.store().checkbox(id).map(|(_, v)| v),
            Some(CheckboxValue::Checked)
        );
        if id == ids::INSP_REGION_FILTER_CLIP {
            host.bus_mut().push(EditorAction::InspectorSpriteEdit {
                entity_bits: info.entity_bits,
                edit: SpriteFieldEdit::RegionFilterClip(checked),
            });
        } else {
            host.bus_mut().push(EditorAction::InspectorSpriteEdit {
                entity_bits: info.entity_bits,
                edit: SpriteFieldEdit::RegionEnabled(checked),
            });
            // Enabling region on a still-zero rect would make the sprite
            // vanish (zero-area UV = no-op). Seed the rect to the full
            // source (spec §3.3 default `[0, 0, w, h]`) so toggling on is
            // visible and editable. SINGLE-SELECT ONLY: on a multi-select
            // the source size is per-sprite, so seeding the primary's dims
            // onto all would give every other sprite a wrong rect (audit
            // D-3). For a multi-select the user sets the rect explicitly.
            if checked
                && info.selected_count == 1
                && (info.region_rect[2] <= 0.0 || info.region_rect[3] <= 0.0)
                && let Some((sw, sh)) = info.source_pixels
            {
                host.bus_mut().push(EditorAction::InspectorSpriteEdit {
                    entity_bits: info.entity_bits,
                    edit: SpriteFieldEdit::RegionRect([0.0, 0.0, sw as f32, sh as f32]),
                });
            }
        }
        return true;
    }
    // W2 Region — X/Y/W/H px NumberInputs. Each dispatches ONLY its own
    // axis (per-axis SpriteFieldEdit) so a bulk edit of one axis can't
    // re-read + stomp a diverging sibling (audit D-1). W/H floor at 0 at
    // the commit boundary.
    if let WidgetEvent::ValueChanged(id) = ev
        && let Some(axis) = match id {
            ids::INSP_REGION_X => Some(0usize),
            ids::INSP_REGION_Y => Some(1),
            ids::INSP_REGION_W => Some(2),
            ids::INSP_REGION_H => Some(3),
            _ => None,
        }
        && let Some(info) = state::current_inspector_sprite()
    {
        let v = host
            .store()
            .number_value(id)
            .unwrap_or(info.region_rect[axis] as f64) as f32;
        let edit = match axis {
            0 => SpriteFieldEdit::RegionX(v),
            1 => SpriteFieldEdit::RegionY(v),
            2 => SpriteFieldEdit::RegionW(v),
            _ => SpriteFieldEdit::RegionH(v),
        };
        host.bus_mut().push(EditorAction::InspectorSpriteEdit {
            entity_bits: info.entity_bits,
            edit,
        });
        return true;
    }
    // W2 origin (spec §3.4) — Centered toggle.
    if let WidgetEvent::Toggled(id) = ev
        && id == ids::INSP_SPRITE_CENTERED
        && let Some(info) = state::current_inspector_sprite()
    {
        let checked = matches!(
            host.store().checkbox(id).map(|(_, v)| v),
            Some(CheckboxValue::Checked)
        );
        host.bus_mut().push(EditorAction::InspectorSpriteEdit {
            entity_bits: info.entity_bits,
            edit: SpriteFieldEdit::Centered(checked),
        });
        return true;
    }
    // W2 origin — Offset X/Y px NumberInputs. Per-axis dispatch (not the
    // whole Offset vector) so editing one axis can't stomp a diverging
    // sibling on a multi-selection (audit D-1).
    if let WidgetEvent::ValueChanged(id) = ev
        && matches!(id, ids::INSP_SPRITE_OFFSET_X | ids::INSP_SPRITE_OFFSET_Y)
        && let Some(info) = state::current_inspector_sprite()
    {
        let is_x = id == ids::INSP_SPRITE_OFFSET_X;
        let fallback = if is_x { info.offset[0] } else { info.offset[1] };
        let v = host.store().number_value(id).unwrap_or(fallback as f64) as f32;
        let edit = if is_x {
            SpriteFieldEdit::OffsetX(v)
        } else {
            SpriteFieldEdit::OffsetY(v)
        };
        host.bus_mut().push(EditorAction::InspectorSpriteEdit {
            entity_bits: info.entity_bits,
            edit,
        });
        return true;
    }
    // **Os dois pares da seção Render Source** — estratégia e precisão — moram num irmão.
    // Ver [`crate::event_precision`], e a nota de LOC lá dentro.
    if crate::event_precision::render_source_click(host, ev) {
        return true;
    }
    if section_text_changed(host, ev) {
        return true;
    }
    // ADR-0029 Phase C.1: showcase-shared events
    // (`CTX_MENU_OUTLINE_*`, `CTX_MENU_CREATE_NOTE`, `SECTION_IDS`,
    // radio/tab/tree pinning) are now routed at host level via
    // `widget::showcase::apply_showcase_event`. The Inspector panel
    // returns `Ignored` for those — host picks them up after the
    // registry walk.
    false
}

/// **Os campos de TEXTO do Inspector.** Os dois nomeiam alguma coisa — como o
/// objeto se chama, e o que ele GRITA quando algo chega nele — e os dois
/// viajam pelo mesmo barramento por-entidade, com o `InspectorNameInfo` no
/// lugar de um campo de `PhysicsFieldEdit`: o valor é uma STRING, e todo o
/// resto da §11 fala em número ou em chip. Um braço de string no enum dos
/// campos numéricos seria um segundo formato de edição vivendo dentro do
/// primeiro.
///
/// ⚠️ **Função própria pelo MESMO motivo que a `section_color_click` abaixo**, e
/// pela mesma catraca: o `apply_event_impl` vive sob um teto que só pode
/// ENCOLHER, e a row de sinal da W-Signal o empurrou de 452 para 470. Movê-la
/// para cá é a correção certa — subir o número do allowlist seria usar como
/// licença de crescimento uma entrada cuja prosa diz o contrário.
///
/// ⚠️ E ela ficou latente por uma causa que esta linha já pagou antes: este gate
/// mora em `ph2d-editor-core/tests/`, então um fechamento por `cargo test -p`
/// nas crates da física **não o alcança**.
fn section_text_changed(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    let WidgetEvent::TextChanged(id) = ev else {
        return false;
    };
    // M14.E — o nome da entidade.
    if id == ids::INSP_ENTITY_NAME
        && let Some(info) = state::current_inspector_name()
    {
        let text = host.store().text(id).unwrap_or("").to_string();
        host.bus_mut()
            .push(EditorAction::InspectorNameEdit(InspectorNameInfo {
                entity_bits: info.entity_bits,
                name: text,
            }));
        return true;
    }
    // W-Signal · W-SignalLeave — os nomes que este objeto grita quando algo
    // CHEGA nele e quando algo SAI. Duas rows, dois contratos, duas ações: um
    // `leave` enfiado na mesma ação com um bool tornaria impossível ler o
    // barramento sem perguntar duas coisas para saber uma.
    if id == ids::INSP_PHYS_SIGNAL
        && let Some(info) = state::current_inspector_physics()
    {
        let text = host.store().text(id).unwrap_or("").to_string();
        host.bus_mut()
            .push(EditorAction::InspectorSignalEdit(InspectorNameInfo {
                entity_bits: info.entity_bits,
                name: text,
            }));
        return true;
    }
    if id == ids::INSP_PHYS_SIGNAL_LEAVE
        && let Some(info) = state::current_inspector_physics()
    {
        let text = host.store().text(id).unwrap_or("").to_string();
        host.bus_mut()
            .push(EditorAction::InspectorSignalLeaveEdit(InspectorNameInfo {
                entity_bits: info.entity_bits,
                name: text,
            }));
        return true;
    }
    false
}

/// A click on a section's colour dot — seed the canonical `BlenderPicker` at
/// that section's colour id, the same flow the Widget Gallery uses for its
/// `SECTION_COLOR_IDS`. The picker writes the chosen rgba back via
/// `set_widget_color(<color_id>, rgba)` (drained in `hero.rs`), and the next
/// `paint_section_header` paints the dot in it. UI canon 2026-05-24: every
/// section can carry a per-user accent colour.
///
/// Its own function because `apply_event_impl` is under a ratcheting LOC cap
/// and the two physics dots (§11/§12) pushed it over. Returns whether the
/// event was consumed.
/// **A caixa «Visible» do topo** (M14.D) — extraída da mãe em 2026-08-21 pela catraca de LOC.
///
/// ⚠️ O que a fez crescer foi o **fan-out**: esta caixa editava só a primária enquanto a §8
/// Visibility logo abaixo editava toda a seleção (auditoria `docs/Sprite_projeto/20` §3). Ganhar o
/// espalhamento exigiu ganhar antes a afordância de divergência — *espalhar sem sinal troca um
/// sub-aplicar silencioso por um esmagamento silencioso*.
fn visibility_toggle(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    if let WidgetEvent::Toggled(id) = ev
        && id == ids::INSP_VISIBILITY_CHECK
        && let Some(info) = state::current_inspector_visibility()
    {
        let visible = matches!(
            host.store().checkbox(id).map(|(_, v)| v),
            Some(CheckboxValue::Checked),
        );
        host.bus_mut().push(EditorAction::InspectorVisibilityEdit(
            InspectorVisibilityInfo {
                entity_bits: info.entity_bits,
                visible,
                // ⚠️ **A ação carrega `false` porque ela é uma DECISÃO, não um estado.** O `mixed`
                // do snapshot descreve o que a seleção era *antes*; o que sobe aqui é o que o
                // artista acabou de escolher para todos. Ecoar a divergência de volta faria o
                // dreno ter de a interpretar — e ele já não a lê.
                mixed: false,
            },
        ));
        return true;
    }
    false
}

fn section_color_click(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    // ⚠️ **A condição deixou de ENUMERAR os seus leitores** (2026-08-21). Ela listava seis dos
    // treze pontos, e a nota no topo deste ficheiro — que já denunciava a podridão — dizia
    // **três**: *uma nota de dívida também envelhece*. Agora a fonte é `ids::LIVE_SECTIONS`, a
    // mesma tabela que o `pre_populate` lê para registar o ponto e a dobra. Um ponto novo arma no
    // dia em que a seção entra na tabela.
    if let WidgetEvent::Click(id) = ev
        && ids::LIVE_SECTION_COLOR_IDS.contains(&id)
    {
        let seed = host
            .store()
            .widget_color(id)
            .unwrap_or([0x88, 0x88, 0x88, 0xff]); // LITERAL-COLOR-OK: neutral seed
        host.store_mut().set_widget_color(id, seed);
        host.store_mut().set_picker_target(Some(id));
        host.store_mut().set_blender_value(
            ids::INSP_BLENDER_PICKER,
            ph2d_tokens::ColorValue::from_rgba8(seed[0], seed[1], seed[2], seed[3]),
        );
        return true;
    }
    false
}

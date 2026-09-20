//! ⭐⭐ **As SEIS amostras de cor da sprite** — Tinta, Tinta Própria e os quatro cantos — e o
//! predicado que diz quais delas prendem o selector à selecção actual.
//!
//! ⛔ **Saiu do [`crate::sync`] por CAP de FICHEIRO** (`603` de `600`), e o corte é por
//! RESPONSABILIDADE: o irmão é a semente do painel inteiro, e isto é **um assunto fechado** — a
//! ida-e-volta de uma cor pelo selector partilhado, com os dois regimes na mesma função.
//! ⛔ Curado por CORTE, **nunca** por uma entrada no `FILE_OVERAGE_OK`.
//!
//! ⚠️ Ele saiu **verbatim e na MESMA posição da semente** — a seguir aos dois sliders-com-chip —,
//! porque a ordem decide quem lê o `picker_target` primeiro: *um corte que muda a ordem da
//! semente não é um corte, é uma mudança de comportamento com cara de arrumação.*

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::{InspectorSpriteInfo, SpriteFieldEdit};

/// As seis amostras, na ordem em que a semente sempre as correu.
pub(crate) fn sync_tint_swatches(host: &mut dyn PanelHostInternal, sp: &InspectorSpriteInfo) {
    // Tint / Self Tint swatches. The BlenderColorPicker round-trips
    // the chosen color through `widget_color(<swatch>)` (mirrored
    // each frame from the picker in `hero.rs`, BEFORE this panel
    // paints). Two regimes:
    //   • picker targets THIS swatch → the user is actively picking:
    //     dispatch the divergence as a Sprite edit (live preview,
    //     same 1-frame post-commit lag the Opacity slider tolerates).
    //     Byte-compare against the committed channel so sub-1/255
    //     float dust doesn't spin the bus every frame, and so the
    //     stream stops the instant the commit lands (round-trip is
    //     exact: u8 → /255 → ×255 round-nearest → same u8).
    //   • otherwise → keep the swatch fill in lock-step with the
    //     committed channel so undo / external edits reflect.
    // On an entity switch the top `entity_changed` block has already
    // closed any tint picker bound to the prior selection, so this
    // reads `None` on the switch frame and the loop reseeds both
    // swatches from the new sprite's committed channels.
    let picker_target = host.store().picker_target();
    for (swatch_id, chan) in [
        (crate::ids::INSP_SPRITE_TINT_SWATCH, sp.tint),
        (crate::ids::INSP_SPRITE_SELF_TINT_SWATCH, sp.self_tint),
    ] {
        let committed = crate::state_tint::tint_f32_to_u8(chan);
        if picker_target == Some(swatch_id) {
            if let Some(picked) = host.store().widget_color(swatch_id)
                && picked != committed
            {
                let new_chan = crate::state_tint::tint_u8_to_f32(picked);
                let edit = if swatch_id == crate::ids::INSP_SPRITE_TINT_SWATCH {
                    SpriteFieldEdit::Tint(new_chan)
                } else {
                    SpriteFieldEdit::SelfTint(new_chan)
                };
                host.bus_mut().push(EditorAction::InspectorSpriteEdit {
                    entity_bits: sp.entity_bits,
                    edit,
                });
            }
        } else {
            host.store_mut().set_widget_color(swatch_id, committed);
        }
    }
    // Per-corner tint swatches (TL, TR, BL, BR). Same regime as Tint/Self.
    //
    // ⚠️ **Um canto por edição** (`PerCornerTintAt`), e não o array inteiro. Enquanto o edit
    // carregava os quatro cantos da PRIMÁRIA, o fan-out de BulkSelect atropelava os cantos
    // divergentes de todas as outras — enquanto o painter pintava «Mixed» para esse mesmo estado.
    // *A promessa e o verbo discordavam* (auditoria `docs/Sprite_projeto/20` §3.2). É a lei que já
    // tinha criado `OffsetX`/`OffsetY` e `RegionX/Y/W/H`; faltava aplicá-la aqui.
    let corner_ids = [
        crate::ids::INSP_SPRITE_CORNER_TL,
        crate::ids::INSP_SPRITE_CORNER_TR,
        crate::ids::INSP_SPRITE_CORNER_BL,
        crate::ids::INSP_SPRITE_CORNER_BR,
    ];
    for (i, &corner_id) in corner_ids.iter().enumerate() {
        let committed = crate::state_tint::tint_f32_to_u8(sp.per_corner_tint[i]);
        if picker_target == Some(corner_id) {
            if let Some(picked) = host.store().widget_color(corner_id)
                && picked != committed
            {
                host.bus_mut().push(EditorAction::InspectorSpriteEdit {
                    entity_bits: sp.entity_bits,
                    edit: SpriteFieldEdit::PerCornerTintAt(
                        u8::try_from(i).unwrap_or(0),
                        crate::state_tint::tint_u8_to_f32(picked),
                    ),
                });
            }
        } else {
            host.store_mut().set_widget_color(corner_id, committed);
        }
    }
}

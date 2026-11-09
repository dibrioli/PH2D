//! **O cartão Animation sobre o Pixel Sprite** (P14a-4, `docs/PixelArt/P14_plano.md` §6): os clips vêm das tags
//! do documento; o cartão mostra a lista (só leitura), Who Picks (Me = Autoplay) e Starts with, e não emite
//! nenhum editor que escreveria os clips ou a grelha derivados.

use crate::o_inspector_em_cartoes_por_tipo::{BITS, limpa, objeto, pinta};
use crate::seam_anim::anim;
use ph2d_a11y::NodeId;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::InspectorAnimInfo;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::ids::*;
use ph2d_panel_inspector::{
    InspectorPanel, InspectorState, set_cards_on, set_current_inspector_anim,
};
use ph2d_ui_testkit::MockPanelHost;

fn do_pixel() -> InspectorAnimInfo {
    let mut a = anim(true, true);
    a.entity_bits = BITS;
    a.clips_do_documento = true;
    a
}

fn pinta_com(cartoes: bool, a: InspectorAnimInfo) -> Vec<(NodeId, Rect)> {
    limpa();
    set_cards_on(cartoes);
    objeto();
    set_current_inspector_anim(Some(a));
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut st = InspectorState::default();
    host.store_mut().set_collapsed(INSP_ANIM_ADVANCED, false);
    let r = pinta(&mut host, &mut st);
    limpa();
    r
}

fn emitidos(r: &[(NodeId, Rect)], ids: &[NodeId]) -> Vec<NodeId> {
    ids.iter()
        .copied()
        .filter(|id| r.iter().any(|(i, q)| i == id && q.w > 0.0))
        .collect()
}

/// Todo id que escreve `SpriteAnimations`/`SpriteGrid` (ou edita o clip) no cartão e na saída de referência.
fn editores_de_clip() -> Vec<NodeId> {
    let mut v = vec![
        INSP_ANIM_ADD,
        INSP_ANIM_REMOVE,
        INSP_ANIM_OPEN_EDITOR,
        INSP_ANIM_NAME,
        INSP_ANIM_FROM,
        INSP_ANIM_TO,
        INSP_ANIM_FPS,
        INSP_ANIM_HOLD_MS,
        INSP_ANIM_DELAY_MS,
        INSP_ANIM_REPEAT,
        INSP_ANIM_FRAME_MS_THIS,
        INSP_ANIM_SIGNAL_FINISH,
        INSP_ANIM_SIGNAL_LOOP,
        INSP_ANIM_ADD_PLAYER,
        INSP_ANIM_PLAYING,
        INSP_ANIM_SPEED,
        INSP_ANIM_REWIND,
        INSP_ANIM_FRAME_SCRUB,
        INSP_ANIM_ADVANCED,
        INSP_ANIM_MARKER_ADD,
        INSP_ANIM_REMOVE_EMPTY_FRAMES,
        INSP_ANIM_DELETE_EMPTY_ANIMS,
        INSP_IMAGE_SHEET_LOAD,
    ];
    v.extend(INSP_ANIM_ROW);
    v.extend(INSP_ANIM_PLAY_ROW);
    v.extend(INSP_ANIM_CYCLE);
    v.extend(INSP_ANIM_DIR);
    v.extend(INSP_ANIM_DIR_OVERRIDE);
    v.extend(INSP_ANIM_LOOP_OVERRIDE);
    v.extend(INSP_ANIM_MARKER_FRAME);
    v.extend(INSP_ANIM_MARKER_SIGNAL);
    v.extend(INSP_ANIM_MARKER_DEL);
    v
}

/// Pintado e registado: Who Picks (Me · Script · Rules) e o menu Starts with.
#[test]
fn o_cartao_do_pixel_sprite_pinta_autoplay_e_starts_with() {
    let r = pinta_com(true, do_pixel());
    assert_eq!(emitidos(&r, &INSP_ANIM_CHOOSER).len(), 3, "Who Picks");
    assert_eq!(emitidos(&r, &[INSP_ANIM_START_DD]).len(), 1, "Starts with");
}

/// Censo: com os clips do documento, nenhum editor de clip é emitido — nos cartões e na saída de referência.
/// Controlo: o Animated Sprite comum emite-os.
#[test]
fn o_pixel_sprite_nao_emite_editor_de_clip() {
    let ed = editores_de_clip();
    for cartoes in [true, false] {
        let r = pinta_com(cartoes, do_pixel());
        assert_eq!(
            emitidos(&r, &ed),
            Vec::<NodeId>::new(),
            "cartões={cartoes}: editores de clip emitidos"
        );
        assert!(
            !emitidos(&r, &INSP_ANIM_CHOOSER).is_empty(),
            "cartões={cartoes}: o Who Picks fica"
        );
        let controlo = pinta_com(cartoes, {
            let mut a = anim(true, true);
            a.entity_bits = BITS;
            a
        });
        assert!(
            emitidos(&controlo, &ed).len() > 5,
            "cartões={cartoes}: o controlo emite os editores"
        );
    }
}

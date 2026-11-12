//! **O cartão do Top-Down Player, versão A** (ficha `docs/GameEngineFuncional/componentes/05_topdown.md`,
//! dono 12/11): todo clique passa pelo ponteiro REAL sobre o rect pintado, nos cartões e fora deles.

use crate::a_seccao_topdown_mostra_o_objecto::mover;
use crate::o_cartao_da_fisica::{achado, clica};
use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::topdown_edits::{
    InspectorFacing, InspectorMoveDirections, InspectorTopDownInfo, InspectorViewpoint,
    TopDownFieldEdit,
};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::ids as i;
use ph2d_panel_inspector::{
    InspectorPanel, InspectorState, set_cards_on, set_current_inspector_topdown,
};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 3200.0,
};

/// Os campos que moram no More.
const DENTRO: [NodeId; 5] = [
    i::INSP_TD_ACCEL,
    i::INSP_TD_DECEL,
    i::INSP_TD_KNOCKBACK_RECOVERY,
    i::INSP_TD_MIN_SLIDE,
    i::INSP_TD_DEFAULT_CONTROLS,
];

fn nos_dois_modos(f: impl Fn(bool)) {
    for on in [true, false] {
        set_cards_on(on);
        f(on);
    }
    set_cards_on(false);
}

/// O host assente com `info`; `mais` abre a dobra More.
fn assente(
    info: &InspectorTopDownInfo,
    cartoes: bool,
    mais: bool,
) -> (MockPanelHost, InspectorState, Vec<(NodeId, Rect)>) {
    let mut h = crate::host_em_metros();
    h.store_mut()
        .set_collapsed(ph2d_editor_core::ids::INSP_LIVE_TOPDOWN_SECTION, false);
    let mut st = InspectorState::default();
    set_current_inspector_topdown(Some(info.clone()));
    let _ = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    h.set_collapsed(i::INSP_TD_FOLD_MORE, !mais);
    let r = if cartoes {
        crate::o_cartao_da_fisica::pinta(&mut h, &mut st)
    } else {
        h.settle_section_folds();
        h.paint::<InspectorPanel>(&mut st, VIEWPORT)
    };
    (h, st, r)
}

fn edicoes(a: Vec<EditorAction>) -> Vec<TopDownFieldEdit> {
    a.into_iter()
        .filter_map(|a| match a {
            EditorAction::InspectorComponentEdit {
                edit: ComponentEdit::TopDown(e),
                ..
            } => Some(e),
            _ => None,
        })
        .collect()
}

/// Clica no MEIO do rect pintado de `id` e devolve as edições + o foco.
fn clique(
    info: &InspectorTopDownInfo,
    id: NodeId,
    cartoes: bool,
    mais: bool,
) -> (Vec<TopDownFieldEdit>, Option<NodeId>) {
    let (mut h, mut st, r) = assente(info, cartoes, mais);
    let rect =
        achado(&r, id).unwrap_or_else(|| panic!("{id:?} não foi pintado (cartões {cartoes})"));
    let out = edicoes(clica(&mut h, &mut st, rect));
    let foco = h.store().focus_id();
    set_current_inspector_topdown(None);
    (out, foco)
}

/// ⭐ (a) Cada segmento de Directions e de Facing escreve o SEU modo.
#[test]
fn directions_e_facing_escrevem_o_modo_clicado() {
    nos_dois_modos(|c| {
        for (n, &id) in i::INSP_TD_DIRECTIONS.iter().enumerate() {
            let (e, _) = clique(&mover(), id, c, false);
            assert_eq!(
                e,
                vec![TopDownFieldEdit::Directions(
                    InspectorMoveDirections::ALL[n]
                )]
            );
        }
        for (n, &id) in i::INSP_TD_FACING.iter().enumerate() {
            let (e, _) = clique(&mover(), id, c, false);
            assert_eq!(e, vec![TopDownFieldEdit::Facing(InspectorFacing::ALL[n])]);
        }
    });
}

/// ⭐ (a) Viewpoint (no More aberto) escreve o modo clicado.
#[test]
fn viewpoint_no_more_escreve_o_modo_clicado() {
    nos_dois_modos(|c| {
        for (n, &id) in i::INSP_TD_VIEWPOINT.iter().enumerate() {
            let (e, _) = clique(&mover(), id, c, true);
            assert_eq!(
                e,
                vec![TopDownFieldEdit::Viewpoint(InspectorViewpoint::ALL[n])]
            );
        }
    });
}

/// ⭐ (a) Os números (Speed à vista, Turn quando gira, Board Angle em Custom) ganham o foco.
#[test]
fn os_numeros_ganham_o_foco_sob_o_dedo() {
    let gira = InspectorTopDownInfo {
        facing: InspectorFacing::Movement,
        viewpoint: InspectorViewpoint::Custom,
        ..mover()
    };
    nos_dois_modos(|c| {
        for (id, mais) in [
            (i::INSP_TD_SPEED, false),
            (i::INSP_TD_TURN_SPEED, false),
            (i::INSP_TD_VIEW_ANGLE, true),
            (i::INSP_TD_ACCEL, true),
        ] {
            let (_, foco) = clique(&gira, id, c, mais);
            assert_eq!(foco, Some(id), "{id:?} está morto sob o dedo (cartões {c})");
        }
    });
}

/// ⭐ (a) «Turn on» liga os controlos; a caixa Default Controls (no More) alterna.
#[test]
fn turn_on_e_a_caixa_default_controls_escrevem_o_componente() {
    let desligado = InspectorTopDownInfo {
        default_controls: false,
        ..mover()
    };
    nos_dois_modos(|c| {
        let (e, _) = clique(&desligado, i::INSP_TD_TURN_ON_CONTROLS, c, false);
        assert_eq!(e, vec![TopDownFieldEdit::DefaultControls(true)], "Turn on");
        let (e, _) = clique(&mover(), i::INSP_TD_DEFAULT_CONTROLS, c, true);
        assert_eq!(
            e,
            vec![TopDownFieldEdit::DefaultControls(false)],
            "desmarcar"
        );
        let (e, _) = clique(&desligado, i::INSP_TD_DEFAULT_CONTROLS, c, true);
        assert_eq!(e, vec![TopDownFieldEdit::DefaultControls(true)], "marcar");
        // O botão só existe com a caixa desligada.
        let (_, _, r) = assente(&mover(), c, false);
        assert!(
            achado(&r, i::INSP_TD_TURN_ON_CONTROLS).is_none(),
            "Turn on com os controlos ligados"
        );
        set_current_inspector_topdown(None);
    });
}

/// ⭐ (b) O More nasce fechado, abre e fecha pelo clique no cabeçalho, e os campos aparecem e somem.
#[test]
fn a_dobra_more_abre_e_fecha_pelo_clique() {
    nos_dois_modos(|c| {
        let mut h = crate::host_em_metros();
        h.store_mut()
            .set_collapsed(ph2d_editor_core::ids::INSP_LIVE_TOPDOWN_SECTION, false);
        let mut st = InspectorState::default();
        set_current_inspector_topdown(Some(mover()));
        let pinta = |h: &mut MockPanelHost, st: &mut InspectorState| {
            if c {
                crate::o_cartao_da_fisica::pinta(h, st)
            } else {
                let _ = h.paint::<InspectorPanel>(st, VIEWPORT);
                h.settle_section_folds();
                h.paint::<InspectorPanel>(st, VIEWPORT)
            }
        };
        let mut r = pinta(&mut h, &mut st);
        for id in DENTRO {
            assert!(
                achado(&r, id).is_none(),
                "{id:?} à vista com o More fechado de fábrica"
            );
        }
        for aberto in [true, false] {
            let cab = achado(&r, i::INSP_TD_FOLD_MORE).expect("o cabeçalho More");
            let _ = clica(&mut h, &mut st, cab);
            r = pinta(&mut h, &mut st);
            for id in DENTRO {
                assert_eq!(
                    achado(&r, id).is_some(),
                    aberto,
                    "{id:?} com o More aberto={aberto} (cartões {c})"
                );
            }
        }
        set_current_inspector_topdown(None);
    });
}

/// ⭐ (b) Board Angle só em Custom; Turn só quando gira.
#[test]
fn board_angle_so_em_custom_e_turn_so_se_gira() {
    nos_dois_modos(|c| {
        for v in InspectorViewpoint::ALL {
            let (_, _, r) = assente(
                &InspectorTopDownInfo {
                    viewpoint: v,
                    ..mover()
                },
                c,
                true,
            );
            assert_eq!(
                achado(&r, i::INSP_TD_VIEW_ANGLE).is_some(),
                v == InspectorViewpoint::Custom,
                "{v:?}"
            );
        }
        for f in InspectorFacing::ALL {
            let (_, _, r) = assente(
                &InspectorTopDownInfo {
                    facing: f,
                    ..mover()
                },
                c,
                false,
            );
            assert_eq!(
                achado(&r, i::INSP_TD_TURN_SPEED).is_some(),
                f != InspectorFacing::None,
                "{f:?}"
            );
        }
        set_current_inspector_topdown(None);
    });
}

fn desenho(info: InspectorTopDownInfo) -> Vec<u32> {
    let (mut h, mut st, _) = assente(&info, false, false);
    // Os glifos pintados: o resumo é TEXTO, e as cores dele não mudam com a frase.
    let cena = h.paint_vector_scene::<InspectorPanel>(&mut st, VIEWPORT);
    let d = cena
        .inner()
        .encoding()
        .resources
        .glyphs
        .iter()
        .map(|g| g.id)
        .collect();
    set_current_inspector_topdown(None);
    d
}

/// ⭐ (c) O resumo do More fechado muda com a rampa (No easing ↔ Eased) e com o Viewpoint — e só
/// com eles: dois «Eased» diferentes pintam igual (o controlo).
#[test]
fn o_resumo_do_more_muda_com_a_rampa_e_a_vista() {
    let com = |a: f32, d: f32, v| InspectorTopDownInfo {
        acceleration: a,
        deceleration: d,
        viewpoint: v,
        ..mover()
    };
    let top = InspectorViewpoint::TopDown;
    let sem = desenho(com(0.0, 0.0, top));
    let eased = desenho(com(5.0, 0.0, top));
    assert_ne!(sem, eased, "accel 0 → 5 não mudou o resumo");
    assert_ne!(
        sem,
        desenho(com(0.0, 3.0, top)),
        "decel 0 → 3 não mudou o resumo"
    );
    assert_eq!(
        eased,
        desenho(com(9.0, 2.0, top)),
        "o controlo: dois Eased pintam igual"
    );
    assert_ne!(
        sem,
        desenho(com(0.0, 0.0, InspectorViewpoint::Iso30)),
        "o Viewpoint não muda o resumo"
    );
}

/// ⭐ Toda linha (cada segmento, cada número, a caixa, o botão) e a própria dobra têm dica de hover.
#[test]
fn toda_linha_e_a_dobra_tem_dica_de_hover() {
    let h = crate::host_em_metros();
    let ids = i::INSP_TD_DIRECTIONS
        .iter()
        .chain(&i::INSP_TD_FACING)
        .chain(&i::INSP_TD_VIEWPOINT)
        .chain(&[
            i::INSP_TD_SPEED,
            i::INSP_TD_TURN_SPEED,
            i::INSP_TD_ACCEL,
            i::INSP_TD_DECEL,
            i::INSP_TD_VIEW_ANGLE,
            i::INSP_TD_KNOCKBACK_RECOVERY,
            i::INSP_TD_MIN_SLIDE,
            i::INSP_TD_MAX_SLIDES,
            i::INSP_TD_DEFAULT_CONTROLS,
            i::INSP_TD_TURN_ON_CONTROLS,
            i::INSP_TD_FOLD_MORE,
        ]);
    for &id in ids {
        assert!(
            h.store().tooltip_for(id).is_some_and(|t| !t.is_empty()),
            "{id:?} sem dica de hover"
        );
    }
    // O nome inteiro do Facing mora na dica do botão curto.
    let still = h
        .store()
        .tooltip_for(i::INSP_TD_FACING[0])
        .unwrap_or_default();
    assert!(still.starts_with("Don't turn"), "{still}");
}

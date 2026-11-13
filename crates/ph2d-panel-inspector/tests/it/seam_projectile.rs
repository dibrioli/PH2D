//! **O cartão do Projectile Motion, versão B** (ficha `docs/GameEngineFuncional/componentes/06_projectile.md`,
//! dono 13/11): todo clique passa pelo ponteiro REAL sobre o rect pintado, nos cartões e fora deles.

use crate::o_cartao_da_fisica::{achado, clica};
use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::projectile_edits::{InspectorProjectileInfo, ProjectileFieldEdit};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::ids as i;
use ph2d_panel_inspector::{
    InspectorPanel, InspectorState, set_cards_on, set_current_inspector_projectile,
};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 3200.0,
};

/// A bala de fábrica: reta, sem ricochete, sem perseguição, sem mudança de rapidez.
fn bala() -> InspectorProjectileInfo {
    InspectorProjectileInfo {
        entity_bits: 0x00B0_1E7A,
        initial_speed: 12.0,
        acceleration: 0.0,
        max_speed: 0.0,
        gravity: 0.0,
        bounciness: 1.0,
        max_bounces: 0,
        range: 0.0,
        face_velocity: true,
        homing_accel: 0.0,
        homing_target: String::new(),
        homing_target_missing: false,
        body_is_kinematic: true,
        has_body: true,
        clock_playing: true,
        flight_over: false,
        target_pick_armed: false,
        selected_count: 1,
    }
}

/// A bala com as duas caixas ligadas.
fn cheia() -> InspectorProjectileInfo {
    InspectorProjectileInfo {
        max_bounces: 3,
        bounciness: 0.8,
        homing_accel: 30.0,
        homing_target: "Hero".into(),
        ..bala()
    }
}

/// As linhas que só existem com Bounce, e as que só existem com Homing.
const DO_RICOCHETE: [NodeId; 2] = [i::INSP_PJ_MAX_BOUNCES, i::INSP_PJ_BOUNCINESS];
const DA_PERSEGUICAO: [NodeId; 3] = [
    i::INSP_PJ_HOMING_TARGET,
    i::INSP_PJ_TARGET_PICK,
    i::INSP_PJ_HOMING_ACCEL,
];
/// As que moram no Speed change.
const DENTRO: [NodeId; 2] = [i::INSP_PJ_ACCEL, i::INSP_PJ_MAX_SPEED];

fn nos_dois_modos(f: impl Fn(bool)) {
    for on in [true, false] {
        set_cards_on(on);
        f(on);
    }
    set_cards_on(false);
}

fn pinta(h: &mut MockPanelHost, st: &mut InspectorState, cartoes: bool) -> Vec<(NodeId, Rect)> {
    if cartoes {
        crate::o_cartao_da_fisica::pinta(h, st)
    } else {
        let _ = h.paint::<InspectorPanel>(st, VIEWPORT);
        h.settle_section_folds();
        h.paint::<InspectorPanel>(st, VIEWPORT)
    }
}

/// O host assente com `info`; `aberta` abre a dobra Speed change.
fn assente(
    info: &InspectorProjectileInfo,
    cartoes: bool,
    aberta: bool,
) -> (MockPanelHost, InspectorState, Vec<(NodeId, Rect)>) {
    let mut h = crate::host_em_metros();
    h.store_mut()
        .set_collapsed(ph2d_editor_core::ids::INSP_LIVE_PROJECTILE_SECTION, false);
    let mut st = InspectorState::default();
    set_current_inspector_projectile(Some(info.clone()));
    let _ = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    h.set_collapsed(i::INSP_PJ_FOLD_SPEED_CHANGE, !aberta);
    let r = pinta(&mut h, &mut st, cartoes);
    (h, st, r)
}

fn edicoes(a: Vec<EditorAction>) -> Vec<ProjectileFieldEdit> {
    a.into_iter()
        .filter_map(|a| match a {
            EditorAction::InspectorComponentEdit {
                edit: ComponentEdit::Projectile(e),
                ..
            } => Some(e),
            _ => None,
        })
        .collect()
}

/// Clica no MEIO do rect pintado de `id` e devolve as edições + o foco.
fn clique(
    info: &InspectorProjectileInfo,
    id: NodeId,
    cartoes: bool,
    aberta: bool,
) -> (Vec<ProjectileFieldEdit>, Option<NodeId>) {
    let (mut h, mut st, r) = assente(info, cartoes, aberta);
    let rect =
        achado(&r, id).unwrap_or_else(|| panic!("{id:?} não foi pintado (cartões {cartoes})"));
    let out = edicoes(clica(&mut h, &mut st, rect));
    let foco = h.store().focus_id();
    set_current_inspector_projectile(None);
    (out, foco)
}

/// ⭐ (a) As três caixas escrevem o contrário do que o ecrã mostra: Bounce, Homing e Faces path.
#[test]
fn as_caixas_escrevem_o_contrario_do_ecra() {
    nos_dois_modos(|c| {
        for (info, id, esperado) in [
            (bala(), i::INSP_PJ_BOUNCE, ProjectileFieldEdit::Bounce(true)),
            (
                cheia(),
                i::INSP_PJ_BOUNCE,
                ProjectileFieldEdit::Bounce(false),
            ),
            (bala(), i::INSP_PJ_HOMING, ProjectileFieldEdit::Homing(true)),
            (
                cheia(),
                i::INSP_PJ_HOMING,
                ProjectileFieldEdit::Homing(false),
            ),
            (
                bala(),
                i::INSP_PJ_FACE_VELOCITY,
                ProjectileFieldEdit::FaceVelocity(false),
            ),
        ] {
            let (e, _) = clique(&info, id, c, false);
            assert_eq!(e, vec![esperado], "{id:?} (cartões {c})");
        }
    });
}

/// ⭐ (a) O conta-gotas arma e, armado, desarma — e ganha o clique à caixa de texto onde mora.
#[test]
fn o_conta_gotas_arma_e_desarma_e_mora_na_caixa_do_target() {
    nos_dois_modos(|c| {
        let (e, foco) = clique(&cheia(), i::INSP_PJ_TARGET_PICK, c, false);
        assert_eq!(e, vec![ProjectileFieldEdit::PickTarget(true)], "armar");
        assert_ne!(
            foco,
            Some(i::INSP_PJ_HOMING_TARGET),
            "o texto roubou o clique"
        );
        let armada = InspectorProjectileInfo {
            target_pick_armed: true,
            ..cheia()
        };
        let (e, _) = clique(&armada, i::INSP_PJ_TARGET_PICK, c, false);
        assert_eq!(e, vec![ProjectileFieldEdit::PickTarget(false)], "desarmar");
        let (_, _, r) = assente(&cheia(), c, false);
        let (caixa, gotas) = (
            achado(&r, i::INSP_PJ_HOMING_TARGET).expect("Target"),
            achado(&r, i::INSP_PJ_TARGET_PICK).expect("conta-gotas"),
        );
        assert!(
            gotas.x >= caixa.x
                && gotas.x + gotas.w <= caixa.x + caixa.w + 0.5
                && gotas.y >= caixa.y
                && gotas.y + gotas.h <= caixa.y + caixa.h + 0.5,
            "o conta-gotas {gotas:?} fora da caixa {caixa:?} (cartões {c})"
        );
        set_current_inspector_projectile(None);
    });
}

/// ⭐ (a) Todo número e o Target ganham o foco sob o dedo.
#[test]
fn os_campos_ganham_o_foco_sob_o_dedo() {
    nos_dois_modos(|c| {
        for (id, aberta) in [
            (i::INSP_PJ_SPEED, false),
            (i::INSP_PJ_GRAVITY, false),
            (i::INSP_PJ_RANGE, false),
            (i::INSP_PJ_MAX_BOUNCES, false),
            (i::INSP_PJ_BOUNCINESS, false),
            (i::INSP_PJ_HOMING_ACCEL, false),
            (i::INSP_PJ_ACCEL, true),
            (i::INSP_PJ_MAX_SPEED, true),
        ] {
            let (_, foco) = clique(&cheia(), id, c, aberta);
            assert_eq!(foco, Some(id), "{id:?} está morto sob o dedo (cartões {c})");
        }
        let (h, mut st, r) = assente(&cheia(), c, false);
        let mut h = h;
        let alvo = achado(&r, i::INSP_PJ_HOMING_TARGET).expect("Target");
        // À esquerda do conta-gotas: o texto.
        let texto = Rect {
            w: alvo.w * 0.4,
            ..alvo
        };
        let _ = clica(&mut h, &mut st, texto);
        assert_eq!(h.store().focus_id(), Some(i::INSP_PJ_HOMING_TARGET));
        set_current_inspector_projectile(None);
    });
}

/// ⭐ (b) As linhas de dentro só existem com a caixa ligada.
#[test]
fn as_linhas_de_dentro_so_existem_com_a_caixa_ligada() {
    nos_dois_modos(|c| {
        for (info, ricochete, perseguicao) in [
            (bala(), false, false),
            (
                InspectorProjectileInfo {
                    max_bounces: 2,
                    ..bala()
                },
                true,
                false,
            ),
            (
                InspectorProjectileInfo {
                    homing_accel: 5.0,
                    ..bala()
                },
                false,
                true,
            ),
            (cheia(), true, true),
        ] {
            let (_, _, r) = assente(&info, c, false);
            for id in DO_RICOCHETE {
                assert_eq!(achado(&r, id).is_some(), ricochete, "{id:?} (cartões {c})");
            }
            for id in DA_PERSEGUICAO {
                assert_eq!(
                    achado(&r, id).is_some(),
                    perseguicao,
                    "{id:?} (cartões {c})"
                );
            }
            for id in [i::INSP_PJ_BOUNCE, i::INSP_PJ_HOMING, i::INSP_PJ_SPEED] {
                assert!(achado(&r, id).is_some(), "{id:?} sempre à vista");
            }
        }
        set_current_inspector_projectile(None);
    });
}

/// ⭐ (b) O Speed change nasce fechado, abre e fecha pelo clique no cabeçalho, e os campos aparecem e somem.
#[test]
fn a_dobra_speed_change_abre_e_fecha_pelo_clique() {
    nos_dois_modos(|c| {
        let mut h = crate::host_em_metros();
        h.store_mut()
            .set_collapsed(ph2d_editor_core::ids::INSP_LIVE_PROJECTILE_SECTION, false);
        let mut st = InspectorState::default();
        set_current_inspector_projectile(Some(bala()));
        let mut r = pinta(&mut h, &mut st, c);
        for id in DENTRO {
            assert!(achado(&r, id).is_none(), "{id:?} à vista de fábrica");
        }
        for aberta in [true, false] {
            let cab = achado(&r, i::INSP_PJ_FOLD_SPEED_CHANGE).expect("o cabeçalho Speed change");
            let _ = clica(&mut h, &mut st, cab);
            r = pinta(&mut h, &mut st, c);
            for id in DENTRO {
                assert_eq!(
                    achado(&r, id).is_some(),
                    aberta,
                    "{id:?} com o Speed change aberto={aberta} (cartões {c})"
                );
            }
        }
        set_current_inspector_projectile(None);
    });
}

fn desenho(info: InspectorProjectileInfo) -> Vec<u32> {
    let (mut h, mut st, _) = assente(&info, false, false);
    let cena = h.paint_vector_scene::<InspectorPanel>(&mut st, VIEWPORT);
    let d = cena
        .inner()
        .encoding()
        .resources
        .glyphs
        .iter()
        .map(|g| g.id)
        .collect();
    set_current_inspector_projectile(None);
    d
}

/// ⭐ (c) O resumo do Speed change fechado muda com a aceleração e o tecto — e só com o sentido deles: duas
/// acelerações positivas pintam igual (o controlo).
#[test]
fn o_resumo_do_speed_change_muda_com_a_aceleracao_e_o_tecto() {
    let com = |a: f32, m: f32| InspectorProjectileInfo {
        acceleration: a,
        max_speed: m,
        ..bala()
    };
    let nada = desenho(com(0.0, 0.0));
    let sobe = desenho(com(5.0, 0.0));
    assert_ne!(nada, sobe, "accel 0 → 5 não mudou o resumo");
    assert_ne!(sobe, desenho(com(-5.0, 0.0)), "subir e descer pintam igual");
    assert_ne!(nada, desenho(com(0.0, 20.0)), "o tecto não muda o resumo");
    assert_ne!(
        sobe,
        desenho(com(5.0, 20.0)),
        "o tecto com accel não muda o resumo"
    );
    assert_eq!(
        sobe,
        desenho(com(9.0, 0.0)),
        "o controlo: dois Speeds up pintam igual"
    );
}

/// ⭐ Toda linha nova (caixas, conta-gotas, Speed, Max Bounces, Target) e a dobra têm dica de hover.
#[test]
fn toda_linha_e_a_dobra_tem_dica_de_hover() {
    let h = crate::host_em_metros();
    for id in [
        i::INSP_PJ_SPEED,
        i::INSP_PJ_GRAVITY,
        i::INSP_PJ_RANGE,
        i::INSP_PJ_FACE_VELOCITY,
        i::INSP_PJ_BOUNCE,
        i::INSP_PJ_MAX_BOUNCES,
        i::INSP_PJ_BOUNCINESS,
        i::INSP_PJ_HOMING,
        i::INSP_PJ_HOMING_TARGET,
        i::INSP_PJ_TARGET_PICK,
        i::INSP_PJ_HOMING_ACCEL,
        i::INSP_PJ_ACCEL,
        i::INSP_PJ_MAX_SPEED,
        i::INSP_PJ_FOLD_SPEED_CHANGE,
    ] {
        assert!(
            h.store().tooltip_for(id).is_some_and(|t| !t.is_empty()),
            "{id:?} sem dica de hover"
        );
    }
}

/// O texto que o store guarda para `id`.
fn texto(h: &MockPanelHost, id: NodeId) -> Option<String> {
    match h.store().get(id) {
        Some(ph2d_editor_core::interaction::InteractiveState::TextInput { text, .. }) => {
            Some(text.clone())
        }
        _ => None,
    }
}

/// ⭐ (f) O Target MOSTRA o alvo: o nome que o instantâneo resolveu chega ao campo pintado, e trocar de
/// alvo troca o texto. (Mutação: tirar o `escreve_texto` do `sync_fabrica` ⇒ o campo fica vazio e reprova.)
#[test]
fn o_target_mostra_o_nome_do_alvo() {
    nos_dois_modos(|c| {
        let (mut h, mut st, _) = assente(&cheia(), c, false);
        assert_eq!(
            texto(&h, i::INSP_PJ_HOMING_TARGET).as_deref(),
            Some("Hero"),
            "cartões {c}"
        );
        let outro = InspectorProjectileInfo {
            homing_target: "Boss".into(),
            ..cheia()
        };
        set_current_inspector_projectile(Some(outro));
        let _ = pinta(&mut h, &mut st, c);
        assert_eq!(
            texto(&h, i::INSP_PJ_HOMING_TARGET).as_deref(),
            Some("Boss"),
            "cartões {c}"
        );
        set_current_inspector_projectile(None);
    });
}

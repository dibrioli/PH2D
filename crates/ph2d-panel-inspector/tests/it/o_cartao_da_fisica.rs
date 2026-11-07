//! ⭐ **Os cartões Body e Shape não perdem poder** ([ficha f1](../../../../docs/GameEngineFuncional/componentes/f1_physics_static_shape.md)):
//! a varredura da tabela `M` da maquete. Em cada tipo, com as dobras abertas, cada id do mapa está pintado,
//! e o clique ou o número real chega ao barramento com a edição certa. O efeito de cada edição no componente é
//! provado em `ph2d-app-physics` (`physics_tests`).

use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::{
    InspectorPhysicsInfo, InspectorTransformInfo, PhysicsFieldEdit as E,
};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind};
use ph2d_i18n::tr;
use ph2d_panel_inspector::ids::*;
use ph2d_panel_inspector::{
    InspectorPanel, InspectorState, set_cards_on, set_current_inspector_physics,
    set_current_inspector_sprite, set_current_inspector_transform,
};
use ph2d_ui_testkit::MockPanelHost;

use crate::seam_physics::{as_part, with_body};

const ENTIDADE: u64 = 0xABCD_1234;

fn pose(scale: [f32; 2]) -> InspectorTransformInfo {
    InspectorTransformInfo {
        entity_bits: ENTIDADE,
        translation: [0.0, 0.0],
        rotation_rad: 0.0,
        scale,
        skew_rad: [0.0, 0.0],
    }
}

/// O Inspector em cartões, em metros, com os dois cartões e todas as dobras abertos.
pub(crate) fn cena(info: InspectorPhysicsInfo, scale: [f32; 2]) -> (MockPanelHost, InspectorState) {
    set_cards_on(true);
    set_current_inspector_sprite(None);
    set_current_inspector_physics(Some(info));
    set_current_inspector_transform(Some(pose(scale)));
    let mut host = crate::host_em_metros();
    for id in INSP_PHYS_FOLDS
        .into_iter()
        .chain([core_ids::INSP_LIVE_PHYSICS_SECTION, CARD_PHYS_SHAPE])
    {
        host.store_mut().set_collapsed(id, false);
    }
    (host, InspectorState::default())
}

pub(crate) fn sai() {
    set_cards_on(false);
    set_current_inspector_physics(None);
    set_current_inspector_transform(None);
}

pub(crate) fn pinta(host: &mut MockPanelHost, st: &mut InspectorState) -> Vec<(NodeId, Rect)> {
    use ph2d_editor_core::screens::layout::{CenterSplit, ChromeBands, DockSides, HeroLayout};
    let janela = Rect::new(0.0, 0.0, 1600.0, 3200.0);
    // O 1.º quadro semeia as dobras (`cards::prepare`); só depois há o que assentar.
    let _ = host.paint_with_layout::<InspectorPanel>(
        st,
        HeroLayout::for_viewport_bands(
            janela,
            false,
            ChromeBands::DEFAULT,
            CenterSplit::None,
            DockSides::BOTH,
        ),
        janela,
    );
    host.settle_section_folds();
    let mut r = Vec::new();
    for _ in 0..60 {
        let layout = HeroLayout::for_viewport_bands(
            janela,
            false,
            ChromeBands::DEFAULT,
            CenterSplit::None,
            DockSides::BOTH,
        );
        r = ph2d_editor_core::aquecimento::com_passo_fixo(
            ph2d_editor_core::aquecimento::PASSO_DE_QUADRO_S,
            || host.paint_with_layout::<InspectorPanel>(st, layout, janela),
        );
    }
    r
}

pub(crate) fn achado(r: &[(NodeId, Rect)], id: NodeId) -> Option<Rect> {
    r.iter()
        .rev()
        .find(|(i, q)| *i == id && q.w > 0.0)
        .map(|(_, q)| *q)
}

pub(crate) fn clica(host: &mut MockPanelHost, st: &mut InspectorState, r: Rect) -> Vec<EditorAction> {
    let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let _ = host.drained_actions();
    let mut eventos = Vec::new();
    for kind in [PointerKind::Move, PointerKind::Down, PointerKind::Up] {
        eventos.extend(host.dispatch_pointer_event(PointerEvent {
            x,
            y,
            pressure: 1.0,
            kind,
            source: ph2d_host::PointerSource::Mouse,
            button: PointerButton::Primary,
            timestamp_ns: 0,
        }));
    }
    for ev in eventos {
        let _ = host.apply_panel_event::<InspectorPanel>(st, ev);
    }
    host.drained_actions()
}

/// O que se faz a um id do mapa, e a edição que tem de sair.
enum G {
    Clica(E),
    Digita(&'static str, E),
    /// Texto e lista: pintado e focável; o evento tem varredura própria em `seam_physics`.
    Pintado,
}

fn opcoes(ids: &[NodeId], edit: impl Fn(usize) -> E) -> Vec<(NodeId, G)> {
    ids.iter()
        .enumerate()
        .map(|(i, id)| (*id, G::Clica(edit(i))))
        .collect()
}

/// Os ajustes de Shape, Touch e Material, e tudo o que vale em toda forma (tabela `M`).
fn da_forma() -> Vec<(NodeId, G)> {
    let mut m = opcoes(&INSP_PHYS_SHAPE, |i| E::Shape(i as u8));
    m.extend(opcoes(&INSP_PHYS_SENSOR, |i| E::Sensor(i == 1)));
    m.extend(opcoes(&INSP_PHYS_REST_COMBINE, |i| {
        E::RestitutionCombine(i as u8)
    }));
    m.extend(opcoes(&INSP_PHYS_FRIC_COMBINE, |i| {
        E::FrictionCombine(i as u8)
    }));
    m.extend(opcoes(&INSP_PHYS_WALLMAT, |i| E::NoWallCling(i == 1)));
    m.extend([
        (INSP_PHYS_HALF_X, G::Digita("2", E::HalfX(1.0))),
        (INSP_PHYS_HALF_Y, G::Digita("3", E::HalfY(1.5))),
        (INSP_PHYS_OFFSET_X, G::Digita("0.25", E::OffsetX(0.25))),
        (INSP_PHYS_OFFSET_Y, G::Digita("0.5", E::OffsetY(0.5))),
        (INSP_PHYS_RESTITUTION, G::Digita("0.5", E::Restitution(0.5))),
        (INSP_PHYS_FRICTION, G::Digita("0.75", E::Friction(0.75))),
        (INSP_PHYS_WALK_GRIP, G::Digita("2", E::WalkGrip(2.0))),
        (INSP_PHYS_WALK_BELT, G::Digita("3", E::WalkBelt(3.0))),
        (INSP_PHYS_SIGNAL, G::Pintado),
        (INSP_PHYS_SIGNAL_LEAVE, G::Pintado),
        (INSP_PHYS_SIGNAL_TAG, G::Pintado),
    ]);
    m
}

/// Os ajustes de todo corpo.
fn de_corpo() -> Vec<(NodeId, G)> {
    let mut m = opcoes(&INSP_PHYS_KIND, |i| E::Kind(i as u8));
    m.extend(opcoes(&INSP_PHYS_JOIN_KIND, |i| E::JoinKind(i as u8)));
    m.extend([
        (INSP_PHYS_JOIN_DRAW, G::Clica(E::JoinDraw)),
        (INSP_PHYS_REMOVE, G::Clica(E::Remove)),
    ]);
    m
}

/// O que só um corpo Dynamic tem.
fn de_dinamico() -> Vec<(NodeId, G)> {
    let mut m = opcoes(&INSP_PHYS_LOCKROT, |i| E::LockRotation(i == 1));
    m.extend(opcoes(&INSP_PHYS_MASSMODE, |i| E::MassMode(i == 1)));
    m.extend(opcoes(&INSP_PHYS_CCD, |i| E::Ccd(i == 1)));
    m.extend(opcoes(&INSP_PHYS_LOCKX, |i| E::LockPositionX(i == 1)));
    m.extend(opcoes(&INSP_PHYS_LOCKY, |i| E::LockPositionY(i == 1)));
    m.extend(opcoes(&INSP_PHYS_DAMPMODE, |i| E::DampMode(i as u8)));
    m.extend(opcoes(&INSP_PHYS_BAKE_CH, |i| E::BakeChannels(i as u8)));
    m.extend([
        (INSP_PHYS_BAKE, G::Clica(E::Bake)),
        (
            INSP_PHYS_GRAVITY_SCALE,
            G::Digita("2", E::GravityScale(2.0)),
        ),
        (INSP_PHYS_DENSITY, G::Digita("3", E::Density(3.0))),
        (INSP_PHYS_LINVEL_X, G::Digita("4", E::LinvelX(4.0))),
        (INSP_PHYS_LINVEL_Y, G::Digita("5", E::LinvelY(5.0))),
        (INSP_PHYS_DOMINANCE, G::Digita("3", E::Dominance(3))),
        (
            INSP_PHYS_LINEAR_DAMPING,
            G::Digita("0.5", E::LinearDamping(0.5)),
        ),
        (
            INSP_PHYS_ANGULAR_DAMPING,
            G::Digita("0.25", E::AngularDamping(0.25)),
        ),
        (
            INSP_PHYS_ANGVEL,
            G::Digita("180", E::Angvel(180.0_f32.to_radians())),
        ),
    ]);
    m
}

/// O que só a zona tem.
fn de_zona() -> Vec<(NodeId, G)> {
    let mut m = opcoes(&INSP_PHYS_FORCE_AXES, |i| E::ForceWorldAxes(i == 1));
    m.extend([
        (INSP_PHYS_FORCE_X, G::Digita("4", E::ForceX(4.0))),
        (INSP_PHYS_FORCE_Y, G::Digita("5", E::ForceY(5.0))),
        (INSP_PHYS_AREA_TORQUE, G::Digita("6", E::AreaTorque(6.0))),
        (
            INSP_PHYS_AREA_FALLOFF,
            G::Digita("0.5", E::AreaFalloff(0.5)),
        ),
        (INSP_PHYS_AREA_DRAG, G::Digita("2", E::AreaDrag(2.0))),
        (INSP_PHYS_AREA_DENSITY, G::Digita("3", E::AreaDensity(3.0))),
        (
            INSP_PHYS_AREA_FORM_DRAG,
            G::Digita("4", E::AreaFormDrag(4.0)),
        ),
    ]);
    m
}

/// Toda a varredura de um tipo: pintado, e o gesto real produz a edição.
fn varre(nome: &str, info: InspectorPhysicsInfo, mapa: Vec<(NodeId, G)>) {
    // Só a forma em uso tem as suas medidas: a caixa aqui; bola e cápsula têm teste próprio.
    assert_eq!(info.shape_tag, 1, "{nome}: o mapa é o de uma caixa");
    let (mut host, mut st) = cena(info, [1.0, 1.0]);
    for (id, gesto) in mapa {
        let r = pinta(&mut host, &mut st);
        let caixa = achado(&r, id);
        let caixa = caixa.unwrap_or_else(|| panic!("{nome}: {id:?} sumiu do cartão"));
        match gesto {
            G::Pintado => assert!(
                host.store().get(id).is_some(),
                "{nome}: {id:?} não está registado"
            ),
            G::Clica(edit) => {
                let a = clica(&mut host, &mut st, caixa);
                assert_eq!(
                    a,
                    [EditorAction::InspectorPhysicsEdit {
                        entity_bits: ENTIDADE,
                        edit,
                    }],
                    "{nome}: o clique em {id:?} não pediu {edit:?}"
                );
            }
            G::Digita(texto, edit) => {
                let _ = host.drained_actions();
                for ev in host.type_into_number(id, texto) {
                    let _ = host.apply_panel_event::<InspectorPanel>(&mut st, ev);
                }
                let a = host.drained_actions();
                assert!(
                    a.contains(&EditorAction::InspectorPhysicsEdit {
                        entity_bits: ENTIDADE,
                        edit,
                    }),
                    "{nome}: digitar {texto} em {id:?} deu {a:?}, não {edit:?}"
                );
            }
        }
    }
    sai();
}

fn soma(partes: Vec<Vec<(NodeId, G)>>) -> Vec<(NodeId, G)> {
    partes.into_iter().flatten().collect()
}

fn uma_via() -> Vec<(NodeId, G)> {
    opcoes(&INSP_PHYS_ONEWAY, |i| E::OneWay(i == 1))
}

fn estatico(kind_tag: u8) -> InspectorPhysicsInfo {
    InspectorPhysicsInfo {
        kind_tag,
        mass_is_read: false,
        ..with_body()
    }
}

fn zona() -> InspectorPhysicsInfo {
    InspectorPhysicsInfo {
        is_sensor: true,
        ..estatico(1)
    }
}

#[test]
fn nenhum_ajuste_do_mapa_some_em_dynamic() {
    let mapa = soma(vec![de_corpo(), de_dinamico(), da_forma(), uma_via()]);
    varre("Dynamic", with_body(), mapa);
}

#[test]
fn nenhum_ajuste_do_mapa_some_em_static_e_kinematic() {
    varre(
        "Static",
        estatico(1),
        soma(vec![de_corpo(), da_forma(), uma_via()]),
    );
    varre(
        "Kinematic",
        estatico(2),
        soma(vec![de_corpo(), da_forma(), uma_via()]),
    );
}

#[test]
fn nenhum_ajuste_do_mapa_some_na_zona() {
    varre(
        "Zona",
        zona(),
        soma(vec![de_corpo(), da_forma(), de_zona()]),
    );
    // A Zona Dynamic continua com o que é de Dynamic, Density incluída.
    let dinamica = InspectorPhysicsInfo {
        is_sensor: true,
        ..with_body()
    };
    varre(
        "Zona Dynamic",
        dinamica,
        soma(vec![de_corpo(), de_dinamico(), da_forma(), de_zona()]),
    );
}

#[test]
fn nenhum_ajuste_do_mapa_some_na_forma_sem_corpo() {
    let mut mapa = soma(vec![da_forma(), uma_via()]);
    mapa.extend([
        (INSP_PHYS_DENSITY, G::Digita("3", E::Density(3.0))),
        (INSP_PHYS_ADD, G::Clica(E::Add)),
        (INSP_PHYS_REMOVE, G::Clica(E::Remove)),
    ]);
    varre("Peça", as_part(), mapa);
    let solta = InspectorPhysicsInfo {
        part_owner: String::new(),
        ..as_part()
    };
    let mut mapa = soma(vec![da_forma(), uma_via()]);
    mapa.push((INSP_PHYS_ADD, G::Clica(E::Add)));
    varre("Solta", solta, mapa);
}

/// A massa manual troca Density por Mass; bola e cápsula trocam as medidas. Largura e altura são inteiras.
#[test]
fn as_medidas_de_bola_capsula_e_a_massa_manual_chegam() {
    let casos = [
        (
            InspectorPhysicsInfo {
                mass_manual: true,
                ..with_body()
            },
            INSP_PHYS_MASS,
            "7",
            E::Mass(7.0),
        ),
        (
            InspectorPhysicsInfo {
                shape_tag: 0,
                ..with_body()
            },
            INSP_PHYS_RADIUS,
            "2",
            E::Radius(2.0),
        ),
        (
            InspectorPhysicsInfo {
                shape_tag: 2,
                ..with_body()
            },
            INSP_PHYS_RADIUS,
            "2",
            E::Radius(2.0),
        ),
        // Height = 2 × (Half Height + Radius); o `with_body` tem Radius 0,4.
        (
            InspectorPhysicsInfo {
                shape_tag: 2,
                ..with_body()
            },
            INSP_PHYS_CAP_HALF_H,
            "3",
            E::CapHalfHeight(1.1),
        ),
        // Mais baixa que as duas tampas: é uma bola, nunca um segmento negativo.
        (
            InspectorPhysicsInfo {
                shape_tag: 2,
                ..with_body()
            },
            INSP_PHYS_CAP_HALF_H,
            "0.1",
            E::CapHalfHeight(0.0),
        ),
    ];
    for (info, id, texto, edit) in casos {
        let (mut host, mut st) = cena(info, [1.0, 1.0]);
        let r = pinta(&mut host, &mut st);
        assert!(achado(&r, id).is_some(), "{id:?} não pintou");
        let _ = host.drained_actions();
        for ev in host.type_into_number(id, texto) {
            let _ = host.apply_panel_event::<InspectorPanel>(&mut st, ev);
        }
        let a = host.drained_actions();
        let saiu = a.iter().find_map(|x| match x {
            EditorAction::InspectorPhysicsEdit { edit, .. } => Some(*edit),
            _ => None,
        });
        match (saiu, &edit) {
            (Some(E::CapHalfHeight(v)), E::CapHalfHeight(w)) => {
                assert!((v - w).abs() < 1e-5, "{v} ≠ {w}");
            }
            (saiu, _) => assert_eq!(saiu.as_ref(), Some(&edit), "{id:?}"),
        }
        sai();
    }
}

/// A conta do dono: quem quer 200 px digita 200, e o `Collider` guarda a metade em metros.
#[test]
fn digitar_200_na_largura_em_px_da_200_px() {
    set_cards_on(true);
    set_current_inspector_sprite(None);
    set_current_inspector_physics(Some(with_body()));
    set_current_inspector_transform(Some(pose([1.0, 1.0])));
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    host.project_mut().display_unit = ph2d_editor_core::project::DisplayUnit::Pixels;
    let ppm = host.project_mut().pixels_per_meter;
    host.store_mut().set_collapsed(CARD_PHYS_SHAPE, false);
    let mut st = InspectorState::default();
    assert!(achado(&pinta(&mut host, &mut st), INSP_PHYS_HALF_X).is_some());
    let _ = host.drained_actions();
    for ev in host.type_into_number(INSP_PHYS_HALF_X, "200") {
        let _ = host.apply_panel_event::<InspectorPanel>(&mut st, ev);
    }
    let a = host.drained_actions();
    let metade = a.iter().find_map(|x| match x {
        EditorAction::InspectorPhysicsEdit {
            edit: E::HalfX(v), ..
        } => Some(*v),
        _ => None,
    });
    let metade = metade.expect("o número não chegou");
    assert!((metade * ppm - 100.0).abs() < 1e-3, "{metade} m × {ppm}");
    sai();
}

/// Density some de quem a massa ninguém lê; o player cinemático que a transmite segue com ela.
#[test]
fn density_some_de_static_e_kinematic_e_fica_onde_e_lida() {
    for kind in [1, 2] {
        let (mut host, mut st) = cena(estatico(kind), [1.0, 1.0]);
        assert!(
            achado(&pinta(&mut host, &mut st), INSP_PHYS_DENSITY).is_none(),
            "kind {kind}: Density à vista num corpo sem efeito"
        );
        sai();
    }
    let lida = InspectorPhysicsInfo {
        kind_tag: 2,
        mass_is_read: true,
        ..with_body()
    };
    let (mut host, mut st) = cena(lida, [1.0, 1.0]);
    assert!(achado(&pinta(&mut host, &mut st), INSP_PHYS_DENSITY).is_some());
    sai();
}

/// As dobras nascem fechadas e abrem pelo clique no cabeçalho.
#[test]
fn as_dobras_nascem_fechadas_e_o_clique_abre() {
    set_cards_on(true);
    set_current_inspector_sprite(None);
    set_current_inspector_physics(Some(with_body()));
    set_current_inspector_transform(Some(pose([1.0, 1.0])));
    let mut host = crate::host_em_metros();
    for id in [core_ids::INSP_LIVE_PHYSICS_SECTION, CARD_PHYS_SHAPE] {
        host.store_mut().set_collapsed(id, false);
    }
    let mut st = InspectorState::default();
    let r = pinta(&mut host, &mut st);
    // À vista, sem abrir nada: Rotation e Width; dobrado: Init Velocity e Offset.
    assert!(achado(&r, INSP_PHYS_LOCKROT[1]).is_some());
    assert!(achado(&r, INSP_PHYS_HALF_X).is_some());
    assert!(achado(&r, INSP_PHYS_LINVEL_X).is_none());
    assert!(achado(&r, INSP_PHYS_OFFSET_X).is_none());
    let cab = achado(&r, INSP_PHYS_FOLD_OFFSET).expect("o cabeçalho de Offset");
    let _ = clica(&mut host, &mut st, cab);
    assert!(achado(&pinta(&mut host, &mut st), INSP_PHYS_OFFSET_X).is_some());
    sai();
}

#[test]
fn fit_to_picture_so_em_custom_size() {
    for (segue, tem) in [(None, false), (Some(true), false), (Some(false), true)] {
        let info = InspectorPhysicsInfo {
            shape_follows: segue,
            ..with_body()
        };
        let (mut host, mut st) = cena(info, [1.0, 1.0]);
        let r = pinta(&mut host, &mut st);
        assert_eq!(
            achado(&r, INSP_PHYS_FIT_PICTURE).is_some(),
            tem,
            "{segue:?}"
        );
        if tem {
            let a = clica(
                &mut host,
                &mut st,
                achado(&r, INSP_PHYS_FIT_PICTURE).unwrap(),
            );
            assert_eq!(
                a,
                [EditorAction::InspectorPhysicsEdit {
                    entity_bits: ENTIDADE,
                    edit: E::FitToPicture,
                }]
            );
        }
        sai();
    }
}

#[test]
fn bola_esticada_avisa_com_as_duas_saidas() {
    let bola = || InspectorPhysicsInfo {
        shape_tag: 0,
        ..with_body()
    };
    let (mut host, mut st) = cena(bola(), [1.0, 1.0]);
    assert!(achado(&pinta(&mut host, &mut st), INSP_PHYS_SCALE_ONE).is_none());
    sai();
    let (mut host, mut st) = cena(bola(), [-2.0, 1.0]);
    let r = pinta(&mut host, &mut st);
    let a = clica(&mut host, &mut st, achado(&r, INSP_PHYS_SCALE_ONE).unwrap());
    assert_eq!(
        a,
        [EditorAction::InspectorTransformEdit(pose([-1.0, 1.0]))],
        "o espelho fica"
    );
    let a = clica(&mut host, &mut st, achado(&r, INSP_PHYS_USE_BOX).unwrap());
    assert_eq!(
        a,
        [EditorAction::InspectorPhysicsEdit {
            entity_bits: ENTIDADE,
            edit: E::Shape(1),
        }]
    );
    sai();
    // A caixa esticada é só uma caixa maior: sem aviso.
    let (mut host, mut st) = cena(with_body(), [2.0, 1.0]);
    assert!(achado(&pinta(&mut host, &mut st), INSP_PHYS_SCALE_ONE).is_none());
    sai();
}

/// O objecto fica sem secção de física: o recibo segura o lugar, e o Undo é o Desfazer global.
#[test]
fn remove_deixa_o_recibo_e_o_undo_e_o_desfazer() {
    let (mut host, mut st) = cena(with_body(), [1.0, 1.0]);
    let r = pinta(&mut host, &mut st);
    let _ = clica(&mut host, &mut st, achado(&r, INSP_PHYS_REMOVE).unwrap());
    set_current_inspector_physics(None);
    let r = pinta(&mut host, &mut st);
    let undo = achado(&r, INSP_PHYS_REMOVED_UNDO).expect("o recibo sumiu com a secção");
    assert_eq!(
        clica(&mut host, &mut st, undo),
        [EditorAction::UndoStep { redo: false }]
    );
    assert!(achado(&pinta(&mut host, &mut st), INSP_PHYS_REMOVED_UNDO).is_none());
    sai();
}

/// Com o objecto movido, o Desfazer já não é o do Remove: o recibo não pode mentir.
#[test]
fn o_recibo_cai_se_o_objecto_mudou_ou_a_selecao_trocou() {
    let (mut host, mut st) = cena(with_body(), [1.0, 1.0]);
    let r = pinta(&mut host, &mut st);
    let _ = clica(&mut host, &mut st, achado(&r, INSP_PHYS_REMOVE).unwrap());
    set_current_inspector_physics(None);
    assert!(achado(&pinta(&mut host, &mut st), INSP_PHYS_REMOVED_UNDO).is_some());
    set_current_inspector_transform(Some(pose([3.0, 1.0])));
    assert!(achado(&pinta(&mut host, &mut st), INSP_PHYS_REMOVED_UNDO).is_none());
    set_current_inspector_transform(Some(pose([1.0, 1.0])));
    assert!(
        achado(&pinta(&mut host, &mut st), INSP_PHYS_REMOVED_UNDO).is_none(),
        "voltar à pose não ressuscita o recibo"
    );
    sai();
}

/// O recibo promete desfazer a remoção: outra edição por cima, ou outra fila a responder, tiram-lhe a validade.
#[test]
fn o_recibo_so_vale_enquanto_o_topo_da_fila_e_a_remocao() {
    use ph2d_panel_inspector::recibo_fisica::publica_o_topo;
    let (mut host, mut st) = cena(with_body(), [1.0, 1.0]);
    publica_o_topo(Some(10), true);
    let r = pinta(&mut host, &mut st);
    let _ = clica(&mut host, &mut st, achado(&r, INSP_PHYS_REMOVE).unwrap());
    set_current_inspector_physics(None);
    publica_o_topo(Some(10), true);
    assert!(
        achado(&pinta(&mut host, &mut st), INSP_PHYS_REMOVED_UNDO).is_some(),
        "o passo ainda não chegou à fila"
    );
    publica_o_topo(Some(11), true);
    let r = pinta(&mut host, &mut st);
    let undo = achado(&r, INSP_PHYS_REMOVED_UNDO).expect("o topo é a remoção");
    publica_o_topo(Some(12), true);
    assert!(
        achado(&pinta(&mut host, &mut st), INSP_PHYS_REMOVED_UNDO).is_none(),
        "um componente novo por cima: o Undo já não desfaz a remoção"
    );
    assert!(
        clica(&mut host, &mut st, undo).is_empty(),
        "o id que ficou no store não despacha"
    );
    publica_o_topo(Some(11), true);
    assert!(
        achado(&pinta(&mut host, &mut st), INSP_PHYS_REMOVED_UNDO).is_none(),
        "desfazer a edição de cima não ressuscita o recibo"
    );
    // Outra fila a responder (o Painter, um quadro, uma aba): o clique desfaria outra coisa.
    set_current_inspector_physics(Some(with_body()));
    publica_o_topo(Some(20), true);
    let r = pinta(&mut host, &mut st);
    let _ = clica(&mut host, &mut st, achado(&r, INSP_PHYS_REMOVE).unwrap());
    set_current_inspector_physics(None);
    publica_o_topo(Some(21), true);
    assert!(achado(&pinta(&mut host, &mut st), INSP_PHYS_REMOVED_UNDO).is_some());
    publica_o_topo(Some(21), false);
    assert!(achado(&pinta(&mut host, &mut st), INSP_PHYS_REMOVED_UNDO).is_none());
    sai();
}

/// Abre a lista da camada pelo clique real e devolve o que se pinta com ela aberta.
fn abre_a_lista_da_camada(
    host: &mut MockPanelHost,
    st: &mut InspectorState,
) -> Vec<(NodeId, Rect)> {
    let r = pinta(host, st);
    let chip = achado(&r, INSP_PHYS_LAYER_PICK).expect("a lista da camada");
    let _ = clica(host, st, chip);
    pinta(host, st)
}

/// A lista usa os mesmos ids e a mesma edição dos chips de antes: cada opção escolhe a sua camada.
#[test]
fn a_lista_da_camada_escolhe_cada_camada_pela_mesma_edicao() {
    // As oito que se veem sem nada publicado; a lista das visíveis tem a sua prova em `lista_tests`.
    for (i, &id) in INSP_PHYS_LAYER.iter().enumerate().take(8) {
        let (mut host, mut st) = cena(with_body(), [1.0, 1.0]);
        let abertas = abre_a_lista_da_camada(&mut host, &mut st);
        let op = achado(&abertas, id).unwrap_or_else(|| panic!("a opção {i} não apareceu"));
        assert_eq!(
            clica(&mut host, &mut st, op),
            [EditorAction::InspectorPhysicsEdit {
                entity_bits: ENTIDADE,
                edit: E::Layer(i as u8),
            }],
            "a opção {i}"
        );
        sai();
    }
}

/// Sem corpo acima nada bate na forma: os quadros contradiriam «Nothing bumps into it yet» e saem; a lista e o
/// «Rename…» ficam. Com dono ou com corpo, a fila continua entre os dois.
#[test]
fn a_forma_solta_nao_mostra_os_quadros_mas_mantem_a_camada_e_o_rename() {
    let vao = |info: InspectorPhysicsInfo| {
        let (mut host, mut st) = cena(info, [1.0, 1.0]);
        let r = pinta(&mut host, &mut st);
        let lista = achado(&r, INSP_PHYS_LAYER_PICK).expect("a lista da camada");
        let rename = achado(&r, INSP_PHYS_LAYER_RENAME).expect("o Rename…");
        sai();
        rename.y - lista.y
    };
    let solta = InspectorPhysicsInfo {
        part_owner: String::new(),
        ..as_part()
    };
    let (sem, peca, corpo) = (vao(solta), vao(as_part()), vao(with_body()));
    assert_eq!(peca, corpo, "peça e corpo mostram a mesma linha");
    assert!(
        sem < peca,
        "a solta ainda reserva a fila: {sem} × {peca}"
    );
}

/// «Rename…» abre o campo ali mesmo; escrever pede o novo nome da camada do objecto.
#[test]
fn rename_abre_o_campo_e_escrever_renomeia_a_camada_do_objecto() {
    let mut info = with_body();
    info.layer = 3;
    let (mut host, mut st) = cena(info, [1.0, 1.0]);
    let r = pinta(&mut host, &mut st);
    assert!(
        achado(&r, INSP_PHYS_LAYER_NAME).is_none(),
        "fechado por omissão"
    );
    let _ = clica(
        &mut host,
        &mut st,
        achado(&r, INSP_PHYS_LAYER_RENAME).unwrap(),
    );
    let r = pinta(&mut host, &mut st);
    assert!(achado(&r, INSP_PHYS_LAYER_NAME).is_some());
    let _ = host.drained_actions();
    host.set_text(INSP_PHYS_LAYER_NAME, "Enemy");
    let _ = host.apply_panel_event::<InspectorPanel>(
        &mut st,
        ph2d_editor_core::interaction::WidgetEvent::TextChanged(INSP_PHYS_LAYER_NAME),
    );
    assert_eq!(
        host.drained_actions().last(),
        Some(&EditorAction::LayerRename {
            layer: 3,
            name: "Enemy".into(),
        })
    );
    sai();
}

/// «Edit shape on canvas», pelo clique real: Shape e Collision Shape têm o botão, e ele pede a mesma edição.
#[test]
fn o_botao_das_alcas_liga_pelo_clique_em_shape_e_em_collision_shape() {
    for info in [
        with_body(),
        InspectorPhysicsInfo {
            has_body: false,
            ..with_body()
        },
    ] {
        let (mut host, mut st) = cena(info, [1.0, 1.0]);
        let r = pinta(&mut host, &mut st);
        let b = achado(&r, INSP_PHYS_EDIT_SHAPE).expect("o botão no cartão");
        assert_eq!(
            clica(&mut host, &mut st, b),
            [EditorAction::InspectorPhysicsEdit {
                entity_bits: ENTIDADE,
                edit: E::EditShapeOnCanvas,
            }]
        );
        sai();
    }
}

/// Aceso, diz como se sai, e o clique é o mesmo: o segundo clique encerra.
#[test]
fn o_botao_aceso_diz_como_sair_e_o_segundo_clique_pede_o_mesmo() {
    let info = InspectorPhysicsInfo {
        shape_edit_on: true,
        ..with_body()
    };
    let (mut host, mut st) = cena(info, [1.0, 1.0]);
    let r = pinta(&mut host, &mut st);
    let b = achado(&r, INSP_PHYS_EDIT_SHAPE).expect("o botão aceso");
    assert_eq!(
        clica(&mut host, &mut st, b),
        [EditorAction::InspectorPhysicsEdit {
            entity_bits: ENTIDADE,
            edit: E::EditShapeOnCanvas,
        }]
    );
    assert_eq!(tr("card.insp.phys.edit_shape"), "Edit shape on canvas");
    assert_eq!(
        tr("card.insp.phys.editing_shape"),
        "Editing shape · Esc to finish"
    );
    sai();
}

/// O nome diz o que a coisa É; o aviso cabe inteiro.
#[test]
fn os_textos_do_cartao_estao_na_tabela() {
    for k in [
        "card.insp.phys.body",
        "card.insp.phys.shape",
        "card.insp.phys.collision_shape",
        "card.insp.phys.touch",
        "card.insp.phys.loose",
    ] {
        assert_ne!(tr(k), k, "{k} sem texto");
    }
}

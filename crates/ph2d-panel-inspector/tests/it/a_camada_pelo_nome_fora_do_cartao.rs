//! ⭐ **A camada pelo NOME fora do cartão** (dono 07/11, fecho da GEF02). A face clássica (`PH2D_CARDS=0`) e o Ray
//! Sensor diziam só o número da camada; o Ray Sensor tem os mesmos campos do collider (dono 08/11).
//!
//! Mutações: o chip da face clássica sem o contorno, ou com a cor de outra camada; a lista sem as camadas além da 7
//! (os chips «0»–«7» de antes); a lista do Ray a mandar outra camada, os quadros «Sees» noutra linha, sem losango.

use crate::seam_physics::{click_real, expect, with_body};
use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
use ph2d_editor_core::ray_edits::{InspectorRayInfo, RayFieldEdit};
use ph2d_editor_core::screens::hero::PhysicsFieldEdit;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::camadas_fisica::set_current;
use ph2d_panel_inspector::ids::*;
use ph2d_panel_inspector::{
    InspectorPanel, InspectorState, set_cards_on, set_current_inspector_physics,
    set_current_inspector_ray,
};

fn mundo() {
    let mut nomes: [String; 32] = Default::default();
    for (i, n) in [(0, "Ground"), (1, "Player"), (2, "Props"), (20, "Lava")] {
        nomes[i] = n.into();
    }
    set_current(nomes, [u32::MAX; 32], 0b111 | 1 << 20);
}

fn fim() {
    set_current(Default::default(), [u32::MAX; 32], 0xFF);
    set_cards_on(false);
}

#[test]
fn a_face_classica_escolhe_a_camada_pelo_nome_em_todas_as_visiveis() {
    set_cards_on(false);
    mundo();
    let mut info = with_body();
    info.layer = 2;
    let mut host = crate::host_em_metros();
    let mut state = InspectorState::default();
    set_current_inspector_physics(Some(info.clone()));
    let rects = host.paint::<InspectorPanel>(&mut state, Rect::new(0.0, 0.0, 320.0, 2400.0));
    set_current_inspector_physics(None);
    let chip = rects
        .iter()
        .find(|(n, _)| *n == INSP_PHYS_LAYER_PICK)
        .map(|(_, r)| *r)
        .expect("a face clássica pinta o seletor Layer");
    assert!(
        !rects.iter().any(|(n, _)| INSP_PHYS_LAYER.contains(n)),
        "fechada, nenhum chip de número"
    );
    let c = ph2d_editor_core::paint::layer_outline_of(INSP_PHYS_LAYER_PICK).expect("contorno");
    assert_eq!(
        (c.camada, c.rect),
        (2, chip),
        "o chip veste a cor da camada do objecto"
    );
    // A camada 20 (além dos oito chips de antes) escolhe-se pelo clique real, um passo.
    expect(
        &click_real(info, INSP_PHYS_LAYER[20]),
        PhysicsFieldEdit::Layer(20),
        "a camada 20 pela lista da face clássica",
    );
    // E as 0–7 de antes, mesmo sem uso (a 5 não é visível): o alcance dos 8 chips não se perde.
    let mut info5 = with_body();
    info5.layer = 2;
    expect(
        &click_real(info5, INSP_PHYS_LAYER[5]),
        PhysicsFieldEdit::Layer(5),
        "a camada 5, sem uso, pela lista da face clássica",
    );
    fim();
}

/// Pinta o Inspector com a secção do raio aberta (as secções de componente nascem dobradas), como depois do clique
/// no título. No cartão, pela moldura do cartão da física (as dobras assentam ao longo dos quadros).
fn pinta_aberto(
    cartoes: bool,
    host: &mut ph2d_ui_testkit::MockPanelHost,
    state: &mut InspectorState,
) -> Vec<(ph2d_a11y::NodeId, Rect)> {
    use ph2d_editor_core::panel::PanelHostInternal as _;
    set_cards_on(cartoes);
    host.store_mut()
        .set_collapsed(ph2d_editor_core::ids::INSP_LIVE_RAY_SECTION, false);
    if cartoes {
        return crate::o_cartao_da_fisica::pinta(host, state);
    }
    let tela = Rect::new(0.0, 0.0, 1600.0, 900.0);
    let _ = host.paint::<InspectorPanel>(state, tela);
    host.store_mut()
        .set_collapsed(ph2d_editor_core::ids::INSP_LIVE_RAY_SECTION, false);
    host.paint::<InspectorPanel>(state, tela)
}

fn achar(rects: &[(ph2d_a11y::NodeId, Rect)], id: ph2d_a11y::NodeId) -> Option<Rect> {
    crate::o_cartao_da_fisica::achado(rects, id)
}

/// O clique real (mover, premir, soltar) no meio de `r`; devolve as acções.
fn clica(
    host: &mut ph2d_ui_testkit::MockPanelHost,
    state: &mut InspectorState,
    r: Rect,
) -> Vec<EditorAction> {
    crate::o_cartao_da_fisica::clica(host, state, r)
}

/// Pinta o Ray Sensor `olho`, abre a lista se `id` for uma opção dela, e clica no meio de `id`.
fn clique_no_raio(
    cartoes: bool,
    olho: InspectorRayInfo,
    id: ph2d_a11y::NodeId,
) -> Vec<EditorAction> {
    let mut host = crate::host_em_metros();
    let mut state = InspectorState::default();
    set_current_inspector_ray(Some(olho));
    let mut rects = pinta_aberto(cartoes, &mut host, &mut state);
    if INSP_RAY_LAYER_OPTS.contains(&id) {
        assert!(achar(&rects, id).is_none(), "fechada, nenhuma opção");
        let r = achar(&rects, INSP_RAY_LAYER).expect("o chip Layer do raio");
        let _ = clica(&mut host, &mut state, r);
        rects = pinta_aberto(cartoes, &mut host, &mut state);
    }
    let r = achar(&rects, id).unwrap_or_else(|| panic!("{id:?} não foi pintado ({cartoes})"));
    let out = clica(&mut host, &mut state, r);
    set_current_inspector_ray(None);
    out
}

fn olho_na(camada: u32) -> InspectorRayInfo {
    let mut olho = crate::a_seccao_ray_sensor_esta_viva::olho();
    olho.layer = camada;
    olho
}

/// Dono 08/11: o Ray Sensor com os MESMOS campos de camada do collider — a lista com o nome e o contorno da cor,
/// o losango do campo (o mesmo canal `component.field.ray_sensor.3`) e «Sees», a linha da matriz do mundo.
///
/// Mutações: uma opção que manda outra camada; os quadros a ler outra linha da matriz; o chip sem losango.
#[test]
fn o_ray_sensor_escolhe_a_camada_pela_lista_e_ve_pela_matriz() {
    use ph2d_editor_core::widget::key_mark;
    for cartoes in [true, false] {
        mundo();
        let mut host = crate::host_em_metros();
        let mut state = InspectorState::default();
        set_current_inspector_ray(Some(olho_na(20)));
        key_mark::set_key_marks(std::collections::BTreeMap::new());
        let rects = pinta_aberto(cartoes, &mut host, &mut state);
        set_current_inspector_ray(None);
        let chip = achar(&rects, INSP_RAY_LAYER).expect("o chip Layer do raio");
        let c = ph2d_editor_core::paint::layer_outline_of(INSP_RAY_LAYER).expect("contorno");
        assert_eq!(
            (c.camada, c.rect),
            (20, chip),
            "o chip veste a cor da camada do raio ({cartoes})"
        );
        let canal: Vec<_> = key_mark::rotulos_pintados()
            .into_iter()
            .filter(|(_, k)| *k == "component.field.ray_sensor.3")
            .map(|(id, _)| id)
            .collect();
        assert_eq!(
            canal,
            vec![INSP_RAY_LAYER],
            "o losango do Layer é o canal de sempre ({cartoes})"
        );
        // Os quadros «Sees»: as visíveis e a do raio (a 0, 1, 2 e 20), mais o «+».
        for j in [0, 1, 2, 20] {
            assert!(
                rects.iter().any(|(n, _)| *n == INSP_RAY_SEES[j]),
                "o quadro {j} ({cartoes})"
            );
        }
        assert!(
            rects.iter().any(|(n, _)| *n == INSP_RAY_SEES_ADD),
            "o «+» ({cartoes})"
        );
        // A camada 20 pela lista, o clique real.
        let fora = clique_no_raio(cartoes, olho_na(2), INSP_RAY_LAYER_OPTS[20]);
        assert_eq!(
            fora,
            [EditorAction::InspectorComponentEdit {
                entity_bits: 0x00AB_CDEF,
                edit: ComponentEdit::Ray(RayFieldEdit::Layer(20)),
            }],
            "a camada 20 pela lista do raio ({cartoes})"
        );
        // «Sees» edita a MESMA célula da matriz que o «Collides with» do collider na mesma camada.
        let raio = clique_no_raio(cartoes, olho_na(2), INSP_RAY_SEES[1]);
        assert_eq!(
            raio,
            [EditorAction::LayerPairToggle { a: 2, b: 1 }],
            "o quadro do raio ({cartoes})"
        );
        {
            let mut corpo = with_body();
            corpo.layer = 2;
            // O «Collides with» da FORMA edita a exceção dela (GEF02 B9), nunca a célula da matriz.
            use crate::o_cartao_da_fisica::{achado, cena, clica, pinta, sai};
            let (mut host, mut st) = cena(corpo, [1.0, 1.0]);
            let r = pinta(&mut host, &mut st);
            let caixa = achado(&r, INSP_PHYS_COLLIDES[1]).expect("o quadro do collider");
            let collider = clica(&mut host, &mut st, caixa);
            sai();
            let edit = ph2d_editor_core::PhysicsFieldEdit::CollidesWith {
                layer: 1,
                on: false,
            };
            assert_eq!(
                collider,
                [EditorAction::InspectorPhysicsEdit {
                    entity_bits: crate::seam_physics::ENTITY,
                    edit
                }]
            );
        }
        fim();
    }
}

/// Emenda 4 (dono 08/11): o Ray Sensor tem «Rename layers…» e «Open in tab…» no lugar dos do collider (abaixo de
/// Layer, acima de «Sees»), com ids próprios: o objecto pode ter collider E raio, e os dois cartões à vista. Rename
/// abre os campos com o nome todo selecionado (só num cartão de cada vez) e Enter grava UM `LayerRename`; Open in
/// tab abre a aba na camada do RAIO.
///
/// Mutações: Open in tab com a camada do collider; o Rename do raio sem fechar o do collider.
#[test]
fn o_ray_sensor_renomeia_e_abre_a_aba_pelos_botoes_do_collider() {
    use crate::o_cartao_da_fisica::{achado, cena, clica, pinta, sai};
    use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
    use ph2d_editor_core::panel::PanelHostInternal as _;
    mundo();
    let mut corpo = with_body();
    corpo.layer = 2;
    let (mut host, mut st) = cena(corpo, [1.0, 1.0]);
    set_current_inspector_ray(Some(olho_na(20)));
    host.store_mut()
        .set_collapsed(ph2d_editor_core::ids::INSP_LIVE_RAY_SECTION, false);
    let r = pinta(&mut host, &mut st);
    let y = |id| achado(&r, id).unwrap_or_else(|| panic!("{id:?} pintado")).y;
    assert!(
        achado(&r, INSP_PHYS_LAYER_RENAME).is_some(),
        "o do collider"
    );
    let rename = y(INSP_RAY_LAYER_RENAME);
    assert_eq!(rename, y(INSP_RAY_EDIT_MATRIX), "os dois na mesma linha");
    assert!(
        y(INSP_RAY_LAYER) < rename && rename < y(INSP_RAY_SEES[0]),
        "abaixo de Layer, acima de «Sees»"
    );
    let aba = clica(
        &mut host,
        &mut st,
        achado(&r, INSP_RAY_EDIT_MATRIX).unwrap(),
    );
    assert_eq!(
        aba,
        [EditorAction::LayerMatrixOpen { layer: 20 }],
        "a camada do raio"
    );
    // O collider em Rename; o do raio fecha-o e abre o seu, com o nome da 20 todo selecionado.
    let _ = clica(
        &mut host,
        &mut st,
        achado(&r, INSP_PHYS_LAYER_RENAME).unwrap(),
    );
    let r = pinta(&mut host, &mut st);
    let _ = clica(
        &mut host,
        &mut st,
        achado(&r, INSP_RAY_LAYER_RENAME).unwrap(),
    );
    let r = pinta(&mut host, &mut st);
    assert!(
        host.store().is_collapsed(INSP_PHYS_LAYER_RENAMING),
        "um cartão de cada vez"
    );
    assert!(
        achado(&r, INSP_RAY_LAYER_RENAME_DONE).is_some(),
        "o Done do raio"
    );
    assert!(achado(&r, INSP_PHYS_LAYER_RENAME_DONE).is_none());
    let campo = achado(&r, INSP_PHYS_LAYER_NAMES[20]).expect("o campo da 20");
    assert!(campo.y > y(INSP_RAY_LAYER), "no cartão do raio");
    assert_eq!(
        r.iter()
            .filter(|(n, q)| *n == INSP_PHYS_LAYER_NAMES[20] && q.w > 0.0)
            .count(),
        1,
        "o campo uma vez só"
    );
    assert_eq!(host.store().focus_id(), Some(INSP_PHYS_LAYER_NAMES[20]));
    match host.store().get(INSP_PHYS_LAYER_NAMES[20]) {
        Some(InteractiveState::TextInput {
            text,
            caret,
            selection_anchor,
            ..
        }) => assert_eq!(
            (*selection_anchor, *caret),
            (Some(0), text.len()),
            "{text:?}"
        ),
        outro => panic!("o campo: {outro:?}"),
    }
    host.set_text(INSP_PHYS_LAYER_NAMES[20], "Magma");
    let _ = host.drained_actions();
    let _ = host.apply_panel_event::<InspectorPanel>(
        &mut st,
        WidgetEvent::Submit(INSP_PHYS_LAYER_NAMES[20]),
    );
    assert_eq!(
        host.drained_actions(),
        [EditorAction::LayerRename {
            layer: 20,
            name: "Magma".into()
        }],
        "Enter grava UM passo"
    );
    set_current_inspector_ray(None);
    sai();
    fim();
}

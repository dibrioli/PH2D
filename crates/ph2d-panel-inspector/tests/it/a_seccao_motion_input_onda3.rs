//! **O cartão do MOTION INPUT na onda 3** (GEF 08): a entrada é um objeto FILHO (as saídas anexam no pai), dez
//! fontes (Velocity e Move) e a leitura do par. Clique REAL (`Down` + `Up` no rect pintado).

use super::a_seccao_motion_input_esta_viva::{
    SEC, VIEWPORT, clica, frases, info, pinta, pointer, rect_de,
};
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::motion_input_edits::{
    InspectorMotionInputInfo, MOTION_INPUT_SOURCES, MotionInputAoVivo as V,
    MotionInputFieldEdit as E, SOURCE_HEALTH, SOURCE_MOVE, SOURCE_SPEED, SOURCE_VALUE,
    SOURCE_VELOCITY,
};
use ph2d_editor_core::panel::PanelHostInternal as _;
use ph2d_host::PointerKind;
use ph2d_panel_inspector::{InspectorPanel, ids};

const PAI: u64 = 0x0000_0B0B;

/// Clique real; devolve tudo o que chegou ao barramento, e o host (para ler o store).
fn clica_tudo(
    i: InspectorMotionInputInfo,
    id: ph2d_a11y::NodeId,
) -> (Vec<EditorAction>, ph2d_ui_testkit::MockPanelHost) {
    let (mut h, mut st, rects) = pinta(i);
    let r = rect_de(&rects, id).unwrap_or_else(|| panic!("{id:?} não foi pintado"));
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let _ = h.drained_actions();
    let mut evs = h.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
    evs.extend(h.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100)));
    for ev in evs {
        h.apply_panel_event::<InspectorPanel>(&mut st, ev);
    }
    (h.drained_actions(), h)
}

fn pede(acoes: &[EditorAction], bits: u64, comp: &str) -> bool {
    acoes.iter().any(|a| {
        matches!(a, EditorAction::InspectorAddComponentRequested { entity_bits, component: Some(c) }
            if *entity_bits == bits && *c == comp)
    })
}

/// ⭐ «Add Health» e «Add Physics Body» (Speed e Velocity) anexam no objecto que a fonte LÊ (o pai).
///
/// **Mutação que deve sangrar:** voltar a `entity_bits: bits` no despacho, ou tirar o Velocity da saída.
#[test]
fn as_saidas_anexam_no_pai_e_nao_no_filho() {
    let mut h = info(SOURCE_HEALTH);
    h.tem_vida = false;
    for (fonte, id, comp) in [
        (
            SOURCE_HEALTH,
            ids::INSP_MI_ADD_HEALTH,
            "ph2d::physics::Health",
        ),
        (
            SOURCE_SPEED,
            ids::INSP_MI_ADD_BODY,
            "ph2d::physics::RigidBody",
        ),
        (
            SOURCE_VELOCITY,
            ids::INSP_MI_ADD_BODY,
            "ph2d::physics::RigidBody",
        ),
    ] {
        let mut i = info(fonte);
        i.tem_vida = false;
        i.alvo_bits = Some(PAI);
        let (a, _) = clica_tudo(i, id);
        assert!(pede(&a, PAI, comp), "fonte {fonte}: {a:?}");
    }
}

/// ⭐ As queixas dizem o NOME do pai («Hero has no Health»), e quem tapa o nome é «o Motion Input do Hero».
#[test]
fn as_frases_dizem_o_nome_do_pai() {
    let mut i = info(SOURCE_HEALTH);
    i.tem_vida = false;
    i.pai = "Hero".into();
    let f = frases(i);
    assert!(
        f.iter().any(|f| f.starts_with("Hero has no Health")),
        "{f:?}"
    );
    let mut c = info(SOURCE_VALUE);
    c.a_correr = true;
    c.tapado_por = Some((
        ph2d_editor_core::motion_input_edits::MotionInputOutro::Objeto("Boss".into()),
        V::Numero(0.2),
    ));
    let f = frases(c);
    assert!(
        f.iter()
            .any(|f| f.starts_with("The Motion Input on Boss sets this name first (0.20)")),
        "{f:?}"
    );
}

/// ⭐ As DEZ fontes: cada chip pintado manda o índice dele; nenhum chip cobre outro (hit areas certos).
///
/// **Mutação que deve sangrar:** tirar um id de `INSP_MI_SOURCE`, ou registar dois chips no mesmo rect.
#[test]
fn as_dez_fontes_tem_cada_uma_o_seu_chip() {
    let (mut h, mut st, rects) = pinta(info(SOURCE_VALUE));
    let chips: Vec<_> = ids::INSP_MI_SOURCE
        .iter()
        .map(|&id| rect_de(&rects, id).expect("chip pintado"))
        .collect();
    assert_eq!(chips.len(), MOTION_INPUT_SOURCES);
    for (a, ra) in chips.iter().enumerate() {
        for rb in &chips[a + 1..] {
            let cruza = ra.x < rb.x + rb.w
                && rb.x < ra.x + ra.w
                && ra.y < rb.y + rb.h
                && rb.y < ra.y + ra.h;
            assert!(!cruza, "chips {a} cruzam-se: {ra:?} {rb:?}");
        }
    }
    for n in 0..MOTION_INPUT_SOURCES {
        let n8 = u8::try_from(n).unwrap_or(0);
        let esperado = vec![E::Source(n8)];
        assert_eq!(
            clica(&mut h, &mut st, &rects, ids::INSP_MI_SOURCE[n]),
            esperado,
            "chip {n}"
        );
    }
}

/// ⭐ Velocity: From sim, Value/Color/Action/Property não. Move: nenhum deles, a linha do que lê, e sem
/// acções `move_*` a queixa com a saída «Open Input Map» (pede a aba).
#[test]
fn velocity_e_move_mostram_so_o_que_leem() {
    let mut v = info(SOURCE_VELOCITY);
    v.tem_corpo = true;
    let (_h, _st, rects) = pinta(v);
    let pintado = |id| rect_de(&rects, id).is_some();
    assert!(pintado(ids::INSP_MI_FROM));
    for id in [
        ids::INSP_MI_VALUE,
        ids::INSP_MI_ACTION,
        ids::INSP_MI_PROPERTY,
        ids::INSP_MI_COLOR,
    ] {
        assert!(!pintado(id), "Velocity esconde {id:?}");
    }
    let m = info(SOURCE_MOVE);
    let (_h, _st, rects) = pinta(m.clone());
    for id in [
        ids::INSP_MI_FROM,
        ids::INSP_MI_VALUE,
        ids::INSP_MI_ACTION,
        ids::INSP_MI_PROPERTY,
    ] {
        assert!(rect_de(&rects, id).is_none(), "Move esconde {id:?}");
    }
    let (_, mut h) = clica_tudo(m, ids::INSP_MI_OPEN_INPUT_MAP);
    assert!(
        h.store_mut().take_input_map_tab_request(),
        "o clique pede a aba do Input Map"
    );
    let mut com = info(SOURCE_MOVE);
    com.acoes_do_mapa = vec!["jump".into(), "move_left".into(), "move_right".into()];
    let f = frases(com.clone());
    assert!(
        f.contains(&"Reads move_right \u{b7} move_left from the Input Map".to_string()),
        "{f:?}"
    );
    let (_h, _st, rects) = pinta(com);
    assert!(rect_de(&rects, ids::INSP_MI_OPEN_INPUT_MAP).is_none());
}

/// ⭐ O par: parado «Reads Velocity here: (3.20, −1.00)», em Play «Sends (3.20, −1.00)» (menos tipográfico).
#[test]
fn o_par_le_e_manda_com_duas_casas() {
    let mut v = info(SOURCE_VELOCITY);
    v.tem_corpo = true;
    v.ao_vivo = Some(V::Par([3.2, -1.0]));
    let f = frases(v.clone());
    assert!(
        f.contains(&"Reads Velocity here: (3.20, \u{2212}1.00)".to_string()),
        "{f:?}"
    );
    v.a_correr = true;
    let f = frases(v);
    assert!(
        f.contains(&"Sends (3.20, \u{2212}1.00)".to_string()),
        "{f:?}"
    );
    assert!(!f.iter().any(|f| f.starts_with("Reads")), "{f:?}");
}

/// ⭐ A lista Property longa abre ABAIXO do chip (nunca cobre o topo do Inspector) e a opção continua a ser
/// clicável; o campo de par diz «(x, y)».
#[test]
fn a_lista_longa_abre_abaixo_do_chip_e_rola() {
    let mut p = info(ph2d_editor_core::motion_input_edits::SOURCE_PROPERTY);
    p.campos_do_alvo = (0..60)
        .map(|n| (format!("Field {n}"), format!("ph2d::x::X/f{n}")))
        .collect();
    p.campos_par = vec!["ph2d::x::X/f1".into()];
    let (mut h, mut st, rects) = pinta(p);
    let chip = rect_de(&rects, ids::INSP_MI_PROPERTY).expect("chip");
    let _ = clica(&mut h, &mut st, &rects, ids::INSP_MI_PROPERTY);
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    let opcoes: Vec<_> = (0..60)
        .filter_map(|n| rect_de(&rects, ids::INSP_MI_PROPERTY_OPT.id(n)))
        .collect();
    assert!(!opcoes.is_empty(), "a lista abriu");
    assert!(
        opcoes.iter().all(|r| r.y >= chip.y + chip.h),
        "a lista cobre o que está acima do chip"
    );
    assert!(opcoes.len() < 60 || opcoes.iter().all(|r| r.y + r.h <= VIEWPORT.y + VIEWPORT.h));
    assert_eq!(
        clica(&mut h, &mut st, &rects, ids::INSP_MI_PROPERTY_OPT.id(1)),
        vec![E::Property("ph2d::x::X/f1".into())]
    );
}

/// ⭐ Com o chip no FUNDO do Inspector (cartão rolado ao fim), a lista abre PARA CIMA, não sobe até o topo do
/// Inspector, e a roda do rato rola até a última opção, que se escolhe por clique real.
///
/// **Mutação que deve sangrar:** o ramo de cima de `abaixo_do_chip` devolver a `region` inteira.
#[test]
fn sem_espaco_abaixo_a_lista_abre_para_cima_e_rola_ate_a_ultima() {
    const N: usize = 60;
    let mut p = info(ph2d_editor_core::motion_input_edits::SOURCE_PROPERTY);
    p.campos_do_alvo = (0..N)
        .map(|n| (format!("Field {n}"), format!("ph2d::x::X/f{n}")))
        .collect();
    let (_, _, alto) = pinta(p.clone());
    let chip = rect_de(&alto, ids::INSP_MI_PROPERTY).expect("chip");
    // O Inspector acaba logo abaixo do chip: o que o cartão rolado ao fundo dá.
    let vp = ph2d_editor_core::zones::Rect::new(
        VIEWPORT.x,
        VIEWPORT.y,
        VIEWPORT.w,
        chip.y + chip.h * 2.0,
    );
    let mut h = ph2d_ui_testkit::MockPanelHost::with_panel::<InspectorPanel>();
    ph2d_panel_inspector::set_current_inspector_motion_input(Some(p));
    let mut st = ph2d_panel_inspector::InspectorState::default();
    let rects = h.paint::<InspectorPanel>(&mut st, vp);
    assert_eq!(
        rect_de(&rects, ids::INSP_MI_PROPERTY),
        Some(chip),
        "o chip mexeu"
    );
    let _ = clica(&mut h, &mut st, &rects, ids::INSP_MI_PROPERTY);
    let mut rects = h.paint::<InspectorPanel>(&mut st, vp);
    let (_, lista) = h.store().dropdown_popover().expect("a lista abriu");
    assert!(
        lista.y + lista.h <= chip.y,
        "abre para cima: {lista:?} × chip {chip:?}"
    );
    // O topo do Inspector = a primeira linha que ele pinta.
    let topo = alto.iter().map(|(_, r)| r.y).fold(f32::MAX, f32::min);
    assert!(
        lista.y >= topo,
        "a lista cobre o topo do Inspector ({topo}): {lista:?}"
    );
    let ultima = ids::INSP_MI_PROPERTY_OPT.id(N - 1);
    for _ in 0..12 {
        if rect_de(&rects, ultima).is_some() {
            break;
        }
        let _ = h.wheel_at(lista.x + lista.w * 0.5, lista.y + lista.h * 0.5, -20000.0);
        rects = h.paint::<InspectorPanel>(&mut st, vp);
    }
    let r = rect_de(&rects, ultima).expect("a roda do rato chega à última opção");
    assert!(
        r.y >= lista.y && r.y + r.h <= lista.y + lista.h + 1.0,
        "{r:?} fora de {lista:?}"
    );
    assert_eq!(
        clica(&mut h, &mut st, &rects, ultima),
        vec![E::Property(format!("ph2d::x::X/f{}", N - 1))]
    );
}

/// ⭐ MI-10: o filho «Motion Input» não tem lugar no espaço — o Inspector dele não pinta Transform nem Ordering
/// (nem corpo), só o cartão Motion Input; sem ele, as mesmas fotos pintam a Transform (controlo).
///
/// **Mutação que deve sangrar:** `com_lugar = true` em `LiveSnapshots::fetch`.
#[test]
fn a_seccao_motion_input_do_filho_nao_tem_transform() {
    use ph2d_editor_core::ids as core;
    use ph2d_editor_core::screens::hero::InspectorTransformInfo;
    let poe = || {
        ph2d_panel_inspector::set_current_inspector_transform(Some(InspectorTransformInfo {
            entity_bits: super::a_seccao_motion_input_esta_viva::BITS,
            translation: [0.0, 0.0],
            rotation_rad: 0.0,
            scale: [1.0, 1.0],
            skew_rad: [0.0, 0.0],
        }));
    };
    poe();
    let (_, _, rects) = pinta(info(SOURCE_VELOCITY));
    let tem = |id| rect_de(&rects, id).is_some();
    assert!(
        tem(core::INSP_LIVE_MOTION_INPUT_SECTION),
        "o cartão Motion Input"
    );
    assert!(
        !tem(core::INSP_LIVE_TRANSFORM_SECTION),
        "Transform no filho"
    );
    ph2d_panel_inspector::set_current_inspector_motion_input(None);
    poe();
    let mut h = ph2d_ui_testkit::MockPanelHost::with_panel::<InspectorPanel>();
    let mut st = ph2d_panel_inspector::InspectorState::default();
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    let controlo = rect_de(&rects, core::INSP_LIVE_TRANSFORM_SECTION).is_some();
    ph2d_panel_inspector::set_current_inspector_transform(None);
    assert!(controlo, "controlo: sem Motion Input a Transform pinta");
}

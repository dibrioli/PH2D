//! **As linhas da onda 2 do MOTION INPUT estão VIVAS sob o dedo** (GEF 08): From, Action, Property, o grupo
//! «Shape» e as saídas das queixas. Clique REAL (`Down` + `Up` no rect pintado), pelas ajudas da irmã.

use super::a_seccao_motion_input_esta_viva::{
    BITS, SEC, VIEWPORT, clica, info, pinta, pointer, rect_de,
};
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::motion_input_edits::{
    CURVE_PRESETS, InspectorMotionInputInfo, MotionInputFieldEdit as E, SOURCE_ACTION,
    SOURCE_GROUNDED, SOURCE_HEALTH, SOURCE_PROPERTY, SOURCE_SPEED, SOURCE_VALUE,
};
use ph2d_host::PointerKind;
use ph2d_panel_inspector::{InspectorPanel, ids};

/// Cada linha nova só existe com a fonte que a lê: From nas cinco que leem um objecto, Action e Property
/// cada uma na sua, o «Shape» só com uma fonte que publica número.
///
/// **Mutação que deve sangrar:** pintar o From sem o `if i.le_from()`, ou o Shape sem o `le_numero()`.
#[test]
fn as_linhas_da_onda_2_so_existem_com_a_fonte_que_as_le() {
    for fonte in 0..8_u8 {
        let mut i = info(fonte);
        i.acoes_do_mapa = vec!["jump".into()];
        let (_h, _st, rects) = pinta(i);
        let pintado = |id| rect_de(&rects, id).is_some();
        let le_from = matches!(fonte, 2 | 3 | 4 | 5 | 7);
        assert_eq!(pintado(ids::INSP_MI_FROM), le_from, "From (fonte {fonte})");
        assert_eq!(
            pintado(ids::INSP_MI_FROM_PICK),
            le_from,
            "conta-gotas (fonte {fonte})"
        );
        assert_eq!(
            pintado(ids::INSP_MI_ACTION),
            fonte == SOURCE_ACTION,
            "Action ({fonte})"
        );
        assert_eq!(
            pintado(ids::INSP_MI_PROPERTY),
            fonte == SOURCE_PROPERTY,
            "Property ({fonte})"
        );
        assert_eq!(
            pintado(ids::INSP_MI_FOLD_SHAPE),
            !matches!(fonte, 1 | 3),
            "Shape só com número (fonte {fonte})"
        );
        assert!(
            !pintado(ids::INSP_MI_REMAP),
            "o Shape nasce FECHADO (fonte {fonte})"
        );
    }
}

/// Um chip de cada fonte nova envia o índice dela.
#[test]
fn as_fontes_novas_enviam_o_indice() {
    let (mut h, mut st, rects) = pinta(info(SOURCE_VALUE));
    for n in 4..8_u8 {
        assert_eq!(
            clica(&mut h, &mut st, &rects, ids::INSP_MI_SOURCE[usize::from(n)]),
            vec![E::Source(n)]
        );
    }
}

/// Abre o Shape com um clique real no cabeçalho e devolve o que se pinta depois.
fn com_o_shape_aberto(
    i: InspectorMotionInputInfo,
) -> (
    ph2d_ui_testkit::MockPanelHost,
    ph2d_panel_inspector::InspectorState,
    Vec<(ph2d_a11y::NodeId, ph2d_editor_core::zones::Rect)>,
) {
    let (mut h, mut st, rects) = pinta(i);
    let _ = clica_sem_edicao(&mut h, &mut st, &rects, ids::INSP_MI_FOLD_SHAPE);
    let _ = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    h.settle_section_folds();
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    (h, st, rects)
}

/// O clique real que não precisa de mandar edição (abrir um grupo, abrir uma lista).
fn clica_sem_edicao(
    h: &mut ph2d_ui_testkit::MockPanelHost,
    st: &mut ph2d_panel_inspector::InspectorState,
    rects: &[(ph2d_a11y::NodeId, ph2d_editor_core::zones::Rect)],
    id: ph2d_a11y::NodeId,
) -> Vec<EditorAction> {
    let r = rect_de(rects, id).unwrap_or_else(|| panic!("{id:?} não foi pintado"));
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let _ = h.drained_actions();
    let mut evs = h.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
    evs.extend(h.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100)));
    for ev in evs {
        h.apply_panel_event::<InspectorPanel>(st, ev);
    }
    h.drained_actions()
}

/// ⭐ O grupo «Shape»: aberto, a caixa Remap envia `Remap(true)`; a faixa e a Curve só existem com ela; uma
/// predefinição escreve o texto `ph2d-curve` dela; o Smooth está sempre.
#[test]
fn o_shape_aberto_edita_remap_curva_e_smooth() {
    let (mut h, mut st, rects) = com_o_shape_aberto(info(SOURCE_HEALTH));
    assert!(
        rect_de(&rects, ids::INSP_MI_SMOOTH).is_some(),
        "Smooth sem Remap"
    );
    assert!(
        rect_de(&rects, ids::INSP_MI_REMAP_FROM).is_none(),
        "faixa só com Remap"
    );
    assert!(
        rect_de(&rects, ids::INSP_MI_CURVE[0]).is_none(),
        "curva só com Remap"
    );
    assert_eq!(
        clica(&mut h, &mut st, &rects, ids::INSP_MI_REMAP),
        vec![E::Remap(true)]
    );
    let mut com = info(SOURCE_HEALTH);
    com.remap = true;
    let (mut h, mut st, rects) = com_o_shape_aberto(com);
    for id in [ids::INSP_MI_REMAP_FROM, ids::INSP_MI_REMAP_TO] {
        assert!(rect_de(&rects, id).is_some(), "From | To com Remap");
    }
    assert_eq!(
        clica(&mut h, &mut st, &rects, ids::INSP_MI_CURVE[1]),
        vec![E::Curve(CURVE_PRESETS[1].1.to_string())],
        "Ease In"
    );
    assert_eq!(
        clica(&mut h, &mut st, &rects, ids::INSP_MI_CURVE[0]),
        vec![E::Curve(String::new())],
        "Linear = sem curva"
    );
}

/// Abre a lista `chip`, repinta, e clica a opção `opcao`.
fn escolhe_na_lista(
    i: InspectorMotionInputInfo,
    chip: ph2d_a11y::NodeId,
    opcao: ph2d_a11y::NodeId,
) -> Vec<E> {
    let (mut h, mut st, rects) = pinta(i);
    let _ = clica_sem_edicao(&mut h, &mut st, &rects, chip);
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    clica(&mut h, &mut st, &rects, opcao)
}

/// ⭐ A lista Action escreve o NOME da acção; a Property mostra o rótulo e escreve a CHAVE.
#[test]
fn as_listas_escrevem_a_accao_e_a_chave_do_campo() {
    let mut a = info(SOURCE_ACTION);
    a.acoes_do_mapa = vec!["jump".into(), "fire".into()];
    a.action = "jump".into();
    assert_eq!(
        escolhe_na_lista(a, ids::INSP_MI_ACTION, ids::INSP_MI_ACTION_OPT.id(1)),
        vec![E::Action("fire".into())]
    );
    let mut p = info(SOURCE_PROPERTY);
    p.campos_do_alvo = vec![
        (
            "Health \u{b7} Current".into(),
            "ph2d::physics::Health/current".into(),
        ),
        (
            "Health \u{b7} Max".into(),
            "ph2d::physics::Health/max".into(),
        ),
    ];
    assert_eq!(
        escolhe_na_lista(p, ids::INSP_MI_PROPERTY, ids::INSP_MI_PROPERTY_OPT.id(1)),
        vec![E::Property("ph2d::physics::Health/max".into())]
    );
}

/// ⭐ As saídas das queixas novas: «Add Physics Body» só com Speed sem corpo (Grounded não: pede um
/// Player); «Pick object» arma o conta-gotas; o conta-gotas da linha também.
#[test]
fn as_saidas_das_queixas_novas() {
    let mut g = info(SOURCE_GROUNDED);
    g.tem_corpo = false;
    let (_h, _st, rects) = pinta(g);
    assert!(
        rect_de(&rects, ids::INSP_MI_ADD_BODY).is_none(),
        "Grounded sem botão"
    );

    let mut s = info(SOURCE_SPEED);
    s.tem_corpo = false;
    let (mut h, mut st, rects) = pinta(s);
    let acoes = clica_sem_edicao(&mut h, &mut st, &rects, ids::INSP_MI_ADD_BODY);
    assert!(
        acoes.iter().any(|a| matches!(
            a,
            EditorAction::InspectorAddComponentRequested {
                entity_bits: BITS,
                component: Some("ph2d::physics::RigidBody")
            }
        )),
        "o clique pede o corpo"
    );

    // «Pick object» é a saída do From que não acha ninguém, e só dela: um campo que o alvo perdeu não a mostra.
    let mut p = info(SOURCE_PROPERTY);
    p.property = "ph2d::physics::Gone/x".into();
    assert_eq!(
        p.queixa(),
        Some(ph2d_editor_core::motion_input_edits::MotionInputQueixa::CampoAusente)
    );
    let (_h, _st, rects) = pinta(p);
    assert!(
        rect_de(&rects, ids::INSP_MI_PICK_OBJECT).is_none(),
        "Pick object só na queixa SemAlvo"
    );

    let mut f = info(SOURCE_HEALTH);
    f.from = "Boss".into();
    f.alvo_achado = false;
    let (mut h, mut st, rects) = pinta(f);
    assert_eq!(
        clica(&mut h, &mut st, &rects, ids::INSP_MI_PICK_OBJECT),
        vec![E::PickFrom(true)]
    );
    assert_eq!(
        clica(&mut h, &mut st, &rects, ids::INSP_MI_FROM_PICK),
        vec![E::PickFrom(true)]
    );
}

/// ⭐ As predefinições da Curve: lêem-se na `ph2d-curve`, vão de 0 a 1 e têm a forma do nome em ¼ e ¾.
#[test]
fn as_predefinicoes_da_curva_sao_o_que_dizem() {
    let x = [0.0, 0.25, 0.75, 1.0];
    let f = ph2d_editor_core::motion_input_edits::CURVE_PRESETS
        .map(|(_, t)| ph2d_curve::parse(t).map_or(x, |c| x.map(|x| c.eval(x))));
    assert_eq!(f[0], x, "Linear = \"\"");
    assert!(
        f.iter()
            .all(|v| v[0].abs() < 1e-6 && (v[3] - 1.0).abs() < 1e-6),
        "{f:?}"
    );
    assert!(
        f[1][1] < 0.2 && f[1][2] < 0.75 && f[2][1] > 0.25 && f[2][2] > 0.8,
        "{f:?}"
    );
    assert!(f[3][1] < 0.25 && f[3][2] > 0.75, "Smooth {f:?}");
}

/// ⭐ As fontes novas dizem «Sends …» em Play (o número que a ponte publicou); paradas, nenhum «Sends».
#[test]
fn as_fontes_novas_dizem_o_que_mandam_em_play() {
    use super::a_seccao_motion_input_esta_viva::frases;
    use ph2d_editor_core::motion_input_edits::MotionInputAoVivo as V;
    for (source, v, frase) in [
        (SOURCE_SPEED, 3.25, "Sends 3.25"),
        (SOURCE_GROUNDED, 1.0, "Sends 1.00"),
        (SOURCE_ACTION, 0.5, "Sends 0.50"),
        (SOURCE_PROPERTY, 10.0, "Sends 10.00"),
    ] {
        let mut i = info(source);
        i.tem_corpo = true;
        i.action = "jump".into();
        i.acoes_do_mapa = vec!["jump".into()];
        i.property = "ph2d::physics::Health/max".into();
        i.campos_do_alvo = vec![("Health · Max".into(), i.property.clone())];
        i.ao_vivo = Some(V::Numero(v));
        assert!(
            !frases(i.clone()).iter().any(|f| f.starts_with("Sends")),
            "parado, a fonte {source} não manda nada"
        );
        i.a_correr = true;
        let f = frases(i);
        assert!(f.contains(&frase.to_string()), "fonte {source}: {f:?}");
    }
}

/// ⭐ Parada, a Property diz o que lê no alvo, com o rótulo do campo (o da lista) e duas casas; em Play
/// a frase vira «Sends …». Campo que o alvo já não tem ⇒ nenhuma leitura (a queixa fala).
///
/// **Mutação que deve sangrar:** tirar o braço da Property, trocar o rótulo pela chave, ou o `!a_correr`.
#[test]
fn a_property_parada_diz_o_que_le_no_alvo() {
    use super::a_seccao_motion_input_esta_viva::frases;
    use ph2d_editor_core::motion_input_edits::MotionInputAoVivo as V;
    let mut i = info(SOURCE_PROPERTY);
    i.property = "ph2d::physics::Health/max".into();
    i.campos_do_alvo = vec![
        (
            "Health \u{b7} Current".into(),
            "ph2d::physics::Health/current".into(),
        ),
        ("Health \u{b7} Max".into(), i.property.clone()),
    ];
    i.ao_vivo = Some(V::Numero(100.0));
    let parada = frases(i.clone());
    assert!(
        parada.contains(&"Reads Health \u{b7} Max here: 100.00".to_string()),
        "{parada:?}"
    );
    i.a_correr = true;
    let jogo = frases(i.clone());
    assert!(!jogo.iter().any(|f| f.starts_with("Reads")), "{jogo:?}");
    assert!(jogo.contains(&"Sends 100.00".to_string()), "{jogo:?}");
    i.a_correr = false;
    i.property = "ph2d::physics::Gone/x".into();
    assert!(!frases(i).iter().any(|f| f.starts_with("Reads")));
}

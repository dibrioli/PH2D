//! **O cartão do Ray Sensor, versão B** (ficha `docs/GameEngineFuncional/componentes/07_ray_sensor.md`, dono
//! 15/11): Angle no lugar de Direction X·Y, o grupo Signals, o aviso da camada vazia — todo clique pelo
//! ponteiro REAL sobre o rect pintado, nos cartões e fora deles.

use crate::o_cartao_da_fisica::{achado, clica};
use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::ray_edits::{InspectorRayInfo, RayFieldEdit, RayQueixa, RaySee};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::ids as i;
use ph2d_panel_inspector::{
    InspectorPanel, InspectorState, set_cards_on, set_current_inspector_ray,
};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 3200.0,
};

fn raio() -> InspectorRayInfo {
    InspectorRayInfo {
        entity_bits: 0x0BAD_CAFE,
        angle: 0.0,
        reach: 6.0,
        layer: 0,
        layer_empty: false,
        turns_with_facing: false,
        ignore_body: true,
        ignore_parts: true,
        only_see: RaySee::Everything,
        see_tag: 0,
        walls_block: true,
        hit_triggers: false,
        starts_inside: false,
        all_hits: false,
        sees_all: Vec::new(),
        on_enter: "wall_ahead".into(),
        on_exit: "wall_gone".into(),
        sees: "Wall".into(),
        sees_at: 4.37,
        clock_playing: true,
        selected_count: 1,
    }
}

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

fn assente(
    info: &InspectorRayInfo,
    cartoes: bool,
) -> (MockPanelHost, InspectorState, Vec<(NodeId, Rect)>) {
    assente_com(info, cartoes, false)
}

/// Como [`assente`], e com o grupo «Filter» aberto quando `filtro`.
fn assente_com(
    info: &InspectorRayInfo,
    cartoes: bool,
    filtro: bool,
) -> (MockPanelHost, InspectorState, Vec<(NodeId, Rect)>) {
    let mut h = crate::host_em_metros();
    h.store_mut()
        .set_collapsed(ph2d_editor_core::ids::INSP_LIVE_RAY_SECTION, false);
    if filtro {
        h.store_mut().set_collapsed(i::INSP_RAY_FOLD_FILTER, false);
    }
    let mut st = InspectorState::default();
    set_current_inspector_ray(Some(info.clone()));
    let r = pinta(&mut h, &mut st, cartoes);
    (h, st, r)
}

fn edicoes(a: Vec<EditorAction>) -> Vec<RayFieldEdit> {
    a.into_iter()
        .filter_map(|a| match a {
            EditorAction::InspectorComponentEdit {
                edit: ComponentEdit::Ray(e),
                ..
            } => Some(e),
            _ => None,
        })
        .collect()
}

/// ⭐ Angle, Reach e os dois sinais ganham o foco sob o dedo; Origin e Direction X·Y não existem mais.
///
/// **Mutação que deve sangrar:** tirar a linha Angle do `sections/ray.rs`.
#[test]
fn os_campos_ganham_o_foco_sob_o_dedo() {
    nos_dois_modos(|c| {
        for id in [
            i::INSP_RAY_ANGLE,
            i::INSP_RAY_REACH,
            i::INSP_RAY_ON_ENTER,
            i::INSP_RAY_ON_EXIT,
        ] {
            let (mut h, mut st, r) = assente(&raio(), c);
            let rect = achado(&r, id).unwrap_or_else(|| panic!("{id:?} não pintado (cartões {c})"));
            let _ = clica(&mut h, &mut st, rect);
            assert_eq!(
                h.store().focus_id(),
                Some(id),
                "{id:?} morto sob o dedo (cartões {c})"
            );
            set_current_inspector_ray(None);
        }
    });
}

/// ⭐ Digitar 45 no Angle chega ao barramento como `Angle(45)` — em graus, sem conversão de mundo.
///
/// **Mutação que deve sangrar:** mapear `INSP_RAY_ANGLE` a outra edição no `event_ray`, ou pô-lo na
/// tabela de metros do `unidade_mundo` (sairia 0,45 ou 4500).
#[test]
fn digitar_45_no_angle_da_45_graus() {
    let (mut h, mut st, r) = assente(&raio(), true);
    assert!(achado(&r, i::INSP_RAY_ANGLE).is_some());
    let _ = h.drained_actions();
    for ev in h.type_into_number(i::INSP_RAY_ANGLE, "45") {
        let _ = h.apply_panel_event::<InspectorPanel>(&mut st, ev);
    }
    let saiu = edicoes(h.drained_actions());
    assert!(
        saiu.iter()
            .any(|e| matches!(e, RayFieldEdit::Angle(v) if (*v - 45.0).abs() < 1e-4)),
        "o Angle não chegou como 45 graus: {saiu:?}"
    );
    set_current_inspector_ray(None);
    set_cards_on(false);
}

/// ⭐ O grupo Signals nasce ABERTO, fecha e abre pelo clique no cabeçalho, e os sinais somem e voltam.
///
/// **Mutação que deve sangrar:** pintar os sinais fora do grupo, ou o grupo nascer fechado.
#[test]
fn o_grupo_signals_abre_e_fecha_pelo_clique() {
    nos_dois_modos(|c| {
        let (mut h, mut st, mut r) = assente(&raio(), c);
        assert!(
            achado(&r, i::INSP_RAY_ON_ENTER).is_some(),
            "Signals fechado de fábrica (cartões {c})"
        );
        for aberto in [false, true] {
            let cab = achado(&r, i::INSP_RAY_FOLD_SIGNALS).expect("o cabeçalho Signals");
            let _ = clica(&mut h, &mut st, cab);
            r = pinta(&mut h, &mut st, c);
            for id in [i::INSP_RAY_ON_ENTER, i::INSP_RAY_ON_EXIT] {
                assert_eq!(
                    achado(&r, id).is_some(),
                    aberto,
                    "{id:?} com Signals aberto={aberto} (cartões {c})"
                );
            }
        }
        set_current_inspector_ray(None);
    });
}

/// ⭐ O aviso «Nothing in the layers this ray sees» é a queixa da camada vazia e passa à frente do
/// «não vê nada»; o Angle tem dica de hover (0 = direita, 90 = cima).
#[test]
fn a_camada_vazia_e_a_dica_do_angle() {
    let mut info = raio();
    info.layer_empty = true;
    info.sees = String::new();
    assert_eq!(info.queixa(), Some(RayQueixa::CamadaVazia));
    let (h, _, _) = assente(&info, true);
    let dica = h.store().tooltip_for(i::INSP_RAY_ANGLE).unwrap_or_default();
    assert!(dica.contains("90"), "o Angle sem dica de direção: «{dica}»");
    set_current_inspector_ray(None);
    set_cards_on(false);
}

/// ⭐ A caixa «Turns with facing» (dono 15/11): o clique no rect pintado escreve o CONTRÁRIO do que o
/// snapshot mostra, nos cartões e fora deles.
///
/// **Mutação que deve sangrar:** escrever `info.turns_with_facing` sem o `!`.
#[test]
fn a_caixa_turns_with_facing_escreve_o_contrario_do_ecra() {
    nos_dois_modos(|c| {
        for ligada in [false, true] {
            let mut info = raio();
            info.turns_with_facing = ligada;
            let (mut h, mut st, r) = assente(&info, c);
            let rect = achado(&r, i::INSP_RAY_TURNS).expect("a caixa pintada");
            let saiu = edicoes(clica(&mut h, &mut st, rect));
            assert_eq!(
                saiu,
                vec![RayFieldEdit::TurnsWithFacing(!ligada)],
                "cartões {c}, ligada {ligada}"
            );
            set_current_inspector_ray(None);
        }
    });
}

/// ⭐ O grupo «Filter» nasce FECHADO (as caixas não existem), e o clique no cabeçalho abre-o e fecha-o.
///
/// **Mutação que deve sangrar:** o `set_collapsed_if_unchosen(.., true)` do `populate_ray` a `false`.
#[test]
fn o_grupo_filter_nasce_fechado_e_abre_pelo_clique() {
    nos_dois_modos(|c| {
        let (mut h, mut st, mut r) = assente(&raio(), c);
        assert!(
            achado(&r, i::INSP_RAY_IGNORE_BODY).is_none(),
            "Filter aberto de fábrica (cartões {c})"
        );
        for aberto in [true, false] {
            let cab = achado(&r, i::INSP_RAY_FOLD_FILTER).expect("o cabeçalho Filter");
            let _ = clica(&mut h, &mut st, cab);
            r = pinta(&mut h, &mut st, c);
            for id in [i::INSP_RAY_IGNORE_BODY, i::INSP_RAY_ALL_HITS] {
                assert_eq!(
                    achado(&r, id).is_some(),
                    aberto,
                    "{id:?} com Filter aberto={aberto} (cartões {c})"
                );
            }
        }
        set_current_inspector_ray(None);
    });
}

/// ⭐ Cada caixa do Filter: o clique REAL no rect pintado escreve o CONTRÁRIO do snapshot, ligada e
/// desligada, nos cartões e fora deles; cada uma tem dica de hover.
///
/// **Mutação que deve sangrar:** tirar o `!` de qualquer braço do `clique_do_filtro`.
#[test]
fn as_caixas_do_filter_escrevem_o_contrario_do_ecra() {
    type Caixa = (
        NodeId,
        fn(&mut InspectorRayInfo, bool),
        fn(bool) -> RayFieldEdit,
    );
    let caixas: [Caixa; 6] = [
        (
            i::INSP_RAY_IGNORE_BODY,
            |r, v| r.ignore_body = v,
            RayFieldEdit::IgnoreBody,
        ),
        (
            i::INSP_RAY_IGNORE_PARTS,
            |r, v| r.ignore_parts = v,
            RayFieldEdit::IgnoreParts,
        ),
        (
            i::INSP_RAY_WALLS_BLOCK,
            |r, v| r.walls_block = v,
            RayFieldEdit::WallsBlock,
        ),
        (
            i::INSP_RAY_HIT_TRIGGERS,
            |r, v| r.hit_triggers = v,
            RayFieldEdit::HitTriggers,
        ),
        (
            i::INSP_RAY_STARTS_INSIDE,
            |r, v| r.starts_inside = v,
            RayFieldEdit::StartsInside,
        ),
        (
            i::INSP_RAY_ALL_HITS,
            |r, v| r.all_hits = v,
            RayFieldEdit::AllHits,
        ),
    ];
    nos_dois_modos(|c| {
        for (id, poe, edicao) in caixas {
            for ligada in [false, true] {
                let mut info = raio();
                info.only_see = RaySee::OnlyTag;
                poe(&mut info, ligada);
                let (mut h, mut st, r) = assente_com(&info, c, true);
                assert!(h.store().tooltip_for(id).is_some(), "{id:?} sem dica");
                let rect =
                    achado(&r, id).unwrap_or_else(|| panic!("{id:?} não pintado (cartões {c})"));
                let saiu = edicoes(clica(&mut h, &mut st, rect));
                assert_eq!(
                    saiu,
                    vec![edicao(!ligada)],
                    "{id:?} cartões {c}, ligada {ligada}"
                );
                set_current_inspector_ray(None);
            }
        }
    });
}

/// ⭐ «Only see»: o clique em cada peça escreve o modo dela; a tag e «Walls block» só existem fora de
/// Everything (com controlo: com Everything somem).
///
/// **Mutação que deve sangrar:** pintar a tag e Walls block sem o `if i.only_see != Everything`.
#[test]
fn only_see_escolhe_e_a_tag_so_existe_com_filtro() {
    nos_dois_modos(|c| {
        for (n, modo) in [RaySee::Everything, RaySee::OnlyTag, RaySee::ExceptTag]
            .into_iter()
            .enumerate()
        {
            let mut info = raio();
            info.only_see = modo;
            let (mut h, mut st, r) = assente_com(&info, c, true);
            let com_filtro = modo != RaySee::Everything;
            for id in [i::INSP_RAY_SEE_TAG, i::INSP_RAY_WALLS_BLOCK] {
                assert_eq!(
                    achado(&r, id).is_some(),
                    com_filtro,
                    "{id:?} com {modo:?} (cartões {c})"
                );
            }
            let outro = (n + 1) % 3;
            let peca = achado(&r, i::INSP_RAY_ONLY_SEE[outro]).expect("a peça pintada");
            let saiu = edicoes(clica(&mut h, &mut st, peca));
            let esperado = [RaySee::Everything, RaySee::OnlyTag, RaySee::ExceptTag][outro];
            assert_eq!(saiu, vec![RayFieldEdit::OnlySee(esperado)], "cartões {c}");
            set_current_inspector_ray(None);
        }
    });
}

/// ⭐ A tag: o clique no chip abre a lista da árvore, e o clique numa opção escreve o `TagId` dela.
///
/// **Mutação que deve sangrar:** o despacho resolver a opção pela lista do Nav (`INSP_NAV_TAG_OPT`).
#[test]
fn a_tag_do_filtro_escolhe_se_na_lista_da_arvore() {
    use ph2d_editor_core::screens::hero::InspectorTagRow;
    ph2d_panel_inspector::set_current_tag_tree(vec![
        InspectorTagRow {
            id: 11,
            path: "Enemy".into(),
            label: "Enemy".into(),
            depth: 0,
        },
        InspectorTagRow {
            id: 22,
            path: "Coin".into(),
            label: "Coin".into(),
            depth: 0,
        },
    ]);
    nos_dois_modos(|c| {
        let mut info = raio();
        info.only_see = RaySee::OnlyTag;
        let (mut h, mut st, r) = assente_com(&info, c, true);
        let chip = achado(&r, i::INSP_RAY_SEE_TAG).expect("o chip da tag");
        let _ = clica(&mut h, &mut st, chip);
        let r = pinta(&mut h, &mut st, c);
        let opcao = achado(&r, i::INSP_RAY_SEE_TAG_OPT[1]).expect("a lista aberta, com Coin");
        let saiu = edicoes(clica(&mut h, &mut st, opcao));
        assert_eq!(saiu, vec![RayFieldEdit::SeeTag(22)], "cartões {c}");
        set_current_inspector_ray(None);
    });
    ph2d_panel_inspector::set_current_tag_tree(Vec::new());
}

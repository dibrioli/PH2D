//! **A costura da secção DRAWING RULES** (W4.5): o «×» e a medida de cada linha chegam ao barramento com o
//! `RegraId` DELA, pelo gesto REAL (`click_at`, `type_into_number`) — os ids da fixtura são `7` e `42`, nunca a
//! posição. O lado do documento está em `ph2d-app-vec` (`cad_inspector`).

use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
use ph2d_editor_core::cad_edits::{
    CadFieldEdit as E, EstadoDaRegra, InspectorCadInfo, InspectorRegraRow,
};
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::{InspectorPanel, InspectorState, ids, set_current_inspector_cad};
use ph2d_ui_testkit::MockPanelHost;

const ENTITY: u64 = 0x1_0000_00C7;
const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 320.0,
    h: 2400.0,
};

fn regra(id: u32, nome: &'static str, medida: Option<f32>, referencia: bool) -> InspectorRegraRow {
    InspectorRegraRow {
        id,
        nome,
        estado: EstadoDaRegra::Ok,
        medida,
        graus: false,
        referencia,
        // A família CAD é a 9 (`ph2d_ecs::canal::familia::CAD`); o painel não depende da `ph2d-ecs`.
        canal: [9, 1, u16::try_from(id).unwrap_or(0), 0],
    }
}

/// Três linhas: uma horizontal (sem medida, id `7`), uma distância de 1,24 m (`42`) e uma distância REF (`5`).
/// O canal fala METROS (graus num ângulo); o ecrã, o que o projecto escolheu.
fn regras() -> InspectorCadInfo {
    InspectorCadInfo {
        entity_bits: ENTITY,
        rows: vec![
            regra(7, "app.vec.cad.lei.horizontal", None, false),
            regra(42, "app.vec.cad.lei.distancia", Some(1.24), false),
            regra(5, "app.vec.cad.lei.distancia", Some(0.8), true),
        ],
    }
}

fn pinta(host: &mut MockPanelHost, state: &mut InspectorState) -> Vec<(ph2d_a11y::NodeId, Rect)> {
    host.paint::<InspectorPanel>(state, VIEWPORT)
}

fn aplica(host: &mut MockPanelHost, state: &mut InspectorState, evs: Vec<WidgetEvent>) {
    for ev in evs {
        let _ = host.apply_panel_event::<InspectorPanel>(state, ev);
    }
}

fn edicoes(acoes: Vec<EditorAction>) -> Vec<E> {
    acoes
        .into_iter()
        .filter_map(|a| match a {
            EditorAction::InspectorComponentEdit {
                entity_bits,
                edit: ComponentEdit::Cad(e),
            } if entity_bits == ENTITY => Some(e),
            _ => None,
        })
        .collect()
}

fn rect_de(rects: &[(ph2d_a11y::NodeId, Rect)], id: ph2d_a11y::NodeId) -> Option<Rect> {
    rects.iter().find(|(n, _)| *n == id).map(|(_, r)| *r)
}

/// ⭐⭐ O «×» da linha `k` manda apagar a regra DELA — o `42` da 2.ª linha, nunca `1`.
///
/// **Mutações que devem sangrar:** publicar a posição em vez do `row.id` · tirar o `populate_cad` · tirar o
/// braço do `event_cad`.
#[test]
fn o_x_de_cada_linha_apaga_a_regra_dela() {
    for (k, esperado) in [(1, 42), (0, 7)] {
        let mut host = MockPanelHost::with_panel::<InspectorPanel>();
        let mut state = InspectorState::default();
        set_current_inspector_cad(Some(regras()));
        let rects = pinta(&mut host, &mut state);
        let id = ids::INSP_CAD_APAGAR[k];
        let r = rect_de(&rects, id).unwrap_or_else(|| panic!("a secção não pintou o «×» {k}"));
        let evs = host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5);
        assert!(!evs.is_empty(), "o «×» {k} está MORTO sob o dedo");
        aplica(&mut host, &mut state, evs);
        let e = edicoes(host.drained_actions());
        set_current_inspector_cad(None);
        assert_eq!(e, vec![E::Apagar(esperado)]);
    }
}

/// ⭐⭐ Escrever na medida da 2.ª linha manda `Medida(42, v)`; a lei sem medida não pinta campo, e a REF mostra o
/// valor sem campo nenhum.
#[test]
fn escrever_a_medida_manda_o_valor_com_o_id_da_regra() {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_cad(Some(regras()));
    let rects = pinta(&mut host, &mut state);
    assert!(
        rect_de(&rects, ids::INSP_CAD_MEDIDA[0]).is_none(),
        "a horizontal não tem medida"
    );
    assert!(
        rect_de(&rects, ids::INSP_CAD_MEDIDA[2]).is_none(),
        "uma REF só mostra a medida: um campo ali mentiria"
    );
    assert!(rect_de(&rects, ids::INSP_CAD_MEDIDA[1]).is_some());
    let evs = host.type_into_number(ids::INSP_CAD_MEDIDA[1], "150");
    aplica(&mut host, &mut state, evs);
    let e = edicoes(host.drained_actions());
    set_current_inspector_cad(None);
    assert!(
        matches!(e.last(), Some(&E::Medida(42, v)) if (v - 1.5).abs() < 1e-6),
        "150 px a 100 px/m são 1,5 m: {e:?}"
    );
}

/// Mostra a medida de `info` com o projecto em `unidade`/`angulo` a 100 px/m, escreve `texto` no campo da linha
/// `k` e devolve `(o que o campo mostrou, as edições)`.
fn mostra_e_escreve(
    info: InspectorCadInfo,
    unidade: ph2d_editor_core::project::DisplayUnit,
    angulo: ph2d_editor_core::project::DisplayAngle,
    k: usize,
    texto: &str,
) -> (Option<f64>, Vec<E>) {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    host.project_mut().display_unit = unidade;
    host.project_mut().pixels_per_meter = 100.0;
    host.project_mut().display_angle = angulo;
    set_current_inspector_cad(Some(info));
    pinta(&mut host, &mut state);
    let mostrado = host.store().number_value(ids::INSP_CAD_MEDIDA[k]);
    let evs = host.type_into_number(ids::INSP_CAD_MEDIDA[k], texto);
    aplica(&mut host, &mut state, evs);
    let e = edicoes(host.drained_actions());
    set_current_inspector_cad(None);
    (mostrado, e)
}

/// ⭐⭐ **A medida obedece ao px/metros e graus/radianos do projecto** (o defeito da cena 6: o dente de ~124 px
/// lia «1.239 px»). Em px, 1,24 m mostra 124 e o 150 escrito volta 1,5 m; em metros mostra 1,24 e o 1,5 volta
/// 1,5; um ângulo de 90° em radianos mostra π/2 e o 1 escrito volta 57,3°.
///
/// **Mutações que devem sangrar:** semear o número do canal sem converter · emitir o escrito sem converter.
#[test]
fn a_medida_obedece_a_px_metros_e_graus_radianos_do_projecto() {
    use ph2d_editor_core::project::{DisplayAngle, DisplayUnit};
    let perto = |m: Option<f64>, v: f64| m.is_some_and(|m| (m - v).abs() < 1e-4);
    let (m, e) = mostra_e_escreve(
        regras(),
        DisplayUnit::Pixels,
        DisplayAngle::Degrees,
        1,
        "150",
    );
    assert!(perto(m, 124.0), "em px, 1,24 m mostrou {m:?}");
    assert!(
        matches!(e.last(), Some(&E::Medida(42, v)) if (v - 1.5).abs() < 1e-6),
        "{e:?}"
    );
    let (m, e) = mostra_e_escreve(
        regras(),
        DisplayUnit::Meters,
        DisplayAngle::Degrees,
        1,
        "1.5",
    );
    assert!(perto(m, 1.24), "em metros, 1,24 m mostrou {m:?}");
    assert!(
        matches!(e.last(), Some(&E::Medida(42, v)) if (v - 1.5).abs() < 1e-6),
        "{e:?}"
    );
    let mut info = regras();
    info.rows[1].nome = "app.vec.cad.lei.angulo";
    info.rows[1].graus = true;
    info.rows[1].medida = Some(90.0);
    let (m, e) = mostra_e_escreve(info, DisplayUnit::Pixels, DisplayAngle::Radians, 1, "1");
    assert!(
        perto(m, std::f64::consts::FRAC_PI_2),
        "90° em radianos mostrou {m:?}"
    );
    assert!(
        matches!(e.last(), Some(&E::Medida(42, v)) if (v - 57.29578).abs() < 1e-3),
        "{e:?}"
    );
}

/// ⭐ Sem regras do path, a secção não aparece.
#[test]
fn sem_regras_a_seccao_nao_aparece() {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_cad(None);
    let rects = pinta(&mut host, &mut state);
    assert!(rect_de(&rects, ph2d_editor_core::ids::INSP_LIVE_CAD_SECTION).is_none());
    assert!(rect_de(&rects, ids::INSP_CAD_APAGAR[0]).is_none());
}

//! ⭐⭐⭐ **A secção RAY SENSOR é PINTADA, está VIVA sob o dedo, e mostra os números DO OBJECTO**
//! (suplente #21).
//!
//! # ⛔⛔⛔ Porque a terceira metade existe, e porque ela nasceu de uma FOTO
//!
//! A W4 shipou a secção com os seis campos pintados, registados e clicáveis — e **a mostrar os
//! valores de FÁBRICA do `populate_ray`**. A foto da cena `PH2D_RAY_SMOKE=1` (19/09) apanhou-a:
//! `Direction 0 / −1` e `Reach 1 m` sobre um olho autorado a `(1, 0)` com alcance `6`, **com a
//! linha desenhada a 6 m no canvas ao lado e a leitura viva a dizer `Sees Caixa, at 3,57 m`**.
//!
//! ⚠️⚠️ *Quatro números plausíveis são a pior forma deste defeito:* eles não parecem partidos, e o
//! artista que escreva por cima de um deles **grava o default nos outros três**. É a mesma família
//! que a secção do ÁUDIO e a da CÂMERA pagaram em 10/09 e a da VIGIA DO CONTADOR em 16/09.
//!
//! ⛔ **Nenhum gate da W4 podia vê-la:** eles medem a porta da queixa e o dreno das edições, e os
//! dois entram **abaixo** do `WidgetStore`. *A semente é a única metade da secção cujo sujeito é o
//! widget*, e por isso ela precisa de um gate que PINTE.

use ph2d_editor_core::ray_edits::InspectorRayInfo;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::{InspectorPanel, InspectorState, set_current_inspector_ray};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};

/// ⭐ Um olho que **NÃO** é o `RaySensor::default()` em nenhum dos seis números — é isso que faz o
/// gate da semente discriminar: com uma fixtura no ponto neutro de um campo, esse campo não é
/// testado (*um corpus no NEUTRO de um knob não testa esse knob*).
fn olho() -> InspectorRayInfo {
    InspectorRayInfo {
        entity_bits: 0x00AB_CDEF,
        origin_x: 0.25,
        origin_y: -0.5,
        dir_x: 1.0,
        dir_y: 0.0,
        reach: 6.0,
        layer: 3,
        on_enter: "vi".into(),
        on_exit: "perdi".into(),
        sees: "Caixa".into(),
        sees_at: 3.57,
        clock_playing: true,
        selected_count: 1,
    }
}

fn host(i: InspectorRayInfo) -> (MockPanelHost, InspectorState) {
    let h = MockPanelHost::with_panel::<InspectorPanel>();
    set_current_inspector_ray(Some(i));
    (h, InspectorState::default())
}

/// Os seis números da secção, na ordem em que o painel os pinta.
const NUMEROS: [(ph2d_a11y::NodeId, f64); 6] = [
    (ph2d_panel_inspector::ids::INSP_RAY_ORIGIN_X, 0.25),
    (ph2d_panel_inspector::ids::INSP_RAY_ORIGIN_Y, -0.5),
    (ph2d_panel_inspector::ids::INSP_RAY_DIR_X, 1.0),
    (ph2d_panel_inspector::ids::INSP_RAY_DIR_Y, 0.0),
    (ph2d_panel_inspector::ids::INSP_RAY_REACH, 6.0),
    (ph2d_panel_inspector::ids::INSP_RAY_LAYER, 3.0),
];

/// ⭐⭐ **TODO campo da secção é pintado com área clicável** — a lição que o `Timers` cobrou.
///
/// **Mutação que deve sangrar:** tirar qualquer linha do pintor da secção.
#[test]
fn todo_campo_da_seccao_e_pintado() {
    let (mut h, mut st) = host(olho());
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    for (id, _) in NUMEROS {
        let r = rects
            .iter()
            .find(|(n, _)| *n == id)
            .map(|(_, r)| *r)
            .unwrap_or_else(|| panic!("o campo {id:?} nao foi PINTADO com area clicavel"));
        assert!(r.w > 0.0 && r.h > 0.0, "campo {id:?} sem area: {r:?}");
    }
    for id in [
        ph2d_panel_inspector::ids::INSP_RAY_ON_ENTER,
        ph2d_panel_inspector::ids::INSP_RAY_ON_EXIT,
    ] {
        assert!(
            rects.iter().any(|(n, _)| *n == id),
            "o campo de texto {id:?} nao foi pintado"
        );
    }
    set_current_inspector_ray(None);
}

/// ⭐⭐⭐ **OS CAMPOS MOSTRAM OS NÚMEROS DO OBJECTO, e não os de fábrica** — o gate que a foto
/// encomendou.
///
/// ⚠️ **A fixtura difere do `RaySensor::default()` nos SEIS campos**, senão um campo semeado no
/// ponto neutro passaria com a semente apagada.
///
/// **Mutação que deve sangrar:** tirar a chamada do `sync_ray` do `sync_sections`.
#[test]
fn os_campos_mostram_os_numeros_do_objecto_e_nao_os_de_fabrica() {
    let (mut h, mut st) = host(olho());
    let _ = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    for (id, esperado) in NUMEROS {
        let lido = h
            .store()
            .number_value(id)
            .unwrap_or_else(|| panic!("o campo {id:?} nem sequer esta' registado"));
        assert!(
            (lido - esperado).abs() < 1.0e-6,
            "o campo {id:?} mostra {lido} e o objecto tem {esperado} — o painel esta' a mostrar os \
             valores de FABRICA do `populate_ray`, e quem escrever por cima de UM grava o default \
             nos outros cinco"
        );
    }
    set_current_inspector_ray(None);
}

/// ⭐⭐ **E a MÃO do artista ganha ao instantâneo** — com o campo em foco, a semente não lhe toca.
///
/// ⚠️ Sem esta metade a semente apagaria o que ele está a digitar antes de o commit da shell voltar
/// — a lei que as cinco irmãs já escrevem, e a razão de a semente ser de ARESTA.
///
/// **Mutação que deve sangrar:** tirar o `continue` do `focus == Some(id)`.
#[test]
fn a_mao_do_artista_ganha_a_semente() {
    let (mut h, mut st) = host(olho());
    let _ = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    let id = ph2d_panel_inspector::ids::INSP_RAY_REACH;

    // ⚠️ O `store_mut` vive no trait do HOST, não no mock — sem ele em alcance, um gate que quer
    //    fingir um dedo no campo não compila.
    use ph2d_editor_core::panel::PanelHostInternal as _;
    // O artista pega no campo e escreve outro número.
    h.store_mut().set_focus(Some(id));
    h.store_mut().set_number_value(id, 42.0);
    // E o instantâneo muda por baixo dele (outro objecto, outra aresta).
    let mut outro = olho();
    outro.entity_bits = 0x00FF_0001;
    outro.reach = 9.0;
    set_current_inspector_ray(Some(outro));
    let _ = h.paint::<InspectorPanel>(&mut st, VIEWPORT);

    assert!(
        (h.store().number_value(id).unwrap_or_default() - 42.0).abs() < 1.0e-6,
        "a semente escreveu por cima do campo em FOCO — o artista perde o que estava a escrever"
    );
    set_current_inspector_ray(None);
}

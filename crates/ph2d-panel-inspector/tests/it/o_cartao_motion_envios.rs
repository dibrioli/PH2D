//! **O cartão MOTION** (GEF 08 MI-8): em Play, «Sent <signal> N times» por nó «Send signal» do
//! objecto; sem instantâneo (parado, ou sem nó) o cartão não existe.

use super::a_seccao_motion_input_esta_viva::VIEWPORT;
use ph2d_editor_core::motion_sent::{InspectorMotionSentInfo, MotionSent};
use ph2d_panel_inspector::{
    InspectorPanel, InspectorState, censo_dos_avisos, set_current_inspector_motion_sent,
};
use ph2d_ui_testkit::MockPanelHost;

fn pinta(i: Option<InspectorMotionSentInfo>) -> (Vec<String>, bool) {
    let mut h = MockPanelHost::with_panel::<InspectorPanel>();
    set_current_inspector_motion_sent(i);
    let mut st = InspectorState::default();
    let (rects, avisos) =
        censo_dos_avisos::medindo(|| h.paint::<InspectorPanel>(&mut st, VIEWPORT));
    set_current_inspector_motion_sent(None);
    let cartao = rects
        .iter()
        .any(|(n, _)| *n == ph2d_editor_core::ids::INSP_LIVE_MOTION_SECTION);
    let frases = avisos
        .into_iter()
        .filter(|a| a.seccao() == "motion_sent")
        .map(|a| a.texto)
        .collect();
    (frases, cartao)
}

fn envio(signal: &str, times: u64) -> MotionSent {
    MotionSent {
        signal: signal.into(),
        times,
    }
}

/// **Mutação que deve sangrar:** tirar o `push` do cartão em `paint_familias`, ou trocar a frase.
#[test]
fn o_cartao_motion_diz_quantas_vezes_cada_sinal_foi_enviado() {
    let (frases, cartao) = pinta(Some(InspectorMotionSentInfo {
        entity_bits: 7,
        sent: vec![
            envio("door_open", 2),
            envio("low_health", 1),
            envio("hit", 0),
        ],
    }));
    assert!(cartao, "o cabeçalho do cartão Motion é pintado");
    assert_eq!(
        frases,
        [
            "Sent door_open 2 times",
            "Sent low_health once",
            "hit not sent yet"
        ]
    );
    let (frases, cartao) = pinta(None);
    assert!(
        !cartao && frases.is_empty(),
        "parado: sem cartão {frases:?}"
    );
}

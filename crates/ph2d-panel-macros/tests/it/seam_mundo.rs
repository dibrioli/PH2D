//! A costura das etapas do MUNDO (W19 §5): a etapa «Simulate» PINTA-SE com a duração, e escolhida não
//! pinta o Apagar (uma etapa do mundo não se apaga — a app recusaria; o botão mentiria).

use crate::seam_macros::{BITS, Palco, info};
use ph2d_editor_core::macro_edits::{EtapaView, FamiliaDaMacro, PropsDaEtapa, TipoDaEtapaView};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_macros::{MacrosPanel, ids};

#[test]
fn a_etapa_simular_do_mundo_pinta_se_e_nao_oferece_apagar() {
    let mut i = info(None, None);
    let o = i.objeto.as_mut().expect("o objecto");
    assert_eq!(o.entity_bits, BITS);
    o.etapas.push(EtapaView {
        trilha: 1,
        k: 0,
        familia: FamiliaDaMacro::Mundo,
        tipo: TipoDaEtapaView::Simula { segundos: 2.0 },
        nome: None,
        dica: None,
        t0_s: 3.0,
        t1_s: 5.0,
        espera_s: 0.0,
        props: PropsDaEtapa::Nenhuma,
    });
    let mut p = Palco::novo(i);
    let medidos = p
        .host
        .medindo_a_pintura::<MacrosPanel>(&mut p.st, Rect::new(0.0, 0.0, 320.0, 1600.0));
    let pintada = medidos
        .iter()
        .find(|m| m.texto == "4. Simulate \u{b7} 2.00 s")
        .unwrap_or_else(|| {
            panic!(
                "a etapa Simular nunca se pintou; pintou-se: {:?}",
                medidos.iter().map(|m| &m.texto).collect::<Vec<_>>()
            )
        });
    assert_eq!(pintada.pintado, pintada.texto, "a frase cortou-se na linha");
    p.pinta();
    let _ = p.carrega(ids::row_id(1, 0));
    p.pinta();
    assert!(
        p.rect(ids::MACROS_WAIT).is_some(),
        "a espera de uma etapa do mundo serve"
    );
    assert!(
        p.rect(ids::MACROS_DELETE).is_none(),
        "o Apagar de uma etapa do mundo pintou-se: a app recusá-lo-ia"
    );
    // Controlo: uma pincelada escolhida pinta o Apagar.
    let _ = p.carrega(ids::row_id(0, 0));
    p.pinta();
    assert!(p.rect(ids::MACROS_DELETE).is_some());
}

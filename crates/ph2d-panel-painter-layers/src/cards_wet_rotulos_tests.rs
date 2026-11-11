//! B35.11 — nenhum rótulo do cartão Wet repete o «Hardness» do Shape (a silhueta): o do Wet é o dente do papel.

use crate::seccoes::{WETPAINT, WETPAINT_CARTAO};
use ph2d_i18n::tr;

#[test]
fn os_rotulos_do_cartao_wet_nao_repetem_hardness() {
    let shape = tr("panel.painter_layers.shape.hardness");
    let repetidos: Vec<_> = WETPAINT
        .iter()
        .chain(WETPAINT_CARTAO)
        .filter(|k| tr(k) == shape)
        .collect();
    assert!(
        repetidos.is_empty(),
        "rótulo «{shape}» repetido no cartão Wet: {repetidos:?}"
    );
}

#[test]
fn cada_dica_do_cartao_wet_existe() {
    for (_, k) in super::DICAS {
        assert_ne!(tr(k), k, "dica sem texto: {k}");
    }
}

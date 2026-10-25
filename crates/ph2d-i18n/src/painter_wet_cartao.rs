//! ⭐ **Os knobs do motor que o cartão Wet Paint também mostra** (dono 25/10: os que mais decidem o pincel e o
//! preset, ao lado do Tuning). Tabela IRMÃ da [`super::painter_layers`] (tecto de tamanho; `CLAUDE.md` §0.2).

/// A tradução de uma chave `panel.painter_wet_cartao.*`, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "panel.painter_wet_cartao.backrun" => "Backruns",
        "panel.painter_wet_cartao.granulation" => "Granulation",
        "panel.painter_wet_cartao.wet_bloom" => "Wet bloom",
        "panel.painter_wet_cartao.tip_pickup" => "Tip pickup",
        _ => return None,
    })
}

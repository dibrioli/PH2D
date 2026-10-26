//! ⭐ **Os knobs do motor que o cartão Wet Paint também mostra** (dono 25/10: os que mais decidem o pincel e o
//! preset, ao lado do Tuning). Tabela IRMÃ da [`super::painter_layers`] (tecto de tamanho; `CLAUDE.md` §0.2).

/// A tradução de uma chave `panel.painter_wet_cartao.*`, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "panel.painter_wet_cartao.backrun" => "Backruns",
        "panel.painter_wet_cartao.granulation" => "Granulation",
        // «Pigment diffusion» corta no dock mínimo (`cada_nome_deste_painel_cabe_na_coluna_da_seccao`); o nome inteiro
        // vai na dica.
        "panel.painter_wet_cartao.diffusion" => "Diffusion",
        "panel.painter_wet_cartao.diffusion_tip" => {
            "Pigment diffusion: how far the pigment spreads in the wet paint."
        }
        "panel.painter_wet_cartao.wet_bloom" => "Wet bloom",
        "panel.painter_wet_cartao.tip_pickup" => "Tip pickup",
        "panel.painter_wet_cartao.wet_bloom_tip" => {
            "Only acts when the brush enters WET paint of another colour: the wet paint opens around the stroke."
        }
        "panel.painter_wet_cartao.tip_pickup_tip" => {
            "Only acts when the brush enters WET paint of another colour: the tip picks it up and carries it along."
        }
        // A linha da ferramenta da água na mão (dono 25/10, o painel do Rebelle).
        "panel.painter_wet_cartao.strength" => "Strength",
        "panel.painter_wet_cartao.smear_water" => "Water",
        "panel.painter_wet_cartao.absorbency" => "Absorbency",
        _ => return None,
    })
}

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
        // A água que re-molha e a tinta que se deixa dissolver (dono 30/10, J10).
        "panel.painter_wet_cartao.rewet" => "Re-wet",
        "panel.painter_wet_cartao.rewet_tip" => {
            "How much this brush's water dissolves soluble paint under it"
        }
        "panel.painter_wet_cartao.rewet_drag" => "Drag",
        "panel.painter_wet_cartao.rewet_drag_tip" => {
            "How much this brush's water drags soluble paint along the stroke, like Smear"
        }
        "panel.painter_wet_cartao.solubility" => "Solubility",
        "panel.painter_wet_cartao.solubility_tip" => {
            "How much this paint, once laid, dissolves under later water"
        }
        // Como a tinta se dá com a de baixo (dono 30/10).
        "panel.painter_wet_cartao.staining" => "Staining",
        "panel.painter_wet_cartao.staining_tip" => {
            "How much this paint stains the paper: a staining paint resists being lifted by later water, Smear or Blend"
        }
        "panel.painter_wet_cartao.glaze" => "Glaze",
        "panel.painter_wet_cartao.glaze_tip" => {
            "How much this paint filters the colour under it like a transparent layer, instead of covering it"
        }
        "panel.painter_wet_cartao.cover" => "Cover",
        "panel.painter_wet_cartao.cover_tip" => {
            "How much this paint hides WET paint of another colour under it instead of mixing with it, like gouache"
        }
        "panel.painter_wet_cartao.wet_soften" => "Wet soften",
        "panel.painter_wet_cartao.wet_soften_tip" => {
            "How much the edge of the stroke softens on wet paper"
        }
        // A linha da ferramenta da água na mão (dono 25/10, o painel do Rebelle).
        "panel.painter_wet_cartao.strength" => "Strength",
        "panel.painter_wet_cartao.smear_water" => "Water",
        "panel.painter_wet_cartao.absorbency" => "Absorbency",
        _ => return None,
    })
}

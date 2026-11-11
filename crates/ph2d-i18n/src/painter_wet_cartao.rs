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
            "Only acts when the brush enters WET paint of another color: the wet paint opens around the stroke."
        }
        "panel.painter_wet_cartao.tip_pickup_tip" => {
            "Only acts when the brush enters WET paint of another color: the tip picks it up and carries it along."
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
            "How much this paint filters the color under it like a transparent layer, instead of covering it"
        }
        "panel.painter_wet_cartao.cover" => "Cover",
        "panel.painter_wet_cartao.cover_tip" => {
            "How much this paint hides WET paint of another color under it instead of mixing with it, like gouache"
        }
        "panel.painter_wet_cartao.dry_cover" => "Dry cover",
        "panel.painter_wet_cartao.dry_cover_tip" => {
            "How much this paint hides DRY paint of another color under it, like opaque gouache over a dried layer"
        }
        // B35.12: o arrasto, o relevo e a guache saíram do Tuning.
        "panel.painter_wet_cartao.drag" => "Paint drag",
        "panel.painter_wet_cartao.drag_tip" => {
            "How much the wet paint under the brush gets pulled along the stroke. 0 lays each dab where it lands"
        }
        "panel.painter_wet_cartao.emboss" => "Emboss",
        "panel.painter_wet_cartao.emboss_tip" => {
            "How tall thick paint stands on the canvas, so light and shadow show its ridges. 0 is flat"
        }
        "panel.painter_wet_cartao.opaque" => "Opaque",
        "panel.painter_wet_cartao.opaque_tip" => {
            "How much this paint hides a different color under it, like gouache, instead of letting it show through"
        }
        "panel.painter_wet_cartao.wet_soften" => "Wet soften",
        "panel.painter_wet_cartao.wet_soften_tip" => {
            "How much the edge of the stroke softens on wet paper"
        }
        // A linha da ferramenta da água na mão (dono 25/10, o painel do Rebelle).
        "panel.painter_wet_cartao.strength" => "Strength",
        "panel.painter_wet_cartao.smear_water" => "Water",
        "panel.painter_wet_cartao.absorbency" => "Absorbency",
        "panel.painter_wet_cartao.water_drag" => "Drag",
        "panel.painter_wet_cartao.water_drag_tip" => {
            "How much the water carries the pigment it lifted along the stroke: the color fades in the direction of the gesture"
        }
        // Os controlos de lei da E10 (`docs/Painter/Arquitetura/F3_controlos.md` §9).
        "panel.painter_wet_cartao.paint_type_tip" => {
            "Water: the paint thins with water, flows and dries fast. Oil: the thinner is oily — thick paint stays put, thinned paint flows slowly and stays wet much longer."
        }
        "panel.painter_wet_cartao.dilution_tip" => {
            "How much water or medium is in the mix: 0 = paint straight from the tube, 1 = almost clear."
        }
        "panel.painter_wet_cartao.volume_tip" => {
            "How much paint each touch leaves on the paper, in micrometres: a thin film or a thick paste."
        }
        "panel.painter_wet_cartao.body_tip" => {
            "How stiff the paint is: high stays exactly where the brush leaves it, low levels out."
        }
        "panel.painter_wet_cartao.smooth_edges_tip" => {
            "How soft the outline of the next strokes is: 0 = crisp, higher = softer, in color and in relief. Paint already on the canvas stays as it is."
        }
        "panel.painter_wet_cartao.hardness_tip" => {
            "How deep the tip reaches into the paper's tooth: 0 = only the peaks, 1 = fills the valleys."
        }
        "panel.painter_wet_cartao.absorbency_tip" => {
            "How fast the paper drinks water: high = soft edges and quick soaking, low = paint sits on top."
        }
        _ => return None,
    })
}

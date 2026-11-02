//! A secção **Convert** do painel Vector (o Offset Path virou ferramenta em F48d, `bars_row`) —
//! irmãs de `paint_sections.rs` pelo teto de 600 LOC. O perfil de largura (Power Stroke) mora no
//! «Stroke settings» desde 02/11; as linhas dele continuam neste ficheiro.

use super::BodyCtx;
use ph2d_i18n::TextKey;
use ph2d_i18n::tr;
use ph2d_tool_vector::params;

const SLIDERS: [(TextKey, ph2d_a11y::NodeId, ph2d_a11y::NodeId); 4] = [
    (
        TextKey::new("panel.vector.expand.w_start"),
        crate::ids::VECTOR_EXPAND_W_START,
        ph2d_tool_vector::ids::VECTOR_EXPAND_W_START_NUM,
    ),
    (
        TextKey::new("panel.vector.expand.w_mid"),
        crate::ids::VECTOR_EXPAND_W_MID,
        ph2d_tool_vector::ids::VECTOR_EXPAND_W_MID_NUM,
    ),
    (
        TextKey::new("panel.vector.expand.w_end"),
        crate::ids::VECTOR_EXPAND_W_END,
        ph2d_tool_vector::ids::VECTOR_EXPAND_W_END_NUM,
    ),
    (
        TextKey::new("panel.vector.expand.w_pos"),
        crate::ids::VECTOR_EXPAND_W_POS,
        ph2d_tool_vector::ids::VECTOR_EXPAND_W_POS_NUM,
    ),
];

/// **Os quatro trilhos do perfil que estão na TELA** — o store quando o artista já tocou, o default
/// da tool quando não. A fileira de perfis e os quatro sliders leem daqui.
fn live_tracks(store: &ph2d_editor_core::interaction::WidgetStore) -> [f32; 4] {
    let fallback = params::preset_tracks(&params::WPROFILE_DEFAULT);
    let mut tracks = [0.0_f32; 4];
    for (i, (_, slider, _)) in SLIDERS.iter().enumerate() {
        tracks[i] = store.slider(*slider).map_or(fallback[i], |(_, v)| v);
    }
    tracks
}

impl BodyCtx<'_> {
    /// ⭐ A secção **Convert** — o traço vira forma (o *Outline Stroke* do Figma e do Illustrator,
    /// também em `Ctrl+Alt+O`).
    pub(crate) fn convert_section(&mut self, y: f32) -> f32 {
        let (y, collapsed) = self.section_header(
            crate::ids::VECTOR_SECTION_CONVERT,
            tr("panel.vector.section.convert"),
            y,
        );
        if collapsed {
            return y;
        }
        self.action_button(
            ph2d_tool_vector::ids::VECTOR_EXPAND_OUTLINE_STROKE,
            tr("panel.vector.expand.outline_stroke"),
            y,
        )
    }

    /// As quatro linhas do **perfil de largura** + o botão que o CONSOLIDA.
    ///
    /// Os três multiplicadores são MULTIPLICADORES da largura do traço, não medidas: o
    /// artista já escolheu a largura no slider de Width, e o perfil diz o que acontece com
    /// ELA ao longo do caminho. `1 · 1 · 1` é o traço uniforme — e é por isso que o botão
    /// recusa esse caso (aí a operação é o Outline Stroke, logo acima).
    ///
    /// ⚠️ **Desde o ADR-0148 estes quatro sliders AUTORAM** (não são mais parâmetros de um
    /// comando): arrastá-los arma um `VecStrokeProfile` na seleção e a fita aparece na hora,
    /// como os chips de Corner/Side do Offset. O botão materializa.
    pub(crate) fn power_stroke_rows(&mut self, y: f32) -> f32 {
        // Os ids são tipados como `ph2d_a11y::NodeId` porque é o que eles de fato são: cada um
        // vira um nó de AccessKit lá dentro (`slider_row` → `paint_slider_with_chip…`,
        // `action_button` → `paint_button`, que é quem emite). A seção **delega** o a11y aos
        // primitivos canónicos — o gate HR-12 lê esta delegação aqui.
        // **Os quatro trilhos que estão na TELA**, lidos UMA vez: o store quando o artista já
        // tocou, o default do tool quando não. A fileira de perfis e os quatro sliders leem
        // daqui — perguntar duas vezes é como a linha acesa passaria a discordar do knob que
        // está logo abaixo dela.
        let tracks = live_tracks(self.store);
        let mut y = y;
        for (i, (label, slider, chip)) in SLIDERS.iter().enumerate() {
            // O `W Pos` já É a fração de arco (domínio `[0,1]`); os outros três são
            // multiplicadores numa faixa que o `params` remapeia.
            let val = if i == 3 {
                f64::from(tracks[i])
            } else {
                params::slider_to_wprofile(tracks[i])
            };
            y = self.slider_row(
                label.tr(),
                *slider,
                *chip,
                tracks[i],
                val,
                &format!("{val:.2}"),
                y,
            );
        }
        // "Apply Power Stroke", pelo MESMO argumento do "Apply Offset" acima (ADR-0148): desde
        // que os quatro sliders autoram um perfil VIVO, arrastá-los já mostra a fita na tela —
        // quem consolida a curva é ESTE botão. O nome do botão é a promessa, e um "Power Stroke"
        // solto prometeria que nada acontece antes de clicá-lo.
        self.action_button(
            ph2d_tool_vector::ids::VECTOR_EXPAND_POWER_STROKE,
            tr("panel.vector.expand.apply_power_stroke"),
            y,
        )
    }

    /// A fileira do catálogo com a linha `active` acesa. O Expand acende a dos quatro sliders (o
    /// perfil que eles AUTORAM); o painel do traço da barra ② (F16) acende o da FORMA selecionada.
    pub(crate) fn width_preset_row(&mut self, active: Option<usize>, y: f32) -> f32 {
        let opts: Vec<(ph2d_a11y::NodeId, &str, bool)> = ph2d_vec_scene::WIDTH_PRESETS
            .iter()
            .enumerate()
            .take(crate::ids::MAX_WIDTH_PRESETS)
            .map(|(i, p)| {
                (
                    crate::ids::vector_width_preset_id(i),
                    tr(p.key),
                    active == Some(i),
                )
            })
            .collect();
        self.segmented(tr("panel.vector.expand.profile"), &opts, y)
    }
}

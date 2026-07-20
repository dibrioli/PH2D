//! A seção **Expand** do painel Vector — irmã de `paint_sections.rs` pelo teto de 600 LOC.
//!
//! Os três comandos que convertem estilo em GEOMETRIA: o traço vira forma (Outline Stroke), a
//! borda anda (Offset Path), e a largura varia ao longo do caminho (Power Stroke). São irmãos
//! da Boolean logo acima — destrutivos, sobre a seleção, pelo mesmo motor
//! (`ph2d_vec_boolean::expand`).

use super::BodyCtx;
use crate::ids;
use ph2d_i18n::tr;
use ph2d_tool_vector::params;

impl BodyCtx<'_> {
    /// Seção **EXPAND** — Outline Stroke (o traço vira forma) + Offset Path (a borda anda).
    ///
    /// Irmã da Boolean logo acima: os dois são comandos DESTRUTIVOS sobre a seleção, pelo
    /// mesmo motor. O `Join` aqui é o do OFFSET (a quina que ele produz), nunca o do traço.
    pub(crate) fn expand_section(&mut self, y: f32) -> f32 {
        let (mut y, collapsed) = self.section_header(
            ids::VECTOR_SECTION_EXPAND,
            tr("panel.vector.section.expand"),
            y,
        );
        if collapsed {
            return y;
        }
        let track = self
            .store
            .slider(ids::VECTOR_EXPAND_OFFSET)
            .map(|(_, v)| v)
            .unwrap_or_else(|| params::offset_to_slider(params::OFFSET_DEFAULT));
        let d = params::slider_to_offset(track);
        y = self.slider_row(
            "Offset",
            ids::VECTOR_EXPAND_OFFSET,
            ids::VECTOR_EXPAND_OFFSET_NUM,
            track,
            d,
            &format!("{d:.2}"),
            y,
        );
        // **Qual contorno o offset move** — num compound (forma com furos), a borda de fora e
        // os furos são coisas separadas; é o que faz a quina aparecer no furo (não só no
        // externo). Num caminho sem furo os três dão o mesmo (só há o contorno de fora).
        let side = crate::state::expand_side();
        y = self.segmented3(
            "Side",
            [
                (ids::VECTOR_EXPAND_SIDE_OUTER, "Outer", side == 0),
                (ids::VECTOR_EXPAND_SIDE_INNER, "Inner", side == 1),
                (ids::VECTOR_EXPAND_SIDE_BOTH, "Both", side == 2),
            ],
            y,
        );
        let join = crate::state::expand_join();
        y = self.segmented3(
            "Join",
            [
                (ids::VECTOR_EXPAND_JOIN_MITER, "Miter", join == 0),
                (ids::VECTOR_EXPAND_JOIN_ROUND, "Round", join == 1),
                (ids::VECTOR_EXPAND_JOIN_BEVEL, "Bevel", join == 2),
            ],
            y,
        );
        y = self.action_button(ids::VECTOR_EXPAND_OFFSET_PATH, "Offset Path", y);
        y = self.action_button(ids::VECTOR_EXPAND_OUTLINE_STROKE, "Outline Stroke", y);
        self.power_stroke_rows(y)
    }

    /// As quatro linhas do **perfil de largura** + o botão que o assa.
    ///
    /// Os três multiplicadores são MULTIPLICADORES da largura do traço, não medidas: o
    /// artista já escolheu a largura no slider de Width, e o perfil diz o que acontece com
    /// ELA ao longo do caminho. `1 · 1 · 1` é o traço uniforme — e é por isso que o botão
    /// recusa esse caso (aí a operação é o Outline Stroke, logo acima).
    fn power_stroke_rows(&mut self, y: f32) -> f32 {
        let mut y = y;
        // Os ids são tipados como `ph2d_a11y::NodeId` porque é o que eles de fato são: cada um
        // vira um nó de AccessKit lá dentro (`slider_row` → `paint_slider_with_chip…`,
        // `action_button` → `paint_button`, que é quem emite). A seção **delega** o a11y aos
        // primitivos canónicos — o gate HR-12 lê esta delegação aqui.
        let rows: [(&str, ph2d_a11y::NodeId, ph2d_a11y::NodeId, f64); 3] = [
            (
                "W Start",
                ids::VECTOR_EXPAND_W_START,
                ids::VECTOR_EXPAND_W_START_NUM,
                params::WPROFILE_DEFAULT_START,
            ),
            (
                "W Mid",
                ids::VECTOR_EXPAND_W_MID,
                ids::VECTOR_EXPAND_W_MID_NUM,
                params::WPROFILE_DEFAULT_MID,
            ),
            (
                "W End",
                ids::VECTOR_EXPAND_W_END,
                ids::VECTOR_EXPAND_W_END_NUM,
                params::WPROFILE_DEFAULT_END,
            ),
        ];
        for (label, slider, chip, default) in rows {
            let track = self
                .store
                .slider(slider)
                .map(|(_, v)| v)
                .unwrap_or_else(|| params::wprofile_to_slider(default));
            let val = params::slider_to_wprofile(track);
            y = self.slider_row(label, slider, chip, track, val, &format!("{val:.2}"), y);
        }
        let track = self
            .store
            .slider(ids::VECTOR_EXPAND_W_POS)
            .map(|(_, v)| v)
            .unwrap_or(params::WPROFILE_DEFAULT_POS as f32);
        let pos = f64::from(track);
        y = self.slider_row(
            "W Pos",
            ids::VECTOR_EXPAND_W_POS,
            ids::VECTOR_EXPAND_W_POS_NUM,
            track,
            pos,
            &format!("{pos:.2}"),
            y,
        );
        self.action_button(ids::VECTOR_EXPAND_POWER_STROKE, "Power Stroke", y)
    }
}

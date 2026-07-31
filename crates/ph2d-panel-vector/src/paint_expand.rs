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
            .unwrap_or_else(|| params::offset_frac_to_slider(params::OFFSET_DEFAULT_FRAC));
        // O chip mostra PERCENTUAL do tamanho da forma (−100 = morte garantida, +100 =
        // dobrar) — o mundo-d é `fração × vec_expand::offset_scale` na shell. Percentual
        // porque o mapa do store é estático: um rótulo em unidades de mundo mentiria
        // sempre que a seleção mudasse de tamanho.
        let pct = params::slider_to_offset_frac(track) * 100.0; // LITERAL-PX-OK: unit conversion (fraction -> percent readout), not a design measure.
        y = self.slider_row(
            "Offset",
            ids::VECTOR_EXPAND_OFFSET,
            ids::VECTOR_EXPAND_OFFSET_NUM,
            track,
            pct,
            &format!("{pct:.0}"),
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
        // ⚠️ O rótulo é **"Corner"**, não "Join": a seção Stroke, no MESMO painel, tem uma
        // fileira "Join · Miter/Round/Bevel" IDÊNTICA (a quina do traço) — duas fileiras
        // gêmeas para perguntas diferentes é como o clique do artista cai na errada e
        // "não faz nada" (metade do report de 2026-07-20). "Corner" nomeia o que o
        // artista VÊ: a quina do resultado do offset (vocabulário dos Live Corners).
        let join = crate::state::expand_join();
        y = self.segmented3(
            "Corner",
            [
                (ids::VECTOR_EXPAND_JOIN_MITER, "Miter", join == 0),
                (ids::VECTOR_EXPAND_JOIN_ROUND, "Round", join == 1),
                (ids::VECTOR_EXPAND_JOIN_BEVEL, "Bevel", join == 2),
            ],
            y,
        );
        // "Apply Offset", não "Offset Path" (Enio 2026-07-21): os chips de Corner/Side
        // são PREVIEW do offset recém-solto — quem consolida a curva é ESTE botão (ou
        // Convert to Curves, ou qualquer edição seguinte). O nome do botão é a promessa.
        y = self.action_button(ids::VECTOR_EXPAND_OFFSET_PATH, "Apply Offset", y);
        y = self.action_button(ids::VECTOR_EXPAND_OUTLINE_STROKE, "Outline Stroke", y);
        self.power_stroke_rows(y)
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
    fn power_stroke_rows(&mut self, y: f32) -> f32 {
        // Os ids são tipados como `ph2d_a11y::NodeId` porque é o que eles de fato são: cada um
        // vira um nó de AccessKit lá dentro (`slider_row` → `paint_slider_with_chip…`,
        // `action_button` → `paint_button`, que é quem emite). A seção **delega** o a11y aos
        // primitivos canónicos — o gate HR-12 lê esta delegação aqui.
        const SLIDERS: [(&str, ph2d_a11y::NodeId, ph2d_a11y::NodeId); 4] = [
            (
                "W Start",
                ids::VECTOR_EXPAND_W_START,
                ids::VECTOR_EXPAND_W_START_NUM,
            ),
            (
                "W Mid",
                ids::VECTOR_EXPAND_W_MID,
                ids::VECTOR_EXPAND_W_MID_NUM,
            ),
            (
                "W End",
                ids::VECTOR_EXPAND_W_END,
                ids::VECTOR_EXPAND_W_END_NUM,
            ),
            (
                "W Pos",
                ids::VECTOR_EXPAND_W_POS,
                ids::VECTOR_EXPAND_W_POS_NUM,
            ),
        ];
        // **Os quatro trilhos que estão na TELA**, lidos UMA vez: o store quando o artista já
        // tocou, o default do tool quando não. A fileira de perfis e os quatro sliders leem
        // daqui — perguntar duas vezes é como a linha acesa passaria a discordar do knob que
        // está logo abaixo dela.
        let fallback = params::preset_tracks(&params::WPROFILE_DEFAULT);
        let mut tracks = [0.0_f32; 4];
        for (i, (_, slider, _)) in SLIDERS.iter().enumerate() {
            tracks[i] = self.store.slider(*slider).map_or(fallback[i], |(_, v)| v);
        }
        let mut y = self.width_presets(&tracks, y);
        for (i, (label, slider, chip)) in SLIDERS.iter().enumerate() {
            // O `W Pos` já É a fração de arco (domínio `[0,1]`); os outros três são
            // multiplicadores numa faixa que o `params` remapeia.
            let val = if i == 3 {
                f64::from(tracks[i])
            } else {
                params::slider_to_wprofile(tracks[i])
            };
            y = self.slider_row(
                label,
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
        self.action_button(ids::VECTOR_EXPAND_POWER_STROKE, "Apply Power Stroke", y)
    }

    /// **O catálogo de perfis** (W2b) — as formas que se escolhem por NOME, acima dos quatro
    /// sliders que as refinam. É o *Width Profile* do Illustrator: escolhe-se a curva e depois,
    /// se for o caso, mexe-se nos números.
    ///
    /// ⚠️ **A lista vem da TABELA** (`ph2d_stroke_width::PRESETS`), nunca escrita aqui: um perfil
    /// novo é uma linha lá e **zero** mudança neste arquivo — o idioma dos presets de gaiola do
    /// envelope, e o da rack de áudio que se popula de `KINDS`. O `MAX_WIDTH_PRESETS` é só o teto
    /// que o `populate` registra de uma vez.
    ///
    /// ⚠️ **A linha ACESA é DERIVADA, nunca guardada** — não há campo "preset corrente" em lugar
    /// nenhum, e é isso que mantém a fileira honesta depois de o artista arrastar um slider ou uma
    /// alça do Width Tool: aí nenhuma acende, que é a verdade (a forma não é mais nenhuma delas).
    /// A comparação é em **TRILHO** e exata; o porquê de não poder ser em multiplicador está no
    /// doc de [`params::preset_tracks`].
    fn width_presets(&mut self, tracks: &[f32; 4], y: f32) -> f32 {
        let active = params::active_preset(tracks);
        let opts: Vec<(ph2d_a11y::NodeId, &str, bool)> = ph2d_vec_scene::WIDTH_PRESETS
            .iter()
            .enumerate()
            .take(ids::MAX_WIDTH_PRESETS)
            .map(|(i, p)| (ids::vector_width_preset_id(i), tr(p.key), active == Some(i)))
            .collect();
        self.segmented("Profile", &opts, y)
    }
}

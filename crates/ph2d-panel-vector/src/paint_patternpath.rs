//! A seção **PATTERN ON PATH** do painel Vector — módulo irmão do [`super`] (teto de 600 LOC).
//!
//! Um motivo se repete, **rígido**, ao longo de uma curva (plano 23): cada cópia translada e gira
//! para a tangente, e não deforma. É outra feature que o Envelope (mapa não-afim) e o Repeater
//! (grade/radial, sem guia). As cópias são DESENHO derivado (`pattern_live`) — a curva do motivo,
//! que o Node edita, nunca é tocada.
//!
//! **Esta seção é a única porta do produto para o vínculo.** Sem ela o motor existiria, gateado e
//! smokado, e a feature não existiria para o artista.
//!
//! # As duas caras da seção, e por que ela SÓ aparece quando há o que fazer
//!
//! Ao contrário do texto (que tem uma forma própria em foco), um motivo é um caminho qualquer —
//! então a seção não pode aparecer para toda forma selecionada sem virar ruído. Aparece só quando
//! **há um vínculo** (mostra os controles + Detach) ou quando a seleção **permite prender** (dois
//! caminhos → o botão). Fora disso, o cabeçalho nem sobe — é a lei do `Join Selected Bodies`.

use super::*;

impl BodyCtx<'_> {
    /// Seção **PATTERN ON PATH**.
    pub(crate) fn patternpath_section(&mut self, y: f32) -> f32 {
        // Só quando há o que dizer: um vínculo vivo, ou a seleção que permite criar um — por DOIS
        // caminhos (a auto-ligação) ou por UM (o Picker, a fonte à espera do clique do guia).
        if !state::pp_linked() && !state::pp_can_link() && !state::pp_can_pick() {
            return y;
        }
        let (mut y, collapsed) = self.section_header(
            ids::VECTOR_SECTION_PATTERNPATH,
            tr("panel.vector.section.patternpath"),
            y,
        );
        if collapsed {
            return y;
        }
        if !state::pp_linked() {
            // Duas portas para prender, por seleção. Com DOIS caminhos, a auto-ligação (o guia é o
            // de maior extensão). Com UM, o **Picker**: apertar arma e o clique seguinte escolhe o
            // guia — a porta EXPLÍCITA, sem a adivinhação por bbox. As duas são exclusivas (uma quer
            // dois selecionados, a outra um), então nunca aparecem juntas. Um botão que recusa é
            // pior que um que falta: cada porta é oferecida só na seleção em que funciona.
            if state::pp_can_link() {
                return self.action_button(ids::VECTOR_PATTERNPATH_LINK, "Pattern on Path", y);
            }
            return self.action_button(ids::VECTOR_PATTERNPATH_PICK, "Pick Path", y);
        }
        // Spacing — o controle-assinatura (quão densas as cópias). Track `0..1` → valor `0.25..4.0`
        // (o mapa vive na fronteira, `event::track_slider_event`, e é o mesmo do Bend bipolar).
        let spacing = self
            .store
            .number_value(ids::VECTOR_PATTERNPATH_SPACING_NUM)
            .unwrap_or_else(state::pp_spacing);
        let sp_track = self
            .store
            .slider(ids::VECTOR_PATTERNPATH_SPACING)
            .map_or_else(|| spacing_track(state::pp_spacing()), |(_, v)| v);
        y = self.slider_row(
            "Spacing",
            ids::VECTOR_PATTERNPATH_SPACING,
            ids::VECTOR_PATTERNPATH_SPACING_NUM,
            sp_track,
            spacing,
            &format!("{spacing:.2}"),
            y,
        );
        // Start — onde a tilagem começa, em FRAÇÃO do comprimento (track = valor, como o Offset do
        // texto: `0.50` é meio caminho em qualquer curva).
        let start_track = self
            .store
            .slider(ids::VECTOR_PATTERNPATH_START)
            .map_or_else(|| state::pp_start() as f32, |(_, v)| v);
        let start = self
            .store
            .number_value(ids::VECTOR_PATTERNPATH_START_NUM)
            .unwrap_or_else(state::pp_start);
        y = self.slider_row(
            "Start",
            ids::VECTOR_PATTERNPATH_START,
            ids::VECTOR_PATTERNPATH_START_NUM,
            start_track,
            start,
            &format!("{start:.2}"),
            y,
        );
        // End — o fim do trecho, FRAÇÃO (track == valor). `[Start, End]` é onde as cópias caem.
        let end_track = self
            .store
            .slider(ids::VECTOR_PATTERNPATH_END)
            .map_or_else(|| state::pp_end() as f32, |(_, v)| v);
        let end = self
            .store
            .number_value(ids::VECTOR_PATTERNPATH_END_NUM)
            .unwrap_or_else(state::pp_end);
        y = self.slider_row(
            "End",
            ids::VECTOR_PATTERNPATH_END,
            ids::VECTOR_PATTERNPATH_END_NUM,
            end_track,
            end,
            &format!("{end:.2}"),
            y,
        );
        // Slide — o CENTRO do trecho: arrastar desliza Start e End JUNTOS (o comprimento se preserva).
        // ⚠️ O valor é DERIVADO de `(start+end)/2` a cada frame, NÃO lido do store: ele muda quando o
        // artista mexe em Start/End ou nas alças, e o store só teria a última posição de arrasto de
        // Slide. O drain re-centra a janela a partir deste número (a porta é o `render_loop`).
        let slide = (state::pp_start() + state::pp_end()) * 0.5;
        y = self.slider_row(
            "Slide",
            ids::VECTOR_PATTERNPATH_SLIDE,
            ids::VECTOR_PATTERNPATH_SLIDE_NUM,
            slide as f32,
            slide,
            &format!("{slide:.2}"),
            y,
        );
        // Offset — desvio perpendicular, BIPOLAR (unidades de mundo). O track `0..1` mapeia
        // `−OFFSET_MAX..OFFSET_MAX` (o mesmo mapa bipolar do Bend), `0.5` = sobre a curva.
        let off = self
            .store
            .number_value(ids::VECTOR_PATTERNPATH_OFFSET_NUM)
            .unwrap_or_else(state::pp_offset);
        let off_track = self
            .store
            .slider(ids::VECTOR_PATTERNPATH_OFFSET)
            .map_or_else(|| offset_track(state::pp_offset()), |(_, v)| v);
        y = self.slider_row(
            "Offset",
            ids::VECTOR_PATTERNPATH_OFFSET,
            ids::VECTOR_PATTERNPATH_OFFSET_NUM,
            off_track,
            off,
            &format!("{off:.2}"),
            y,
        );
        // Rotation — a ATITUDE do motivo em cima da guia, BIPOLAR em GRAUS. Fica ao lado do Offset
        // porque os dois dizem como a cópia se põe em relação à curva (um desloca, o outro vira);
        // o track `0..1` mapeia `−ROTATION_MAX..ROTATION_MAX` e `0.5` é deitado ao longo da curva.
        let rot = self
            .store
            .number_value(ids::VECTOR_PATTERNPATH_ROTATION_NUM)
            .unwrap_or_else(state::pp_rotation);
        let rot_track = self
            .store
            .slider(ids::VECTOR_PATTERNPATH_ROTATION)
            .map_or_else(|| rotation_track(state::pp_rotation()), |(_, v)| v);
        y = self.slider_row(
            "Rotation",
            ids::VECTOR_PATTERNPATH_ROTATION,
            ids::VECTOR_PATTERNPATH_ROTATION_NUM,
            rot_track,
            rot,
            &format!("{rot:.0}\u{00b0}"),
            y,
        );
        // O lado é um par exclusivo (deste / do outro), não um checkbox — a mesma razão do texto.
        let flip = state::pp_flip();
        let sides: [(ph2d_a11y::NodeId, &str, bool); 2] = [
            (ids::VECTOR_PATTERNPATH_FLIP_OFF, "This side", !flip),
            (ids::VECTOR_PATTERNPATH_FLIP, "Other side", flip),
        ];
        y = self.segmented("Side", &sides, y);
        self.action_button(ids::VECTOR_PATTERNPATH_DETACH, "Detach from Path", y)
    }
}

/// O track `0..1` do slider a partir do Spacing `0.25..4.0` — o inverso do mapa da fronteira.
fn spacing_track(spacing: f64) -> f32 {
    (((spacing - crate::SPACING_MIN) / (crate::SPACING_MAX - crate::SPACING_MIN)) as f32)
        .clamp(0.0, 1.0)
}

/// O track `0..1` do slider a partir do Offset bipolar `−OFFSET_MAX..OFFSET_MAX` — `0.5` = zero.
fn offset_track(offset: f64) -> f32 {
    (((offset / crate::OFFSET_MAX) * 0.5 + 0.5) as f32).clamp(0.0, 1.0)
}

/// O track `0..1` do slider a partir da Rotation bipolar `−ROTATION_MAX..ROTATION_MAX` — `0.5` = 0°.
fn rotation_track(deg: f64) -> f32 {
    (((deg / crate::ROTATION_MAX) * 0.5 + 0.5) as f32).clamp(0.0, 1.0)
}

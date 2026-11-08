//! **O ◆ marcador na fila** do editor ([ficha](../../../../docs/GameEngineFuncional/componentes/02_animated_sprite.md),
//! decisão 4): pôr e tirar no quadro escolhido, e o nome do sinal. O modelo é `ph2d_ecs::SpriteAnimMarkers`, pelas
//! edições de sempre. O marcador anda com o desenho quando o quadro muda de lugar.

use super::doc::{AnimDoc, Field, reorder};
use ph2d_editor_core::AnimFieldEdit;

impl AnimDoc {
    fn mark_steps(&self) -> Vec<u32> {
        self.snap
            .info
            .as_ref()
            .and_then(|i| i.rows.get(self.selected))
            .map(|r| r.markers.iter().map(|m| m.step).collect())
            .unwrap_or_default()
    }

    /// O marcador do quadro `frame` da animação aberta. Um passado do fim mostra-se no último, para se poder tirar.
    #[must_use]
    pub fn marker_at(&self, frame: usize) -> Option<usize> {
        let last = u32::try_from(self.authored_frames().len().checked_sub(1)?).ok()?;
        let f = u32::try_from(frame).ok()?;
        self.mark_steps().iter().position(|s| (*s).min(last) == f)
    }

    /// O passo gravado do marcador `frame`, se ele já passou do fim e por isso não grita.
    #[must_use]
    pub fn marker_past_end(&self, frame: usize) -> Option<u32> {
        let n = u32::try_from(self.authored_frames().len()).ok()?;
        let step = *self.mark_steps().get(self.marker_at(frame)?)?;
        (step >= n).then_some(step)
    }

    /// O nome do sinal do marcador `frame`.
    #[must_use]
    pub fn marker_signal(&self, frame: usize) -> String {
        self.snap
            .info
            .as_ref()
            .and_then(|i| i.rows.get(self.selected))
            .zip(self.marker_at(frame))
            .and_then(|(r, i)| r.markers.get(i))
            .map(|m| m.signal.clone())
            .unwrap_or_default()
    }

    /// O quadro escolhido, se ainda existe.
    #[must_use]
    pub fn marked_frame(&self) -> Option<usize> {
        self.frame_selected
            .filter(|f| *f < self.authored_frames().len())
    }

    /// Põe ou tira o marcador do quadro escolhido; ao pôr, o campo do nome abre-se.
    pub(super) fn toggle_marker(&mut self) {
        let (Some(row), Some(frame)) = (self.row(), self.marked_frame()) else {
            return;
        };
        if let Some(i) = self.marker_at(frame) {
            self.editing = None;
            self.pending.push(AnimFieldEdit::MarkerRemove(row, i as u8));
            return;
        }
        let i = self.mark_steps().len() as u8;
        self.pending.push(AnimFieldEdit::MarkerAdd(row));
        self.pending
            .push(AnimFieldEdit::MarkerStep(row, i, frame as u32));
        self.editing = Some((Field::Signal(frame), String::new()));
    }

    /// A edição do nome do sinal do quadro `frame`. O marcador acabado de pôr ainda não está no snapshot: é o último.
    pub(super) fn signal_edit(&self, frame: usize, text: &str) -> Option<AnimFieldEdit> {
        let i = self
            .marker_at(frame)
            .unwrap_or_else(|| self.mark_steps().len());
        Some(AnimFieldEdit::MarkerSignal(
            self.row()?,
            u8::try_from(i).ok()?,
            text.to_owned(),
        ))
    }

    /// Os marcadores seguem os quadros: `map` diz o lugar novo de cada passo (`None` = o quadro saiu, e ele com o quadro).
    fn markers_follow(&mut self, map: impl Fn(u32) -> Option<u32>) {
        let Some(row) = self.row() else {
            return;
        };
        let steps = self.mark_steps();
        for (i, step) in steps.iter().enumerate() {
            if let Some(to) = map(*step).filter(|to| to != step) {
                self.pending
                    .push(AnimFieldEdit::MarkerStep(row, i as u8, to));
            }
        }
        // Do fim para o começo: tirar um desloca os índices dos de depois.
        for (i, step) in steps.iter().enumerate().rev() {
            if map(*step).is_none() {
                self.pending.push(AnimFieldEdit::MarkerRemove(row, i as u8));
            }
        }
    }

    /// O quadro `from` foi para o lugar `to`.
    pub(super) fn markers_follow_move(&mut self, from: usize, to: usize) {
        let n = self.authored_frames().len() as u32;
        let order = reorder(&(0..n).collect::<Vec<u32>>(), from, to);
        self.markers_follow(|s| {
            let at = order.iter().position(|o| *o == s);
            Some(at.map_or(s, |p| p as u32))
        });
    }

    /// Entraram `n` quadros no intervalo `at`: os marcadores dali para a frente andam `n`.
    pub(super) fn markers_follow_insert(&mut self, at: usize, n: usize) {
        let (at, n) = (at as u32, n as u32);
        self.markers_follow(|s| Some(if s >= at { s + n } else { s }));
    }

    /// O quadro `gone` saiu.
    pub(super) fn markers_follow_delete(&mut self, gone: usize) {
        let g = gone as u32;
        self.markers_follow(|s| match s.cmp(&g) {
            std::cmp::Ordering::Less => Some(s),
            std::cmp::Ordering::Equal => None,
            std::cmp::Ordering::Greater => Some(s - 1),
        });
    }

    /// **Split by rows**: o gesto lê a folha pintada; quem junta as animações é a porta do Inspector.
    pub(super) fn split_by_rows(&mut self) {
        let Some(s) = &self.snap.sheet else {
            return;
        };
        let rows =
            super::split::drawn_by_row(&s.rgba, s.w, s.h, self.snap.hframes, self.snap.vframes);
        self.pending.push(AnimFieldEdit::SplitByRows(rows));
    }
}

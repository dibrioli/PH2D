//! O olhar de cada controlo da A por estado (repouso · pairado · premido), D32: o fundo muda NO QUADRO, sem fade.
//! A pintura lê o alvo quente nos rects da pintura ANTERIOR (os desta ainda se registam): um controlo nunca se
//! pinta frio só por o vizinho que o cobre ainda não se ter registado.

use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::paint::{fill_rounded_rect, paint_icon};
use ph2d_editor_core::widget::card_ink::CardInk;
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, StrokeToken};
use ph2d_vector::VectorScene;

use crate::census::ControlLook;
use crate::doc::InputMapDoc;
use crate::interact::{Control, ControlRole};

/// O alvo quadrado mínimo de um botão de ícone (×, «+», lixo).
pub(super) const ICON_HIT: f32 = m::HEAD_ICON_BOX;

/// Em que estado um controlo se pinta.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum State {
    Rest,
    Hover,
    Press,
}

impl InputMapDoc {
    /// Começo de uma passagem de pintura: o alvo quente lido nos rects da anterior.
    pub(crate) fn begin_hot(&mut self) {
        self.ui.paint_hot = self.hot();
    }

    /// O alvo quente mudou com os rects desta pintura (um controlo novo por cima): a pintura repete-se.
    pub(crate) fn hot_moved(&self) -> bool {
        self.ui.paint_hot != self.hot()
    }

    /// `id` pinta-se pairado neste quadro.
    #[must_use]
    pub fn lit(&self, id: NodeId) -> bool {
        self.ui.paint_hot == Some(id)
    }

    /// `id` pinta-se premido neste quadro (capturado e com o ponteiro dentro).
    #[must_use]
    pub fn sunk(&self, id: NodeId) -> bool {
        self.ui.pressed == Some(id) && self.lit(id)
    }

    pub(super) fn state_of(&self, id: NodeId) -> State {
        if self.sunk(id) {
            State::Press
        } else if self.lit(id) {
            State::Hover
        } else {
            State::Rest
        }
    }

    pub(super) fn record_look(&mut self, look: ControlLook) {
        self.census.looks.push(look);
    }
}

/// Um botão de ícone da linha (×, «+», lixo).
pub(super) struct IconButton {
    pub id: NodeId,
    /// O alvo (≥ [`ICON_HIT`] de lado).
    pub hit: Rect,
    /// A face redonda que o estado enche.
    pub face: Rect,
    pub icon: IconId,
    pub tone: Tone,
    pub label: String,
    pub shown: bool,
}

/// Como a face de um botão de ícone se enche por estado.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Tone {
    /// Sem face em repouso; pairado enche-se deste tom; premido, do acento suave.
    Plain(ColorToken),
    /// Um emblema redondo que apaga (o × do chip, dono 10/11: «mais destaque»): em repouso o neutro forte com o
    /// ícone claro; pairado o perigo com a tinta de quem se enche (a do botão Danger); premido o perigo premido, mais
    /// fundo que o pairado em todo tema (o `danger-soft` clareava no tema claro).
    Destructive,
}

impl Tone {
    fn look(self, state: State) -> (Option<ColorToken>, ColorToken) {
        match (self, state) {
            (Self::Plain(_), State::Rest) => (None, ColorToken::Text2),
            (Self::Plain(f), State::Hover) => (Some(f), ColorToken::Text1),
            (Self::Plain(_), State::Press) => (Some(ColorToken::AccentSoft), ColorToken::Text1),
            (Self::Destructive, State::Rest) => (Some(ColorToken::BorderStrong), ColorToken::Text1),
            (Self::Destructive, State::Hover) => (Some(ColorToken::Danger), ColorToken::AccentFg),
            (Self::Destructive, State::Press) => {
                (Some(ColorToken::DangerPress), ColorToken::AccentFg)
            }
        }
    }
}

/// Pinta um botão de ícone: o alvo regista-se sempre — só se chega a ele com o ponteiro na linha, que o mostra, ou
/// pelo Tab, que também —; com `shown`, a face pinta-se pelo estado e o ícone acende fora do repouso.
pub(super) fn icon_button(
    doc: &mut InputMapDoc,
    scene: &mut VectorScene,
    ink: CardInk,
    b: IconButton,
) {
    let IconButton {
        id,
        hit,
        face,
        icon,
        tone,
        label,
        shown,
    } = b;
    doc.control(Control::new(id, hit, ControlRole::Button, label));
    if !shown {
        return;
    }
    let (fill, tint) = tone.look(doc.state_of(id));
    if let Some(f) = fill {
        fill_rounded_rect(scene, face, face.w.min(face.h) * 0.5, ink.color(f));
    }
    if tone == Tone::Destructive {
        doc.census.badges.push((id, face));
    }
    let g = m::KEY_MARK_BOX;
    let gl = Rect::new(
        face.x + (face.w - g) * 0.5,
        face.y + (face.h - g) * 0.5,
        g,
        g,
    );
    paint_icon(scene, icon, gl, ink.color(tint), StrokeToken::Default.px());
    doc.record_look(ControlLook {
        id,
        rect: hit,
        fill,
        edge: None,
        ink: tint,
        marker: false,
    });
}

/// A face redonda de um botão de ícone, centrada no alvo, do lado `side`.
pub(super) fn face_in(hit: Rect, side: f32) -> Rect {
    Rect::new(
        hit.x + (hit.w - side) * 0.5,
        hit.y + (hit.h - side) * 0.5,
        side,
        side,
    )
}

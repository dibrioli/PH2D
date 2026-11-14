//! O estado do **cartão da chave** da timeline no [`WidgetStore`]: a pega do arrasto pelo título, a
//! régua dos eases que os botões desenham e o editor da curva **Custom** (pontos e alças).
//!
//! Quem sabe o documento é o painel da timeline: ele publica a régua e a Bézier da chave
//! ([`WidgetStore::set_key_card_curve`]) e drena o arrasto ([`WidgetStore::take_key_card_bez_edit`]).

use super::*;
use crate::interaction::{EaseSampler, TimelineBezierEdit, TimelineInterpScope};
use crate::zones::Rect;
use ph2d_anim::EaseSpline;

/// A área do editor da curva tal como foi pintada: o rectângulo e a faixa de `y` que ele mostra.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BezArea {
    pub rect: Rect,
    pub y_min: f32,
    pub y_max: f32,
}

impl BezArea {
    /// O ponto `(u, y)` do quadrado unitário sob `(px, py)`, preso à área.
    #[must_use]
    pub fn unidade(&self, px: f32, py: f32) -> (f32, f32) {
        let r = self.rect;
        let u = ((px - r.x) / r.w).clamp(0.0, 1.0);
        let t = ((r.y + r.h - py) / r.h).clamp(0.0, 1.0);
        (u, self.y_min + t * (self.y_max - self.y_min))
    }

    /// O ponto de tela de `(u, y)`.
    #[must_use]
    pub fn tela(&self, u: f32, y: f32) -> (f32, f32) {
        let r = self.rect;
        (
            r.x + u * r.w,
            r.y + r.h - (y - self.y_min) / (self.y_max - self.y_min) * r.h,
        )
    }
}

/// O que o editor da curva segura: um ponto (nó do meio) ou uma alça de um nó.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BezAlvo {
    /// O ponto do nó `i`.
    Ponto(usize),
    /// A alça `lado` do nó `i`.
    Alca(usize, ph2d_anim::SplineSide),
}

#[derive(Debug, Default)]
pub(in crate::interaction) struct KeyCardState {
    grab: Option<(f32, f32)>,
    sampler: Option<EaseSampler>,
    curva: Option<EaseSpline>,
    area: core::cell::Cell<Option<BezArea>>,
    alvo: Option<BezAlvo>,
    edit: Option<TimelineBezierEdit>,
    pendente: bool,
}

impl WidgetStore {
    /// A régua dos eases do cartão da chave, publicada por quem os conhece (a timeline): os botões
    /// desenham a curva que a chave vai ganhar, a mesma conta que o documento usa.
    pub fn set_ease_sampler(&mut self, sampler: EaseSampler) {
        self.key_card.sampler = Some(sampler);
    }

    /// A régua publicada por [`Self::set_ease_sampler`] (`None` = ninguém publicou: sem curva).
    #[must_use]
    pub fn ease_sampler(&self) -> Option<EaseSampler> {
        self.key_card.sampler
    }

    /// A pega do arrasto do cartão da chave: `(dx, dy)` do cursor ao canto do cartão.
    #[must_use]
    pub fn key_card_grab(&self) -> Option<(f32, f32)> {
        self.key_card.grab
    }

    /// Arma (`Some`) ou solta (`None`) o arrasto do cartão da chave.
    pub fn set_key_card_grab(&mut self, grab: Option<(f32, f32)>) {
        self.key_card.grab = grab;
    }

    /// A curva Custom da chave (normalizada), publicada pela timeline; `None` = a chave não é
    /// Custom ou não tem forma normalizável.
    pub fn set_key_card_curve(&mut self, curva: Option<EaseSpline>) {
        self.key_card.curva = curva;
    }

    /// A curva que o editor desenha: a da edição em curso, senão a publicada.
    #[must_use]
    pub fn key_card_curve(&self) -> Option<EaseSpline> {
        self.key_card.edit.map(|e| e.curve).or(self.key_card.curva)
    }

    /// O pintor guarda onde pintou o editor (o arrasto converte o cursor por ela).
    pub fn set_key_card_bez_area(&self, area: Option<BezArea>) {
        self.key_card.area.set(area);
    }

    /// A área pintada do editor da curva.
    #[must_use]
    pub fn key_card_bez_area(&self) -> Option<BezArea> {
        self.key_card.area.get()
    }

    /// O que o arrasto segura agora.
    #[must_use]
    pub fn key_card_bez_alvo(&self) -> Option<BezAlvo> {
        self.key_card.alvo
    }

    /// Começa a arrastar `alvo` da curva `curve` (já com o ponto novo, se foi criado agora).
    pub fn begin_key_card_bez(
        &mut self,
        scope: TimelineInterpScope,
        alvo: BezAlvo,
        curve: EaseSpline,
    ) {
        self.key_card.alvo = Some(alvo);
        self.key_card.edit = Some(TimelineBezierEdit {
            scope,
            curve,
            begin: true,
            end: false,
        });
        self.key_card.pendente = true;
    }

    /// Uma edição de um toque (apagar um ponto): abre e fecha o passo de undo no mesmo lote.
    pub fn apply_key_card_bez(&mut self, scope: TimelineInterpScope, curve: EaseSpline) {
        self.key_card.alvo = None;
        self.key_card.edit = Some(TimelineBezierEdit {
            scope,
            curve,
            begin: true,
            end: true,
        });
        self.key_card.pendente = true;
    }

    /// O que está seguro vai a `(u, y)`.
    pub fn move_key_card_bez(&mut self, (u, y): (f32, f32)) {
        let (Some(a), Some(e)) = (self.key_card.alvo, self.key_card.edit.as_mut()) else {
            return;
        };
        match a {
            BezAlvo::Ponto(i) => e.curve.move_point(i, u, y),
            BezAlvo::Alca(i, lado) => e.curve.move_handle(i, lado, u, y),
        }
        self.key_card.pendente = true;
    }

    /// Larga o que estava seguro (no-op sem arrasto).
    pub fn end_key_card_bez(&mut self) {
        if self.key_card.alvo.take().is_none() {
            return;
        }
        if let Some(e) = self.key_card.edit.as_mut() {
            e.end = true;
            self.key_card.pendente = true;
        }
    }

    /// O lote da edição desde a última drenagem (o painel da timeline aplica-o). Depois do lote
    /// que acaba, o editor volta a desenhar o que a timeline publica.
    pub fn take_key_card_bez_edit(&mut self) -> Option<TimelineBezierEdit> {
        if !std::mem::take(&mut self.key_card.pendente) {
            return None;
        }
        let out = self.key_card.edit;
        match self.key_card.edit.as_mut() {
            Some(e) if e.end => self.key_card.edit = None,
            Some(e) => e.begin = false,
            None => {}
        }
        out
    }
}

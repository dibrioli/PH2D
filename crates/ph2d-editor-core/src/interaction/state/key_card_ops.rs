//! O estado do **cartão da chave** da timeline no [`WidgetStore`]: a pega do arrasto pelo título, a
//! régua dos eases que os botões desenham e o editor da curva **Custom** (as duas alças).
//!
//! Quem sabe o documento é o painel da timeline: ele publica a régua e a Bézier da chave
//! ([`WidgetStore::set_key_card_bezier`]) e drena o arrasto ([`WidgetStore::take_key_card_bez_edit`]).

use super::*;
use crate::interaction::{EaseSampler, TimelineBezierEdit, TimelineInterpScope};
use crate::zones::Rect;

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

#[derive(Debug, Default)]
pub(in crate::interaction) struct KeyCardState {
    grab: Option<(f32, f32)>,
    sampler: Option<EaseSampler>,
    bezier: Option<[f32; 4]>,
    area: core::cell::Cell<Option<BezArea>>,
    alca: Option<usize>,
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

    /// A Bézier `[x1, y1, x2, y2]` da chave (normalizada), publicada pela timeline; `None` = a
    /// chave não é Custom ou não tem forma normalizável.
    pub fn set_key_card_bezier(&mut self, h: Option<[f32; 4]>) {
        self.key_card.bezier = h;
    }

    /// A Bézier que o editor desenha: a do arrasto em curso, senão a publicada.
    #[must_use]
    pub fn key_card_bezier(&self) -> Option<[f32; 4]> {
        self.key_card
            .edit
            .map(|e| e.handles)
            .or(self.key_card.bezier)
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

    /// A alça a ser arrastada (`0` = a de saída, `1` = a de chegada).
    #[must_use]
    pub fn key_card_bez_alca(&self) -> Option<usize> {
        self.key_card.alca
    }

    /// Começa o arrasto da alça `alca` a partir de `handles`, para as chaves de `scope`.
    pub fn begin_key_card_bez(
        &mut self,
        scope: TimelineInterpScope,
        alca: usize,
        handles: [f32; 4],
    ) {
        self.key_card.alca = Some(alca);
        self.key_card.edit = Some(TimelineBezierEdit {
            scope,
            handles,
            begin: true,
            end: false,
        });
        self.key_card.pendente = true;
    }

    /// A alça arrastada vai a `(u, y)`.
    pub fn move_key_card_bez(&mut self, (u, y): (f32, f32)) {
        let (Some(a), Some(e)) = (self.key_card.alca, self.key_card.edit.as_mut()) else {
            return;
        };
        e.handles[a * 2] = u;
        e.handles[a * 2 + 1] = y;
        self.key_card.pendente = true;
    }

    /// Larga a alça (no-op sem arrasto).
    pub fn end_key_card_bez(&mut self) {
        if self.key_card.alca.take().is_none() {
            return;
        }
        if let Some(e) = self.key_card.edit.as_mut() {
            e.end = true;
            self.key_card.pendente = true;
        }
    }

    /// O lote do arrasto desde a última drenagem (o painel da timeline aplica-o). Depois do lote
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

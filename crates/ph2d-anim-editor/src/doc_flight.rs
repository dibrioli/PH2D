//! ⭐ **O voo da inserção** (dono 08/11): as células escolhidas na folha viram cartões que descem até a marca de
//! inserção; os quadros já postos abrem espaço para eles, e a tira rola até a marca se ela está fora da vista. O
//! progresso é uma mola do app (`Role::Travel`): com «movimento reduzido» não há mola, e o voo acaba no quadro seguinte.

use super::{AnimDoc, Rect};
use ph2d_editor_core::AnimFrameRef;
use ph2d_spring::{Spring, SpringState};

/// Um voo em curso.
#[derive(Clone, Debug)]
pub struct Flight {
    /// A lista que a animação passa a ter (a faixa mostra-a já, com o vão a abrir).
    pub frames: Vec<AnimFrameRef>,
    /// Onde os cartões entram, e quantos são.
    pub at: usize,
    pub n: usize,
    /// De onde cada cartão parte: a célula dele na folha.
    pub from: Vec<Rect>,
    pub spring: SpringState,
    /// A rolagem da tira quando o voo partiu (a tira anda dela até mostrar a marca).
    pub start_scroll: f32,
    /// A disposição já correu com o voo ASSENTE (`x = 1`): só então ele acaba, com a rolagem no destino.
    pub laid_out: bool,
    /// A animação já tem a lista nova (o mundo aplicou a edição): a faixa passa a ler a autorada.
    pub seen: bool,
}

impl Flight {
    /// O progresso do percurso: `0` na folha, `1` no lugar (pode passar um pouco, com a mola expressiva).
    #[must_use]
    pub fn t(&self) -> f32 {
        self.spring.x as f32
    }

    /// Quanto o vão já abriu, `0..=1`.
    #[must_use]
    pub fn open(&self) -> f32 {
        self.t().clamp(0.0, 1.0)
    }
}

/// A posição de um cartão a meio do voo, de `a` (a célula) a `b` (o lugar na tira).
#[must_use]
pub fn between(a: Rect, b: Rect, t: f32) -> Rect {
    let l = |p: f32, q: f32| p + (q - p) * t;
    Rect::new(l(a.x, b.x), l(a.y, b.y), l(a.w, b.w), l(a.h, b.h))
}

impl AnimDoc {
    /// Parte um voo: `frames` é a lista nova, com `added` metidas em `at`.
    pub(crate) fn launch(&mut self, frames: Vec<AnimFrameRef>, at: usize, added: &[AnimFrameRef]) {
        let from = added.iter().map(|c| self.cell_rect(c.cell)).collect();
        self.flight = Some(Flight {
            frames,
            at,
            n: added.len(),
            from,
            spring: SpringState::at_rest(),
            start_scroll: self.strip_scroll,
            laid_out: false,
            seen: false,
        });
    }

    /// Anda o voo `dt` segundos pela lei `law` (`None` = instantâneo).
    pub fn tick(&mut self, dt: f64, law: Option<Spring>) {
        let Some(f) = self.flight.as_mut() else {
            return;
        };
        if f.laid_out {
            self.flight = None;
            return;
        }
        if law.is_none_or(|s| f.spring.advance(dt, s)) {
            f.spring = SpringState { x: 1.0, v: 0.0 };
        }
    }

    /// Antes de cada disposição: a lista nova chegou ao mundo? E se outra edição a mudou depois (undo, apagar a
    /// folha), o voo cai — o cartão não pode tapar um quadro que já é outro.
    pub(crate) fn follow_flight(&mut self) {
        let authored = self.authored_frames();
        let Some(f) = self.flight.as_mut() else {
            return;
        };
        if authored == f.frames {
            f.seen = true;
        } else if f.seen {
            self.flight = None;
        }
    }

    /// O quadro `i` da faixa é um dos que ainda voam (a faixa não o pinta: o cartão é ele).
    #[must_use]
    pub fn flying(&self, i: usize) -> bool {
        self.flight
            .as_ref()
            .is_some_and(|f| i >= f.at && i < f.at + f.n)
    }
}

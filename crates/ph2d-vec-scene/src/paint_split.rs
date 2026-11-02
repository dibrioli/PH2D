//! ⭐⭐ **PARTIR a pilha em objectos, na ordem em que ela desenha** (dono 02/11: *«Expand e suas funções
//! não funcionam corretamente com nosso novo modo de camadas stroke/fill»*).
//!
//! Outline Stroke, Apply Power Stroke e o alinhamento vivo (Inner/Outer) trocam traços por FORMAS. Com a
//! pilha (F45) um traço pode estar por baixo do miolo, numa camada, deslocado, escondido ou desligado —
//! e os três liam só `stroke`. Esta é a porta única: percorre [`VecPath::draw_order`], cada traço que
//! se vê pergunta ao conversor, e o resto da pilha fica em pedaços de forma viva entre as faixas.

use super::{BaseOrder, DrawnPaint, PaintEntry, PaintRef, StackItem};
use crate::{Opacity, Paint, StrokeSpec, VecPath};

use ph2d_blend_mode::BlendMode;

/// A resposta do conversor para UM traço que se vê.
pub enum Split {
    /// Fica como traço vivo, no pedaço de forma em que cai.
    Keep,
    /// Vira estas geometrias (a tinta é posta aqui: cor do traço, opacidade e mistura da camada).
    /// Vazia = o traço não desenhava nada (um Offset que o aniquilou) e sai.
    Into(Vec<VecPath>),
    /// O motor não respondeu — o gesto inteiro desiste e a arte fica como estava.
    Fail,
}

impl VecPath {
    /// **O «look» de `src`** (tudo o que não é geometria nem efeito de geometria): tintas de base, as
    /// guardadas pelo olho, a pilha e a ordem dela, a opacidade e a mistura do objecto.
    ///
    /// ⛔ É a porta de quem refaz a geometria de uma forma (offset, booleana, faca, Shape Builder):
    /// copiar só `fill`/`stroke` deixava as camadas para trás.
    pub fn carry_look_from(&mut self, src: &VecPath) {
        self.fill.clone_from(&src.fill);
        self.stroke.clone_from(&src.stroke);
        self.fill_hidden.clone_from(&src.fill_hidden);
        self.stroke_hidden.clone_from(&src.stroke_hidden);
        self.paints.clone_from(&src.paints);
        self.base_order = src.base_order;
        self.opacity = src.opacity;
        self.blend = src.blend;
    }

    /// A geometria desta forma com **só o traço `s`** — o operando que os motores de traço recebem.
    #[must_use]
    pub fn stroke_alone(&self, s: &StrokeSpec) -> VecPath {
        let mut p = self.clone();
        p.carry_look_from(&VecPath {
            stroke: Some(s.clone()),
            ..VecPath::default()
        });
        p
    }

    /// ⭐⭐ **Parte a pilha, de baixo para cima.** `convert` é chamado para cada traço que se VÊ (base ou
    /// camada) com a linha, a camada resolvida e o traço; o que ele não converte — e todo preenchimento,
    /// escondido ou desligado — fica em pedaços desta forma entre as faixas, com a ordem preservada.
    ///
    /// `None` se nenhum traço foi convertido ou se algum [`Split::Fail`]. O deslocamento X/Y da camada
    /// é aplicado aqui; o Offset (dilatação) é do conversor, que tem o motor.
    pub fn split_stack(
        &self,
        mut convert: impl FnMut(StackItem, &DrawnPaint<'_>, &StrokeSpec) -> Split,
    ) -> Option<Vec<VecPath>> {
        let mut out = Vec::new();
        let mut run: Vec<StackItem> = Vec::new();
        let mut any = false;
        for it in self.draw_order() {
            let drawn = self.drawn(it);
            let res = match &drawn {
                Some(d) => match d.paint {
                    PaintRef::Stroke(s) => convert(it, d, s),
                    PaintRef::Fill(_) => Split::Keep,
                },
                None => Split::Keep,
            };
            match res {
                Split::Keep => run.push(it),
                Split::Fail => return None,
                Split::Into(pieces) => {
                    let d = drawn?;
                    let PaintRef::Stroke(s) = d.paint else {
                        return None;
                    };
                    any = true;
                    out.extend(self.run_piece(&run));
                    run.clear();
                    out.extend(pieces.into_iter().map(|g| self.inked(g, &d, s)));
                }
            }
        }
        if !any {
            return None;
        }
        out.extend(self.run_piece(&run));
        Some(out)
    }

    /// Um pedaço vivo desta forma com só as linhas `run` (na ordem dada). `None` se nada nele se vê.
    fn run_piece(&self, run: &[StackItem]) -> Option<VecPath> {
        if !run.iter().any(|it| self.drawn(*it).is_some()) {
            return None;
        }
        let mut p = self.clone();
        if !run.contains(&StackItem::BaseFill) {
            p.fill = None;
            p.fill_hidden = None;
        }
        if !run.contains(&StackItem::BaseStroke) {
            p.stroke = None;
            p.stroke_hidden = None;
        }
        p.paints = run
            .iter()
            .filter_map(|it| match it {
                StackItem::Entry(i) => self.paints.get(*i).cloned(),
                _ => None,
            })
            .collect();
        // A ordem do pedaço: a do `run`, e a base que não está nele vai para o topo (não desenha).
        let mut order: Vec<StackItem> = Vec::with_capacity(run.len() + 2);
        let mut j = 0;
        for it in run {
            order.push(match it {
                StackItem::Entry(_) => {
                    j += 1;
                    StackItem::Entry(j - 1)
                }
                base => *base,
            });
        }
        for b in [StackItem::BaseFill, StackItem::BaseStroke] {
            if !order.contains(&b) {
                order.push(b);
            }
        }
        let pos = |b| {
            order
                .iter()
                .position(|x| *x == b)
                .and_then(|k| u8::try_from(k).ok())
                .unwrap_or(u8::MAX)
        };
        p.base_order = BaseOrder {
            fill: pos(StackItem::BaseFill),
            stroke: pos(StackItem::BaseStroke),
        };
        Some(p)
    }

    /// A faixa `g` vestida como o traço que ela substitui: a cor dele; a opacidade e a mistura do
    /// objecto; e as da CAMADA numa camada de preenchimento (ela compõe-se DENTRO da forma, e assim
    /// a faixa viva e a assada pintam igual). O deslocamento X/Y da camada move a geometria.
    fn inked(&self, mut g: VecPath, d: &DrawnPaint<'_>, s: &StrokeSpec) -> VecPath {
        let tinta = Paint::Solid(s.color());
        g.carry_look_from(&VecPath {
            opacity: self.opacity,
            blend: self.blend,
            ..VecPath::default()
        });
        if d.opacity < 1.0 || d.blend != BlendMode::Normal {
            let mut e = PaintEntry::fill(tinta);
            e.opacity = Opacity::new(d.opacity);
            e.blend = d.blend;
            g.paints = vec![e];
        } else {
            g.fill = Some(tinta);
        }
        if d.offset != [0.0, 0.0] {
            let [dx, dy] = d.offset;
            let mv = |p: &mut [f64; 2]| *p = [p[0] + dx, p[1] + dy];
            g.for_each_vert_mut(|v| {
                mv(&mut v.anchor);
                mv(&mut v.in_handle);
                mv(&mut v.out_handle);
            });
        }
        g
    }
}

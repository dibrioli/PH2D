//! **O `Style: Solid` na família dos SHAPE EDITORS** — Arc/Curve, Line, Ellipse, Polygon e o Free
//! Hand parqueado (plano 38 §5.1: *"Solid é oferecido para todos que forem possíveis"*).
//!
//! O gesto à mão livre acumula o próprio caminho ponto a ponto (`super::solid_deposit`); aqui a
//! pergunta é outra e mais fácil, porque **um shape editor JÁ é um caminho fechado**: o artista
//! autora a geometria diretamente. Então tudo o que falta é responder *que laço esta forma
//! desenha?* e entregá-lo à MESMA porta de preenchimento.
//!
//! ⚠️ **A geometria sai dos MESMOS produtores que o carimbo de dabs usa** — `offset_curve_spine`,
//! `ellipse_perimeter`, `polygon_perimeter`, `line_corner::expand`+`offset_polyline`+trim. Não há
//! uma segunda ideia de *onde a forma está*: se houvesse, a mancha sólida e o contorno tracejado do
//! gizmo divergiriam no dia em que alguém afinasse um dos dois.
//!
//! ⚠️ **A OPERAÇÃO booleana entra pelo SENTIDO do laço, não por um segundo motor:** o
//! preenchimento é `nonzero`, então orientar os `Add`/`Overlay` num sentido e os `Remove` no oposto
//! **é** a união-menos-subtração, pela aritmética que o `fill_coverage` já faz. Um `Remove` vira um
//! buraco porque a soma corrida volta a zero ali — não porque alguém rodou um boolean.

use super::stroke_multi::StrokeOp;
use super::*;
use crate::undo::ShapeEditState;

/// Área com sinal do polígono (fórmula do sapateiro). Positiva = anti-horária no espaço do
/// documento. Só o SINAL é lido, então a falta do fator ½ não importa.
fn signed_area(pts: &[[f32; 2]]) -> f32 {
    let n = pts.len();
    if n < 3 {
        return 0.0;
    }
    let mut acc = 0.0;
    for i in 0..n {
        let a = pts[i];
        let b = pts[(i + 1) % n];
        acc += a[0] * b[1] - b[0] * a[1];
    }
    acc
}

impl PainterTool {
    /// O laço FECHADO que esta forma desenha, ou `None` se ela é degenerada.
    ///
    /// ⚠️ **Uma forma ABERTA fecha sozinha** — o `fill_coverage` liga o último ponto ao primeiro por
    /// construção (`(i + 1) % n`), que é exatamente a decisão do Enio para o gesto à mão livre
    /// (*"fechar sozinho"*). Um arco em C vira a lente entre a corda e a curva; recusá-lo seria a
    /// ferramenta ficar muda num caso que tem resposta óbvia.
    pub(super) fn shape_loop(&self, st: &ShapeEditState) -> Option<Vec<[f32; 2]>> {
        let off = self.shape_offset_px();
        let path = match st {
            ShapeEditState::Curve(c) => {
                self.offset_curve_spine(&c.points, &c.handles, c.closed, off)
            }
            ShapeEditState::Ellipse(e) => {
                let (erx, ery) = self.ellipse_offset_radii(e.rx, e.ry);
                if erx < 0.5 || ery < 0.5 {
                    return None;
                }
                let mut perim = Vec::new();
                ph2d_painter_brush::ellipse_perimeter(e.center, e.u, erx, ery, &mut perim);
                perim
            }
            ShapeEditState::Polygon(p) => {
                let (erx, ery) = ((p.rx + off).max(0.5), (p.ry + off).max(0.5));
                let mut perim = Vec::new();
                ph2d_painter_brush::polygon_perimeter(p.center, p.u, erx, ery, p.sides, &mut perim);
                perim
            }
            ShapeEditState::Line(l) => {
                let mods: Vec<line_corner::CornerMod> = l
                    .corner_mods
                    .iter()
                    .copied()
                    .map(line_corner::CornerMod::from_wire)
                    .collect();
                let mut path =
                    line_corner::expand(&l.points, l.closed, &mods, self.paint.shape_grab_tol_px);
                path = line_offset::offset_polyline(&path, l.closed, off);
                if self.paint.offset_trim && off != 0.0 {
                    path = if l.closed {
                        curve_trim::trim_self_intersections_closed(&path)
                    } else {
                        curve_trim::trim_self_intersections(&path)
                    };
                }
                path
            }
        };
        (path.len() >= 3).then_some(path)
    }

    /// Todos os laços que a cena de formas preenche AGORA — a forma ativa mais cada uma das
    /// parqueadas, orientadas pela Operação de cada uma.
    pub(super) fn solid_loops(&self) -> Vec<Vec<[f32; 2]>> {
        let mut out = Vec::new();
        let mut push = |mut path: Vec<[f32; 2]>, op: StrokeOp| {
            // `Remove` no sentido oposto ⇒ a soma corrida do `nonzero` volta a zero na sobreposição,
            // que É o buraco. `Overlay` acompanha o `Add`: duas formas independentes da mesma cor
            // fundem, nunca se cancelam.
            let want_ccw = op != StrokeOp::Remove;
            if (signed_area(&path) > 0.0) != want_ccw {
                path.reverse();
            }
            out.push(path);
        };
        if let Some(p) = self.capture_shape().and_then(|st| self.shape_loop(&st)) {
            push(p, self.paint.active_op);
        }
        for s in &self.paint.parked_shapes {
            if let Some(p) = self.shape_loop(&s.state) {
                push(p, s.op);
            }
        }
        out
    }
}

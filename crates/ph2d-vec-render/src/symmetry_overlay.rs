//! **As linhas de simetria no canvas** (plano 25 §9, W6.3).
//!
//! *"Quando ligada linhas aparecem no canvas"* (Enio, 2026-08-01). Este módulo desenha-as, e nada
//! mais — o eixo já foi resolvido por quem sabe onde ele está.
//!
//! # A direção vem da MESMA porta que reflete
//!
//! O chamador pergunta `SymmetrySpec::mirror_dir()` e passa o resultado. Uma segunda derivação
//! aqui desenharia um eixo onde a geometria não espelha — e ninguém lê um número numa screenshot,
//! então a divergência só apareceria como *"a linha está torta"*, meses depois.
//!
//! # Um espelho tem UMA linha; uma rosácea tem `segments` raios
//!
//! São figuras diferentes porque são fatos diferentes: um espelho tem um eixo (e os dois lados
//! dele), uma rosácea tem um CENTRO e as fatias que saem dele. Desenhar a rosácea como uma linha
//! só a faria parecer um espelho que não funciona.

use ph2d_vector::{Affine, BezPath, Brush, Color, Point, Stroke, VectorScene};

use crate::guides::clip_line_to_rect;

/// Âmbar apagado — a cor de um eixo AUTORADO que atravessa a tela. Distinta do ciano das guias
/// (aquelas são do documento e valem para tudo; esta pertence a UMA forma) e do realce de
/// selecção, que já ocupa a silhueta.
const AXIS: Color = Color::from_rgba8(235, 175, 90, 190);

/// O raio, em px de TELA, dos raios de uma rosácea. Eles não atravessam a tela como um espelho:
/// uma rosácea tem um alcance, e desenhá-la infinita esconderia onde ela de facto está.
const SPOKE_PX: f64 = 120.0;

/// **Uma linha de simetria a desenhar**, já resolvida em MUNDO pelo chamador.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SymmetryAxis {
    /// Um ponto sobre a linha (ou o centro da rosácea), em coordenadas de mundo.
    pub at: [f64; 2],
    /// A direcção da linha em mundo — a que [`ph2d_symmetry::SymmetrySpec::mirror_dir`] deu,
    /// levada pela pose da forma. Não precisa de estar normalizada.
    pub dir: [f64; 2],
    /// `Some(n)` ⇒ rosácea de `n` fatias (desenha `n` raios). `None` ⇒ espelho (uma linha).
    pub segments: Option<u32>,
}

/// Desenha os eixos de simetria vivos, recortados à janela da cena.
///
/// `canvas` é `[x, y, w, h]` em px de tela — a MESMA janela que as guias do documento usam, pela
/// mesma razão: o chrome dos painéis é pintado depois, por cima.
pub fn draw_symmetry_axes(
    axes: &[SymmetryAxis],
    canvas: [f64; 4],
    transform: Affine,
    target: &mut VectorScene,
) {
    let thin = Stroke::new(1.0);
    for ax in axes {
        // O ponto sobe pela câmera; a DIREÇÃO sobe pelo mesmo afim mas sem a translação — levar
        // uma direção como ponto é o erro clássico, e aqui ele deixaria toda linha apontando
        // para o canto do ecrã.
        let a = transform * Point::new(ax.at[0], ax.at[1]);
        let b = transform * Point::new(ax.at[0] + ax.dir[0], ax.at[1] + ax.dir[1]);
        let d = Point::new(b.x - a.x, b.y - a.y);
        match ax.segments {
            None => {
                let Some((s, e)) = clip_line_to_rect(a, d, canvas) else {
                    continue;
                };
                stroke_seg(target, &thin, s, e);
            }
            Some(n) => {
                // A rosácea: `n` raios a partir do centro, o primeiro sobre a direcção autorada.
                // O passo é o mesmo `TAU / n` do kernel — e é por isso que o desenho e a
                // geometria concordam sobre onde a primeira fatia começa.
                let len = (d.x * d.x + d.y * d.y).sqrt();
                if len < 1e-9 {
                    continue;
                }
                let (ux, uy) = (d.x / len, d.y / len);
                let step = core::f64::consts::TAU / f64::from(n.max(1));
                for k in 0..n {
                    let (sn, cs) = (step * f64::from(k)).sin_cos();
                    let (rx, ry) = (ux * cs - uy * sn, ux * sn + uy * cs);
                    stroke_seg(
                        target,
                        &thin,
                        a,
                        Point::new(a.x + rx * SPOKE_PX, a.y + ry * SPOKE_PX),
                    );
                }
            }
        }
    }
}

fn stroke_seg(target: &mut VectorScene, stroke: &Stroke, s: Point, e: Point) {
    let mut line = BezPath::new();
    line.move_to(s);
    line.line_to(e);
    target
        .inner_mut()
        .stroke(stroke, Affine::IDENTITY, &Brush::Solid(AXIS), None, &line);
}

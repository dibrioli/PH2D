//! **A METADE DO TRAÇO** do desenho de um caminho — módulo irmão de [`super`] pelo teto de LOC.
//!
//! ⚠️ **Extraída, e não copiada.** O que um traço desenha é o que o [`ph2d_vec_scene::stroke_plan`]
//! lista — tracejado, pontas, alinhamento — e uma segunda cópia disso divergiria no primeiro
//! ajuste. Quem quiser traçar passa por aqui, incluindo a rota de INSTÂNCIA de Motion (que precisa
//! do traço sem a metade do preenchimento: a cor de um primitivo é o `tint` da instância).
//!
//! ⭐ O corte por RESPONSABILIDADE já estava escrito no doc-comment da função; o teto de LOC só
//! cobrou o que a wave B do plano 35 acrescentou (o padrão no traço).

use std::borrow::Cow;

use ph2d_vec_scene::{StrokePiece, VecPath};
use ph2d_vector::{Affine, Brush, Fill, Shape, Stroke, VectorScene};

use crate::{PathTess, PatternTile, build_bezpath, color, kurbo_stroke, pattern, stroke_uniform};

/// **A metade do TRAÇO** de [`draw_path_with`] — extraída porque a rota de INSTÂNCIA de
/// Motion precisa dela sem a metade do preenchimento (a cor de um primitivo é o `tint` da
/// instância, não o `path.fill`).
///
/// ⚠️ **Extraída, e não copiada.** O traço é o que o [`ph2d_vec_scene::stroke_plan`] lista —
/// tracejado, pontas, alinhamento — e uma segunda cópia disso divergiria no primeiro ajuste
/// ([[feedback_two_doors_to_the_same_question_diverge]]). Quem quiser traçar passa por aqui.
pub(crate) fn draw_stroke_with(
    path: &VecPath,
    tess: &PathTess,
    transform: Affine,
    target: &mut VectorScene,
    tile: Option<&PatternTile>,
) {
    let fill_bp = tess.fill_bp.as_ref();
    let stroke_own = tess.stroke_bp.as_ref();
    if let Some(s) = path.stroke.as_ref() {
        let bp = stroke_own
            .or(fill_bp)
            .expect("stroke => um dos dois desenhos existe");
        // O QUE um traço desenha é decidido em `ph2d_vec_scene::stroke_plan` — a porta
        // única, que o Outline Stroke também consome. Aqui só se PINTA o que ela lista.
        let brush = Brush::Solid(color(s.color()));
        // ⭐⭐ **O PADRÃO NO TRAÇO** (plano 35, wave B) — a mesma lei do preenchimento: desenha a
        // IMAGEM quando o ladrilho existe, e a `fallback` (o `brush` acima) quando não. As duas
        // metades são desenho certo; ⛔ desenhar NADA seria pior.
        //
        // ⚠️ **A caixa que o `Clamp` enquadra é a do TRAÇO**, e não a do preenchimento: um traço
        // pode existir numa forma sem `fill` nenhum, e ali `fill_bp` é `None`.
        let pat_tile = s.pattern().zip(tile);
        if let Some((pat, t)) = pat_tile {
            let placement = pat.placement_in(t.cells, t.tile_px, {
                let b = bp.bounding_box();
                ([b.x0, b.y0], [b.x1, b.y1])
            });
            let ext = pattern::extend_of(pat.mode);
            for piece in ph2d_vec_scene::stroke_plan(path, s) {
                // ⚠️ Só a FAIXA recebe o padrão. Uma ponta de seta é um preenchimento sólido, e o
                // `stroke_plan` já a devolve como peça própria — pintá-la com o ladrilho poria o
                // motivo dentro de um triângulo de 3 px, onde ele não se lê.
                let (line, own): (std::borrow::Cow<'_, VecPath>, Option<[f64; 2]>) = match piece {
                    StrokePiece::Line {
                        path: Cow::Borrowed(_),
                    } => (Cow::Borrowed(path), None),
                    StrokePiece::Line {
                        path: Cow::Owned(p),
                    } => {
                        let d = ph2d_vec_scene::dash_for(&p, s);
                        (Cow::Owned(p), d)
                    }
                    // ⚠️⚠️ **Um marcador ABERTO traça-se; um CHEIO preenche-se** — e a 1.ª redacção
                    // desta wave tratou os dois como preenchimento, o que fazia uma seta aberta
                    // sair **maciça** assim que o traço ganhava padrão. A distinção é do
                    // `stroke_plan` (`Marker::is_filled`), e quem pinta só a obedece; ela existe
                    // idêntica no ramo sólido logo abaixo. *Duas cópias da lista de peças divergem
                    // no primeiro ajuste — e esta divergiu antes de o ajuste chegar.*
                    StrokePiece::Symbol { path: geo } => {
                        stroke_uniform::stroke_uniform(
                            target,
                            &Stroke::new(s.width),
                            transform,
                            &brush,
                            &build_bezpath(&geo),
                        );
                        continue;
                    }
                    // ⚠️ As pontas ficam **SÓLIDAS de propósito** (a cor de recurso): um motivo
                    // dentro de um triângulo de 3 px não se lê. Só a FAIXA recebe o ladrilho.
                    StrokePiece::Fill { path: geo } => {
                        target.inner_mut().fill(
                            Fill::NonZero,
                            transform,
                            &brush,
                            None,
                            &build_bezpath(&geo),
                        );
                        continue;
                    }
                };
                let owned_bp;
                let line_bp = match &line {
                    Cow::Borrowed(_) => bp,
                    Cow::Owned(p) => {
                        owned_bp = build_bezpath(p);
                        &owned_bp
                    }
                };
                stroke_uniform::stroke_uniform_image(
                    target,
                    &kurbo_stroke(s, own.or(tess.dash)),
                    transform,
                    &t.image,
                    Affine::new(placement),
                    ext,
                    ext,
                    t.quality,
                    pat.alpha,
                    line_bp,
                );
            }
            return;
        }
        for piece in ph2d_vec_scene::stroke_plan(path, s) {
            match piece {
                StrokePiece::Line { path: line } => match line {
                    // Emprestado = a peça É o path, e `bp` já o descreve (o caso de 99% dos
                    // traços, e o motivo de o plano devolver `Cow`). ⚠️ Passa por REFERÊNCIA:
                    // clonar o `BezPath` por instância era um custo por-instância que o cache de
                    // tesselação não remove — e a 160k estrelas era metade do que sobrava (byte-
                    // idêntico: um clone e o original encodam os mesmos bytes no Vello).
                    Cow::Borrowed(_) => {
                        stroke_uniform::stroke_uniform(
                            target,
                            &kurbo_stroke(s, tess.dash),
                            transform,
                            &brush,
                            bp,
                        );
                    }
                    Cow::Owned(p) => {
                        // Encurtada pelos marcadores ⇒ o padrão tem de encaixar NA LINHA que se
                        // traça, não no objeto: o `tess.dash` mede o caminho INTEIRO, e com a
                        // seta a ponta ficava DESCOLADA do último traço (Enio, 2026-08-22). A
                        // peça emprestada (acima) É o caminho, e aí o cache já é a resposta —
                        // o caso comum segue sem medir nada por quadro.
                        let dash = ph2d_vec_scene::dash_for(&p, s);
                        let line_bp = build_bezpath(&p);
                        stroke_uniform::stroke_uniform(
                            target,
                            &kurbo_stroke(s, dash),
                            transform,
                            &brush,
                            &line_bp,
                        );
                    }
                },
                StrokePiece::Symbol { path: geo } => {
                    stroke_uniform::stroke_uniform(
                        target,
                        &Stroke::new(s.width),
                        transform,
                        &brush,
                        &build_bezpath(&geo),
                    );
                }
                StrokePiece::Fill { path: geo } => {
                    target.inner_mut().fill(
                        Fill::NonZero,
                        transform,
                        &brush,
                        None,
                        &build_bezpath(&geo),
                    );
                }
            }
        }
    }
}

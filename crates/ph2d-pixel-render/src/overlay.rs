//! **A selecção por cima da arte** (P6): as formigas a marchar pela borda da máscara (as arestas
//! das células, em células do documento). A moldura solta é o gizmo do sprite da casa (P6b).
//! Cores: o núcleo claro e o anel escuro do Draw — o fundo é a arte, não o chrome.

use ph2d_pixel_core::selection::Mask;
use ph2d_pixel_core::{Rect, SliceShape};
use ph2d_pixel_slices::{Grab, Lados};
use ph2d_vector::{Affine, BezPath, Color, Fill, Point, Rect as KRect, Stroke as Pen, VectorScene};

use crate::view::{CURSOR_CORE, HALO, HALO_PX};

/// O comprimento de cada traço (e de cada vão) das formigas, em píxeis de ECRÃ.
pub const ANTS_DASH_PX: f64 = 4.0; // LITERAL-PX-OK: padrão das formigas do Aseprite, em píxeis de ecrã

/// Um segmento de aresta entre cantos de células `((x0, y0), (x1, y1))`.
pub type Edge = ((i32, i32), (i32, i32));

/// **A borda da máscara**: os segmentos MÁXIMOS onde uma célula seleccionada encosta numa que não
/// é (fora do sprite conta como não seleccionada), horizontais e depois verticais.
#[must_use]
pub fn mask_edges(m: &Mask) -> Vec<Edge> {
    let (w, h) = (m.width() as i32, m.height() as i32);
    let mut out = Vec::new();
    for y in 0..=h {
        let mut run: Option<i32> = None;
        for x in 0..=w {
            let edge = x < w && m.get(x, y - 1) != m.get(x, y);
            match (edge, run) {
                (true, None) => run = Some(x),
                (false, Some(x0)) => {
                    out.push(((x0, y), (x, y)));
                    run = None;
                }
                _ => {}
            }
        }
    }
    for x in 0..=w {
        let mut run: Option<i32> = None;
        for y in 0..=h {
            let edge = y < h && m.get(x - 1, y) != m.get(x, y);
            match (edge, run) {
                (true, None) => run = Some(y),
                (false, Some(y0)) => {
                    out.push(((x, y0), (x, y)));
                    run = None;
                }
                _ => {}
            }
        }
    }
    out
}

/// **As formigas**: as arestas `edges` (células do documento) pelo afim `to_screen`, o anel escuro
/// cheio por baixo e o traço claro tracejado por cima, deslocado `phase_px` (o relógio do host).
pub fn draw_marching_ants(
    scene: &mut VectorScene,
    edges: &[Edge],
    to_screen: Affine,
    phase_px: f64,
    width_px: f64,
) {
    if edges.is_empty() {
        return;
    }
    let mut path = BezPath::new();
    for &((x0, y0), (x1, y1)) in edges {
        path.move_to(to_screen * Point::new(f64::from(x0), f64::from(y0)));
        path.line_to(to_screen * Point::new(f64::from(x1), f64::from(y1)));
    }
    let nucleo = 2.0 * width_px;
    let s = scene.inner_mut();
    s.stroke(
        &Pen::new(nucleo + 2.0 * HALO_PX),
        Affine::IDENTITY,
        HALO,
        None,
        &path,
    );
    let dash = Pen::new(nucleo).with_dashes(phase_px, [ANTS_DASH_PX, ANTS_DASH_PX]);
    s.stroke(&dash, Affine::IDENTITY, CURSOR_CORE, None, &path);
}

/// A largura do quadradinho de uma alça e do pivot, em píxeis de ECRÃ: a mesma a qualquer zoom.
pub const SLICE_HANDLE_PX: f64 = 6.0; // LITERAL-PX-OK: alça da slice, em píxeis de ecrã
/// A alça (ou o pivot) sob o ponteiro: maior, para se ver o que o premir agarra.
pub const SLICE_HANDLE_HOT_PX: f64 = 10.0; // LITERAL-PX-OK: alça quente da slice, em píxeis de ecrã

/// Uma slice como se desenha neste quadro.
#[derive(Debug, Clone, Copy)]
pub struct SliceMark {
    pub shape: SliceShape,
    pub color: [u8; 4],
    pub selected: bool,
    /// O que o premir agarraria sob o ponteiro (ou o que está agarrado): realça-se.
    pub hot: Option<Grab>,
}

/// Uma aresta horizontal ou vertical cortada em `clip`, se sobra alguma coisa.
fn cut(a: (i32, i32), b: (i32, i32), clip: Rect) -> Option<Edge> {
    let (x0, y0) = (clip.x, clip.y);
    let (x1, y1) = (x0 + clip.w as i32, y0 + clip.h as i32);
    if a.1 == b.1 {
        let (l, r) = (a.0.max(x0), b.0.min(x1));
        ((y0..=y1).contains(&a.1) && l < r).then_some(((l, a.1), (r, a.1)))
    } else {
        let (t, d) = (a.1.max(y0), b.1.min(y1));
        ((x0..=x1).contains(&a.0) && t < d).then_some(((a.0, t), (a.0, d)))
    }
}

/// **O contorno de `r`, cortado no canvas** (F3: só o desenho corta).
#[must_use]
pub fn rect_edges(r: Rect, canvas: Rect) -> Vec<Edge> {
    let (x1, y1) = (r.x + r.w as i32, r.y + r.h as i32);
    [
        ((r.x, r.y), (x1, r.y)),
        ((r.x, y1), (x1, y1)),
        ((r.x, r.y), (r.x, y1)),
        ((x1, r.y), (x1, y1)),
    ]
    .into_iter()
    .filter_map(|(a, b)| cut(a, b, canvas))
    .collect()
}

/// As linhas do 9-slice (esquerda, direita, cima, baixo), de lado a lado da caixa e dentro dela,
/// cortadas no canvas; `so` escolhe quais.
fn center_edges_of(s: &SliceShape, canvas: Rect, so: Lados) -> Vec<Edge> {
    let Some(c) = s.center else {
        return Vec::new();
    };
    let b = s.bounds;
    let clip = b.intersect(canvas);
    let (x1, y1) = (b.x + b.w as i32, b.y + b.h as i32);
    let (cx, cy) = (b.x + c.x, b.y + c.y);
    let (cx1, cy1) = (cx + c.w as i32, cy + c.h as i32);
    [
        (so.left, (cx, b.y), (cx, y1)),
        (so.right, (cx1, b.y), (cx1, y1)),
        (so.top, (b.x, cy), (x1, cy)),
        (so.bottom, (b.x, cy1), (x1, cy1)),
    ]
    .into_iter()
    .filter(|e| e.0)
    .filter_map(|(_, p, q)| cut(p, q, clip))
    .collect()
}

const TODAS: Lados = Lados {
    left: true,
    right: true,
    top: true,
    bottom: true,
};

#[cfg(test)]
fn center_edges(s: &SliceShape, canvas: Rect) -> Vec<Edge> {
    center_edges_of(s, canvas, TODAS)
}

/// As bordas da caixa presas a `l`, cortadas no canvas.
fn bound_edges(r: Rect, canvas: Rect, l: Lados) -> Vec<Edge> {
    let (x1, y1) = (r.x + r.w as i32, r.y + r.h as i32);
    [
        (l.top, (r.x, r.y), (x1, r.y)),
        (l.bottom, (r.x, y1), (x1, y1)),
        (l.left, (r.x, r.y), (r.x, y1)),
        (l.right, (x1, r.y), (x1, y1)),
    ]
    .into_iter()
    .filter(|e| e.0)
    .filter_map(|(_, a, b)| cut(a, b, canvas))
    .collect()
}

/// O realce do que o ponteiro agarra: cheio, núcleo claro com o anel, como o contorno selecionado.
fn realce(scene: &mut VectorScene, path: &BezPath, core: f64) {
    let s = scene.inner_mut();
    let halo = Pen::new(core + 2.0 * HALO_PX);
    s.stroke(&halo, Affine::IDENTITY, HALO, None, path);
    s.stroke(&Pen::new(core), Affine::IDENTITY, CURSOR_CORE, None, path);
}

/// O lado do quadradinho: maior se é o que o ponteiro agarra.
const fn lado(quente: bool) -> f64 {
    if quente {
        SLICE_HANDLE_HOT_PX
    } else {
        SLICE_HANDLE_PX
    }
}

fn edges_path(edges: &[Edge], m: Affine) -> BezPath {
    let mut path = BezPath::new();
    for &((x0, y0), (x1, y1)) in edges {
        path.move_to(m * Point::new(f64::from(x0), f64::from(y0)));
        path.line_to(m * Point::new(f64::from(x1), f64::from(y1)));
    }
    path
}

/// Um quadradinho de tamanho fixo no ecrã, cheio de `fill`, com o anel escuro.
fn handle(scene: &mut VectorScene, at: Point, fill: Color, side_px: f64) {
    let h = side_px / 2.0;
    let r = KRect::new(at.x - h, at.y - h, at.x + h, at.y + h);
    let s = scene.inner_mut();
    s.stroke(&Pen::new(2.0 * HALO_PX), Affine::IDENTITY, HALO, None, &r);
    s.fill(Fill::NonZero, Affine::IDENTITY, fill, None, &r);
}

/// **Uma slice sobre a arte**: não selecionada, o contorno fino na cor dela; selecionada, o contorno
/// duplo claro/escuro e as alças (T2). As linhas do 9-slice e o pivot, na cor dela. O pivot de uma
/// selecionada não se corta: tem de se ver para se agarrar, mesmo fora do canvas.
pub fn draw_slice(
    scene: &mut VectorScene,
    mark: &SliceMark,
    canvas: Rect,
    to_screen: Affine,
    width_px: f64,
) {
    let s = mark.shape;
    let [r, g, b, a] = mark.color;
    let cor = Color::from_rgba8(r, g, b, a);
    let nucleo = 2.0 * width_px;
    let contorno = edges_path(&rect_edges(s.bounds, canvas), to_screen);
    let linhas = edges_path(&center_edges_of(&s, canvas, TODAS), to_screen);
    let dash = Pen::new(width_px).with_dashes(0.0, [ANTS_DASH_PX, ANTS_DASH_PX]);
    let inner = scene.inner_mut();
    inner.stroke(&dash, Affine::IDENTITY, cor, None, &linhas);
    if let Some(Grab::Center(l)) = mark.hot {
        realce(
            scene,
            &edges_path(&center_edges_of(&s, canvas, l), to_screen),
            nucleo,
        );
    }
    if mark.selected {
        let inner = scene.inner_mut();
        let halo = Pen::new(nucleo + 2.0 * HALO_PX);
        inner.stroke(&halo, Affine::IDENTITY, HALO, None, &contorno);
        inner.stroke(
            &Pen::new(nucleo),
            Affine::IDENTITY,
            CURSOR_CORE,
            None,
            &contorno,
        );
        let (x0, y0) = (f64::from(s.bounds.x), f64::from(s.bounds.y));
        let (w, h) = (f64::from(s.bounds.w), f64::from(s.bounds.h));
        let (x1, y1) = (x0 + w, y0 + h);
        let (mx, my) = (x0 + w / 2.0, y0 + h / 2.0);
        let mut quente = None;
        if let Some(Grab::Bounds(l)) = mark.hot {
            let borda = edges_path(&bound_edges(s.bounds, canvas, l), to_screen);
            realce(scene, &borda, 2.0 * nucleo);
            let x = if l.left {
                x0
            } else if l.right {
                x1
            } else {
                mx
            };
            quente = Some((
                x,
                if l.top {
                    y0
                } else if l.bottom {
                    y1
                } else {
                    my
                },
            ));
        }
        for (x, y) in [
            (x0, y0),
            (mx, y0),
            (x1, y0),
            (x0, my),
            (x1, my),
            (x0, y1),
            (mx, y1),
            (x1, y1),
        ] {
            let at = to_screen * Point::new(x, y);
            handle(scene, at, CURSOR_CORE, lado(quente == Some((x, y))));
        }
    } else {
        scene
            .inner_mut()
            .stroke(&Pen::new(nucleo), Affine::IDENTITY, cor, None, &contorno);
    }
    if let Some((px, py)) = s.pivot {
        let (x, y) = (s.bounds.x + px, s.bounds.y + py);
        if mark.selected || canvas.contains(x, y) {
            let at = Point::new(f64::from(x) + 0.5, f64::from(y) + 0.5);
            handle(
                scene,
                to_screen * at,
                cor,
                lado(mark.hot == Some(Grab::Pivot)),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{center_edges, mask_edges, rect_edges};
    use ph2d_pixel_core::selection::Mask;
    use ph2d_pixel_core::{Rect, SliceShape};

    /// F3: só o desenho corta. A aresta que cai fora do canvas não se desenha; a que entra, corta-se;
    /// w e h separados numa caixa não quadrada.
    #[test]
    fn a_slice_outline_is_cut_at_the_canvas_only_in_the_drawing() {
        let canvas = Rect::new(0, 0, 32, 16);
        assert_eq!(
            rect_edges(Rect::new(4, 2, 10, 6), canvas),
            vec![
                ((4, 2), (14, 2)),
                ((4, 8), (14, 8)),
                ((4, 2), (4, 8)),
                ((14, 2), (14, 8))
            ]
        );
        assert_eq!(
            rect_edges(Rect::new(28, 12, 10, 10), canvas),
            vec![((28, 12), (32, 12)), ((28, 12), (28, 16))],
            "a direita e o fundo caem fora"
        );
        assert!(rect_edges(Rect::new(40, 2, 4, 4), canvas).is_empty());
        let s = SliceShape {
            bounds: Rect::new(28, 2, 10, 6),
            center: Some(Rect::new(2, 1, 6, 4)),
            pivot: None,
        };
        assert_eq!(
            center_edges(&s, canvas),
            vec![((30, 2), (30, 8)), ((28, 3), (32, 3)), ((28, 7), (32, 7))],
            "a linha x=36 cai fora"
        );
    }

    #[test]
    fn the_outline_of_a_box_is_four_maximal_edges_and_a_hole_adds_four() {
        let mut m = Mask::marquee(8, 8, (1, 1), (4, 3));
        assert_eq!(
            mask_edges(&m),
            vec![
                ((1, 1), (5, 1)),
                ((1, 4), (5, 4)),
                ((1, 1), (1, 4)),
                ((5, 1), (5, 4))
            ]
        );
        m.set(2, 2, false);
        assert_eq!(mask_edges(&m).len(), 4 + 4);
        assert!(mask_edges(&Mask::empty(4, 4)).is_empty());
        // a borda do sprite também é borda
        assert_eq!(mask_edges(&Mask::full(3, 2)).len(), 4);
    }
}

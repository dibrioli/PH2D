//! **O overlay das guias** (P13 U-Guias): as linhas de [`ph2d_pixel_guia::linhas_da_guia`] dentro
//! da janela e as alças (PF, origem iso) — acima da grade, abaixo do cursor. As cores vêm do
//! chamador (tokens resolvidos); aqui só a geometria.

use ph2d_pixel_core::Guias;
use ph2d_pixel_guia::{alcas, linhas_da_guia};
use ph2d_vector::{Affine, BezPath, Circle, Color, Fill, Point, Stroke as Pen, VectorScene};

/// O raio da alça de uma guia, em píxeis de ecrã.
pub const GUIDE_HANDLE_PX: f64 = 5.0; // LITERAL-PX-OK: alça de guia, em píxeis de ecrã

/// Desenha as guias `g` de um sprite posto por `to_screen`, recortadas à janela `janela` (px).
/// `cores = [linha, alça]`; `traco` = espessura de ecrã da linha.
pub fn desenha(
    scene: &mut VectorScene,
    g: &Guias,
    to_screen: Affine,
    janela: (u32, u32),
    [linha, alca]: [Color; 2],
    traco: f64,
) {
    if *g == Guias::default() || to_screen.determinant().abs() < f64::MIN_POSITIVE {
        return;
    }
    let inv = to_screen.inverse();
    let (jw, jh) = (f64::from(janela.0), f64::from(janela.1));
    let cantos = [(0.0, 0.0), (jw, 0.0), (0.0, jh), (jw, jh)].map(|(x, y)| inv * Point::new(x, y));
    let lo = |f: fn(&Point) -> f64| cantos.iter().map(f).fold(f64::INFINITY, f64::min) as f32;
    let hi = |f: fn(&Point) -> f64| cantos.iter().map(f).fold(f64::NEG_INFINITY, f64::max) as f32;
    let rect = [lo(|p| p.x), lo(|p| p.y), hi(|p| p.x), hi(|p| p.y)];
    let mut path = BezPath::new();
    for (a, b) in linhas_da_guia(g, rect) {
        path.move_to(to_screen * Point::new(f64::from(a[0]), f64::from(a[1])));
        path.line_to(to_screen * Point::new(f64::from(b[0]), f64::from(b[1])));
    }
    let s = scene.inner_mut();
    s.stroke(&Pen::new(traco), Affine::IDENTITY, linha, None, &path);
    for (_, c) in alcas(g) {
        let p = to_screen * Point::new(f64::from(c[0]), f64::from(c[1]));
        let disco = Circle::new(p, GUIDE_HANDLE_PX);
        s.fill(Fill::NonZero, Affine::IDENTITY, alca, None, &disco);
        s.stroke(&Pen::new(traco), Affine::IDENTITY, linha, None, &disco);
    }
}

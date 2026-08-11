//! **O que o GESTO de região desenha** — irmão de [`super`] pelo teto de 700 LOC.
//!
//! Os dois pintores da região em curso: o **retângulo** entre dois cantos e o **LAÇO** que a mão
//! desenhou. O corte é por assunto — tudo o mais na crate desenha o que o documento É (ou as alças
//! que o editam), e estes desenham uma coisa que ainda não existe e que some ao soltar.
//!
//! ⚠️ **px de TELA, sob `Affine::IDENTITY`** — no Vello o transform do `stroke` MULTIPLICA a
//! largura, e é o que já transformou o realce do Flip num borrão: uma espessura de 1,0 sob o afim
//! mundo→tela é 1,0 unidade de MUNDO.

use ph2d_vector::{Affine, BezPath, Brush, Color, Fill, Point, Rect, Stroke, VectorScene};

/// Desenha a **caixa** da região em px de tela (o shell passa cantos de tela): preenchimento
/// translúcido + contorno.
///
/// ⚠️ O doc antigo dizia *"chamada só enquanto o Shift+arrasto está ativo"* — falso desde a W3b
/// (o retângulo deixou de exigir Shift; ele é o modificador de ADIÇÃO) e duplamente falso desde o
/// laço (ela é UMA das duas formas). Um comentário que contradiz o código shipado é pior que
/// comentário nenhum.
pub fn draw_marquee(min: [f64; 2], max: [f64; 2], target: &mut VectorScene) {
    let (x0, x1) = (min[0].min(max[0]), min[0].max(max[0]));
    let (y0, y1) = (min[1].min(max[1]), min[1].max(max[1]));
    let rect = Rect::new(x0, y0, x1, y1);
    target.inner_mut().fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &Brush::Solid(Color::from_rgba8(90, 200, 235, 40)),
        None,
        &rect,
    );
    let mut outline = BezPath::new();
    outline.move_to(Point::new(x0, y0));
    outline.line_to(Point::new(x1, y0));
    outline.line_to(Point::new(x1, y1));
    outline.line_to(Point::new(x0, y1));
    outline.close_path();
    target.inner_mut().stroke(
        &Stroke::new(1.0),
        Affine::IDENTITY,
        &Brush::Solid(Color::from_rgba8(90, 200, 235, 200)),
        None,
        &outline,
    );
}

/// **O LAÇO em curso** — a região que a mão está a desenhar, em px de TELA (como o
/// [`draw_marquee`], sob `Affine::IDENTITY`).
///
/// ⚠️ **A aresta de FECHO é desenhada**, da ponta viva de volta ao começo, e é a metade que faz
/// deste desenho um oráculo em vez de um enfeite: a região que vai selecionar é o polígono
/// FECHADO, e um laço aberto na tela mentiria sobre o que a soltura vai apanhar — o artista
/// julgaria a forma sem ver o corte que ele próprio fez ao começar longe de onde acabou.
///
/// A mesma tinta do retângulo (o gesto é o mesmo, com outra forma), e o miolo preenchido pela
/// MESMA regra que o selecciona (`NonZero` desenha, `even-odd` selecciona — em laço simples são
/// a mesma região; num laço que se cruza o miolo pintado é mais generoso que o apanhado, e é o
/// preço honesto de o Vello não expor even-odd por aqui).
pub fn draw_lasso(points: &[(f64, f64)], target: &mut VectorScene) {
    if points.len() < 2 {
        return;
    }
    let mut path = BezPath::new();
    path.move_to(Point::new(points[0].0, points[0].1));
    for p in &points[1..] {
        path.line_to(Point::new(p.0, p.1));
    }
    path.close_path();
    target.inner_mut().fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &Brush::Solid(Color::from_rgba8(90, 200, 235, 40)),
        None,
        &path,
    );
    target.inner_mut().stroke(
        &Stroke::new(1.0),
        Affine::IDENTITY,
        &Brush::Solid(Color::from_rgba8(90, 200, 235, 200)),
        None,
        &path,
    );
}

//! ⭐ **O CONTA-GOTAS do vetor** (D10, tecla `I` — o *Eyedropper* do Illustrator): clicar num
//! caminho copia o PREENCHIMENTO e o TRAÇO dele para a ferramenta (o que se desenha a seguir) e,
//! pela mesma porta da cor, para os caminhos seleccionados.
//!
//! ⚠️ As cores lêem-se pela MESMA lei que semeia a ferramenta quando se selecciona um caminho
//! (`vector_bridge::style::seed_fill_from_paint`, `StrokeSpec::color`): um gradiente fica quieto
//! (não há uma cor só para copiar), e alfa 0 é o «sem preenchimento».

use ph2d_vec_edit::PenTool;
use ph2d_vec_scene::VecScene;

/// As cores do caminho sob `world` (com a tolerância de clique de `tol_world`), ou `None` se o
/// clique caiu no vazio. `(preenchimento, traço)` — cada um `None` quando o caminho não o tem
/// numa cor só.
#[must_use]
pub fn sample(
    scene: &VecScene,
    pen: &PenTool,
    world: [f64; 2],
    tol_world: f64,
) -> Option<(Option<[u8; 4]>, Option<[u8; 4]>)> {
    let id = pen.path_at(scene, world, tol_world)?;
    let path = scene.paths().iter().find(|p| p.id == id)?;
    let fill = crate::vector_bridge::seed_fill_from_paint(path.fill.as_ref());
    let stroke = path.stroke.as_ref().map(|s| {
        let c = s.color();
        [c.r, c.g, c.b, c.a]
    });
    Some((fill, stroke))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⭐ Clicar numa forma devolve o preenchimento e o traço DELA; clicar no vazio não devolve nada.
    #[test]
    fn a_click_on_a_shape_takes_its_colours_and_the_void_takes_none() {
        let mut vec = crate::state::VecState::default();
        let mut scene = VecScene::default();
        let style = ph2d_vec_edit::PenStyle {
            fill: ph2d_vec_scene::Rgba8::new(200, 30, 40, 255),
            stroke: ph2d_vec_scene::Rgba8::new(10, 220, 30, 255),
            ..ph2d_vec_edit::PenStyle::default()
        };
        vec.shape.set_style(style);
        let kind = ph2d_vec_scene::ShapeKind::Rectangle;
        let c = ph2d_vec_edit::ShapeConstraint::default();
        vec.shape
            .on_press(&mut scene, kind, kind.defaults(), [-1.0, -1.0], 0.01, c);
        vec.shape.on_drag(&mut scene, [1.0, 1.0], c);
        vec.shape.on_release(&mut scene);
        let (fill, stroke) = sample(&scene, &vec.pen, [0.0, 0.0], 0.1).expect("o centro é a forma");
        assert_eq!(fill, Some([200, 30, 40, 255]));
        assert_eq!(stroke, Some([10, 220, 30, 255]));
        assert!(
            sample(&scene, &vec.pen, [9.0, 9.0], 0.1).is_none(),
            "o vazio não tem cor"
        );
    }
}

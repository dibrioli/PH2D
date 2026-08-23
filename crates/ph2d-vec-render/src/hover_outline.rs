//! **O CONTORNO DE PROVENIÊNCIA** — a forma que o ponteiro aponta ganha uma linha à volta.
//!
//! A metade de CANVAS do realce C2 (estudo de UI viva): passar sobre uma linha da Hierarquia
//! acende a forma dela aqui, e passar sobre uma forma acende a linha lá. Quem decide *qual* objecto
//! é a porta única da shell (`App::pick_hovered_object`); este módulo só o desenha.
//!
//! ⚠️ **px de TELA, sob `Affine::IDENTITY`** — a lei que o [`super::marquee`] escreve: no Vello o
//! transform do `stroke` **multiplica** a largura, então uma espessura de 1,5 sob o afim
//! mundo→tela seria 1,5 unidades de MUNDO. A geometria atravessa a câmara; a caneta não.
//!
//! ⚠️ **É um CONTORNO, nunca um preenchimento.** A forma apontada continua a mostrar a própria
//! tinta — tapá-la responderia *"o que está aqui"* apagando a resposta.

use ph2d_vec_scene::VecPath;
use ph2d_vector::{Affine, Brush, Color, Stroke, VectorScene};

/// A espessura do contorno, em px de TELA.
const OUTLINE_PX: f64 = 1.5;

/// A tinta do contorno. ⚠️ **Não passa por token de propósito**: os overlays desta crate desenham
/// em tinta própria (o marquee, o laço, a gaiola), porque a crate não alcança o tema — e um
/// segundo caminho de tema só para esta linha seria a porta que diverge das outras três.
const OUTLINE: Color = Color::from_rgba8(120, 200, 255, 170);

/// **Desenha o contorno de `world`** — caminhos já em coordenadas de MUNDO — através de `camera`.
///
/// ⚠️ **Ele recebe o que se DESENHA, não um id.** Quem resolve *"que geometria é esta forma neste
/// quadro"* é a shell, e ela tem de responder o mesmo que o clique responde: para uma forma comum
/// é a entrada do mapa vivo (o offset/pattern dela); para um operando ABSORVIDO por uma booleana
/// viva o mapa vivo está **vazio**, e o contorno certo é a pegada PRÓPRIA dela — que é exactamente
/// a forma que a linha da Hierarquia nomeia.
pub fn draw_hover_outline(world: &[VecPath], camera: Affine, target: &mut VectorScene) {
    for path in world {
        let screen = camera * super::build::build_bezpath(path);
        if screen.elements().is_empty() {
            continue;
        }
        target.inner_mut().stroke(
            &Stroke::new(OUTLINE_PX),
            Affine::IDENTITY,
            &Brush::Solid(OUTLINE),
            None,
            &screen,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_vec_scene::rectangle;
    use ph2d_vector::Shape;

    /// **O contorno atravessa a CÂMARA, e a caneta não.**
    ///
    /// ⚠️ É a lei que o marquee escreve e que já transformou um realce noutro módulo num borrão:
    /// sob o afim mundo→tela, o `stroke` do Vello **multiplica** a largura. Este gate mede o
    /// efeito observável — a geometria move-se com o zoom, e o desenho continua a sair — em vez de
    /// afirmar a intenção.
    #[test]
    fn the_outline_travels_through_the_camera_and_the_pen_does_not() {
        let r: VecPath = rectangle([0.0, 0.0], [10.0, 10.0]);
        let a = super::super::build::build_bezpath(&r);
        let zoomed = Affine::scale(4.0) * a.clone();
        assert!(
            zoomed.bounding_box().width() > a.bounding_box().width() * 3.9,
            "a geometria não atravessou a câmara"
        );
        // E a caneta é sempre a mesma: a constante é de TELA, não de mundo.
        assert!((OUTLINE_PX - 1.5).abs() < f64::EPSILON);
    }

    /// **Um caminho vazio não desenha nada** — e o `continue` é o que impede um `stroke` de um
    /// `BezPath` sem elementos, que o Vello aceita e que custa um comando por quadro.
    #[test]
    fn an_empty_path_draws_nothing() {
        let mut scene = VectorScene::new();
        draw_hover_outline(&[VecPath::default()], Affine::IDENTITY, &mut scene);
        // O oráculo é o próprio `build_bezpath`: sem vértices ele não emite elementos.
        assert!(
            super::super::build::build_bezpath(&VecPath::default())
                .elements()
                .is_empty()
        );
    }
}

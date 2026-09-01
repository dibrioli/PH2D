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
    stroke_all(world, camera, target, OUTLINE, OUTLINE_PX);
}

/// A espessura do realce do **Trim**, em px de TELA. Mais grossa que o contorno de proveniência de
/// propósito: aquele responde *"o que é isto"*, este responde *"isto vai SUMIR"*.
const TRIM_PX: f64 = 3.0;

/// A tinta do pedaço que o Trim vai apagar. ⚠️ **Vermelha, como no Fusion** — e a mesma nota do
/// [`OUTLINE`] vale: os overlays desta crate desenham em tinta própria porque não alcançam o tema.
const TRIM: Color = Color::from_rgba8(255, 82, 82, 220);

/// ⭐⭐⭐ **Desenha o PEDAÇO que o Trim vai apagar** (plano 38), já em coordenadas de MUNDO.
///
/// ⚠️ **A geometria vem da MESMA porta que o corte usa** (`trim_tool::piece_geometry`, o
/// complemento exacto do `sever`) — construí-la aqui por outra conta acenderia uma coisa e apagaria
/// outra, que é o pior defeito possível numa ferramenta destrutiva.
pub fn draw_trim_piece(world: &[VecPath], camera: Affine, target: &mut VectorScene) {
    stroke_all(world, camera, target, TRIM, TRIM_PX);
}

/// A espessura do contorno da face que o Balde vai preencher, em px de TELA.
const BUCKET_PX: f64 = 2.0;

/// Quanto da tinta do artista o realce mostra — o resto é transparência.
///
/// ⚠️ **Não é a tinta CHEIA**: com ela o realce e o resultado ficariam indistinguíveis, e o artista
/// não saberia se já preencheu. ⛔ Nem é uma cor NEUTRA: o balde deposita a tinta corrente, e um
/// realce noutra cor prometeria uma coisa e entregaria outra.
const BUCKET_ALPHA: f64 = 0.55;

/// ⭐⭐⭐ **Desenha a FACE que o Balde vai preencher** (plano 40), já em MUNDO, com a tinta que ele
/// vai depositar.
///
/// ⚠️ **A geometria vem da MESMA porta que o preenchimento usa** (`Rede::geometria`): uma segunda
/// conta aqui acenderia uma região e depositaria outra — a divergência que o Trim já documenta.
///
/// ⚠️ **É um PREENCHIMENTO, e não um contorno** (ao contrário dos dois acima): a pergunta que ele
/// responde é *"que ÁREA vai ficar pintada"*, e um contorno responderia *"que linha é esta"*.
pub fn draw_bucket_face(world: &VecPath, tinta: [u8; 4], camera: Affine, target: &mut VectorScene) {
    let screen = camera * super::build::build_bezpath(world);
    if screen.elements().is_empty() {
        return;
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let a = (f64::from(tinta[3]) * BUCKET_ALPHA) as u8;
    let cor = Color::from_rgba8(tinta[0], tinta[1], tinta[2], a);
    target.inner_mut().fill(
        ph2d_vector::Fill::NonZero,
        Affine::IDENTITY,
        &Brush::Solid(cor),
        None,
        &screen,
    );
    target.inner_mut().stroke(
        &Stroke::new(BUCKET_PX),
        Affine::IDENTITY,
        &Brush::Solid(Color::from_rgba8(tinta[0], tinta[1], tinta[2], 255)),
        None,
        &screen,
    );
}

/// O traçado partilhado dos dois realces — px de TELA sob `Affine::IDENTITY` (a lei do cabeçalho).
fn stroke_all(world: &[VecPath], camera: Affine, target: &mut VectorScene, tinta: Color, px: f64) {
    for path in world {
        let screen = camera * super::build::build_bezpath(path);
        if screen.elements().is_empty() {
            continue;
        }
        target.inner_mut().stroke(
            &Stroke::new(px),
            Affine::IDENTITY,
            &Brush::Solid(tinta),
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

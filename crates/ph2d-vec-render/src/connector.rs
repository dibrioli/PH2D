//! **As alças de ponta do conector** — os dois círculos que dizem onde a linha encosta.
//!
//! # O raio mora AQUI, e o hit-test importa daqui
//!
//! O círculo é desenhado num raio e agarrado noutro: se os dois números divergirem, o usuário
//! clica exatamente no meio da bolinha e não pega nada — e não há sintoma mais enlouquecedor,
//! porque a tela está certa. É o mesmo tropeço do tempo remapeado
//! (`feedback_derived_coordinate_seed_must_match_sample`), na versão de dois pixels: **quem
//! desenha e quem agarra leem a MESMA constante.**
//!
//! # Verde e azul não são decoração
//!
//! É a convenção do draw.io e do Visio, e ela carrega informação que não cabe em outro lugar:
//! **verde** = a ponta escolhe o lado sozinha (ela se re-decide quando a outra forma se move);
//! **azul** = o usuário a pregou naquele ponto. Sem a distinção não há como saber, olhando, se
//! aquela linha ainda vai se reorganizar sozinha ou não.

use ph2d_vector::{Affine, Brush, Circle, Color, Fill, Point, Stroke, VectorScene};

/// O raio do círculo da alça, em PIXELS de tela. Em pixels e não em mundo: uma alça que
/// encolhesse com o zoom-out ficaria impegável exatamente quando o diagrama fica grande.
pub const HANDLE_R_PX: f64 = 7.0;

/// A ponta **automática** (o lado é re-escolhido a cada frame conforme a outra forma se move).
const FLOATING: Color = Color::from_rgba8(90, 210, 130, 255);
/// A ponta **fixada** pelo usuário num ponto da forma.
const PINNED: Color = Color::from_rgba8(90, 160, 240, 255);
const RING: Color = Color::from_rgba8(255, 255, 255, 255);

/// Desenha as alças. `points` = `(ponto de MUNDO, está fixada?)`; `transform` leva mundo→tela.
///
/// O círculo é desenhado em espaço de TELA (raio constante em pixels), então o ponto sobe pelo
/// afim e o raio não.
pub fn draw_connector_handles(
    points: &[([f64; 2], bool)],
    transform: Affine,
    target: &mut VectorScene,
) {
    for &(w, pinned) in points {
        let p = transform * Point::new(w[0], w[1]);
        let c = Circle::new(p, HANDLE_R_PX);
        target.inner_mut().fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(if pinned { PINNED } else { FLOATING }),
            None,
            &c,
        );
        // O anel branco destaca a alça de qualquer fundo — a linha do conector passa por baixo
        // dela, e sem o anel a bolinha some sobre um traço claro.
        target.inner_mut().stroke(
            &Stroke::new(1.5),
            Affine::IDENTITY,
            &Brush::Solid(RING),
            None,
            &c,
        );
    }
}

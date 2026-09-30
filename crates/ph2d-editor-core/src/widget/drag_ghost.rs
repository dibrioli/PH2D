//! ⭐⭐ **O FANTASMA de um cartão arrastado** (ordem do dono, 2026-09-30: *«Ao arrastar um card
//! para organizar o painel, permita ver o card sendo arrastado, menor e meio transparente»*).
//!
//! Quem arrasta (o plano do Inspector, para uma secção; o mesmo plano, para uma nota) pinta o
//! conteúdo do cartão numa cena À PARTE — `conteudo` —, pousa-a no sítio de sempre e entrega-a
//! aqui uma segunda vez. O fantasma é essa mesma cena, com o fundo do cartão por baixo, encolhida
//! [`ph2d_tokens::DRAG_GHOST_SCALE`] à volta do ponto por onde a mão pegou e composta a
//! [`ph2d_tokens::DRAG_GHOST_ALPHA`].
//!
//! ⚠️ **O ponto por onde a mão pegou fica DEBAIXO DO CURSOR** — escalar à volta do canto (ou do
//! centro) faria o cartão fugir da mão logo no primeiro pixel do arrasto.
//!
//! ⛔ **Ele não regista rectângulo nenhum no `HitIndex`**: o que segue o cursor não é clicável —
//! taparia sempre o alvo que a mão procura (a mesma lei do fantasma de um recurso arrastado).

use crate::paint::fill_rounded_rect;
use crate::zones::Rect;
use ph2d_vector::{Affine, Color, Compose, Mix, RoundedRect, VectorScene, VelloBlend};

/// O afim que leva o cartão ao fantasma: escala `s` à volta de `pega`, e `pega` → `cursor`.
#[must_use]
pub fn afim_do_fantasma(pega: (f32, f32), cursor: (f32, f32)) -> Affine {
    let s = f64::from(ph2d_tokens::DRAG_GHOST_SCALE);
    let (gx, gy) = (f64::from(pega.0), f64::from(pega.1));
    let (cx, cy) = (f64::from(cursor.0), f64::from(cursor.1));
    Affine::translate((cx - s * gx, cy - s * gy)) * Affine::scale(s)
}

/// ⭐ Pinta o fantasma de um cartão: o fundo `cor` no rect `cartao` (raio `raio`) com o
/// `conteudo` por cima, pelo [`afim_do_fantasma`] e meio transparente. `cor: None` quando o
/// conteúdo já traz o próprio fundo (uma nota pinta o dela).
pub fn paint_card_ghost(
    scene: &mut VectorScene,
    conteudo: &VectorScene,
    cartao: Rect,
    raio: f32,
    cor: Option<Color>,
    pega: (f32, f32),
    cursor: (f32, f32),
) {
    let afim = afim_do_fantasma(pega, cursor);
    let mut ghost = VectorScene::new();
    if let Some(cor) = cor {
        fill_rounded_rect(&mut ghost, cartao, raio, cor);
    }
    ghost.inner_mut().append(conteudo.inner(), None);
    let forma = RoundedRect::new(
        f64::from(cartao.x),
        f64::from(cartao.y),
        f64::from(cartao.x + cartao.w),
        f64::from(cartao.y + cartao.h),
        f64::from(raio),
    );
    let normal = VelloBlend::new(Mix::Normal, Compose::SrcOver);
    scene.push_layer_shape(normal, ph2d_tokens::DRAG_GHOST_ALPHA, afim, &forma);
    scene.inner_mut().append(ghost.inner(), Some(afim));
    scene.pop_layer();
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⭐⭐ **O ponto por onde a mão pegou fica debaixo do cursor, e o resto encolhe.** *Mutação:
    /// escalar à volta da origem ⇒ o ponto pegado foge do cursor.*
    #[test]
    fn o_ponto_pegado_fica_debaixo_do_cursor_e_o_cartao_encolhe() {
        let a = afim_do_fantasma((100.0, 200.0), (130.0, 90.0));
        let p = a * ph2d_vector::Point::new(100.0, 200.0);
        assert!((p.x - 130.0).abs() < 1e-9 && (p.y - 90.0).abs() < 1e-9);
        let q = a * ph2d_vector::Point::new(140.0, 200.0);
        let largura = q.x - p.x;
        assert!(
            (largura - 40.0 * f64::from(ph2d_tokens::DRAG_GHOST_SCALE)).abs() < 1e-9,
            "o fantasma não encolheu: {largura}"
        );
        const {
            assert!(ph2d_tokens::DRAG_GHOST_SCALE < 1.0);
            assert!(ph2d_tokens::DRAG_GHOST_ALPHA < 1.0 && ph2d_tokens::DRAG_GHOST_ALPHA > 0.0);
        }
    }

    /// Pintar um fantasma acrescenta desenho à cena (a camada e o conteúdo).
    #[test]
    fn o_fantasma_pinta_alguma_coisa() {
        let mut scene = VectorScene::new();
        let mut conteudo = VectorScene::new();
        fill_rounded_rect(
            &mut conteudo,
            Rect::new(10.0, 10.0, 20.0, 20.0),
            2.0,
            Color::from_rgba8(0xFF, 0, 0, 0xFF), // LITERAL-COLOR-OK: fixture
        );
        let antes = scene.inner().encoding().n_paths;
        paint_card_ghost(
            &mut scene,
            &conteudo,
            Rect::new(0.0, 0.0, 100.0, 50.0),
            4.0,
            Some(Color::from_rgba8(0x20, 0x20, 0x20, 0xFF)), // LITERAL-COLOR-OK: fixture
            (10.0, 10.0),
            (50.0, 50.0),
        );
        assert!(scene.inner().encoding().n_paths > antes);
    }
}

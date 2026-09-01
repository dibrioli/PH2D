//! ⭐⭐⭐ **A MARCA DO NÓ SOLDADO** — o anel que diz *"estas pontas são UMA"*.
//!
//! # O report que isto existe para responder (Enio, 2026-09-01)
//!
//! *"ainda não consegue conectar as duas curvas … mas as linhas não compartilham o mesmo nó"*.
//!
//! ⚠️⚠️ **Ele não tinha como VER.** Uma coordenada partilhada não se desenha: duas pontas no mesmo
//! sítio e duas pontas a um pixel de distância pintam o mesmo quadradinho. O único instrumento era
//! **arrastar e observar**, que é um teste destrutivo para responder a uma pergunta de leitura.
//!
//! # A figura, e por que é uma figura e não uma cor
//!
//! É o vocabulário que a própria crate já usa nas guias de encaixe
//! ([`super::guides`]): *afirmações diferentes recebem FIGURAS diferentes, nunca só um tom
//! diferente*. As âncoras são **quadrados** (brancos, laranja no conjunto, ciano escolhidas) — o
//! anel é a única forma que não colide com nenhuma delas, e é a marca de coincidência do desenho de
//! CAD.
//!
//! ⛔ **Ele não é agarrável e não é estado**: é a leitura de um FACTO da geometria (duas pontas na
//! mesma coordenada), recomputado por quadro. Não há o que sincronizar.

use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::{Affine, BezPath, Brush, Color as VelloColor, Point, Stroke, VectorScene};

/// Raio do anel, em pixels de TELA.
///
/// ⚠️ **DERIVADO da âncora que ele abraça:** o quadradinho de uma âncora escolhida tem meia-aresta
/// `4,5` px, logo o círculo circunscrito a ele mede `4,5·√2 ≈ 6,36`. O anel fica **fora** disso —
/// senão ele passa por dentro do quadrado e lê-se como parte dele.
const RING_PX: f64 = 7.5;

/// Espessura do anel, em pixels de TELA.
///
/// ⚠️ Em espaço de tela: no Vello o transform de um `stroke` MULTIPLICA a largura.
const RING_STROKE_PX: f64 = 1.6;

#[cfg(test)]
thread_local! {
    /// **Quantos anéis este passe desenhou** — o oráculo do gate, pelo precedente do
    /// `PICKED_DRAWN` dos overlays: uma `VectorScene` é um buffer de comandos do Vello e não se
    /// deixa perguntar *"que círculos há aqui?"*.
    pub(crate) static MARKS_DRAWN: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Desenha um anel em cada nó partilhado. `nodes` já vem em **pixels de tela** — quem os transforma
/// é o chamador, com a MESMA composição que põe as âncoras no sítio
/// ([`super::overlay_transform`]).
///
/// ⚠️ **É por isso que este módulo não recebe câmera nem poses:** um segundo caminho de projeção
/// aqui poria a marca ao lado da âncora que ela afirma abraçar, e a afirmação seria falsa
/// exactamente nos casos (auto layout) em que ela é mais difícil de conferir a olho.
pub fn draw_weld_marks(nodes: &[[f64; 2]], theme: Theme, target: &mut VectorScene) {
    if nodes.is_empty() {
        return;
    }
    let c = ColorToken::Success.resolve(theme);
    let cor = VelloColor::from_rgba8(c.r, c.g, c.b, c.a);
    let pena = Stroke::new(RING_STROKE_PX);
    for n in nodes {
        let p = Point::new(n[0], n[1]);
        let mut anel = BezPath::new();
        anel.move_to(Point::new(p.x + RING_PX, p.y));
        // Quatro arcos de círculo pelo `kappa` da aproximação cúbica — a mesma forma que o
        // `ellipse` do documento usa, e que o Vello desenha sem aliasing de polígono.
        let k = 0.552_284_749_83 * RING_PX;
        anel.curve_to(
            Point::new(p.x + RING_PX, p.y + k),
            Point::new(p.x + k, p.y + RING_PX),
            Point::new(p.x, p.y + RING_PX),
        );
        anel.curve_to(
            Point::new(p.x - k, p.y + RING_PX),
            Point::new(p.x - RING_PX, p.y + k),
            Point::new(p.x - RING_PX, p.y),
        );
        anel.curve_to(
            Point::new(p.x - RING_PX, p.y - k),
            Point::new(p.x - k, p.y - RING_PX),
            Point::new(p.x, p.y - RING_PX),
        );
        anel.curve_to(
            Point::new(p.x + k, p.y - RING_PX),
            Point::new(p.x + RING_PX, p.y - k),
            Point::new(p.x + RING_PX, p.y),
        );
        anel.close_path();
        target
            .inner_mut()
            .stroke(&pena, Affine::IDENTITY, &Brush::Solid(cor), None, &anel);
        #[cfg(test)]
        MARKS_DRAWN.with(|c| c.set(c.get() + 1));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn marcas(f: impl FnOnce(&mut VectorScene)) -> usize {
        MARKS_DRAWN.with(|c| c.set(0));
        let mut s = VectorScene::new();
        f(&mut s);
        MARKS_DRAWN.with(std::cell::Cell::get)
    }

    /// **Um anel por nó** — e nenhum quando não há nó nenhum.
    #[test]
    fn one_ring_per_shared_node_and_none_without() {
        assert_eq!(marcas(|s| draw_weld_marks(&[], Theme::default(), s)), 0);
        assert_eq!(
            marcas(|s| draw_weld_marks(&[[10.0, 10.0], [80.0, 40.0]], Theme::default(), s)),
            2
        );
    }
}

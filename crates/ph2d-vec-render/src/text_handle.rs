//! Desenho da **alça do texto em caminho** (plano 22, W5) — módulo irmão do overlay do Envelope
//! (`envelope.rs`), pelo teto de LOC.
//!
//! Uma bolinha, onde o texto **começa** no caminho-guia (o `start_offset`). Arrastá-la no modo
//! Node corre o texto ao longo da curva — é a versão de manipulação direta do slider de Offset,
//! e as duas escrevem o MESMO `start_offset` (o host garante isso; ver `vec_text_ride::handle`).
//!
//! **Vazada** = em repouso; **cheia** = é a alça que o dedo tem agora. É a gramática do
//! `envelope.rs` e do `connector.rs`: a forma e o preenchimento carregam o estado, para lê-lo
//! sem clicar.
//!
//! Como as irmãs, o raio é uma constante que o HIT-TEST do host também lê (em mundo, `×
//! px_to_world`) — dois números fariam o dedo pegar num sítio e a bolinha acender noutro; aqui a
//! constante mora no host (`vec_text_ride::HANDLE_R_PX`) porque é ele que decide o alcance do
//! gesto, e o desenho recebe o ponto já pronto. O desenho é em espaço de TELA: o ponto sobe pelo
//! afim da câmera, o raio não cresce com o zoom.

use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::{Affine, Brush, Circle, Color as VelloColor, Fill, Point, Stroke, VectorScene};

/// Raio da bolinha, em pixels de tela. Espelha o `vec_text_ride::HANDLE_R_PX` do host — o mesmo
/// número dos dois lados, senão o desenho e o hit-test discordariam sobre o tamanho da alça.
const HANDLE_R_PX: f64 = 7.0;

/// Espessura do contorno da bolinha, em pixels.
const LINE_PX: f64 = 1.5;

/// Desenha a alça no ponto de MUNDO `at` (o host o leu do `start_offset` do texto).
///
/// `dragging` deixa a bolinha **cheia** enquanto o dedo a segura, e **vazada** em repouso — o
/// mesmo estado-na-forma das outras alças de nó.
pub fn draw_text_handle(
    at: [f64; 2],
    dragging: bool,
    transform: Affine,
    theme: Theme,
    target: &mut VectorScene,
) {
    let vello = |t: ColorToken| {
        let c = t.resolve(theme);
        VelloColor::from_rgba8(c.r, c.g, c.b, c.a)
    };
    let (fill, edge) = (vello(ColorToken::Accent), vello(ColorToken::BorderEmph));

    let dot = Circle::new(transform * Point::new(at[0], at[1]), HANDLE_R_PX);
    if dragging {
        target.inner_mut().fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(fill),
            None,
            &dot,
        );
    }
    target.inner_mut().stroke(
        &Stroke::new(LINE_PX),
        Affine::IDENTITY,
        &Brush::Solid(edge),
        None,
        &dot,
    );
}

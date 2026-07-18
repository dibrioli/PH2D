//! Desenho da **gaiola do Envelope** (ADR-0129, Fatia 1) — módulo irmão do overlay de alças de
//! raio (`corner.rs`), LOC cap.
//!
//! Os 4 cantos da gaiola (`ph2d_ecs::VecEnvelope::corners`, ordem `[BL, BR, TR, TL]`), ligados num
//! quadrilátero, com uma bolinha por canto. Arrastar um canto no modo Node deforma a forma que a
//! gaiola contém — a homografia é re-cozida a cada frame (`shells/desktop/src/envelope_live`).
//!
//! **Vazada** = o canto está em repouso ou parado; **cheia** = é o canto que o dedo tem agora. É a
//! mesma gramática do `corner.rs` e do `connector.rs`: a forma e o preenchimento carregam o estado,
//! para lê-lo sem clicar.
//!
//! Como no `corner.rs`, o raio da bolinha é uma constante única ([`ENVELOPE_HANDLE_R_PX`]) que o
//! HIT-TEST do host também lê (em unidades de mundo, `× px_to_world`): dois números fariam o dedo
//! pegar um canto e a bolinha acender noutro. E o desenho é em espaço de TELA — o ponto sobe pelo
//! afim da câmera, o raio não cresce com o zoom.

use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::{
    Affine, BezPath, Brush, Circle, Color as VelloColor, Fill, Point, Stroke, VectorScene,
};

/// Raio da bolinha do canto, em pixels — e o mesmo alcance que o hit-test do host usa (`×
/// px_to_world`). Igual ao da alça de raio de quina (`CORNER_HANDLE_R_PX`), para as duas alças de
/// nó pesarem o mesmo na mão.
pub const ENVELOPE_HANDLE_R_PX: f64 = 6.0;

/// Espessura da aresta da gaiola e do contorno da bolinha, em pixels.
const LINE_PX: f64 = 1.5;

/// A gaiola a desenhar neste frame: os 4 cantos em MUNDO (o host os leu do `VecEnvelope`) e qual
/// deles está sendo arrastado (`Some(idx)` ⇒ bolinha cheia).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EnvelopeCageView {
    /// Os 4 cantos em coordenadas de MUNDO, ordem `[BL, BR, TR, TL]`.
    pub corners: [[f64; 2]; 4],
    /// O índice do canto sob arrasto agora, se houver — o único que se desenha cheio.
    pub dragging: Option<usize>,
}

/// Desenha a gaiola do envelope. `transform` leva mundo→tela; a bolinha é pintada em espaço de tela
/// (raio em px), então o ponto sobe pelo afim e o raio não.
pub fn draw_envelope_cage(
    cage: &EnvelopeCageView,
    transform: Affine,
    theme: Theme,
    target: &mut VectorScene,
) {
    // O token resolve num RGBA de 8 bits; o Vello quer o `Color` dele — a mesma ponte do `corner.rs`
    // (não reusável: este crate não depende do editor-core, e a pipeline de render não conhece o chrome).
    let vello = |t: ColorToken| {
        let c = t.resolve(theme);
        VelloColor::from_rgba8(c.r, c.g, c.b, c.a)
    };
    let (fill, edge, wire) = (
        vello(ColorToken::Accent),
        vello(ColorToken::BorderEmph),
        vello(ColorToken::AccentSoft),
    );

    // A moldura da gaiola: BL→BR→TR→TL→BL, fechada. Diz onde a forma está sendo deformada e a que
    // canto cada bolinha pertence.
    let pts: [Point; 4] =
        std::array::from_fn(|i| transform * Point::new(cage.corners[i][0], cage.corners[i][1]));
    let mut frame = BezPath::new();
    frame.move_to(pts[0]);
    for p in &pts[1..] {
        frame.line_to(*p);
    }
    frame.close_path();
    target.inner_mut().stroke(
        &Stroke::new(LINE_PX),
        Affine::IDENTITY,
        &Brush::Solid(wire),
        None,
        &frame,
    );

    for (i, p) in pts.iter().enumerate() {
        let dot = Circle::new(*p, ENVELOPE_HANDLE_R_PX);
        if cage.dragging == Some(i) {
            target.inner_mut().fill(
                Fill::NonZero,
                Affine::IDENTITY,
                &Brush::Solid(fill),
                None,
                &dot,
            );
        }
        // O contorno vem sempre: destaca a bolinha de qualquer fundo, e é o corpo inteiro dela
        // enquanto o canto está parado (vazada).
        target.inner_mut().stroke(
            &Stroke::new(LINE_PX),
            Affine::IDENTITY,
            &Brush::Solid(edge),
            None,
            &dot,
        );
    }
}

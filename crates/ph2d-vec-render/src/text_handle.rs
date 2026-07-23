//! Desenho da **alça do texto em caminho** (plano 22, W5) — módulo irmão do overlay do Envelope
//! (`envelope.rs`), pelo teto de LOC.
//!
//! Uma bolinha, onde o texto **começa** no caminho-guia (o `start_offset`). Arrastá-la no modo
//! **Select** corre o texto ao longo da curva — é a versão de manipulação direta do slider de
//! Offset, e as duas escrevem o MESMO `start_offset` (o host garante isso; ver
//! `vec_text_ride::handle`).
//!
//! # Grande, sólida e âmbar — e por quê (Enio, smoke ×2)
//!
//! ⚠️ **Duas correções, dois smokes.** A 1ª versão era pequena (7 px), VAZADA e no modo Node — e
//! ali se confundia com as âncoras dos outros paths (anéis pequenos). A 2ª virou disco sólido no
//! Select, mas **preenchido de `Accent`** — e o `Accent` deste tema é o MESMO rosa/magenta do
//! gizmo de sprite, então a ficha se perdia entre os quadradinhos da caixa de transformação.
//!
//! Agora é **âmbar** (`Warn`): distinto do rosa do gizmo E do azul do texto, sem depender do
//! matiz do accent (que varia por tema). Não é semântica de aviso — âmbar é a cor clássica de
//! *alça que se agarra*, e é o que a separa de tudo o mais na tela. Disco SÓLIDO grande (10 px vs
//! 6 das âncoras), com anel de contraste — lê como a alça primária de uma relação, não como um
//! ponto de geometria.
//!
//! **Cheia sempre**, e mais escura durante o arrasto (`WarnSoft`→`Warn`): o estado no
//! preenchimento, como as irmãs, mas o corpo nunca some — ela é o objeto que o artista agarra.
//!
//! O raio espelha o `vec_text_ride::HANDLE_R_PX` do host (o hit-test lê o dele, `× px_to_world`);
//! dois números fariam o dedo pegar num sítio e a bolinha acender noutro. O desenho é em espaço de
//! TELA: o ponto sobe pelo afim da câmera, o raio não cresce com o zoom.

use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::{Affine, Brush, Circle, Color as VelloColor, Fill, Point, Stroke, VectorScene};

/// Raio da bolinha, em pixels de tela. Espelha o `vec_text_ride::HANDLE_R_PX` do host — o mesmo
/// número dos dois lados, senão o desenho e o hit-test discordariam sobre o tamanho da alça.
/// **Grande de propósito** (as âncoras/cantos são 6): a alça de uma relação não é um ponto de
/// geometria, e o tamanho é metade do que a separa deles. 15 px = 50% maior que o 10 inicial,
/// a pedido do Enio (smoke).
const HANDLE_R_PX: f64 = 15.0;

/// Espessura do anel de contraste, em pixels — um pouco mais grosso que o das alças de nó, para a
/// ficha ler como sólida a qualquer zoom.
const LINE_PX: f64 = 2.0;

/// Desenha a alça no ponto de MUNDO `at` (o host o leu do `start_offset` do texto).
///
/// **Sempre preenchida** (é a ficha que o artista agarra); `dragging` só a escurece
/// (`AccentPress`), o estado no tom e não no corpo.
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
    // O corpo é **âmbar** (`Warn`) — distinto do rosa do gizmo e do azul do texto; sob o dedo
    // escurece para `WarnSoft`. O anel é `BorderEmph` (o mesmo contorno das alças irmãs), que
    // contrasta com o âmbar em qualquer tema.
    let body = vello(if dragging {
        ColorToken::WarnSoft
    } else {
        ColorToken::Warn
    });
    let ring = vello(ColorToken::BorderEmph);

    let dot = Circle::new(transform * Point::new(at[0], at[1]), HANDLE_R_PX);
    target.inner_mut().fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &Brush::Solid(body),
        None,
        &dot,
    );
    target.inner_mut().stroke(
        &Stroke::new(LINE_PX),
        Affine::IDENTITY,
        &Brush::Solid(ring),
        None,
        &dot,
    );
}

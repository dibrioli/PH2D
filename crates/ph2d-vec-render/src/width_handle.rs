//! Desenho da **alça de largura** (plano 25 §5) — módulo irmão do [`super::text_handle`].
//!
//! Um **braço**: a ficha agarrável fica **SOBRE a curva**, no ponto da parada, e uma haste fina
//! sai dela até a borda da fita, mostrando a largura que aquela parada dita.
//!
//! # Por que a ficha está na curva, e não na borda da fita
//!
//! ⚠️ **Report do Enio (2026-07-30), com medição.** A 1ª versão punha a ficha **na borda da
//! fita** — "onde a tinta acaba", a manipulação direta da largura. Mas a borda é `meia-largura ×
//! multiplicador` fora da curva, e com o multiplicador alto isso **atravessa a vizinhança**: num
//! grampo de braços a `0,30` de distância, um arrasto que produziu multiplicador `3,75` sobre um
//! traço de `0,16` pôs a ficha em `y = 0,30` — **exatamente sobre o outro braço**. O artista
//! clicava numa linha e a alça nascia na linha de ao lado; clicava de novo, e ficava com duas
//! alças, uma em cada segmento. É isso que ele reportou.
//!
//! Com a ficha **na curva** a pergunta *"de que linha é esta alça?"* deixa de ter resposta errada
//! possível: ela está sobre a sua própria linha, por construção, e um clique entre duas linhas
//! próximas cria uma parada só — na mais próxima do rato, que é o que ele pediu. É também o que
//! o *Width Tool* do Illustrator faz (o ponto de largura senta no traçado) e o que os nós do
//! *Power Stroke* do Inkscape fazem.
//!
//! A largura continua **diretamente manipulável**: arrastar para longe da curva cresce a haste e
//! engrossa a fita — o mesmo gesto, com a ficha noutro sítio.

use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::{Affine, BezPath, Brush, Color as VelloColor, Point, Stroke, VectorScene};

/// Espessura da haste, em pixels de TELA.
///
/// ⚠️ **A haste é desenhada em espaço de tela** (os dois pontos sobem pelo afim da câmera, a
/// espessura não): no Vello o transform de um `stroke` MULTIPLICA a largura, e foi isso que já
/// transformou o realce do Flip num borrão e a borda do véu do Shape Builder em 150 px de tinta.
const STEM_PX: f64 = 1.5;

/// Desenha o braço da parada: a haste `at → tip` e a ficha agarrável em `at` (sobre a curva).
///
/// `tip` é a borda da fita — o que a parada MEDE. `at` é o que a mão agarra.
pub fn draw_width_handle(
    at: [f64; 2],
    tip: [f64; 2],
    dragging: bool,
    transform: Affine,
    theme: Theme,
    target: &mut VectorScene,
) {
    let c = ColorToken::Warn.resolve(theme);
    let stem = VelloColor::from_rgba8(c.r, c.g, c.b, c.a);
    let a = transform * Point::new(at[0], at[1]);
    let b = transform * Point::new(tip[0], tip[1]);
    let mut arm = BezPath::new();
    arm.move_to(a);
    arm.line_to(b);
    target.inner_mut().stroke(
        &Stroke::new(STEM_PX),
        Affine::IDENTITY,
        &Brush::Solid(stem),
        None,
        &arm,
    );
    // A ficha é a MESMA do texto e do pattern: uma alça arrastável deste módulo tem uma aparência
    // só, e o que a distingue é ONDE ela está.
    super::draw_text_handle(at, dragging, transform, theme, target);
}

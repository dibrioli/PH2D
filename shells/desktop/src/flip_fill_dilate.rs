//! **A ponte entre o DOCUMENTO e a dilatação do preenchimento** — módulo irmão do
//! `flip_fill.rs` (teto de LOC do shell, 600).
//!
//! A **lei** da dilatação (que largura o contorno veste) mora em
//! `ph2d_flip_fill::dilate` desde 2026-07-18, e não aqui. O motivo está documentado lá:
//! enquanto ela morava no shell, o oráculo de pixel que a verifica (`gpu_fill_fit`, na
//! crate de render) **não conseguia alcançá-la** e montava a própria cópia — oito gates
//! ficaram verdes durante o BUGS #20, incluindo um chamado *"a cor nunca transborda para
//! fora da linha"*, enquanto o produto transbordava 100×.
//!
//! O que sobra aqui é o que **precisa do tipo do documento**: transformar `FlipDrawing`
//! na lista de fronteiras, e transformar as larguras num `FlipStroke`.

use ph2d_core::Vec2;
use ph2d_flip::{Fill, FlipDrawing, FlipStroke, Point, Rgba};

/// As linhas que delimitam o preenchimento: TODOS os traços do desenho que não são,
/// eles próprios, um preenchimento sem contorno.
///
/// Um fill anterior (`hide_stroke`) **não** é fronteira — senão a 2ª cor não conseguiria
/// entrar por baixo da 1ª. Mas um fechamento de gap persistente (que também é
/// `hide_stroke`) **é** — é exatamente para isso que ele existe. Os dois se distinguem
/// pelo `fill`: o preenchimento tem cor; o fechamento não.
///
/// **Esta lista é perguntada DUAS vezes** — o `fill_at` a usa para delimitar a região, e
/// o `contour_widths` para saber que linha cada ponto do contorno veste. É a mesma lista
/// de propósito: a versão anterior re-derivava o conjunto para a 2ª pergunta com um
/// filtro próprio, e os dois só concordavam por acidente (ver o doc do `local_line`).
pub(crate) fn boundaries(drawing: &FlipDrawing) -> Vec<(Vec<Vec2>, Vec<f32>, bool)> {
    drawing
        .strokes
        .iter()
        .filter(|s| !(s.hide_stroke && s.fill.is_some())) // fills anteriores não barram
        .filter(|s| s.len() >= 2)
        .map(|s| {
            let pts = s.positions().to_vec();
            // **A conversão SUMIU em §4.C.6, e some por CURA, não por descuido.**
            //
            // Ela existia porque o documento falava duas línguas: os PONTOS em unidades de
            // mundo e as LARGURAS em px de TELA. Misturar as duas punha uma linha de 3
            // unidades de mundo (≈324 px!) num desenho de 2,8 — o clique caía sempre
            // DENTRO do traço e o balde respondia "clicked on a line", sempre. O `×
            // px_to_world` era o remendo.
            //
            // Agora o Size é uma medida do MUNDO (`size_to_world`), então `width` e `pos`
            // são a mesma unidade e a meia-espessura é só `w/2`. De quebra o balde deixa
            // de depender do ZOOM do clique — o mesmo clique no mesmo lugar responde o
            // mesmo em qualquer aproximação.
            let half: Vec<f32> = s.widths().iter().map(|w| w * 0.5).collect();
            (pts, half, s.closed)
        })
        .collect()
}

/// O traço que materializa a região preenchida.
///
/// **A largura do contorno é a espessura da LINHA** (mais a margem) — e isso não é um
/// contorno de verdade (o `hide_stroke` segue ligado): é a **dilatação da cor por baixo
/// do line-art**, sem a qual a arte não fecha. Quem calcula os números é
/// `ph2d_flip_fill::contour_widths`; aqui eles só viram geometria do documento.
///
/// A geometria do fill termina no **eixo** da linha (BUGS #14 — a única âncora imune ao
/// zoom), e o eixo fica a meia-espessura da silhueta. Sem dilatar, a metade externa da
/// linha não tem cor por baixo: com um pincel MACIO ela mistura com o fundo, e o contorno
/// ganha um halo escuro (o *"o fill não se ajusta à linha"* do smoke).
pub(crate) fn fill_stroke(
    outer: &[Vec2],
    holes: Vec<Vec<Vec2>>,
    color: Rgba,
    opacity: f32,
    widths: &[f32],
) -> FlipStroke {
    let mut s = FlipStroke::new();
    for (i, &p) in outer.iter().enumerate() {
        s.push_point(Point {
            pos: p,
            width: widths.get(i).copied().unwrap_or(0.0),
            opacity: 1.0,
            color,
        });
    }
    s.closed = true;
    s.hide_stroke = true; // não é line-art: é a região (o `is_fill` continua valendo)
    s.holes = holes;
    s.fill = Some(Fill { color, opacity });
    s
}

#[cfg(test)]
#[path = "flip_fill_dilate_tests.rs"]
mod tests;

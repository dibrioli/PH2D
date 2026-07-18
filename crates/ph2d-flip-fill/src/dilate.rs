//! **A DILATAÇÃO do contorno** — com que largura o contorno que o solver devolve é
//! DESENHADO, para a cor entrar por baixo do line-art (BUGS #15).
//!
//! # Por que isto mora aqui, e não no shell
//!
//! Morou no shell até 2026-07-18, e o preço foi **oito oráculos de pixel cegos**: o
//! `gpu_fill_fit` (a suíte que rasteriza a cena e MEDE o encaixe da cor na linha —
//! inclusive um gate chamado *"a cor nunca transborda para fora da linha"*) montava a
//! **própria** dilatação, com uma cópia da constante e uma cópia da fórmula. Os dois
//! números eram os mesmos por acordo tácito, não por construção.
//!
//! O resultado é a assinatura do problema: quando a dilatação do PRODUTO ficou 100×
//! grande demais (BUGS #20 — uma constante em px somada a uma largura em unidades de
//! mundo), **os oito gates continuaram verdes**, porque nenhum deles olhava para o
//! número do produto. O defeito foi achado por um humano olhando a tela.
//!
//! Um oráculo que reconstrói o que deveria verificar não verifica nada: ele afirma que
//! *a sua própria* aritmética é consistente. A lei mora aqui — junto de quem produz o
//! contorno — para que a pergunta *"que largura o contorno veste?"* tenha **uma
//! resposta** e o oráculo tenha de perguntá-la em vez de respondê-la.
//!
//! (Irmão de `feedback_two_doors_to_the_same_question_diverge`.)

use crate::Vec2;

/// Margem da dilatação, em **px de TELA** — a folga que cobre o erro de VETORIZAÇÃO do
/// contorno (marching squares + RDP + alisamento deixam o contorno até ~1,5 px DENTRO
/// do eixo nos picos de tremor).
///
/// **O valor saiu de uma varredura no pixel**, não do olho: dois defeitos OPOSTOS se
/// tocam aqui (`gpu_fill_fit::sweep_tuck`, medido no anel da linha, 256 raios):
///
/// | margem | fundo sob a linha | transbordo além dela |
/// |---|---|---|
/// | 0,0 | **4 px** (o defeito do smoke de 2026-07-13) | 5 |
/// | **0,5** | **0** | **16** |
/// | 1,5 | 0 | 99 |
/// | 2,0 | 0 | 195 |
///
/// `0,5` é o menor valor que zera o vazamento — margem a mais volta a empurrar a cor
/// para FORA da linha, que é exatamente o defeito que matou o `grow = +2` default
/// (BUGS #11).
///
/// ⚠️ **É uma medida de TELA.** Quem a usa num documento que fala unidades de MUNDO
/// tem de atravessar `tuck_world` — somá-la crua a uma largura de mundo foi o BUGS #20
/// ([[feedback_geometry_over_mixed_units_needs_the_consumers_conversion]]).
pub const FILL_TUCK_PX: f32 = 0.5; // LITERAL-PX-OK: erro de vetorizacao do contorno, MEDIDO

/// A margem acima **na unidade do documento**, dado quantos px de tela vale uma unidade
/// de mundo (`ph2d_tool_flip::SIZE_PX_PER_WORLD` no produto; `1.0` numa fixture cujo
/// mundo JÁ é pixel).
///
/// Dobrada porque a largura de um traço é um DIÂMETRO: para a cor avançar `FILL_TUCK_PX`
/// para cada lado do eixo, o traço tem de engordar o dobro disso.
#[must_use]
pub fn tuck_world(px_per_world: f32) -> f32 {
    margin_world(FILL_TUCK_PX, px_per_world)
}

/// A mesma conversão para uma margem **arbitrária**, em px de tela.
///
/// ⚠️ **Existe para a VARREDURA que escolhe a constante** (`gpu_fill_fit::sweep_tuck`),
/// e para mais nada. A varredura precisa perguntar *"e se a margem fosse X?"* — e a
/// única coisa que ela pode variar é a **constante**: se ela também reescrevesse a
/// fórmula, voltaria a medir a própria aritmética em vez da do produto, que foi
/// exatamente como os oito oráculos ficaram cegos. O produto chama `tuck_world`.
#[must_use]
pub fn margin_world(margin_px: f32, px_per_world: f32) -> f32 {
    2.0 * margin_px / px_per_world
}

/// **A linha que este ponto do contorno está vestindo**: `(espessura, distância)`, na
/// unidade do documento.
///
/// O contorno do balde termina no EIXO da linha (BUGS #14), então o line-art mais
/// próximo de um ponto do contorno é, por construção, a linha que ele veste.
///
/// `strokes` é a MESMA lista que o `fill_at` recebeu — `(pontos, MEIA-espessura por
/// ponto, fechado)`. Perguntar a dilatação à mesma lista que delimitou a região não é
/// conveniência: a versão anterior re-derivava o conjunto de linhas do documento com um
/// filtro **próprio** (`!hide_stroke`, contra o `!(hide_stroke && fill.is_some())` das
/// fronteiras), e os dois só concordavam por acidente — um fechamento de gap tem
/// espessura zero, então caía no filtro `w > 0` mais adiante. Acidente enumerado é a
/// forma que um bug futuro toma.
#[must_use]
pub fn local_line(strokes: &[(Vec<Vec2>, Vec<f32>, bool)], p: Vec2) -> Option<(f32, f32)> {
    let mut best: Option<(f32, f32)> = None; // (dist, largura)
    for (pts, half, closed) in strokes {
        // ⚠️ **Distância ao SEGMENTO, nunca ao VÉRTICE.** O eixo é a polilinha, não a
        // nuvem de pontos dela: um ponto do contorno pousado exatamente sobre o eixo,
        // mas no meio de dois vértices, fica a até meia-amostragem do vértice mais
        // próximo. Medir ao vértice faria a compensação pagar o **espaçamento da
        // amostragem** como se fosse erro de vetorização — e num traço de 64 amostras
        // isso é maior que o erro real (BUGS #18).
        for (i, a, b) in segments(pts, *closed) {
            let ab = Vec2::new(b.x - a.x, b.y - a.y);
            let l2 = ab.x * ab.x + ab.y * ab.y;
            let t = if l2 <= 0.0 {
                0.0
            } else {
                (((p.x - a.x) * ab.x + (p.y - a.y) * ab.y) / l2).clamp(0.0, 1.0)
            };
            let (dx, dy) = (p.x - (a.x + t * ab.x), p.y - (a.y + t * ab.y));
            let d = (dx * dx + dy * dy).sqrt();
            // A espessura interpolada ao longo do segmento (a pressão varia por ponto).
            // `half` guarda MEIA espessura (a convenção do `fill_at`); a dilatação veste
            // o diâmetro.
            let n = half.len();
            let (ha, hb) = (
                half.get(i).copied().unwrap_or(0.0),
                half.get((i + 1) % n.max(1)).copied().unwrap_or(0.0),
            );
            let w = 2.0 * (ha + (hb - ha) * t);
            if w > 0.0 && best.is_none_or(|(bd, _)| d < bd) {
                best = Some((d, w));
            }
        }
    }
    best.map(|(d, w)| (w, d))
}

/// **A LEI**, ponto a ponto: a largura com que cada ponto do contorno é desenhado.
///
/// `largura da linha LOCAL + a margem` — a dilatação veste a linha que o contorno abraça
/// NAQUELE ponto, e não a média do desenho: num desenho com espessuras diferentes a
/// média fica entre elas, então onde o contorno abraça a linha FINA a cor saía larga
/// demais e aparecia do outro lado dela (o smoke do Enio, BUGS #20).
///
/// ⚠️ **A margem é FIXA, de propósito, e é a franja que sobra** (smoke do Enio,
/// 2026-07-18: *"o fill está extrapolando um pouquinho para fora da linha, como se a
/// referência não fosse o centro da linha mas a borda externa"*). Ele está certo: a
/// margem dilata o contorno INTEIRO para compensar um erro que só acontece nos **picos
/// de tremor**. Uma compensação por-ponto (`w + 2d`) foi implementada, MEDIDA e
/// **revertida sem shipar**: ela precisa do **sinal** (o contorno caiu para dentro ou
/// para fora do eixo?), e sem ele dobra o erro metade das vezes — 0,0178 na mediana
/// contra 0,005 da margem fixa, ou seja PIOR.
#[must_use]
pub fn contour_widths(
    strokes: &[(Vec<Vec2>, Vec<f32>, bool)],
    contour: &[Vec2],
    px_per_world: f32,
) -> Vec<f32> {
    contour_widths_with_margin(strokes, contour, px_per_world, FILL_TUCK_PX)
}

/// A lei com a margem **parametrizada** — ver o aviso do `margin_world`: isto é para a
/// varredura que ESCOLHE a constante, e o produto usa `contour_widths`.
#[must_use]
pub fn contour_widths_with_margin(
    strokes: &[(Vec<Vec2>, Vec<f32>, bool)],
    contour: &[Vec2],
    px_per_world: f32,
    margin_px: f32,
) -> Vec<f32> {
    let tuck = margin_world(margin_px, px_per_world);
    let fallback = mean_line_width(strokes);
    contour
        .iter()
        .map(|&p| match local_line(strokes, p) {
            Some((w, _)) => w + tuck,
            None => fallback + tuck,
        })
        .collect()
}

/// A espessura MÉDIA do line-art (unidade do documento) — o fallback de um ponto do
/// contorno que não achou linha nenhuma. Ignora os fechamentos de gap (espessura zero).
#[must_use]
pub fn mean_line_width(strokes: &[(Vec<Vec2>, Vec<f32>, bool)]) -> f32 {
    let (sum, n) = strokes
        .iter()
        .flat_map(|(_, half, _)| half.iter().copied())
        .filter(|h| *h > 0.0)
        .fold((0.0f32, 0usize), |(sum, n), h| (sum + 2.0 * h, n + 1));
    if n == 0 { 0.0 } else { sum / n as f32 }
}

/// Os segmentos de uma polilinha — **um traço `closed` inclui a COSTURA** (último →
/// primeiro); um aberto NUNCA a ganha.
///
/// ⚠️ Isto **espelha `ph2d_flip::FlipStroke::segments()`**, que é a porta única daquela
/// pergunta para um traço do documento. Esta crate não conhece o documento (só depende
/// de `ph2d-core`), então o espelho é inevitável — e por isso ele é **pinado por um
/// gate no shell**, que é o único lugar onde os dois tipos coexistem
/// (`flip_fill_dilate` → `the_two_segment_walks_agree`). Espelho sem gate é como a
/// dilatação duplicada do `gpu_fill_fit` nasceu.
fn segments(pts: &[Vec2], closed: bool) -> impl Iterator<Item = (usize, Vec2, Vec2)> + '_ {
    let n = pts.len();
    let seam = closed && n >= 2;
    (0..n.saturating_sub(1))
        .chain(seam.then(|| n - 1))
        .map(move |i| (i, pts[i], pts[(i + 1) % n]))
}

#[cfg(test)]
#[path = "dilate_tests.rs"]
mod tests;

//! **A DILATAÇÃO do preenchimento** — módulo irmão do `flip_fill.rs` (teto de LOC do
//! shell, 600). Um assunto só: *com que largura o contorno da região é desenhado para a
//! cor entrar por baixo do line-art* (BUGS #15), e por que essa largura é **local**
//! (BUGS #20).

use ph2d_core::Vec2;
use ph2d_flip::{Fill, FlipDrawing, FlipStroke, Point, Rgba};

/// Margem da dilatação do fill, em px de tela — a folga que cobre o erro de
/// VETORIZAÇÃO do contorno (marching squares + RDP + alisamento deixam o contorno até
/// ~1,5 px DENTRO do eixo nos picos de tremor).
///
/// **O valor saiu de uma varredura no pixel**, não do olho: dois defeitos OPOSTOS se
/// tocam aqui (`gpu_fill_fit::sweep_tuck`, medido no anel da linha, 256 raios):
///
/// | margem | fundo sob a linha | transbordo além dela |
/// |---|---|---|
/// | 0,0 | **4 px** (o defeito do smoke) | 5 |
/// | **0,5** | **0** | **16** |
/// | 1,5 | 0 | 99 |
/// | 2,0 | 0 | 195 |
///
/// `0,5` é o menor valor que zera o vazamento — margem a mais volta a empurrar a cor
/// para FORA da linha, que é exatamente o defeito que matou o `grow = +2` default
/// (BUGS #11).
pub(crate) const FILL_TUCK_PX: f32 = 0.5; // LITERAL-PX-OK: erro de vetorizacao do contorno, MEDIDO

/// A margem acima, **em unidades de MUNDO** — que é a unidade em que a espessura vive
/// desde o §4.C.6 ("o Size mede o MUNDO", `SIZE_PX_PER_WORLD = 100`).
///
/// ⚠️ **Isto era um bug de UNIDADE, e ele dominava tudo.** O `FILL_TUCK_PX` é medido em
/// PIXELS (a tabela do sweep acima está em px), e estava sendo somado direto a uma
/// largura em unidades de mundo: `2 × 0,5 = 1,0` **unidade de mundo = 100 px** de
/// dilatação espúria, onde se queria 1 px. Num desenho com pincel default (~0,06 de
/// mundo) a margem ficava **17× mais larga que a própria linha** — a cor virava um blob
/// arredondado que ignorava o line-art, e nenhum valor de Gap ou Trap mexia nisso
/// (os dois movem a GEOMETRIA; isto é a dilatação do RENDER).
///
/// Regressão do §4.C.6: a lei nova foi aplicada ao `boundaries()` (de onde o
/// `× px_to_world` sumiu) e esta constante ficou para trás. É o preço documentado de
/// uma medida de TELA sobreviver numa fronteira que passou a falar MUNDO
/// ([[feedback_geometry_over_mixed_units_needs_the_consumers_conversion]]).
///
/// A margem acima, **em unidades de MUNDO**.
///
/// ⚠️ **ABERTO (smoke do Enio, 2026-07-18):** *"o fill está extrapolando um pouquinho
/// para fora da linha, como se a referência não fosse o centro da linha mas a borda
/// externa"*. Ele está certo, e a franja é ESTA margem: ela dilata o contorno INTEIRO
/// para compensar um erro que só acontece nos **picos de tremor** — onde o contorno
/// acerta o eixo, é transbordo puro. A tabela do sweep acima já registrava o preço sem
/// o nomear: `tuck 0,5` zera o fundo vazando **e TRIPLICA** o transbordo (5 → 16).
///
/// **Uma compensação por-ponto (`w + 2d`) foi implementada, MEDIDA e REVERTIDA sem
/// shipar:** ela precisa do **sinal** (o contorno caiu para dentro ou para fora do
/// eixo?), e sem ele dobra o erro metade das vezes — medida, a compensação adaptativa
/// dava **0,0178** na mediana contra **0,005** da margem fixa, ou seja, PIOR. (Ao
/// medi-la apareceu junto um erro de vértice-vs-segmento no cálculo da distância, a
/// lição do BUGS #18; corrigido antes de medir, e a medida acima já é a boa.)
///
/// **O caminho para fechar isto é instrumental, não algorítmico:** o oráculo certo é o
/// PIXEL (`gpu_fill_fit::sweep_tuck`, que escolheu o 0,5 original), e ele hoje **monta a
/// própria dilatação** em vez de consumir a do produto — o gap nomeado no BUGS #20. Fecha
/// o gap primeiro; depois o valor (ou a fórmula) se escolhe com a curva na mão, como o
/// FILL_TUCK_PX original foi escolhido.
pub(crate) fn fill_tuck_world() -> f32 {
    2.0 * FILL_TUCK_PX / ph2d_tool_flip::SIZE_PX_PER_WORLD
}

/// As linhas que delimitam o preenchimento: TODOS os traços do desenho que não são,
/// eles próprios, um preenchimento sem contorno.
///
/// Um fill anterior (`hide_stroke`) **não** é fronteira — senão a 2ª cor não conseguiria
/// entrar por baixo da 1ª. Mas um fechamento de gap persistente (que também é
/// `hide_stroke`) **é** — é exatamente para isso que ele existe. Os dois se distinguem
/// pelo `fill`: o preenchimento tem cor; o fechamento não.
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
/// **A largura do contorno é a espessura da LINHA** — e isso não é um contorno de
/// verdade (o `hide_stroke` segue ligado): é a **dilatação da cor por baixo do
/// line-art**, sem a qual a arte não fecha.
///
/// A geometria do fill termina no **eixo** da linha (BUGS #14 — a única âncora imune
/// ao zoom), e o eixo fica a meia-espessura da silhueta. Sem dilatar, a metade externa
/// da linha não tem cor por baixo: com um pincel MACIO ela mistura com o fundo, e o
/// contorno ganha um halo escuro (o *"o fill não se ajusta à linha"* do smoke). Com a
/// dilatação, a cor vai exatamente até a silhueta — e como as duas grandezas estão na
/// MESMA unidade (MUNDO, §4.C.6), o encaixe é invariante ao zoom por CONSTRUÇÃO, que era
/// todo o ponto da âncora no eixo.
///
/// Largura VARIÁVEL (pressão): usa-se a média. A dilatação erra por (w_local − w_média)/2
/// nos pontos extremos — sub-pixel num traço de mouse (largura constante), e sempre
/// menor que o erro de não dilatar nada.
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
            // A dilatação (a cor entra por baixo da linha), **por ponto**: ela veste a
            // linha LOCAL, não a média do desenho.
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

/// **A linha que este ponto do contorno está vestindo**: `(espessura, distância)`, em
/// unidades de mundo — os dois dados que a média global não tinha.
///
/// O contorno do balde termina no EIXO da linha (BUGS #14), então o line-art mais
/// próximo de um ponto do contorno é, por construção, a linha que ele veste.
pub(crate) fn local_line(drawing: &FlipDrawing, p: Vec2) -> Option<(f32, f32)> {
    let mut best: Option<(f32, f32)> = None; // (dist, width)
    for s in drawing.strokes.iter().filter(|s| !s.hide_stroke) {
        // ⚠️ **Distância ao SEGMENTO, nunca ao VÉRTICE.** O eixo é a polilinha, não a
        // nuvem de pontos dela: um ponto do contorno pousado exatamente sobre o eixo,
        // mas no meio de dois vértices, fica a até meia-amostragem do vértice mais
        // próximo. Medir ao vértice faria a compensação pagar o **espaçamento da
        // amostragem** como se fosse erro de vetorização — e num traço de 64 amostras
        // isso é maior que o erro real. É o BUGS #18 outra vez: quem pergunta "quais são
        // os segmentos deste traço?" usa a porta do modelo (`segments()`), e ela já
        // fecha a costura do traço fechado.
        for (i, a, b) in s.segments() {
            let ab = Vec2::new(b.x - a.x, b.y - a.y);
            let l2 = ab.x * ab.x + ab.y * ab.y;
            let t = if l2 <= 0.0 {
                0.0
            } else {
                (((p.x - a.x) * ab.x + (p.y - a.y) * ab.y) / l2).clamp(0.0, 1.0)
            };
            let (dx, dy) = (p.x - (a.x + t * ab.x), p.y - (a.y + t * ab.y));
            let d = (dx * dx + dy * dy).sqrt();
            // A espessura interpolada ao longo do segmento (pressão varia por ponto).
            let ws = s.widths();
            let (wa, wb) = (
                ws.get(i).copied().unwrap_or(0.0),
                ws.get((i + 1) % ws.len().max(1)).copied().unwrap_or(0.0),
            );
            let w = wa + (wb - wa) * t;
            if w > 0.0 && best.is_none_or(|(bd, _)| d < bd) {
                best = Some((d, w));
            }
        }
    }
    best.map(|(d, w)| (w, d))
}

/// A espessura MÉDIA do line-art que delimita a região (unidades de MUNDO) — a dilatação que
/// o contorno do fill veste. Ignora as regiões (que não têm tinta) e os fechamentos de
/// gap (largura zero).
pub(crate) fn mean_line_width(drawing: &FlipDrawing) -> f32 {
    let (sum, n) = drawing
        .strokes
        .iter()
        .filter(|s| !s.hide_stroke)
        .flat_map(|s| s.widths().iter().copied())
        .filter(|w| *w > 0.0)
        .fold((0.0f32, 0usize), |(sum, n), w| (sum + w, n + 1));
    if n == 0 { 0.0 } else { sum / n as f32 }
}

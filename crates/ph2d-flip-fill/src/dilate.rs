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

/// Margem da dilatação, como **fração da meia-espessura da linha** (adimensional).
///
/// O emprego dela é encostar a cor por baixo do **falloff MACIO** do pincel: a borda de
/// um pincel macio é translúcida, e a cor tem de ir um pouco além da silhueta nominal
/// para o fundo não aparecer através dela.
///
/// ⚠️ **É uma FRAÇÃO, e essa é a correção do smoke de 2026-07-18** (*"extrapolando um
/// pouco da borda externa"*, com o traço fino e a câmera perto). Antes era uma distância
/// FIXA — e uma distância fixa somada a uma linha de qualquer espessura é uma fração
/// GRANDE numa linha fina e desprezível numa grossa. A assinatura estava na minha própria
/// medição e eu passei por ela: o transbordo media **4 / 2 / 1** em linhas de 8 / 16 /
/// 32 px — *pior na mais fina*, monotonicamente. Uma margem que só depende do PINCEL não
/// pode fazer isso.
///
/// E a extensão do falloff é proporcional ao raio do pincel — não é um número de pixels
/// que exista fora dele. Então a grandeza certa era adimensional desde o começo, o que de
/// quebra torna o BUGS #20 (somar px a unidades de mundo) **impossível neste termo**: não
/// há unidade para atravessar.
///
/// **O valor é MEDIDO.** Cobertura = amostras de fundo sob a linha somadas na faixa de
/// zoom; transbordo = `the_colour_never_spills_outside_the_line` (8/16/32 px, bar 2 %):
///
/// | lei | cobertura | transbordo | engorda: fino (0,02) / default (0,06) / grosso (0,12) |
/// |---|---|---|---|
/// | margem FIXA (até 2026-07-18) | 158 | 0,2 / 0,1 / 0,0 % | **25 %** / 8,3 % / 4,2 % |
/// | fração 0,02 | 170 | 0,1 / 0,1 / 0,1 % | 2 % / 2 % / 2 % |
/// | **fração 0,03** | **156** | **0,2 / 0,1 / 0,5 %** | **3 % / 3 % / 3 %** |
/// | fração 0,04 | 149 | 0,2 / 0,2 / **1,6 %** | 4 % / 4 % / 4 % |
/// | fração 0,06 | 116 | 0,2 / 0,3 / **4,6 %** ✗ | 6 % / 6 % / 6 % |
///
/// ⚠️ **O ganho NÃO está nos agregados, e é honesto dizer.** Contra a lei antiga, 0,03
/// mexe pouco na cobertura (156 contra 158) e pouco no transbordo. O que ele conserta é a
/// **DISTRIBUIÇÃO**: a franja deixa de ser 25 % da linha no traço fino e 4 % no grosso, e
/// passa a ser 3 % em todos. Foi isso que o Enio viu, e nenhum número somado sobre a
/// figura inteira ia mostrar — some na média.
///
/// O teto é o gate de transbordo, que é anterior a esta mudança e vale para o produto:
/// 0,06 estoura o limite de 2 % no traço grosso (4,6 %), porque uma fração de um traço
/// gordo é muita tinta em termos absolutos. 0,03 passa com folga nas três espessuras.
pub const FILL_TUCK_FRACTION: f32 = 0.03; // adimensional: fracao da meia-espessura

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
    nearest_on_axis(strokes, p).map(|(w, q)| {
        let (dx, dy) = (p.x - q.x, p.y - q.y);
        (w, (dx * dx + dy * dy).sqrt())
    })
}

/// A linha mais próxima **e o ponto exato do eixo** que a representa: `(espessura, q)`.
///
/// A distância sozinha não basta para compensar o erro de vetorização, porque ela não diz
/// **de que lado**. Quem quer o sinal precisa de `q`.
#[must_use]
pub fn nearest_on_axis(strokes: &[(Vec<Vec2>, Vec<f32>, bool)], p: Vec2) -> Option<(f32, Vec2)> {
    let mut best: Option<(f32, f32, Vec2)> = None; // (dist, largura, q)
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
            if w > 0.0 && best.is_none_or(|(bd, _, _)| d < bd) {
                best = Some((d, w, Vec2::new(a.x + t * ab.x, a.y + t * ab.y)));
            }
        }
    }
    best.map(|(_, w, q)| (w, q))
}

/// A **normal EXTERNA** do anel em cada vértice, na ordem dos pontos.
///
/// O sinal sai da área com sinal (a orientação do anel), e não de um chute geométrico
/// tipo *"para longe do centroide"* — que só funciona em forma convexa, e uma região
/// preenchida à mão raramente é.
fn outward_normals(ring: &[Vec2]) -> Vec<Vec2> {
    let n = ring.len();
    // Área positiva = uma orientação; negativa = a outra. Qual delas é "anti-horária"
    // depende do eixo y apontar para cima ou para baixo, e este documento usa y para
    // BAIXO — então a constante certa é a que o gate `the_outward_normal_points_away`
    // fixa, medida num círculo, e não a que o livro de geometria assume.
    let orient = if crate::signed_area(ring) >= 0.0 {
        1.0
    } else {
        -1.0
    };
    (0..n)
        .map(|i| {
            // Tangente pela DIFERENÇA CENTRAL (vizinho anterior → próximo): num anel
            // reamostrado ela é muito mais estável que a aresta de um lado só, e é a
            // mesma escolha que o cálculo da normal do relevo faz no Painter.
            let (a, b) = (ring[(i + n - 1) % n], ring[(i + 1) % n]);
            let t = Vec2::new(b.x - a.x, b.y - a.y);
            let len = (t.x * t.x + t.y * t.y).sqrt();
            if len <= 0.0 {
                return Vec2::new(0.0, 0.0);
            }
            Vec2::new(orient * t.y / len, -orient * t.x / len)
        })
        .collect()
}

/// **A LEI**, ponto a ponto: a largura com que cada ponto do contorno é desenhado.
///
/// `largura da linha LOCAL + a margem` — a dilatação veste a linha que o contorno abraça
/// NAQUELE ponto, e não a média do desenho: num desenho com espessuras diferentes a
/// média fica entre elas, então onde o contorno abraça a linha FINA a cor saía larga
/// demais e aparecia do outro lado dela (o smoke do Enio, BUGS #20).
///
/// # A compensação tem SINAL, e é isso que a torna certa
///
/// O contorno sai do marching squares + RDP, então ele erra em torno do eixo — às vezes
/// caindo **para dentro** (a cor não alcança a linha, e num pincel macio o fundo aparece
/// pelo vão), às vezes **para fora** (a cor transborda a silhueta). A distância `d` até o
/// eixo diz **quanto**, e nunca disse **de que lado**.
///
/// Sem o lado, a única defesa possível era uma **margem uniforme**: dilatar o contorno
/// inteiro o bastante para cobrir o pior caso. Ela funciona e cobra o preço em todo ponto
/// que já estava certo — a *"referência não é o centro da linha mas a borda externa"* que
/// o Enio viu, e a sub-cobertura ao afastar que o `sweep_zoom` mediu (41/55 amostras a
/// 25 px/unidade). Os dois defeitos são a MESMA constante, vista dos dois lados.
///
/// O sinal vem da geometria que já está na mão: a **normal externa do anel** contra a
/// direção do eixo. `s = (q − p) · n_out` é positivo quando o eixo está para FORA do
/// contorno (o contorno ficou aquém, precisa crescer) e negativo quando ficou além
/// (precisa encolher). A meia-espessura que faz a cor pousar exatamente na silhueta é
/// `w/2 + s`, logo a largura é **`w + 2s`** — e onde o contorno acertou o eixo (`s ≈ 0`)
/// ela é exatamente `w`: **nem um pixel de transbordo, sem margem nenhuma**.
///
/// ⚠️ **A tentativa anterior (2026-07-18, revertida sem shipar) era `w + 2d`** — o mesmo
/// termo com o sinal sempre positivo. Ela mede **pior** que a margem fixa (0,0178 contra
/// 0,005 na mediana) porque dobra o erro exatamente nos pontos que transbordaram: metade
/// das correções ia para o lado errado. Não era a ideia que estava errada, era o `d` nu.
#[must_use]
pub fn contour_widths(strokes: &[(Vec<Vec2>, Vec<f32>, bool)], contour: &[Vec2]) -> Vec<f32> {
    contour_widths_with_margin(strokes, contour, FILL_TUCK_FRACTION)
}

/// A lei com a margem **parametrizada** — ver o aviso do `margin_world`: isto é para a
/// varredura que ESCOLHE a constante, e o produto usa `contour_widths`.
#[must_use]
pub fn contour_widths_with_margin(
    strokes: &[(Vec<Vec2>, Vec<f32>, bool)],
    contour: &[Vec2],
    margin_fraction: f32,
) -> Vec<f32> {
    let fallback = mean_line_width(strokes);
    let normals = outward_normals(contour);

    // Passo 1: a largura da linha e o desvio COM SINAL, ponto a ponto.
    let mut widths = Vec::with_capacity(contour.len());
    let mut offsets = Vec::with_capacity(contour.len());
    for (i, &p) in contour.iter().enumerate() {
        match nearest_on_axis(strokes, p) {
            Some((w, q)) => {
                let n = normals[i];
                // `s > 0`: o eixo está para FORA — o contorno ficou aquém e a cor precisa
                // avançar. `s < 0`: o contorno passou do eixo e a cor precisa recuar.
                widths.push(w);
                offsets.push((q.x - p.x) * n.x + (q.y - p.y) * n.y);
            }
            None => {
                widths.push(fallback);
                offsets.push(0.0);
            }
        }
    }

    // Passo 2: **alisa o desvio ao longo do anel.**
    //
    // O erro que se quer compensar é de BAIXA frequência (trechos inteiros onde o
    // contorno ficou aquém do eixo); o que sobra por cima é o tremor de alta frequência
    // do próprio traçado, que o RDP e o alisamento binomial deixam. Corrigir ponto a
    // ponto sem separar os dois faz a largura **seguir o ruído** — e uma largura que
    // segue ruído desenha uma borda serrilhada, que é trocar um defeito por outro (a
    // lição da borda do Inflate, no Painter).
    //
    // É o mesmo binomial [1,2,1] cíclico que o `simplify_ring` já usa, pela mesma razão.
    smooth_ring(&mut offsets, OFFSET_SMOOTH_PASSES);

    widths
        .iter()
        .zip(&offsets)
        .map(|(&w, &s)| {
            // O `max(0)` não é um teto disfarçado: largura negativa não existe, e um
            // contorno que passou do eixo mais do que a linha é grossa não tem cor
            // nenhuma para pôr ali.
            // A margem é FRAÇÃO da linha: numa linha fina ela é fina, numa grossa é
            // grossa — que é o que "encostar no falloff" quer dizer.
            (w * (1.0 + margin_fraction) + 2.0 * s).max(0.0)
        })
        .collect()
}

/// Quantos passes de binomial alisam o desvio antes de ele virar largura.
///
/// **Medido**, não escolhido. O critério é a divergência da MESMA arte em duas escalas
/// (`the_same_art_at_the_products_scale_renders_the_same`), que é ruído puro — quanto
/// menor, menos a largura está seguindo o traçado em vez da forma:
///
/// | passes | pior delta entre escalas | transbordo (`sweep_tuck` margem 0: 8/16/32px) |
/// |---|---|---|
/// | 0 | 75 | 11 / 10 / 11 |
/// | **2** | **20** | **8 / 9 / 12** |
/// | 4 | 20 | 8 / 9 / 14 |
/// | 8 | 79 | 7 / 9 / 14 |
/// | 12 | 79 | 7 / 10 / 14 |
///
/// ⚠️ **A curva NÃO é monótona, e é por isso que ela foi medida em vez de arbitrada.** A
/// intuição ("mais alisamento = menos ruído") acerta até 4 e depois **inverte**: em 8
/// passes o desvio já perdeu a forma que devia corrigir, a compensação vira uma constante
/// local — quase a média que ela veio substituir — e o erro entre escalas volta a 79.
/// Um número escolhido no olho teria pousado em 8 com toda a confiança do mundo.
///
/// 2 ganha de 4 no traço grosso (12 contra 14 de transbordo em 32px) com o mesmo delta.
const OFFSET_SMOOTH_PASSES: usize = 2;

/// Binomial `[1,2,1]` **cíclico** — o anel não tem pontas.
fn smooth_ring(v: &mut [f32], passes: usize) {
    let n = v.len();
    if n < 3 {
        return;
    }
    let mut tmp = vec![0.0f32; n];
    for _ in 0..passes {
        for i in 0..n {
            tmp[i] = 0.25 * v[(i + n - 1) % n] + 0.5 * v[i] + 0.25 * v[(i + 1) % n];
        }
        v.copy_from_slice(&tmp);
    }
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

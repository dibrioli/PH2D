//! **O AJUSTE — quais pontos o traço guarda.**
//!
//! ⚠️ Irmão do [`super`], e o corte é por assunto: lá mora *o que se faz com uma lista de pontos*
//! (alisar, reamostrar); aqui, ***quantos pontos ela deve ter***.

use super::{Vec2, resample_smooth};

/// **SIMPLIFICAÇÃO CONTRA A CURVA QUE SERÁ DESENHADA, DA ESQUERDA PARA A DIREITA** — o traço
/// guardado deixa de ser re-decidido enquanto a mão continua desenhando.
///
/// ## ⚠️ A propriedade que este ajuste tem e o anterior não tinha: ele é PREFIXO-ESTÁVEL
///
/// Report do Enio (2026-07-30, terceira rodada): *"o traço deve parar de reconstruir a curva toda;
/// os pontos já postos não devem ser modificados"*.
///
/// A versão anterior era **divide-e-conquista com fila por pior erro**: a primeira partição saía do
/// máximo **GLOBAL**, então uma amostra nova na ponta podia re-decidir um corte no COMEÇO — e o
/// preview roda o ajuste **a cada frame**. Medido no pipeline do produto, traço de 1200 amostras
/// (`measure_how_often_an_already_placed_point_is_moved`): **762 de 1001 frames** mexiam em pontos
/// já postos, atrás do cursor. O deslocamento por frame é pequeno (~1 % da espessura), e é
/// exatamente por isso que ele **não** foi curado pelas duas rodadas anteriores: o que o artista vê
/// não é a amplitude, é 1 % **mudando a cada frame** sobre um traço que ele já considera pronto.
///
/// Andar da esquerda para a direita apaga a classe inteira: **a decisão de um nó só olha amostras
/// que já chegaram**, então `fit(amostras[..n])` é prefixo de `fit(amostras[..m])` para todo
/// `m ≥ n`, a menos do último span — o que está literalmente sob a caneta. Não é uma calibração
/// melhor, é a propriedade que faltava.
///
/// ## Como
///
/// Do último nó guardado, estende o span **enquanto a reconstrução couber na tolerância**, e fecha o
/// nó no último ponto que coube. A reconstrução vem da porta do produto ([`resample_smooth`], a
/// MESMA que o traço usa) — uma cópia da avaliação da curva aqui recriaria o desacordo que este
/// ajuste existe para acabar (era o defeito da rodada 1: o `simplify_rdp` cobrava contra a CORDA
/// RETA enquanto quem desenha traça uma Catmull-Rom, e num gancho fechado a corda parecia ótima
/// com o traço final a **8,46 %** da espessura de onde a mão passou).
///
/// ⚠️ **A tangente do fim do span sai do [`espelho`], nunca de um nó futuro** — é a única
/// dependência de "frente" que existe aqui, ela vale **um span** (não o traço inteiro), e é o que
/// mantém o erro cobrado contra a curva que de fato será desenhada. Medido
/// (`measure_how_far_behind_the_pen_a_point_freezes`): a re-decisão mais funda fica a **12 amostras
/// atrás do cursor**.
///
/// ⚠️ **Não há mais TETO de pontos, e a remoção é consequência da lei, não economia.** Um teto
/// global é uma decisão sobre o traço INTEIRO: sob ele, o mesmo começo recebe orçamentos diferentes
/// conforme o traço cresce — que é precisamente o defeito acima. Quem limita a contagem agora é o
/// piso anti-redundância, que é **local** (uma pergunta por span). Medido pelo pipeline do produto
/// num traço de 9000 amostras: **919 pontos em 1,92 ms** — contra o teto anterior de 512.
///
/// ⚠️ **O que esta lei NÃO faz, com o número ao lado:** ela para de *re-decidir*, não de
/// *re-computar*. O ajuste segue percorrendo o traço inteiro a cada frame do preview — medido
/// (`measure_what_a_live_preview_frame_is_made_of`), o frame custa **0,33 ms a 1200 amostras e
/// 2,42 ms a 9000**, dos quais o ajuste é **95 %**. Um cache incremental levaria isso a `O(cauda)`
/// e **só é seguro por causa desta lei** (com a decisão global de antes, um prefixo em cache
/// mentiria); fica MEDIDO e não construído — 2,42 ms são 15 % de um quadro de 60 fps, num traço de
/// 18 000 px de percurso.
#[must_use]
pub(crate) fn simplify_to_curve(points: &[Vec2], tol: f32, step: f32) -> Vec<usize> {
    fit(points, tol, step, SAFETY)
}

/// **A MARGEM DE ACEITAÇÃO** — o span é fechado quando o erro ESTIMADO passa de `tol × SAFETY`.
///
/// ⚠️ **Ela existe porque a estimativa é aproximada por construção** (a tangente do fim do span sai
/// do [`espelho`], um palpite sobre onde o próximo nó vai cair), e o erro que importa é o da curva
/// DESENHADA. Sem margem o desenhado saía a **1,27× a tolerância**; a varredura mediu o gancho da
/// fixture de precisão e o arco liso ao lado dela:
///
/// | margem | gancho (% da espessura) | pts do gancho | pts do arco liso |
/// |---|---|---|---|
/// | 1,00 | 2,53 | 11 | 16 |
/// | 0,80 | 1,94 | 11 | 17 |
/// | **0,65** | **1,40** | **13** | **19** |
/// | 0,50 | 0,92 | 22 | **49** |
///
/// **0,65 bate o divide-e-conquista na precisão** (1,40 % contra 1,85 %) **e ainda é econômico**;
/// **0,50 é o joelho e ele é do lado errado** — o arco LISO salta de 19 para 49 pontos, que é
/// literalmente a queixa de 2026-07-18 (*"muitos pontos muito próximos e até sobrepostos"*)
/// voltando pela porta dos fundos. Abaixo de 0,65 a margem deixa de comprar precisão e passa a
/// comprar pontos.
const SAFETY: f32 = 0.65;

/// O ajuste com a margem como PARÂMETRO — é assim que a sonda a varre. A produção passa a
/// [`SAFETY`]; ninguém mais tem escolha.
fn fit(points: &[Vec2], tol: f32, step: f32, safety: f32) -> Vec<usize> {
    let n = points.len();
    if n < 3 || tol <= 0.0 {
        return (0..n).collect();
    }
    // ⚠️ **Um corte que o RENDER não consegue representar não é precisão, é lixo.** O
    // `resample_smooth` densifica a curva a cada `step`, então dois pontos guardados a menos de
    // meio `step` carregam informação que o traço desenhado não tem como mostrar — e é exatamente a
    // queixa de 2026-07-18 (*"pontos muito próximos e até sobrepostos"*).
    //
    // ⚠️ **A régua NÃO é minha: é a que o gate anti-redundância já cobra** — `0,05 × espessura`
    // (`the_resampled_stroke_tracks_the_drawing_without_redundant_points`), que nesta unidade é
    // `step × 0,125`, a MESMA razão `STROKE_SIMPLIFY_FRACTION / RESAMPLE_STEP_FRACTION` que o
    // `resample_smooth` documenta. Uma primeira tentativa usou `step × 0,5` e era grande demais:
    // dá `0,4 × raio` contra a cerca de vizinhos do render, que é `0,1875 × raio`.
    let minimo = step * 0.125;
    let longe = |i: usize, j: usize| -> bool {
        let d = Vec2::new(points[j].x - points[i].x, points[j].y - points[i].y);
        (d.x * d.x + d.y * d.y).sqrt() >= minimo
    };
    let mut keep = vec![0_usize];
    let mut a = 0_usize;
    while a < n - 1 {
        // O nó ANTERIOR já guardado dá a tangente de entrada do span. Ele é passado, nunca futuro.
        let antes = (keep.len() >= 2).then(|| keep[keep.len() - 2]);
        let mut b = a + 1;
        while b < n - 1 {
            let candidato = b + 1;
            // Abaixo do piso o span é estendido SEM perguntar o erro: guardar ali seria guardar o
            // que o render não desenha.
            if longe(a, b) {
                let depois = espelho(a, candidato, n);
                if span_error(points, (antes, a, candidato, depois), step, tol) > tol * safety {
                    break;
                }
            }
            b = candidato;
        }
        keep.push(b);
        a = b;
    }
    keep
}

/// O ajuste com a margem forçada — só a sonda que a escolheu chama por aqui.
#[cfg(test)]
#[must_use]
pub(crate) fn fit_tuned(points: &[Vec2], tol: f32, step: f32, safety: f32) -> Vec<usize> {
    fit(points, tol, step, safety)
}

/// **Onde estará o PRÓXIMO nó** — a amostra a uma distância de span adiante de `b`, espelhando o
/// span que acabou de ser medido.
///
/// ⚠️ **Isto é a única dependência de "frente" do ajuste, e ela precisa ser esta.** A tangente que a
/// curva final terá em `b` sai dos vizinhos `(a, próximo nó)`, não de `(a, amostra colada em b)` —
/// e a diferença NÃO é sutil: com a amostra adjacente o gancho da fixture de precisão saiu a
/// **14,62 %** da espessura da mão, contra **1,85 %** com o espelho. Numa região de curvatura
/// parecida os spans saem parecidos, então espelhar o span é o palpite certo sobre onde o vizinho
/// vai cair.
///
/// ⚠️ **E o preço é nomeado:** a decisão sobre `b` passa a depender de amostras até `b + (b − a)`,
/// ou seja a franja provisória do traço deixa de ser "dois nós" e passa a ser "um span" — que é o
/// pedaço sob a caneta de qualquer forma. Passado isso, congela.
fn espelho(a: usize, b: usize, n: usize) -> Option<usize> {
    let alvo = (b + (b - a)).min(n - 1);
    (alvo > b).then_some(alvo)
}

/// O pior desvio das amostras de `[a, b]` contra a curva que o produto desenharia ali.
///
/// ⚠️ **Reconstrói só a VIZINHANÇA** (`antes, a, b, depois`), e é isso que mantém o custo em
/// `O(n · m_local)` em vez de `O(k · n · m)`: a Catmull-Rom é local — a tangente em `a` só olha o
/// vizinho anterior —, então o span não precisa do traço inteiro para saber a própria curva.
fn span_error(
    points: &[Vec2],
    span: (Option<usize>, usize, usize, Option<usize>),
    step: f32,
    tol: f32,
) -> f32 {
    let (antes, a, b, depois) = span;
    let mut viz: Vec<Vec2> = Vec::with_capacity(4);
    viz.extend(antes.map(|i| points[i]));
    viz.push(points[a]);
    viz.push(points[b]);
    viz.extend(depois.map(|i| points[i]));
    let prs = vec![1.0_f32; viz.len()];
    let (curva, _) = resample_smooth(&viz, &prs, step, tol);
    let mut pior = 0.0_f32;
    for q in points.iter().take(b).skip(a + 1) {
        let mut d = f32::MAX;
        for w in curva.windows(2) {
            d = d.min(dist_to_seg(*q, w[0], w[1]));
        }
        pior = pior.max(d);
    }
    pior
}

/// Distância de `q` ao SEGMENTO `a→b` (clampada), que é o que a curva desenhada de fato ocupa — a
/// [`perp_dist`] mede até a reta INFINITA, e usá-la aqui cobraria distância a prolongamentos que
/// ninguém desenha.
fn dist_to_seg(q: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = Vec2::new(b.x - a.x, b.y - a.y);
    let len2 = ab.x * ab.x + ab.y * ab.y;
    let t = if len2 < 1e-12 {
        0.0
    } else {
        (((q.x - a.x) * ab.x + (q.y - a.y) * ab.y) / len2).clamp(0.0, 1.0)
    };
    let (dx, dy) = (q.x - (a.x + ab.x * t), q.y - (a.y + ab.y * t));
    (dx * dx + dy * dy).sqrt()
}

//! **O AJUSTE — quais pontos o traço guarda.**
//!
//! ⚠️ Irmão do [`super`], e o corte é por assunto: lá mora *o que se faz com uma lista de pontos*
//! (alisar, reamostrar); aqui, ***quantos pontos ela deve ter***.

use super::{Vec2, resample_smooth};

/// **SIMPLIFICAÇÃO CONTRA A CURVA QUE SERÁ DESENHADA** — o simplificador e o reconstrutor passam a
/// concordar sobre o que um ponto guardado significa.
///
/// ⚠️ **O defeito que ela corrige** (Enio 2026-07-30, com screenshot): o [`simplify_rdp`] cobra a
/// tolerância contra a **CORDA RETA** entre dois pontos, mas quem desenha depois é o
/// [`resample_smooth`], que traça uma **Catmull-Rom centrípeta** pelos sobreviventes. Num gancho
/// fechado a corda parece ótima — e a curva reconstruída estoura a curva da mão. Medido no gancho
/// (240 amostras, espessura 12): o RDP guardava **11 pontos** e o traço final ficava a **8,46 % da
/// espessura** de onde a mão passou.
///
/// ⚠️ **E é por isso que a CONSTANTE não era o instrumento.** Ela já tinha sido reclamada nas duas
/// pontas — `0,0008` deu *"muitos pontos muito próximos e até sobrepostos"* (2026-07-18) e `0,05`
/// deu *"poucos pontos"* (hoje). Um terceiro ajuste do mesmo número é o sinal de que o modelo está
/// errado ([[feedback_a_new_remedy_makes_the_old_one_double_counting]]).
///
/// **Como:** refinamento guloso. Começa com as pontas, reconstrói **pela porta do produto**
/// ([`resample_smooth`], a MESMA que o traço usa) e acrescenta o ponto da mão que mais se afasta da
/// reconstrução, até que ninguém passe da tolerância.
///
/// ⚠️ **A reconstrução vem da porta única, e isso é a espinha:** uma cópia da avaliação da curva
/// aqui recriaria exatamente o desacordo que esta função existe para acabar.
///
/// Ganho estrutural: num arco liso a Catmull-Rom já reconstrói bem ⇒ **poucos pontos** (nada de
/// pontos sobrepostos); num gancho ela não reconstrói ⇒ **pontos onde a precisão é necessária**.
/// As duas queixas opostas param de disputar o mesmo número.
#[must_use]
pub(crate) fn simplify_to_curve(points: &[Vec2], tol: f32, step: f32) -> Vec<usize> {
    let n = points.len();
    if n < 3 || tol <= 0.0 {
        return (0..n).collect();
    }
    let mut keep: Vec<usize> = vec![0, n - 1];
    let mut fila: Vec<(usize, usize)> = vec![(0, n - 1)];
    while let Some((a, b)) = fila.pop() {
        if b <= a + 1 || keep.len() >= MAX_FIT_POINTS {
            continue;
        }
        // Os vizinhos GUARDADOS dão as tangentes do span — a curva é local, e é isso que torna o
        // erro de um span computável sem reconstruir o traço inteiro.
        let pos = keep.binary_search(&a).unwrap_or(0);
        let antes = pos.checked_sub(1).map(|k| keep[k]);
        let depois = keep.get(pos + 2).copied();
        let (pior, idx) = span_error(points, (antes, a, b, depois), step, tol);
        if pior <= tol || idx <= a || idx >= b {
            continue;
        }
        // ⚠️ **Um corte que o RENDER não consegue representar não é precisão, é lixo.** O
        // `resample_smooth` densifica a curva a cada `step`, então dois pontos guardados a menos de
        // meio `step` carregam informação que o traço desenhado não tem como mostrar — e é
        // exatamente a queixa de 2026-07-18 (*"pontos muito próximos e até sobrepostos"*), que o
        // gate `the_resampled_stroke_tracks_the_drawing_without_redundant_points` pegou quando o
        // ajuste passou a ser dividir-e-conquistar.
        //
        // ⚠️ **A régua NÃO é minha: é a que o gate anti-redundância já cobra** — `0,05 × espessura`
        // (`the_resampled_stroke_tracks_the_drawing_without_redundant_points`). Na unidade que esta
        // função tem, isso é `step × 0,125`, a MESMA razão
        // `STROKE_SIMPLIFY_FRACTION / RESAMPLE_STEP_FRACTION` que o `resample_smooth` documenta.
        //
        // ⚠️ **A primeira tentativa foi `step × 0,5` e ela era grande demais — provado por gate:**
        // dá `0,4 × raio`, contra a cerca de vizinhos do render, que é `0,1875 × raio`. Um piso
        // ACIMA do que o render suporta proíbe espaçamentos que o produto sabe desenhar, e o
        // `the_slow_hand_star_is_denser_than_the_old_neighbour_fence` ficou vermelho dizendo
        // exatamente isso — a cena de mão lenta deixava de conter o fenômeno.
        let minimo = step * 0.125;
        let longe = |i: usize, j: usize| -> bool {
            let d = Vec2::new(points[j].x - points[i].x, points[j].y - points[i].y);
            (d.x * d.x + d.y * d.y).sqrt() >= minimo
        };
        if !longe(a, idx) || !longe(idx, b) {
            continue;
        }
        if let Err(at) = keep.binary_search(&idx) {
            keep.insert(at, idx);
        }
        fila.push((a, idx));
        fila.push((idx, b));
    }
    keep
}

/// O pior desvio das amostras de `[a, b]` contra a curva que o produto desenharia ali, e ONDE.
///
/// ⚠️ **Reconstrói só a VIZINHANÇA** (`antes, a, b, depois`), e é isso que derruba o custo de
/// `O(k · n · m)` para `O(n · m_local)`: a Catmull-Rom é local — a tangente em `a` só olha o
/// vizinho anterior —, então o span não precisa do traço inteiro para saber a própria curva.
fn span_error(
    points: &[Vec2],
    span: (Option<usize>, usize, usize, Option<usize>),
    step: f32,
    tol: f32,
) -> (f32, usize) {
    let (antes, a, b, depois) = span;
    let mut viz: Vec<Vec2> = Vec::with_capacity(4);
    viz.extend(antes.map(|i| points[i]));
    viz.push(points[a]);
    viz.push(points[b]);
    viz.extend(depois.map(|i| points[i]));
    let prs = vec![1.0_f32; viz.len()];
    let (curva, _) = resample_smooth(&viz, &prs, step, tol);
    let (mut pior, mut onde) = (0.0_f32, a);
    for (i, q) in points.iter().enumerate().take(b).skip(a + 1) {
        let mut d = f32::MAX;
        for w in curva.windows(2) {
            d = d.min(dist_to_seg(*q, w[0], w[1]));
        }
        if d > pior {
            pior = d;
            onde = i;
        }
    }
    (pior, onde)
}

/// Teto de pontos do ajuste guloso — **custo**, não qualidade (ver [`simplify_to_curve`]).
const MAX_FIT_POINTS: usize = 512;

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

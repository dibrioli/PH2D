//! ⭐⭐ **OS BLOCOS 2D DE UMA CHAPA** — o disco, o rectângulo, a faixa e o arco, numa porta só.
//!
//! # Por que eles saíram dos dois módulos que os tinham
//!
//! O [`crate::ops_plates`] tinha `disco`/`rect`/`rect_round`; o [`crate::ops_arrows`] tinha
//! `rect_em`/`rect_round_em` — **a mesma lei com um centro a mais**. ⚠️ *Uma lei escrita em dois
//! sítios ainda não é uma lei; só uma PORTA é* — a mesma frase que o `stroke_uniform.rs` do vetor
//! pagou. Os dois módulos passam a chamar daqui, e as árvores que eles produzem são **as mesmas ao
//! bit** (os antigos eram o caso `cx = cy = 0`).
//!
//! # ⚠️ O que todo bloco daqui promete
//!
//! **Distância EXACTA, dentro e fora** — e é isso que os torna dilatáveis: a W104 mediu que
//! `offset` sobre um `max` de semiespaços é **inerte** (dilatar um semiespaço dá outro semiespaço,
//! sem canto para arredondar), e a receita que funciona é *encolher uma distância exacta e
//! deslocá-la*. Um bloco novo aqui que não seja exacto quebra essa promessa para todos.

use fidget::context::Tree;

use crate::ops::length2;

/// Um disco deslocado.
pub(crate) fn disco_em(cx: f64, cy: f64, r: f64) -> Tree {
    length2(
        &(Tree::x() - Tree::constant(cx)),
        &(Tree::y() - Tree::constant(cy)),
    ) - Tree::constant(r)
}

/// ⭐ **A CASCA de um disco** — a distância ao próprio círculo, engrossada em `meia`.
///
/// ⚠️ Exacta dos dois lados, ao contrário de `max(disco_fora, −disco_dentro)`: aquela é exacta só
/// enquanto uma das duas manda, e no meio da parede as duas mentem pela metade da espessura.
pub(crate) fn anel_em(cx: f64, cy: f64, raio: f64, meia: f64) -> Tree {
    let r = length2(
        &(Tree::x() - Tree::constant(cx)),
        &(Tree::y() - Tree::constant(cy)),
    );
    (r - Tree::constant(raio)).abs() - Tree::constant(meia)
}

/// Um rectângulo 2D de meias-extensões `(hx, hy)` centrado em `(cx, cy)` — distância **exacta**.
pub(crate) fn rect_em(cx: f64, cy: f64, hx: f64, hy: f64) -> Tree {
    let dx = (Tree::x() - Tree::constant(cx)).abs() - Tree::constant(hx);
    let dy = (Tree::y() - Tree::constant(cy)).abs() - Tree::constant(hy);
    length2(&dx.max(0.0), &dy.max(0.0)) + dx.max(dy).min(0.0)
}

/// O mesmo rectângulo **com as quatro quinas arredondadas** em `r` — ver o cabeçalho do módulo.
pub(crate) fn rect_round_em(cx: f64, cy: f64, hx: f64, hy: f64, r: f64) -> Tree {
    let r = r.min(hx * 0.999).min(hy * 0.999).max(0.0);
    crate::ops::offset(&rect_em(cx, cy, hx - r, hy - r), r)
}

/// As quatro paredes de um rectângulo, em semiplanos **separados** — a receita do braço da cruz.
///
/// ⚠️ Ela existe porque a mistura do aro precisa das peças **inteiras**: um rectângulo já composto
/// carrega a bissectriz das quinas para dentro do aro.
pub(crate) fn paredes(cx: f64, cy: f64, hx: f64, hy: f64) -> [Tree; 4] {
    [
        Tree::x() - Tree::constant(cx + hx),
        Tree::constant(cx - hx) - Tree::x(),
        Tree::y() - Tree::constant(cy + hy),
        Tree::constant(cy - hy) - Tree::y(),
    ]
}

/// Os quatro pares de quinas de um rectângulo dado em [`paredes`].
pub(crate) fn quinas(p: &[Tree; 4]) -> Vec<(Tree, Tree)> {
    vec![
        (p[0].clone(), p[2].clone()),
        (p[0].clone(), p[3].clone()),
        (p[1].clone(), p[2].clone()),
        (p[1].clone(), p[3].clone()),
    ]
}

/// ⭐ **UMA FAIXA de `p` a `q`**, de meia-espessura `t` e pontas **quadradas** — o rectângulo do
/// segmento, num referencial rodado.
///
/// ⚠️ **Uma rotação é uma ISOMETRIA**, então a distância exacta do rectângulo continua exacta aqui.
/// É a mesma receita que o losango do coração usa, e a razão de ela funcionar lá e não numa
/// **escala** por eixo.
pub(crate) fn faixa(p: [f64; 2], q: [f64; 2], t: f64, r: f64) -> Tree {
    let (dx, dy) = (q[0] - p[0], q[1] - p[1]);
    let l = (dx * dx + dy * dy).sqrt().max(f64::MIN_POSITIVE);
    let (ux, uy) = (dx / l, dy / l);
    let (mx, my) = ((p[0] + q[0]) * 0.5, (p[1] + q[1]) * 0.5);
    let px = Tree::x() - Tree::constant(mx);
    let py = Tree::y() - Tree::constant(my);
    // As coordenadas no referencial da faixa: ao longo e através.
    let ao_longo = px.clone() * Tree::constant(ux) + py.clone() * Tree::constant(uy);
    let atraves = px * Tree::constant(-uy) + py * Tree::constant(ux);
    let (hx, hy) = (l * 0.5, t);
    let r = r.min(hx * 0.999).min(hy * 0.999).max(0.0);
    let dx = ao_longo.abs() - Tree::constant(hx - r);
    let dy = atraves.abs() - Tree::constant(hy - r);
    (length2(&dx.clone().max(0.0), &dy.clone().max(0.0)) + dx.max(dy).min(0.0)) - Tree::constant(r)
}

/// As quatro paredes de uma [`faixa`], em semiplanos **separados** — a receita do braço da cruz,
/// num referencial rodado.
pub(crate) fn paredes_faixa(p: [f64; 2], q: [f64; 2], t: f64) -> [Tree; 4] {
    let (dx, dy) = (q[0] - p[0], q[1] - p[1]);
    let l = (dx * dx + dy * dy).sqrt().max(f64::MIN_POSITIVE);
    let (ux, uy) = (dx / l, dy / l);
    let (mx, my) = ((p[0] + q[0]) * 0.5, (p[1] + q[1]) * 0.5);
    let px = Tree::x() - Tree::constant(mx);
    let py = Tree::y() - Tree::constant(my);
    let ao_longo = px.clone() * Tree::constant(ux) + py.clone() * Tree::constant(uy);
    let atraves = px * Tree::constant(-uy) + py * Tree::constant(ux);
    [
        ao_longo.clone() - Tree::constant(l * 0.5),
        -ao_longo - Tree::constant(l * 0.5),
        atraves.clone() - Tree::constant(t),
        -atraves - Tree::constant(t),
    ]
}

/// ⭐ **UM ARCO** — a casca de um círculo cortada pelo sector `[a0, a1]` (em radianos, CCW).
///
/// ⚠️ **O sector é a intersecção de dois semiplanos**, e por isso a abertura tem de ser `≤ π`; quem
/// precisar de mais parte o arco em dois. É a mesma cerca da [`crate::ops_plates::sd_pie`], pela
/// mesma razão: em `π` os dois semiplanos são opostos e a união deles vale zero sobre um eixo
/// inteiro, que é uma fenda fantasma dentro da peça.
pub(crate) fn arco(
    cx: f64,
    cy: f64,
    raio: f64,
    meia: f64,
    a0: f64,
    a1: f64,
    e: crate::ops_joint::Edge,
) -> Tree {
    let px = Tree::x() - Tree::constant(cx);
    let py = Tree::y() - Tree::constant(cy);
    let corte = |ang: f64, sinal: f64| {
        let (s, c) = (ang + std::f64::consts::FRAC_PI_2).sin_cos();
        (px.clone() * Tree::constant(c * sinal) + py.clone() * Tree::constant(s * sinal)).clone()
    };
    // ⚠️ **As pontas do arco são ARESTAS de verdade** (onde a casca encontra o corte do sector), e
    // um `max` duro deixava-as vivas: o chanfro cortava `87,3 %` das arestas de uma chave, e as que
    // ficavam eram estas.
    //
    // ⚠️⚠️ **UMA mistura só, e não duas encaixadas** — encaixadas, a segunda recebe a composta da
    // primeira e compõe a inflação: a chave media `passo × ‖∇f‖ = 1,06` com o chanfro ligado, acima
    // do `1` em que a marcha atravessa a superfície.
    //
    // ⭐ **E só DUAS arestas se declaram**: as pontas, onde cada plano do sector encontra a casca.
    // Os dois planos encontram-se no CENTRO do círculo, muito fora da banda — declarar esse par
    // poria um plano de corte onde não há quina nenhuma.
    let (c0, c1) = (corte(a0, -1.0), corte(a1, 1.0));
    let casca = anel_em(cx, cy, raio, meia);
    if e.chamfer <= 0.0 {
        // ⛔⛔ **Com o chanfro a ZERO a mistura é BINÁRIA** — os planos de corte que a n-ária
        // acrescenta passam **exactamente** pela aresta, não cortam nada, e contam à mesma para o
        // `length` dela, cujo tecto é `√(activas)`. Medido: `passo × ‖∇f‖ = 1,51` na chave.
        //
        // ⚠️ **É a lei que este módulo já pagou CINCO vezes nesta jornada e na anterior** — e a
        // sexta foi esta. *Uma lei que se repete em seis sítios é uma porta que falta.*
        // ⭐ **AS MESMAS TRÊS PEÇAS, sem os planos de corte** — a estrutura não muda entre os dois
        // caminhos, e é isso que mantém a sonda do chanfro a medir a mesma forma. Com juntas
        // ENCAIXADAS a forma viva ficava diferente da chanfrada, e a fracção cortada caía a `62 %`.
        return crate::ops::intersection_round_n(&[c0, c1, casca], e.round);
    }
    crate::ops_joint::intersection_joint_n(
        &[c0.clone(), c1.clone(), casca.clone()],
        &[(c0, casca.clone()), (c1, casca)],
        e,
    )
}

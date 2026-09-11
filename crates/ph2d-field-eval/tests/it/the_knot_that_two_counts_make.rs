//! ⭐⭐⭐ **O NÓ DE TORO `(p, q)`** (W134) — dois inteiros dão uma família inteira, e nada do que já
//! existe a alcança (o toro é `q = 0`, a mola é aberta, o `Link` é uma argola achatada).
//!
//! | gate | o defeito que ele apanha |
//! |---|---|
//! | `the_knot_holds_the_curve_it_promises` | o campo desenhar OUTRA curva que não a `(p, q)` |
//! | `the_two_counts_are_not_interchangeable` | `p` e `q` serem lidos ao contrário (a peça sai plausível) |
//! | `the_seam_of_the_angle_does_not_crack_the_piece` | a costura do `atan2` — a lição da W128 |
//! | `the_cord_ceiling_is_where_the_strands_touch` | o tecto da corda ser um número escolhido |
//! | `a_count_written_with_a_fraction_lands_on_a_count` | um `p` fraccionário chegar ao campo |
//! | `raising_a_count_reseats_the_cord` | uma escrita deixar a peça INVÁLIDA (apaga a cena inteira) |
//! | `the_knot_offers_no_fillet_because_it_has_no_edge` | um controlo de filete morto no painel |

use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Primitive, Xform, dims, set_dim};
use ph2d_field_eval::Field;

const R: f32 = 0.55;
const TUBE: f32 = 0.24;

fn campo(p: Primitive) -> Field {
    Field::new(
        &FieldDoc::new(
            vec![Node::new(Xform::IDENTITY, NodeKind::Leaf(p))],
            NodeId(0),
        )
        .expect("a peça"),
    )
}

fn no(winds: u32, loops: u32, fraccao: f32) -> Primitive {
    Primitive::TorusKnot {
        radius: R,
        tube: TUBE,
        cord: ph2d_field::knot_cord_ceiling(R, TUBE, winds, loops) * fraccao,
        winds,
        loops,
    }
}

/// O ponto da curva paramétrica `(p, q)` em `t` — a **definição**, escrita aqui e não importada, para
/// o oráculo não partilhar nenhuma linha com o produto.
fn ponto_da_curva(t: f64, winds: u32, loops: u32) -> [f64; 3] {
    let (sq, cq) = (f64::from(loops) * t).sin_cos();
    let (sp, cp) = (f64::from(winds) * t).sin_cos();
    let raio = f64::from(TUBE).mul_add(cq, f64::from(R));
    [raio * cp, raio * sp, f64::from(TUBE) * sq]
}

/// ⭐⭐⭐ **A CURVA QUE ELE PROMETE ESTÁ DENTRO DA PEÇA, E O RESTO DO TORO NÃO ESTÁ.**
///
/// ⚠️ **As duas metades são obrigatórias.** Só a primeira aprova uma peça que seja o **toro inteiro**
/// (ela contém a curva, e mais tudo o resto); só a segunda aprova uma peça vazia. *Uma forma prova-se
/// pelo que ela tem E pelo que ela não tem.*
#[test]
fn the_knot_holds_the_curve_it_promises() {
    for (p, q) in [(2_u32, 3_u32), (3, 2), (2, 5), (5, 2), (1, 1)] {
        let f = campo(no(p, q, 0.55));
        // (a) toda a curva está DENTRO
        let mut pior = f64::NEG_INFINITY;
        for i in 0..4_000 {
            let t = std::f64::consts::TAU * f64::from(i) / 4_000.0;
            let [x, y, z] = ponto_da_curva(t, p, q);
            pior = pior.max(f.at(x, y, z));
        }
        assert!(
            pior < 0.0,
            "({p},{q}): um ponto da curva lê {pior:+.5} — a peça não contém o nó que ela diz ter"
        );
        // (b) e o PONTO MAIS LONGE de todo fio, na mesma superfície do toro, está FORA.
        //     ⚠️ Ele acha-se por varredura, e não por conta: o `ψ` que fica no meio depende de `p`.
        let mut melhor_fora = f64::NEG_INFINITY;
        for i in 0..240 {
            let phi = std::f64::consts::TAU * f64::from(i) / 240.0;
            for j in 0..240 {
                let psi = std::f64::consts::TAU * f64::from(j) / 240.0;
                let raio = f64::from(TUBE).mul_add(psi.cos(), f64::from(R));
                let v = f.at(
                    raio * phi.cos(),
                    raio * phi.sin(),
                    f64::from(TUBE) * psi.sin(),
                );
                melhor_fora = melhor_fora.max(v);
            }
        }
        assert!(
            melhor_fora > 0.0,
            "({p},{q}): TODA a superfície do toro está dentro da peça ({melhor_fora:+.5}) — isto \
             é um toro grosso, não um nó"
        );
    }
}

/// ⭐⭐ **`(p, q)` NÃO É `(q, p)`** — e um par lido ao contrário dá uma peça plausível, que é
/// exactamente por que este gate tem de existir.
#[test]
fn the_two_counts_are_not_interchangeable() {
    let a = campo(no(2, 3, 0.55));
    let b = campo(no(3, 2, 0.55));
    let mut maior = 0.0_f64;
    for i in 0..24 {
        for j in 0..24 {
            for k in 0..24 {
                let at = |t: usize| -1.0 + 2.0 * (t as f64 + 0.5) / 24.0;
                let (x, y, z) = (at(i), at(j), at(k));
                maior = maior.max((a.at(x, y, z) - b.at(x, y, z)).abs());
            }
        }
    }
    assert!(
        maior > 0.05,
        "o (2,3) e o (3,2) diferem no máximo {maior:.6} — as duas contagens estão a ser lidas como \
         uma só"
    );
}

/// ⭐⭐⭐ **A COSTURA DO `atan2` NÃO RACHA A PEÇA** — a lição que a superfórmula pagou (§129.4).
///
/// ⚠️ **O mecanismo aqui é OUTRO, e é isso que o gate afirma:** ali o `φ` entrava cru na fórmula e um
/// `m` fraccionário deixava um degrau; aqui o `φ` só entra dentro do CONJUNTO `{ψ_n}`, que é
/// invariante a `φ → φ − 2π`. *Um `min` sobre um conjunto invariante é contínuo mesmo quando cada
/// termo não é.*
#[test]
fn the_seam_of_the_angle_does_not_crack_the_piece() {
    // ⚠️ O `(2, 6)` está EXACTAMENTE no tecto de `q` para `p = 2` — a costura tem de aguentar a
    // ponta da faixa, que é onde o divisor é mais apertado.
    for (p, q) in [(2_u32, 3_u32), (3, 2), (5, 2), (2, 6), (7, 2)] {
        let f = campo(no(p, q, 0.55));
        // ⚠️⚠️ **A RÉGUA MEDE SE O DEGRAU ENCOLHE COM O PASSO, e as duas primeiras não mediam.**
        // Um limite ABSOLUTO reprova sobre produto correcto (perto do eixo o mesmo passo em `y` é
        // um passo enorme em `φ`, e o campo depende de `φ` por construção); e um CONTROLO noutro
        // ângulo compara pontos com sensibilidades diferentes — ele leu `0` onde a costura lia
        // `0,0018`, e a diferença era a geometria, não a costura.
        //
        // ⭐ *Uma descontinuidade não encolhe.* Um campo contínuo atravessado por um passo `8×`
        // menor dá um degrau `8×` menor; uma racha dá o **mesmo** degrau. É isso que se mede.
        let salto = |eps: f64| {
            let mut pior = 0.0_f64;
            for i in 0..80 {
                let raio = 0.05 + 0.95 * f64::from(i) / 80.0;
                for j in 0..40 {
                    let z = -0.35 + 0.7 * f64::from(j) / 40.0;
                    let d = (f.at(-raio, eps, z) - f.at(-raio, -eps, z)).abs();
                    pior = pior.max(d);
                }
            }
            pior
        };
        let grosso = salto(1.0e-4);
        let fino = salto(1.0e-4 / 8.0);
        assert!(
            fino <= grosso / 4.0 + 1.0e-9,
            "({p},{q}): o degrau na costura do `atan2` NÃO encolhe com o passo ({grosso:.7} com \
             `ε`, {fino:.7} com `ε/8`) — a peça está rachada ao longo de `φ = π`"
        );
    }
}

/// ⭐⭐ **O TECTO DA CORDA É ONDE OS FIOS SE TOCAM** — o número não é escolhido, é o sítio de um
/// acontecimento, e este gate mede o acontecimento.
///
/// ⚠️ **A sonda é o ponto MÉDIO entre dois fios vizinhos no plano meridiano**: abaixo do tecto ele
/// está FORA (há vazio entre os fios, e é isso que faz um nó parecer um nó); acima, está DENTRO.
#[test]
fn the_cord_ceiling_is_where_the_strands_touch() {
    let (p, q) = (4_u32, 3_u32);
    let tecto = ph2d_field::knot_cord_ceiling(R, TUBE, p, q);
    // O meio entre o fio `n = 0` e o vizinho dele, no plano `φ = 0`.
    let psi0 = 0.0_f64;
    let passo = std::f64::consts::TAU / f64::from(p);
    let meio = psi0 + passo * 0.5;
    let a = f64::from(TUBE).mul_add(meio.cos(), f64::from(R));
    let sonda = [a, 0.0, f64::from(TUBE) * meio.sin()];
    let leitura = |fraccao: f32| {
        campo(Primitive::TorusKnot {
            radius: R,
            tube: TUBE,
            cord: tecto * fraccao,
            winds: p,
            loops: q,
        })
        .at(sonda[0], sonda[1], sonda[2])
    };
    let abaixo = leitura(0.80);
    assert!(
        abaixo > 0.0,
        "com a corda a 80 % do tecto o meio entre dois fios já está DENTRO ({abaixo:+.5}) — o tecto \
         está alto e a peça deixa de ser um nó antes de o slider acabar"
    );
    // ⚠️ **Acima do tecto o documento RECUSA**, então a leitura é feita na árvore, sem a porta.
    let acima = ph2d_field_eval::ops_knot::sd_torus_knot(
        f64::from(R),
        f64::from(TUBE),
        f64::from(tecto) * 1.6,
        p,
        q,
    );
    let acima = Field::from_tree(&acima).at(sonda[0], sonda[1], sonda[2]);
    assert!(
        acima < 0.0,
        "a 160 % do tecto os fios ainda não se tocaram ({acima:+.5}) — o tecto está baixo e o \
         controlo pára antes de a forma acabar"
    );
}

/// ⭐⭐ **UMA CONTAGEM ESCRITA COM FRACÇÃO ATERRA NUMA CONTAGEM** — e não numa peça rachada.
///
/// ⚠️ **A representação já o garante** (`u32`), e é por isso que este gate mede a PORTA: quem escreve
/// é o painel, com um `f32`, e o que interessa é onde esse `f32` aterra.
#[test]
fn a_count_written_with_a_fraction_lands_on_a_count() {
    let mut p = no(2, 3, 0.5);
    set_dim(&mut p, 0, 3, 2.6).expect("a escrita");
    let Primitive::TorusKnot { winds, .. } = p else {
        panic!("continua um nó")
    };
    assert_eq!(winds, 3, "2,6 tinha de aterrar em 3");
    // E as pontas da faixa são as declaradas.
    let mut p = no(2, 3, 0.5);
    set_dim(&mut p, 0, 3, 1000.0).expect("a escrita");
    let Primitive::TorusKnot { winds, .. } = p else {
        panic!("continua um nó")
    };
    assert_eq!(winds, ph2d_field::MAX_KNOT_WINDS);
    // ⭐⭐ **E o tecto de `q` é RELATIVO a `p`** — o recurso dele é a marcha, que paga a razão.
    let mut p = no(2, 3, 0.5);
    set_dim(&mut p, 0, 4, 1000.0).expect("a escrita");
    let Primitive::TorusKnot { loops, winds, .. } = p else {
        panic!("continua um nó")
    };
    assert_eq!(loops, ph2d_field::max_knot_loops(winds));
    assert_eq!(loops, 2 * ph2d_field::MAX_KNOT_LOOPS_OVER_WINDS);
    // ⚠️ **E DESCER `p` re-assenta `q`** — senão a peça fica fora da faixa que o painel oferece, e
    // a validação recusa-a: é a mesma lei da corda, um andar acima.
    set_dim(&mut p, 0, 3, 1.0).expect("a escrita");
    let Primitive::TorusKnot { loops, winds, .. } = p else {
        panic!("continua um nó")
    };
    assert_eq!(winds, 1);
    assert_eq!(loops, ph2d_field::max_knot_loops(1));
    // ⚠️ **E o NEGATIVO é RECUSADO na porta, não coagido** — a faixa declarada é quem decide, e uma
    // [`ph2d_field::Span::Count`] não admite o zero. ⛔ *Isto não é uma excepção desta forma*: é a
    // mesma lei do `sides` do prisma, e a faixa que o painel oferece nunca chega lá — um negativo
    // só entra por um ficheiro estragado, e aí recusar em voz alta é a resposta certa.
    let mut p = no(2, 3, 0.5);
    assert!(set_dim(&mut p, 0, 3, -5.0).is_err());
    let Primitive::TorusKnot { winds, .. } = p else {
        panic!("continua um nó")
    };
    assert_eq!(winds, 2, "uma escrita recusada não pode mexer na peça");
}

/// ⭐⭐⭐ **SUBIR UMA CONTAGEM RE-ASSENTA A CORDA** — a lei que a W127 pagou: *uma escrita que deixa a
/// peça inválida apaga a CENA INTEIRA.*
///
/// Com a corda no tecto de `(2, q)`, pedir `p = 8` aperta o tecto (mais fios no mesmo tubo). Se a
/// porta não reassentasse a corda, o documento passava a ser recusado pela validação — e o que o
/// artista vê é a peça toda desaparecer ao arrastar um slider.
#[test]
fn raising_a_count_reseats_the_cord() {
    let mut p = no(2, 3, 0.999);
    set_dim(&mut p, 0, 3, 8.0).expect("a escrita");
    let doc = FieldDoc::new(
        vec![Node::new(Xform::IDENTITY, NodeKind::Leaf(p.clone()))],
        NodeId(0),
    );
    assert!(
        doc.is_ok(),
        "subir `p` de 2 para 8 com a corda no tecto deixou a peça INVÁLIDA: {p:?}"
    );
    let Primitive::TorusKnot {
        cord, winds, loops, ..
    } = p
    else {
        panic!("continua um nó")
    };
    assert!(
        cord <= ph2d_field::knot_cord_ceiling(R, TUBE, winds, loops),
        "a corda ficou acima do tecto novo"
    );
    // ⚠️ E a metade que falta: a corda **não** se mexe quando ainda cabe.
    let mut p = no(8, 3, 0.30);
    let Primitive::TorusKnot { cord: antes, .. } = p else {
        panic!("é um nó")
    };
    set_dim(&mut p, 0, 3, 4.0).expect("a escrita");
    let Primitive::TorusKnot { cord: depois, .. } = p else {
        panic!("continua um nó")
    };
    assert!(
        (antes - depois).abs() < 1.0e-9,
        "descer `p` mexeu numa corda que já cabia: {antes} → {depois}"
    );
}

/// ⭐⭐ **ELE NÃO OFERECE FILETE, E A RAZÃO É QUE NÃO TEM ARESTA** — a corda é fechada.
///
/// ⚠️ **As duas pontas**: o documento diz que não há tecto (`round_limit` = `None`) **e** o painel
/// não pinta a linha. *Um controlo pintado sobre um tecto que não existe é um slider morto, que é a
/// falha mais cara de diagnosticar.*
#[test]
fn the_knot_offers_no_fillet_because_it_has_no_edge() {
    let p = no(2, 3, 0.5);
    assert!(ph2d_field::round_limit(&p).is_none());
    let chaves: Vec<&str> = dims(&p).iter().map(|d| d.key).collect();
    assert!(
        !chaves
            .iter()
            .any(|k| k.contains("round") || k.contains("chamfer")),
        "o painel do nó oferece uma linha de aresta: {chaves:?}"
    );
    assert_eq!(
        chaves,
        vec![
            "field.dim.radius",
            "field.dim.thickness",
            "field.dim.cord",
            "field.dim.knot_p",
            "field.dim.knot_q",
        ],
        "a ORDEM das linhas é a identidade delas — o painel manda o índice"
    );
}

/// ⭐⭐⭐ **O TECTO DA CORDA É UM MINORANTE DO ALCANCE DA CURVA** — a fórmula fechada contra a curva
/// que se mede a si própria.
///
/// # ⚠️ O oráculo tem DUAS metades, e a primeira redacção dele tinha uma
///
/// 1. metade da menor distância entre pontos **duplamente críticos** (mínimos locais, longe da
///    diagonal) — a auto-aproximação entre fios;
/// 2. o menor **raio de curvatura** — um tubo mais gordo do que ele cruza-se **sozinho, sem vizinho
///    nenhum**, e é esta metade que manda com `p = 1`.
///
/// ⛔ **Sem a segunda, a fila inteira de `p = 1` passava com o tecto `1,05×` acima do alcance.**
///
/// ⚠️ **`gcd(p, q) > 1` percorre a MESMA curva `g` vezes**, e o alcance de uma curva que se repete é
/// **zero** — o oráculo mede a curva reduzida, que é o desenho que ali sai. *Sem isso metade da
/// grelha lia `inf` e o gate ficava verde por divisão por zero.*
#[test]
fn the_cord_ceiling_is_below_the_curve_that_measures_itself() {
    let n = 2_400_usize;
    let mut pior = (0.0_f64, 0_u32, 0_u32);
    // ⚠️⚠️ **DUAS proporções, e a segunda entrou por uma mutação que SOBREVIVEU.** Num toro FINO a
    // candidata da curvatura nunca decide (as duas de vizinhança são sempre menores), então apagá-la
    // não mudava um único número — *a fixtura não produzia o fenómeno*. É num toro **gordo** que ela
    // manda, e é lá que ela tem de ser medida.
    for (rr, tt) in [(R, TUBE), (0.55_f32, 0.40_f32)] {
        let ponto = |i: usize, p: u32, q: u32| {
            let t = std::f64::consts::TAU * ((i % n) as f64) / (n as f64);
            let (sq, cq) = (f64::from(q) * t).sin_cos();
            let (sp, cp) = (f64::from(p) * t).sin_cos();
            let rad = f64::from(tt).mul_add(cq, f64::from(rr));
            [rad * cp, rad * sp, f64::from(tt) * sq]
        };
        let dist = |a: [f64; 3], b: [f64; 3]| {
            ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
        };
        for pp in 1..=6_u32 {
            for qq in 1..=(4 * pp).min(8) {
                let g = {
                    let (mut a, mut b) = (pp, qq);
                    while b != 0 {
                        let t = b;
                        b = a % b;
                        a = t;
                    }
                    a
                };
                let (p, q) = (pp / g, qq / g);
                let janela = n / 40;
                let mut menor = f64::INFINITY;
                for i in (0..n).step_by(3) {
                    let a = ponto(i, p, q);
                    for j in (i + janela)..(i + n - janela) {
                        let d = dist(a, ponto(j, p, q));
                        if d < dist(a, ponto(j - 1, p, q)) && d < dist(a, ponto(j + 1, p, q)) {
                            menor = menor.min(d);
                        }
                    }
                }
                let mut raio_min = f64::INFINITY;
                for i in 0..n {
                    let (a, b, c) = (ponto(i, p, q), ponto(i + 1, p, q), ponto(i + 2, p, q));
                    let (ab, bc, ca) = (dist(a, b), dist(b, c), dist(c, a));
                    let s = (ab + bc + ca) * 0.5;
                    let area = (s * (s - ab) * (s - bc) * (s - ca)).max(0.0).sqrt();
                    if area > 1.0e-18 {
                        raio_min = raio_min.min(ab * bc * ca / (4.0 * area));
                    }
                }
                let alcance = (menor * 0.5).min(raio_min);
                let tecto = f64::from(ph2d_field::knot_cord_ceiling(rr, tt, pp, qq));
                let razao = tecto / alcance;
                println!(
                    "r={tt:.2} ({pp},{qq}) tecto {tecto:.4} alcance {alcance:.4} razão {razao:.3}"
                );
                if razao > pior.0 {
                    pior = (razao, pp, qq);
                }
            }
        }
    }
    assert!(
        pior.0 <= 1.0,
        "o tecto da corda excede o alcance da curva em {:.3}× na ({}, {}) — no topo do controlo os \
         fios FUNDEM-SE e a peça deixa de ser um nó",
        pior.0,
        pior.1,
        pior.2
    );
}

/// ⭐⭐⭐ **A CORDA É REDONDA — tão grossa ATRAVÉS do fio como AO LONGO dele.**
///
/// # ⛔ Por que este gate nasceu: uma mutação SOBREVIVEU a todos os outros
///
/// Apagar o `sin β` (medir a corda **crua** no plano meridiano, como a `sd_helix` faz) deixa a peça
/// com a curva lá dentro, os fios separados, a costura contínua e o campo 1-Lipschitz — **todos os
/// outros gates verdes**. O que muda é a **SECÇÃO**: um fio inclinado cortado por um plano dá uma
/// **elipse**, e medir nela como se fosse um círculo estica a corda `1/sin β` numa direcção só.
///
/// ⚠️ **Nenhuma régua de campo o vê**, porque não é um defeito do campo: é a forma. A régua é
/// perguntar, no plano PERPENDICULAR ao fio, quanto a peça mede em cada direcção.
#[test]
fn the_cord_is_as_thick_across_the_strand_as_along_it() {
    for (p, q) in [(2_u32, 3_u32), (3, 2), (5, 2), (2, 6)] {
        let no = no(p, q, 0.55);
        let Primitive::TorusKnot { cord, .. } = no else {
            panic!("é um nó")
        };
        let f = campo(no);
        let t = 0.7_f64;
        let c = ponto_da_curva(t, p, q);
        // A tangente, por diferença central sobre a própria definição.
        let h = 1.0e-5_f64;
        let (a, b) = (ponto_da_curva(t - h, p, q), ponto_da_curva(t + h, p, q));
        let u = {
            let v = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let n = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
            [v[0] / n, v[1] / n, v[2] / n]
        };
        // Dois eixos perpendiculares ao fio.
        let cruz = |x: [f64; 3], y: [f64; 3]| {
            [
                x[1] * y[2] - x[2] * y[1],
                x[2] * y[0] - x[0] * y[2],
                x[0] * y[1] - x[1] * y[0],
            ]
        };
        let unit = |v: [f64; 3]| {
            let n = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
            [v[0] / n, v[1] / n, v[2] / n]
        };
        let e1 = unit(cruz(u, [0.0, 0.0, 1.0]));
        let e2 = unit(cruz(u, e1));
        let (mut menor, mut maior) = (f64::INFINITY, 0.0_f64);
        for i in 0..48 {
            let a = std::f64::consts::TAU * f64::from(i) / 48.0;
            let (s, co) = a.sin_cos();
            let dir = [
                e1[0] * co + e2[0] * s,
                e1[1] * co + e2[1] * s,
                e1[2] * co + e2[2] * s,
            ];
            // Bisseca a fronteira ao longo desta direcção.
            let (mut lo, mut hi) = (0.0_f64, f64::from(cord) * 4.0);
            for _ in 0..40 {
                let m = 0.5 * (lo + hi);
                let v = f.at(c[0] + dir[0] * m, c[1] + dir[1] * m, c[2] + dir[2] * m);
                if v < 0.0 { lo = m } else { hi = m }
            }
            menor = menor.min(lo);
            maior = maior.max(lo);
        }
        println!("({p},{q}) excentricidade {:.4}", maior / menor);
        // ⭐ **A BARRA SAI DO VAZIO ENTRE OS DOIS LADOS**, e não de um palpite (a pior das quatro
        // peças do corpus):
        //
        // | | produto | sem o encolher do eixo | sem a correcção de curvatura | com a inclinação lida no PONTO |
        // |---|---:|---:|---:|---:|
        // | hoje | **`1,050`** | `1,329` | `1,139` | `1,079` |
        //
        // ⛔ E ela desceu de `1,15` para `1,10` no dia em que o tecto da pegada apertou o produto:
        // *uma barra que o produto folgou de deixar para trás é uma barra que parou de medir.*
        assert!(
            maior / menor < 1.10,
            "({p},{q}): a secção da corda é uma ELIPSE de {maior:.4} por {menor:.4} \
             ({:.3}×) — a corda foi medida no plano do corte em vez de perpendicular ao fio",
            maior / menor
        );
    }
}

/// ⭐⭐⭐ **TODA CANDIDATA DO TECTO DA CORDA DECIDE ALGUMA VEZ** — o censo de obsolescência da lista
/// de cercas.
///
/// # ⛔⛔ Por que ele nasceu: uma candidata MORTA sobreviveu a duas rondas de mutação
///
/// O tecto é o `min` de várias cercas. Uma que **nunca** é o mínimo é invisível a toda régua de
/// resultado — apagá-la não muda um único número, logo a mutação que a apaga **sobrevive**, e a
/// leitura natural (*«falta um gate»*) manda escrever um gate para uma lei que já está gateada.
///
/// Foi o que aconteceu com a **curvatura da própria corda**: cerca legítima em teoria, mínimo em
/// **`0` de `486 720` células**. ⇒ *num `min` de cercas, a pergunta que separa «falta gate» de
/// «código morto» é «quantas células ela decide?»*, e ela tem de ser feita por um teste.
///
/// ⚠️ **A metade justa da catraca**: este censo também reprova se alguém acrescentar uma candidata
/// que nunca morde — e é aí que ele paga o preço dele.
#[test]
fn every_candidate_of_the_cord_ceiling_decides_somewhere() {
    let (mut fios, mut voltas, mut furo, mut total) = (0_u32, 0_u32, 0_u32, 0_u32);
    for ri in 1..=40 {
        let radius = 0.05 * f64::from(ri);
        for ti in 1..40 {
            let tube = radius * 0.025 * f64::from(ti);
            for p in 1..=ph2d_field::MAX_KNOT_WINDS {
                for q in 1..=ph2d_field::max_knot_loops(p) {
                    let (pf, qf) = (f64::from(p), f64::from(q));
                    let k = tube * qf / pf;
                    let dentro = radius - tube;
                    let sb = dentro / dentro.hypot(k);
                    let cb = k / dentro.hypot(k);
                    let a = if p <= 1 {
                        tube
                    } else {
                        tube * (std::f64::consts::PI / pf).sin() * sb
                    };
                    let b = std::f64::consts::PI * dentro * pf / qf * cb;
                    let d = dentro;
                    let m = a.min(b).min(d);
                    total += 1;
                    if (m - a).abs() < 1.0e-12 {
                        fios += 1;
                    } else if (m - b).abs() < 1.0e-12 {
                        voltas += 1;
                    } else {
                        furo += 1;
                    }
                }
            }
        }
    }
    println!("de {total} células: entre_fios {fios} · entre_voltas {voltas} · furo {furo}");
    for (nome, n) in [
        ("entre_fios", fios),
        ("entre_voltas", voltas),
        ("furo", furo),
    ] {
        assert!(
            n > 0,
            "a candidata `{nome}` do tecto da corda NUNCA é o mínimo, sobre {total} células — ela é \
             código morto, e apagá-la é uma mutação que sobrevive a todos os outros gates"
        );
    }
}

/// ⭐⭐⭐ **A CORDA NÃO TEM COSTURA AO COMPRIDO** — o gate que o report do Enio de 07/09 obrigou a
/// escrever (*«Torus Knot não é perfeito»*, com foto).
///
/// # ⚠️ O que ele mede, e as DUAS réguas que morreram antes desta
///
/// A primeira perguntava *«qual TERMO decide aqui?»* e acusava a casca do toro em `60` de `60`
/// secções — e deixou de medir seja o que for no dia em que a casca mudou de sítio. *Uma régua que
/// mede o NOME do termo morre quando o termo muda de sítio; a que mede a superfície não.*
///
/// ⛔⛔ A segunda **bissecava num intervalo largo**, e isso pressupõe que o campo muda de sinal uma
/// vez: ela convergia para a superfície do fio **VIZINHO** e acusava `87,5°` numa peça cuja secção é
/// redonda a `1,0001`. ⇒ acha-se a **primeira** travessia a passo, e só depois se bisseca nela.
///
/// # A catraca, e por que ela é por PAR
///
/// A costura **não é uniforme na família**: ela cresce com o quão inclinado o fio corre, e o modelo
/// (um cruzamento por plano meridiano, com o eixo encolhido pela inclinação) é de 1.ª ordem nisso.
/// ⛔ **Uma barra única ou seria vácua para os pares bons ou proibiria metade da família** — o
/// cinquefoil `(2,5)` e o `7₁ (2,7)` são nós clássicos, não casos de canto.
///
/// ⇒ cada par carrega o número que hoje MEDE, com folga de `2°`. Uma regressão reprova; uma
/// melhoria também reprova, **e é assim que a tabela desce** em vez de envelhecer.
#[test]
fn the_cord_has_no_seam_running_along_it() {
    // ⚠️ **A tabela é o estado MEDIDO em 07/09**, com a folga de `2°`. Uma secção lisa amostrada em
    // `24` direcções dá `15,0°` — é esse o chão, e não zero.
    for (p, q, medido) in [
        (2_u32, 3_u32, 16.1_f64),
        (3, 2, 15.1),
        (5, 2, 15.0),
        (2, 5, 19.1),
        (1, 2, 17.2),
        (1, 3, 21.4),
        (1, 4, 27.5),
        (2, 8, 26.4),
        (8, 32, 21.1),
    ] {
        let peca = no(p, q, 0.95);
        let f = campo(peca);
        let unit = |v: [f64; 3]| {
            let n = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2])
                .sqrt()
                .max(1.0e-30);
            [v[0] / n, v[1] / n, v[2] / n]
        };
        let cruz = |x: [f64; 3], y: [f64; 3]| {
            [
                x[1] * y[2] - x[2] * y[1],
                x[2] * y[0] - x[0] * y[2],
                x[0] * y[1] - x[1] * y[0],
            ]
        };
        let mut pior = 0.0_f64;
        for ti in 0..24 {
            let t = std::f64::consts::TAU * f64::from(ti) / 24.0;
            let c = ponto_da_curva(t, p, q);
            let h = 1.0e-5;
            let (aa, bb) = (ponto_da_curva(t - h, p, q), ponto_da_curva(t + h, p, q));
            let u = unit([bb[0] - aa[0], bb[1] - aa[1], bb[2] - aa[2]]);
            let e1 = unit(cruz(u, [0.0, 0.0, 1.0]));
            let e2 = unit(cruz(u, e1));
            let mut anterior: Option<[f64; 3]> = None;
            for i in 0..24 {
                let ang = std::f64::consts::TAU * f64::from(i) / 24.0;
                let (s, co) = ang.sin_cos();
                let d = [
                    e1[0] * co + e2[0] * s,
                    e1[1] * co + e2[1] * s,
                    e1[2] * co + e2[2] * s,
                ];
                // ⚠️ A PRIMEIRA travessia, achada a passo — ver o cabeçalho.
                let passo = 0.002_f64;
                let mut lo = 0.0_f64;
                let mut hi = passo;
                while hi < 0.5 && f.at(c[0] + d[0] * hi, c[1] + d[1] * hi, c[2] + d[2] * hi) < 0.0 {
                    lo = hi;
                    hi += passo;
                }
                for _ in 0..40 {
                    let m = 0.5 * (lo + hi);
                    if f.at(c[0] + d[0] * m, c[1] + d[1] * m, c[2] + d[2] * m) < 0.0 {
                        lo = m;
                    } else {
                        hi = m;
                    }
                }
                let pt = [c[0] + d[0] * lo, c[1] + d[1] * lo, c[2] + d[2] * lo];
                let eps = 1.0e-5;
                let n = unit([
                    f.at(pt[0] + eps, pt[1], pt[2]) - f.at(pt[0] - eps, pt[1], pt[2]),
                    f.at(pt[0], pt[1] + eps, pt[2]) - f.at(pt[0], pt[1] - eps, pt[2]),
                    f.at(pt[0], pt[1], pt[2] + eps) - f.at(pt[0], pt[1], pt[2] - eps),
                ]);
                if let Some(a) = anterior {
                    let cosang = n[0] * a[0] + n[1] * a[1] + n[2] * a[2];
                    pior = pior.max(cosang.clamp(-1.0, 1.0).acos().to_degrees());
                }
                anterior = Some(n);
            }
        }
        println!("  ({p},{q}) pior salto {pior:.1}° (tabela {medido:.1}°)");
        assert!(
            pior <= medido + 2.0,
            "({p},{q}): a normal salta {pior:.1}° e a tabela diz {medido:.1}° — a costura ao \
             comprido da corda PIOROU"
        );
        assert!(
            pior >= medido - 2.0,
            "({p},{q}): a costura MELHOROU para {pior:.1}° e a tabela ainda diz {medido:.1}° — \
             desça a tabela, senão ela vira licença"
        );
    }
}

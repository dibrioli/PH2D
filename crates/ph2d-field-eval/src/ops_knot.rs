//! ⭐⭐⭐ **O NÓ DE TORO `(p, q)`** (W134) — a corda que dá `p` voltas ao eixo enquanto dá `q` voltas
//! ao tubo, sem um único segmento desenhado.
//!
//! # ⭐⭐ A metade que a torna exprimível: num plano meridiano há EXACTAMENTE `p` fios
//!
//! A curva é `φ = p·t`, `ψ = q·t`, com `φ` o ângulo em torno do eixo e `ψ` o ângulo em torno do
//! tubo. Fixado `φ`, os parâmetros que lá passam são `t_n = (φ + 2πn)/p` para `n = 0..p−1` — logo o
//! plano meridiano daquele `φ` é cortado por **`p` pontos**, nos ângulos `ψ_n = q·(φ + 2πn)/p`.
//!
//! ⇒ a distância ao nó é o **`min` sobre `p` ramos**, e cada ramo é uma distância no plano. É a
//! mesma forma do polígono da W132 (`min` sobre segmentos), e não a do arredondamento da espiral.
//!
//! ⭐⭐⭐ **E é por isso que a costura do `atan2` não existe aqui.** Na W128 um `m` fraccionário
//! rachava a peça porque a fórmula lia `φ` **cru**; aqui `φ` só entra dentro do conjunto
//! `{ψ_n}`, e sob `φ → φ − 2π` esse conjunto **é o mesmo** (`n` desliza um, e `n = −1` reencontra
//! `n = p−1` a menos de `2πq`, que é zero módulo `2π`). *Um `min` sobre um conjunto invariante é
//! contínuo mesmo quando cada termo não é.*
//!
//! ⚠️ **E `p` e `q` são `u32`, não `f32`**: aqui a lição da W128 não se paga com coerção — a
//! contagem de ramos de uma árvore **não pode** ser fraccionária, então o estado inválido é
//! inexprimível. *A representação apaga o caso especial.*
//!
//! # ⚠️ O divisor, e de que recurso ele é
//!
//! `g` mede no plano meridiano, e o alvo dele **move-se com `φ`**: `dψ_n/dφ = q/p`, logo o alvo
//! anda `r·q/p` por radiano de `φ`, e `‖∇φ‖ = 1/ρ`. Como a direcção meridiana e a azimutal são
//! ortogonais,
//!
//! ```text
//! ‖∇g‖ = √(1 + (r·q/(p·ρ))²)   ≤   √(1 + (r·q/(p·ρ_min))²)
//! ```
//!
//! com `ρ_min = R − r − cord`, que é **o menor raio onde há matéria**. Dividir por esse majorante dá
//! um **minorante** honesto da distância — que é tudo o que a marcha pede (doc 06 §124). ⚠️ Mais
//! para dentro do que `ρ_min` o quociente subestima ainda mais, e *uma inexactidão que subestima é
//! folga*.

use fidget::context::Tree;

use crate::ops::safe_sqrt;

/// ⭐⭐⭐ **O MAJORANTE DE `‖∇f‖` DESTA PEÇA** — e o campo é dividido por ele.
///
/// # ⭐⭐ Escalar o campo INTEIRO não move a superfície
///
/// `λ·(m − corda)` tem **exactamente** o mesmo conjunto-zero que `m − corda`: a peça é a mesma, e só
/// o *ritmo* com que o campo cresce muda. ⚠️ **Isto é o oposto de encolher o divisor**, que foi a
/// primeira tentativa desta wave e que ENGORDA a peça — um minorante frouxo composto com a
/// subtracção da corda desloca a superfície para fora, e a `(2,3)` saía um toro maciço.
/// *`(minorante) − raio` não é o offset de `raio`.*
///
/// # A conta, e por que ela depende da SATURAÇÃO
///
/// Junto da superfície a parte **azimutal** é a que estoura: o referencial meridiano roda com `φ` à
/// taxa `q/p`, e `‖∇φ‖ = 1/ρ`. Com as duas componentes saturadas em `S` (o tecto da corda, que é a
/// meia-distância entre fios — ver [`ph2d_field::knot_cord_ceiling`]):
///
/// ```text
/// |∂m/∂ψ| ≤ S/c + c·(S + r)         com c = ρ/√(ρ² + K²),  K = r·q/p
/// azimutal ≤ (q/p)/ρ · (S/c + c·(S + r))
/// ‖∇f‖ ≤ √(1 + azimutal²)
/// ```
///
/// ⚠️ `ρ = R − r − corda` é o **menor raio onde há matéria**, e é lá que a fracção é maior.
///
/// ⛔⛔ **A primeira redacção derivava com `m ≈ corda` e o censo apanhou-a nas DUAS pontas**: ela
/// lia `1,14` na peça representativa e **`5,65`** ao arrastar o raio para dentro do tubo. *Uma
/// derivação «junto da superfície» não descreve a banda em que a marcha decide* — e a saturação, que
/// entrou para curar o eixo, é justamente o que a torna derivável em toda a parte.
#[must_use]
pub fn knot_gradient_bound(radius: f64, tube: f64, cord: f64, winds: u32, loops: u32) -> f64 {
    let (p, q) = (f64::from(winds.max(1)), f64::from(loops.max(1)));
    let k = tube * q / p;
    let s = f64::from(saturacao(radius, tube, winds, loops));
    let rho = (radius - tube - cord).max(cord.max(tube * 0.02));
    let c = rho / rho.hypot(k);
    let azimutal = (q / p) / rho * (s / c + c * (s + tube));
    KNOT_BOUND_MARGIN * 1.0_f64.hypot(azimutal)
}

/// ⭐⭐ **ONDE AS DUAS COMPONENTES SATURAM** — a meia-distância entre fios vizinhos.
///
/// ⚠️ **Além dela o valor não quer dizer nada**: o ponto já está mais perto de OUTRO fio, e o ramo
/// dele é que responde. Saturar ali não perde nada e é o que torna o majorante acima finito junto
/// do eixo. ⭐ **É a mesma função que o painel usa para a parede da corda** — *uma lei, uma porta.*
fn saturacao(radius: f64, tube: f64, winds: u32, loops: u32) -> f32 {
    #[allow(clippy::cast_possible_truncation)]
    ph2d_field::knot_cord_ceiling(radius as f32, tube as f32, winds, loops)
}

/// ⭐ **O que a conta acima ainda SUBESTIMA, medido.**
///
/// A derivação é um majorante de termos independentes, e nenhum ponto os atinge todos ao mesmo
/// tempo. Medido em `probe_knot_grid` (a corda a `60 %` do tecto, sobre as `144` células da grelha
/// `p, q ∈ 1..12`) e no censo: com margem `1,00` a pior célula lê `1,18`; com **`1,25`** todas as
/// `144` lêem `1,00`.
///
/// ⇒ *um máximo AMOSTRADO que vira limite de segurança erra sempre PARA BAIXO*. ⛔ E ele custa
/// apenas passos **dentro da coroa** — longe da peça quem manda é a casca do toro, que não é
/// multiplicada por nada.
pub const KNOT_BOUND_MARGIN: f64 = 1.25;

/// ⭐⭐⭐ **NÓ DE TORO** — a corda de raio `cord` que percorre a superfície do toro `(radius, tube)`
/// dando `winds` voltas ao eixo e `loops` voltas ao tubo.
///
/// ⚠️ **Fechada por construção, logo SEM ARESTA**: não há corte, não há tampa, não há aro — e é por
/// isso que esta primitiva não tem filete nem chanfro (ver [`ph2d_field::round_limit`], que devolve
/// `None` para ela).
///
/// ⚠️ **`winds` de zero não existe** — ele é o número de ramos do `min`, e a porta do documento
/// obriga-o a `≥ 1` ([`ph2d_field::MIN_KNOT_WINDS`]).
///
/// ⛔ **`gcd(p, q) > 1` não é recusado, e não é esquecimento:** ali a curva fecha antes de gastar os
/// `p` ramos e o desenho degenera para o nó `(p/g, q/g)` percorrido `g` vezes — que é uma peça
/// válida, com os fios coincidentes. *Recusar seria proibir uma forma por causa do nome dela.*
#[must_use]
pub fn sd_torus_knot(radius: f64, tube: f64, cord: f64, winds: u32, loops: u32) -> Tree {
    let tau = std::f64::consts::TAU;
    let p = f64::from(winds.max(1));
    let q = f64::from(loops);
    let rho = safe_sqrt(Tree::x().square() + Tree::y().square());
    let phi = Tree::y().atan2(Tree::x());
    // As coordenadas do ponto no plano meridiano: `a` para fora do anel, `b` para cima.
    let a = rho.clone() - Tree::constant(radius);
    let b = Tree::z();
    // ⭐⭐⭐ O SENO DA INCLINAÇÃO do fio — ver o cabeçalho. Ele encolhe **um** eixo, e é local.
    let k = tube * q / p;
    let c = rho.clone() / safe_sqrt(rho.clone().square() + Tree::constant(k * k));
    // ⭐ Os `p` fios que cortam este plano — ver o cabeçalho.
    let mut melhor: Option<Tree> = None;
    for n in 0..winds.max(1) {
        let psi = (phi.clone() + Tree::constant(tau * f64::from(n))) * Tree::constant(q / p);
        let (cs, sn) = (psi.clone().cos(), psi.sin());
        let va = a.clone() - cs.clone() * Tree::constant(tube);
        let vb = b.clone() - sn.clone() * Tree::constant(tube);
        // ⭐⭐ As DUAS componentes: `radial` atravessa o fio, `ao_longo` corre com ele.
        let radial = va.clone() * cs.clone() + vb.clone() * sn.clone();
        let ao_longo = vb * cs - va * sn;
        // ⭐⭐⭐ **SATURADAS NA MEIA-DISTÂNCIA ENTRE FIOS** — e isto não toca na peça: na superfície
        // as duas valem no máximo a corda, que é sempre `≤` o tecto. ⛔ **Fora** dela é que elas
        // cresciam com a distância ao eixo e rodavam com `φ` a `q/(p·ρ)` por unidade de comprimento
        // — era daí que vinha `‖∇f‖ = 3,59` na `(1,1)`, junto do eixo. Saturar **só baixa** o valor,
        // logo o minorante continua minorante, e o zero fica onde estava.
        let teto = f64::from(saturacao(radius, tube, winds, loops));
        let sat = |t: Tree| t.max(Tree::constant(-teto)).min(Tree::constant(teto));
        let d = safe_sqrt(sat(radial).square() + sat(ao_longo * c.clone()).square());
        melhor = Some(match melhor {
            None => d,
            Some(m) => m.min(d),
        });
    }
    let melhor = melhor.expect("winds >= 1 garante pelo menos um ramo");
    // ⭐⭐⭐ O campo INTEIRO dividido pelo majorante — o zero não se mexe, o ritmo sim.
    let fio = (melhor - Tree::constant(cord))
        * Tree::constant(1.0 / knot_gradient_bound(radius, tube, cord, winds, loops));
    // ⭐⭐⭐ **A CASCA DO TORO, que é EXACTA** — ver o cabeçalho: fora da coroa o divisor constante
    // deixa de valer, e é este termo que manda lá.
    let casca =
        (safe_sqrt(a.square() + b.square()) - Tree::constant(tube)).abs() - Tree::constant(cord);
    fio.max(casca)
}

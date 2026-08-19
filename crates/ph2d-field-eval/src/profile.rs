//! **O perfil 2D como árvore de avaliação** — e as duas formas que ele gera: `extrude` e `revolve`.
//!
//! ⚠️ `DIRETIVA_IMPLEMENTACAO` §1, como no [`crate::ops`]: nada aqui é inventado. A distância
//! ponto-segmento é a fórmula do [*2D distance functions*](https://iquilezles.org/articles/distfunctions2d/)
//! de Inigo Quilez; o teste de dentro/fora é o **winding number de Dan Sunday** (a variante
//! semi-aberta do *crossing number*, que é a mesma que o `ph2d_vec_scene::inside` da casa usa —
//! duas implementações da mesma regra, e é de propósito que a regra seja **a mesma**).
//!
//! # A parte que exigiu derivação: o sinal, sem ramificação
//!
//! Um algoritmo de winding number é um `for` com um `if` e um acumulador. Uma árvore de avaliação
//! **não tem `if`** — ela tem `compare`, que devolve −1/0/+1. A tradução:
//!
//! ```text
//! acima_i  = max(compare(y_i, v), 0)          1 se o vértice i está acima do ponto
//! dir      = acima_j − acima_i                +1 subindo · −1 descendo · 0 sem cruzar
//! cross    = e_x·w_y − e_y·w_x                de que lado da aresta o ponto está
//! hit      = max(compare(dir·cross, 0), 0)    1 sse cruza E o raio +x o alcança
//! ```
//!
//! ⭐ **`dir · cross > 0` casa os dois sentidos de uma vez.** Uma aresta que sobe é cruzada pelo
//! raio `+x` quando o ponto está à esquerda dela (`cross > 0`); uma que desce, quando está à direita
//! (`cross < 0`). Multiplicar pelo sentido colapsa os dois `if` do algoritmo original num só
//! `compare` — e, de quebra, **elimina a divisão** que a forma ingénua faz para achar o `x` do
//! cruzamento (`t = (v − a_y)/(b_y − a_y)`), que numa árvore seria avaliada em **todas** as arestas,
//! inclusive nas horizontais, onde é `0/0`.
//!
//! `acima_i` é calculado **uma vez por vértice** e usado pelas duas arestas que o tocam — metade dos
//! `compare` de graça.
//!
//! # ⚠️ O que o intervalo faz com isto, e por que não morde onde importa
//!
//! O sinal é uma função **descontínua**: sobre uma região que atravessa a fronteira, a aritmética de
//! intervalo devolve `[−1, 1]`, e o produto com a distância fica frouxo. Isso é irrelevante para o
//! **traçado**, que é o caminho pelo qual o artista vê a peça: ele avalia ponto a ponto
//! (`float_slice_tape`), onde não há intervalo nenhum e o valor é exato. Quem paga é a **malhagem**,
//! que poda o octree por intervalo — e a malha é o artefato de exportação (ADR-0161 §2). O número
//! está medido em `docs/3DModeling/01_resultados_spike.md`.

use fidget::context::Tree;
use ph2d_field::{FillRule, Profile};

/// A **distância com sinal** do ponto `(u, v)` à figura do perfil — negativa dentro.
///
/// `u` e `v` são árvores, e não `x`/`y` fixos, porque é isso que deixa o mesmo perfil servir ao
/// `extrude` (que passa `x`, `y`) e ao `revolve` (que passa `√(x²+z²)`, `y`) **sem uma segunda
/// cópia da fórmula**.
#[must_use]
pub fn sd_profile(profile: &Profile, u: &Tree, v: &Tree) -> Tree {
    let non_zero = profile.fill() == FillRule::NonZero;
    let mut dist2: Option<Tree> = None;
    let mut crossings: Option<Tree> = None;

    for contour in profile.contours() {
        let n = contour.len();
        // Um `compare` por VÉRTICE, partilhado pelas duas arestas que o tocam.
        let above: Vec<Tree> = contour
            .iter()
            .map(|p| Tree::constant(f64::from(p[1])).compare(v.clone()).max(0.0))
            .collect();

        for i in 0..n {
            let j = (i + 1) % n;
            let (ax, ay) = (f64::from(contour[i][0]), f64::from(contour[i][1]));
            let (bx, by) = (f64::from(contour[j][0]), f64::from(contour[j][1]));
            let (ex, ey) = (bx - ax, by - ay);
            // `Profile::new` removeu os pontos consecutivos repetidos, logo a aresta tem
            // comprimento — e o recíproco abaixo é uma CONSTANTE, calculada aqui e nunca no ponto.
            let inv_ee = 1.0 / (ex * ex + ey * ey);

            let wx = u.clone() - Tree::constant(ax);
            let wy = v.clone() - Tree::constant(ay);

            // A projeção do ponto no segmento, presa a [0, 1] — é o `clamp` que faz a fórmula valer
            // para o segmento e não para a reta infinita dele.
            let h = ((wx.clone() * Tree::constant(ex) + wy.clone() * Tree::constant(ey))
                * Tree::constant(inv_ee))
            .max(0.0)
            .min(1.0);
            let qx = wx.clone() - h.clone() * Tree::constant(ex);
            let qy = wy.clone() - h * Tree::constant(ey);
            let seg2 = qx.square() + qy.square();
            dist2 = Some(match dist2 {
                None => seg2,
                Some(acc) => acc.min(seg2),
            });

            let dir = above[j].clone() - above[i].clone();
            let cross = Tree::constant(ex) * wy - Tree::constant(ey) * wx;
            let hit = (dir.clone() * cross).compare(0.0).max(0.0);
            let term = if non_zero { dir * hit } else { hit };
            crossings = Some(match crossings {
                None => term,
                Some(acc) => acc + term,
            });
        }
    }

    // `Profile` garante ≥1 contorno com ≥3 pontos: os dois acumuladores existem.
    let (dist2, crossings) = (
        dist2.expect("um perfil válido tem ao menos uma aresta"),
        crossings.expect("um perfil válido tem ao menos uma aresta"),
    );

    // ⚠️ `crossings` é um INTEIRO exato (soma de ±1), então as duas reduções abaixo são exatas:
    // `min(|w|, 1)` vale 1 para qualquer enrolamento não-nulo, e o resto euclidiano por 2 é a
    // paridade. Nenhuma delas precisa de tolerância — e uma tolerância aqui seria um número
    // inventado a defender uma conta que já é exata.
    let inside = if non_zero {
        crossings.abs().min(1.0)
    } else {
        crossings.modulo(2.0)
    };
    let sign = Tree::constant(1.0) - Tree::constant(2.0) * inside;
    dist2.sqrt() * sign
}

/// **O perfil puxado ao longo de Z**, com o aro arredondado em `round`.
///
/// A receita é a mesma da caixa ([`crate::ops::sd_box`]) com uma dimensão a menos: encolher a fonte
/// em `round` — no plano *e* na altura — e deslocar a superfície de volta. É o que faz o raio ser
/// **exatamente** o pedido, e não uma aproximação.
///
/// ⚠️ Encolher o perfil é uma **abertura morfológica**: um pescoço mais fino que `2·round`
/// desaparece. É o comportamento correto de arredondar com esse raio, e é o mesmo que qualquer CAD
/// faz — não é um caso de erro, e por isso o documento não o recusa.
#[must_use]
pub fn sd_extrude(profile: &Profile, half_height: f64, round: f64) -> Tree {
    let flat = sd_profile(profile, &Tree::x(), &Tree::y());
    if round <= 0.0 {
        // ⚠️ Caminho DURO de propósito, pelo mesmo motivo do `ops::union`: com `round = 0` a versão
        // arredondada é algebricamente idêntica, e paga dois nós a mais **por amostra** — e o
        // traçado avalia milhões de amostras por quadro.
        let w = Tree::z().abs() - Tree::constant(half_height);
        let outside = (flat.max(0.0).square() + w.max(0.0).square()).sqrt();
        return outside + flat.max(w).min(0.0);
    }
    let d = flat + Tree::constant(round);
    let w = Tree::z().abs() - Tree::constant(half_height - round);
    let outside = (d.max(0.0).square() + w.max(0.0).square()).sqrt();
    outside + d.max(w).min(0.0) - Tree::constant(round)
}

/// **O perfil girado em torno do eixo Y.**
///
/// ⭐ A substituição `x → √(x² + z²)` dá a distância **exata**, e não uma aproximação: o ponto da
/// superfície mais próximo de `p` está no mesmo semiplano que `p`, porque girar um ponto da
/// superfície em direção a esse semiplano preserva raio e altura e só reduz a separação angular.
/// Por isso a distância 3D é literalmente a distância 2D no plano `(r, y)`.
///
/// Vale enquanto o perfil não cruzar o eixo — o que `ph2d_field::FieldError::ProfileCrossesAxis`
/// garante no documento, antes de qualquer avaliação.
#[must_use]
pub fn sd_revolve(profile: &Profile) -> Tree {
    let r = (Tree::x().square() + Tree::z().square()).sqrt();
    sd_profile(profile, &r, &Tree::y())
}

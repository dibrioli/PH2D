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
//! intervalo devolve `[−1, 1]`, e o produto com a distância fica frouxo. Isso é irrelevante para os
//! **dois** consumidores que existem hoje, e é por medição: o traçado avalia ponto a ponto
//! (`float_slice_tape`) e a extração da W20 varre uma grade **uniforme**, também ponto a ponto —
//! nenhum dos dois pergunta um intervalo a esta árvore. ⚠️ *Quem reabrir a poda por intervalos na
//! extração (`ph2d_field_eval::extract`, secção "o que ele NÃO é") herda este parágrafo de volta.*

use fidget::context::Tree;
use ph2d_field::{FillRule, Profile};

/// A **distância com sinal** do ponto `(u, v)` à figura do perfil — negativa dentro.
///
/// `u` e `v` são árvores, e não `x`/`y` fixos, porque é isso que deixa o mesmo perfil servir ao
/// `extrude` (que passa `x`, `y`) e ao `revolve` (que passa `√(x²+z²)`, `y`) **sem uma segunda
/// cópia da fórmula**.
#[must_use]
pub fn sd_profile(profile: &Profile, u: &Tree, v: &Tree) -> Tree {
    sd_profile_inner(profile, u, v, false)
}

/// A mesma coisa, com uma opção: `axis_seam` tira as arestas que **assentam no eixo** da conta da
/// distância — e só dela.
///
/// # ⭐ Por que uma aresta no eixo não é superfície
///
/// Um torno gira o perfil em torno de `x = 0`. Uma aresta com `x > 0` varre um **anel**: é
/// superfície. Uma aresta **sobre** o eixo varre uma **linha** — medida zero, superfície nenhuma. Ela
/// existe no desenho porque um contorno tem de fechar, e é a costura do desenho, não uma parede da
/// peça.
///
/// ⚠️ **Deixá-la na conta põe um nível zero DENTRO do sólido**, ao longo do eixo, e o efeito é
/// exatamente o de uma parede que não existe: a extração encontra ali uma superfície e malha-a.
/// Medido no vaso da cena 5 (§21): sobre o eixo o campo lia `f = −0,0000` com `‖∇f‖ = 0` onde devia
/// ler `−0,02 … −0,08`, e a malha saía com um leque de lascas em `r ≈ 0,2`, `y = −0,45`.
///
/// ⚠️ **Só a DISTÂNCIA muda; o enrolamento continua a ver a aresta inteira** — é ele que sabe o que é
/// dentro, e tirar a costura de lá abriria o contorno e inverteria o sinal de meia peça.
fn sd_profile_inner(profile: &Profile, u: &Tree, v: &Tree, axis_seam: bool) -> Tree {
    let non_zero = profile.fill() == FillRule::NonZero;
    // A mesma tolerância com que o perfil foi achatado: um vértice a menos que isso do eixo **é** o
    // eixo, e um número próprio aqui seria uma segunda resposta a "o que encosta no eixo".
    let on_axis = f64::from(profile.tolerance());
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
            if !(axis_seam && ax.abs() <= on_axis && bx.abs() <= on_axis) {
                dist2 = Some(match dist2 {
                    None => seg2,
                    Some(acc) => acc.min(seg2),
                });
            }

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

    // `Profile` garante ≥1 contorno com ≥3 pontos: o acumulador do enrolamento existe sempre.
    let crossings = crossings.expect("um perfil válido tem ao menos uma aresta");
    // ⚠️ O da distância pode não existir — se `axis_seam` tirou TODAS as arestas, o perfil é um
    // segmento sobre o eixo, e a revolução dele é uma linha. Recair na conta completa é o que faz
    // esse caso degenerado continuar a devolver o mesmo que devolvia, em vez de entrar em pânico.
    let Some(dist2) = dist2 else {
        return sd_profile_inner(profile, u, v, false);
    };

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
    crate::ops::safe_sqrt(dist2) * sign
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
pub fn sd_extrude(profile: &Profile, half_height: f64, round: f64, chamfer: f64) -> Tree {
    extrude_from(
        &sd_profile(profile, &Tree::x(), &Tree::y()),
        half_height,
        round,
        chamfer,
    )
}

/// ⭐ **A casca da extrusão sobre um perfil já baixado** — a metade que não conhece as arestas.
///
/// ⚠️ Ela existe para a [`crate::compile_in_region`] poder trocar a metade PLANA por uma versão
/// especializada sem uma segunda cópia desta receita. *Duas cópias da casca divergiriam no dia em
/// que o filete mudasse, e só uma das formas de perfil o notaria.*
#[must_use]
pub fn extrude_from(flat: &Tree, half_height: f64, round: f64, chamfer: f64) -> Tree {
    let flat = flat.clone();
    if chamfer > 0.0 {
        // ⭐ **O aro é a junta de DUAS peças** — a parede (o contorno) e a laje —, e é a mesma forma
        // que o [`crate::ops::slab_and_walls`] usa para o resto da família. ⚠️ O caminho abaixo fica
        // intocado: com `chamfer = 0` esta forma nem é construída.
        let laje = Tree::z().abs() - Tree::constant(half_height);
        return crate::ops_joint::intersection_joint(
            &flat,
            &laje,
            crate::ops_joint::Edge::square(round, chamfer),
        );
    }
    if round <= 0.0 {
        // ⚠️ Caminho DURO de propósito, pelo mesmo motivo do `ops::union`: com `round = 0` a versão
        // arredondada é algebricamente idêntica, e paga dois nós a mais **por amostra** — e o
        // traçado avalia milhões de amostras por quadro.
        let w = Tree::z().abs() - Tree::constant(half_height);
        let outside = crate::ops::safe_sqrt(flat.max(0.0).square() + w.max(0.0).square());
        return outside + flat.max(w).min(0.0);
    }
    let d = flat + Tree::constant(round);
    let w = Tree::z().abs() - Tree::constant(half_height - round);
    let outside = crate::ops::safe_sqrt(d.max(0.0).square() + w.max(0.0).square());
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
///
/// ⚠️ **E vale sobre as arestas que de facto varrem superfície.** A frase «a distância 3D é a
/// distância 2D no plano (r, y)» é verdadeira para o **contorno**, e o contorno não é a fronteira do
/// sólido: a aresta que fecha o desenho **no eixo** varre uma linha, não um anel. Ela sai da conta
/// da distância (e só dela) — ver [`sd_profile_inner`].
#[must_use]
pub fn sd_revolve(profile: &Profile) -> Tree {
    let r = crate::ops::safe_sqrt(Tree::x().square() + Tree::z().square());
    sd_profile_inner(profile, &r, &Tree::y(), true)
}

/// ⭐⭐⭐ **O PERFIL BAIXADO PARA UMA REGIÃO** (W56) — a mesma lei, com uma fração das arestas.
///
/// # Por que isto e não uma folha nativa
///
/// A [`ProfileIndex`] responde em `40 ns` o que a fita responde em `155` — mas ela é **dados**, e a
/// álgebra da `fidget` é fechada. Pôr o perfil no caminho das folhas **amostradas** (o da escultura)
/// custaria duas coisas que o produto tem hoje, e uma leitura do [`crate::hybrid`] disse quais:
///
/// - ⛔ **os modificadores**: uma folha amostrada não passa pela pilha (`FieldError::ModsOnSampled`)
///   ⇒ uma peça desenhada perderia *Hollow*, *Offset*, *Array*, *Taper*;
/// - ⛔ **a quina viva**: o gradiente exacto só existe com `sampled.is_empty()` ⇒ a normal cairia
///   para diferença central, que é o que a razão de ser deste módulo não admite.
///
/// ⭐ **A saída é especializar a ÁRVORE, e não sair dela.** O que se mantém é tudo: fusão numa fita
/// só, JIT, gradiente exacto, modificadores, poses e booleanas. O que muda é **quantas arestas** a
/// expressão contém.
///
/// # As duas metades, e por que os conjuntos são DIFERENTES
///
/// | metade | de que arestas precisa | porquê |
/// |---|---|---|
/// | **distância** | as que podem ser a mais próxima de **algum** ponto da caixa | `min`: uma aresta longe pode ganhar |
/// | **sinal** | só as que **atravessam** a caixa | o enrolamento é invariante de caminho |
///
/// ⭐⭐ **O enrolamento vira uma CONSTANTE mais um punhado de termos.** `w(p) = w(c) + os
/// atravessamentos do caminho `c → p``, e `c` é o canto da caixa: `w(c)` calcula-se **na
/// construção** e entra na árvore como número. O caminho `c → p` não sai da caixa ⇒ só uma aresta
/// que a atravessa o pode cruzar — e são tipicamente uma ou duas.
///
/// ⚠️ **A árvore devolvida só vale DENTRO de `[lo, hi]`.** Fora dela a distância pode sair maior que
/// a verdadeira (arestas cortadas) e o sinal pode sair errado (enrolamento com a base errada). Quem
/// chama é quem sabe onde a vai avaliar — e o gate `the_specialised_tree_agrees_inside_its_region`
/// mede exactamente essa fronteira.
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn sd_profile_in_region(
    profile: &Profile,
    index: &crate::profile_index::ProfileIndex,
    u: &Tree,
    v: &Tree,
    lo: [f32; 2],
    hi: [f32; 2],
    axis_seam: bool,
    // ⭐⭐ **A região REAL, quando ela é um polígono** (W59) — `None` = usa a caixa.
    //
    // ⚠️ **Só a DISTÂNCIA a consome.** O conjunto do SINAL (`crossing_edges`) e a ÂNCORA continuam
    // a sair da caixa, e é de propósito: o enrolamento é um invariante de CAMINHO, e o caminho
    // âncora→ponto anda dentro da CAIXA. *A metade que compra está na distância; a outra é risco
    // sem prémio.*
    hull: Option<&[[f32; 2]]>,
) -> Tree {
    let non_zero = profile.fill() == FillRule::NonZero;
    // A mesma tolerância do [`sd_profile_inner`]: uma aresta a menos que isso do eixo **é** o eixo.
    let on_axis = f64::from(profile.tolerance());
    let near = hull.map_or_else(
        || index.distance_edges(lo, hi),
        |h| index.distance_edges_hull(h),
    );
    let mut dist2: Option<Tree> = None;
    for i in &near {
        let (a, b) = index.edge(*i);
        let (ax, ay) = (f64::from(a[0]), f64::from(a[1]));
        // ⚠️ **A costura do eixo sai da DISTÂNCIA e fica no enrolamento** — a mesma lei (e a mesma
        // razão medida) do [`sd_profile_inner`]: uma aresta sobre o eixo varre uma linha, não uma
        // parede, e deixá-la na conta põe um nível zero DENTRO do sólido.
        if axis_seam && ax.abs() <= on_axis && f64::from(b[0]).abs() <= on_axis {
            continue;
        }
        let (ex, ey) = (f64::from(b[0]) - ax, f64::from(b[1]) - ay);
        let inv_ee = 1.0 / (ex * ex + ey * ey);
        let wx = u.clone() - Tree::constant(ax);
        let wy = v.clone() - Tree::constant(ay);
        let h = ((wx.clone() * Tree::constant(ex) + wy.clone() * Tree::constant(ey))
            * Tree::constant(inv_ee))
        .max(0.0)
        .min(1.0);
        let qx = wx - h.clone() * Tree::constant(ex);
        let qy = wy - h * Tree::constant(ey);
        let seg2 = qx.square() + qy.square();
        dist2 = Some(match dist2 {
            None => seg2,
            Some(acc) => acc.min(seg2),
        });
    }
    // ⚠️ **Um perfil cujo corte não deixou aresta nenhuma é impossível** — a regra do corte guarda
    // sempre pelo menos a aresta que realiza o `dmax`. Recair na conta completa é o degenerado
    // seguro, e não um caso que se espera.
    let Some(dist2) = dist2 else {
        return sd_profile_inner(profile, u, v, axis_seam);
    };

    // ⭐⭐ **A ÂNCORA do enrolamento, e ela tem de estar LONGE de toda aresta.**
    //
    // ⛔ **Defeito medido (W56, gate de imagem):** com o canto da região a assentar **em cima** de
    // uma aresta, o enrolamento ali é ambíguo — a regra do raio `+x` decide por um lado, o caminho
    // decide por outro — e a região inteira sai com o **sinal invertido**. O sintoma na tela foi um
    // pixel a acertar onde não há peça: um `d` negativo é um acerto imediato para a esfera-marcha.
    //
    // ⚠️ **Ela tem de ficar DENTRO da região**, senão o caminho âncora→ponto sai dela e passa a poder
    // ser cruzado por arestas que não estão na conta. A região é convexa, logo o segmento entre dois
    // pontos dela nunca a deixa.
    let crossing = index.crossing_edges(lo, hi);
    let Some(anchor) = anchor_in(index, lo, hi, &crossing) else {
        // Uma região tão povoada que nenhum candidato fica livre: a conta completa, que não tem
        // âncora nenhuma. Correcta, só não mais rápida.
        return sd_profile_inner(profile, u, v, axis_seam);
    };
    let base = index.winding_at(anchor);
    let mut w: Tree = Tree::constant(f64::from(base));
    for i in crossing {
        w += crossing_term(index, i, u, v, anchor);
    }
    let inside = if non_zero {
        w.abs().min(1.0)
    } else {
        w.modulo(2.0)
    };
    let sign = Tree::constant(1.0) - Tree::constant(2.0) * inside;
    crate::ops::safe_sqrt(dist2) * sign
}

/// ⭐ **Um ponto da região que não assenta em aresta nenhuma** — a âncora do enrolamento.
///
/// ⚠️ **A barra é uma fração do TAMANHO DA REGIÃO, e não um epsilon absoluto.** Uma peça desenhada em
/// milímetros e a mesma em metros têm de escolher a mesma âncora; um número fixo aqui faria a
/// robustez depender da unidade do documento — o mesmo erro que o `TOLERANCE_RATIO` existe para não
/// cometer.
fn anchor_in(
    index: &crate::profile_index::ProfileIndex,
    lo: [f32; 2],
    hi: [f32; 2],
    crossing: &[u32],
) -> Option<[f32; 2]> {
    let span = (hi[0] - lo[0]).max(hi[1] - lo[1]).max(f32::MIN_POSITIVE);
    let bar = (span * 1.0e-3).powi(2);
    // Os cantos primeiro (é o que dá as constantes mais simples), depois o centro e as medianas.
    let mid = [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5];
    [
        lo,
        [hi[0], lo[1]],
        [lo[0], hi[1]],
        hi,
        mid,
        [mid[0], lo[1]],
        [mid[0], hi[1]],
        [lo[0], mid[1]],
        [hi[0], mid[1]],
    ]
    .into_iter()
    .find(|p| index.min_dist2_to(crossing, *p) > bar)
}

/// **Quantas vezes (com sinal) a aresta atravessa o caminho `c → p`**, como árvore.
///
/// ⚠️ **Sem `if`, como o resto do módulo.** Dois segmentos cruzam-se sse cada um separa os extremos
/// do outro — dois produtos de orientação negativos. `compare(−d, 0)` é `+1` exactamente quando `d`
/// é negativo, e o `max(·, 0)` transforma os `−1`/`0` em zero: o produto dos dois é a **indicadora**
/// do cruzamento.
///
/// ⭐ Duas das quatro orientações são **constantes** (`c` é o canto da região, e a aresta é
/// conhecida): `d1` sai da conta na construção, e o que resta é linear no ponto.
fn crossing_term(
    index: &crate::profile_index::ProfileIndex,
    edge: u32,
    u: &Tree,
    v: &Tree,
    c: [f32; 2],
) -> Tree {
    let (a, b) = index.edge(edge);
    let (ax, ay) = (f64::from(a[0]), f64::from(a[1]));
    let (bx, by) = (f64::from(b[0]), f64::from(b[1]));
    let (ex, ey) = (bx - ax, by - ay);
    let (cx, cy) = (f64::from(c[0]), f64::from(c[1]));

    // d1 = orient(a, b, c) — CONSTANTE.
    let d1 = ex * (cy - ay) - ey * (cx - ax);
    // d2 = orient(a, b, p) — linear no ponto.
    let d2 = (v.clone() - Tree::constant(ay)) * Tree::constant(ex)
        - (u.clone() - Tree::constant(ax)) * Tree::constant(ey);
    // d3 = orient(c, p, a) e d4 = orient(c, p, b) — bilineares no ponto.
    let px = u.clone() - Tree::constant(cx);
    let py = v.clone() - Tree::constant(cy);
    let d3 = px.clone() * Tree::constant(ay - cy) - py.clone() * Tree::constant(ax - cx);
    let d4 = px.clone() * Tree::constant(by - cy) - py.clone() * Tree::constant(bx - cx);

    // ⭐⭐ **A regra é SEMIABERTA** — ver [`crate::profile_index`]: um caminho que passa por um
    // **vértice** tem de contar **uma** vez, e o teste simétrico conta zero. `lt0(t)` vale 1 sse
    // `t < 0`, e o `|a − b|` de dois valores 0/1 é o «ou exclusivo»: as duas pontas de um lado
    // diferente.
    let lt0 = |t: Tree| Tree::constant(0.0).compare(t).max(0.0);
    let hit_ab = (Tree::constant(f64::from(d1 < 0.0)) - lt0(d2)).abs();
    let hit_cp = (lt0(d3) - lt0(d4)).abs();
    // O lado por que a aresta atravessa o caminho: `sign(caminho × aresta)`, invertido para casar
    // com a convenção do raio `+x` do [`sd_profile`].
    let side = (px * Tree::constant(ey) - py * Tree::constant(ex)).compare(0.0);
    Tree::constant(0.0) - side * hit_ab * hit_cp
}

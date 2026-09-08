//! ⭐⭐⭐ **A BEZIER QUADRÁTICA** (W136) — um traço curvo com espessura, sem desenhar um segmento.
//!
//! # ⭐⭐ A cúbica tem DOIS ramos, e a árvore não tem `if` — mas tem `compare`
//!
//! A distância a uma parábola resolve-se numa cúbica deprimida, e o sinal do **discriminante**
//! escolhe a fórmula: `h ≥ 0` dá **uma** raiz (Cardano, duas raízes cúbicas) e `h < 0` dá **três**
//! (Viète, um `acos`). Elas não se sobrepõem — no ramo de cada uma, a outra produz `NaN`.
//!
//! ⭐ **A saída é o [`Tree::compare`], e ela é CONTÍNUA aqui por um motivo geométrico:** em `h = 0`
//! a cúbica tem **raiz dupla**, e as duas fórmulas dão o **mesmo** ponto. Uma mistura meio-a-meio na
//! fronteira devolve esse valor, e não um degrau. *Um selector duro entre dois ramos que coincidem
//! na fronteira não é uma costura.*
//!
//! ⚠️ **As duas correm SEMPRE** (não há salto numa árvore), então cada uma tem de ser inofensiva no
//! domínio da outra: `safe_sqrt` no `√h` e no `√−P`, e o argumento do `acos` preso a `[−1, 1]`.
//! *Sem isso o `NaN` de um ramo envenena a soma com o outro.*

use fidget::context::Tree;

use crate::ops_norm::safe_sqrt;

/// O menor `|b|²` que a fórmula aceita — abaixo dele os três pontos são colineares e a cúbica
/// degenera. ⚠️ **Não é uma cerca do produto:** o documento tem a dele; isto só impede uma divisão
/// por zero se alguém construir a árvore à mão.
const B_FLOOR: f64 = 1.0e-9;

/// ⚠️ **A fracção do furo abaixo da qual o campo da onda é PLANO** — e o piso a que o divisor dela
/// se toma. As duas metades da mesma lei; ver [`sd_circle_wave`].
///
/// ⚠️ **MEDIDO** — ver o [doc 06 §137](../../../docs/3DModeling/06_resultados_cena_e_gizmo.md).
const WAVE_FLOOR: f64 = 0.5;

/// O quadrado da distância ao segmento `a`–`c` — **exacta**, e é o que a Bezier degenerada é.
fn segment_dist2(px: &Tree, py: &Tree, a: [f64; 2], c: [f64; 2]) -> Tree {
    let (ex, ey) = (c[0] - a[0], c[1] - a[1]);
    let ee = ex.mul_add(ex, ey * ey).max(f64::MIN_POSITIVE);
    let (wx, wy) = (
        px.clone() - Tree::constant(a[0]),
        py.clone() - Tree::constant(a[1]),
    );
    let t = ((wx.clone() * Tree::constant(ex) + wy.clone() * Tree::constant(ey))
        * Tree::constant(1.0 / ee))
    .max(Tree::constant(0.0))
    .min(Tree::constant(1.0));
    (wx - t.clone() * Tree::constant(ex)).square() + (wy - t * Tree::constant(ey)).square()
}

/// A raiz cúbica com sinal, `sign(x)·|x|^⅓` — não há `cbrt` na árvore.
///
/// ⚠️ O `TINY` está lá porque `ln(0)` é `−∞`: com ele a raiz de zero é um zero honesto em vez de um
/// `NaN` que atravessa a soma.
fn cbrt(x: &Tree) -> Tree {
    const TINY: f64 = 1.0e-30;
    let mag = (x.abs() + Tree::constant(TINY)).ln() * Tree::constant(1.0 / 3.0);
    x.compare(Tree::constant(0.0)) * mag.exp()
}

/// ⭐ **O quadrado da distância do ponto `(px, py)` à Bezier quadrática `A B C`.**
///
/// Porte da forma fechada publicada (Inigo Quílez, `sdBezier`), com os dois ramos calculados sempre
/// e escolhidos pelo [`Tree::compare`] — ver o cabeçalho do módulo.
#[must_use]
pub fn bezier_dist2(px: &Tree, py: &Tree, a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> Tree {
    // ⚠️ Tudo o que é dos PONTOS é constante — só `d` depende de onde se avalia.
    let va = [b[0] - a[0], b[1] - a[1]];
    let vb = [a[0] - 2.0 * b[0] + c[0], a[1] - 2.0 * b[1] + c[1]];
    // ⭐⭐⭐ **A CURVA DEGENERADA É UM SEGMENTO, e o `if` é do HOST.**
    //
    // Com `b` no meio de `a` e `c` o `vb` é zero, a cúbica deixa de o ser e o `kk = 1/|vb|²`
    // explode. ⛔ **Recusar seria mau produto** — um traço recto é uma autoria legítima —, e um
    // ramo na ÁRVORE seria pago em toda avaliação. ⭐ Os três pontos são **constantes de
    // construção**, então a escolha acontece aqui, em Rust, e custa zero nós.
    if vb[0].mul_add(vb[0], vb[1] * vb[1]) < B_FLOOR {
        return segment_dist2(px, py, a, c);
    }
    let vc = [2.0 * va[0], 2.0 * va[1]];
    let kk = 1.0 / (vb[0] * vb[0] + vb[1] * vb[1]).max(B_FLOOR);
    let kx = kk * (va[0] * vb[0] + va[1] * vb[1]);
    let dx = Tree::constant(a[0]) - px.clone();
    let dy = Tree::constant(a[1]) - py.clone();
    let aa = va[0] * va[0] + va[1] * va[1];
    let ky = (dx.clone() * Tree::constant(vb[0])
        + dy.clone() * Tree::constant(vb[1])
        + Tree::constant(2.0 * aa))
        * Tree::constant(kk / 3.0);
    let kz = (dx.clone() * Tree::constant(va[0]) + dy.clone() * Tree::constant(va[1]))
        * Tree::constant(kk);

    let pp = ky.clone() - Tree::constant(kx * kx);
    let qq = ky * Tree::constant(-3.0 * kx) + kz + Tree::constant(2.0 * kx * kx * kx);
    let hh = qq.square() + pp.clone() * pp.clone() * pp.clone() * Tree::constant(4.0);

    // O ponto da curva em `t`, e o quadrado da distância a ele.
    let em = |t: &Tree| {
        let ex =
            dx.clone() + (Tree::constant(vc[0]) + Tree::constant(vb[0]) * t.clone()) * t.clone();
        let ey =
            dy.clone() + (Tree::constant(vc[1]) + Tree::constant(vb[1]) * t.clone()) * t.clone();
        ex.square() + ey.square()
    };
    let preso = |t: Tree| t.max(Tree::constant(0.0)).min(Tree::constant(1.0));

    // ── ramo de UMA raiz (`h ≥ 0`) ──
    let raiz = safe_sqrt(hh.clone());
    let x1 = (raiz.clone() - qq.clone()) * Tree::constant(0.5);
    let x2 = (-raiz - qq.clone()) * Tree::constant(0.5);
    let t_um = preso(cbrt(&x1) + cbrt(&x2) - Tree::constant(kx));
    let um = em(&t_um);

    // ── ramo de TRÊS raízes (`h < 0`) ──
    // ⚠️ Aqui `P < 0`, logo `√−P` é real; fora do ramo o `safe_sqrt` devolve o piso.
    let z = safe_sqrt(-pp.clone());
    let arg = (qq / (pp * z.clone() * Tree::constant(2.0)))
        .max(Tree::constant(-1.0))
        .min(Tree::constant(1.0));
    let v = arg.acos() * Tree::constant(1.0 / 3.0);
    let (m, n) = (v.clone().cos(), v.sin() * Tree::constant(3.0_f64.sqrt()));
    let t_a = preso((m.clone() + m.clone()) * z.clone() - Tree::constant(kx));
    let t_b = preso((-n - m) * z - Tree::constant(kx));
    // ⭐ A terceira raiz nunca é a mais próxima — é o que a fonte publicada afirma, e o gate
    // `the_third_root_is_never_the_closest` mede-o contra o oráculo denso.
    let tres = em(&t_a).min(em(&t_b));

    // ⭐⭐ O SELECTOR: `compare` devolve `−1/0/1`, logo `s` é `0`, `½` ou `1`. Em `h = 0` a cúbica
    // tem raiz DUPLA e os dois ramos coincidem — o `½` ali devolve o valor deles, não um degrau.
    let s = (hh.compare(Tree::constant(0.0)) + Tree::constant(1.0)) * Tree::constant(0.5);
    um * s.clone() + tres * (Tree::constant(1.0) - s)
}

/// ⭐⭐⭐ **A ONDA EM ANEL** (W136) — o anel de raio `radius` cuja distância ao eixo ondula
/// `amplitude` em `lobes` lóbulos, com meia-espessura `thickness`.
///
/// # ⭐ O minorante é o do `Document`, e a divisão vem DEPOIS da subtracção
///
/// `g = |ρ − R(φ)| − thickness` anula-se exactamente na fronteira da faixa, e `‖∇g‖ ≤ lip` com
/// `lip = √(1 + (a·n/ρ_min)²)` — a mesma conta que a onda do `Document` (W123) estreou, com `ρ` no
/// lugar de `x`. ⇒ `g/lip` é 1-Lipschitz e anula-se na fronteira, logo é um minorante da distância.
///
/// ⚠️⚠️ **A divisão é do CAMPO INTEIRO, e é a lei que a W134 pagou:** `(|ρ−R|/lip) − thickness`
/// teria o zero em `|ρ−R| = thickness·lip` — a faixa sairia **`lip` vezes mais gorda** do que o
/// painel diz. *Um minorante frouxo não deixa a peça conservadora: ele INFLA-A.*
///
/// ⚠️ **`lobes` é `u32`, e é isso que faz a costura do `atan2` não existir:** em `φ → φ − 2π` o
/// `sin(n·φ)` só é invariante com `n` inteiro. A mesma lei do nó (W134) e da rosca (W135), pela
/// terceira via.
#[must_use]
pub fn sd_circle_wave(
    radius: f64,
    amplitude: f64,
    lobes: u32,
    thickness: f64,
    half_height: f64,
    round: f64,
    chamfer: f64,
) -> Tree {
    let e = crate::ops_joint::Edge::square(round, chamfer);
    let n = f64::from(lobes.max(1));
    let rho = safe_sqrt(Tree::x().square() + Tree::y().square());
    let phi = Tree::y().atan2(Tree::x());
    let phi_n = phi * Tree::constant(n);
    let r_de = phi_n.clone().sin() * Tree::constant(amplitude) + Tree::constant(radius);
    // ⚠️ **O `lip` é tomado no MENOR raio onde há matéria** — a mesma lei do divisor da espiral: um
    // valor local sobrestimaria a distância quando o ponto mais próximo estivesse mais para dentro.
    let dentro = (radius - amplitude - thickness).max(radius * 0.02);
    // ⭐⭐⭐ **O ARREDONDAMENTO DO ARO ACONTECE NO PLANO `(radial, z)`, E A FOLGA VEM DEPOIS.**
    //
    // ⛔⛔ **Duas construções foram medidas e recusadas antes desta:**
    //
    // 1. **O divisor CONSTANTE aplicado à banda ANTES da junta** — o campo é um minorante honesto e
    //    o **aro sai ELÍPTICO**: a junta supõe duas distâncias verdadeiras e recebe uma encolhida
    //    `lip` vezes. Medido (`probe_wave_fillet_is_round_or_oval`, `lip = 4,033`): com
    //    `round = 0,0182` o recuo em `z` é `0,0179` ✓ e em `ρ` é **`0,0711`** (`3,96×`), e a meio do
    //    curso o filete come a **parede toda**. *Um controlo que recua quatro vezes o que diz mente.*
    // 2. **O divisor LOCAL** (`√(1 + (R'/ρ)²)`, que devolveria a distância honesta) — o filete fica
    //    redondo e o campo **morre**: `‖∇f‖` até **`2 156`** na região permitida, o que pediria uma
    //    folga de `0,0005`. ⚠️ Isso confirma, com número, a recusa que a ESPIRAL (W123) já tinha
    //    escrito — e refuta a minha ideia de que «faltava a segunda metade»: dividir por uma função
    //    que varia depressa põe o `∇L/L²` no gradiente, e ele é que explode.
    //
    // ⭐ **A saída é fazer o aro no plano em que as DUAS coordenadas são honestas.** O `g` radial é
    // a distância EXACTA ao longo do raio, e o `z` é exacto: a junta entre eles devolve um arco
    // genuinamente **circular** na secção `(ρ, z)`, que é o que um filete desta forma É. Só depois o
    // campo inteiro se divide pelo `lip` — e o zero de um campo dividido por uma constante positiva
    // não se mexe (a lei da W134, no nível certo).
    // ⭐⭐⭐ **O PISO DO DIVISOR E O TECTO DO CAMPO SÃO UMA LEI SÓ, e nenhuma metade chega sozinha.**
    //
    // `‖∇g‖ = √(1 + (R'/ρ)²)` cresce quando `ρ` encolhe, e o campo é avaliado na CAIXA. Com o
    // divisor tomado em `dentro` fica uma janela `ρ < dentro` em que ele é excedido — e o `max` com
    // o furo **não a fecha**: mesmo com um tecto, junto de `ρ = dentro` a folga do furo vai a zero
    // enquanto o campo da crista ainda é positivo. ⛔ Medido: `‖∇f‖` até **`7,30`** com o tecto em
    // `dentro`, e a álgebra diz que **nenhum tecto** o cura com este piso.
    //
    // ⇒ as duas metades: o divisor toma-se em [`WAVE_FLOOR`]`·dentro`, e o campo é capado em
    // `(1 − `[`WAVE_FLOOR`]`)·dentro`, que é exactamente o valor abaixo do qual **todo** `φ` já está
    // capado. Assim `ρ < piso` é plano (gradiente zero) e `ρ ≥ piso` está dentro do divisor.
    let piso = dentro * WAVE_FLOOR;
    let lip = (amplitude * n / piso)
        .mul_add(amplitude * n / piso, 1.0)
        .sqrt();
    let tecto = dentro * (1.0 - WAVE_FLOOR);
    // ⚠️ **O TECTO em `dentro` é o que mata o termo angular junto ao eixo.** `g_radial` carrega
    // `R(φ)`, e perto do eixo um passo minúsculo em `x` roda `φ` muito — a derivada angular
    // explode. ⭐ Capar por cima **reduz** o campo, logo continua minorante, e acima de `dentro` o
    // ponto já está mais longe da peça do que o furo inteiro: ali a precisão não serve a ninguém.
    // Medido: sem o tecto o gradiente lia **`7,30`** a `ρ = 0,039` (com a pele em `1,0000`).
    let g_radial =
        ((rho.clone() - r_de).abs() - Tree::constant(thickness)).min(Tree::constant(tecto));
    let peca = crate::ops::slab_and_walls(&g_radial, half_height, e) * Tree::constant(1.0 / lip);
    // ⭐⭐ **O FURO fecha o campo, e sem ele o `lip` é uma promessa que só vale onde há matéria.**
    //
    // ⛔ `‖∇(ρ − R(φ))‖ = √(1 + (R'/ρ)²)` cresce quando `ρ` **encolhe**, e o campo é avaliado na
    // CAIXA INTEIRA — junto ao eixo o termo angular explode e o divisor, tomado no menor raio com
    // matéria, deixa de o segurar. Medido: `‖∇f‖ = 2,4638` no representante e **`13,67`** no tecto
    // de lóbulos, contra o `1` que a marcha exige.
    //
    // ⭐ A peça vive toda em `ρ ≥ dentro` e `|z| ≤ h`, logo o complemento disso é um minorante
    // **exacto e 1-Lipschitz**. Um `max` de dois minorantes ainda é um minorante — e este ganha
    // exactamente na região em que o outro mente.
    //
    // ⚠️⚠️ **É a SEGUNDA vez em duas waves que eu limito um divisor sobre «onde há matéria»** — a
    // rosca (W135) teve a cunha infinita a ganhar o `min` dentro da peça pela mesma razão. *O
    // domínio de um divisor é a CAIXA, não a peça.*
    let fora = (Tree::constant(dentro) - rho).max(Tree::z().abs() - Tree::constant(half_height));
    peca.max(fora)
}

/// ⭐⭐⭐ **A BEZIER COM ESPESSURA, puxada em Z** — a peça que o catálogo entrega.
///
/// ⚠️ **A faixa é `dist − thickness`, e isso é uma distância EXACTA**: o conjunto `{d ≤ t}` é um
/// sub-nível de uma distância, logo ele nunca se auto-intersecta por mais grossa que a faixa seja.
/// *É por isso que esta forma não tem tecto de espessura, e a onda em anel tem.*
#[must_use]
pub fn sd_bezier(
    a: [f64; 2],
    b: [f64; 2],
    c: [f64; 2],
    thickness: f64,
    half_height: f64,
    round: f64,
    chamfer: f64,
) -> Tree {
    let e = crate::ops_joint::Edge::square(round, chamfer);
    let d2 = bezier_dist2(&Tree::x(), &Tree::y(), a, b, c);
    let faixa = safe_sqrt(d2) - Tree::constant(thickness);
    crate::ops::slab_and_walls(&faixa, half_height, e)
}

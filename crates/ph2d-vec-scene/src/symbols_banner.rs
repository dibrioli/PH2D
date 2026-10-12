//! **Banner / fita** — módulo irmão de [`super`] (`symbols.rs`), separado pelo tecto de LOC
//! (HR-18). Guarda a família do banner: as constantes do `ribbon` do ECMA-376, o vinco da
//! dobra ([`banner_crease`]) e o construtor [`banner`], reexportado pelo pai.

use crate::space::{Unit, add_sub, closed, fit, straighten};
use crate::{VecPath, VecVertex};

// ─────────────────────────────────────────────────────────────────────────────
// BANNER
// ─────────────────────────────────────────────────────────────────────────────

/// Largura do painel central (`adj2` do `ribbon` do ECMA-376).
const BANNER_PANEL: f64 = 0.50;
/// Altura da faixa dobrada (`adj1`).
pub(super) const BANNER_FOLD: f64 = 1.0 / 6.0;
/// Raio HORIZONTAL dos cantos da dobra (`1/32` da espec). O vertical é `fold/4` — os cantos
/// são ELÍPTICOS, não circulares, e a razão entre os dois (0,75) é o que dá o formato da
/// dobra. Numa caixa larga, um raio em fração da LARGURA esticaria a dobra num gancho; por
/// isso o raio viaja com o `aspect` (a dobra é um detalhe de tamanho fixo, não uma proporção
/// da largura) — em caixa quadrada o valor volta a ser exatamente o da espec.
pub(super) const BANNER_RX: f64 = 1.0 / 32.0;
/// Teto de VALIDADE do entalhe: em `notch = x2` o "V" da cauda alcançaria a borda do painel
/// e cortaria a fita ao meio.
const BANNER_NOTCH_MAX: f64 = 0.90;

/// O **vinco** de uma dobra: a aresta que dobra PARA TRÁS. Sai da silhueta no ombro
/// `(x5, ry)`, curva, volta para a esquerda e desce até o topo do painel — e fecha a face
/// dobrada contra o vinco vertical. É a face `darkenLess` da espec, contornada.
///
/// Espelhar em `u` é: `x → 1 − x`, ângulo `a → 180° − a`, sweep `s → −s`.
fn banner_crease(u: &Unit, rx: f64, ry: f64, x3: f64, x4: f64, mirrored: bool) -> Vec<VecVertex> {
    let m = |x: f64| if mirrored { 1.0 - x } else { x };
    let deg = |a: f64| if mirrored { 180.0 - a } else { a };
    let sw = |s: f64| if mirrored { -s } else { s };

    // O ombro: a metade do arco da cauda que fica DENTRO da forma (a de fora é silhueta).
    let mut cr = u.arc((m(x4), ry), rx, ry, deg(0.0), sw(90.0));
    let seam = cr.len() - 1;
    // O degrau de volta e a descida até o topo do painel.
    cr.extend(u.arc((m(x3), 3.0 * ry), rx, ry, deg(-90.0), sw(-180.0)));
    straighten(&mut cr, seam);
    let seam = cr.len() - 1;
    // E a base da face, de volta ao vinco vertical da silhueta.
    cr.push(u.corner((m(x4 + rx), 4.0 * ry)));
    straighten(&mut cr, seam);
    cr
}

/// **Banner / fita** — a dobra fica em CIMA, o painel pendura embaixo (o `ribbon` do
/// ECMA-376).
///
/// As caudas ocupam `v ∈ [0, 1 − fold]` e o painel `v ∈ [fold, 1]`: a cauda é MAIS ALTA que o
/// painel, e o que os liga é a **dobra** — a fita passa para trás e volta. A silhueta da
/// dobra é o ombro arredondado da cauda descendo até o topo do painel; a aresta dobrada em
/// si (o vinco) entra como contorno ABERTO, e é ela que faz a fita parecer dobrada.
///
/// **Cuidado com a espec aqui:** o `moveTo/lnTo/arcTo` do `ribbon` desenha só as faces da
/// FRENTE — sozinho ele deixa um BURACO no meio da dobra, que o ECMA tapa com uma segunda
/// face (`fill="darkenLess"`, a parte de trás do papel). Uma forma nossa cozinha para UM
/// `VecPath`, então o contorno externo aqui é a silhueta da UNIÃO (frente + face dobrada) —
/// sem buraco — e a face dobrada aparece pelo seu vinco. Copiar o path da espec ao pé da
/// letra produz a fita furada.
///
/// `notch` = profundidade do "V" (rabo-de-peixe) de cada cauda.
#[must_use]
pub fn banner(a: [f64; 2], b: [f64; 2], notch: f64) -> VecPath {
    let u = Unit::of(a, b);
    // Raio da dobra: fixo em relação à ALTURA (ver [`BANNER_RX`]), com teto para não engolir
    // a cauda numa caixa muito alta.
    let rx = (BANNER_RX * u.aspect()).clamp(0.006, 0.055);
    let ry = 0.25 * BANNER_FOLD; // quatro deles empilham a dobra inteira (`4·ry = fold`)

    let x2 = 0.5 - 0.5 * BANNER_PANEL; // borda do painel
    let x3 = x2 + rx; // onde o vinco encosta no topo do painel
    let x4 = x3 + 2.0 * rx; // onde o ombro da dobra encosta na cauda
    let x5 = x4 + rx; // o vinco VERTICAL (borda da face dobrada)
    let (x6, x7, x8, x9) = (1.0 - x5, 1.0 - x4, 1.0 - x3, 1.0 - x2);
    let y_fold = 4.0 * ry; // = BANNER_FOLD: o topo do painel
    let y_tail = 1.0 - BANNER_FOLD; // a base das caudas
    let y_pan = 1.0 - ry; // centro dos cantos de baixo do painel
    let n = notch.clamp(0.0, BANNER_NOTCH_MAX * x2);
    let apex = 0.5 * y_tail; // o "V" morde no meio da cauda

    let mut verts = vec![u.corner((0.0, 0.0))];
    // Cada índice em `seams` é o ÚLTIMO vértice antes de uma RETA — sem `straighten` a reta
    // herda os handles do arco (o usuário vê e arrasta fantasmas).
    let mut seams: Vec<usize> = vec![0];

    // Ombro da dobra esquerda: da aresta da cauda até o vinco vertical.
    let arc = u.arc((x4, ry), rx, ry, -90.0, 90.0);
    seams.push(verts.len() + arc.len() - 1);
    verts.extend(arc);
    verts.push(u.corner((x5, y_fold))); // desce o vinco
    verts.push(u.corner((x6, y_fold))); // topo do painel
    seams.push(verts.len() - 1);
    // Ombro da dobra direita (espelho).
    let arc = u.arc((x7, ry), rx, ry, 180.0, 90.0);
    seams.push(verts.len() + arc.len() - 1);
    verts.extend(arc);

    verts.push(u.corner((1.0, 0.0))); // cauda direita, aresta de cima
    verts.push(u.corner((1.0 - n, apex))); // o "V"
    verts.push(u.corner((1.0, y_tail))); // cauda direita, aresta de baixo
    verts.push(u.corner((x9, y_tail)));
    seams.push(verts.len() - 1);

    for (c, start) in [((x8, y_pan), 0.0), ((x3, y_pan), 90.0)] {
        let arc = u.arc(c, rx, ry, start, 90.0);
        seams.push(verts.len() + arc.len() - 1);
        verts.extend(arc);
    }

    verts.push(u.corner((x2, y_tail)));
    verts.push(u.corner((0.0, y_tail))); // cauda esquerda, aresta de baixo
    verts.push(u.corner((n, apex))); // o "V"
    for s in seams {
        straighten(&mut verts, s);
    }

    let mut p = closed(verts);
    for mirrored in [false, true] {
        add_sub(&mut p, banner_crease(&u, rx, ry, x3, x4, mirrored), false);
    }
    fit(&u, &mut p);
    p
}

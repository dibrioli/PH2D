//! **Suavização de quina** (o *corner smoothing* do Figma / o canto contínuo da Apple) —
//! módulo irmão de [`crate::corners`], que fica com a quina de arco puro.
//!
//! ## O problema que ela resolve
//!
//! Uma quina arredondada por um arco de círculo é `G1`: a tangente é contínua, mas a
//! **curvatura SALTA** de `0` (a aresta reta) para `1/r` (o arco) num ponto. O olho vê esse
//! salto — é o que faz um round-rect comum parecer "colado", e é por isso que o ícone do iOS
//! não é um round-rect. O *squircle* troca o salto por uma **rampa**.
//!
//! ## A construção (a de verdade, não a superelipse)
//!
//! A aproximação ingênua é a superelipse `|x|^n + |y|^n = 1`. Ela é bonita no papel e
//! **errada na prática**, por dois motivos:
//!
//! 1. ela não é uma cúbica — só se aproxima dela, e o erro cresce justamente onde a forma
//!    importa (o joelho do canto), então um `n` grande vira um canto visivelmente "quadrado
//!    demais" com as retas dos lados chegando fora de tangente;
//! 2. ela **redefine a forma inteira**, não a quina: não há mais um raio, não há lado reto
//!    verdadeiro, e um retângulo comprido vira um bojo. O usuário pediu um canto suave, não
//!    outra silhueta.
//!
//! A construção real (a que o Figma usa, e que reverse-engineers documentaram a partir dela)
//! mantém o lado RETO e substitui o arco de 90° por **arco mais curto + duas asas de Bézier**
//! que se estendem pela aresta:
//!
//! ```text
//!   reta ──── P0 ─── asa (cúbica) ─── A ══ arco ══ A' ─── asa ─── P0' ──── reta
//! ```
//!
//! - o arco encolhe: de `α` (o suplemento do ângulo interno) para `α·(1 − s)`;
//! - de cada ponta dele corta-se `β = α·s/2`, e o **centro de curvatura não se mexe** (é o que
//!   mantém o canto reconhecível enquanto o slider anda);
//! - a asa é uma cúbica cujos DOIS primeiros pontos de controle ficam **sobre a aresta**
//!   (`P0`, `P1`, `P2` colineares) ⇒ a curvatura na saída da reta é **exatamente 0** — a
//!   junção com o lado reto vira `G2` de graça;
//! - `P2` é a interseção da tangente-no-arco com a aresta (as duas tangentes de um círculo
//!   cujos pontos de contato distam `β` se cruzam a `r·tan(β/2)` de cada contato);
//! - a quina passa a comer `p = (1 + s)·t` de cada aresta (`t` = o recuo do arco puro).
//!   Em 90° isto é o `(1 + s)·r` do Figma; aqui vale para QUALQUER ângulo, que é o que dá o
//!   canto suave de polígono e estrela pelo mesmo motor.
//!
//! ## O `b` (a posição de `P1`): a CONSTRIÇÃO, não o palpite do Figma
//!
//! Sobra um grau de liberdade: onde fica `P1` entre `P0` e `P2` (isto é, como repartir o
//! orçamento `|P0 P2|` em `a + b`). Ele governa a curvatura com que a asa CHEGA no arco:
//!
//! ```text
//! κ_asa(1) = (2/3)·|(P3 − P2) x (P2 − P1)| / |P3 − P2|³ = (2/3)·d·b / L³ ,  L = r·tan(β/2)
//! ```
//!
//! O **Figma reparte `2:1`** (`a = 2b`) — um palpite, não a solução. Em 90° ele passa perto
//! (a curvatura da asa chega a ~1,47/r em `s = 0,6`, contra o `1/r` do arco: um overshoot de
//! 47%), mas **generalizado para outro ângulo ele INVERTE o efeito**: numa quina de 45° (a
//! ponta de uma estrela) o `b` fica grande demais e a curvatura da asa chega a ~2,1/r — o
//! salto na junção sobe de 0,98/r para 1,01/r, e o slider de "suavizar" passa a piorar o
//! canto. Medido, não postulado: era o teste `the_smoothing_generalises_to_any_corner_angle`
//! ficando vermelho.
//!
//! Aqui `b` sai da **constrição de curvatura** — a mesma que os reverse-engineers derivam e
//! que o Figma aproxima —, resolvendo `κ_asa(1) = 1/r`:
//!
//! ```text
//! b = (3/2)·L³ / (d·r) = (3/2)·r·tan²(β/2) / sin β        (d = L·sin β)
//! ```
//!
//! Com ele a quina inteira fica **G2 de verdade** — `0` na reta, rampa pela asa, `1/r` no arco,
//! sem overshoot — para QUALQUER ângulo. A silhueta em 90° é a do Figma (`P0`, `P2`, `P3` e o
//! arco são os mesmos; só `P1` desliza sobre a aresta, ~9% de `r`), o consumo de aresta é o
//! mesmo `(1 + s)·r`, e o slider "sente" igual. Trocamos um palpite que quebra fora de 90° por
//! a solução fechada que vale sempre — sem custo.
//!
//! `s = 0` não passa por aqui: [`crate::corners::round_closed_corners_smooth`] desvia para a
//! quina de arco puro, byte-a-byte a de sempre. **A identidade é sagrada.**

use crate::{VecVertex, VertexKind};

/// Abaixo disto (unidades de mundo) um raio / aresta / recuo é tratado como zero.
const EPS: f64 = 1e-9;

/// Abaixo disto (radianos) o arco remanescente é tratado como INEXISTENTE: as duas asas se
/// encontram num vértice só, em vez de num par de âncoras coincidentes (que viraria um
/// segmento de comprimento zero — veneno para booleana e para a edição à mão).
const ARC_EPS: f64 = 1e-9;

/// Um ponto 2D no mundo.
type P = [f64; 2];

fn sub(p: P, q: P) -> P {
    [p[0] - q[0], p[1] - q[1]]
}

fn add(p: P, q: P) -> P {
    [p[0] + q[0], p[1] + q[1]]
}

fn mul(p: P, k: f64) -> P {
    [p[0] * k, p[1] * k]
}

fn dot(p: P, q: P) -> f64 {
    p[0] * q[0] + p[1] * q[1]
}

fn cross(p: P, q: P) -> f64 {
    p[0] * q[1] - p[1] * q[0]
}

/// Versor + comprimento; `None` se o vetor é degenerado.
fn unit(v: P) -> Option<(P, f64)> {
    let len = v[0].hypot(v[1]);
    (len > EPS).then(|| ([v[0] / len, v[1] / len], len))
}

/// Gira `d` (um versor) por `ang` radianos.
fn rotate(d: P, ang: f64) -> P {
    let (s, c) = ang.sin_cos();
    [d[0] * c - d[1] * s, d[0] * s + d[1] * c]
}

/// A interseção das tangentes ao círculo `(c, r)` nos pontos de direções radiais `d1` e `d2`:
/// `Q = c + r·(d1 + d2)/(1 + d1·d2)`. Fórmula fechada, sem branch e sem caçar sinal — e ela
/// cai exatamente sobre a ARESTA quando `d1` é a direção do ponto de tangência original.
/// `None` só se as duas direções forem opostas (impossível aqui: `β < 90°`).
fn tangent_meet(c: P, r: f64, d1: P, d2: P) -> Option<P> {
    let k = 1.0 + dot(d1, d2);
    (k > EPS).then(|| add(c, mul(add(d1, d2), r / k)))
}

/// **A quina suavizada.** Substitui a quina `v` (entre os vizinhos `a` e `b`) por
/// `asa + arco + asa`, com raio `r` e suavização `s ∈ (0, 1]`.
///
/// `None` quando não há o que suavizar (raio ~0, aresta degenerada ou quina colinear) — o
/// chamador mantém o vértice cru, exatamente como faz no caminho de arco puro.
///
/// O consumo total de cada aresta (`p`) satura em **meia-aresta**, como no arco puro: é `p`
/// — não o recuo do arco — que satura, senão a asa vazaria para dentro da quina vizinha. O
/// raio efetivo é o que sobra depois disso (é ele que dita o arco, não o `r` pedido).
///
/// Trigonometria de geometria de editor (como o resto do catálogo: `kurbo`/`vello` já usam
/// trig internamente) — não é caminho de simulação determinística.
pub(crate) fn smooth_corner(a: P, v: P, b: P, r: f64, s: f64) -> Option<Vec<VecVertex>> {
    if r <= EPS || s <= 0.0 {
        return None;
    }
    let s = s.min(1.0);
    // Versores das duas arestas, SAINDO da quina.
    let (ua, len_a) = unit(sub(a, v))?;
    let (ub, len_b) = unit(sub(b, v))?;
    // Ângulo interno da quina (o que as duas arestas abrem entre si), em [0, π].
    let theta = dot(ua, ub).clamp(-1.0, 1.0).acos();
    let half = theta * 0.5;
    if half <= EPS || (std::f64::consts::PI - theta) <= EPS {
        return None; // colinear (nada a arredondar) ou aresta dobrada sobre si
    }
    let tan_half = half.tan();
    // O que a quina PEDE de cada aresta: o recuo do arco puro, esticado pelo fator (1 + s)
    // das asas. Satura em meia-aresta — assim duas quinas vizinhas nunca se invadem.
    let p = ((1.0 + s) * (r / tan_half))
        .min(len_a * 0.5)
        .min(len_b * 0.5);
    if p <= EPS {
        return None;
    }
    let t = p / (1.0 + s); // o recuo do ARCO depois do clamp
    let r_eff = t * tan_half; // e o raio que ele implica
    // O arco que a quina descreve (suplemento do ângulo interno) e o que se corta de CADA
    // ponta dele. `s = 1` zera o arco: as duas asas se encontram na bissetriz.
    let alpha = std::f64::consts::PI - theta;
    let beta = alpha * 0.5 * s;
    let arc = alpha - 2.0 * beta;
    // Suavização INFINITESIMAL (um `s` subnormal de um save corrompido) volta para o arco
    // puro: com `β = 0`, o `tan²(β/2)/sin β` do `b` seria `0/0` — NaN, e a forma sumiria da
    // tela. E é a resposta certa de qualquer jeito: uma suavização de 1e-9 rad é o arco.
    if beta <= EPS || r_eff <= EPS {
        return None;
    }

    // O centro do arco: na bissetriz, a `r/sin(θ/2)` da quina. Vale para quina convexa E
    // côncava — o círculo tangente às duas arestas mora na cunha entre elas, dos dois casos.
    let (bis, _) = unit(add(ua, ub))?;
    let sin_half = half.sin();
    if sin_half <= EPS {
        return None;
    }
    let c = add(v, mul(bis, r_eff / sin_half));
    // Direções radiais dos pontos de tangência do arco PURO (onde o arco tocaria as arestas).
    let d_ta = mul(sub(add(v, mul(ua, t)), c), 1.0 / r_eff);
    let d_tb = mul(sub(add(v, mul(ub, t)), c), 1.0 / r_eff);
    // O SENTIDO em que o arco corre, de `a` para `b`. Estável: `alpha > 0` por construção.
    let sigma = cross(d_ta, d_tb).signum();
    // As pontas do arco encurtado: cada tangência girada `β` rumo ao MEIO do arco.
    let d_aa = rotate(d_ta, sigma * beta);
    let d_ab = rotate(d_tb, -sigma * beta);
    let (aa, ab) = (add(c, mul(d_aa, r_eff)), add(c, mul(d_ab, r_eff)));
    // `P2` de cada asa: onde a tangente do arco encontra a aresta.
    let qa = tangent_meet(c, r_eff, d_ta, d_aa)?;
    let qb = tangent_meet(c, r_eff, d_tb, d_ab)?;
    // `b` = |P1 P2|, a distância que faz a curvatura da asa CHEGAR em 1/r no arco (a
    // constrição de curvatura; ver o módulo — o `a = 2b` do Figma inverte fora de 90°).
    let b_len = 1.5 * r_eff * (beta * 0.5).tan().powi(2) / beta.sin();
    // `P0` de cada asa (o fim do lado reto) e `P1` (a `a = |P0 P2| − b` dele, SOBRE a aresta —
    // é o que mantém `P0`, `P1`, `P2` colineares e a curvatura ZERO na saída da reta).
    let wing = |u: P, q: P| {
        let p0 = add(v, mul(u, p));
        let seg = sub(q, p0);
        let len = seg[0].hypot(seg[1]);
        if len <= EPS {
            return (p0, p0);
        }
        let a_len = (len - b_len).max(0.0);
        (p0, add(p0, mul(seg, a_len / len)))
    };
    let (p0a, p1a) = wing(ua, qa);
    let (p0b, p1b) = wing(ub, qb);
    // Comprimento de handle exato de uma cúbica que segue o arco remanescente:
    // (4/3)·tan(arco/4)·r — a generalização do KAPPA (que é esse valor em 90°).
    let h = (4.0 / 3.0) * (arc * 0.25).tan() * r_eff;
    // Tangente unitária no ponto de direção radial `d`, no sentido do percurso.
    let tang = |d: P| [-d[1] * sigma, d[0] * sigma];

    let mut out = Vec::with_capacity(4);
    // A âncora onde o lado reto acaba: handle nulo do lado da reta, `P1` do lado da asa.
    out.push(VecVertex {
        anchor: p0a,
        in_handle: p0a,
        out_handle: p1a,
        kind: VertexKind::Corner,
    });
    if arc <= ARC_EPS {
        // `s = 1`: o arco sumiu — as duas asas se encontram na bissetriz, num vértice SÓ
        // (âncoras coincidentes seriam um segmento de comprimento zero).
        let m = mul(add(aa, ab), 0.5);
        out.push(VecVertex {
            anchor: m,
            in_handle: qa,
            out_handle: qb,
            kind: VertexKind::Corner,
        });
    } else {
        out.push(VecVertex {
            anchor: aa,
            in_handle: qa,
            out_handle: add(aa, mul(tang(d_aa), h)),
            kind: VertexKind::Corner,
        });
        out.push(VecVertex {
            anchor: ab,
            in_handle: sub(ab, mul(tang(d_ab), h)),
            out_handle: qb,
            kind: VertexKind::Corner,
        });
    }
    out.push(VecVertex {
        anchor: p0b,
        in_handle: p1b,
        out_handle: p0b,
        kind: VertexKind::Corner,
    });
    Some(out)
}

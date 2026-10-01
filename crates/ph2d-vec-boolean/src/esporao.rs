//! ⭐⭐ **O ESPORÃO QUE A UNIÃO DEIXA** (F46, o braço dobrado de VOLTA).
//!
//! Com uma junta a `~174°` a pele dos dois membros quase coincide, e a união emite um pedaço de
//! contorno que VOLTA pelo mesmo caminho: medido a `(0°, 174°)`, um segmento recto de `0,0605` que
//! refaz o fim do segmento anterior e um nó a virar `180°` na ponta. A área dele é ZERO — como forma
//! não existe —, mas o traço desenha a meia-volta. ⛔ Nem a bola nem o desfazer dos ganchos o tiram:
//! a bola lê o sentido de uma viragem de `180°` pelo sinal de um produto vectorial que é ruído, e o
//! desfazer dos ganchos pára no tamanho da bola ([`crate::gancho`]) — e este recuo é maior.
//!
//! # A cura
//!
//! Um nó que vira acima de [`crate::gancho::VIRAGEM_DO_GANCHO`] e cujo segmento MAIS CURTO dos dois
//! que o tocam fica todo a menos de `tol` do MAIS LONGO é a ponta de um esporão: o mais longo é cortado
//! no ponto onde o curto acaba, o curto sai, e o nó da ponta com ele. A forma desenhada não muda —
//! ela é o mesmo conjunto de pontos —, e o comprimento não tem tecto, porque um esporão de área zero
//! nunca é forma, seja de que tamanho for.

use kurbo::{CubicBez, ParamCurve, Point, Vec2};
use ph2d_vec_scene::VecVertex;

/// Amostras por cúbica na comparação.
const AMOSTRAS: usize = 32;

fn pt(a: [f64; 2]) -> Point {
    Point::new(a[0], a[1])
}

fn arr(p: Point) -> [f64; 2] {
    [p.x, p.y]
}

fn cubica(verts: &[VecVertex], j: usize) -> CubicBez {
    let n = verts.len();
    let (c, q) = (&verts[j], &verts[(j + 1) % n]);
    CubicBez::new(
        pt(c.anchor),
        pt(c.out_handle),
        pt(q.in_handle),
        pt(q.anchor),
    )
}

/// `(distância, parâmetro)` do ponto de `c` mais perto de `p`, por amostragem densa e refinamento
/// local por secção áurea.
fn mais_perto(c: &CubicBez, p: Point) -> (f64, f64) {
    let n = 4 * AMOSTRAS;
    #[allow(clippy::cast_precision_loss)]
    let (mut t, mut d) = (0..=n)
        .map(|k| {
            let t = k as f64 / n as f64;
            (t, (c.eval(t) - p).hypot())
        })
        .fold((0.0, f64::INFINITY), |a, b| if b.1 < a.1 { b } else { a });
    #[allow(clippy::cast_precision_loss)]
    let passo = 1.0 / n as f64;
    let (mut lo, mut hi) = ((t - passo).max(0.0), (t + passo).min(1.0));
    let g = 0.5 * (5.0_f64.sqrt() - 1.0);
    for _ in 0..60 {
        let (x1, x2) = (hi - g * (hi - lo), lo + g * (hi - lo));
        if (c.eval(x1) - p).hypot() < (c.eval(x2) - p).hypot() {
            hi = x2;
        } else {
            lo = x1;
        }
    }
    let tm = 0.5 * (lo + hi);
    let dm = (c.eval(tm) - p).hypot();
    if dm < d {
        (t, d) = (tm, dm);
    }
    (d, t)
}

/// O segmento `curto` fica todo a menos de `tol` do `longo`?
fn em_cima(curto: &CubicBez, longo: &CubicBez, tol: f64) -> bool {
    (0..=AMOSTRAS).all(|k| {
        #[allow(clippy::cast_precision_loss)]
        let t = k as f64 / AMOSTRAS as f64;
        mais_perto(longo, curto.eval(t)).0 <= tol
    })
}

/// Amostras por cúbica na procura da passagem pela ponta.
const PASSAGENS: usize = 128;

/// ⭐ **O esporão DENTRO da cúbica** (medido a `(36°, −144°)`): a cúbica passa pelo nó de chegada,
/// segue `~1` solda além dele e VOLTA pela mesma recta. O nó vira só `21°` pelas tangentes, e a
/// quina VERDADEIRA — o fundo de uma cunha de `~20°` entre os dois membros, `~160°` de viragem —
/// fica escondida dentro da cúbica: nem a bola a lê como vinco nem o desfazer dos ganchos a pode
/// trocar (a viragem real passa dos `150°` dele). ⇒ a cúbica acaba onde passou pelo nó pela 1.ª
/// vez, se o resto dela for e voltar pelo MESMO caminho (a menos de `tol`) — e só se a quina que
/// isso expõe contra a direcção de saída do nó (`depois`, unitária) passar do
/// [`crate::gancho::VIRAGEM_DO_GANCHO`]. ⚠️ Abaixo dele a bola já a arredonda: medido a `(166°, −40°)`,
/// aparar uma quina escondida de `18°` trocava o arco que a bola lá punha por um nó em bico.
/// Devolve a cúbica aparada, ou `None` se não há o que aparar.
fn apara_a_ponta(c: &CubicBez, depois: Vec2, tol: f64) -> Option<CubicBez> {
    let d = |t: f64| (c.eval(t) - c.p3).hypot();
    #[allow(clippy::cast_precision_loss)]
    let ts: Vec<f64> = (0..=PASSAGENS)
        .map(|k| k as f64 / PASSAGENS as f64)
        .collect();
    let ds: Vec<f64> = ts.iter().map(|&t| d(t)).collect();
    let g = 0.5 * (5.0_f64.sqrt() - 1.0);
    let refina = |k: usize| {
        let (mut lo, mut hi) = (ts[k - 1], ts[k + 1]);
        for _ in 0..60 {
            let (x1, x2) = (hi - g * (hi - lo), lo + g * (hi - lo));
            if d(x1) < d(x2) {
                hi = x2;
            } else {
                lo = x1;
            }
        }
        0.5 * (lo + hi)
    };
    // A 1.ª passagem pelo nó: o 1.º mínimo local da distância a ele que, REFINADO, fica a menos de
    // `tol`. ⚠️ Refinado e não amostrado: uma cúbica rápida anda `~0,016` entre duas amostras, mais
    // que a solda, e passa exactamente pelo nó entre elas (o laço do gate). E sem cerca própria para
    // «PELO nó»: a cúbica aparada acaba nesse ponto com o nó onde estava — o desenho anda menos que
    // `tol`, e uma cerca mais apertada não tinha fixtura que a distinguisse (a mutação sobreviveu).
    let t1 = (1..PASSAGENS)
        .filter(|&k| ds[k] <= ds[k - 1] && ds[k] < ds[k + 1])
        .map(refina)
        .find(|&t| d(t) <= tol)?;
    // A cauda `[t1, 1]` vai até ao ponto mais longe do nó e VOLTA: as duas metades têm de ficar uma
    // em cima da outra (área zero). ⚠️ Não contra a cabeça: o pedaço a mais passa ALÉM do nó, logo
    // não está em cima de nada que a cúbica já tenha feito.
    #[allow(clippy::cast_precision_loss)]
    let ponta = (0..=PASSAGENS)
        .map(|k| t1 + (1.0 - t1) * k as f64 / PASSAGENS as f64)
        .fold((t1, 0.0), |a, t| if d(t) > a.1 { (t, d(t)) } else { a })
        .0;
    let (ida, volta) = (c.subsegment(t1..ponta), c.subsegment(ponta..1.0));
    if !(em_cima(&volta, &ida, tol) && em_cima(&ida, &volta, tol)) {
        return None;
    }
    let a = c.subsegment(0.0..t1);
    let chega = [a.p2, a.p1, a.p0]
        .into_iter()
        .find_map(|q| unit(a.p3 - q))?;
    let vira = chega.dot(depois).clamp(-1.0, 1.0).acos().to_degrees();
    (vira > crate::gancho::VIRAGEM_DO_GANCHO).then_some(a)
}

fn unit(v: Vec2) -> Option<Vec2> {
    let l = v.hypot();
    (l > 1e-12 * (1.0 + v.x.abs().max(v.y.abs()))).then(|| v / l)
}

/// [`apara_a_ponta`] nas DUAS pontas da cúbica `j` — a de partida pela cúbica invertida. Uma ponta
/// que é QUINA DO ARTISTA não se apara: ali a ponta é desenho.
fn apara(verts: &mut [VecVertex], j: usize, quinas: &[([f64; 2], f64)], tol: f64) {
    let n = verts.len();
    let jb = (j + 1) % n;
    let c = cubica(verts, j);
    let saida = |i: usize, v: &[VecVertex]| {
        crate::overlap::tangentes_do_vertice(v, i)
            .map(|(e, s)| (Vec2::new(e[0], e[1]), Vec2::new(s[0], s[1])))
    };
    if !crate::bola::quina_do_artista(verts, jb, quinas)
        && let Some((_, depois)) = saida(jb, verts)
        && let Some(a) = apara_a_ponta(&c, depois, tol)
    {
        verts[j].out_handle = arr(a.p1);
        verts[jb].in_handle = arr(a.p2);
    }
    let c = cubica(verts, j);
    let inv = CubicBez::new(c.p3, c.p2, c.p1, c.p0);
    // Ao contrário, quem «sai» do nó `j` é o contorno a recuar pela tangente de ENTRADA dele.
    if !crate::bola::quina_do_artista(verts, j, quinas)
        && let Some((antes, _)) = saida(j, verts)
        && let Some(a) = apara_a_ponta(&inv, -antes, tol)
    {
        verts[jb].in_handle = arr(a.p1);
        verts[j].out_handle = arr(a.p2);
    }
}

/// ⭐⭐ **Tira os esporões do contorno fechado `verts`** — ver o módulo. `quinas` são as quinas do
/// artista (as da bola, [`crate::bola::quina_do_artista`]): uma ponta que ele DESENHOU — um bigode
/// traçado em ida e volta — fica. Nada a tirar ⇒ `verts` intacto, ao bit.
#[must_use]
pub fn tira_os_esporoes(
    mut verts: Vec<VecVertex>,
    quinas: &[([f64; 2], f64)],
    tol: f64,
) -> Vec<VecVertex> {
    if !tol.is_finite() || tol <= 0.0 {
        return verts;
    }
    for j in 0..verts.len() {
        apara(&mut verts, j, quinas, tol);
    }
    // Cada volta tira um esporão; um contorno de `n` nós tem no máximo `n` deles.
    for _ in 0..verts.len() {
        let n = verts.len();
        if n < 4 {
            return verts;
        }
        let ponta = (0..n).find(|&i| {
            !crate::bola::quina_do_artista(&verts, i, quinas)
                && crate::overlap::viragem_do_vertice(&verts, i)
                    .is_some_and(|v| v > crate::gancho::VIRAGEM_DO_GANCHO)
                && {
                    let (ant, seg) = (cubica(&verts, (i + n - 1) % n), cubica(&verts, i));
                    if ant.p0.distance(ant.p3) >= seg.p0.distance(seg.p3) {
                        em_cima(&seg, &ant, tol)
                    } else {
                        em_cima(&ant, &seg, tol)
                    }
                }
        });
        let Some(i) = ponta else {
            return verts;
        };
        let (a, b) = ((i + n - 1) % n, (i + 1) % n);
        let (ant, seg) = (cubica(&verts, a), cubica(&verts, i));
        if ant.p0.distance(ant.p3) >= seg.p0.distance(seg.p3) {
            // O curto é o de SAÍDA: o anterior acaba onde ele acaba, no nó `b`.
            let (_, t) = mais_perto(&ant, seg.p3);
            let c = ant.subsegment(0.0..t);
            verts[a].out_handle = arr(c.p1);
            verts[b].in_handle = arr(c.p2);
        } else {
            // O curto é o de ENTRADA: o seguinte começa onde ele começa, no nó `a`.
            let (_, t) = mais_perto(&seg, ant.p0);
            let c = seg.subsegment(t..1.0);
            verts[a].out_handle = arr(c.p1);
            verts[b].in_handle = arr(c.p2);
        }
        verts.remove(i);
    }
    verts
}

#[cfg(test)]
#[path = "esporao_tests.rs"]
mod tests;

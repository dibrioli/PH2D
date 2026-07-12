//! **Arredondar as quinas de uma polilinha** — o filete de cada dobra.
//!
//! É o que transforma o cotovelo duro de um fluxograma numa linha que dobra suave. O uso
//! imediato é o conector (`VecConnector::corner_radius`), mas a função é geral: entra uma
//! polilinha, sai o mesmo caminho com as quinas cortadas por arcos tangentes.
//!
//! # O arco é EXATO em qualquer ângulo — e isso não custou transcendental nenhuma
//!
//! A tentação é escrever o caso fácil: as rotas ortogonais só têm quinas de 90°, e para 90°
//! o controle da cúbica fica a `KAPPA · t` do vértice (`KAPPA = 0,5523`), fim. Funciona — até
//! alguém arredondar uma polilinha que não veio do roteador (uma rota reta em diagonal, um
//! caminho da caneta), e aí o "arco" deixa de ser um arco. A forma sai errada de um jeito
//! difícil de ver e impossível de explicar.
//!
//! O geral custa três raízes quadradas. Com `θ` = o ângulo INTERNO da quina, o arco que a
//! substitui vira `φ = π − θ`, e a aproximação cúbica de um arco de ângulo `φ` e raio `R` põe
//! os controles a `h = (4/3)·tan(φ/4)·R` **ao longo das tangentes**. Tudo isso sai de
//! `cos θ = u₁·u₂` por meias-tangentes, sem uma única chamada trigonométrica (HR-5):
//!
//! ```text
//! cos(θ/2) = √((1+cos θ)/2)      sin(θ/2) = √((1−cos θ)/2)
//! t = R · cos(θ/2)/sin(θ/2)       (a distância do vértice ao ponto de tangência)
//! h = (4/3) · t · sin(θ/2)/(1 + sin(θ/2))
//! ```
//!
//! Confira em 90°: `sin(θ/2) = √2/2`, logo `h = (4/3)·t·0,7071/1,7071 = 0,5523·t` — o `KAPPA`,
//! de volta, como caso particular. É o mesmo número, e agora ele é uma consequência em vez de
//! uma constante mágica.

use crate::VecVertex;

/// Tolerância geométrica. Abaixo disto uma aresta é um ponto e uma quina é uma reta.
const EPS: f64 = 1e-12;

/// O vetor unitário de `a` para `b`, e o comprimento. `None` se os dois pontos coincidem.
fn unit(a: [f64; 2], b: [f64; 2]) -> Option<([f64; 2], f64)> {
    let d = [b[0] - a[0], b[1] - a[1]];
    let len = d[0].hypot(d[1]);
    if len < EPS {
        return None;
    }
    Some(([d[0] / len, d[1] / len], len))
}

/// **A polilinha com as quinas arredondadas.** `radius ≤ 0` devolve os vértices de quina
/// intactos — o caminho afiado é o default, e ele tem de sair **idêntico**.
///
/// O raio é um TETO, não uma promessa: cada filete é clampado a **metade da menor aresta
/// vizinha**. Sem isso, dois filetes numa mesma aresta curta se comeriam, e a linha faria um
/// laço para trás — o modo mais feio possível de um raio grande demais falhar. Com o clamp, um
/// raio exagerado apenas satura: a quina fica o mais redonda que aquela aresta permite.
///
/// As **pontas não são tocadas**: o primeiro e o último vértice ficam onde estão, e com eles a
/// tangente que a ponta de seta usa para saber para onde apontar.
#[must_use]
pub fn round_polyline(pts: &[[f64; 2]], radius: f64) -> Vec<VecVertex> {
    if radius <= EPS || pts.len() < 3 {
        return pts.iter().map(|&p| VecVertex::corner(p)).collect();
    }

    let mut out = Vec::with_capacity(pts.len() * 2);
    out.push(VecVertex::corner(pts[0]));

    for i in 1..pts.len() - 1 {
        let (a, b, c) = (pts[i - 1], pts[i], pts[i + 1]);
        let (Some((u1, la)), Some((u2, lc))) = (unit(b, a), unit(b, c)) else {
            out.push(VecVertex::corner(b)); // aresta degenerada: não há quina a cortar
            continue;
        };

        // O ângulo INTERNO da quina, por meias-tangentes (zero transcendental).
        let cos_t = (u1[0] * u2[0] + u1[1] * u2[1]).clamp(-1.0, 1.0);
        let half_c = ((1.0 + cos_t) * 0.5).sqrt(); // cos(θ/2)
        let half_s = ((1.0 - cos_t) * 0.5).sqrt(); // sin(θ/2)
        if half_c < EPS || half_s < EPS {
            // θ = 180° (colineares: não há quina) ou θ = 0° (a linha volta sobre si mesma, e
            // não há filete que faça sentido). Nos dois casos, o vértice fica como está.
            out.push(VecVertex::corner(b));
            continue;
        }

        // A distância do vértice ao ponto de tangência — clampada a metade de cada aresta, que
        // é o que impede dois filetes vizinhos de se comerem.
        let t = (radius * half_c / half_s).min(la * 0.5).min(lc * 0.5);
        // O braço da cúbica, ao longo das tangentes. Em 90° isto é exatamente KAPPA·t.
        let h = (4.0 / 3.0) * t * half_s / (1.0 + half_s);

        let p1 = [b[0] + u1[0] * t, b[1] + u1[1] * t];
        let p2 = [b[0] + u2[0] * t, b[1] + u2[1] * t];
        // Os controles caminham de cada ponto de tangência DE VOLTA para o vértice.
        out.push(VecVertex::smooth(
            p1,
            p1,
            [p1[0] - u1[0] * h, p1[1] - u1[1] * h],
        ));
        out.push(VecVertex::smooth(
            p2,
            [p2[0] - u2[0] * h, p2[1] - u2[1] * h],
            p2,
        ));
    }

    out.push(VecVertex::corner(pts[pts.len() - 1]));
    out
}

#[cfg(test)]
#[path = "polyline_tests.rs"]
mod tests;

/// **A polilinha SUAVIZADA num spline** — o conector "curvo".
///
/// # A decisão: curvo é a rota ORTOGONAL suavizada, não uma curva à parte
///
/// O caminho óbvio seria uma cúbica única de A a B, com os controles ao longo das direções de
/// saída. É o que o Visio faz, é bonito entre duas caixas — e **atravessa qualquer coisa que
/// esteja no meio**, porque não passou pelo roteador.
///
/// Aqui o conector curvo roda exatamente a MESMA busca A\* que o ortogonal, com os MESMOS
/// obstáculos, e só a escrita da geometria muda: as dobras do cotovelo viram pontos de
/// controle de um spline em vez de quinas. O desvio de obstáculo sai **de graça**, e não há um
/// segundo roteador para manter em pé.
///
/// # O braço do handle é proporcional ao SEGMENTO, não à corda dos vizinhos
///
/// A tangente num ponto interno é a **direção** da corda entre os vizinhos — isso é
/// Catmull-Rom, e está certo. O que a primeira versão errou foi o **comprimento**: ela usava a
/// corda inteira (`(next − prev) · k`), e numa rota ortogonal com uma perna longa e outra curta
/// essa corda é muito maior que o segmento curto. O handle passa do vértice seguinte, a cúbica
/// estoura para fora do caminho, e o conector vira um S enorme que não tem nada a ver com a
/// rota que o A\* escolheu. Foi o que o Enio viu.
///
/// A cura é dar a **cada lado o seu próprio comprimento**: o braço de saída mede uma fração do
/// segmento que ele vai percorrer; o de entrada, do segmento de onde veio. Com `arm = 1/3` isto
/// é o spline cardinal canônico — passa pelos pontos, e não escapa deles.
///
/// **As duas pontas guardam a tangente do próprio segmento** — é dela que a ponta de seta tira
/// a direção, e uma tangente "suavizada" no extremo faria a seta apontar para o lado.
///
/// `arm` é o comprimento do braço em frações do segmento (o ajuste do painel: "mais perto ou
/// mais longe do ponto"). `0` devolve a polilinha crua; `1/3` é o canônico; acima disso a curva
/// abre de propósito.
#[must_use]
pub fn smooth_polyline(pts: &[[f64; 2]], arm: f64) -> Vec<VecVertex> {
    let n = pts.len();
    if n < 3 || arm <= EPS {
        return pts.iter().map(|&p| VecVertex::corner(p)).collect();
    }
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let a = pts[i];
        let (prev, next) = (pts[i.saturating_sub(1)], pts[(i + 1).min(n - 1)]);
        // A DIREÇÃO da tangente: a corda entre os vizinhos (nas pontas, o próprio segmento).
        let Some((dir, _)) = unit(prev, next) else {
            out.push(VecVertex::corner(a));
            continue;
        };
        // O COMPRIMENTO de cada braço: uma fração do segmento QUE ELE PERCORRE. É isto que
        // impede o handle de passar do vértice seguinte.
        let back = (a[0] - prev[0]).hypot(a[1] - prev[1]) * arm;
        let fwd = (next[0] - a[0]).hypot(next[1] - a[1]) * arm;
        let i_h = [a[0] - dir[0] * back, a[1] - dir[1] * back];
        let o_h = [a[0] + dir[0] * fwd, a[1] + dir[1] * fwd];
        out.push(match i {
            0 => VecVertex::smooth(a, a, o_h),
            _ if i == n - 1 => VecVertex::smooth(a, i_h, a),
            _ => VecVertex::smooth(a, i_h, o_h),
        });
    }
    out
}

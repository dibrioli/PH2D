//! **Setas** — módulo irmão de [`crate::shapes`], autorado no espaço unitário
//! ([`crate::space`]: `u` 0→1 da esquerda para a direita, `v` 0→1 do TOPO para a base).
//!
//! Cuidado com a palavra "seta": num editor vetorial ela é **duas coisas diferentes**.
//! Aqui estão as setas-FORMA — contornos preenchíveis, que se escalam, giram e recebem
//! gradiente como qualquer outra forma. A outra é a *ponta de traço* (arrowhead), que é
//! propriedade do **stroke** e vale para qualquer path aberto; essa não mora aqui.
//!
//! ## As três leis que estas cinco formas obedecem
//!
//! A geometria vem da definição publicada do OOXML (ECMA-376, Parte 1, Apêndice D), lida
//! como referência e re-derivada em forma fechada. Dela saem três invariantes que valem
//! para as 24 setas do padrão — e que a versão anterior deste módulo violava:
//!
//! 1. **A ponta é UM ponto sobre a linha de centro**, e os flancos da cabeça são
//!    **retas**. Nenhuma das 24 setas do ECMA-376 tem uma única `cubicBezTo`: todo `arcTo`
//!    delas pertence a uma faixa ou a um cotovelo, nunca ao flanco de uma cabeça.
//! 2. **A contenção na caixa é resolvida ANTES, encolhendo a faixa** pelo transbordo da
//!    cabeça (`r_out = 0,5 + th/2 − hh`) — nunca clampando a cabeça depois. É a linha que
//!    faz a esquina externa da cabeça pousar EXATAMENTE no círculo inscrito.
//! 3. **Curva é CURVA.** Todo arco sai em cúbicas de Bézier de ≤45° ([`arc45`]). A versão
//!    anterior amostrava o arco da seta circular em cordas retas de 15°: o desvio radial
//!    de uma corda de 15° é `8,6e-3·r` — **8,6 px** num raio de 1000, visível a olho nu, e
//!    foi exatamente o polígono que o Enio fotografou. A cúbica de 45° erra `4,2e-6·r`.
//!
//! Os parâmetros são **frações** da caixa, não medidas absolutas — assim a seta guarda a
//! proporção ao ser redimensionada, que é o que se espera de uma forma paramétrica.

use crate::space::{Unit, Uv, closed, poly, straighten};
use crate::{VecPath, VecVertex};

/// **Arco em cúbicas de ≤45°** — o antídoto da poligonização.
///
/// [`Unit::arc`] já emite cúbicas exatas, mas fatia em pedaços de 90° (desvio radial
/// `2,7e-4·r`). Aqui os pedaços são de ≤45° (`4,2e-6·r`, ~60× melhor) — o número que a
/// referência exige. Como cada chamada interna recebe um sweep de ≤45°, ela devolve **uma**
/// cúbica (`n = ceil(|s|/90) = 1`), e o vértice de junção — que já vem completo, com os
/// dois handles calculados pelo mesmo comprimento `h` — é reaproveitado, não duplicado.
fn arc45(u: &Unit, c: Uv, ru: f64, rv: f64, start_deg: f64, sweep_deg: f64) -> Vec<VecVertex> {
    let steps = arc_segments(sweep_deg);
    let seg = sweep_deg / steps as f64;
    let mut out = Vec::with_capacity(steps + 1);
    for i in 0..steps {
        let part = u.arc(c, ru, rv, start_deg + seg * i as f64, seg);
        // O primeiro vértice de cada pedaço (menos o inicial) É o último do anterior.
        out.extend(part.into_iter().skip(usize::from(i > 0)));
    }
    out
}

/// Quantas cúbicas [`arc45`] emite para este sweep. É a ÚNICA definição do fatiamento —
/// o teste a usa para saber onde um arco acaba e o outro começa.
#[expect(clippy::cast_sign_loss, reason = "ceil(|x|).max(1) e positivo")]
fn arc_segments(sweep_deg: f64) -> usize {
    (sweep_deg.abs() / 45.0).ceil().max(1.0) as usize
}

/// Um quarto de elipse — ou uma QUINA, quando o raio degenera. É o que faz
/// `corner = 0` virar o cotovelo quadrado sem um caso especial espalhado pelo desenho.
fn elbow(u: &Unit, c: Uv, ru: f64, rv: f64, start_deg: f64, sweep_deg: f64) -> Vec<VecVertex> {
    if ru.abs() < 1e-9 && rv.abs() < 1e-9 {
        return vec![u.corner(c)]; // os dois extremos do arco colapsam no centro
    }
    arc45(u, c, ru, rv, start_deg, sweep_deg)
}

/// **Seta reta.** Haste retangular + cabeça triangular apontando para +X: a ponta é um
/// ponto ÚNICO, encostada na borda direita da caixa e na linha de centro.
///
/// - `tail` = espessura da haste (fração da altura);
/// - `head_len` = comprimento da cabeça (fração da largura);
/// - `head_w` = largura CHEIA da cabeça (fração da altura) — `1,0` = a cabeça ocupa a
///   caixa toda. O OOXML fixa isto em 1,0; aqui é parâmetro.
#[must_use]
pub fn arrow_block(a: [f64; 2], b: [f64; 2], tail: f64, head_len: f64, head_w: f64) -> VecPath {
    let u = Unit::of(a, b);
    let body = tail.clamp(0.02, 1.0);
    // A cabeça nunca é mais estreita que a haste, senão o contorno inverte.
    let span = head_w.clamp(0.05, 1.0).max(body);
    let x1 = 1.0 - head_len.clamp(0.02, 1.0);
    let (y1, y2) = (0.5 - body * 0.5, 0.5 + body * 0.5);
    let (h1, h2) = (0.5 - span * 0.5, 0.5 + span * 0.5);
    poly(
        &u,
        &[
            (0.0, y1),
            (x1, y1),
            (x1, h1),
            (1.0, 0.5), // a PONTA
            (x1, h2),
            (x1, y2),
            (0.0, y2),
        ],
    )
}

/// **Seta dupla** (cabeça nas duas pontas) — a mesma haste, espelhada. As duas pontas
/// encostam nas bordas esquerda e direita, ambas na linha de centro.
#[must_use]
pub fn arrow_double(a: [f64; 2], b: [f64; 2], tail: f64, head_len: f64, head_w: f64) -> VecPath {
    let u = Unit::of(a, b);
    let body = tail.clamp(0.02, 1.0);
    let span = head_w.clamp(0.05, 1.0).max(body);
    // Cada cabeça leva `head_len` da largura; juntas não podem passar da caixa.
    let l = head_len.clamp(0.02, 0.5);
    let (x2, x3) = (l, 1.0 - l);
    let (y1, y2) = (0.5 - body * 0.5, 0.5 + body * 0.5);
    let (h1, h2) = (0.5 - span * 0.5, 0.5 + span * 0.5);
    poly(
        &u,
        &[
            (0.0, 0.5), // ponta ESQUERDA
            (x2, h1),
            (x2, y1),
            (x3, y1),
            (x3, h1),
            (1.0, 0.5), // ponta DIREITA
            (x3, h2),
            (x3, y2),
            (x2, y2),
            (x2, h2),
        ],
    )
}

/// **Seta em L** (cotovelo): entra pela esquerda rente à base, dobra e sai para **CIMA**.
/// É a seta de fluxo — a que liga uma caixa à de cima sem um conector de verdade.
///
/// O cotovelo é um par de arcos **concêntricos** (raios `corner` e `corner − espessura`),
/// que é o que mantém a espessura constante ao longo da curva; `corner = 0` degenera no
/// cotovelo quadrado (o `bentUpArrow` do OOXML) sem uma linha de código a mais.
///
/// - `tail` = espessura do corpo, fração do lado **CURTO** da caixa: os dois braços são
///   perpendiculares, então medir cada um no seu próprio eixo os deixaria com espessuras
///   diferentes numa caixa larga (é a normalização por `ss = min(w, h)` do OOXML). É
///   também o que faz o cotovelo sair CIRCULAR, e não elíptico;
/// - `head_len` = comprimento da cabeça (fração da altura);
/// - `head_w` = largura CHEIA da cabeça (fração da largura);
/// - `corner` = raio EXTERNO do cotovelo (fração do lado curto).
#[must_use]
pub fn arrow_bent(
    a: [f64; 2],
    b: [f64; 2],
    tail: f64,
    head_len: f64,
    head_w: f64,
    corner: f64,
) -> VecPath {
    let u = Unit::of(a, b);
    // Uma medida de MUNDO vale `k.0` no eixo `u` e `k.1` no eixo `v`.
    let s = u.aspect();
    let k = (s.min(1.0), (1.0 / s).min(1.0));
    let body = tail.clamp(0.02, 0.9);
    let (bu, bv) = (body * k.0, body * k.1);

    // A cabeça: mais larga que a haste (senão o contorno inverte) e curta o bastante para
    // pousar ACIMA do braço horizontal (senão o braço a engoliria e o contorno cruzaria).
    let span = head_w.clamp(0.05, 1.0).max(bu);
    let l = head_len.clamp(0.05, 0.6).min((1.0 - bv) * 0.8);
    let (hw, sx) = (span * 0.5, 1.0 - span * 0.5); // meia-largura + eixo da haste vertical
    let (left, right) = (sx - bu * 0.5, sx + bu * 0.5); // as duas faces da haste

    // O raio é UM número de mundo, projetado nos dois eixos (é isso que o deixa CIRCULAR).
    // Os tetos vêm do desenho: o arco não pode furar a borda esquerda nem subir acima da
    // base da cabeça — e as margens deixam sobrar um pedaço de cauda e um de haste, senão
    // o extremo do slider colapsaria um lado reto em nada.
    let r_max = (right * 0.95 / k.0.max(1e-9)).min((1.0 - l) * 0.9 / k.1.max(1e-9));
    let r = corner.clamp(0.0, 1.0).min(r_max);
    let (ru, rv) = (r * k.0, r * k.1);
    let (iu, iv) = ((ru - bu).max(0.0), (rv - bv).max(0.0)); // o arco INTERNO, concêntrico
    let c_out: Uv = (right - ru, 1.0 - rv);
    let c_in: Uv = (left - iu, 1.0 - bv - iv);

    let mut verts = vec![u.corner((0.0, 1.0))]; // a cauda, no canto inferior esquerdo
    let tail_end = verts.len() - 1;
    // Cotovelo EXTERNO: da base (90°) para a face direita da haste (0°).
    verts.extend(elbow(&u, c_out, ru, rv, 90.0, -90.0));
    let out_end = verts.len() - 1;

    verts.push(u.corner((right, l)));
    verts.push(u.corner((sx + hw, l))); // esquina direita da cabeça
    verts.push(u.corner((sx, 0.0))); // a PONTA — para CIMA, na borda de cima
    verts.push(u.corner((sx - hw, l))); // esquina esquerda da cabeça
    verts.push(u.corner((left, l)));
    let head_end = verts.len() - 1;
    // Cotovelo INTERNO: da face esquerda da haste (0°) de volta para a aresta de cima do
    // braço horizontal (90°) — mesmo centro do externo, deslocado pela espessura.
    verts.extend(elbow(&u, c_in, iu, iv, 0.0, 90.0));
    let in_end = verts.len() - 1;
    verts.push(u.corner((0.0, 1.0 - bv)));

    // Toda junção arco↔reta é QUINA: sem isto o vértice do arco leva a sua tangente para
    // dentro da reta vizinha, que nasce com handles fantasmas que o usuário vê e arrasta.
    for i in [tail_end, out_end, head_end, in_end] {
        straighten(&mut verts, i);
    }
    closed(verts)
}

/// **Chevron** (seta-fita de processo): pentágono com um entalhe atrás, para encaixar no
/// próximo — o vocabulário de "etapa 1, etapa 2" de qualquer diagrama. Com `notch` igual a
/// `point` os chevrons ladrilham sem folga; com `notch = 0` é o pentágono (`homePlate`).
#[must_use]
pub fn chevron(a: [f64; 2], b: [f64; 2], point: f64, notch: f64) -> VecPath {
    let u = Unit::of(a, b);
    let x2 = 1.0 - point.clamp(0.02, 0.9);
    // O entalhe entra pela traseira, mas nunca alcança o pescoço da ponta.
    let x1 = notch.clamp(0.0, 0.9).min(x2 * 0.9);
    poly(
        &u,
        &[
            (0.0, 0.0),
            (x2, 0.0),
            (1.0, 0.5), // a PONTA
            (x2, 1.0),
            (0.0, 1.0),
            (x1, 0.5), // o entalhe
        ],
    )
}

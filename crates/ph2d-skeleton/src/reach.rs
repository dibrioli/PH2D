//! **O ALCANCE** — cinemática INVERSA: o artista põe a ponta onde quer, e a corrente dobra.
//!
//! É o degrau que separa *"um editor de esqueletos"* de *"um editor de animação"*: com FK o artista
//! gira o ombro, gira o cotovelo, gira o punho e a mão cai onde cair — e ele sabe onde a MÃO tem de
//! estar; os ângulos são exactamente o que ele não quer digitar.
//!
//! # ⭐ Duas leis, e a partição é do problema, não de conveniência
//!
//! | cadeia | lei | por quê |
//! |---|---|---|
//! | **2 ossos** | lei dos cossenos | o triângulo tem os três lados conhecidos ⇒ **fechado e EXACTO** |
//! | **3+** | FABRIK | não há forma fechada; o Aristidou & Lasenby (2011) é o padrão-ouro |
//!
//! ⚠️ **Herdadas da família `ph2d-node-rig-*`**, que as trouxe medidas contra Rive/Spine/Blender em
//! 2026-07-12 — ⛔ aquelas crates são **outro substrato** (uma stream de instâncias do grafo de nós,
//! não entidades da cena) e continuam de pé; o que se herda é a MATEMÁTICA, que é a mesma.
//!
//! # ⭐⭐ Sem `acos`, e não por aproximar o solver (HR-5)
//!
//! O manual escreve `a = acos((l1² + d² − l2²) / (2·l1·d))` — mas o ângulo nunca é preciso, só a
//! DIRECÇÃO do primeiro osso. Ficando em vectores, `cos a` sai da lei dos cossenos, `sin a = √(1 −
//! cos²a)`, e rodar um unitário por um ângulo cujo seno e cosseno já estão na mão é multiplicação e
//! soma. **O solve é EXACTO** — não há polinómio nenhum a aproximar `acos`.
//!
//! # ⭐⭐⭐ A SOFTNESS, e por que ela também é algébrica
//!
//! Sem ela o `d` é cortado a seco em `l1 + l2`, e o joelho **estala** no instante em que a corrente
//! estica: a mão pára de repente e o cotovelo dá um solavanco. É o *Softness* do Spine —
//! *«slows down the bones as the constrained bones straighten»*.
//!
//! A lei é um amortecimento **racional**, sem transcendental (HR-5):
//!
//! ```text
//! sobra   = d − (R − s)
//! d_efic  = (R − s) + s · sobra / (s + sobra)
//! ```
//!
//! ⚠️ Ela tem as três propriedades que a fazem servir, e todas se verificam à mão: em `sobra = 0`
//! vale `R − s` (**contínua**), a derivada aí é `1` (**C¹**, sem quina), e quando `sobra → ∞` tende
//! a `R` **sem nunca lá chegar** — a corrente aproxima-se da recta e não trava. ⛔ A forma óbvia
//! (`1 − e^{-x}`) tem as mesmas propriedades e paga um transcendental.

/// Quantas passagens do FABRIK, e quanto se amortece a extensão máxima.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Reach {
    /// Passagens do FABRIK. Só conta numa corrente de 3+ ossos — a de 2 é fechada.
    pub iterations: usize,
    /// A **SOFTNESS** do Spine, em unidades de MUNDO: a que distância do alcance máximo os ossos
    /// começam a abrandar. `0` ⇒ o corte a seco de sempre.
    pub softness: f64,
}

impl Default for Reach {
    fn default() -> Self {
        Self {
            iterations: DEFAULT_ITERATIONS,
            softness: 0.0,
        }
    }
}

/// Passagens por omissão — **MEDIDO** (2026-09-06,
/// `measure_how_many_passes_fabrik_actually_needs`), não escolhido:
///
/// | ossos | passagens até o erro cair abaixo da tolerância |
/// |---|---|
/// | 3 | 8 |
/// | 4 | 9 |
/// | 6 | 12 |
/// | 12 | 22 |
/// | 24 | **40** |
///
/// ⚠️ **A 1.ª redacção pôs `10` de cabeça, e a medição mostrou que isso PARTE uma cauda longa**: a
/// 12 ossos ela precisa de 22 e a 24 de 40 — com `10` a ponta simplesmente não chega, em silêncio.
///
/// ⭐ **E subir não custa nada, porque o laço SAI CEDO**: ele quebra assim que o erro cai, então
/// uma corrente de 3 ossos continua a fazer 8 passagens. O número é um TECTO do laço, não um custo
/// fixo. Preço no pior caso medido (24 ossos × 40): `≈ 17 µs`, **0,10 % de um quadro**.
///
/// ⛔ A família de nós de onde esta lei veio tinha aqui um número **sem medição nenhuma ao lado**, e
/// a folha 16 da conferência do Motion nomeia-o exactamente assim.
pub const DEFAULT_ITERATIONS: usize = 40;

/// Tecto duro de passagens. ⚠️ **O recurso NÃO é o relógio** — a `24` ossos × `64` passagens o
/// alcance custa `≈ 27 µs`, `0,16 %` de um quadro (`measure_the_price_of_one_reach`). O recurso é a
/// **confiança**: o `iterations` pode um dia vir de um documento carregado ou de uma edição por
/// MCP, e um laço cujo limite vem de um ficheiro é uma porta aberta. `64` é `1,6×` o pior caso
/// medido, então ele nunca aperta um uso honesto.
pub const MAX_ITERATIONS: usize = 64;

/// Quando o erro cai abaixo disto, a corrente chegou — em fracção do alcance total, para a
/// tolerância ser adimensional (um rig de 10 unidades e outro de 10 000 param no mesmo sítio).
const TOLERANCE: f64 = 1e-4;

/// Quanto se arqueia uma corrente perfeitamente recta antes de iterar, em fracção do alcance.
///
/// ⛔ **Sem isto o FABRIK fica preso:** numa corrente recta com o alvo fora da recta, cada passagem
/// devolve exactamente a mesma recta — não há lado para onde cair. E o caso não é exótico: é o
/// estado em que um esqueleto acabado de desenhar nasce.
const BOW: f64 = 1e-3;

fn unit(v: [f64; 2], fallback: [f64; 2]) -> [f64; 2] {
    let n = v[0].hypot(v[1]);
    if n > f64::EPSILON {
        [v[0] / n, v[1] / n]
    } else {
        fallback
    }
}

/// **A distância efectiva ao alvo**, com a extensão máxima amortecida.
///
/// ⚠️ `None` de propósito não existe: com `softness <= 0` ela é o corte a seco, que é o
/// comportamento de sempre **ao bit**.
#[must_use]
pub fn softened_distance(d: f64, reach: f64, softness: f64) -> f64 {
    if softness <= 0.0 || !softness.is_finite() {
        return d.min(reach);
    }
    let s = softness.min(reach);
    let joelho = reach - s;
    if d <= joelho {
        return d;
    }
    let sobra = d - joelho;
    joelho + s * sobra / (s + sobra)
}

/// **A corrente de DOIS ossos, fechada e exacta.**
///
/// `bend` é o sinal do produto vectorial que a pose ACTUAL tem — ⭐ e é assim, e não por um bit de
/// «lado», que se evita o defeito clássico: um cotovelo que **salta** para o outro lado no instante
/// em que a mão cruza a recta. *O solver preserva a dobra que o artista já vê.*
fn two_bone(
    root: [f64; 2],
    l1: f64,
    l2: f64,
    goal: [f64; 2],
    bend: f64,
    soft: f64,
) -> [[f64; 2]; 2] {
    let to_goal = [goal[0] - root[0], goal[1] - root[1]];
    let u = unit(to_goal, [1.0, 0.0]);
    let bruto = to_goal[0].hypot(to_goal[1]);
    // Longe demais ⇒ amortecido; perto demais (a corrente dobrada sobre si) ⇒ o piso é o que os
    // dois ossos conseguem encolher.
    let d = softened_distance(bruto, l1 + l2, soft).max((l1 - l2).abs().max(f64::EPSILON));
    // Higiene de vírgula flutuante: na extensão máxima o quociente é exactamente `1` em aritmética
    // real e `1 + 1e-16` nesta — e a raiz de um negativo devolveria um membro `NaN`.
    let cos_a = ((l1 * l1 + d * d - l2 * l2) / (2.0 * l1 * d)).clamp(-1.0, 1.0);
    let sin_a = (1.0 - cos_a * cos_a).sqrt() * if bend < 0.0 { -1.0 } else { 1.0 };
    let osso1 = [u[0] * cos_a - u[1] * sin_a, u[0] * sin_a + u[1] * cos_a];
    [
        [root[0] + l1 * osso1[0], root[1] + l1 * osso1[1]],
        [root[0] + d * u[0], root[1] + d * u[1]],
    ]
}

/// Arqueia uma corrente RECTA para o FABRIK ter um lado para onde cair.
fn break_collinearity(p: &mut [[f64; 2]], reach: f64, goal: [f64; 2]) {
    let n = p.len();
    let to_goal = [goal[0] - p[0][0], goal[1] - p[0][1]];
    let d = to_goal[0].hypot(to_goal[1]);
    if reach - d < BOW * reach {
        return; // o alvo está na extensão máxima (ou além): a recta É a resposta
    }
    let u = unit(to_goal, [1.0, 0.0]);
    let torto = (1..n)
        .map(|i| {
            let v = [p[i][0] - p[0][0], p[i][1] - p[0][1]];
            (v[0] * u[1] - v[1] * u[0]).abs()
        })
        .fold(0.0, f64::max);
    if torto > BOW * reach {
        return; // já está fora da recta — a iteração tem para onde cair
    }
    let perp = [-u[1], u[0]];
    let arco = BOW * reach;
    for q in p.iter_mut().take(n - 1).skip(1) {
        q[0] += perp[0] * arco;
        q[1] += perp[1] * arco;
    }
}

/// **A CORRENTE ALCANÇA O ALVO** — em lugar, sobre as posições de MUNDO das juntas.
///
/// `joints` tem `lengths.len() + 1` entradas: `joints[0]` é a âncora (fica onde está) e
/// `joints[i+1]` é a ponta do osso `i`.
///
/// ⚠️ **Fora de alcance, a corrente ESTICA na direcção do alvo** e nunca se rasga: é o que todo
/// solver faz, e é o que um braço faz. ⛔ Ela nunca alonga um osso — os comprimentos são invariantes
/// das duas leis, e há gate.
pub fn reach(joints: &mut [[f64; 2]], lengths: &[f64], goal: [f64; 2], opts: Reach) {
    if lengths.is_empty() || joints.len() != lengths.len() + 1 {
        return;
    }
    let total: f64 = lengths.iter().sum();
    if total <= f64::EPSILON {
        return;
    }
    if lengths.len() == 2 {
        // O sinal da dobra ACTUAL, para a preservar.
        let (a, b, c) = (joints[0], joints[1], joints[2]);
        let (v1, v2) = ([b[0] - a[0], b[1] - a[1]], [c[0] - b[0], c[1] - b[1]]);
        let bend = v1[0] * v2[1] - v1[1] * v2[0];
        let [cotovelo, mao] = two_bone(a, lengths[0], lengths[1], goal, bend, opts.softness);
        joints[1] = cotovelo;
        joints[2] = mao;
        return;
    }
    // Um osso só: aponta, e o comprimento manda.
    if lengths.len() == 1 {
        let u = unit([goal[0] - joints[0][0], goal[1] - joints[0][1]], [1.0, 0.0]);
        joints[1] = [
            joints[0][0] + lengths[0] * u[0],
            joints[0][1] + lengths[0] * u[1],
        ];
        return;
    }
    // ⚠️ O alvo é amortecido ANTES de iterar: o FABRIK sozinho estica até à recta, e a softness é
    // exactamente a lei que diz que ele não deve chegar lá de repente.
    let dir = [goal[0] - joints[0][0], goal[1] - joints[0][1]];
    let bruto = dir[0].hypot(dir[1]);
    let u = unit(dir, [1.0, 0.0]);
    let d = softened_distance(bruto, total, opts.softness);
    let alvo = [joints[0][0] + d * u[0], joints[0][1] + d * u[1]];

    // ⭐⭐⭐ **FORA DE ALCANCE, A RECTA É EXACTA — e o FABRIK NÃO a encontra.**
    //
    // ⛔ **A doc da família de nós de onde esta lei veio afirma o contrário** (*«out of reach →
    // fully extended toward the goal, which the algorithm produces on its own»*), e a medição
    // derrubou-a: numa corrente de 5 ossos com o alvo a 20× o alcance, a 1.ª junta fica fora de
    // linha **6,07** unidades de 50 com uma passagem, **2,75** com dez e ainda **1,18 com
    // sessenta e quatro** — o FABRIK aproxima-se da recta assimptoticamente, que é exactamente
    // onde ele é mais lento. *O artista vê um braço esticado com uma barriga.*
    //
    // ⇒ quando o alvo efectivo já está no alcance total, a resposta é fechada: deitar a corrente
    // sobre a direcção. ⚠️ Com `softness > 0` isto **nunca** dispara, por construção — a distância
    // amortecida é sempre menor que o alcance, que é a razão de a softness existir.
    if d >= total * (1.0 - TOLERANCE) {
        let mut acc = 0.0;
        for (i, len) in lengths.iter().enumerate() {
            acc += len;
            joints[i + 1] = [joints[0][0] + acc * u[0], joints[0][1] + acc * u[1]];
        }
        return;
    }

    break_collinearity(joints, total, alvo);
    let ancora = joints[0];
    let n = joints.len();
    for _ in 0..opts.iterations.clamp(1, MAX_ITERATIONS) {
        // PARA TRÁS: a ponta toma o alvo e a corrente é arrastada atrás dela.
        joints[n - 1] = alvo;
        for i in (0..n - 1).rev() {
            let v = [
                joints[i][0] - joints[i + 1][0],
                joints[i][1] - joints[i + 1][1],
            ];
            let w = unit(v, [1.0, 0.0]);
            joints[i] = [
                joints[i + 1][0] + lengths[i] * w[0],
                joints[i + 1][1] + lengths[i] * w[1],
            ];
        }
        // PARA A FRENTE: a raiz volta ao sítio onde está pregada, e a corrente vem atrás.
        joints[0] = ancora;
        for i in 1..n {
            let v = [
                joints[i][0] - joints[i - 1][0],
                joints[i][1] - joints[i - 1][1],
            ];
            let w = unit(v, [1.0, 0.0]);
            joints[i] = [
                joints[i - 1][0] + lengths[i - 1] * w[0],
                joints[i - 1][1] + lengths[i - 1] * w[1],
            ];
        }
        let erro = (joints[n - 1][0] - alvo[0]).hypot(joints[n - 1][1] - alvo[1]);
        if erro < TOLERANCE * total {
            break;
        }
    }
}

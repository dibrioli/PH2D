//! **A ROTA DA CORDA** — a geometria pura de uma corda que passa por N roldanas
//! de raio próprio, tangenciando a superfície de cada uma (ADR-0131, W-Pulley
//! W1; plano [`docs/Physics/03_plano_polia.md`] §5).
//!
//! O que a v1 do W-Pulley chamava de "roldana" era um **ponto**: a corda ia do
//! corpo até ele e mudava de direção ali, o que é uma polia de raio zero. O
//! artista pediu diâmetro, e um diâmetro muda três coisas de uma vez — por onde a
//! corda passa (a superfície, não o centro), quanto dela existe (o arco), e o
//! quanto a roldana gira (a mesma corda por um raio maior gira menos).
//!
//! # Uma função responde ao par inteiro
//!
//! Ponto→círculo e círculo→círculo **não são dois casos**: um ponto é um círculo
//! de raio zero, e a fórmula da tangente comum já o contém. Escrever os dois
//! separados seria a segunda resposta que diverge — a representação apaga o caso
//! especial, como a bola limitada do Painter apagou quatro cercas.
//!
//! Sejam duas rodas `(C₁, r₁, s₁)` e `(C₂, r₂, s₂)`, onde `s = ±1` diz **de que
//! lado a corda passa** (`+1` = a corda vira à esquerda ali, e o centro fica à
//! esquerda dela). A tangência exige que os dois centros estejam à distância `r`
//! da reta, cada um do seu lado:
//!
//! ```text
//! D = C₂ − C₁ ,  R = s₂·r₂ − s₁·r₁
//! D = ℓ·u + R·perp(u)            (perp = giro de +90°)
//! ⇒ ℓ = √(|D|² − R²)   e   u = (ℓ·D − R·perp(D)) / |D|²
//! T₁ = C₁ − s₁·r₁·perp(u) ,  T₂ = C₂ − s₂·r₂·perp(u)
//! ```
//!
//! ⚠️ **`|D| > |R|` é a condição de existência**, e ela é o guarda de degeneração
//! honesto: com os lados IGUAIS `R` é a diferença dos raios (uma roda dentro da
//! outra não tem tangente externa); com os lados OPOSTOS `R` é a soma (rodas que
//! se tocam não têm tangente cruzada). Recusar aqui é o que impede um `NaN` de
//! chegar ao `physics_ecs_c9`.
//!
//! # O comprimento inclui o ARCO, e o Jacobiano NÃO
//!
//! `L = Σ|tangentes| + Σ rᵢ·|θᵢ|`, com `θ` o ângulo que a corda vira na roda.
//! Mas a derivada de `L` em relação à âncora de um corpo é **exatamente** o
//! versor daquele último trecho: os pontos de tangência deslizam quando a âncora
//! se move, e a variação do arco **cancela** a variação do trecho (teorema do
//! envelope — o ponto de tangência é estacionário por construção).
//!
//! É por isso que o kernel de impulso quase não muda, e é o mesmo fato que, no
//! W3, dará o Jacobiano de uma roldana MONTADA num corpo — `∂L/∂C = −(u_in +
//! u_out)`, a resultante que também é a carga de ruptura daquele centro. **Uma
//! conta, dois consumidores.**
//!
//! # Determinismo
//!
//! Só `+ − * /`, comparação e `sqrt` — todos exatos no IEEE-754 — mais **UM**
//! transcendental, o `libm::atan2f` do ângulo de arco, pinado cross-OS pela lei 6
//! (o mesmo motivo do `libm::sincosf` do W-AreaFrame). `f32::atan2` viria da libm
//! da plataforma e este número alimenta o hash.

/// Uma roldana na rota: onde ela está, que tamanho tem, e **de que lado a corda
/// passa**.
///
/// `side` é `+1` (a corda vira à esquerda ao passar) ou `−1` (à direita). Ele é
/// **resolvido em autoria** e congelado no play — uma corda real não troca de
/// lado da polia no meio da corrida sem sair da canaleta, e um lado recomputado
/// por frame pisca perto da configuração degenerada, o que muda o comprimento e
/// dá um puxão.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RopeWheel {
    /// O centro, em **mundo**. É isto que uma roldana é: um ponto pregado no
    /// cenário (ou, no W3, num corpo).
    pub centre: [f32; 2],
    /// O raio, em metros. `0` reduz ao modelo de ponto do W-Pulley v1 — e essa
    /// redução é **exata**, o que a torna a âncora de regressão da wave.
    pub radius: f32,
    /// `+1` = a corda vira à ESQUERDA aqui; `−1` = à direita.
    pub side: i8,
    /// **Quem esta roldana É, através das trocas de arena** — o `stable_name_id`
    /// do nome dela (W2).
    ///
    /// A arena é reconstruída por dispatch, então o que uma roldana ACUMULOU (a
    /// carga de pico do eixo) e o fato de ela ter ROMPIDO não podem ser guardados
    /// por posição: acrescentar uma roldana deslocaria os índices e o eixo partido
    /// migraria para a vizinha. Mesma chave, mesmo motivo, que o `id` da corda.
    pub id: u64,
    /// **O que este EIXO aguenta**, newtons — `∞` é uma roldana que não parte, e
    /// é também o que uma roldana que ninguém dimensionou carrega.
    ///
    /// A carga aqui **não é a tensão da corda**: é a RESULTANTE que o desvio
    /// produz (`T·|u_saída − u_entrada|`), então um enlace de 180° carrega `2T` e
    /// um que quase não desvia a corda carrega quase nada. É a mesma conta do
    /// Jacobiano — uma conta, dois consumidores.
    pub break_force: f32,
}

/// O que a rota entrega ao kernel de impulso.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RopeRoute {
    /// Do primeiro ponto de tangência PARA a âncora em A, unitário — a direção
    /// em que o impulso do ramo A age.
    pub dir_a: [f32; 2],
    /// O mesmo na ponta B.
    pub dir_b: [f32; 2],
    /// `Σ|tangentes| + Σ arcos`, em metros.
    pub length: f32,
}

/// Abaixo disto um trecho não tem direção definida e normalizar produziria `NaN`.
/// Mesma constante e mesmo motivo do `MIN_BRANCH` do modelo de ponto.
const MIN_SEG: f32 = 1.0e-4;

/// Os pontos de tangência de um passo da rota, mais a direção do trecho.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Tangent {
    /// Onde a corda LARGA a roda anterior (ou a âncora, se for a primeira).
    pub from: [f32; 2],
    /// Onde ela ENCOSTA na roda seguinte (ou a âncora, se for a última).
    pub to: [f32; 2],
    /// Unitário, de `from` para `to`.
    pub dir: [f32; 2],
    /// `|to − from|`.
    pub len: f32,
}

#[inline]
fn perp(v: [f32; 2]) -> [f32; 2] {
    [-v[1], v[0]]
}

/// **A tangente comum a duas rodas, cada uma pelo lado pedido.**
///
/// Um ponto é uma roda de raio zero (o `side` dele é ignorado pela aritmética,
/// porque ele entra multiplicado pelo raio) — é isso que faz esta ser a única
/// função de geometria da rota.
///
/// `None` quando os círculos estão próximos demais para a tangente existir; ver o
/// cabeçalho do módulo.
#[must_use]
pub fn tangent(c1: [f32; 2], r1: f32, s1: i8, c2: [f32; 2], r2: f32, s2: i8) -> Option<Tangent> {
    let d = [c2[0] - c1[0], c2[1] - c1[1]];
    let dd = d[0] * d[0] + d[1] * d[1];
    let rr = f32::from(s2) * r2 - f32::from(s1) * r1;
    let inner = dd - rr * rr;
    if inner <= 0.0 || dd < MIN_SEG * MIN_SEG {
        return None;
    }
    let len = inner.sqrt();
    if len < MIN_SEG {
        return None;
    }
    let pd = perp(d);
    let dir = [
        (len * d[0] - rr * pd[0]) / dd,
        (len * d[1] - rr * pd[1]) / dd,
    ];
    let pu = perp(dir);
    let from = [
        c1[0] - f32::from(s1) * r1 * pu[0],
        c1[1] - f32::from(s1) * r1 * pu[1],
    ];
    let to = [
        c2[0] - f32::from(s2) * r2 * pu[0],
        c2[1] - f32::from(s2) * r2 * pu[1],
    ];
    Some(Tangent { from, to, dir, len })
}

/// O ângulo com que a corda vira ao passar por uma roda de lado `side`, em
/// radianos e **com sinal**.
///
/// ⚠️ O sinal do `atan2` sozinho não basta: ele devolve o ângulo no intervalo
/// `(−π, π]`, e uma roda com enlace maior que meia volta viraria pelo lado
/// errado. O `side` diz qual dos dois sentidos é o real, então o ângulo é
/// **desdobrado** para aquele sentido — é assim que um enlace de 270° mede 270°
/// e não −90°.
#[must_use]
pub fn turn_angle(u_in: [f32; 2], u_out: [f32; 2], side: i8) -> f32 {
    let cross = u_in[0] * u_out[1] - u_in[1] * u_out[0];
    let dot = u_in[0] * u_out[0] + u_in[1] * u_out[1];
    let mut t = libm::atan2f(cross, dot);
    if side > 0 && t < 0.0 {
        t += std::f32::consts::TAU;
    } else if side < 0 && t > 0.0 {
        t -= std::f32::consts::TAU;
    }
    t
}

/// **Resolver a rota inteira**, escrevendo os `wheels.len() + 1` trechos em
/// `out`.
///
/// `out` é do CHAMADOR e é limpo aqui: o passe de polias roda uma vez por
/// sub-passo, então uma alocação por rota apareceria no gate de zero-alloc do
/// caminho quente.
///
/// `None` quando qualquer trecho é degenerado — a corda inteira é pulada, que é a
/// mesma recusa que o modelo de ponto faz e pela mesma razão: uma rota com um
/// trecho sem direção não tem impulso definido, e meia rota seria pior que
/// nenhuma.
pub fn route(
    anchor_a: [f32; 2],
    anchor_b: [f32; 2],
    wheels: &[RopeWheel],
    out: &mut Vec<Tangent>,
) -> Option<RopeRoute> {
    out.clear();
    let mut prev_c = anchor_a;
    let mut prev_r = 0.0;
    let mut prev_s = 1_i8;
    for w in wheels {
        out.push(tangent(prev_c, prev_r, prev_s, w.centre, w.radius, w.side)?);
        prev_c = w.centre;
        prev_r = w.radius;
        prev_s = w.side;
    }
    out.push(tangent(prev_c, prev_r, prev_s, anchor_b, 0.0, 1)?);

    let mut length = 0.0;
    for t in out.iter() {
        length += t.len;
    }
    // Os arcos: cada roda vive ENTRE dois trechos, e o que ela acrescenta é o
    // pedaço de circunferência que a corda abraça.
    for (i, w) in wheels.iter().enumerate() {
        if w.radius <= 0.0 {
            continue;
        }
        length += w.radius * turn_angle(out[i].dir, out[i + 1].dir, w.side).abs();
    }

    // As duas pontas: o versor aponta do ponto de tangência PARA a âncora, que é
    // a direção em que afastar o corpo estica a corda.
    let first = out[0];
    let last = out[out.len() - 1];
    Some(RopeRoute {
        dir_a: [-first.dir[0], -first.dir[1]],
        dir_b: [last.dir[0], last.dir[1]],
        length,
    })
}

/// **De que lado a corda passa em cada roda** — o (7) do pedido do artista.
///
/// Ponto fixo: chuta pela poligonal dos CENTROS, resolve a rota com esse chute,
/// re-lê o sentido de giro que os trechos de fato fazem, e repete. Converge em
/// uma ou duas rodadas para toda montagem sã (medido em
/// `tests/measure_rope_route.rs`); o cap existe para o caso patológico, onde a
/// resposta é *fique com o último*, nunca *itere para sempre*.
///
/// ⚠️ Roda por AUTORIA, não por frame — ver o cabeçalho do `RopeWheel`.
pub fn resolve_sides(
    anchor_a: [f32; 2],
    anchor_b: [f32; 2],
    wheels: &mut [RopeWheel],
    scratch: &mut Vec<Tangent>,
) {
    if wheels.is_empty() {
        return;
    }
    // O chute: o sentido de giro da poligonal que liga âncora → centros → âncora.
    for i in 0..wheels.len() {
        let prev = if i == 0 {
            anchor_a
        } else {
            wheels[i - 1].centre
        };
        let next = if i + 1 == wheels.len() {
            anchor_b
        } else {
            wheels[i + 1].centre
        };
        let c = wheels[i].centre;
        let a = [c[0] - prev[0], c[1] - prev[1]];
        let b = [next[0] - c[0], next[1] - c[1]];
        wheels[i].side = sign_or(a[0] * b[1] - a[1] * b[0], wheels[i].side);
    }
    for _ in 0..MAX_SIDE_PASSES {
        if route(anchor_a, anchor_b, wheels, scratch).is_none() {
            return;
        }
        let mut changed = false;
        for i in 0..wheels.len() {
            let (u_in, u_out) = (scratch[i].dir, scratch[i + 1].dir);
            let s = sign_or(u_in[0] * u_out[1] - u_in[1] * u_out[0], wheels[i].side);
            if s != wheels[i].side {
                wheels[i].side = s;
                changed = true;
            }
        }
        if !changed {
            return;
        }
    }
}

/// Quantas vezes o ponto fixo do lado pode reavaliar antes de aceitar o que tem.
///
/// **MEDIDO** em `tests/measure_rope_route.rs` — 18 montagens, de 1 a 6 roldanas
/// em zigue-zague, com três espalhamentos:
///
/// | roldanas | 1 | 2 | 3 | 4 | 5 | 6 |
/// |---|---|---|---|---|---|---|
/// | passadas até assentar | 1 | 1 | 1 | 1 | 1 | 1 |
///
/// ⚠️ **UMA passada em todo caso são** — o chute pela poligonal dos centros já
/// É o ponto fixo, e a re-avaliação existe para confirmá-lo. (Eu havia escrito
/// *"1 ou 2"* antes de medir; a medição é mais forte que a estimativa, e a
/// diferença é o tipo de número que ninguém re-mede depois.)
///
/// O cap fica em 4 para o caso patológico — uma montagem que oscile —, e ali
/// *ficar com o último* é a resposta certa: o artista tem o override por roda.
const MAX_SIDE_PASSES: usize = 4;

/// O sinal de `x`, ou `fallback` quando ele é exatamente zero.
///
/// Zero é a corda **colinear** — nem esquerda nem direita —, e ali o lado
/// anterior é a única resposta que não faz a roda pular de um lado ao outro por
/// ruído de `f32`.
#[inline]
fn sign_or(x: f32, fallback: i8) -> i8 {
    if x > 0.0 {
        1
    } else if x < 0.0 {
        -1
    } else {
        fallback
    }
}

#[cfg(test)]
#[path = "rope_route_tests.rs"]
mod tests;

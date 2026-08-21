//! **A pose de um nó**, e a aritmética que a compõe.
//!
//! # Por que a aritmética de quaternion mora AQUI
//!
//! Ela mora onde mora o tipo que a usa. A rotação de um [`Xform`] **é** um quaternion, então
//! multiplicar duas poses é aritmética do documento — e uma segunda cópia dela noutra crate seria
//! duas respostas para *"qual é a pose resultante?"*, com a chance normal de divergirem numa das
//! duas.
//!
//! É o que já estava a acontecer: a câmera do traçador tinha o seu próprio `quat_mul`. Ela passou a
//! ler estas ([`quat_mul`], [`quat_rotate`], …), porque uma composição de rotações não muda de
//! resposta por ser de câmera ou de peça.

use serde::{Deserialize, Serialize};

/// Pose de um nó: translação, rotação e escala **uniforme**.
///
/// ⚠️ **É LOCAL, relativa ao pai** — a pose de mundo obtém-se compondo a cadeia
/// ([`Xform::compose`]). É a lei da casa para o vetorial, dita por extenso: *o que se vê e se
/// aponta é MUNDO; o que o documento guarda é LOCAL.*
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Xform {
    pub translation: [f32; 3],
    /// Quaternion `(x, y, z, w)`.
    pub rotation: [f32; 4],
    /// ⛔ **UNIFORME de propósito.** Escala não-uniforme **destrói a propriedade de distância**
    /// (‖∇f‖ = 1), que é a fundação de tudo neste módulo: sem ela o raio deixa de ser o raio, a
    /// casca perde a espessura e a marcha de raios atravessa a superfície. Quem quer um elipsoide
    /// usa uma primitiva de elipsoide — não uma esfera esticada (ADR-0161 §6).
    pub scale: f32,
}

impl Default for Xform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Xform {
    pub const IDENTITY: Self = Self {
        translation: [0.0; 3],
        rotation: [0.0, 0.0, 0.0, 1.0],
        scale: 1.0,
    };

    #[must_use]
    pub fn at(x: f32, y: f32, z: f32) -> Self {
        Self {
            translation: [x, y, z],
            ..Self::IDENTITY
        }
    }

    /// Leva um ponto do espaço **local** deste nó para o espaço do **pai**: `t + R·(s·p)`.
    ///
    /// ⚠️ É a inversa exacta do que o avaliador faz ao campo (`p' = R⁻¹(p − t)/s`), e tem de o ser:
    /// se as duas contas discordarem, a alça do gizmo pousa num sítio e a superfície aparece noutro.
    /// O gate `the_gizmo_and_the_field_agree_on_where_a_node_is` prende as duas juntas.
    #[must_use]
    pub fn apply(self, p: [f32; 3]) -> [f32; 3] {
        let s = [p[0] * self.scale, p[1] * self.scale, p[2] * self.scale];
        let r = quat_rotate(self.rotation, s);
        [
            r[0] + self.translation[0],
            r[1] + self.translation[1],
            r[2] + self.translation[2],
        ]
    }

    /// Leva uma **direção** (sem translação) do local para o pai. Escala junto: uma seta de gizmo
    /// desenhada num nó escalado 2× tem de medir 2× no mundo, senão ela mente sobre o que arrasta.
    #[must_use]
    pub fn apply_dir(self, d: [f32; 3]) -> [f32; 3] {
        quat_rotate(
            self.rotation,
            [d[0] * self.scale, d[1] * self.scale, d[2] * self.scale],
        )
    }

    /// `self ∘ child` — a pose de `child` expressa no referencial do **pai de `self`**.
    ///
    /// ⭐ É a única porta para descer a hierarquia. `translation` compõe por [`Xform::apply`],
    /// `rotation` por [`quat_mul`], `scale` por produto — e é essa terceira que costuma faltar:
    /// sem ela um filho dentro de um pai escalado fica no sítio certo com o tamanho errado.
    #[must_use]
    pub fn compose(self, child: Self) -> Self {
        Self {
            translation: self.apply(child.translation),
            rotation: quat_normalize(quat_mul(self.rotation, child.rotation)),
            scale: self.scale * child.scale,
        }
    }
}

/// O **inverso** de uma rotação: o conjugado, que num quaternion unitário é a inversa exata.
#[must_use]
pub fn quat_conj(q: [f32; 4]) -> [f32; 4] {
    [-q[0], -q[1], -q[2], q[3]]
}

/// `a ⊗ b` — aplicar `b` **depois** de `a`, no referencial de `a`.
#[must_use]
pub fn quat_mul(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    let ([ax, ay, az, aw], [bx, by, bz, bw]) = (a, b);
    [
        aw * bx + ax * bw + ay * bz - az * by,
        aw * by - ax * bz + ay * bw + az * bx,
        aw * bz + ax * by - ay * bx + az * bw,
        aw * bw - ax * bx - ay * by - az * bz,
    ]
}

/// O quaternion de uma rotação de `angle` em torno de `axis`. Eixo degenerado ⇒ identidade, que é
/// a única resposta honesta (rodar em torno de nada é não rodar).
#[must_use]
pub fn quat_axis_angle(axis: [f32; 3], angle: f32) -> [f32; 4] {
    let len = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
    if len <= 0.0 || !len.is_finite() {
        return [0.0, 0.0, 0.0, 1.0];
    }
    let (s, c) = (angle * 0.5).sin_cos();
    [axis[0] / len * s, axis[1] / len * s, axis[2] / len * s, c]
}

/// ⚠️ **Re-normalizar a cada composição não é zelo.** Uma orientação acumulada são centenas de
/// multiplicações, e o erro de `f32` faz a norma derivar. Um quaternion que deixa de ser unitário
/// deixa de ser uma rotação — ele passa a **escalar**, e o sintoma é a forma a encolher devagar.
#[must_use]
pub fn quat_normalize(q: [f32; 4]) -> [f32; 4] {
    let n = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
    if n <= 0.0 || !n.is_finite() {
        return [0.0, 0.0, 0.0, 1.0];
    }
    [q[0] / n, q[1] / n, q[2] / n, q[3] / n]
}

/// Roda um vetor: `v + 2·w·(u×v) + 2·u×(u×v)`, com `u` a parte vetorial — a forma sem construir a
/// matriz.
#[must_use]
pub fn quat_rotate(q: [f32; 4], v: [f32; 3]) -> [f32; 3] {
    let u = [q[0], q[1], q[2]];
    let t = cross(u, v);
    let tt = cross(u, t);
    [
        v[0] + 2.0 * (q[3] * t[0] + tt[0]),
        v[1] + 2.0 * (q[3] * t[1] + tt[1]),
        v[2] + 2.0 * (q[3] * t[2] + tt[2]),
    ]
}

/// ⭐ **A rotação como três ângulos X/Y/Z**, em radianos — na forma **canónica**.
///
/// # A ordem, e de onde ela vem
///
/// `R = Rz(γ) · Ry(β) · Rx(α)`: roda-se primeiro em torno do X, depois do Y, depois do Z. É o
/// «XYZ Euler» do Blender, que é a referência declarada da casa para as palavras desta UI — quem já
/// modela lê os três números sem os ter de experimentar.
///
/// # ⚠️ O TRIO não é único; a ORIENTAÇÃO é
///
/// Uma orientação tem infinitos trios que a nomeiam (somar uma volta a qualquer um deles; e, nos
/// polos, uma família inteira). Esta função devolve o **canónico**: `β ∈ [−90°, 90°]`, os outros
/// dois em `(−180°, 180°]`. O que fecha o ciclo é
/// `quat_from_euler(quat_to_euler(q)) == q` **como rotação** — nunca a igualdade do trio, e o gate
/// `the_orientation_round_trips_even_though_the_triple_need_not` diz isso por extenso.
///
/// # ⚠️ Trava de cardan
///
/// Em `β = ±90°` o X e o Z deixam de ser distinguíveis: só a soma (ou a diferença) deles é um facto
/// do mundo. Ali esta função põe `γ = 0` e dá o resto ao X — uma escolha **determinística**, e não
/// um `NaN` nem um trio que muda de quadro para quadro.
#[must_use]
pub fn quat_to_euler(q: [f32; 4]) -> [f32; 3] {
    let [x, y, z, w] = quat_normalize(q);
    // Só as entradas da matriz que a extração usa — construir as nove seria trabalho para deitar
    // fora, e cada uma a mais é uma a poder estar errada.
    let r00 = 1.0 - 2.0 * (y * y + z * z);
    let r10 = 2.0 * (x * y + w * z);
    let r20 = 2.0 * (x * z - w * y);
    let r21 = 2.0 * (y * z + w * x);
    let r22 = 1.0 - 2.0 * (x * x + y * y);
    // ⚠️ `hypot(r21, r22)` **é** `|cos β|`, e é por aí que o `β` sai robusto: um `asin(−r20)` com o
    // argumento a passar de 1 por um ULP devolveria `NaN` exatamente no caso que mais interessa.
    let cos_beta = r21.hypot(r22);
    let beta = (-r20).atan2(cos_beta);
    if cos_beta > EULER_LOCK_EPS {
        [r21.atan2(r22), beta, r10.atan2(r00)]
    } else {
        // Trava: `γ = 0` e o X fica com a combinação que **é** um facto.
        let r01 = 2.0 * (x * y - w * z);
        let r02 = 2.0 * (x * z + w * y);
        let alpha = if r20 < 0.0 {
            r01.atan2(r02)
        } else {
            (-r01).atan2(-r02)
        };
        [alpha, beta, 0.0]
    }
}

/// ⭐ **O quaternion de três ângulos X/Y/Z**, em radianos — a inversa de [`quat_to_euler`].
#[must_use]
pub fn quat_from_euler(e: [f32; 3]) -> [f32; 4] {
    let qx = quat_axis_angle([1.0, 0.0, 0.0], e[0]);
    let qy = quat_axis_angle([0.0, 1.0, 0.0], e[1]);
    let qz = quat_axis_angle([0.0, 0.0, 1.0], e[2]);
    quat_normalize(quat_mul(quat_mul(qz, qy), qx))
}

/// **Abaixo deste `|cos β|` a divisão entre X e Z é RUÍDO**, e a extração passa ao ramo da trava.
///
/// ⚠️ O número é derivado de um recurso, e o recurso é a **precisão de `f32`**: as entradas da
/// matriz saem de produtos de componentes unitárias, com erro absoluto de alguns ULP de 1,0 (~1e-7).
/// Perto do polo `r21` e `r22` valem ambas ~`cos β`, então o `atan2` deles carrega ~`1e-7 / cos β`
/// radianos de erro. Em `cos β = 1e-4` isso é ~1e-3 rad (0,06°) — abaixo do que qualquer casa
/// decimal deste painel mostra. Entrar no ramo da trava a essa altura custa, do outro lado, um
/// desvio da mesma ordem (~1e-4 rad) na orientação reconstruída: as duas pontas estão medidas, e é
/// por isso que a tolerância do gate de ida-e-volta é `2e-4` e não `f32::EPSILON`.
const EULER_LOCK_EPS: f32 = 1.0e-4;

/// ⭐ **A rotação de uma pose em GRAUS** — os três números que o painel mostra.
///
/// ⚠️ Graus e não radianos, e a conversão mora **aqui**, num sítio: ninguém escreve 1,5708 num campo
/// para pôr uma peça de pé.
#[must_use]
pub fn rotation_degrees(pose: Xform) -> [f32; 3] {
    quat_to_euler(pose.rotation).map(f32::to_degrees)
}

/// ⭐ **Escreve UM dos três ângulos**, em graus, deixando os outros dois onde estão.
///
/// # ⚠️ A lei inteira: escrever o MESMO valor duas vezes não pode mexer a peça
///
/// Um arrasto escreve o alvo **quadro após quadro**. Uma escrita que não seja **ponto fixo** vira um
/// ciclo de dois: a peça alterna entre duas orientações com o dedo parado.
///
/// Foi exatamente o que a primeira versão fazia, e o Enio viu-o (20/08: *"bug em rot y. Acima de 70
/// muda x e z e treme"*). Ela escrevia o alvo cru no trio e deixava a leitura seguinte
/// **renomear**; a escrita seguinte partia então de um trio **diferente** e produzia outra
/// orientação. Medido: `Y = 93,6` dava `(180, 86,4, 180)` e, repetido, `(0, 86,4, 0)`.
///
/// ⭐ **A cura é fazer o alvo entrar já CANÓNICO**, porque é a única forma de a leitura seguinte o
/// devolver intacto:
///
/// | eixo | faixa canónica | o que se faz a um alvo fora dela |
/// |---|---|---|
/// | X, Z | `(−180°, 180°]` | **enrola** — 200° é o mesmo sítio que −160° |
/// | Y (o do meio) | `[−90°, 90°]` | **prende** — ver abaixo |
///
/// ⚠️ **Prender o eixo do meio não perde orientação nenhuma**: toda orientação tem um trio canónico
/// com `|β| ≤ 90°`. O que se perde é o **nome** — «Y = 120» deixa de ser digitável, e o mesmo sítio
/// escreve-se `X = 180 · Y = 60 · Z = 180`. É a diferença face ao Blender, e ela é o preço, já
/// pago e medido, de não guardar o trio (ver abaixo).
///
/// # ⚠️ Na trava de cardan o Z é INERTE, e tem de ser
///
/// Em `β = ±90°` o X e o Z são o **mesmo** eixo: só a soma (ou a diferença) deles é um facto, e a
/// forma canónica dá tudo ao X. Um Z escrito ali não tem onde ficar — aplicá-lo faria o X escorregar
/// mais um tanto a **cada quadro** do arrasto, que é o mesmo ciclo por outro caminho. Aqui ele é
/// **recusado**, e a recusa é visível: o número volta ao 0 que a linha já mostrava. Sair da trava é
/// mexer o Y, que continua a responder.
///
/// ⛔ **A alternativa foi pesada e recusada**: guardar o trio autorado ao lado do quaternion. O gizmo
/// roda em torno de eixos **arbitrários** (a argola da vista não é X, Y nem Z) e escreve o
/// quaternion — logo o trio guardado seria um cache invalidado por **todo** arrasto. E o preço maior
/// é o undo: ele compara **bytes**, e duas poses com a mesma orientação e trios diferentes seriam
/// snapshots diferentes — todo quadro viraria um passo espúrio, que é a doença que o
/// `canonicalize()` do shell já pagou uma vez.
///
/// Um eixo fora do alcance (`>= 3`) é no-op silencioso: não há quarto ângulo a escrever.
pub fn set_rotation_degree(pose: &mut Xform, axis: u8, degrees: f32) {
    if axis >= 3 || !degrees.is_finite() {
        return;
    }
    // ⚠️ **A MESMA porta que o painel usa para saber se pinta um controle** — ver
    // [`rotation_axis_is_free`]. Duas respostas para *"este eixo responde?"* seria um controle vivo
    // sobre uma escrita recusada, e o gate `the_inert_axis_is_exactly_the_one_the_write_refuses`
    // prende as duas.
    if !rotation_axis_is_free(*pose, axis) {
        return;
    }
    let mut e = quat_to_euler(pose.rotation);
    let target = degrees.to_radians();
    e[axis as usize] = if axis == 1 {
        target.clamp(-QUARTER_TURN, QUARTER_TURN)
    } else {
        wrap_half_turn(target)
    };
    pose.rotation = quat_from_euler(e);
}

/// ⭐ **Este eixo do trio responde AGORA?** — a porta única de *"há aqui um número a mexer"*.
///
/// ⚠️ Ela existe para o painel e a escrita darem **a mesma** resposta. Um controle vivo sobre uma
/// escrita recusada é a affordance que mente — e este módulo já paga caro por cada par de portas que
/// discorda sobre a mesma coisa.
///
/// # ⚠️ Na trava de cardan o terceiro ângulo não é um eixo
///
/// Em `β = ±90°` o X e o Z rodam em torno do **mesmo** eixo físico: só a soma (ou a diferença) deles
/// é um facto do mundo, e a forma canónica dá tudo ao X. O Z ali não é um número pequeno nem um
/// número difícil — ele **não existe** como grandeza independente.
///
/// ⛔ **As três alternativas foram pesadas e não sobrou nenhuma sem memória:**
///
/// | | por que não |
/// |---|---|
/// | aplicar o Z na mesma, encaminhado para o X | não é ponto fixo: o X escorrega mais um tanto a cada quadro do arrasto — é o ciclo que o Enio viu, por outro caminho |
/// | reinterpretar o Z como o parâmetro livre | é idempotente **e destrutivo**: escrever o Z deitaria fora o X que lá estava |
/// | impedir o Y de chegar a 90 | `90` é o ângulo que mais se digita — pô-lo fora de alcance é pior do que a trava |
///
/// Sobra guardar o trio autorado (o que o Blender faz), e isso é a segunda verdade que
/// [`set_rotation_degree`] recusa, com o preço escrito lá.
#[must_use]
pub fn rotation_axis_is_free(pose: Xform, axis: u8) -> bool {
    if axis >= 3 {
        return false;
    }
    // O X e o Y respondem sempre — é pelo Y, aliás, que se sai da trava.
    axis != 2 || quat_to_euler(pose.rotation)[1].cos().abs() > EULER_LOCK_EPS
}

/// Um quarto de volta, em radianos — a ponta da faixa canónica do eixo do **meio**.
pub const QUARTER_TURN: f32 = std::f32::consts::FRAC_PI_2;

/// Meia volta, em radianos — a ponta da faixa canónica dos eixos de **fora**.
pub const HALF_TURN: f32 = std::f32::consts::PI;

/// Traz um ângulo para `(−π, π]`, que é o que o `atan2` devolve e portanto o que a leitura mostra.
///
/// ⚠️ **Enrolar não é prender.** 200° e −160° são o mesmo sítio, e recusar o primeiro seria recusar
/// uma orientação que existe; prendê-lo em 180° seria pô-la noutro sítio. Só o eixo do meio se
/// prende, e por outra razão (a faixa dele é `[−90°, 90°]` na própria representação).
#[must_use]
pub fn wrap_half_turn(radians: f32) -> f32 {
    let turn = 2.0 * HALF_TURN;
    let mut a = radians % turn;
    if a > HALF_TURN {
        a -= turn;
    } else if a <= -HALF_TURN {
        a += turn;
    }
    a
}

#[must_use]
pub fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[must_use]
pub fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0].mul_add(b[0], a[1].mul_add(b[1], a[2] * b[2]))
}

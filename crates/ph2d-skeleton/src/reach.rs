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

/// ⭐⭐⭐ **DE QUE LADO A CORRENTE DOBRA** — e por que ele é um dado AUTORADO e não uma dedução.
///
/// Sem isto o lado sai da pose que a corrente tem no instante em que se resolve, e há uma pose em
/// que ela **não tem lado nenhum**: a recta. ⛔ Medido (2026-09-07,
/// `the_elbow_flips_when_the_chain_passes_through_straight`): um cotovelo do lado `-99,498744`
/// estica até ficar colinear (`0,000000`) e, ao voltar **ao mesmo alvo**, vem do lado
/// `+99,498744` — a mesma magnitude, o sinal trocado. *O joelho inverteu sozinho, e o artista não
/// fez nada.* É determinístico, não é ruído: quem decide é o desempate da corrente recta.
///
/// # As referências, e por que a nossa resposta é um INTERRUPTOR e não um alvo
///
/// | ferramenta | dimensão | o que ela oferece |
/// |---|---|---|
/// | **Godot** (`SkeletonModification2DTwoBoneIK`, MIT) | 2D | **`flip_bend_direction: bool`** |
/// | **Spine** (`IkConstraint`) | 2D | `bendDirection` `+1`/`−1`, animável |
/// | **Blender** (`Inverse Kinematics`) | 3D | *Pole Target* (um objecto) + *Pole Angle* |
/// | **Maya** (`ikRPsolver`) | 3D | `poleVector` + twist |
///
/// ⭐⭐ **O *pole target* responde à pergunta do 3D, que aqui não existe.** Em três dimensões o
/// triângulo raiz–cotovelo–ponta pode **rodar em torno** do eixo raiz→ponta, e é esse grau de
/// liberdade contínuo que um objecto no espaço fixa. No plano ele não existe: sobra **um bit**, de
/// que lado da recta o cotovelo cai. Um alvo arrastável que codifica um bit dá ao artista a ilusão
/// de um controlo contínuo e depois **salta** quando ele cruza a recta — e é por isso que as duas
/// referências 2D, independentes uma da outra, escolheram a mesma forma.
///
/// ⚠️ O Godot volta a oferecer um ponto (`magnet_position`) na modificação **FABRIK**, e isso não
/// contradiz o de cima: ali não há forma fechada, então o ímã é uma heurística de arranque — não a
/// resposta exacta que a lei dos cossenos dá com um sinal.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum BendSide {
    /// **Preserva** a dobra que a pose já tem — e, quando ela não tem nenhuma, o desempate da
    /// corrente recta. É o que um **gesto** quer (o artista arrasta a ponta e o cotovelo fica onde
    /// ele o deixou) e é o comportamento de sempre, **ao bit**.
    #[default]
    Keep,
    /// O cotovelo fica do lado **anti-horário** da recta `raiz → alvo` ([`side_of`] positivo).
    Ccw,
    /// ... do lado **horário** ([`side_of`] negativo).
    ///
    /// ⚠️ É este o lado para que as duas leis desempatam hoje quando a corrente está recta —
    /// **medido**, não deduzido (`n=2` `−7,416198` · `n=3` `−4,472136` · `n=5` `−4,486331`).
    Cw,
}

impl BendSide {
    /// Os três, na ordem em que um selector os oferece. ⚠️ Quem pinta um segmento por variante
    /// alinha-se por ÍNDICE com esta lista — é o que impede a fileira do painel e o vocabulário de
    /// divergirem em silêncio (o padrão do `BoneAction::ALL`).
    pub const ALL: [Self; 3] = [Self::Keep, Self::Ccw, Self::Cw];

    /// O sinal que este lado impõe a [`side_of`], ou `None` quando ele não impõe nenhum.
    #[must_use]
    pub const fn forced(self) -> Option<f64> {
        match self {
            Self::Keep => None,
            Self::Ccw => Some(1.0),
            Self::Cw => Some(-1.0),
        }
    }

    /// O oposto — `Keep` não tem, e devolve-se a si mesmo.
    #[must_use]
    pub const fn flipped(self) -> Self {
        match self {
            Self::Keep => Self::Keep,
            Self::Ccw => Self::Cw,
            Self::Cw => Self::Ccw,
        }
    }
}

/// ⭐⭐⭐ **A GRANDEZA CANÓNICA DO LADO** — o desvio de `p` em relação à recta `raiz → alvo`,
/// positivo no sentido anti-horário.
///
/// ⚠️⚠️ **Ela existe porque as duas leis desta crate mediam o lado com grandezas de SINAL
/// OPOSTO**, e nada escrito o dizia: o ramo de dois ossos lê `v1 × v2` (que a álgebra mostra valer
/// `−l1·d·sin a`) e o arqueamento de 3+ lê o desvio perpendicular (`+sin a`). Duas réguas do mesmo
/// facto, uma o negativo da outra, é como um sinal trocado sobrevive a uma revisão — e este módulo
/// já pagou exactamente isso uma vez, com a inversão que vivia no `two_bone` desde que ele existe.
///
/// ⇒ **uma porta.** Quem quiser saber de que lado a corrente está pergunta aqui, e quem quiser
/// impor um lado escreve o sinal desta.
#[must_use]
pub fn side_of(root: [f64; 2], goal: [f64; 2], p: [f64; 2]) -> f64 {
    let u = unit([goal[0] - root[0], goal[1] - root[1]], [1.0, 0.0]);
    let v = [p[0] - root[0], p[1] - root[1]];
    u[0] * v[1] - u[1] * v[0]
}

/// Quantas passagens do FABRIK, quanto se amortece a extensão máxima, e de que lado se dobra.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Reach {
    /// Passagens do FABRIK. Só conta numa corrente de 3+ ossos — a de 2 é fechada.
    pub iterations: usize,
    /// A **SOFTNESS** do Spine, em unidades de MUNDO: a que distância do alcance máximo os ossos
    /// começam a abrandar. `0` ⇒ o corte a seco de sempre.
    pub softness: f64,
    /// De que lado a corrente dobra. [`BendSide::Keep`] ⇒ o comportamento de sempre, **ao bit** —
    /// e é o valor por omissão de propósito, porque o **gesto** de arrastar a ponta quer preservar
    /// a dobra que o artista vê. Quem trava o lado é a **restrição**, que persiste.
    pub bend: BendSide,
}

impl Default for Reach {
    fn default() -> Self {
        Self {
            iterations: DEFAULT_ITERATIONS,
            softness: 0.0,
            bend: BendSide::Keep,
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

/// **Quão recta é "recta"** — o seno do ângulo entre dois ossos abaixo do qual eles não têm lado.
///
/// ⚠️ **Mesmo valor do [`BOW`] e constante PRÓPRIA, de propósito.** Os dois nascem do mesmo facto
/// (uma corrente colinear não tem para onde cair) mas medem grandezas diferentes — aquele é uma
/// fracção do ALCANCE e este o seno de um ÂNGULO. Partilhar o literal e não o nome é o que impede
/// que afinar um mude o outro em silêncio, que é o defeito que este módulo já pagou com o raio da
/// ponta.
///
/// ⭐ E ele cobre com folga o ruído medido: um `Transform` em `f32` erra a posição em `~1e-6`, e a
/// raiz quadrada da lei dos cossenos amplifica isso para `~5e-4` de ângulo (medido no braço da cena
/// de smoke). `1e-3` são `0,057°` — invisíveis, e acima do ruído.
const STRAIGHT: f64 = BOW;

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
    side: BendSide,
) -> [[f64; 2]; 2] {
    let to_goal = [goal[0] - root[0], goal[1] - root[1]];
    let u = unit(to_goal, [1.0, 0.0]);
    let bruto = to_goal[0].hypot(to_goal[1]);
    // ⭐⭐⭐ **UMA CORRENTE RECTA NÃO TEM LADO, e ler o sinal do resíduo faz o joelho VIBRAR.**
    //
    // ⚠️ Isto é um defeito MEDIDO (2026-09-07, `probe_a_still_chain_keeps_writing`): o braço da cena
    // de smoke, **parado**, escrevia em `300` de `300` quadros e o punho oscilava `±5e-4 rad` com a
    // amplitude a **crescer**. O mecanismo é uma raiz quadrada: perto da extensão máxima
    // `cos a → 1`, e `sin a = √(1 − cos²a)` transforma um erro de POSIÇÃO de `1e-6` (o que o
    // `Transform` da casa perde por ser `f32`) num erro de ÂNGULO de `1e-3` — mil vezes maior. O
    // sinal desse ângulo vem de `bend`, que numa corrente recta é o produto vectorial de dois
    // vectores paralelos: **ruído**. Cada quadro sorteava um lado.
    //
    // ⇒ o desempate é DETERMINÍSTICO, e é o mesmo lado para que o [`break_collinearity`] arqueia a
    // corrente de 3+ ossos: *não se escolhe um desempate melhor, não se tem empate.*
    //
    // ⭐ E é aqui que um lado AUTORADO entra: ele substitui a leitura da pose em vez de a corrigir,
    // porque o defeito que ele cura é precisamente a pose deixar de ter lado.
    let sinal = match side.forced() {
        Some(s) => s,
        None => {
            let bend = if bend.abs() <= STRAIGHT * l1 * l2 {
                1.0
            } else {
                bend
            };
            if bend > 0.0 { -1.0 } else { 1.0 }
        }
    };
    // Longe demais ⇒ amortecido; perto demais (a corrente dobrada sobre si) ⇒ o piso é o que os
    // dois ossos conseguem encolher.
    let d = softened_distance(bruto, l1 + l2, soft).max((l1 - l2).abs().max(f64::EPSILON));
    // ⭐⭐⭐ **NA EXTENSÃO MÁXIMA A RESPOSTA É A RECTA, EXACTAMENTE** — e é a MESMA lei que o ramo de
    // 3+ ossos já tem trinta linhas abaixo, em falta aqui.
    //
    // ⚠️ Lá ela existe porque o FABRIK se aproxima da recta **assimptoticamente**; aqui porque a
    // forma fechada, embora exacta, é **mal condicionada** ali: `cos a → 1`, e `sin a = √(1 − cos²a)`
    // transforma um erro de posição de `1e-6` — o que o `Transform` da casa perde por ser `f32` —
    // num erro de ângulo de `1e-3`. *A derivada de uma raiz quadrada na origem é infinita.*
    //
    // ⛔ **Medido, e o sintoma é visível:** sem esta linha o braço da cena de smoke, **parado**,
    // reescrevia a pose em `300` de `300` quadros e o punho oscilava `±5e-4 rad` com a amplitude a
    // crescer. Uma corrente esticada que treme é a queixa que o artista faz.
    //
    // ⚠️ Com `softness > 0` isto **nunca** dispara, por construção — a distância amortecida é sempre
    // menor que o alcance, que é a razão de a suavidade existir.
    if d >= (l1 + l2) * (1.0 - TOLERANCE) {
        return [
            [root[0] + l1 * u[0], root[1] + l1 * u[1]],
            [root[0] + (l1 + l2) * u[0], root[1] + (l1 + l2) * u[1]],
        ];
    }
    // Higiene de vírgula flutuante: na extensão máxima o quociente é exactamente `1` em aritmética
    // real e `1 + 1e-16` nesta — e a raiz de um negativo devolveria um membro `NaN`.
    let cos_a = ((l1 * l1 + d * d - l2 * l2) / (2.0 * l1 * d)).clamp(-1.0, 1.0);
    // ⭐⭐⭐ **O SINAL É O OPOSTO do produto vectorial, e a álgebra prova-o.**
    //
    // Com `b = raiz + l1·osso1` e `c = raiz + d·u`, o produto vectorial das duas metades é
    // `v1 × v2 = l1·d·(osso1 × u)`, e `osso1 × u = −sin a` (basta expandir a rotação de `u` por
    // `a`). ⇒ `cross = −l1·d·sin a`: **eles têm sinal contrário**.
    //
    // ⛔ **Isto estava ao contrário desde que a lei existe** (medido 2026-09-07,
    // `a_straight_chain_never_draws_lots_for_the_bend_side`): igualar os dois sinais faz cada
    // passagem **negar** o `sin a` da anterior, e o cotovelo salta de lado a cada resolução. O doc
    // desta função promete *«o solver preserva a dobra que o artista já vê»* e ela fazia o
    // contrário — invisível enquanto a cinemática inversa era só um arrasto (ali cada passagem tem
    // um alvo novo e a troca lê-se como tremor), e **impossível de ignorar** com uma restrição que
    // re-resolve todo quadro.
    // ⚠️ `sinal` é o sinal de [`side_of`] que se quer (a álgebra acima mostra `u × osso1 = sin a`),
    // logo ele entra em `sin_a` **directamente** — sem o `−1` que a leitura por `v1 × v2` exigia.
    let sin_a = (1.0 - cos_a * cos_a).sqrt() * sinal;
    let osso1 = [u[0] * cos_a - u[1] * sin_a, u[0] * sin_a + u[1] * cos_a];
    [
        [root[0] + l1 * osso1[0], root[1] + l1 * osso1[1]],
        [root[0] + d * u[0], root[1] + d * u[1]],
    ]
}

/// **De que lado a corrente está**, com o sinal de [`side_of`]: o desvio DOMINANTE das juntas
/// interiores. `0` ⇒ colinear, e uma corrente colinear não tem lado.
///
/// ⚠️ O **dominante** e não a soma: uma corrente em S tem desvios de sinais opostos que se anulam,
/// e uma soma nula leria «recta» sobre uma pose que é tudo menos recta.
#[must_use]
pub fn dominant_side(p: &[[f64; 2]], goal: [f64; 2]) -> f64 {
    let mut pior = 0.0f64;
    for q in &p[1..] {
        let d = side_of(p[0], goal, *q);
        if d.abs() > pior.abs() {
            pior = d;
        }
    }
    pior
}

/// ⭐⭐⭐ **DE QUE LADO ESTA CORRENTE ESTÁ** — incluindo a resposta *«de nenhum»*.
///
/// É a porta que quem **captura** um lado usa (o verbo que cria uma âncora grava a dobra que o
/// artista já posou). ⚠️ Ela devolve [`BendSide::Keep`] para uma corrente **recta**, e isso é a
/// resposta certa e não uma desistência: ali o desvio é ruído de `f32` amplificado, e escolher um
/// lado a partir dele seria inventar uma decisão do artista.
///
/// `reach` é o alcance da corrente (a soma dos comprimentos) — a barra da rectidão é uma **fracção**
/// dele, a mesma que o [`BOW`] usa, para a lei ser adimensional.
///
/// ⛔ Ela existe para o chamador não replicar a conta: uma segunda barra para *«isto é recto?»*
/// diverge da primeira no dia em que alguém afinar uma das duas.
#[must_use]
pub fn bend_side_of(joints: &[[f64; 2]], goal: [f64; 2], reach: f64) -> BendSide {
    let desvio = dominant_side(joints, goal);
    if !desvio.is_finite() || desvio.abs() <= BOW * reach {
        return BendSide::Keep;
    }
    if desvio > 0.0 { BendSide::Ccw } else { BendSide::Cw }
}

/// Arqueia uma corrente RECTA para o FABRIK ter um lado para onde cair — para o lado PEDIDO, se
/// houver um.
fn break_collinearity(p: &mut [[f64; 2]], reach: f64, goal: [f64; 2], side: BendSide) {
    let n = p.len();
    let to_goal = [goal[0] - p[0][0], goal[1] - p[0][1]];
    let d = to_goal[0].hypot(to_goal[1]);
    if reach - d < BOW * reach {
        return; // o alvo está na extensão máxima (ou além): a recta É a resposta
    }
    if dominant_side(p, goal).abs() > BOW * reach {
        return; // já está fora da recta — a iteração tem para onde cair
    }
    let u = unit(to_goal, [1.0, 0.0]);
    // `u × perp = +1`, então `+perp` é o lado anti-horário de [`side_of`].
    let perp = [-u[1], u[0]];
    let arco = BOW * reach * side.forced().unwrap_or(1.0);
    for q in p.iter_mut().take(n - 1).skip(1) {
        q[0] += perp[0] * arco;
        q[1] += perp[1] * arco;
    }
}

/// ⭐⭐⭐ **ESPELHA a corrente para o lado pedido** — a metade que o arqueamento não faz.
///
/// ⚠️ **Sem isto o lado autorado não morde numa corrente de 3+ ossos**, e a razão é que o
/// [`break_collinearity`] só age sobre uma pose **recta**: com a corrente já dobrada para o lado
/// errado ele devolve cedo, e o FABRIK parte da pose que encontra — ele **preserva** o lado, que é
/// exactamente o que aqui se quer mudar.
///
/// ⭐ A operação é uma **reflexão sobre a recta `raiz → alvo`**, e por ser uma isometria ela não
/// toca em nenhum comprimento — o invariante das duas leis desta crate sobrevive por construção,
/// não por uma guarda escrita à mão. A raiz fica onde está porque a recta passa por ela.
fn mirror_to_side(p: &mut [[f64; 2]], goal: [f64; 2], side: BendSide) {
    let Some(quero) = side.forced() else {
        return;
    };
    let actual = dominant_side(p, goal);
    if actual == 0.0 || actual.signum() == quero.signum() {
        return;
    }
    let root = p[0];
    let u = unit([goal[0] - root[0], goal[1] - root[1]], [1.0, 0.0]);
    for q in &mut p[1..] {
        let v = [q[0] - root[0], q[1] - root[1]];
        let ao_longo = v[0] * u[0] + v[1] * u[1];
        // `q' = raiz + 2·(v·u)·u − v` — a reflexão de `v` sobre a direcção `u`.
        q[0] = root[0] + 2.0 * ao_longo * u[0] - v[0];
        q[1] = root[1] + 2.0 * ao_longo * u[1] - v[1];
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
        let [cotovelo, mao] = two_bone(
            a,
            lengths[0],
            lengths[1],
            goal,
            bend,
            opts.softness,
            opts.bend,
        );
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

    break_collinearity(joints, total, alvo, opts.bend);
    // ⚠️ **Depois** do arqueamento, não antes: sobre uma corrente recta o espelho não tem o que
    // espelhar (o desvio dominante é zero e ele devolve cedo), então quem lhe dá um lado para
    // corrigir é o arqueamento — e quando a pose já vem torta é o espelho que manda.
    mirror_to_side(joints, alvo, opts.bend);
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

/// ⭐⭐⭐ **A MISTURA** — o *Mix* do Spine, o *Influence* do Blender, o *Strength* do Rive.
///
/// Uma restrição de cinemática inversa que só sabe ligar e desligar não serve para animar: o que o
/// artista quer é **quanto** dela, e quer poder animar esse número (a mão que larga a corrimão).
/// As quatro referências têm-no, e nós tínhamos **zero**.
///
/// ⭐⭐ **Mistura-se o ÂNGULO, nunca a POSIÇÃO**, e a razão é exactidão: interpolar as posições das
/// juntas encurta os ossos (a corda de um arco é mais curta que o arco), e o comprimento de um osso
/// é invariante das duas leis desta crate — há gate a dizê-lo. Misturar o ângulo preserva-o **ao
/// bit**, porque o comprimento nem entra na conta.
///
/// ⚠️ **Pelo caminho CURTO.** Sem o embrulho, um ombro a `+179°` e um alvo a `−179°` (dois graus de
/// distância) fariam a mistura percorrer **358°** ao contrário: o braço dá uma volta completa a meio
/// de uma animação. É o defeito clássico da interpolação de ângulos, e a cura é uma linha.
///
/// `mix <= 0` devolve `de` **ao bit** (a restrição desligada é o no-op que a lei da casa exige);
/// `mix >= 1` devolve `para` ao bit.
#[must_use]
pub fn blend_angle(de: f64, para: f64, mix: f64) -> f64 {
    // ⚠️ O `is_nan` é EXPLÍCITO, e não um `!(mix > 0.0)` a apanhá-lo de lado: uma mistura que não é
    // um número não move nada, e essa decisão tem de se ler no código em vez de sair da forma como
    // a comparação trata `NaN`.
    if mix.is_nan() || mix <= 0.0 {
        return de;
    }
    if mix >= 1.0 {
        return para;
    }
    de + wrap_pi(para - de) * mix
}

/// Traz um ângulo para `(-π, π]` — a diferença mais curta entre duas direcções.
#[must_use]
pub fn wrap_pi(a: f64) -> f64 {
    use std::f64::consts::{PI, TAU};
    let mut r = (a + PI).rem_euclid(TAU) - PI;
    // `rem_euclid` devolve `[0, TAU)`, então o extremo superior cai em `-π` e não em `+π`. A
    // diferença é invisível no produto e não no gate, que compara com o valor exacto.
    if r <= -PI {
        r += TAU;
    }
    r
}

/// ⭐⭐⭐ **O LIMITE DE UMA JUNTA** — até onde ela dobra, e a partir de onde deixa de dobrar.
///
/// É o *IK Limits* do Blender, o *Angle constraints* do Moho e o
/// `ccdik_joint_constraint_angle_min`/`_max` do Godot: sem ele o cotovelo dobra para trás e o
/// joelho hiperextende, e um rig que faz isso não se lê como um corpo.
///
/// # ⚠️ Ela é CIRCULAR, e um `clamp` cru estaria errado
///
/// Ângulos vivem num círculo, então `rot.clamp(min, max)` falha exactamente onde o intervalo
/// atravessa `±π`: com `min = 170°` e `max = −170°` (uma faixa de 20° em torno da meia-volta) o
/// `clamp` devolve sempre um dos extremos. ⇒ a conta é feita **relativa ao CENTRO**, com o
/// [`wrap_pi`] que a mistura já usa — assim o intervalo é uma faixa de arco e não um par de números
/// numa recta.
///
/// # ⛔ Um intervalo INVERTIDO trava no centro, e não é estado inválido
///
/// Com `max < min` a meia-largura seria negativa. Em vez de a deixar propagar (um `clamp` com
/// limites trocados **entra em pânico** em Rust), a lei apara-a em zero: a junta fica presa no
/// centro do intervalo que o artista escreveu. É uma resposta bem definida para uma entrada que o
/// painel não devia produzir, e é o que impede um ficheiro editado à mão de derrubar o app.
#[must_use]
pub fn clamp_to_limit(rot: f64, min: f64, max: f64) -> f64 {
    if !rot.is_finite() || !min.is_finite() || !max.is_finite() {
        return rot;
    }
    let centro = (min + max) * 0.5;
    let meia = ((max - min) * 0.5).max(0.0);
    let d = wrap_pi(rot - centro);
    // ⭐⭐⭐ **DENTRO DO LIMITE, DEVOLVE-SE `rot` AO BIT** — e a 1.ª redacção não o fazia.
    //
    // ⛔ Ela reconstruía sempre `centro + d`, o que **normaliza** o ângulo para a volta do centro:
    // medido, `−6,3` saía como `−0,016815`. Os dois são o MESMO ângulo (diferem por `2π`) e não os
    // mesmos BYTES — e o undo desta casa regista **por diferença de bytes**. ⇒ uma junta parada
    // dentro do próprio limite escreveria um valor novo a cada quadro, e cada clique do artista
    // empilharia um passo cujo conteúdo é *«o limite normalizou um ângulo»*.
    //
    // ⚠️ É a mesma família do defeito que o `two_bone` já pagou (*«uma corrente parada não
    // escreve»*), com outra origem: ali era ruído de `f32`, aqui é uma volta inteira.
    if d.abs() <= meia {
        return rot;
    }
    centro + d.clamp(-meia, meia)
}

/// A maior faixa que um limite pode ter: a volta inteira, que é o **no-op**.
///
/// ⚠️ Ela existe para o valor de nascimento ser derivado e não escrito à mão em dois sítios: um
/// limite tão largo quanto o círculo não apara nada, e é a prova de que ligar o controlo não move
/// a pose.
pub const FULL_TURN: f64 = std::f64::consts::TAU;

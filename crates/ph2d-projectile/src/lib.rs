#![forbid(unsafe_code)]
//! **A LEI de um PROJÉCTIL de arcade** — pura, sem rapier, sem ECS, sem shell.
//!
//! Dado *(config, o estado de voo, o alvo, dt)*, esta crate responde **que velocidade o projéctil
//! passa a ter**, **para onde ele olha**, **em que direcções pedir deslocamento ao mundo** e
//! **quando o voo acabou**. Quem traduz isso em chamadas de solver é a ponte
//! (`ph2d-physics-ecs::bridge::projectile`); quem desenha e autora é a shell.
//! Plano: [`docs/Components/11_plano_projectile_motion.md`].
//!
//! # ⭐⭐⭐ Por que ele existe, se a casa já ricocheteia
//!
//! Foi **medido antes de uma linha ser escrita** (§5.0: *antes de construir um item de lista
//! aberta, MEÇA se a composição já o exprime*), pela sonda
//! `ph2d-physics-ecs/tests/it/mede_o_que_a_composicao_ja_da.rs`:
//!
//! - ⛔ **o ricochete de um corpo DINÂMICO com `restitution = 1` já é EXACTO** — razão `1,000` e o
//!   `vx` do espelho ao terceiro decimal em todos os ângulos (90°..15°). *Escrever uma lei de
//!   ricochete nova seria um segundo motor para o que o solver calcula.*
//! - ⭐ **mas o mesmo tiro contra uma CAIXA LEVE desvia-se e perde rapidez** (`12,001 → 10,252`):
//!   um projéctil dinâmico é um **participante** da física. Uma bala de arcade não tem massa —
//!   ela atravessa a cena com a mesma rapidez independentemente do que bate.
//!
//! ⇒ o projéctil é **CINEMÁTICO**, como os dois controladores irmãos: ele escreve a própria pose
//! (lei transversal 2 da síntese, *um dono do transform por vez*), o que lhe dá massa irrelevante,
//! determinismo cross-OS, e anti-túnel de graça (o `move_character_from` é um **shape-cast**).
//!
//! E o que a casa **não tem** de todo: o **alcance percorrido** (o `Lifetime` mata por TEMPO, que é
//! outra grandeza — *uma bala lenta e uma rápida com o mesmo tempo de vida têm alcances
//! diferentes*), o **homing**, o **«a flecha aponta para onde voa»** e o **tecto de ricochetes**.
//!
//! # ⚠️ A ORDEM dentro do tique é load-bearing
//!
//! ```text
//! nascer ──► acelerações ACUMULAM ──► integrar UMA vez ──► tecto de rapidez
//!            (homing + avanço + gravidade)
//! ```
//!
//! ⭐ **Uma soma e um integrador**, que é a lei que os Motion Nodes desta casa já escrevem (*as
//! `force.*` são Pure e acumulam em `accel`; **UM** integrador aplica*). ⛔ Integrar cada
//! aceleração por sua vez faria os knobs deixarem de compor: o arco de uma bala com homing
//! dependeria da ordem em que alguém escreveu os `if`.
//!
//! # A unidade
//!
//! Metros e segundos, como o resto da casa. O corpus do oráculo está em **pixels**, e a comparação
//! é feita em **ângulos** e em **fracções do orçamento** — adimensionais, que é o que a torna
//! legítima.

pub mod bounce;

/// ⭐⭐ O vocabulário de vector e o **ORÇAMENTO DE MOVIMENTO** vêm da folha partilhada.
///
/// ⚠️ É uma RE-EXPORTAÇÃO, não uma cópia: são os MESMOS itens que a `ph2d-topdown` lê, e é isso que
/// impede um deslize e um ricochete de discordarem sobre o que é um orçamento.
pub use ph2d_sweep::{RESTO_MINIMO, SweepStep, Vec2, dot, first_step, len, mirror, normalize};

/// **A configuração inteira de um projéctil** — o que o componente registado guarda, sem ECS.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProjectileLaw {
    /// A rapidez com que ele **nasce**, m/s, na direcção para que o corpo está virado.
    pub initial_speed: f32,
    /// Aceleração ao longo da direcção de voo, m/s². Negativa trava.
    pub acceleration: f32,
    /// Tecto de rapidez, m/s. ⚠️ **`0` é SEM TECTO**, não «parado».
    pub max_speed: f32,
    /// A gravidade que faz o arco, m/s² para baixo. `0` = tiro recto.
    pub gravity: f32,
    /// A fracção da rapidez que **sobrevive** a um ricochete. `1` = perfeito, `0` = morre a bater.
    pub bounciness: f32,
    /// Quantos ricochetes o voo aguenta. ⚠️ **`0` = acaba no primeiro toque.**
    pub max_bounces: u8,
    /// Metros **percorridos** até o voo acabar. ⚠️ **`0` é SEM LIMITE.**
    pub range: f32,
    /// A flecha aponta para onde voa.
    pub face_velocity: bool,
    /// Aceleração de perseguição, m/s². `0` = não persegue (e aí o alvo é ignorado).
    pub homing_accel: f32,
}

impl Default for ProjectileLaw {
    fn default() -> Self {
        Self {
            initial_speed: 12.0,
            acceleration: 0.0,
            max_speed: 0.0,
            gravity: 0.0,
            bounciness: 1.0,
            max_bounces: 0,
            range: 0.0,
            face_velocity: true,
            homing_accel: 0.0,
        }
    }
}

/// **A MEMÓRIA de um projéctil entre tiques.**
///
/// ⚠️ **Ela não é componente, e não é opcional que não seja:** um campo que muda por tique dentro
/// de um componente registado faria o undo desta casa ver **cada quadro como um passo**. Ela vive
/// na ponte, dentro do `ControllerMemory` que entra no anel de checkpoints — é isso que a faz
/// sobreviver a um scrub.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ProjectileState {
    /// A velocidade de agora, m/s.
    pub velocity: Vec2,
    /// Quantos metros ele já **percorreu de facto** — ⚠️ não `|v|·dt` acumulado: uma bala que foi
    /// barrada não andou, e cobrar-lhe alcance seria matá-la mais cedo por ter batido.
    pub travelled: f32,
    /// Quantos ricochetes já gastou.
    pub bounces_used: u8,
    /// Já nasceu? ⚠️ O tique do nascimento é o que converte `initial_speed` + o ângulo do corpo
    /// numa velocidade — e ele só pode acontecer **uma vez**.
    pub launched: bool,
    /// ⭐⭐ **O voo ACABOU** — e isto tem de ser MEMÓRIA, não uma pergunta feita a cada tique.
    ///
    /// ⚠️ O alcance é derivável do estado (`travelled >= range`), mas o **tecto de ricochetes** só
    /// é observável no tique em que o corpo bate: no tique seguinte ele já não está a bater, e uma
    /// bala que gastou o último salto voltaria a voar. *Uma condição que só é verdadeira num
    /// instante tem de ser LEMBRADA.*
    pub finished: bool,
}

/// **Por que o voo acabou** — a ponte despacha a morte, a lei só diz o motivo.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ended {
    /// Percorreu os `range` metros.
    Range,
    /// Bateu com o tecto de ricochetes já gasto.
    Bounces,
}

/// **Um tique da lei**: acumula as acelerações, integra **uma** vez, e devolve a velocidade nova.
///
/// - `facing` — o ângulo actual do corpo, radianos. Só é lido no **nascimento**.
/// - `homing_target` — a posição do alvo, se houver um e se `homing_accel > 0`.
/// - `position` — onde o projéctil está, para a direcção da perseguição.
///
/// ⚠️ **Ela escreve em `state.velocity` e devolve-a**, para o chamador não ter de escolher: *duas
/// respostas à mesma pergunta divergem no dia em que alguém lê a errada*.
pub fn advance(
    state: &mut ProjectileState,
    law: &ProjectileLaw,
    facing: f32,
    homing_target: Option<Vec2>,
    position: Vec2,
    dt: f32,
) -> Vec2 {
    if !dt.is_finite() || dt <= 0.0 {
        return state.velocity;
    }
    // ── O NASCIMENTO, uma vez só ────────────────────────────────────────────
    if !state.launched {
        state.launched = true;
        let (s, c) = libm::sincosf(facing);
        state.velocity = [c * law.initial_speed, s * law.initial_speed];
    }

    // ── As acelerações ACUMULAM ─────────────────────────────────────────────
    let mut a = [0.0_f32, 0.0];

    // A perseguição: ⚠️ ela olha para onde o alvo ESTÁ, não para onde ele vai. Antecipar é outra
    // lei (a mira do `CameraFollow` do #7 já a tem) e um projéctil que a fizesse deixaria de ser
    // desviável — que é metade da graça de um míssil.
    if law.homing_accel > 0.0
        && let Some(alvo) = homing_target
        && let Some(d) = normalize([alvo[0] - position[0], alvo[1] - position[1]])
    {
        a[0] += d[0] * law.homing_accel;
        a[1] += d[1] * law.homing_accel;
    }

    // O avanço, ao longo da direcção de VOO. ⚠️ Sem velocidade não há direcção de voo, logo não há
    // onde acelerar — e inventar uma (o `facing`, por exemplo) faria uma bala parada arrancar
    // sozinha para um lado que ninguém pediu.
    if law.acceleration != 0.0
        && let Some(d) = normalize(state.velocity)
    {
        a[0] += d[0] * law.acceleration;
        a[1] += d[1] * law.acceleration;
    }

    // A gravidade, que faz o arco. ⚠️ `-y` é para baixo nesta casa.
    a[1] -= law.gravity;

    // ── UM integrador ───────────────────────────────────────────────────────
    let mut v = [state.velocity[0] + a[0] * dt, state.velocity[1] + a[1] * dt];

    // ── O tecto de rapidez ──────────────────────────────────────────────────
    if law.max_speed > 0.0 {
        let r = len(v);
        if r > law.max_speed {
            let k = law.max_speed / r;
            v = [v[0] * k, v[1] * k];
        }
    }
    state.velocity = v;
    v
}

/// **Para onde a flecha aponta** — `None` quando este projéctil não vira, ou quando não há
/// velocidade de onde tirar um ângulo.
///
/// ⚠️ **É instantâneo e não tem rampa**, ao contrário do irmão de vista de cima: uma seta em voo
/// aponta para onde voa *por construção* — uma rampa aqui seria uma flecha a apontar para onde
/// esteve, que é a leitura errada em todo quadro de um arco.
#[must_use]
pub fn facing_of(velocity: Vec2, law: &ProjectileLaw) -> Option<f32> {
    if !law.face_velocity {
        return None;
    }
    let d = normalize(velocity)?;
    Some(libm::atan2f(d[1], d[0]))
}

/// **O voo acabou?** — ⚠️ o `hit` diz se ESTE tique bateu em alguma coisa, e sem isso o tecto de
/// ricochetes nunca poderia acabar um voo.
#[must_use]
pub fn ended(state: &ProjectileState, law: &ProjectileLaw, hit: bool) -> Option<Ended> {
    // ⚠️ **O alcance vem primeiro**: uma bala que gasta o último metro contra uma parede já
    // percorreu o alcance, e dizer «morreu de bater» seria o motivo errado no relatório.
    if law.range > 0.0 && state.travelled >= law.range {
        return Some(Ended::Range);
    }
    if hit && state.bounces_used >= law.max_bounces {
        return Some(Ended::Bounces);
    }
    None
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;

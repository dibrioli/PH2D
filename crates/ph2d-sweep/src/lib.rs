#![forbid(unsafe_code)]
//! ⭐⭐⭐ **O ORÇAMENTO DE MOVIMENTO** — a lei que duas leis de movimento partilham.
//!
//! # O que ela diz
//!
//! Um corpo que se desloca contra o mundo pede `|v|·dt` de deslocamento numa direcção. O mundo
//! deixa andar **parte** disso e devolve uma superfície. A lei do orçamento é: **o que não coube
//! NÃO se perde** — ele é re-emitido noutra direcção com o resto **intacto**.
//!
//! ⚠️⚠️ **Só a RE-EMISSÃO muda entre os consumidores**, e é por isso que ela não vive aqui:
//!
//! | lei | quem | a direcção nova |
//! |---|---|---|
//! | **deslize** | [`ph2d-topdown`] (vista de cima) | a **TANGENTE** da superfície |
//! | **ricochete** | [`ph2d-projectile`] | o **ESPELHO** (`v − 2(v·n)n`) |
//!
//! # ⛔⛔ Por que isto é uma crate e não duas cópias de 25 linhas
//!
//! O orçamento é **a lei da wave do TOP-20 #13**, medida no oráculo (Godot `FLOATING`: a 45° o
//! corpo anda `1,414×` o que a projecção andaria; a 20°, `2,92×`). Escrevê-la uma segunda vez no
//! projéctil seria duplicar exactamente a lei que aquela wave existiu para trazer — e esta linha
//! pagou **duas vezes em 2026-09-15** o preço de uma lei escrita em dois sítios (a entrega do dedo
//! do jogador, e a chave de ordenação das raízes). *Uma lei escrita em dois sítios ainda não é uma
//! lei — só uma PORTA é.*
//!
//! ⭐ E o oráculo confirmou a lei **para o ricochete também**, num corpus próprio
//! (`ph2d-projectile/tests/fixtures/godot_bounce.txt`): o `move_and_collide` parte o orçamento de
//! `10,000000 px` em `3,984375` andados mais `6,015625` de resto — **soma exacta** —, e reflectir o
//! resto conserva-lhe o comprimento ao bit.
//!
//! # ⚠️ Por que um PLANO, e não uma função
//!
//! O orçamento gasta-se contra o **mundo**, e esta crate não tem mundo. Quem a consome chama
//! [`first_step`], pergunta ao solver quanto coube, e devolve isso ao `next_step` **da sua lei**
//! até ele dizer `None`. Uma função `andar(mundo)` poria o rapier dentro de uma folha.

/// **Um vector 2D** — um par cru.
///
/// ⚠️ Deliberadamente **não** é um tipo de uma biblioteca de matemática: uma folha que empurra
/// `glam` para os consumidores deixa de ser barata. `+ - * /` e `sqrt` são exactos no IEEE-754.
pub type Vec2 = [f32; 2];

/// O comprimento de um vector.
#[must_use]
#[inline]
pub fn len(v: Vec2) -> f32 {
    (v[0] * v[0] + v[1] * v[1]).sqrt()
}

/// O produto escalar.
#[must_use]
#[inline]
pub fn dot(a: Vec2, b: Vec2) -> f32 {
    a[0] * b[0] + a[1] * b[1]
}

/// **Normaliza, ou devolve `None`** se o vector for curto demais para ter direcção.
///
/// ⚠️ O piso é `1e-6`, e devolver `Option` em vez de um zero é a lei que a `ph2d-arclen` desta casa
/// pagou por escrito: *velocidade zero não é direcção ausente lida como zero — é direcção ausente,
/// e quem a recebe tem de decidir o que fazer*.
#[must_use]
pub fn normalize(v: Vec2) -> Option<Vec2> {
    let n = len(v);
    if n > 1.0e-6 {
        Some([v[0] / n, v[1] / n])
    } else {
        None
    }
}

/// **O ESPELHO** — `v − 2(v·n)n`, com `n` unitária.
///
/// ⭐ É a lei do ricochete, e ela foi **medida no oráculo**: o `Vector2.bounce(n)` do Godot acerta
/// com esta expressão a **`0,00000°`** nos seis ângulos do corpus.
///
/// ⚠️ **Reflectir não muda o comprimento**, e é isso que faz o orçamento atravessar um ricochete
/// inteiro — a outra metade da mesma lei.
#[must_use]
pub fn mirror(v: Vec2, n: Vec2) -> Vec2 {
    let d = 2.0 * dot(v, n);
    [v[0] - d * n[0], v[1] - d * n[1]]
}

/// A cerca abaixo da qual o que resta do orçamento não vale um passo.
///
/// ⚠️ **Em metros, e derivada do produto:** `1e-5 m` é um centésimo de milímetro, três ordens de
/// grandeza abaixo da margem do controlador (`~1 mm`). Um piso mais fino faria o plano gastar um
/// passo a mover nada.
pub const RESTO_MINIMO: f32 = 1.0e-5;

/// **Um passo do plano** — *«anda `budget` nesta `dir`»*.
///
/// ⚠️ **`steps_left` é o TECTO, e ele é obrigatório**: o oráculo mediu **dois** ricochetes a caberem
/// num tique quando o orçamento chega para isso, e numa quina fechada um corpo rápido ping-pongaria
/// sem fim. O deslize chama-lhe `max_slides`, o projéctil `max_bounces` — é o mesmo número.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SweepStep {
    /// A direcção deste passo, **normalizada**.
    pub dir: Vec2,
    /// Quanto ainda há para andar, em metros.
    pub budget: f32,
    /// Quantas mudanças de direcção ainda restam.
    pub steps_left: u8,
}

/// **O primeiro passo**: a velocidade pedida vira direcção e orçamento.
///
/// Devolve `None` quando não há velocidade — *direcção ausente não é direcção zero*, e quem não tem
/// para onde ir não pede nada ao mundo.
#[must_use]
pub fn first_step(v: Vec2, dt: f32, max_steps: u8) -> Option<SweepStep> {
    if !dt.is_finite() || dt <= 0.0 {
        return None;
    }
    let dir = normalize(v)?;
    let budget = len(v) * dt;
    if budget < RESTO_MINIMO {
        return None;
    }
    Some(SweepStep {
        dir,
        budget,
        steps_left: max_steps,
    })
}

/// **O RESTO do orçamento depois de o mundo ter deixado andar `moved`** — `None` quando não vale
/// um passo novo, seja por falta de orçamento ou por o tecto ter fechado.
///
/// ⚠️ É a metade da lei que as duas re-emissões partilham; o que cada uma escreve por si é a
/// DIRECÇÃO nova.
#[must_use]
pub fn remaining(step: SweepStep, moved: f32) -> Option<f32> {
    let resto = step.budget - moved.max(0.0);
    if resto < RESTO_MINIMO || step.steps_left == 0 {
        return None;
    }
    Some(resto)
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;

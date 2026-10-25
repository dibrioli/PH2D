//! ⭐⭐ **A família MUNDO nos canais** (F30 §B) — a gravidade e os outros campos de `PhysicsSettings`
//! que o painel *Physics* edita, sobre a entidade da Cena (`ph2d_ecs::cena`): [`publicar`] o
//! AUTORADO em `CanaisLidos`, [`aplicar`] o que a curva mandou em `CanaisEscritos`.
//!
//! ⭐ **A curva é pré-visualização e o autorado nunca muda por ela**: o autorado é o
//! `PhysicsBridge::settings` (o que o save grava, `project_save.rs`, e o painel edita); a curva vai
//! para o mundo VIVO por `PhysicsBridge::conduzir_settings`, que ⛔ não limpa o anel do scrub (o
//! `set_settings` limpa). Sem escrita da curva neste quadro, o mundo corre o autorado.
//!
//! ⭐ **O eco** diz ao Auto Key «o documento vale a curva» enquanto o autorado não muda; o quadro em
//! que a MÃO muda o autorado ([`ConducaoMundo`]) mostra a mão — no mundo e em `CanaisLidos` — e o
//! Auto Key grava-a.

use std::collections::BTreeMap;

use ph2d_ecs::canal::familia::MUNDO;
use ph2d_ecs::canal_mundo as m;
use ph2d_ecs::{CanaisEscritos, CanaisLidos, FieldKey, SimWorld};
use ph2d_physics_ecs::{PhysicsBridge, PhysicsSettings};

const fn k(campo: u16) -> FieldKey {
    FieldKey::new(MUNDO, campo, 0, 0)
}

/// Quanto vale `campo` em `s` — `None` fora do vocabulário.
#[must_use]
#[expect(clippy::cast_precision_loss, reason = "sub-passos e iterações ≤ 16")]
pub fn ler(s: &PhysicsSettings, campo: u16) -> Option<f32> {
    Some(match campo {
        m::GRAVIDADE_X => s.gravity_x,
        m::GRAVIDADE_Y => s.gravity_y,
        m::SUBPASSOS => s.substeps as f32,
        m::ITERACOES => s.solver_iterations as f32,
        m::CONTACTO_HZ => s.contact_hz,
        m::AMORT_LINEAR => s.linear_damping,
        m::AMORT_ANGULAR => s.angular_damping,
        m::AR => s.air_drag,
        m::SONO_VELOCIDADE => s.sleep_linear_threshold,
        m::SONO_DEMORA => s.time_until_sleep,
        m::ADORMECE => f32::from(u8::from(s.sleep_enabled())),
        _ => return None,
    })
}

/// Escreve `v` em `campo` de `s` (o `clamped` da ponte prende as faixas).
pub fn escrever(s: &mut PhysicsSettings, campo: u16, v: f32) {
    let inteiro = || {
        if v.is_finite() {
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "≥ 1"
            )]
            let n = v.round().max(1.0) as u32;
            n
        } else {
            1
        }
    };
    match campo {
        m::GRAVIDADE_X => s.gravity_x = v,
        m::GRAVIDADE_Y => s.gravity_y = v,
        m::SUBPASSOS => s.substeps = inteiro(),
        m::ITERACOES => s.solver_iterations = inteiro(),
        m::CONTACTO_HZ => s.contact_hz = v,
        m::AMORT_LINEAR => s.linear_damping = v,
        m::AMORT_ANGULAR => s.angular_damping = v,
        m::AR => s.air_drag = v,
        m::SONO_VELOCIDADE => s.sleep_linear_threshold = v,
        m::SONO_DEMORA => s.time_until_sleep = v,
        m::ADORMECE => *s = s.with_sleep_enabled(v >= 0.5),
        _ => {}
    }
}

fn cena(sim: &SimWorld) -> Option<u64> {
    ph2d_ecs::bits_da_cena(sim.world())
}

/// ⭐ **Publica o AUTORADO de cada campo do mundo** (através do eco) na entidade da Cena.
pub fn publicar(sim: &mut SimWorld, ponte: &PhysicsBridge) {
    let Some(c) = cena(sim) else { return };
    let s = ponte.settings();
    let mut lidos = sim
        .world_mut()
        .get_resource_or_insert_with(CanaisLidos::default);
    lidos.esquecer(c, MUNDO);
    for campo in m::CAMPOS {
        if let Some(v) = ler(&s, campo) {
            lidos.publicar(c, k(campo), v);
        }
    }
}

/// **O autorado que este funil viu no quadro anterior**, por campo — a diferença é a MÃO (o painel
/// mudou o autorado entre dois quadros), e nesse quadro a mão ganha à curva.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct ConducaoMundo {
    visto: BTreeMap<u16, f32>,
}

/// ⭐ **Aplica ao mundo vivo o que a curva mandou neste quadro** (sem limpar o anel) e regista o eco.
/// Devolve `true` se o mundo mudou.
pub fn aplicar(sim: &mut SimWorld, ponte: &mut PhysicsBridge, cond: &mut ConducaoMundo) -> bool {
    let escritas = sim
        .world_mut()
        .get_resource_mut::<CanaisEscritos>()
        .map(|mut w| w.drenar(MUNDO))
        .unwrap_or_default();
    let Some(c) = cena(sim) else { return false };
    let curva: BTreeMap<u16, f32> = escritas
        .into_iter()
        .filter(|&(e, kk, _)| e == c && kk.i == 0 && kk.j == 0)
        .map(|(_, kk, v)| (kk.campo, v))
        .collect();
    let autorado = ponte.settings();
    let mut alvo = autorado;
    let mut ecos = Vec::with_capacity(m::CAMPOS.len());
    for campo in m::CAMPOS {
        let Some(a) = ler(&autorado, campo) else {
            continue;
        };
        let mao = cond.visto.insert(campo, a).is_some_and(|antes| antes != a);
        match curva.get(&campo) {
            Some(&v) if !mao => {
                escrever(&mut alvo, campo, v);
                ecos.push((k(campo), v, a));
            }
            _ => ecos.push((k(campo), a, a)),
        }
    }
    let mut lidos = sim
        .world_mut()
        .get_resource_or_insert_with(CanaisLidos::default);
    for (kk, escrito, ficou) in ecos {
        lidos.eco(c, kk, escrito, ficou);
    }
    ponte.conduzir_settings(alvo)
}

#[cfg(test)]
#[path = "canais_mundo_tests.rs"]
mod tests;

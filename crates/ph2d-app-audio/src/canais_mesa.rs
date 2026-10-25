//! ⭐⭐ **A família MESA nos canais** (F30 §B2) — a mesa de som (`ph2d_audio::Mesa`) sobre a entidade
//! da Cena: [`publicar`] o AUTORADO em `CanaisLidos`, [`aplicar`] o que a curva mandou em
//! `CanaisEscritos` ao VIVO do [`crate::mesa_doc::MesaDoc`].
//!
//! A lei é a do `ph2d_app_physics::canais_mundo`: a curva é pré-visualização e o autorado nunca muda
//! por ela; o quadro em que a MÃO muda o autorado ([`ConducaoMesa`]) mostra a mão — no vivo e em
//! `CanaisLidos` — e o Auto Key grava-a. Sem escrita da curva neste quadro, a mesa corre o autorado.

use std::collections::BTreeMap;

use ph2d_audio::{BusMesa, Mesa};
use ph2d_ecs::canal::familia::MESA;
use ph2d_ecs::canal_mesa as m;
use ph2d_ecs::{CanaisEscritos, CanaisLidos, FieldKey, SimWorld};

use crate::mesa_doc::{MesaDoc, doc};

const fn k(campo: u16) -> FieldKey {
    FieldKey::new(MESA, campo, 0, 0)
}

const fn ligado(b: bool) -> f32 {
    if b { 1.0 } else { 0.0 }
}

fn ler_bus(b: &BusMesa, f: u16) -> Option<f32> {
    Some(match f {
        m::B_GANHO => b.ganho,
        m::B_MUDO => ligado(b.mudo),
        m::B_SOLO => ligado(b.solo),
        m::B_PAN => b.pan,
        m::B_TOM => b.tom_hz,
        m::B_CORTE_BAIXO => b.corte_baixo_hz,
        m::B_ENVIO_REVERB => b.envio_reverb,
        m::B_ENVIO_DELAY => b.envio_delay,
        m::B_COMP => b.comp,
        _ => return None,
    })
}

/// Quanto vale `campo` em `s` — `None` fora do vocabulário.
#[must_use]
pub fn ler(s: &Mesa, campo: u16) -> Option<f32> {
    if let Some((b, f)) = m::de_bus(campo) {
        return ler_bus(s.bus.get(usize::from(b))?, f);
    }
    Some(match campo {
        m::GANHO => s.ganho,
        m::CORTE_ALTO => s.corte_alto_hz,
        m::CORTE_BAIXO => s.corte_baixo_hz,
        m::MUDO => ligado(s.mudo),
        m::PAN => s.pan,
        m::LIMITADOR => ligado(s.limitador),
        m::REVERB => ligado(s.reverb),
        m::REVERB_TAMANHO => s.reverb_tamanho,
        m::REVERB_RETORNO => s.reverb_retorno,
        m::EQ_GRAVE => s.eq[0],
        m::EQ_MEDIO => s.eq[1],
        m::EQ_AGUDO => s.eq[2],
        m::DELAY => ligado(s.delay),
        m::DELAY_TEMPO => s.delay_tempo,
        m::DELAY_FEEDBACK => s.delay_feedback,
        m::DELAY_RETORNO => s.delay_retorno,
        m::DUCKING => ligado(s.ducking),
        m::DUCKING_PROFUNDIDADE => s.ducking_profundidade,
        m::DUCKING_CHAVE => f32::from(s.ducking_chave),
        _ => return None,
    })
}

fn escrever_bus(b: &mut BusMesa, f: u16, v: f32) {
    match f {
        m::B_GANHO => b.ganho = v,
        m::B_MUDO => b.mudo = v >= 0.5,
        m::B_SOLO => b.solo = v >= 0.5,
        m::B_PAN => b.pan = v,
        m::B_TOM => b.tom_hz = v,
        m::B_CORTE_BAIXO => b.corte_baixo_hz = v,
        m::B_ENVIO_REVERB => b.envio_reverb = v,
        m::B_ENVIO_DELAY => b.envio_delay = v,
        m::B_COMP => b.comp = v,
        _ => {}
    }
}

/// Escreve `v` em `campo` de `s`: os interruptores pelo meio, a chave arredondada a um barramento.
pub fn escrever(s: &mut Mesa, campo: u16, v: f32) {
    if let Some((b, f)) = m::de_bus(campo) {
        if let Some(bus) = s.bus.get_mut(usize::from(b)) {
            escrever_bus(bus, f, v);
        }
        return;
    }
    let ligado = v >= 0.5;
    match campo {
        m::GANHO => s.ganho = v,
        m::CORTE_ALTO => s.corte_alto_hz = v,
        m::CORTE_BAIXO => s.corte_baixo_hz = v,
        m::MUDO => s.mudo = ligado,
        m::PAN => s.pan = v,
        m::LIMITADOR => s.limitador = ligado,
        m::REVERB => s.reverb = ligado,
        m::REVERB_TAMANHO => s.reverb_tamanho = v,
        m::REVERB_RETORNO => s.reverb_retorno = v,
        m::EQ_GRAVE => s.eq[0] = v,
        m::EQ_MEDIO => s.eq[1] = v,
        m::EQ_AGUDO => s.eq[2] = v,
        m::DELAY => s.delay = ligado,
        m::DELAY_TEMPO => s.delay_tempo = v,
        m::DELAY_FEEDBACK => s.delay_feedback = v,
        m::DELAY_RETORNO => s.delay_retorno = v,
        m::DUCKING => s.ducking = ligado,
        m::DUCKING_PROFUNDIDADE => s.ducking_profundidade = v,
        m::DUCKING_CHAVE if v.is_finite() => {
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "preso a 0..BUSES-1"
            )]
            let i = v.round().clamp(0.0, f32::from(m::BUSES - 1)) as u8;
            s.ducking_chave = i;
        }
        _ => {}
    }
}

fn cena(sim: &SimWorld) -> Option<u64> {
    ph2d_ecs::bits_da_cena(sim.world())
}

/// ⭐ **Publica o AUTORADO de cada campo da mesa** (através do eco) na entidade da Cena.
pub fn publicar(sim: &mut SimWorld) {
    let Some(c) = cena(sim) else { return };
    let s = doc(sim).autorado;
    let mut lidos = sim
        .world_mut()
        .get_resource_or_insert_with(CanaisLidos::default);
    lidos.esquecer(c, MESA);
    for campo in m::campos() {
        if let Some(v) = ler(&s, campo) {
            lidos.publicar(c, k(campo), v);
        }
    }
}

/// **O autorado que este funil viu no quadro anterior**, por campo — a diferença é a MÃO.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct ConducaoMesa {
    visto: BTreeMap<u16, f32>,
}

/// ⭐ **Aplica ao VIVO o que a curva mandou neste quadro** e regista o eco. Devolve `true` se o
/// vivo mudou.
pub fn aplicar(sim: &mut SimWorld) -> bool {
    let escritas = sim
        .world_mut()
        .get_resource_mut::<CanaisEscritos>()
        .map(|mut w| w.drenar(MESA))
        .unwrap_or_default();
    let Some(c) = cena(sim) else { return false };
    let curva: BTreeMap<u16, f32> = escritas
        .into_iter()
        .filter(|&(e, kk, _)| e == c && kk.i == 0 && kk.j == 0)
        .map(|(_, kk, v)| (kk.campo, v))
        .collect();
    let mut d = doc(sim);
    let autorado = d.autorado;
    let mut vivo = autorado;
    let mut ecos = Vec::new();
    for campo in m::campos() {
        let Some(a) = ler(&autorado, campo) else {
            continue;
        };
        let mao = d
            .conducao
            .visto
            .insert(campo, a)
            .is_some_and(|antes| antes != a);
        match curva.get(&campo) {
            Some(&v) if !mao => {
                escrever(&mut vivo, campo, v);
                ecos.push((k(campo), v, a));
            }
            _ => ecos.push((k(campo), a, a)),
        }
    }
    let mudou = vivo != d.vivo;
    d.vivo = vivo;
    let w = sim.world_mut();
    w.insert_resource::<MesaDoc>(d);
    let mut lidos = w.get_resource_or_insert_with(CanaisLidos::default);
    for (kk, escrito, ficou) in ecos {
        lidos.eco(c, kk, escrito, ficou);
    }
    mudou
}

#[cfg(test)]
#[path = "canais_mesa_tests.rs"]
mod tests;

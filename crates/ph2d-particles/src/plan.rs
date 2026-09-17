//! **O PLANO DA EMISSÃO** — o que o componente e os sinais pedem, traduzido nos números do
//! `motion.emitter` (doc 14 §3.1). Função pura: nada aqui toca num grafo.
//!
//! # A lei que torna isto seguro de mudar a meio da corrida
//!
//! Um segmento só se **acrescenta** num instante `≥` agora, e só se **fecha** agora. ⇒ o relógio
//! de emissão `τ` até agora **não muda**, logo as ids de quem já nasceu não mudam — o integrador,
//! que casa o estado por id, continua a mesma simulação. *Mudar o passado renumeraria o mundo.*

use ph2d_ecs::ParticleEmitter;
use ph2d_node_motion_emitter::schedule::{EMIT_SCHEDULED, Schedule};

/// A explosividade a partir da qual uma rajada única é o `Burst` do nó, exacto.
pub const BURST_FROM: f32 = 0.999;

/// O pulso mais curto que um contínuo com explosividade pede, em fracção da vida.
///
/// ⚠️ **Divergência DECLARADA:** no oráculo um contínuo com explosividade `1` solta o ciclo inteiro
/// num instante. Aqui ele é um pulso de `0,1 %` da vida — as idades das partículas de um pulso
/// diferem menos de um milésimo da vida, o que não se vê, e a agenda continua a saber desligá-lo.
pub const MIN_PULSE: f32 = 0.001;

/// **Quando o emissor esteve ligado**, em segundos LOCAIS dele. `None` no fim = ainda ligado.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Emission {
    segments: Vec<(f64, Option<f64>)>,
}

impl Emission {
    /// A emissão de um emissor que nasce agora (local `0`), com o `emitting` do componente.
    #[must_use]
    pub fn born(cfg: &ParticleEmitter) -> Self {
        Self::born_with(cfg, cfg.emitting)
    }

    /// ⭐⭐ **A emissão de quem nasce por ORDEM** — o sinal de *recomeçar* (e o de *ligar* numa
    /// rajada) **liga**, mesmo que o componente nasça desligado: um emissor de explosão autora-se
    /// `emitting = false` e existe para ser disparado.
    #[must_use]
    pub fn born_with(cfg: &ParticleEmitter, emitting: bool) -> Self {
        let mut e = Self::default();
        if emitting {
            // ⚠️ Uma rajada de explosividade `1` tem ciclo ZERO, e um segmento vazio desaparece —
            // ela nunca rebentaria. O piso é um instante, que o `Burst` do nó nem lê.
            let end = cfg.one_shot.then(|| cycle(cfg).max(f64::MIN_POSITIVE));
            e.segments.push((0.0, end));
        }
        e
    }

    /// Está a emitir agora? (O último segmento continua aberto.)
    #[must_use]
    pub fn is_open(&self) -> bool {
        self.segments.last().is_some_and(|s| s.1.is_none())
    }

    /// Alguma vez ligou?
    #[must_use]
    pub fn ever(&self) -> bool {
        !self.segments.is_empty()
    }

    /// Os segmentos.
    #[must_use]
    pub fn segments(&self) -> &[(f64, Option<f64>)] {
        &self.segments
    }

    /// **Liga em `t`** — nada se já está ligado.
    pub fn start(&mut self, t: f64) {
        if !self.is_open() {
            self.segments.push((t, None));
        }
    }

    /// **Desliga em `t`** (L5) — nada se já está desligado. Um segmento que abriu no mesmo instante
    /// desaparece (um intervalo vazio não é um intervalo).
    pub fn stop(&mut self, t: f64) {
        self.close(t);
    }

    /// Fecha o segmento aberto em `t` — ou, numa rajada, encurta-a se `t` vem antes do fim dela.
    fn close(&mut self, t: f64) {
        let Some(last) = self.segments.last_mut() else {
            return;
        };
        let end = last.1.map_or(t, |b| b.min(t));
        if end <= last.0 {
            self.segments.pop();
        } else {
            last.1 = Some(end);
        }
    }
}

/// A duração da emissão de UMA rajada: `(1 − e)·vida` (L1).
#[must_use]
pub fn cycle(cfg: &ParticleEmitter) -> f64 {
    f64::from((1.0 - cfg.explosiveness.clamp(0.0, 1.0)) * cfg.life.max(0.0))
}

/// Os números que o `motion.emitter` recebe para a emissão pedida.
#[derive(Clone, Debug, PartialEq)]
pub struct EmitParams {
    /// `emit_mode` do nó (`1` = Burst · `2` = Scheduled).
    pub mode: f32,
    /// `rate` (partículas por segundo LIGADO).
    pub rate: f32,
    /// `burst_count` (só no Burst).
    pub burst_count: f32,
    /// A agenda (só no Scheduled).
    pub schedule: Schedule,
}

/// **O plano** — a emissão pedida, nos números do nó.
#[must_use]
pub fn emit_params(cfg: &ParticleEmitter, em: &Emission) -> EmitParams {
    #[expect(clippy::cast_precision_loss, reason = "uma contagem de partículas")]
    let amount = cfg.amount as f32;
    let life = cfg.life.max(0.0);
    let e = cfg.explosiveness.clamp(0.0, 1.0);
    if cfg.one_shot && e >= BURST_FROM {
        // Uma rajada exacta: todas no instante local `0`. Um emissor que nunca ligou não rebenta.
        return EmitParams {
            mode: 1.0,
            rate: 0.0,
            burst_count: if em.ever() { amount } else { 0.0 },
            schedule: Schedule::ALWAYS,
        };
    }
    let (on, pulse) = if cfg.one_shot {
        ((1.0 - e) * life, None)
    } else if e > 0.0 {
        let on = (1.0 - e).max(MIN_PULSE) * life;
        (on, Some((f64::from(on), f64::from(life))))
    } else {
        (life, None)
    };
    let rate = if on > 0.0 { amount / on } else { 0.0 };
    EmitParams {
        mode: EMIT_SCHEDULED as f32,
        rate,
        burst_count: 0.0,
        schedule: schedule_of(em, pulse),
    }
}

/// A agenda dos segmentos (com o pulso do contínuo).
fn schedule_of(em: &Emission, pulse: Option<(f64, f64)>) -> Schedule {
    if !em.ever() {
        return Schedule::never();
    }
    if pulse.is_none() && em.segments == [(0.0, None)] {
        return Schedule::ALWAYS;
    }
    let spans = em
        .segments
        .iter()
        .map(|&(a, b)| (a, b.unwrap_or(f64::INFINITY)))
        .collect();
    Schedule::new(spans, pulse).unwrap_or_else(|_| Schedule::never())
}

/// **Quando a emissão acaba de vez**, em segundos locais — `None` enquanto ela pode continuar.
#[must_use]
pub fn emission_end(cfg: &ParticleEmitter, em: &Emission) -> Option<f64> {
    if !em.ever() {
        return None;
    }
    let p = emit_params(cfg, em);
    if p.mode < 1.5 {
        return Some(0.0);
    }
    p.schedule.end()
}

#[cfg(test)]
#[path = "plan_tests.rs"]
mod tests;

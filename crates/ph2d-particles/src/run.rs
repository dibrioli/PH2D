//! **A CORRIDA de um emissor** — o grafo compilado, o cozinheiro dele e o relógio LOCAL (doc 14
//! §3.4–§3.6).
//!
//! # O relógio
//!
//! Nasce em `0` (mais o **pré-aquecimento**, que marcha a simulação antes do primeiro quadro — L3)
//! e anda `dt × time_scale` por passo (L4). ⚠️ **Em sub-passos de no máximo [`SUB_STEP`]**: o
//! integrador corta um passo maior que `0,03 s` (o `MAX_DT` dele), e um relógio acelerado
//! simularia menos do que o emissor conta.
//!
//! # O fim (L2, L5, L6, C0)
//!
//! [`EmitterRun::step`] devolve `true` no passo em que **a emissão acabou de vez E a última
//! partícula morreu** — uma vez só, até alguém voltar a ligar. Quem nunca teve uma partícula viva
//! não grita.

use std::collections::VecDeque;

use ph2d_ecs::{ParticleEmitter, ParticleSpace};
use ph2d_eval_motion::MotionCookPump;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_render::RenderInstance;

use crate::compile::{Compiled, TRACK_FILE, TRACK_T, TRACK_X, TRACK_Y, apply_emission, compile};
use crate::plan::{Emission, emission_end};

/// O sub-passo mais longo que a simulação dá — um quadro a 60 Hz, abaixo do tecto do integrador.
pub const SUB_STEP: f64 = 1.0 / 60.0;

/// **A pose de MUNDO do objecto** — a translação e a base 2×2 (colunas `[x, y]`, a convenção da
/// `RenderInstance::basis`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
    /// Onde o objecto está, em metros.
    pub translation: [f32; 2],
    /// `[x_basis.x, x_basis.y, y_basis.x, y_basis.y]` — rotação e escala.
    pub basis: [f32; 4],
}

impl Pose {
    /// Na origem, sem rodar.
    pub const IDENTITY: Self = Self {
        translation: [0.0, 0.0],
        basis: [1.0, 0.0, 0.0, 1.0],
    };

    /// Só uma translação.
    #[must_use]
    pub fn at(x: f32, y: f32) -> Self {
        Self {
            translation: [x, y],
            ..Self::IDENTITY
        }
    }

    /// A rotação da base, em graus (o eixo `x` dela).
    #[must_use]
    pub fn rotation_deg(&self) -> f32 {
        libm::atan2f(self.basis[1], self.basis[0]).to_degrees()
    }

    /// Um ponto do espaço do objecto, no mundo.
    #[must_use]
    pub fn apply(&self, p: [f32; 2]) -> [f32; 2] {
        let b = self.basis;
        [
            self.translation[0] + b[0] * p[0] + b[2] * p[1],
            self.translation[1] + b[1] * p[0] + b[3] * p[1],
        ]
    }

    /// Uma base do espaço do objecto, no mundo (`self · m`).
    #[must_use]
    pub fn compose_basis(&self, m: [f32; 4]) -> [f32; 4] {
        let b = self.basis;
        [
            b[0] * m[0] + b[2] * m[1],
            b[1] * m[0] + b[3] * m[1],
            b[0] * m[2] + b[2] * m[3],
            b[1] * m[2] + b[3] * m[3],
        ]
    }
}

/// **Um emissor a correr.**
pub struct EmitterRun {
    cfg: ParticleEmitter,
    compiled: Compiled,
    pump: MotionCookPump,
    emission: Emission,
    tick: u64,
    local_t: f64,
    /// A trajectória do objecto (`[t, x, y]`), só no espaço `World`.
    track: VecDeque<[f32; 3]>,
    angle: f32,
    seen_alive: bool,
    finished: bool,
    uv: [f32; 4],
}

impl EmitterRun {
    /// **Nasce** — o relógio local em `0`, a emissão do componente, o pré-aquecimento corrido.
    ///
    /// `uv` é o recorte do atlas que as partículas desenham (o ladrilho branco).
    #[must_use]
    pub fn born(cfg: &ParticleEmitter, reg: &NodeRegistry, pose: Pose, uv: [f32; 4]) -> Self {
        Self::born_with(cfg, reg, pose, uv, cfg.emitting)
    }

    /// **Nasce já a emitir** — o que um sinal de *recomeçar* pede (ver [`Emission::born_with`]).
    #[must_use]
    pub fn born_started(
        cfg: &ParticleEmitter,
        reg: &NodeRegistry,
        pose: Pose,
        uv: [f32; 4],
    ) -> Self {
        Self::born_with(cfg, reg, pose, uv, true)
    }

    fn born_with(
        cfg: &ParticleEmitter,
        reg: &NodeRegistry,
        pose: Pose,
        uv: [f32; 4],
        emitting: bool,
    ) -> Self {
        let emission = Emission::born_with(cfg, emitting);
        let compiled = compile(cfg, &emission);
        let mut pump = MotionCookPump::new();
        pump.set_time_fans(ph2d_node_motion_emitter::time_fans(
            &compiled.graph,
            reg,
            SUB_STEP,
        ));
        let mut run = Self {
            cfg: cfg.clone(),
            compiled,
            pump,
            emission,
            tick: 0,
            local_t: 0.0,
            track: VecDeque::new(),
            angle: f32::NAN,
            seen_alive: false,
            finished: false,
            uv,
        };
        // ⚠️ **Nascer não cozinha** — parado, um emissor acabado de pôr na cena não tem nenhuma
        // partícula (a corrida é o relógio a andar). Um cook aqui punha a primeira partícula na
        // tela de um emissor que ninguém correu.
        let prewarm = f64::from(run.cfg.prewarm.max(0.0));
        if prewarm > 0.0 {
            run.march(reg, pose, prewarm);
        }
        run
    }

    /// As definições com que nasceu — a ponte recompila quando o componente muda.
    #[must_use]
    pub fn config(&self) -> &ParticleEmitter {
        &self.cfg
    }

    /// O relógio local, em segundos.
    #[must_use]
    pub fn local_time(&self) -> f64 {
        self.local_t
    }

    /// Quantas partículas vivem agora.
    #[must_use]
    pub fn alive(&self) -> usize {
        self.pump.instances.len()
    }

    /// A emissão (os segmentos ligados).
    #[must_use]
    pub fn emission(&self) -> &Emission {
        &self.emission
    }

    /// O grafo compilado — o que «abrir como grafo» vai mostrar.
    #[must_use]
    pub fn graph(&self) -> &ph2d_nodegraph::graph::Graph {
        &self.compiled.graph
    }

    /// Porque a última cozedura recusou, se recusou.
    #[must_use]
    pub fn last_error(&self) -> Option<String> {
        self.pump.last_error().map(|e| format!("{e:?}"))
    }

    /// **Anda `dt` segundos de jogo.** `true` = a emissão acabou e a última partícula morreu
    /// NESTE passo (o `finished` do oráculo).
    pub fn step(&mut self, reg: &NodeRegistry, pose: Pose, dt: f64) -> bool {
        let total = dt * f64::from(self.cfg.time_scale.max(0.0));
        if total > 0.0 {
            self.march(reg, pose, total);
        }
        let ended = emission_end(&self.cfg, &self.emission).is_some_and(|e| self.local_t >= e);
        if ended && self.seen_alive && self.alive() == 0 && !self.finished {
            self.finished = true;
            return true;
        }
        false
    }

    /// **Liga** agora (L6). Numa rajada única é o caller que recomeça — ver [`Self::wants_restart_to_start`].
    pub fn start(&mut self) {
        if self.emission.is_open() {
            return;
        }
        self.emission.start(self.local_t);
        self.finished = false;
        self.reapply();
    }

    /// **Desliga** agora — as vivas acabam a vida (L5).
    pub fn stop(&mut self) {
        self.emission.stop(self.local_t);
        self.reapply();
    }

    /// Numa rajada única, *ligar* é *recomeçar* (L7) — a rajada não tem um «a meio» a retomar.
    #[must_use]
    pub fn wants_restart_to_start(&self) -> bool {
        self.cfg.one_shot
    }

    /// Junta as partículas vivas a `out`, no MUNDO, com a profundidade `z_order` do objecto.
    pub fn append_instances(&self, out: &mut Vec<RenderInstance>, pose: Pose, z_order: u32) {
        out.reserve(self.pump.instances.len());
        for inst in &self.pump.instances {
            let mut i = *inst;
            if self.cfg.space == ParticleSpace::Local {
                i.world_pos = pose.apply(i.world_pos);
                i.basis = pose.compose_basis(i.basis);
            }
            i.z_order = z_order;
            out.push(i);
        }
    }

    fn reapply(&mut self) {
        apply_emission(
            &mut self.compiled.graph,
            self.compiled.emitter,
            &self.cfg,
            &self.emission,
        );
    }

    /// Marcha `total` segundos locais em sub-passos iguais, cada um uma cozedura.
    fn march(&mut self, reg: &NodeRegistry, pose: Pose, total: f64) {
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "um número pequeno e positivo de sub-passos"
        )]
        let n = (total / SUB_STEP).ceil().max(1.0) as u64;
        #[expect(
            clippy::cast_precision_loss,
            reason = "um número pequeno de sub-passos"
        )]
        let h = total / n as f64;
        for _ in 0..n {
            self.local_t += h;
            self.tick += 1;
            self.cook(reg, pose);
        }
    }

    fn cook(&mut self, reg: &NodeRegistry, pose: Pose) {
        if self.cfg.space == ParticleSpace::World {
            self.follow(pose);
        }
        let size = self.cfg.size.max(0.0);
        self.pump.pump(
            &self.compiled.graph,
            reg,
            &[self.compiled.sink],
            self.tick,
            self.local_t,
            self.uv,
            [size, size],
        );
        if !self.pump.instances.is_empty() {
            self.seen_alive = true;
        }
    }

    /// No espaço `World`: grava a pose de agora na trajectória, publica-a, e roda o cone.
    fn follow(&mut self, pose: Pose) {
        #[expect(
            clippy::cast_possible_truncation,
            reason = "segundos locais cabem em f32"
        )]
        let t = self.local_t as f32;
        self.track
            .push_back([t, pose.translation[0], pose.translation[1]]);
        // Só a janela que a história ainda pode pedir: uma vida para trás (mais uma folga).
        let keep_from = t - self.cfg.life.max(0.0) - 4.0 * SUB_STEP as f32;
        while self.track.len() > 2 && self.track[1][0] < keep_from {
            self.track.pop_front();
        }
        let col = |k: usize| Column::Scalar(self.track.iter().map(|s| s[k]).collect());
        let stream = Stream::new(self.track.len())
            .with(TRACK_T, col(0))
            .with(TRACK_X, col(1))
            .with(TRACK_Y, col(2));
        self.pump
            .cook
            .set_external(ph2d_node_registry::table_external_key(TRACK_FILE), stream);
        let angle = self.cfg.angle + pose.rotation_deg();
        if angle.to_bits() != self.angle.to_bits() {
            self.angle = angle;
            self.compiled
                .graph
                .set_param(self.compiled.emitter, "angle", angle);
        }
    }
}

#[cfg(test)]
#[path = "run_tests.rs"]
mod tests;

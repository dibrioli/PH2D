//! **A ponte do player de plataforma** (W2/W3) — amostra o mundo, chama a lei,
//! aplica o motor.
//!
//! Três metades, e nenhuma sabe da outra: o **cast** é do wrapper
//! (`world::cast`), a **lei** é da folha pura (`ph2d-platformer`), e **aplicar**
//! é do wrapper outra vez (`world::player`). Este módulo é só o fio — e é ele
//! que garante que os três falem do mesmo corpo, no mesmo tick.
//!
//! # ⚠️ Onde o sensor pergunta, e por que é DEPOIS do step anterior
//!
//! O cast lê o BVH do broad phase, que descreve o mundo que o **último `step`**
//! deixou (medido em `world::cast`). Rodar aqui — no topo de cada tick devido,
//! antes do `step` daquele tick — significa perguntar sobre o mundo do tick
//! anterior, que é exatamente o que um sensor pode saber: a alternativa seria
//! consultar um futuro que ainda não foi resolvido.
//!
//! **Consequência honesta, nomeada:** no primeiríssimo tick de uma cena o BVH
//! ainda está vazio, o cast devolve `None`, e o player cai por um tick antes de
//! a mola pegá-lo. É invisível a 60 Hz e é o preço de não manter um segundo
//! índice espacial só para o primeiro quadro.
//!
//! # ⚠️ A amostra do chão e a REAÇÃO são a MESMA resposta
//!
//! O plano (§7) nomeou isto como um dos dois lugares onde este módulo tentaria
//! adoecer: *quem decide "estou no chão" e quem decide "em quem eu empurro"
//! têm de ser a mesma consulta*. Por isso o [`CastHit`] inteiro é carregado
//! adiante — corpo, ponto e normal —, e não só a distância. Quando a W6 chegar,
//! a reação nasce **deste mesmo hit**, não de uma segunda pergunta.
//!
//! [`CastHit`]: ph2d_physics::CastHit

use bevy_ecs::entity::Entity;
use ph2d_ecs::SimWorld;
use ph2d_platformer::{GroundSample, PlayerInput, player_motor};

use crate::components::{BodyKind, PlatformPlayer};

use super::PhysicsBridge;

/// A direção "para cima" que este módulo assume.
///
/// ⚠️ Um número, um lugar. Ele governa o eixo da mola, o eixo de caminhada no ar
/// e o limite de rampa, e as três respostas **têm** de concordar: se o eixo da
/// mola e o do limite discordassem, existiria uma inclinação em que o
/// personagem é segurado por uma e recusado pela outra.
///
/// Gravidade lateral segue possível na cena (o mundo aceita qualquer vetor) e o
/// player não a acompanha — é a limitação honesta desta wave, e a cura, se um
/// dia for pedida, é derivar o `up` da gravidade **numa porta só**, esta.
const UP: [f32; 2] = [0.0, 1.0];

impl PhysicsBridge {
    /// **A entrada deste player.** Chamada pela shell a cada frame.
    ///
    /// Escrever `drive = 0` é uma instrução (*"pare"*), não a ausência de uma —
    /// e é por isso que a tabela guarda a entrada em vez de a consumir: um
    /// dispatch que deve vários ticks aplica a MESMA entrada a todos eles, que é
    /// o que um jogador segurando uma tecla quer dizer.
    pub fn set_player_input(&mut self, entity: Entity, input: PlayerInput) {
        self.player_input.insert(entity, input);
    }

    /// O que este player está recebendo agora (`default` = parado).
    #[must_use]
    pub fn player_input(&self, entity: Entity) -> PlayerInput {
        self.player_input.get(&entity).copied().unwrap_or_default()
    }

    /// Esquece toda entrada de player.
    ///
    /// Chamada por quem derruba o mundo derivado (`rebuild`): os bits de
    /// entidade são reciclados ali, então uma entrada guardada passaria a
    /// dirigir **outro** objeto — a mesma armadilha que fez as âncoras de joint
    /// viajarem por NOME em vez de por bits.
    pub fn clear_player_input(&mut self) {
        self.player_input.clear();
    }

    /// **Um tick de todos os players.** Chamado no laço de ticks devidos, antes
    /// do `step` (ver o aviso do módulo).
    ///
    /// No-op numa cena sem player — e é o que mantém esta wave byte-neutra para
    /// todo o resto do módulo.
    pub(super) fn drive_players(&mut self, sim: &SimWorld) {
        let world = sim.world();
        let gravity = self.world.gravity();
        let dt = self.world.dt();
        // A ordem é a do `BTreeMap` de corpos — determinística cross-OS, a lei
        // do módulo. Coletar antes de aplicar porque o cast toma `&self` e o
        // motor toma `&mut self`.
        let mut motors: Vec<(rapier2d_handle::Handle, [f32; 2], [f32; 2])> = Vec::new();
        for (&entity, b) in self.bodies.iter() {
            // Dynamic-only, e é FÍSICA: um impulso não move massa infinita.
            if b.kind != BodyKind::Dynamic {
                continue;
            }
            let Some(cfg) = world.get::<PlatformPlayer>(entity) else {
                continue;
            };
            let Some(pose) = self.world.body_pose(b.handle) else {
                continue;
            };
            let origin = [pose.translation.x, pose.translation.y];
            let Some(vel) = self.world.body_velocity(b.handle) else {
                continue;
            };

            let cfg = cfg.config();
            // O alcance do sensor é o que a lei considera "no chão", e nem um
            // milímetro além: perguntar mais longe faria o cast achar coisas que
            // a lei descartaria, ao preço de descer mais no BVH.
            let reach = cfg.ride.float_height + cfg.ride.cling_distance;
            let hit = self
                .world
                .cast_ray(origin, [0.0, -1.0], reach, Some(b.handle), b.rest.layer);

            let sample = hit.map(|h| GroundSample {
                distance: h.distance,
                normal: h.normal,
                // ⚠️ A velocidade do PONTO, não a do centro: uma plataforma que
                // gira leva a borda mesmo com o centro parado
                // (`PhysicsWorld::point_velocity`).
                ground_velocity: h
                    .body
                    .and_then(|gb| self.world.point_velocity(gb, h.point))
                    .unwrap_or([0.0, 0.0]),
            });

            let input = self.player_input.get(&entity).copied().unwrap_or_default();
            let motor = player_motor(&cfg, sample.as_ref(), input, vel, gravity, UP, dt);
            if motor.accel != [0.0, 0.0] || motor.boost != [0.0, 0.0] {
                motors.push((b.handle, motor.accel, motor.boost));
            }
        }
        for (handle, accel, boost) in motors {
            self.world.apply_player_motor(handle, accel, boost);
        }
    }
}

/// O handle do rapier, nomeado sem importar o rapier aqui — esta crate declara-se
/// *rapier-free* no próprio `Cargo.toml` e só carrega os tipos re-exportados.
mod rapier2d_handle {
    pub type Handle = ph2d_physics::RigidBodyHandle;
}

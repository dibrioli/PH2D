//! **O RAIO persistente** (suplente #21) — um por [`RaySensor`], por TIQUE.
//!
//! # ⚠️ Por TIQUE, e não por dispatch — a mesma razão do canal de contactos
//!
//! Um quadro que deve três tiques anda o mundo três vezes, e uma parede que apareça e desapareça
//! dentro dele seria **invisível** a uma amostragem por quadro. É a frase que o
//! [`ContactEvent`](super::contacts::ContactEvent) já escreve (*«um toque mais curto que um tique
//! ainda grita»*), um nível ao lado.
//!
//! # ⚠️ A POSE sai do MUNDO quando há corpo, e do `Transform` quando não há
//!
//! Dentro do laço de tiques o `Transform` é do quadro ANTERIOR — o `readback` só corre no fim —,
//! logo um raio que o lesse nasceria onde o objecto estava, não onde ele está. *Uma sonda que mede
//! o estado anterior não fica calada: ela responde, e errado.*
//!
//! # ⛔ O que este módulo NÃO faz
//!
//! Ele **não** mantém uma lista própria de sinais. As duas listas que ele enche
//! ([`ray_enters`](PhysicsBridge::ray_enters) / [`ray_exits`](PhysicsBridge::ray_exits)) são
//! **canais**, como os do contacto e os do gatilho, e quem os transforma em sinal é o
//! [`signal_events`](PhysicsBridge::signal_events) — que é onde o `SignalTagFilter` já mora. *Uma
//! segunda lista seria uma segunda resposta a «o que aconteceu neste quadro».*

use std::collections::BTreeMap;

use ph2d_ecs::{Entity, SimWorld};

use super::PhysicsBridge;
use crate::{RayHit, RaySensor};

impl PhysicsBridge {
    /// **Lança um raio por sensor e diz o que cada um viu**, acumulando as arestas.
    ///
    /// ⚠️ **O corpo dono é EXCLUÍDO**, e o gate `the_caster_can_exclude_itself` do motor mede o que
    /// acontece sem isso: *«um personagem acha-se no chão para sempre»*. ⭐ Um sensor num objecto
    /// **sem corpo** não tem o que excluir, e a resposta certa ali é não excluir nada — está medido
    /// no gate `um_raio_sem_corpo_nao_exclui_ninguem`, não suposto.
    ///
    /// ⚠️ **O mapa `handle → entidade` vem de FORA**, como o do canal de contactos: construí-lo
    /// dentro custaria uma travessia de todos os corpos **por tique**, e quem chama já o tem.
    pub(super) fn cast_ray_sensors(
        &mut self,
        sim: &mut SimWorld,
        by_handle: &BTreeMap<(u32, u32), Entity>,
    ) {
        let mut agora: BTreeMap<Entity, RayHit> = BTreeMap::new();
        {
            let world = sim.world_mut();
            let mut q = world.query::<(Entity, &RaySensor)>();
            // ⚠️ A lista sai da consulta ANTES do laço: o `cast_ray` empresta `self.world`, e a
            // consulta empresta o `World` do ECS — separá-los é o que mantém os dois empréstimos
            // disjuntos sem uma cópia do mundo.
            let sensores: Vec<(Entity, RaySensor)> = q.iter(world).map(|(e, r)| (e, *r)).collect();
            for (e, r) in sensores {
                // A pose: do MUNDO se houver corpo (é a de AGORA), do `Transform` se não houver.
                let (px, py, ang, excluir) = match self.bodies.get(&e) {
                    Some(b) => match self.world.body_pose(b.handle) {
                        Some(p) => (
                            p.translation.x,
                            p.translation.y,
                            p.rotation.angle(),
                            Some(b.handle),
                        ),
                        None => continue,
                    },
                    None => match ph2d_ecs::world_transform(world, e) {
                        Some(t) => (t.translation.x, t.translation.y, t.rotation, None),
                        None => continue,
                    },
                };
                // ⭐ **Os dois vectores são LOCAIS**, logo a pose roda-os — é isso que faz a mira de
                // uma torreta seguir a torreta sem uma segunda lei.
                // ⚠️⚠️ **`libm` e nunca o `std`** — este código corre no caminho do `physics_ecs_c9`,
                // e a libc de cada SO devolve o último ulp do seno diferente: o hash partiria entre
                // as três plataformas, e **só o CI o mediria**. O gate
                // `no_std_transcendental_reaches_the_deterministic_hash` apanhou-o na primeira
                // corrida desta wave, com o ficheiro e a linha.
                let (s, c) = (libm::sinf(ang), libm::cosf(ang));
                let gira = |v: ph2d_core::Vec2| [v.x * c - v.y * s, v.x * s + v.y * c];
                let o = gira(r.origin);
                let d = gira(r.dir);
                let origem = [px + o[0], py + o[1]];
                let Some(h) = self.world.cast_ray(origem, d, r.reach, excluir, r.layer) else {
                    continue;
                };
                let Some(alvo) = h
                    .body
                    .and_then(|b| by_handle.get(&b.into_raw_parts()).copied())
                else {
                    continue;
                };
                agora.insert(
                    e,
                    RayHit {
                        body: alvo,
                        distance: h.distance,
                        point: h.point,
                        normal: h.normal,
                    },
                );
            }
        }
        // As ARESTAS, contra o que cada um via no tique anterior.
        //
        // ⚠️ **Trocar de alvo é uma SAÍDA e uma ENTRADA**, não um silêncio: quem escuta
        // `viu_inimigo` tem de ouvir de novo quando o inimigo passa a ser outro, senão a porta
        // reage ao primeiro e ignora todos os seguintes.
        for (&e, h) in &agora {
            match self.ray_hits.get(&e) {
                Some(anterior) if anterior.body == h.body => {}
                Some(anterior) => {
                    self.ray_exits.push((e, anterior.body));
                    self.ray_enters.push((e, h.body));
                }
                None => self.ray_enters.push((e, h.body)),
            }
        }
        for (&e, anterior) in &self.ray_hits {
            if !agora.contains_key(&e) {
                self.ray_exits.push((e, anterior.body));
            }
        }
        self.ray_hits = agora;
    }

    /// **O que cada raio vê AGORA** — a leitura que o painel e o desenho consomem.
    #[must_use]
    pub fn ray_sensor_hits(&self) -> &BTreeMap<Entity, RayHit> {
        &self.ray_hits
    }

    /// Os raios que PASSARAM A ver alguém neste dispatch — `(quem olha, quem foi visto)`.
    #[must_use]
    pub fn ray_enters(&self) -> &[(Entity, Entity)] {
        &self.ray_enters
    }

    /// Os que DEIXARAM de ver — `(quem olha, quem deixou de ser visto)`.
    #[must_use]
    pub fn ray_exits(&self) -> &[(Entity, Entity)] {
        &self.ray_exits
    }
}

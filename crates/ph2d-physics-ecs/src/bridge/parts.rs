//! **A metade PEÇA do reconcile** — um filho que carrega só `Collider` vira mais
//! um collider do corpo ancestral (W-Compound).
//!
//! # O vão, medido
//!
//! Até aqui um corpo tinha exatamente UM collider, e a `BodyQuery` dizia isso no
//! tipo (`RigidBody` **e** `Collider` na mesma entidade). As duas coisas que um
//! artista tentaria foram medidas (`tests/measure_compound.rs`) e as duas falham:
//!
//! - filho com **só** `Collider` — **invisível ao solver**: a perna de um "L"
//!   atravessa o chão (`y = −0,30` com o topo do chão em `0,5`), sem erro e sem
//!   warning;
//! - filho com `Collider` **e** `RigidBody` — vira OUTRO corpo, e as duas peças se
//!   espalham: o offset autorado `[0,8, −1,0]` virou `[2,08, +0,80]`.
//!
//! # A resposta é a HIERARQUIA, e ela já existe
//!
//! Um filho com `Collider` e **sem** `RigidBody` é mais uma forma do corpo
//! ancestral mais próximo — literalmente o `CollisionShape2D` do Godot, com a
//! nossa árvore no lugar da dele, e o norte do
//! [ADR-0110](../../../../docs/architecture/decisions/0110-vector-nodes-are-ecs-entities-one-hierarchy.md)
//! (*tudo é entidade, uma árvore só*) no lugar de um vetor de formas dentro do
//! componente. O que se ganha com isso é o que a Hierarquia já sabe fazer:
//! nomear, selecionar, mover, apagar, desfazer e salvar cada peça.
//!
//! ⚠️ **O ancestral é o mais PRÓXIMO que é corpo**, não o pai literal — o mesmo
//! walk do `rig_edges` (W-Rig), e pela mesma razão: um GRUPO no meio (nó sem
//! desenho, só organização) tem de ser transparente. Sem isso, pôr as formas de
//! uma peça dentro de uma pasta as desligaria do corpo em silêncio.
//!
//! ⚠️ **`part_views` MORREU** (W-PartFace): ela devolvia a forma e a pose que o
//! SOLVER guarda, e nunca teve chamador — o contorno deriva a pose de mundo do
//! `Transform`, que é o que o artista de fato arrasta, e um segundo canal para o
//! mesmo fato é a divergência esperando alguém chamá-la. Quem precisar de *quais
//! formas este corpo tem* pergunta à ÁRVORE (`crate::count_parts`), que é onde a
//! resposta é autorada.
//!
//! # A pose é derivada, nunca autorada duas vezes
//!
//! Onde a peça está no corpo sai de `inverse_compose(peça_mundo, corpo_mundo)` —
//! a álgebra exata que o W5 construiu para o oposto (ler o mundo de um corpo
//! parenteado). Um campo "local" no componente seria o segundo lugar do mesmo
//! fato, e ele divergiria do `Transform` que o artista de fato arrasta.

use ph2d_ecs::{Entity, SimWorld};
use ph2d_physics::{ColliderHandle, RigidBodyHandle};

use super::{PhysicsBridge, space};
use crate::RigidBody;

/// Uma peça viva: o collider no solver, de quem ela é, e a descrição com que
/// nasceu — a mesma dupla `(handle, rest)` que um corpo carrega, e pelo mesmo
/// motivo (um rewind reconstrói o mundo a partir dela).
pub(super) struct PartRef {
    pub(super) handle: ColliderHandle,
    pub(super) owner: Entity,
    pub(super) rest: ph2d_physics::BodyDesc,
    pub(super) local: [f32; 3],
}

impl PhysicsBridge {
    /// O corpo de que `shape` é a forma PRINCIPAL filha (opção B), ou `None`.
    pub(super) fn main_body_of(&self, shape: Entity) -> Option<Entity> {
        self.main_of
            .binary_search_by_key(&shape.to_bits(), |(s, _)| s.to_bits())
            .ok()
            .map(|i| self.main_of[i].1)
    }

    /// Quem GRITA o sinal de um sensor `shape`: o corpo dono quando ela é uma forma filha (a
    /// principal ou uma peça), senão ela mesma (a forma no próprio corpo).
    pub(super) fn signal_source(&self, shape: Entity) -> Entity {
        self.main_body_of(shape)
            .or_else(|| self.parts.get(&shape).map(|p| p.owner))
            .unwrap_or(shape)
    }

    /// A entidade da forma principal de `body` (ele mesmo quando a forma mora nele).
    pub(super) fn shape_of(&self, body: Entity) -> Entity {
        self.bodies.get(&body).map_or(body, |b| b.shape)
    }

    /// O corpo ancestral mais próximo de `e` que a ponte de fato construiu.
    ///
    /// ⚠️ **Pergunta ao `self.bodies` e não só ao componente**: é ele que diz que corpo a
    /// ponte de fato construiu (um sem forma também, de colisor desligado — dono 09/11).
    fn owner_body(&self, sim: &SimWorld, e: Entity) -> Option<(Entity, RigidBodyHandle)> {
        let world = sim.world();
        let mut cur = world
            .get::<ph2d_ecs::ChildOf>(e)
            .map(ph2d_ecs::ChildOf::parent);
        while let Some(p) = cur {
            if let Some(b) = self.bodies.get(&p) {
                return Some((p, b.handle));
            }
            cur = world
                .get::<ph2d_ecs::ChildOf>(p)
                .map(ph2d_ecs::ChildOf::parent);
        }
        None
    }

    /// Pendura / tira / re-descreve as peças para casar com as entidades que
    /// carregam `Collider` sem `RigidBody`. Chamado DEPOIS dos corpos: uma peça
    /// precisa do dono já construído.
    pub(super) fn reconcile_parts(&mut self, sim: &SimWorld) {
        self.part_seen.clear();
        let world = sim.world();
        let mut q = self.part_query.take().expect("query built in dispatch");

        let mut wanted: Vec<(
            Entity,
            Entity,
            RigidBodyHandle,
            ph2d_physics::BodyDesc,
            [f32; 3],
        )> = Vec::new();
        for (e, col, _local) in q.iter(world) {
            // A forma PRINCIPAL filha já viaja no `BodyDesc` do corpo: pendurá-la de novo seria um
            // collider dobrado.
            if self.main_body_of(e).is_some() || crate::forma_desligada(world, e) {
                continue;
            }
            let Some((owner, handle)) = self.owner_body(sim, e) else {
                continue;
            };
            self.part_seen.push(e);
            let Some((desc, local)) = self.desc_da_peca(world, e, owner, col) else {
                continue;
            };
            wanted.push((e, owner, handle, desc, local));
        }
        self.part_query = Some(q);
        // A ordem de pendurar decide os handles: por entidade, nunca pela do arquétipo (HR-5).
        wanted.sort_unstable_by_key(|w| w.0.to_bits());

        // Peças que sumiram (a entidade morreu, ganhou `RigidBody`, ou o dono
        // deixou de ser corpo). `retain` sobre a lista vista, o mesmo padrão do
        // corpo.
        let seen = std::mem::take(&mut self.part_seen);
        let stale: Vec<Entity> = self
            .parts
            .keys()
            .copied()
            .filter(|e| !seen.contains(e))
            .collect();
        for e in stale {
            if let Some(p) = self.parts.remove(&e) {
                self.world.detach_part(p.handle);
                self.ring.clear(); // estrutural: os checkpoints indexam o collider que saiu
            }
        }
        self.part_seen = seen;

        for (e, owner, handle, desc, local) in wanted {
            match self.parts.get(&e) {
                // ⚠️ **Sem gate de repouso** (W-PartFace): a pose local sai da
                // cadeia AUTORADA, então ela não muda sozinha durante o play e a
                // comparação acima é exata. O gate que existia aqui — copiado do
                // corpo, cuja pose o SOLVER escreve — tinha um preço medido:
                // mover uma peça com o relógio andando movia o desenho e deixava
                // o collider parado, e a forma atravessava o estático em
                // silêncio. Uma peça é AUTORADA o tempo todo; o dono é que é
                // simulado.
                Some(p) if p.owner == owner && p.rest == desc && p.local == local => {}
                // O mesmo dono, outra forma, material ou pose no corpo (a curva a meio do Play):
                // reafina-se no lugar, como o corpo e a junta — o ring vale (GEF02 onda 7).
                Some(p) if p.owner == owner && self.world.retune_part(p.handle, &desc, local) => {
                    if let Some(p) = self.parts.get_mut(&e) {
                        p.rest = desc;
                        p.local = local;
                    }
                }
                _ => {
                    self.ring.clear(); // estrutural, como o spawn/remove de um corpo
                    if let Some(old) = self.parts.remove(&e) {
                        self.world.detach_part(old.handle);
                    }
                    if let Some(h) = self.world.attach_part(handle, &desc, local) {
                        self.parts.insert(
                            e,
                            PartRef {
                                handle: h,
                                owner,
                                rest: desc,
                                local,
                            },
                        );
                    }
                }
            }
        }
    }

    /// **O que a peça `e` (de `owner`) é para o solver AGORA**: a descrição (pose zerada, só a
    /// metade-colisor) e a pose dela no corpo, lida da cadeia AUTORADA. A porta única do
    /// `reconcile_parts` (1×/quadro) e do `drive_shapes` (1×/tique).
    pub(super) fn desc_da_peca(
        &mut self,
        world: &bevy_ecs::world::World,
        e: Entity,
        owner: Entity,
        col: &crate::Collider,
    ) -> Option<(ph2d_physics::BodyDesc, [f32; 3])> {
        // A pose de MUNDO da peça — é dela que sai a ESCALA que o
        // `scaled_shape` do W6 lê.
        let t_part = space::world_transform(world, e, &mut self.chain)?;
        // ⚠️ **A pose LOCAL vem da cadeia AUTORADA, não de um round-trip pelo
        // solver** (docs de `space::local_in_ancestor`): é isso que a torna
        // bit-estável durante o play e que apaga a razão do gate de repouso.
        let rel = space::local_in_ancestor(world, e, owner, &mut self.chain)?;
        // O `body_type` sai do DONO: uma peça não tem tipo próprio, e é isso
        // que a torna uma forma a mais em vez de um segundo corpo.
        let owner_kind = world.get::<RigidBody>(owner).copied()?;
        // ⚠️ A escala é a de MUNDO da PEÇA (`t_part`), não a do dono: uma peça
        // pode ser escalada por conta própria, e o `scaled_shape` do W6 lê
        // exatamente esse campo.
        let col = &crate::forma_resolvida(world, e, col);
        let desc = crate::scale::body_desc(
            &owner_kind,
            col,
            &t_part,
            crate::GravityScale::NEUTRAL,
            [0.0, 0.0],
            0.0,
            false,
            false,
            false,
            false,
            None,
            0,
            // ⚠️ **Lido da PEÇA** (W-PartFace): as regras de combine são
            // propriedade do COLLIDER em rapier, e este passava
            // `default()` enquanto o `OneWayPlatform` logo abaixo já vinha
            // da entidade — a única assimetria da lista, e um descuido meu
            // na W-Compound. Com ela, o §11 de uma peça pintaria dois chips
            // que o solver ignora; sem ela, dois chips a menos que o irmão
            // sólido tem. A cura é o lado que faz o knob VIVER.
            world
                .get::<crate::MaterialCombine>(e)
                .copied()
                .unwrap_or_default(),
            None,
            world.get::<crate::OneWayPlatform>(e).is_some(),
            // Uma peça-sensor com componentes de área é uma ZONA (GEF02 B6).
            super::bodies::area_effect(world, e),
        );
        // ⚠️ **A POSE do `desc` é zerada, e é isso que torna a comparação
        // abaixo honesta.** `attach_part` lê só a metade de COLLIDER do
        // descriptor (docs dele); os campos `x`/`y`/`rotation` vêm da pose de
        // MUNDO da peça e portanto mudam a cada frame em que o dono se move.
        // Compará-los fazia a peça ser despendurada e re-pendurada em **133
        // de 301 ticks** numa descida de rampa — churn puro, invisível porque
        // o gate de repouso o escondia. Uma comparação tem de perguntar o que
        // o CONSUMIDOR pergunta.
        let desc = ph2d_physics::BodyDesc {
            x: 0.0,
            y: 0.0,
            rotation: 0.0,
            collide: super::bodies::collide_of(world, e),
            shape: self.outlines.resolve(
                &mut self.world,
                world.get::<crate::ColliderOutline>(e),
                desc.shape,
                t_part.scale,
            ),
            ..desc
        };
        Some((desc, [rel.translation.x, rel.translation.y, rel.rotation]))
    }

    /// Re-pendura toda peça depois de um rewind.
    ///
    /// ⚠️ **No MESMO chamado do `rebuild_from_rest`**, ao lado dos joints e das
    /// cordas, e pela frase que aquele já carrega: *um replay sem elas é outra
    /// simulação*. Uma peça esquecida aqui deixa o corpo composto voltar como
    /// meia peça — e o gate do Weston mostrou como isso fica CALADO (o alvo `0`
    /// replaya zero passos, então o primeiro Reset parece certo).
    pub(super) fn respawn_parts_from_rest(&mut self) {
        for p in self.parts.values_mut() {
            let Some(owner) = self.bodies.get(&p.owner) else {
                continue;
            };
            if let Some(h) = self.world.attach_part(owner.handle, &p.rest, p.local) {
                p.handle = h;
            }
        }
    }
}

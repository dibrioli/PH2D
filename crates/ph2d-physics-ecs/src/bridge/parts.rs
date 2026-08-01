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
    /// O corpo ancestral mais próximo de `e` que a ponte de fato construiu.
    ///
    /// ⚠️ **Pergunta ao `self.bodies` e não só ao componente**: uma entidade com
    /// `RigidBody` mas sem `Collider` não é um corpo para o solver (a `BodyQuery`
    /// exige os dois), e pendurar uma peça nela seria pendurá-la em nada.
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
        let at_rest = self.last_stepped == 0;
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
            let Some((owner, handle)) = self.owner_body(sim, e) else {
                continue;
            };
            self.part_seen.push(e);
            // As duas poses de MUNDO — a mesma razão do corpo: o solver não tem
            // hierarquia, e a pose relativa é o que ele quer.
            let Some(t_part) = space::world_transform(world, e, &mut self.chain) else {
                continue;
            };
            let Some(t_body) = space::world_transform(world, owner, &mut self.chain) else {
                continue;
            };
            let Some(rel) = ph2d_ecs::Transform::inverse_compose(t_body, t_part) else {
                continue;
            };
            // O `body_type` sai do DONO: uma peça não tem tipo próprio, e é isso
            // que a torna uma forma a mais em vez de um segundo corpo.
            let Some(owner_kind) = world.get::<RigidBody>(owner).copied() else {
                continue;
            };
            // ⚠️ A escala é a de MUNDO da PEÇA (`t_part`), não a do dono: uma peça
            // pode ser escalada por conta própria, e o `scaled_shape` do W6 lê
            // exatamente esse campo.
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
                None,
            );
            wanted.push((
                e,
                owner,
                handle,
                desc,
                [rel.translation.x, rel.translation.y, rel.rotation],
            ));
        }
        self.part_query = Some(q);

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
            }
        }
        self.part_seen = seen;

        for (e, owner, handle, desc, local) in wanted {
            match self.parts.get(&e) {
                // ⚠️ **Re-descrever é gateado em REPOUSO**, como o corpo: a pose
                // relativa é derivada das poses de MUNDO, e no play a do dono é
                // a que o solver escreveu. Sem o gate, cada frame re-penduraria a
                // peça na pose que a simulação acabou de produzir — e ela andaria
                // pelo corpo, que é o slide que o W-AnchorFollow curou no pino.
                Some(p) if p.owner == owner && p.rest == desc && p.local == local => {}
                Some(_) if !at_rest => {}
                _ => {
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

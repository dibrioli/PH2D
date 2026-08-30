//! **Um corpo com MAIS DE UMA FORMA** — os colliders extra de um corpo composto.
//!
//! Até aqui um corpo tinha exatamente um collider, e a `BodyQuery` da ponte dizia
//! isso no TIPO (`RigidBody` **e** `Collider` na mesma entidade). Um artista que
//! desenha um "L" — um braço horizontal e uma perna pendurada — recebia metade da
//! peça sem física nenhuma, **em silêncio**: medido, a perna atravessa o chão.
//!
//! rapier já sabe fazer isto: `insert_with_parent` prende quantos colliders se
//! quiser ao mesmo corpo, e a massa deles soma. O que faltava era a AUTORIA, e a
//! resposta dela é a hierarquia que o editor já tem (ADR-0110) — o
//! `CollisionShape2D` do Godot, com a nossa árvore no lugar da dele.
//!
//! # A porta reusa o `build_collider`, e é isso que importa
//!
//! Uma peça é descrita por um [`BodyDesc`] do qual **só a metade de COLLIDER é
//! lida** — `shape`, `density`, `mass_override`, `restitution`, `friction`,
//! `is_sensor`, `one_way`, `material`, `offset` — pela MESMA função que um corpo
//! usa. Um descriptor próprio para peças seria a segunda resposta a *"o que é um
//! collider?"*, e ela divergiria na primeira propriedade nova: quem a
//! acrescentasse ao `build_collider` a daria a corpos e não a peças, sem erro e
//! sem warning.
//!
//! # A pose LOCAL e o offset COMPÕEM, não competem
//!
//! Uma peça tem duas colocações e as duas são autoradas: **onde ela está no
//! corpo** (o `Transform` dela relativo ao dono, que a ponte deriva) e **o offset
//! do próprio collider** (o campo que o W-Offset acrescentou, no frame da peça).
//! O collider nasce do `build_collider` já com o offset em `translation`, e esta
//! porta pré-multiplica a pose local por cima — `L * O`, nessa ordem, porque o
//! offset é medido no frame da PEÇA. Sobrescrever a translação em vez de compor
//! apagaria o offset em silêncio, que é a classe de bug que esta linha já pagou.

use crate::rmath::{Pose, Rotation, Vector};
use rapier2d::dynamics::RigidBodyHandle;
use rapier2d::geometry::ColliderHandle;

use super::desc::BodyDesc;
use super::{PhysicsWorld, collider_build};

impl PhysicsWorld {
    /// **Pendura mais um collider no corpo `body`**, na pose LOCAL `local`
    /// (`[x, y, rotação em radianos]`, no frame do corpo).
    ///
    /// `None` se o corpo não existe — a mesma recusa honesta de
    /// [`PhysicsWorld::spawn_joint`], e pelo mesmo motivo: uma peça pendurada em
    /// nada é um handle que ninguém pode usar.
    ///
    /// ⚠️ **A massa SOMA.** rapier recomputa as propriedades de massa do corpo a
    /// partir de todos os colliders dele, então uma peça com densidade torna o
    /// corpo mais pesado e move o centro de massa — que é exatamente o que um
    /// corpo composto significa, e o que distingue isto de dois corpos ligados
    /// por um Weld (duas massas que o solver pode separar).
    pub fn attach_part(
        &mut self,
        body: RigidBodyHandle,
        desc: &BodyDesc,
        local: [f32; 3],
    ) -> Option<ColliderHandle> {
        self.bodies.get(body)?;
        let mut collider = collider_build::build_collider(desc);
        // O que o `build_collider` deixou em `position` é o OFFSET da peça, no
        // frame dela. A pose local entra POR FORA (docs do módulo).
        let offset = *collider.position();
        // ⚠️ O 1.º argumento era um `Translation2` (daí o `.into()`); no `Pose` do glamx ele é
        // um `Vector` directo — a translação deixou de ter tipo próprio.
        let place = Pose::from_parts(Vector::new(local[0], local[1]), Rotation::new(local[2]));
        collider.set_position(place * offset);
        let handle = self
            .colliders
            .insert_with_parent(collider, body, &mut self.bodies);
        self.stamp_layer(handle, desc.layer as usize);
        Some(handle)
    }

    /// Tira uma peça do corpo. `wake` acorda o dono — uma peça que some muda a
    /// massa dele, e um corpo dormindo não recomputaria nada.
    pub fn detach_part(&mut self, handle: ColliderHandle) {
        let owner = self.colliders.get(handle).and_then(|c| c.parent());
        self.colliders
            .remove(handle, &mut self.island_manager, &mut self.bodies, true);
        // ⚠️ **rapier NÃO recomputa a massa no `remove`, e o gate mediu:** o corpo
        // ficava em 1,6000 kg depois de perder uma peça de 0,8 — a massa da peça
        // que já não existe. Um corpo pesando o que ele não tem mais é a classe de
        // estado velho que ninguém vê: a cena parece certa e cai errado.
        if let Some(owner) = owner
            && let Some(b) = self.bodies.get_mut(owner)
        {
            b.recompute_mass_properties_from_colliders(&self.colliders);
        }
    }

    /// A pose LOCAL com que uma peça está pendurada, ou `None` se o handle morreu.
    ///
    /// Existe para os gates: *"a peça está onde o artista a pôs"* é a pergunta que
    /// separa um corpo composto de duas formas empilhadas por acaso.
    #[must_use]
    pub fn part_local(&self, handle: ColliderHandle) -> Option<[f32; 3]> {
        let c = self.colliders.get(handle)?;
        let p = c.position_wrt_parent()?;
        Some([p.translation.x, p.translation.y, p.rotation.angle()])
    }
}

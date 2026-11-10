//! **O push POR-TIQUE das formas** — o irmão do [`super::joint_drive`] para os corpos e as peças.
//!
//! O `reconcile_structure`/`reconcile_parts` relê as formas uma vez por DISPATCH, e os laços de play
//! e de replay dão N passos dentro dele. Sem este passe, uma escala animada (que a cena escreve no
//! `Transform` a cada tique, `SceneAtTick`) chegava ao solver uma vez por quadro, e um replay depois
//! de um salto corria com as formas do CHECKPOINT: o semeio do ring repõe os colisores daquele tique
//! enquanto o memo (`rest`) já descreve o quadro pedido, e nada voltava a escrevê-las (GEF02 onda 7).
//!
//! Reafina no lugar (`retune_body`/`retune_part`), nunca recria: o ring vale.

use ph2d_ecs::SimWorld;

use super::PhysicsBridge;

impl PhysicsBridge {
    /// Empurrar ao solver as formas que a cena tem neste tique, onde diferem do memo. Depois de um
    /// semeio do ring o memo é o do CHECKPOINT (`seed_player_states`), não o do quadro: é isso que o
    /// torna honesto sem reafinar às cegas. Só os corpos e peças que a ponte já tem: entrar ou sair é
    /// estrutural, do `reconcile_*` do próximo quadro.
    pub(super) fn drive_shapes(&mut self, sim: &SimWorld) {
        let world = sim.world();
        let mut corpos = std::mem::take(&mut self.seen);
        corpos.clear();
        corpos.extend(self.bodies.keys().copied());
        for &e in &corpos {
            let Some(rb) = world.get::<crate::RigidBody>(e) else {
                continue;
            };
            let (_, Some(desc)) = self.desc_do_corpo(world, e, rb) else {
                continue;
            };
            let Some(b) = self.bodies.get_mut(&e) else {
                continue;
            };
            if super::bodies::sem_pose(b.rest) == super::bodies::sem_pose(desc) {
                continue;
            }
            if self.world.retune_body(b.handle, &desc) {
                b.rest = super::bodies::com_a_pose_de(b.rest, desc);
            }
        }
        corpos.clear();
        corpos.extend(self.parts.keys().copied());
        for &e in &corpos {
            let (Some(col), Some(owner)) = (
                world.get::<crate::Collider>(e),
                self.parts.get(&e).map(|p| p.owner),
            ) else {
                continue;
            };
            let Some((desc, local)) = self.desc_da_peca(world, e, owner, col) else {
                continue;
            };
            let Some(p) = self.parts.get_mut(&e) else {
                continue;
            };
            if p.rest == desc && p.local == local {
                continue;
            }
            if self.world.retune_part(p.handle, &desc, local) {
                p.rest = desc;
                p.local = local;
            }
        }
        self.seen = corpos;
    }
}

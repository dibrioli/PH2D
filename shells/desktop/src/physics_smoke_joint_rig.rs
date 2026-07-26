//! **A cena do RIG QUE ANDA JUNTO** (`PH2D_PHYSICS_SMOKE=51`, W-JG).
//!
//! A W-AnchorFollow tornou a âncora **body-local**, e com isso mover UM corpo de
//! um par jointado deixou de mover a âncora do vizinho: o joint nasce esticado e
//! o Play o resolve com um puxão. Esta cena é sobre a cura — arrastar um elo em
//! repouso arrasta o **rig**.
//!
//! Três armações, cada uma respondendo uma pergunta diferente (e a segunda é o
//! CONTROLE da primeira — um "andou junto" só quer dizer alguma coisa ao lado de
//! um "não andou"):
//!
//! - **CORRENTE**: pegar o elo do MEIO leva os três, e o gancho estático fica.
//! - **GÊMEOS**: dois pêndulos no MESMO gancho — arrastar um **não** move o
//!   outro. O gancho é uma parede entre eles, não um fio.
//! - **PAR LIVRE**: dois corpos pinados sem âncora nenhuma — vão juntos por
//!   qualquer um deles, e o Play os derruba de onde você os pôs.
//!
//! ⚠️ **O Alt é o escape**, e ele tem metade visível própria: com Alt o elo anda
//! sozinho e o **segmento âmbar do joint estica** na tela. Não há chrome novo
//! nesta wave — o rig andando junto É o que se vê, e o que fica para trás se vê
//! pelo mesmo desenho.
//!
//! Os números da mensagem saíram da sonda `probe_smoke_51` (`joint_rig_drag`),
//! rodada sobre ESTAS armações antes de a mensagem ser escrita.

use crate::physics_smoke::spawn_floor;
use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform, World, stable_name_id};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, PhysicsJoint, RigidBody};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

/// As três armações, sem chão e sem mensagem — a MESMA construção que a sonda
/// headless mede, para os números da mensagem serem sobre a cena que o artista
/// abre e não sobre uma parecida.
pub(crate) fn spawn_rigs(world: &mut World) {
    let grey = [0.75, 0.75, 0.8, 1.0];
    let hot = [0.95, 0.6, 0.2, 1.0];
    let cool = [0.4, 0.8, 0.95, 1.0];

    let hook = |world: &mut World, name: &str, at: [f32; 2]| {
        world.spawn((
            Transform::from_translation(Vec2::new(at[0], at[1])),
            Sprite::atlas(WHITE_TILE_KEY, [0.16, 0.16], grey),
            Name::new(name.to_string()),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.08 },
                ..Collider::default()
            },
        ));
    };
    let link = |world: &mut World, name: &str, at: [f32; 2], rgba: [f32; 4]| {
        world.spawn((
            Transform::from_translation(Vec2::new(at[0], at[1])),
            Sprite::atlas(WHITE_TILE_KEY, [0.5, 0.5], rgba),
            Name::new(name.to_string()),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.25 },
                ..Collider::default()
            },
        ));
    };
    // O nome do joint é o do par (a convenção que a W-J8 tornou automática), então
    // a Hierarquia já diz quem está preso em quem sem abrir o Inspector.
    let pin = |world: &mut World, a: &str, b: &str, at: [f32; 2]| {
        world.spawn((
            Transform::from_translation(Vec2::new(at[0], at[1])),
            Name::new(format!("{a} : {b}")),
            PhysicsJoint {
                body_a: stable_name_id(a),
                body_b: stable_name_id(b),
                ..PhysicsJoint::default()
            },
        ));
    };

    // ── CORRENTE (esquerda): gancho estático + três elos pendurados.
    hook(world, "Chain Hook", [-5.5, 8.0]);
    link(world, "Chain L1", [-5.5, 7.2], cool);
    link(world, "Chain L2", [-5.5, 6.4], hot);
    link(world, "Chain L3", [-5.5, 5.6], cool);
    pin(world, "Chain Hook", "Chain L1", [-5.5, 7.6]);
    pin(world, "Chain L1", "Chain L2", [-5.5, 6.8]);
    pin(world, "Chain L2", "Chain L3", [-5.5, 6.0]);

    // ── GÊMEOS (meio): UM gancho, dois pêndulos. O controle.
    hook(world, "Twin Hook", [0.0, 8.0]);
    link(world, "Twin Left", [-0.9, 7.0], hot);
    link(world, "Twin Right", [0.9, 7.0], cool);
    pin(world, "Twin Hook", "Twin Left", [0.0, 8.0]);
    pin(world, "Twin Hook", "Twin Right", [0.0, 8.0]);

    // ── PAR LIVRE (direita): sem âncora nenhuma.
    link(world, "Free Left", [5.0, 7.0], hot);
    link(world, "Free Right", [6.0, 7.0], cool);
    pin(world, "Free Left", "Free Right", [5.5, 7.0]);
}

impl crate::App {
    /// **Cena 51 (W-JG).** Uma corrente, dois gêmeos e um par livre, PAUSADOS.
    pub(crate) fn physics_smoke_joint_rig(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        spawn_floor(gfx.sim.world_mut());
        spawn_rigs(gfx.sim.world_mut());

        eprintln!(
            "[physics-smoke 51] Tres rigs em REPOUSO. Arrastar um corpo arrasta o RIG.\n  \
               1. Aperte B (mostra os joints em ambar).\n  \
               2. CORRENTE (esquerda): arraste 'Chain L2' (o elo laranja do meio).\n     \
                  Os TRES elos andam juntos; o gancho cinza NAO sai do lugar.\n     \
                  (medido: pegar L2 leva L1 e L3 -- 2 corpos a mais, o gancho fora)\n  \
               3. Agora com ALT apertado, arraste 'Chain L2' de novo: so ELE anda,\n     \
                  e o segmento ambar do joint ESTICA. Esse e o escape.\n  \
               4. GEMEOS (meio): arraste 'Twin Left'. 'Twin Right' NAO se mexe --\n     \
                  o gancho estatico e uma parede entre os dois, nao um fio.\n     \
                  (medido: pegar Twin Left leva 0 corpos a mais)\n  \
               5. PAR LIVRE (direita): arraste 'Free Left' -- 'Free Right' vem junto\n     \
                  (medido: 1 corpo a mais). Solte, aperte Play: o par cai JUNTO,\n     \
                  sem puxao, de onde voce o deixou.\n\
             Ctrl+Z desfaz cada arrasto em UM passo (o rig inteiro por gesto)."
        );
    }
}

//! **A cena do RIG QUE ANDA JUNTO** (`PH2D_PHYSICS_SMOKE=51`, W-JG).
//!
//! A W-AnchorFollow tornou a âncora **body-local**, e com isso mover UM corpo de
//! um par jointado deixou de mover a âncora do vizinho: o joint nasce esticado e
//! o Play o resolve com um puxão. Esta cena é sobre a cura — arrastar um elo em
//! repouso arrasta o **rig**.
//!
//! **O rig inteiro anda com ALT apertado** (decisão do Enio, 2026-07-26); sem
//! Alt anda só o corpo que se pegou. As duas metades têm leitura própria, e o
//! CONTROLE de cada rig é ele mesmo sem o modificador: sem Alt o **segmento
//! âmbar do joint ESTICA** na tela. Não há chrome novo nesta wave — o rig
//! andando junto É o que se vê, e o que fica para trás se vê pelo mesmo desenho.
//!
//! Três armações, cada uma exercitando uma parte da política *"a cadeia inteira,
//! independente do tipo"*:
//!
//! - **CORRENTE**: pegar o elo do MEIO leva os três **e o gancho ESTÁTICO**.
//! - **GÊMEOS**: um gancho **KINEMATIC** com dois pêndulos — o grafo se
//!   RAMIFICA, então levar um pêndulo leva o gancho *e o irmão*. É o preço
//!   honesto de "independente do tipo" quando a cadeia se abre.
//! - **PAR LIVRE**: dois corpos pinados sem âncora nenhuma — vão juntos por
//!   qualquer um deles, e o Play os derruba de onde você os pôs.
//!
//! ⚠️ Os três tipos de corpo aparecem de propósito (Static · Kinematic ·
//! Dynamic): uma cena só com Static provaria metade da política.
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

    // O tipo é PARÂMETRO porque a cena tem de exercitar os três: a política do
    // arrasto é *independente do tipo*, e uma cena só com Static provaria metade.
    let hook = |world: &mut World, name: &str, at: [f32; 2], kind: BodyKind| {
        world.spawn((
            Transform::from_translation(Vec2::new(at[0], at[1])),
            Sprite::atlas(WHITE_TILE_KEY, [0.16, 0.16], grey),
            Name::new(name.to_string()),
            RigidBody { kind },
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

    // ── CORRENTE (esquerda): gancho ESTÁTICO + três elos pendurados.
    hook(world, "Chain Hook", [-5.5, 8.0], BodyKind::Static);
    link(world, "Chain L1", [-5.5, 7.2], cool);
    link(world, "Chain L2", [-5.5, 6.4], hot);
    link(world, "Chain L3", [-5.5, 5.6], cool);
    pin(world, "Chain Hook", "Chain L1", [-5.5, 7.6]);
    pin(world, "Chain L1", "Chain L2", [-5.5, 6.8]);
    pin(world, "Chain L2", "Chain L3", [-5.5, 6.0]);

    // ── GÊMEOS (meio): UM gancho KINEMATIC, dois pêndulos. O grafo se RAMIFICA
    // aqui, e o terceiro tipo entra na cena.
    hook(world, "Twin Hook", [0.0, 8.0], BodyKind::Kinematic);
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
            "[physics-smoke 51] Tres rigs em REPOUSO. ALT+arrastar um corpo arrasta o RIG.\n  \
               1. Aperte B (mostra os joints em ambar).\n  \
               2. CORRENTE (esquerda): arraste 'Chain L2' (o elo laranja do meio) SEM Alt.\n     \
                  So ELE anda, e o segmento ambar do joint ESTICA. Esse e o default.\n  \
               3. Agora com ALT apertado, arraste 'Chain L2': andam os tres elos E o\n     \
                  gancho cinza -- a cadeia inteira, independente do tipo.\n     \
                  (medido: leva 3 corpos a mais -- Chain Hook + L1 + L3)\n  \
               4. GEMEOS (meio): o gancho e KINEMATIC e o grafo se RAMIFICA nele.\n     \
                  Alt+arraste 'Twin Left': vem o gancho E o irmao 'Twin Right'.\n     \
                  (medido: leva 2 corpos a mais). Sem Alt, so o pendulo que pegou.\n  \
               5. PAR LIVRE (direita): Alt+arraste 'Free Left' -- 'Free Right' vem junto\n     \
                  (medido: 1 corpo a mais). Solte, aperte Play: o par cai JUNTO,\n     \
                  sem puxao, de onde voce o deixou.\n\
             Ctrl+Z desfaz cada arrasto em UM passo (o rig inteiro por gesto)."
        );
    }
}

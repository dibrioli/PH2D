//! ⭐ **A LEI DA F3, num sítio só: o Inspector mostra o que o objeto TEM** (ADR-0166).
//!
//! Até esta fase, **todas** as seções abaixo eram publicadas para qualquer entidade com um
//! `Transform`, e o Inspector de um objeto vazio mostrava doze seções de zeros. *Ausência de
//! autoria não é «zeros»*, e a rota para cada feature era uma **face vazia** dentro da seção que
//! ela própria escondia.
//!
//! # Porque é UM ficheiro e não uma linha em cada builder
//!
//! Cada `build_*_info` sabe gatear-se; nenhum deles sabe que **existe uma lei**. Este ficheiro é a
//! lei: uma varredura em que cada linha afirma as **duas** metades sobre uma seção —
//!
//! | metade | o defeito que ela apanha |
//! |---|---|
//! | sem o componente ⇒ **sem seção** | a face vazia a voltar; doze seções de zeros num objeto novo |
//! | com o componente ⇒ **com seção** | a poda a ir longe demais: o artista anexa e **nada aparece** |
//!
//! ⚠️ **A segunda metade é a que mais custou.** Três seções acenderam-se com um componente que a
//! primeira redacção não listava — o `Collider` sozinho (uma PEÇA de um corpo ancestral), o
//! `AnchorMount` (quem MONTA numa âncora do pai, contra quem as OFERECE) e o `SpriteAnimator` (o
//! transporte, contra a biblioteca). Em cada caso a seção tem duas metades que pertencem a lados
//! opostos da mesma relação, e gatear numa só apagava a UI do outro lado.

use ph2d_ecs::{Entity, SimWorld, Transform, World};

/// Uma seção, e um componente que a ACENDE.
struct Case {
    section: &'static str,
    component: &'static str,
    /// O objeto BASE — o mínimo em que a seção poderia existir, e ainda não existe.
    base: fn(&mut World) -> Entity,
    /// Anexa o componente do caso.
    attach: fn(&mut World, Entity),
    /// A seção está viva?
    live: fn(&SimWorld, u64) -> bool,
}

fn plain(w: &mut World) -> Entity {
    w.spawn((Transform::IDENTITY, ph2d_ecs::Name::new("Object")))
        .id()
}

fn sprite(w: &mut World) -> Entity {
    w.spawn((
        Transform::IDENTITY,
        ph2d_ecs::Name::new("Image"),
        ph2d_render::Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
    ))
    .id()
}

fn body(w: &mut World) -> Entity {
    w.spawn((
        Transform::IDENTITY,
        ph2d_ecs::Name::new("Body"),
        ph2d_physics_ecs::RigidBody::default(),
    ))
    .id()
}

fn ordering(sim: &SimWorld, b: u64) -> bool {
    crate::render_loop::inspector_presence_probe::ordering(sim.world(), b)
}
fn sampling(sim: &SimWorld, b: u64) -> bool {
    crate::render_loop::inspector_presence_probe::sampling(sim.world(), b)
}
fn blend(sim: &SimWorld, b: u64) -> bool {
    crate::render_loop::inspector_presence_probe::blend(sim.world(), b)
}
fn slice(sim: &SimWorld, b: u64) -> bool {
    crate::render_loop::inspector_presence_probe::slice(sim.world(), b)
}
fn visibility_section(sim: &SimWorld, b: u64) -> bool {
    crate::render_loop::inspector_presence_probe::visibility_section(sim.world(), b)
}
fn anchors(sim: &SimWorld, b: u64) -> bool {
    crate::render_loop::inspector_presence_probe::anchors(sim.world(), b)
}
fn anim(sim: &SimWorld, b: u64) -> bool {
    crate::render_loop::inspector_presence_probe::anim(sim.world(), b)
}
fn physics(sim: &SimWorld, b: u64) -> bool {
    crate::render_loop::inspector_presence_probe::physics(sim.world(), b)
}
fn player(sim: &SimWorld, b: u64) -> bool {
    crate::render_loop::inspector_presence_probe::player(sim, b)
}

const CASES: &[Case] = &[
    Case {
        section: "§7 Ordering",
        component: "ZIndexOverride",
        base: plain,
        attach: |w, e| {
            w.entity_mut(e).insert(ph2d_ecs::ZIndexOverride(3));
        },
        live: ordering,
    },
    Case {
        section: "§7 Ordering",
        component: "YSort",
        base: plain,
        attach: |w, e| {
            w.entity_mut(e).insert(ph2d_ecs::YSort::default());
        },
        live: ordering,
    },
    Case {
        section: "§9 Sampling",
        component: "TextureFilter",
        base: sprite,
        attach: |w, e| {
            w.entity_mut(e)
                .insert(ph2d_ecs::TextureFilter(ph2d_ecs::FilterMode::Nearest));
        },
        live: sampling,
    },
    Case {
        section: "§9 Sampling",
        component: "UvTransform",
        base: sprite,
        attach: |w, e| {
            w.entity_mut(e).insert(ph2d_ecs::UvTransform::default());
        },
        live: sampling,
    },
    Case {
        section: "§10 Blend",
        component: "BlendMode",
        base: sprite,
        attach: |w, e| {
            w.entity_mut(e).insert(ph2d_ecs::BlendMode::default());
        },
        live: blend,
    },
    // ⚠️ **A §8 ESCAPOU à 1.ª poda e foi o smoke do Enio que a apanhou** — a lista de seções do
    // plano não a nomeava. ⛔ A caixa «Visible» NÃO está aqui de propósito: ela é chrome ao lado do
    // nome, tem snapshot próprio, e vale para todo objeto.
    Case {
        section: "§8 Visibility",
        component: "ClipChildren",
        base: plain,
        attach: |w, e| {
            w.entity_mut(e).insert(ph2d_ecs::ClipChildren::default());
        },
        live: visibility_section,
    },
    Case {
        section: "§8 Visibility",
        component: "OnScreenEnabler",
        base: plain,
        attach: |w, e| {
            w.entity_mut(e).insert(ph2d_ecs::OnScreenEnabler::default());
        },
        live: visibility_section,
    },
    Case {
        section: "§8 Visibility",
        component: "Mask2D",
        base: sprite,
        attach: |w, e| {
            w.entity_mut(e).insert(ph2d_ecs::Mask2D::default());
        },
        live: visibility_section,
    },
    Case {
        section: "§5 9-Slice",
        component: "SliceNine",
        base: sprite,
        attach: |w, e| {
            w.entity_mut(e).insert(ph2d_ecs::SliceNine::INERT);
        },
        live: slice,
    },
    Case {
        section: "§12 Anchors",
        component: "NamedAnchorList",
        base: sprite,
        attach: |w, e| {
            w.entity_mut(e).insert(ph2d_ecs::NamedAnchorList::default());
        },
        live: anchors,
    },
    // ⚠️ O OUTRO lado da montagem — quem ANDA numa âncora do pai, e não tem lista nenhuma.
    Case {
        section: "§12 Anchors",
        component: "AnchorMount",
        base: sprite,
        attach: |w, e| {
            w.entity_mut(e).insert(ph2d_ecs::AnchorMount::new("hand_r"));
        },
        live: anchors,
    },
    Case {
        section: "§11 Animation",
        component: "SpriteAnimations",
        base: sprite,
        attach: |w, e| {
            w.entity_mut(e)
                .insert(ph2d_ecs::SpriteAnimations::default());
        },
        live: anim,
    },
    // ⚠️ E o TRANSPORTE, que é a outra metade da §11.
    Case {
        section: "§11 Animation",
        component: "SpriteAnimator",
        base: sprite,
        attach: |w, e| {
            w.entity_mut(e).insert(ph2d_ecs::SpriteAnimator::default());
        },
        live: anim,
    },
    Case {
        section: "§11 Physics",
        component: "RigidBody",
        base: sprite,
        attach: |w, e| {
            w.entity_mut(e)
                .insert(ph2d_physics_ecs::RigidBody::default());
        },
        live: physics,
    },
    // ⚠️ Um `Collider` SEM corpo é uma PEÇA de um corpo ancestral, e a §11 tem uma face para ela.
    Case {
        section: "§11 Physics",
        component: "Collider",
        base: sprite,
        attach: |w, e| {
            w.entity_mut(e)
                .insert(ph2d_physics_ecs::Collider::default());
        },
        live: physics,
    },
    Case {
        section: "§14 Platform Player",
        component: "PlatformPlayer",
        base: body,
        attach: |w, e| {
            w.entity_mut(e)
                .insert(ph2d_physics_ecs::PlatformPlayer::default());
        },
        live: player,
    },
];

/// ⭐ **A LEI, nos dois sentidos, sobre as treze combinações.**
///
/// (Mutação: apagar qualquer `if !has_… { return None }` de um `build_*_info` ⇒ a metade de
/// AUSÊNCIA reprova naquela seção, com o nome dela na mensagem.)
#[test]
fn a_section_appears_if_and_only_if_one_of_its_components_is_there() {
    for c in CASES {
        let mut sim = SimWorld::new();
        let e = (c.base)(sim.world_mut());
        let bits = e.to_bits();
        assert!(
            !(c.live)(&sim, bits),
            "{} apareceu SEM o {} — a face vazia voltou",
            c.section,
            c.component
        );
        (c.attach)(sim.world_mut(), e);
        assert!(
            (c.live)(&sim, bits),
            "{} nao apareceu COM o {} — o artista anexa e nada acontece",
            c.section,
            c.component
        );
    }
}

/// ⚠️ **E a BASE continua a existir.** A poda não pode comer o `Transform` nem o `Name`: um objeto
/// vazio tem de mostrar **duas** seções, e não zero — senão o Inspector fica em branco e o `+` não
/// tem onde estar.
#[test]
fn an_empty_object_still_shows_transform_and_name() {
    let mut sim = SimWorld::new();
    let e = plain(sim.world_mut());
    assert!(sim.world().get::<Transform>(e).is_some());
    assert!(sim.world().get::<ph2d_ecs::Name>(e).is_some());
    // E nenhuma das treze acima.
    for c in CASES {
        assert!(
            !(c.live)(&sim, e.to_bits()),
            "um objeto vazio mostrou {}",
            c.section
        );
    }
}

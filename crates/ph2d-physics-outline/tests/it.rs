//! O produtor do «Outline» (W2c): argola com buraco, Animated Sprite pelas tags, imagem no CORPO,
//! a escrita pela fila (um passo de undo) e o tecto que sobe o Detail.

use ph2d_core::Vec2;
use ph2d_ecs::scene::{
    ComponentRegistry, EditorCommandQueue, apply_editor_commands, register_ecs_components,
};
use ph2d_ecs::{AnimationTag, ChildOf, SimWorld, SpriteAnimations, SpriteGrid, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderOutline, ColliderShape, PhysicsBridge, RigidBody,
    register_physics_components,
};
use ph2d_physics_outline::{Picture, frames_mask, picture_owner, retrace, trace, traced_frames};
use ph2d_render::Sprite;

/// RGBA `w × h`, opaco onde `on(x, y)`.
fn img(w: u32, h: u32, on: impl Fn(u32, u32) -> bool) -> Vec<u8> {
    (0..h)
        .flat_map(|y| (0..w).map(move |x| (x, y)))
        .flat_map(|(x, y)| [255, 255, 255, if on(x, y) { 255 } else { 0 }])
        .collect()
}

fn registry() -> ComponentRegistry {
    let mut reg = ComponentRegistry::new();
    register_ecs_components(&mut reg);
    register_physics_components(&mut reg);
    reg
}

/// Argola 32×32 px (buraco 16×16 no meio) num corpo Static de 4×4 m; a forma é a FILHA sem imagem.
fn ring_scene(sim: &mut SimWorld) -> (ph2d_ecs::Entity, ph2d_ecs::Entity) {
    let body = sim
        .world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Static,
            },
            Transform::default(),
            Sprite::atlas(0, [4.0, 4.0], [1.0; 4]),
        ))
        .id();
    let shape = sim
        .world_mut()
        .spawn((Collider::default(), Transform::default(), ChildOf(body)))
        .id();
    (body, shape)
}

fn ring_px() -> Vec<u8> {
    img(32, 32, |x, y| {
        !((8..24).contains(&x) && (8..24).contains(&y))
    })
}

/// (a)+(c)+(d) A argola dá anel de fora + buraco; a bola largada DENTRO do buraco cai até ao fundo dele
/// (y = −1 m + raio), não é cuspida pela caixa. A imagem é a do
/// CORPO. A escrita vai pela fila: nada muda antes de a aplicar, e os dois componentes vão juntos.
/// *Mutações: os buracos fora do traço ⇒ a bola é cuspida; `picture_owner` sem subir ao corpo ⇒ `None`.*
#[test]
fn a_argola_tem_buraco_e_a_bola_atravessa() {
    let reg = registry();
    let mut sim = SimWorld::new();
    let (body, shape) = ring_scene(&mut sim);
    assert_eq!(picture_owner(sim.world(), shape), Some(body));
    let px = ring_px();
    let q = EditorCommandQueue::new();
    let pic = |_, _| {
        Some(Picture {
            w: 32,
            h: 32,
            rgba: &px,
        })
    };
    let t = retrace(sim.world(), shape, pic, &q, &reg).expect("traça");
    assert_eq!(t.outline.rings.len(), 2, "anel de fora + buraco");
    assert!(!t.raised);
    assert_eq!(q.len(), 2, "Collider + ColliderOutline numa só leva");
    assert!(
        sim.world().get::<ColliderOutline>(shape).is_none(),
        "nada escrito à mão"
    );
    apply_editor_commands(sim.world_mut(), &q, &reg).expect("aplica");
    let c = sim
        .world()
        .get::<Collider>(shape)
        .copied()
        .expect("collider");
    assert_eq!(
        c.shape,
        ColliderShape::Outline {
            half_x: 2.0,
            half_y: 2.0
        }
    );
    assert_eq!(sim.world().get::<ColliderOutline>(shape), Some(&t.outline));
    let ball = sim
        .world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.3 },
                density: 1.0,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 0.5)),
        ))
        .id();
    let mut bridge = PhysicsBridge::new();
    for tick in 1..=240 {
        bridge.dispatch(&mut sim, true, tick);
    }
    let y = bridge.body_pose(ball).expect("corpo").1;
    assert!(
        (y + 0.7).abs() < 0.05,
        "a bola parou em y = {y}; no fundo do buraco é ≈ −0,7"
    );
}

/// (b) Animated Sprite: as células das tags (bolha esquerda, bolha direita) dão UM contorno que
/// cobre as duas; a célula do meio, fora das tags, fica de fora — e entra quando não há tags.
/// *Mutação: `traced_frames` ignorar as tags ⇒ o meio entra, vermelho.*
#[test]
fn o_animated_sprite_traca_a_uniao_das_celulas_das_tags() {
    let mut sim = SimWorld::new();
    let px = img(64, 16, |x, y| {
        let (cell, cx) = (x / 16, x % 16);
        (4..12).contains(&y)
            && match cell {
                0 => cx < 6,
                1 => cx >= 10,
                2 => (6..10).contains(&cx),
                _ => false,
            }
    });
    let grid = SpriteGrid {
        hframes: 4,
        vframes: 1,
        frame: 0,
        sheet: 0,
    };
    let mut tags = SpriteAnimations::new();
    tags.insert(AnimationTag::new("idle", 0, 1)).expect("tag");
    let e = sim
        .world_mut()
        .spawn((Sprite::atlas(0, [1.0, 1.0], [1.0; 4]), grid, tags))
        .id();
    let asked = ColliderOutline {
        detail_q4: 4,
        ..ColliderOutline::default()
    };
    let traced_at = |w: &ph2d_ecs::World, frames: &[(u8, u32)]| {
        let pic = |_| {
            Some(Picture {
                w: 64,
                h: 16,
                rgba: &px,
            })
        };
        let m = frames_mask(w, e, frames, pic, asked.alpha_cutoff).expect("máscara");
        let o = ph2d_outline_trace::Outline {
            rings: trace(&m, &asked).outline.rings.to_vec(),
        };
        let at = |x: i32| o.contains_q4([x * 16 + 8, 8 * 16 + 8]);
        (at(2), at(13), at(8))
    };
    let frames = traced_frames(sim.world(), e);
    assert_eq!(frames, vec![(0, 0), (0, 1)]);
    assert_eq!(
        traced_at(sim.world(), &frames),
        (true, true, false),
        "esquerda, direita, sem o meio"
    );
    sim.world_mut().entity_mut(e).remove::<SpriteAnimations>();
    let all = traced_frames(sim.world(), e);
    assert_eq!(all.len(), 4);
    assert_eq!(
        traced_at(sim.world(), &all),
        (true, true, true),
        "sem tags, todas as células"
    );
}

/// (c) A forma com imagem PRÓPRIA traça a dela; acima do corpo não se sobe.
#[test]
fn a_imagem_e_a_da_forma_ou_a_do_antepassado_ate_ao_corpo() {
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    let top = w.spawn(Sprite::atlas(1, [1.0, 1.0], [1.0; 4])).id();
    let body = w
        .spawn((
            RigidBody {
                kind: BodyKind::Static,
            },
            ChildOf(top),
        ))
        .id();
    let bare = w.spawn((Collider::default(), ChildOf(body))).id();
    let own = w
        .spawn((
            Collider::default(),
            ChildOf(body),
            Sprite::atlas(2, [1.0, 1.0], [1.0; 4]),
        ))
        .id();
    assert_eq!(picture_owner(sim.world(), own), Some(own));
    assert_eq!(
        picture_owner(sim.world(), bare),
        None,
        "o corpo sem imagem pára a subida"
    );
}

/// (e) Acima de `MAX_PIECES` o Detail sobe, e o `raised` vai para o cartão (W4).
/// *Mutação: `raised` sempre `false` ⇒ vermelho.*
#[test]
fn acima_do_tecto_o_detail_sobe_e_diz() {
    let px = img(64, 64, |x, y| (x + y) % 2 == 0);
    let m = ph2d_outline_trace::Mask::from_rgba8(64, 64, &px, 128);
    let asked = ColliderOutline {
        detail_q4: 1,
        ..ColliderOutline::default()
    };
    let t = trace(&m, &asked);
    assert!(t.raised, "o tecto tinha de subir o Detail");
    assert!(t.outline.detail_q4 > 1 || t.outline.min_island_px > 0);
}

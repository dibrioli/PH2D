//! P14b-1: o pivô da slice escolhida e a forma de colisão que segue a key da slice.

use ph2d_ecs::scene::incremental::{CaptureCache, capture_incremental};
use ph2d_ecs::scene::registry::{ComponentRegistry, register_ecs_components};
use ph2d_ecs::scene::save::WorldSnapshot;
use ph2d_ecs::scene::{snapshot_to_world, world_to_snapshot};
use ph2d_ecs::{
    ChildOf, Entity, Name, PixelSprite, SimWorld, SpriteGrid, Transform, TransformPropagationState,
    WorklistBuf,
};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, RigidBody};
use ph2d_pixel_cena::fixture::{body, heroi_p14};
use ph2d_pixel_cena::{
    SegueSlice, adiciona_collider_da_slice, animador_de_nascenca, deriva, deriva_formas, pivo_px,
    registar,
};
use ph2d_pixel_core::codec::{read_library, write_library};
use ph2d_pixel_core::{FrameId, Op, Sprite, apply, slice};

/// 16 px por metro: um píxel = 1/16 m.
const M: f64 = 1.0 / 16.0;

fn reg() -> ComponentRegistry {
    let mut r = ComponentRegistry::new();
    register_ecs_components(&mut r);
    ph2d_physics_ecs::register_physics_components(&mut r);
    registar(&mut r);
    r
}

fn quadro(s: &Sprite, i: usize) -> FrameId {
    s.frames()[i].id
}

#[test]
fn o_pivo_e_o_da_key_visivel() {
    let s = heroi_p14();
    assert_eq!(pivo_px(&s, quadro(&s, 0)), (8.0, 15.0), "quadro 0: os pés");
    assert_eq!(
        pivo_px(&s, quadro(&s, 2)),
        (8.0, 15.0),
        "a key do 0 vale até ao 3"
    );
    assert_eq!(
        pivo_px(&s, quadro(&s, 3)),
        (8.0, 15.0),
        "key menor, os mesmos pés"
    );
    assert_eq!(pivo_px(&s, quadro(&s, 4)), (6.0, 15.0), "o pivô muda no 4");
    let mut sem = s.clone();
    let op = slice::set_pivot(&sem, None).expect("muda");
    apply(&mut sem, op).expect("None");
    assert_eq!(
        pivo_px(&sem, quadro(&sem, 4)),
        (8.0, 8.0),
        "None ⇒ o centro"
    );
}

#[test]
fn apagar_a_slice_do_pivo_desfaz_num_passo() {
    let mut s = heroi_p14();
    let antes = s.clone();
    let id = body(&s);
    let op = slice::delete(&s, &[id]).expect("apaga");
    let inv = apply(&mut s, op).expect("apaga");
    assert_eq!(
        s.pivot_slice(),
        None,
        "o pivô saiu com a slice, no MESMO op"
    );
    assert!(s.slice(id).is_none());
    let redo = apply(&mut s, inv).expect("desfaz");
    assert_eq!(s, antes, "um passo devolve a slice E o pivô");
    apply(&mut s, redo).expect("refaz");
    assert_eq!(s.pivot_slice(), None);
    assert!(
        apply(&mut s, Op::PivotSlice(Some(id))).is_err(),
        "pivô numa slice que não existe"
    );
}

#[test]
fn o_pixel_formato_7_le_o_que_grava() {
    let s = heroi_p14();
    assert!(s.pivot_slice().is_some());
    let lib = std::collections::BTreeMap::from([(1_u64, s.clone())]);
    let bytes = write_library(&lib);
    assert_eq!(&bytes[11..13], &7_u16.to_le_bytes(), "MINOR 7");
    let back = read_library(&bytes).expect("lê");
    assert_eq!(back[&1], s);
    assert_eq!(back[&1].pivot_slice(), s.pivot_slice());
}

fn mundo() -> (SimWorld, Entity) {
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            Name::new("Heroi"),
            PixelSprite(1),
            animador_de_nascenca(),
        ))
        .id();
    (sim, e)
}

/// A forma da key `BODY_0` (8×14, centro (8, 9)) com o pivô nos pés (8, 15).
fn cuboide_0() -> (ColliderShape, [f32; 2]) {
    (
        ColliderShape::Cuboid {
            half_x: 0.25,
            half_y: 7.0 / 16.0,
        },
        [0.0, 6.0 / 16.0],
    )
}

/// `BODY_3` (6×10, centro (8, 11)), pés (8, 15).
fn cuboide_3() -> (ColliderShape, [f32; 2]) {
    (
        ColliderShape::Cuboid {
            half_x: 3.0 / 16.0,
            half_y: 5.0 / 16.0,
        },
        [0.0, 4.0 / 16.0],
    )
}

fn forma_de(sim: &SimWorld, f: Entity) -> (ColliderShape, [f32; 2]) {
    let c = sim.world().get::<Collider>(f).expect("collider");
    (c.shape, c.offset)
}

#[test]
fn o_collider_nasce_por_gesto() {
    let s = heroi_p14();
    let (mut sim, e) = mundo();
    let f = adiciona_collider_da_slice(sim.world_mut(), e, body(&s), &s, quadro(&s, 0), M)
        .expect("gesto");
    let w = sim.world();
    assert_eq!(
        w.get::<RigidBody>(e).map(|r| r.kind),
        Some(BodyKind::Dynamic)
    );
    assert_eq!(w.get::<ChildOf>(f).map(ChildOf::parent), Some(e));
    assert_eq!(w.get::<SegueSlice>(f).map(|g| g.slice), Some(body(&s).0));
    assert_eq!(w.get::<Name>(f).map(Name::as_str), Some("body"));
    assert_eq!(ph2d_physics_ecs::main_shape(w, e), Some(f));
    assert_eq!(forma_de(&sim, f), cuboide_0());
}

/// Toca o quadro `i` como o animador o deixa (a célula viva da grelha, a tocar).
fn no_quadro(sim: &mut SimWorld, e: Entity, i: u32) {
    sim.world_mut()
        .get_mut::<SpriteGrid>(e)
        .expect("grelha")
        .frame = i;
    let mut a = sim
        .world_mut()
        .get_mut::<ph2d_ecs::SpriteAnimator>(e)
        .expect("animador");
    a.playing = true;
}

#[test]
fn a_forma_segue_a_key_da_slice() {
    let s = heroi_p14();
    let (mut sim, e) = mundo();
    deriva(sim.world_mut(), |_| Some((&s, 1)));
    let f = adiciona_collider_da_slice(sim.world_mut(), e, body(&s), &s, quadro(&s, 0), M)
        .expect("gesto");
    let ed = quadro(&s, 0);
    no_quadro(&mut sim, e, 2);
    deriva_formas(sim.world_mut(), |_| Some((&s, ed)), M);
    assert_eq!(forma_de(&sim, f), cuboide_0(), "quadro 2: a key do 0");
    no_quadro(&mut sim, e, 3);
    deriva_formas(sim.world_mut(), |_| Some((&s, ed)), M);
    assert_eq!(forma_de(&sim, f), cuboide_3(), "quadro 3: a key menor");
}

#[test]
fn a_forma_derivada_nao_entra_no_undo_nem_no_ficheiro() {
    let r = reg();
    let s = heroi_p14();
    let (mut sim, e) = mundo();
    deriva(sim.world_mut(), |_| Some((&s, 1)));
    let f = adiciona_collider_da_slice(sim.world_mut(), e, body(&s), &s, quadro(&s, 0), M)
        .expect("gesto");
    let ed = quadro(&s, 0);
    // O animador a tocar é do artista (grava-se): liga-se ANTES da 1.ª foto; depois só o quadro anda.
    no_quadro(&mut sim, e, 2);
    deriva_formas(sim.world_mut(), |_| Some((&s, ed)), M);
    let mut cache = CaptureCache::new();
    let mut out = WorldSnapshot::default();
    capture_incremental(sim.world_mut(), &mut cache, &r, &mut out).expect("1.ª");

    no_quadro(&mut sim, e, 3);
    deriva_formas(sim.world_mut(), |_| Some((&s, ed)), M);
    assert_eq!(forma_de(&sim, f), cuboide_3());
    let rep = capture_incremental(sim.world_mut(), &mut cache, &r, &mut out).expect("2.ª");
    assert_eq!(
        rep.reserialized, 0,
        "a forma derivada marcou a filha no undo"
    );

    let snap = foto(&mut sim, &r);
    let seg = r.get_by_name("ph2d::pixel::SegueSlice").expect("registado");
    assert!(
        snap.entities
            .iter()
            .flat_map(|row| row.components.iter())
            .any(|c| c.type_id == seg.type_id),
        "a SegueSlice grava-se"
    );
}

fn foto(sim: &mut SimWorld, r: &ComponentRegistry) -> WorldSnapshot {
    let mut prop = TransformPropagationState::new(sim.world_mut());
    let mut wl = WorklistBuf::default();
    let mut snap = WorldSnapshot::default();
    world_to_snapshot(sim.world_mut(), &mut prop, &mut wl, r, &mut snap).expect("foto");
    snap
}

#[test]
fn a_forma_volta_depois_do_stop() {
    let r = reg();
    let s = heroi_p14();
    let (mut sim, e) = mundo();
    deriva(sim.world_mut(), |_| Some((&s, 1)));
    adiciona_collider_da_slice(sim.world_mut(), e, body(&s), &s, quadro(&s, 0), M).expect("gesto");
    let snap = foto(&mut sim, &r);

    let mut novo = SimWorld::new();
    snapshot_to_world(novo.world_mut(), &snap, &r).expect("respawn");
    let mut q = novo
        .world_mut()
        .query_filtered::<Entity, bevy_ecs::query::With<SegueSlice>>();
    let f = q.single(novo.world()).expect("a filha voltou");
    assert_eq!(
        forma_de(&novo, f).0,
        ColliderShape::default(),
        "o ficheiro leva a forma no ponto neutro"
    );
    deriva(novo.world_mut(), |_| Some((&s, 1)));
    deriva_formas(novo.world_mut(), |_| Some((&s, quadro(&s, 0))), M);
    assert_eq!(forma_de(&novo, f), cuboide_0(), "a deriva repôs a forma");
}

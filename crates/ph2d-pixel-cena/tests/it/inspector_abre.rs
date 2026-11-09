//! P14e: o import do Sprite Inspector abre o que o Pixel grava — campo a campo com o Pixel Sprite.

use ph2d_ecs::{Entity, Name, PixelSprite, SimWorld, SpriteAnimations, SpriteGrid, Transform};
use ph2d_physics_ecs::{Collider, FormaDesligada};
use ph2d_pixel_cena::fixture::{body, heroi_p14};
use ph2d_pixel_cena::{
    adiciona_collider_da_slice, animador_de_nascenca, deriva, deriva_formas, importa_ase, pivo_px,
    slice_e_collider,
};
use ph2d_pixel_core::{Purpose, compose_normal, compose_window};

const M: f64 = 1.0 / 16.0;

fn pixel_sprite() -> (SimWorld, Entity) {
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

/// O `.ase` que o Pixel grava deste documento: a marca de collider vem do MUNDO (D2).
fn grava_do_mundo(s: &ph2d_pixel_core::Sprite, w: &mut ph2d_ecs::World) -> Vec<u8> {
    let marcadas = ph2d_pixel_cena::slices_com_collider(w, 1);
    ph2d_pixel_io::ase::escrever(&ph2d_pixel_cena::com_colliders(s, &marcadas))
        .expect("grava")
        .0
}

/// D2: Add Collider ⇒ gravar ⇒ reabrir dá a slice marcada; sem collider, não marca; o documento
/// nunca leva a marca (nenhum passo de undo). *Mutação: `slices_com_collider` vazio ⇒ vermelho.*
#[test]
fn o_add_collider_marca_a_slice_ao_gravar() {
    let s = heroi_p14();
    let id = body(&s);
    let (mut sim, e) = pixel_sprite();
    let reaberta = |b: &[u8]| {
        let (r, _) = ph2d_pixel_io::ase::ler(b).expect("reabre");
        slice_e_collider(r.slices().iter().find(|c| c.name == "body").expect("body"))
    };
    assert!(
        !reaberta(&grava_do_mundo(&s, sim.world_mut())),
        "sem collider não marca"
    );
    adiciona_collider_da_slice(sim.world_mut(), e, id, &s, s.frames()[0].id, M)
        .expect("Add Collider");
    assert!(
        reaberta(&grava_do_mundo(&s, sim.world_mut())),
        "com collider marca"
    );
    assert!(
        !slice_e_collider(s.slice(id).expect("body")),
        "o documento não leva a marca"
    );
    // Outro documento no mesmo mundo não herda a marca.
    assert!(ph2d_pixel_cena::slices_com_collider(sim.world_mut(), 2).is_empty());
}

#[test]
fn o_inspector_abre_o_que_o_pixel_grava() {
    let s = heroi_p14();
    // O Pixel Sprite da MESMA fixture, pelas portas do mundo.
    let (mut sim, e) = pixel_sprite();
    deriva(sim.world_mut(), |_| Some((&s, 1)));
    let f = adiciona_collider_da_slice(sim.world_mut(), e, body(&s), &s, s.frames()[0].id, M)
        .expect("Add Collider");
    let imp = importa_ase(&grava_do_mundo(&s, sim.world_mut()), "heroi_p14", M).expect("import");

    // Tags.
    let clips_pixel = sim
        .world()
        .get::<SpriteAnimations>(e)
        .expect("clips")
        .clone();
    assert_eq!(imp.clips, clips_pixel, "tags");
    assert_eq!(imp.clips.iter().count(), 3);

    assert_eq!((imp.largura, imp.altura), (16, 16));
    let n = s.frames().len();
    assert_eq!(imp.cor.len(), n);
    assert_eq!(imp.pivos_px.len(), n);
    assert_eq!(imp.colliders.len(), 1, "um collider: body");
    let normal = imp.normal.as_ref().expect("a normal ligada");
    let ed = s.frames()[0].id;
    for (i, fr) in s.frames().iter().enumerate() {
        // Pivô por quadro.
        assert_eq!(imp.pivos_px[i], pivo_px(&s, fr.id), "pivô do quadro {i}");
        // Cor e normal.
        assert_eq!(
            imp.cor[i],
            compose_window(&s, fr.id, None, s.bounds(), Purpose::Export),
            "cor do quadro {i}"
        );
        assert_eq!(normal[i], compose_normal(&s, fr.id), "normal do quadro {i}");
        // Collider: o que o `deriva_formas` do Pixel escreve na filha neste quadro.
        {
            let w = sim.world_mut();
            w.get_mut::<SpriteGrid>(e).expect("grelha").frame = i as u32;
            w.get_mut::<ph2d_ecs::SpriteAnimator>(e)
                .expect("animador")
                .playing = true;
        }
        deriva_formas(sim.world_mut(), |_| Some((&s, ed)), M);
        let w = sim.world();
        let c = w.get::<Collider>(f).expect("filha");
        let esperado = (w.get::<FormaDesligada>(f).is_none()).then_some((c.shape, c.offset));
        let veio = imp.colliders[0].por_quadro[i]
            .as_ref()
            .map(|c| (c.shape, c.offset));
        assert_eq!(veio, esperado, "collider do quadro {i}");
    }
    // A fixture contém o fenómeno: o pivô muda no 4 e a forma no 3.
    assert_ne!(imp.pivos_px[3], imp.pivos_px[4]);
    assert_ne!(
        imp.colliders[0].por_quadro[2],
        imp.colliders[0].por_quadro[3]
    );
}

/// O Sprite do motor importado da `heroi_p14`: grelha 6×1, pivô e formas por célula ligados.
fn sprite_importado(s: &ph2d_pixel_core::Sprite, w: &mut ph2d_ecs::World, y: f32) -> Entity {
    // O `.ase` que o Pixel grava com o Add Collider em `body` (o conjunto vem do mundo, D2).
    let um = std::collections::BTreeSet::from([body(s).0]);
    let (bytes, _) =
        ph2d_pixel_io::ase::escrever(&ph2d_pixel_cena::com_colliders(s, &um)).expect("grava");
    let imp = importa_ase(&bytes, "heroi_p14", M).expect("import");
    let mut t = Transform::IDENTITY;
    t.translation.y = y;
    let e = w
        .spawn((
            t,
            Name::new("Importado"),
            ph2d_physics_ecs::LockRotation,
            SpriteGrid {
                hframes: 6,
                vframes: 1,
                frame: 0,
                sheet: 0,
            },
        ))
        .id();
    ph2d_pixel_cena::liga_celulas(w, e, imp.celulas, M);
    e
}

/// D1: o pivô do quadro 4 importado é o do Pixel (a key do 4 muda os pés de (8,15) para (6,15)), e o
/// anchor da célula mostrada é o desse pivô. *Mutação: `pivos_px` vazio ⇒ o anchor cai no da folha.*
#[test]
fn o_pivo_do_quadro_4_importado_e_o_do_pixel() {
    let s = heroi_p14();
    let mut sim = SimWorld::new();
    let e = sprite_importado(&s, sim.world_mut(), 0.0);
    let w = sim.world();
    let cells = w.get::<ph2d_ecs::SpriteCells>(e).expect("células");
    for (i, fr) in s.frames().iter().enumerate() {
        assert_eq!(
            cells.folha(0).expect("folha 0").pivo_px(i as u32),
            pivo_px(&s, fr.id),
            "pivô da célula {i}"
        );
    }
    // Quad de 1 m (16 px a 16 px/m): anchor = do pivô ao centro, Y para cima.
    let a = |c| ph2d_ecs::anchor_da_celula(w, e, 0, c, [1.0, 1.0]);
    assert_eq!(
        a(0),
        Some([0.0, 15.0 / 16.0 - 0.5]),
        "quadro 0: os pés (8,15)"
    );
    assert_eq!(
        a(4),
        Some([0.5 - 6.0 / 16.0, 15.0 / 16.0 - 0.5]),
        "quadro 4: (6,15)"
    );
}

/// D1, o molde `a_forma_segue_a_key_da_slice_no_rapier`: o Sprite do MOTOR importado, sem documento
/// Pixel, troca a caixa do rapier quando a célula mostrada passa de 2 a 3 e volta.
#[test]
fn a_forma_do_sprite_importado_segue_o_quadro_no_rapier() {
    use ph2d_ecs::Children;
    use ph2d_physics_ecs::{BodyKind, ColliderShape, PhysicsBridge, RigidBody};
    let mut s = heroi_p14();
    let id = body(&s);
    let mut lista = s.slices().to_vec();
    let b = lista.iter_mut().find(|c| c.id == id).expect("body");
    let k3 = b.keys.iter_mut().find(|k| k.frame == 3).expect("key 3");
    k3.shape.bounds = ph2d_pixel_core::Rect::new(4, 0, 4, 8);
    k3.shape.pivot = Some((4, 15));
    ph2d_pixel_core::apply(&mut s, ph2d_pixel_core::Op::Slices(lista)).expect("key 3");
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Name::new("Floor"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 50.0,
                half_y: 0.1,
            },
            ..Collider::default()
        },
        Transform::IDENTITY,
    ));
    let e = sprite_importado(&s, sim.world_mut(), 3.0);
    let f = sim.world().get::<Children>(e).expect("filha")[0];
    let caixa_0 = ([0.0, 6.0 / 16.0], [0.25, 7.0 / 16.0]);
    let caixa_3 = ([-2.0 / 16.0, 11.0 / 16.0], [2.0 / 16.0, 4.0 / 16.0]);
    let mut ponte = PhysicsBridge::new();
    let mut tick = 0_u64;
    let mut passo = |sim: &mut SimWorld, ponte: &mut PhysicsBridge, cell: u32, n: u64| {
        sim.world_mut()
            .get_mut::<SpriteGrid>(e)
            .expect("grelha")
            .frame = cell;
        for _ in 0..n {
            tick += 1;
            deriva_formas(sim.world_mut(), |_| None, M);
            ponte.dispatch(sim, true, tick);
        }
    };
    let caixa = |ponte: &PhysicsBridge| {
        let (lo, hi) = ponte.body_aabb(e).expect("corpo no rapier");
        let (x, y, _) = ponte.body_pose(e).expect("pose");
        (
            [(lo[0] + hi[0]) * 0.5 - x, (lo[1] + hi[1]) * 0.5 - y],
            [(hi[0] - lo[0]) * 0.5, (hi[1] - lo[1]) * 0.5],
        )
    };
    let perto = |a: ([f32; 2], [f32; 2]), b: ([f32; 2], [f32; 2])| {
        let d = |p: [f32; 2], q: [f32; 2]| (p[0] - q[0]).abs().max((p[1] - q[1]).abs());
        d(a.0, b.0) < 1e-4 && d(a.1, b.1) < 1e-4
    };
    passo(&mut sim, &mut ponte, 2, 10);
    let corpos = ponte.arena_body_count();
    assert!(
        perto(caixa(&ponte), caixa_0),
        "célula 2: {:?}",
        caixa(&ponte)
    );
    passo(&mut sim, &mut ponte, 3, 1);
    assert!(
        perto(caixa(&ponte), caixa_3),
        "célula 3: {:?}",
        caixa(&ponte)
    );
    let gizmo = sim.world().get::<Collider>(f).expect("c").local_aabb();
    assert!(perto(gizmo, caixa_3), "célula 3: o gizmo {gizmo:?}");
    assert_eq!(ponte.arena_body_count(), corpos, "trocou no lugar");
    passo(&mut sim, &mut ponte, 2, 1);
    assert!(perto(caixa(&ponte), caixa_0), "de volta à 2");
}

/// D1: a forma que segue a célula é DERIVADA — trocar de célula não marca a filha no undo (a mesma
/// regra do `SegueSlice`, `campos_derivados_quando_algum`).
#[test]
fn a_forma_da_celula_nao_entra_no_undo() {
    use ph2d_ecs::scene::incremental::{CaptureCache, capture_incremental};
    use ph2d_ecs::scene::registry::{ComponentRegistry, register_ecs_components};
    use ph2d_ecs::scene::save::WorldSnapshot;
    let mut r = ComponentRegistry::new();
    register_ecs_components(&mut r);
    ph2d_physics_ecs::register_physics_components(&mut r);
    ph2d_pixel_cena::registar(&mut r);
    let mut s = heroi_p14();
    let id = body(&s);
    let mut lista = s.slices().to_vec();
    let b = lista.iter_mut().find(|c| c.id == id).expect("body");
    b.keys
        .iter_mut()
        .find(|k| k.frame == 3)
        .expect("3")
        .shape
        .bounds = ph2d_pixel_core::Rect::new(4, 0, 4, 8);
    ph2d_pixel_core::apply(&mut s, ph2d_pixel_core::Op::Slices(lista)).expect("key 3");
    let mut sim = SimWorld::new();
    let e = sprite_importado(&s, sim.world_mut(), 0.0);
    sim.world_mut().get_mut::<SpriteGrid>(e).expect("g").frame = 2;
    deriva_formas(sim.world_mut(), |_| None, M);
    let mut cache = CaptureCache::new();
    let mut out = WorldSnapshot::default();
    capture_incremental(sim.world_mut(), &mut cache, &r, &mut out).expect("1.ª");
    sim.world_mut().get_mut::<SpriteGrid>(e).expect("g").frame = 3;
    // A grelha muda (é gravada); a filha só pela forma derivada.
    let f = sim.world().get::<ph2d_ecs::Children>(e).expect("filha")[0];
    let antes = sim.world().get::<Collider>(f).expect("c").shape;
    deriva_formas(sim.world_mut(), |_| None, M);
    assert_ne!(
        sim.world().get::<Collider>(f).expect("c").shape,
        antes,
        "a forma mudou"
    );
    let rep = capture_incremental(sim.world_mut(), &mut cache, &r, &mut out).expect("2.ª");
    assert_eq!(
        rep.reserialized, 1,
        "só a grelha do pai; a filha derivada não"
    );
}

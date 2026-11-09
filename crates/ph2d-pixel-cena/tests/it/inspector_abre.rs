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

#[test]
fn o_inspector_abre_o_que_o_pixel_grava() {
    let s = heroi_p14();
    assert!(
        slice_e_collider(s.slice(body(&s)).expect("body")),
        "a fixture marca body"
    );
    let (bytes, _) = ph2d_pixel_io::ase::escrever(&s).expect("grava");
    let imp = importa_ase(&bytes, "heroi_p14", M).expect("import");

    // O Pixel Sprite da MESMA fixture, pelas portas do mundo.
    let (mut sim, e) = pixel_sprite();
    deriva(sim.world_mut(), |_| Some((&s, 1)));
    let f = adiciona_collider_da_slice(sim.world_mut(), e, body(&s), &s, s.frames()[0].id, M)
        .expect("Add Collider");

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

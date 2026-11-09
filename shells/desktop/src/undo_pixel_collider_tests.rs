//! ⭐ P14 §7 `o_collider_nasce_por_gesto_e_desfaz_num_passo` — o «Add Collider» da janela SLICES
//! contra o undo REAL da shell: a captura `ProjectState::capture` com o registo do produto
//! (`init::build_component_registry`), a comparação do `post_frame_undo` (passo ⇔ estado ≠ baseline),
//! a fila `ProjectUndo` e o `ProjectState::restore` do Ctrl+Z/Y. A mutação do gesto é a do
//! `slices::adiciona_collider` da família (`adiciona_collider_da_slice`); o percurso do pedido do
//! painel até ela tem o seu gate em `shells/pixel-scenes` (`slices_cena::o_collider_nasce_por_gesto`).

use super::*;
use ph2d_ecs::{ChildOf, Name, PixelSprite, SpriteAnimator, SpriteGrid};
use ph2d_physics_ecs::{Collider, ColliderShape, RigidBody};
use ph2d_pixel_cena::fixture::{body, heroi_p14};
use ph2d_pixel_cena::{
    SegueSlice, adiciona_collider_da_slice, animador_de_nascenca, deriva, deriva_formas,
};

const M: f64 = 1.0 / 16.0;

fn captura(sim: &mut SimWorld, reg: &ph2d_ecs::scene::ComponentRegistry) -> ProjectState {
    super::tests::capture(sim, &VecScene::new(), reg)
}

fn filhas_que_seguem(sim: &mut SimWorld) -> Vec<Entity> {
    let w = sim.world_mut();
    let mut q = w.query_filtered::<Entity, With<SegueSlice>>();
    q.iter(w).collect()
}

fn pixel(sim: &mut SimWorld) -> Entity {
    let w = sim.world_mut();
    let mut q = w.query_filtered::<Entity, With<PixelSprite>>();
    q.single(w).expect("um Pixel Sprite")
}

/// *Mutação: o `restore` sem o despawn (o Ctrl+Z não tira a filha) ⇒ vermelho; tirar o
/// `campos_derivados_quando` do registo ⇒ a forma a andar com o quadro vira um 2.º passo.*
#[test]
fn o_collider_nasce_por_gesto_e_desfaz_num_passo() {
    let reg = crate::init::build_component_registry();
    let s = heroi_p14();
    let ed = s.frames()[0].id;
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Transform::IDENTITY,
        Name::new("Heroi"),
        PixelSprite(1),
        animador_de_nascenca(),
    ));
    let e = pixel(&mut sim);
    deriva(sim.world_mut(), |_| Some((&s, 1)));
    // O animador a tocar é do artista: grava-se ANTES da base; depois só a grelha anda.
    sim.world_mut()
        .get_mut::<SpriteAnimator>(e)
        .expect("animador")
        .playing = true;
    let base = captura(&mut sim, &reg);
    let mut fila = ProjectUndo::default();

    // O gesto: uma mutação, e o quadro seguinte da shell vê-a ⇒ UM passo.
    adiciona_collider_da_slice(sim.world_mut(), e, body(&s), &s, ed, M).expect("gesto");
    let depois = captura(&mut sim, &reg);
    assert_ne!(depois, base, "o gesto não mudou o documento");
    fila.push_undo(base.clone(), SelectionMark::default());
    // Os quadros seguintes: a forma segue a key (2 → 3, a key menor) sem novo passo.
    for i in [2, 3, 0] {
        sim.world_mut()
            .get_mut::<SpriteGrid>(e)
            .expect("grelha")
            .frame = i;
        deriva_formas(sim.world_mut(), |_| Some((&s, ed)), M);
        let agora = captura(&mut sim, &reg);
        assert!(
            agora == depois,
            "quadro {i}: a forma derivada fez outro passo em {:?}",
            agora.parts_that_differ(&depois)
        );
    }
    assert_eq!(fila.depth(), 1, "exatamente um passo");

    // Ctrl+Z: o estado-pré volta pelo restauro da shell, e a filha some.
    let (pre, _) = fila
        .undo(depois.clone(), SelectionMark::default())
        .expect("desfaz");
    let _ = pre.restore(&mut sim, &reg);
    deriva(sim.world_mut(), |_| Some((&s, 1)));
    deriva_formas(sim.world_mut(), |_| Some((&s, ed)), M);
    assert!(
        filhas_que_seguem(&mut sim).is_empty(),
        "o Ctrl+Z deixou a filha"
    );
    let e = pixel(&mut sim);
    assert!(
        sim.world().get::<RigidBody>(e).is_none(),
        "o corpo que o gesto criou também sai"
    );
    assert!(captura(&mut sim, &reg) == base, "o desfeito não é a base");

    // Ctrl+Y: a filha volta, com o nome da slice, e a deriva repõe a forma da key.
    let (pos, _) = fila
        .redo(base.clone(), SelectionMark::default())
        .expect("refaz");
    let _ = pos.restore(&mut sim, &reg);
    deriva(sim.world_mut(), |_| Some((&s, 1)));
    deriva_formas(sim.world_mut(), |_| Some((&s, ed)), M);
    let e = pixel(&mut sim);
    let f = filhas_que_seguem(&mut sim);
    assert_eq!(f.len(), 1, "o refazer devolve UMA filha");
    let w = sim.world();
    assert_eq!(w.get::<ChildOf>(f[0]).map(ChildOf::parent), Some(e));
    assert_eq!(w.get::<Name>(f[0]).map(Name::as_str), Some("body"));
    assert_eq!(
        w.get::<Collider>(f[0]).map(|c| c.shape),
        Some(ColliderShape::Cuboid {
            half_x: 0.25,
            half_y: 7.0 / 16.0,
        }),
        "a forma da key do quadro 0"
    );
}

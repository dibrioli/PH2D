//! ⭐ **Espelhar a Sprite: o Transform e o Flip do Look NÃO dão o mesmo** (dono 05/11: só o Flip, se derem o mesmo).
//! O clique dos botões é `girar_e_espelhar_todo_objecto` (troca o sinal da escala) e o collider espelhado na escala é
//! `a_flip_mirrors_the_collider_offset`. Aqui se mede o que o artista perderia: por isso os botões ficam.

use ph2d_core::Vec2;
use ph2d_ecs::{ChildOf, Transform, World};
use ph2d_physics_ecs::Collider;
use ph2d_render::Sprite;

/// Onde o ponto local `p` de quem tem a pose `t` cai no mundo.
fn no_mundo(t: Transform, p: [f32; 2]) -> Vec2 {
    let filho = Transform {
        translation: Vec2::new(p[0], p[1]),
        ..Default::default()
    };
    Transform::compose(t, filho).translation
}

struct Medida {
    centro_do_desenho: Vec2,
    filho: Vec2,
    colisao: Vec2,
    u_no_lado_direito: f32,
}

fn mede(anchor: [f32; 2], espelha: impl Fn(&mut Transform, &mut Sprite)) -> Medida {
    let mut w = World::new();
    let mut s = Sprite::atlas(ph2d_render::WHITE_TILE_KEY, [2.0, 2.0], [1.0; 4]);
    s.anchor = anchor;
    let mut t = Transform {
        translation: Vec2::new(5.0, 7.0),
        ..Default::default()
    };
    espelha(&mut t, &mut s);
    let e = w
        .spawn((
            s,
            t,
            Collider {
                offset: [0.5, 0.0],
                ..Default::default()
            },
        ))
        .id();
    let local = Transform {
        translation: Vec2::new(3.0, 0.0),
        ..Default::default()
    };
    let f = w.spawn((local, ChildOf(e))).id();
    let t = *w.get::<Transform>(e).unwrap();
    let s = *w.get::<Sprite>(e).unwrap();
    // O lado direito do quad no mundo: com a escala negativa, é o canto local esquerdo; o Flip troca o U.
    let lado_local = if t.scale.x < 0.0 { 0.0 } else { 1.0 };
    Medida {
        centro_do_desenho: no_mundo(t, s.resolve_anchor(100.0)),
        filho: Transform::compose(t, *w.get::<Transform>(f).unwrap()).translation,
        colisao: no_mundo(t, w.get::<Collider>(e).unwrap().offset),
        u_no_lado_direito: if s.flip_x {
            1.0 - lado_local
        } else {
            lado_local
        },
    }
}

fn pelo_transform(t: &mut Transform, _: &mut Sprite) {
    t.scale.x = -t.scale.x;
}

fn pelo_flip(_: &mut Transform, s: &mut Sprite) {
    s.flip_x = !s.flip_x;
}

/// Com o pivô no centro, a própria imagem fica igual pelos dois caminhos; os filhos e a colisão, não.
#[test]
fn espelhar_pelo_transform_leva_filhos_e_colisao_e_pelo_flip_nao() {
    let (a, b) = (
        mede([0.0, 0.0], pelo_transform),
        mede([0.0, 0.0], pelo_flip),
    );
    assert_eq!(
        a.centro_do_desenho, b.centro_do_desenho,
        "a imagem no sítio"
    );
    assert_eq!(a.u_no_lado_direito, b.u_no_lado_direito, "a imagem virada");
    assert_eq!(a.filho, Vec2::new(2.0, 7.0), "o Transform leva o filho");
    assert_eq!(b.filho, Vec2::new(8.0, 7.0), "o Flip deixa o filho");
    assert_eq!(a.colisao, Vec2::new(4.5, 7.0), "o Transform leva a colisão");
    assert_eq!(b.colisao, Vec2::new(5.5, 7.0), "o Flip deixa a colisão");
}

/// Com o pivô fora do centro, o Transform vira a imagem à volta do pivô; o Flip vira-a no mesmo sítio.
#[test]
fn com_o_pivo_de_lado_o_transform_muda_o_desenho_de_sitio_e_o_flip_nao() {
    let (a, b) = (
        mede([1.0, 0.0], pelo_transform),
        mede([1.0, 0.0], pelo_flip),
    );
    assert_eq!(a.centro_do_desenho, Vec2::new(4.0, 7.0));
    assert_eq!(b.centro_do_desenho, Vec2::new(6.0, 7.0));
}

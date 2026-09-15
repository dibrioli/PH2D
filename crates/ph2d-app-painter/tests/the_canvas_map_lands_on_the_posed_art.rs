//! ⭐⭐⭐ **O chrome do canvas é pintado ONDE A ARTE DESENHA** — e não onde o quad dela repousa.
//!
//! ⛔⛔ **A wave de 2026-09-14 curou o DEDO e deixou o OLHO** (medido 2026-09-15): a entrega do
//! ponteiro passou a resolver o clique pela malha posada e o editor de curva continuou a pintar os
//! pontos de controlo pelo afim do quad de repouso. ⇒ *um controlo desenhado por um mapa e agarrado
//! por outro*, que é a espécie de controlo morto que o `CLAUDE.md` §5.0 diz que nenhuma sonda deste
//! repo apanha.
//!
//! ⚠️ **Este gate mede a LEI, não o texto:** os gates de arquitectura irmãos (na `ph2d-editor-core`)
//! afirmam que os desenhadores *consultam* a porta; este afirma que a porta *responde certo*. As
//! duas metades são precisas — citar uma porta não é consultá-la, e consultá-la não é tê-la certa.

use ph2d_app_painter::canvas_map::CanvasMap;
use ph2d_core::Vec2;
use ph2d_ecs::{Entity, GlobalTransform, PresentWorld, SimRef, SimWorld};
use ph2d_render::{Camera2d, RenderInstance, SpriteMesh};
use ph2d_vector::{Affine, Point};

/// Uma sprite de `2 × 2` na origem, espelhada no mundo de apresentação.
fn sprite_de_2m(present: &mut PresentWorld, sim: &mut SimWorld) -> (Entity, u64) {
    let e = sim.world_mut().spawn(()).id();
    let gt = GlobalTransform::from_transform(ph2d_ecs::Transform::from_translation(Vec2::new(
        0.0, 0.0,
    )));
    let ri = RenderInstance {
        world_pos: [0.0, 0.0],
        size: [2.0, 2.0],
        atlas_uv: [0.0, 0.0, 1.0, 1.0],
        tint: [1.0; 4],
        basis: RenderInstance::IDENTITY_BASIS,
        texture_id: 0,
        premultiplied: 0.0,
        anchor: [0.0, 0.0],
        per_corner_tint: [[1.0; 4]; 4],
        opacity: 1.0,
        flip_uv: 0,
        z_order: 0,
        sampling: 0,
        uv_xform: RenderInstance::IDENTITY_UV_XFORM,
        clip_group: RenderInstance::CLIP_GROUP_NONE,
        clip_meta: 0,
        sub_order: 0,
    };
    present.world_mut().spawn((SimRef(e), gt, ri));
    (e, e.to_bits())
}

/// A malha POSADA para `x = 3..5` — o caso em que o quad e a malha discordam, e o único que diz
/// qual dos dois um consumidor está a ler (a mesma fixtura que o `ph2d-render` usa).
fn braco_posado(present: &mut PresentWorld, alvo: Entity) {
    let mut q = present.world_mut().query::<(Entity, &SimRef)>();
    let e = q
        .iter(present.world())
        .find(|(_, r)| r.0 == alvo)
        .map(|(e, _)| e)
        .expect("o espelho foi criado");
    present.world_mut().entity_mut(e).insert(SpriteMesh {
        local: vec![[3.0, 0.0], [5.0, 0.0], [3.0, 2.0]],
        uv: vec![[0.0, 1.0], [1.0, 1.0], [0.0, 0.0]],
        tris: vec![[0, 1, 2]],
    });
}

#[test]
fn a_handle_is_painted_where_the_bent_art_draws_that_texel() {
    let mut sim = SimWorld::new();
    let mut present = PresentWorld::new();
    let (alvo, bits) = sprite_de_2m(&mut present, &mut sim);

    let camera = Camera2d::new([0.0, 0.0], 8.0);
    let window = ph2d_host::WindowSize::new(800, 800);
    // Um afim de quad qualquer: o que importa é que ele NÃO é a resposta quando há malha.
    let afim = Affine::scale(1.0);
    let (iw, ih) = (100u32, 100u32);
    // A alça autorada a um quarto da largura e a três quartos da altura da imagem.
    let alca = [25.0f32, 75.0f32];

    // ── Sem malha: o mapa É o afim, ao bit. É este controlo que impede a cura de mudar o caminho
    // de sempre — uma sprite que se desenha como quad não passa a ser pintada por outra lei.
    let mapa = CanvasMap::new(present.world(), bits, iw, ih, afim, &camera, window);
    assert!(!mapa.is_mesh(), "sem `SpriteMesh` não há malha a consultar");
    assert_eq!(
        mapa.point(alca),
        afim * Point::new(f64::from(alca[0]), f64::from(alca[1])),
        "sobre um quad o mapa tem de ser o afim de sempre, sem um epsilon de diferença"
    );

    // ── Com malha: o mesmo texel é pintado onde a arte o DESENHA.
    braco_posado(&mut present, alvo);
    let mapa = CanvasMap::new(present.world(), bits, iw, ih, afim, &camera, window);
    assert!(mapa.is_mesh(), "com `SpriteMesh` a porta responde pela malha");
    // A UV `(0,25 · 0,75)` deste triângulo cai no ponto local `(3,5 · 0,5)` — e o mundo é o local,
    // porque a base é a identidade e o pivô é a origem. No ecrã: `world_to_screen`.
    let esperado = {
        let (x, y) = camera.world_to_screen([3.5, 0.5], window);
        Point::new(f64::from(x), f64::from(y))
    };
    let p = mapa.point(alca);
    assert!(
        (p.x - esperado.x).abs() < 1e-3 && (p.y - esperado.y).abs() < 1e-3,
        "a alça tem de ser pintada em {esperado:?} (onde a arte desenha aquele texel) e foi {p:?}"
    );
    // ⛔ **O DISCRIMINADOR**: o afim do quad punha-a noutro sítio. Sem esta asserção, um mapa que
    // ignorasse a malha passaria a de cima se por acaso os dois coincidissem.
    let pelo_quad = afim * Point::new(f64::from(alca[0]), f64::from(alca[1]));
    assert!(
        (p.x - pelo_quad.x).abs() > 1.0,
        "a resposta é a da MALHA POSADA, nunca a do quad de repouso ({p:?} contra {pelo_quad:?})"
    );

    // ── Um ponto FORA da malha (a metade da imagem que este triângulo não cobre) cai na lei do
    // quad: é o que mantém uma alça arrastada para fora da silhueta visível em vez de desaparecer.
    let fora = [95.0f32, 5.0f32];
    assert_eq!(
        mapa.point(fora),
        afim * Point::new(f64::from(fora[0]), f64::from(fora[1])),
        "fora da arte desenhada o chamador fica com a lei dele"
    );

    // ── E o ladrilho do *Repeat Image* é um deslocamento de ECRÃ, nas DUAS metades.
    let deslocado = mapa.deslocado(40.0, -25.0);
    let q = deslocado.point(alca);
    assert!(
        (q.x - (p.x + 40.0)).abs() < 1e-6 && (q.y - (p.y - 25.0)).abs() < 1e-6,
        "o ladrilho desloca a resposta da MALHA também, senão ele desenha as alças por cima do centro"
    );
}

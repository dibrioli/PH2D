//! ⭐ W5c — **quem reflete** (a lei do rebote, plano §2.4): o corpo que recebe luz, com o albedo
//! `tint∘self_tint`; o `NaoRecebeLuz` e o que brilha não refletem. ⭐ W5k: o translúcido também — o
//! albedo vai CRU e o solver multiplica-o pela opacidade (`α·a`). O caminho vetorial diz só SE reflete
//! (a cor dele vem da camada do Vello).

use bevy_ecs::world::World;
use ph2d_ecs::{GlobalTransform, SpriteEmissive, StableId, Transform, VecPathRef};
use ph2d_light2d_ecs::coleta::cena_de_luz;
use ph2d_light2d_ecs::{BloqueiaLuz, FormaDoSprite, Luz, NaoRecebeLuz, TipoDeLuz};
use ph2d_render::Sprite;

fn pose(x: f32) -> (Transform, GlobalTransform) {
    let mut t = Transform::IDENTITY;
    t.translation.x = x;
    (t, GlobalTransform::from_transform(t))
}

fn mundo() -> World {
    let mut w = World::new();
    let (t, g) = pose(0.0);
    let luz = Luz {
        cor: [1.0; 3],
        intensidade: 1.0,
        tamanho_m: 0.1,
        alcance_m: f32::INFINITY,
        tipo: TipoDeLuz::Ponto,
    };
    w.spawn((luz, t, g, StableId(1)));
    let opaco = BloqueiaLuz { opacidade: 1.0 };
    let tint = [0.8, 0.5, 0.25, 1.0];
    // 2: opaco que recebe ⇒ reflete com o tint (× self_tint).
    let (t, g) = pose(1.0);
    let mut s = Sprite::atlas(0, [1.0, 1.0], tint);
    s.self_tint = [0.5, 1.0, 1.0, 1.0];
    w.spawn((s, opaco, t, g, StableId(2)));
    // 3: translúcido ⇒ sim, com o albedo cru (o solver faz o `α·a`).
    let (t, g) = pose(2.0);
    w.spawn((
        Sprite::atlas(0, [1.0, 1.0], tint),
        BloqueiaLuz { opacidade: 0.5 },
        t,
        g,
        StableId(3),
    ));
    // 4: opaco que NÃO recebe luz ⇒ não.
    let (t, g) = pose(3.0);
    w.spawn((
        Sprite::atlas(0, [1.0, 1.0], tint),
        opaco,
        NaoRecebeLuz,
        t,
        g,
        StableId(4),
    ));
    // 5: o que brilha ⇒ não (a radiância dele é a do autor).
    let (t, g) = pose(4.0);
    w.spawn((
        Sprite::atlas(0, [1.0, 1.0], tint),
        opaco,
        SpriteEmissive(1.0),
        t,
        g,
        StableId(5),
    ));
    // 6–8: caminhos — opaco que recebe, opaco sem luz, translúcido (reflete).
    let (t, g) = pose(5.0);
    w.spawn((VecPathRef(6), opaco, t, g, StableId(6)));
    w.spawn((VecPathRef(7), opaco, NaoRecebeLuz, t, g, StableId(7)));
    w.spawn((
        VecPathRef(8),
        BloqueiaLuz { opacidade: 0.9 },
        t,
        g,
        StableId(8),
    ));
    w
}

#[test]
fn quem_reflete_e_o_corpo_que_recebe_luz() {
    let c = cena_de_luz(&mundo());
    // Os oclusores pela ordem do StableId: 2, 3, 4 (o 5 é emissor).
    let albedos: Vec<[f32; 3]> = c.oclusores.iter().map(|o| o.albedo).collect();
    assert_eq!(
        albedos,
        vec![[0.4, 0.5, 0.25], [0.8, 0.5, 0.25], [0.0; 3]],
        "{c:#?}"
    );
    let por_sprite: Vec<(FormaDoSprite, [f32; 3])> =
        c.sprites.iter().map(|s| (s.forma, s.albedo)).collect();
    assert_eq!(
        por_sprite,
        vec![
            (FormaDoSprite::Oclusor(0), [0.4, 0.5, 0.25]),
            (FormaDoSprite::Oclusor(1), [0.8, 0.5, 0.25]),
            (FormaDoSprite::Oclusor(2), [0.0; 3]),
            (FormaDoSprite::Emissor(1), [0.0; 3]),
        ]
    );
    let vetores: Vec<(u64, bool)> = c.vetores.iter().map(|v| (v.caminho, v.reflete)).collect();
    assert_eq!(vetores, vec![(6, true), (7, false), (8, true)]);
}

/// Um tint corrompido (NaN, negativo) não reflete — nunca envenena a série.
#[test]
fn um_albedo_invalido_nao_reflete() {
    let mut w = World::new();
    let (t, g) = pose(0.0);
    w.spawn((
        Sprite::atlas(0, [1.0, 1.0], [f32::NAN, -0.5, 0.75, 1.0]),
        BloqueiaLuz { opacidade: 1.0 },
        t,
        g,
    ));
    let c = cena_de_luz(&w);
    assert_eq!(c.oclusores[0].albedo, [0.0, 0.0, 0.75]);
}

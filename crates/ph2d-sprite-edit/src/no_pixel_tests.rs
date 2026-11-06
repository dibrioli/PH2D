use super::*;

fn imagem() -> AssetId {
    ph2d_asset::AssetDb::default().insert_image_rgba8(1, 1, vec![0; 4])
}

fn cena(olho: Option<Visibility>) -> (World, Entity, Entity) {
    let mut w = World::new();
    let mut o = w.spawn(Transform::default());
    if let Some(v) = olho {
        o.insert(v);
    }
    let origem = o.id();
    let trabalho = w.spawn(Transform::default()).id();
    abre(
        &mut w,
        origem,
        trabalho,
        (7, imagem()),
        (2, 1),
        100.0,
        ([0.0, 0.0], 1.0),
    );
    (w, origem, trabalho)
}

#[test]
fn o_quadro_de_entrada_nao_fecha_e_sair_do_draw_fecha_uma_vez() {
    let (mut w, origem, trabalho) = cena(Some(Visibility::default()));
    assert!(terminou(&mut w, None).is_none(), "ainda não entrou no Draw");
    assert!(terminou(&mut w, Some(trabalho.to_bits())).is_none());
    assert_eq!(
        terminou(&mut w, None).map(|a| (a.origem, a.key)),
        Some((origem, 7))
    );
    assert!(terminou(&mut w, None).is_none(), "devolve só uma vez");
    assert!(w.get_entity(trabalho).is_err(), "o objecto de trabalho sai");
}

#[test]
fn a_sprite_esconde_se_e_volta_como_estava() {
    let (mut w, origem, _) = cena(None);
    assert!(
        w.get::<Visibility>(origem).is_some_and(|v| v.hidden),
        "sem olho também se esconde"
    );
    encerra(&mut w);
    assert!(w.get::<Visibility>(origem).is_none());

    let escondida = Visibility::hidden();
    let (mut w, origem, _) = cena(Some(escondida));
    encerra(&mut w);
    assert_eq!(w.get::<Visibility>(origem).copied(), Some(escondida));
}

#[test]
fn a_imagem_abre_exactamente_sobre_o_desenho() {
    let mut w = World::new();
    let mut s = Sprite::atlas(ph2d_render::WHITE_TILE_KEY, [3.0, 1.0], [1.0; 4]);
    s.anchor = [0.5, 0.0];
    s.flip_x = true;
    s.flip_y = true;
    let t = Transform {
        translation: ph2d_core::Vec2::new(10.0, 20.0),
        ..Default::default()
    };
    let origem = w.spawn((s, t)).id();
    let trabalho = w.spawn(Transform::default()).id();
    abre(
        &mut w,
        origem,
        trabalho,
        (1, imagem()),
        (300, 200),
        100.0,
        ([0.0, 0.0], 1.0),
    );
    let p = *w.get::<Transform>(trabalho).unwrap();
    assert_eq!(
        (p.translation.x, p.translation.y),
        (10.5, 20.0),
        "o centro do desenho"
    );
    assert_eq!(
        (p.scale.x, p.scale.y),
        (-1.0, -0.5),
        "3 m em 300 px; 1 m em 200 px; espelhada"
    );
    encerra(&mut w);
}

/// *Uma folha 4×1: 1 px do documento mede 1 px da célula, e a célula 2 cai sobre o desenho.*
/// *Mutação: o tamanho da imagem inteira na escala ⇒ espremida.*
#[test]
fn a_celula_da_grelha_cai_sobre_o_desenho_e_o_pixel_mede_o_da_celula() {
    let mut w = World::new();
    let s = Sprite::atlas(ph2d_render::WHITE_TILE_KEY, [1.0, 1.0], [1.0; 4]);
    let grelha = ph2d_ecs::SpriteGrid {
        hframes: 4,
        vframes: 1,
        frame: 2,
    };
    let origem = w.spawn((s, grelha, Transform::default())).id();
    let trabalho = w.spawn(Transform::default()).id();
    abre(
        &mut w,
        origem,
        trabalho,
        (1, imagem()),
        (40, 10),
        100.0,
        ([0.0, 0.0], 1.0),
    );
    let p = *w.get::<Transform>(trabalho).unwrap();
    assert_eq!((p.scale.x, p.scale.y), (10.0, 10.0), "1 m em 10 px");
    let c = (p.translation.x, p.translation.y);
    assert!((c.0 + 0.5).abs() < 1e-5 && c.1.abs() < 1e-5, "{c:?}");
    encerra(&mut w);
}

/// *Uma Region: a mesma lei, com o Y da imagem para baixo.*
#[test]
fn a_region_cai_sobre_o_desenho() {
    let mut w = World::new();
    let s = Sprite::atlas(ph2d_render::WHITE_TILE_KEY, [2.0, 1.0], [1.0; 4]);
    let r = ph2d_ecs::SpriteRegion::individual([10.0, 0.0, 20.0, 10.0]);
    let origem = w.spawn((s, r, Transform::default())).id();
    let trabalho = w.spawn(Transform::default()).id();
    abre(
        &mut w,
        origem,
        trabalho,
        (1, imagem()),
        (40, 20),
        100.0,
        ([0.0, 0.0], 1.0),
    );
    let p = *w.get::<Transform>(trabalho).unwrap();
    assert_eq!((p.scale.x, p.scale.y), (10.0, 10.0));
    let c = (p.translation.x, p.translation.y);
    assert!(c.0.abs() < 1e-5 && (c.1 + 0.5).abs() < 1e-5, "{c:?}");
    encerra(&mut w);
}

/// *Espelhada, a imagem inteira espelha à volta da célula: o que estava à esquerda dela passa à direita.*
#[test]
fn a_celula_espelhada_continua_sobre_o_desenho() {
    let mut w = World::new();
    let mut s = Sprite::atlas(ph2d_render::WHITE_TILE_KEY, [1.0, 1.0], [1.0; 4]);
    s.flip_x = true;
    let grelha = ph2d_ecs::SpriteGrid {
        hframes: 4,
        vframes: 1,
        frame: 2,
    };
    let origem = w.spawn((s, grelha, Transform::default())).id();
    let trabalho = w.spawn(Transform::default()).id();
    abre(
        &mut w,
        origem,
        trabalho,
        (1, imagem()),
        (40, 10),
        100.0,
        ([0.0, 0.0], 1.0),
    );
    let p = *w.get::<Transform>(trabalho).unwrap();
    assert_eq!(p.scale.x, -10.0);
    assert!((p.translation.x - 0.5).abs() < 1e-5, "{}", p.translation.x);
    encerra(&mut w);
}

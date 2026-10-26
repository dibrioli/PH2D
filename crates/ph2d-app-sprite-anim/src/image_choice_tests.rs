use super::*;
use ph2d_ecs::{AnimationTag, ScrollFactor, SpriteAnimator};
use ph2d_render::{HERO_ART, HERO_SHEET_KEY, HILLS_TILE, WHITE_TILE_KEY};

const PPM: f32 = 100.0;
const KEY_IMPORTADA: u32 = 7;
const BRANCO: [f32; 4] = [1.0; 4];

fn perto(a: [f32; 2], b: [f32; 2]) -> bool {
    (a[0] - b[0]).abs() < 1e-4 && (a[1] - b[1]).abs() < 1e-4
}

/// A camada de fundo como o Add a veste (montes de fábrica, 4 ladrilhos de 8 m).
fn camada(world: &mut World) -> Entity {
    let [lw, lh] = HILLS_TILE.native_m();
    world
        .spawn((
            Sprite::atlas(HILLS_TILE.key, [lw * 4.0, lh], BRANCO),
            SpriteGrid {
                hframes: 3,
                vframes: 1,
                frame: 1,
            },
            UvTransform {
                scale: [4.0, 1.0],
                offset: [0.0, 0.0],
            },
            TextureRepeat(RepeatMode::Enabled),
            ScrollFactor { k: [0.4, 0.4] },
            ScrollRepeat { tile: [lw, 0.0] },
        ))
        .id()
}

/// O herói de fábrica (8 × 4) a 1 × 1 m, com duas animações por intervalo.
fn heroi(world: &mut World) -> Entity {
    let mut tags = SpriteAnimations::new();
    tags.insert(AnimationTag::new("idle", 0, 3)).unwrap();
    tags.insert(AnimationTag::new("run", 8, 15)).unwrap();
    world
        .spawn((
            Sprite::atlas(HERO_SHEET_KEY, [1.0, 1.0], BRANCO),
            SpriteGrid {
                hframes: 8,
                vframes: 4,
                frame: 9,
            },
            tags,
            SpriteAnimator::new("run"),
        ))
        .id()
}

#[test]
fn a_camada_de_fundo_repete_a_imagem_nova_e_cobre_a_largura_antiga() {
    let mut w = World::new();
    let e = camada(&mut w);
    let antes = w.get::<Sprite>(e).unwrap().size;
    let nova = imported_facts(300, 100, PPM);
    assert!(choose_image(
        &mut w,
        e,
        Some(factory_facts(&HILLS_TILE)),
        ImageSource::Atlas { key: KEY_IMPORTADA },
        nova,
    ));
    let s = w.get::<Sprite>(e).unwrap();
    assert_eq!(s.source, SpriteSource::Atlas { key: KEY_IMPORTADA });
    // Ladrilho de 3 × 1 m (escala 1), ⌈32 / 3⌉ = 11 ladrilhos.
    assert_eq!(w.get::<ScrollRepeat>(e).unwrap().tile, [3.0, 0.0]);
    assert_eq!(w.get::<UvTransform>(e).unwrap().scale, [11.0, 1.0]);
    assert!(perto(s.size, [33.0, 1.0]));
    assert!(s.size[0] >= antes[0]);
    // A célula do meio do azulejo antigo não sobrevive a uma imagem sem corte.
    assert!(w.get::<SpriteGrid>(e).is_none());
    assert_eq!(w.get::<ScrollFactor>(e).unwrap().k, [0.4, 0.4]);
    assert_eq!(
        w.get::<TextureRepeat>(e),
        Some(&TextureRepeat(RepeatMode::Enabled))
    );
}

#[test]
fn a_camada_volta_aos_montes_de_fabrica_igual() {
    let mut w = World::new();
    let e = camada(&mut w);
    let s0 = *w.get::<Sprite>(e).unwrap();
    let nova = imported_facts(300, 100, PPM);
    let montes = factory_facts(&HILLS_TILE);
    choose_image(
        &mut w,
        e,
        Some(montes),
        ImageSource::Atlas { key: KEY_IMPORTADA },
        nova,
    );
    choose_image(
        &mut w,
        e,
        Some(nova),
        ImageSource::Atlas {
            key: HILLS_TILE.key,
        },
        montes,
    );
    let s = w.get::<Sprite>(e).unwrap();
    assert_eq!(s.source, s0.source);
    assert_eq!(s.size[1], s0.size[1]);
    // 33 m de largura ⇒ ⌈33 / 8⌉ = 5 ladrilhos (cobre pelo menos o que cobria).
    assert_eq!(w.get::<UvTransform>(e).unwrap().scale, [5.0, 1.0]);
    assert_eq!(w.get::<ScrollRepeat>(e).unwrap().tile, [8.0, 0.0]);
    assert_eq!(
        w.get::<SpriteGrid>(e).copied(),
        Some(SpriteGrid {
            hframes: 3,
            vframes: 1,
            frame: 1
        })
    );
}

#[test]
fn a_folha_nova_do_heroi_mantem_as_animacoes_por_intervalo() {
    let mut w = World::new();
    let e = heroi(&mut w);
    let tags = w.get::<SpriteAnimations>(e).unwrap().clone();
    // Uma folha importada do mesmo corte, a 2 × o tamanho de célula da fábrica.
    let folha = imported_facts(384 * 8, 384 * 4, PPM);
    choose_image(
        &mut w,
        e,
        Some(factory_facts(&HERO_ART)),
        ImageSource::Atlas { key: KEY_IMPORTADA },
        folha,
    );
    assert_eq!(w.get::<SpriteAnimations>(e), Some(&tags));
    assert_eq!(
        w.get::<SpriteGrid>(e).copied(),
        Some(SpriteGrid {
            hframes: 8,
            vframes: 4,
            frame: 9
        })
    );
    assert_eq!(w.get::<SpriteAnimator>(e).unwrap().current, "run");
    // A escala fica: a célula nativa de 3,84 m sobre a de 1 m da fábrica (192 px/m).
    assert!(perto(w.get::<Sprite>(e).unwrap().size, [3.84, 3.84]));
    // E a fábrica volta pela MESMA porta, ao tamanho de antes.
    choose_image(
        &mut w,
        e,
        Some(folha),
        ImageSource::Atlas { key: HERO_SHEET_KEY },
        factory_facts(&HERO_ART),
    );
    assert!(perto(w.get::<Sprite>(e).unwrap().size, [1.0, 1.0]));
    assert_eq!(w.get::<SpriteAnimations>(e), Some(&tags));
}

#[test]
fn uma_sprite_nova_mostra_a_imagem_ao_tamanho_nativo() {
    let mut w = World::new();
    let e = w.spawn(Sprite::atlas(WHITE_TILE_KEY, [1.0, 1.0], BRANCO)).id();
    choose_image(
        &mut w,
        e,
        None,
        ImageSource::Atlas {
            key: HILLS_TILE.key,
        },
        factory_facts(&HILLS_TILE),
    );
    // O azulejo de fábrica mostra-se pela célula do meio, a 8 × 6 m.
    assert!(perto(w.get::<Sprite>(e).unwrap().size, [8.0, 6.0]));
    assert_eq!(w.get::<SpriteGrid>(e).unwrap().frame, 1);
}

#[test]
fn um_bloco_que_repete_guarda_o_tamanho_e_as_repeticoes() {
    let mut w = World::new();
    let uv = UvTransform {
        scale: [3.0, 1.0],
        offset: [0.0, 0.0],
    };
    let e = w
        .spawn((
            Sprite::atlas(WHITE_TILE_KEY, [3.0, 1.0], BRANCO),
            uv,
            TextureRepeat(RepeatMode::Enabled),
        ))
        .id();
    choose_image(
        &mut w,
        e,
        Some(imported_facts(100, 100, PPM)),
        ImageSource::Atlas { key: KEY_IMPORTADA },
        imported_facts(50, 25, PPM),
    );
    assert_eq!(w.get::<Sprite>(e).unwrap().size, [3.0, 1.0]);
    assert_eq!(w.get::<UvTransform>(e), Some(&uv));
}

#[test]
fn uma_imagem_so_individual_leva_o_nome_dos_pixels() {
    let mut w = World::new();
    let e = w.spawn(Sprite::atlas(WHITE_TILE_KEY, [1.0, 1.0], BRANCO)).id();
    let id = AssetId::from_digest([9; 32]);
    choose_image(
        &mut w,
        e,
        None,
        ImageSource::Individual {
            texture_id: 3,
            pixels: id,
        },
        imported_facts(200, 100, PPM),
    );
    assert_eq!(w.get::<SpritePixels>(e), Some(&SpritePixels(id)));
    assert_eq!(
        w.get::<Sprite>(e).unwrap().source,
        SpriteSource::Individual { texture_id: 3 }
    );
    assert!(perto(w.get::<Sprite>(e).unwrap().size, [2.0, 1.0]));
    // E voltar ao atlas tira o carimbo (senão o save gravava os pixels velhos).
    choose_image(
        &mut w,
        e,
        Some(imported_facts(200, 100, PPM)),
        ImageSource::Atlas { key: KEY_IMPORTADA },
        imported_facts(200, 100, PPM),
    );
    assert!(w.get::<SpritePixels>(e).is_none());
}

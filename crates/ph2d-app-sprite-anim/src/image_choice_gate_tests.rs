//! Os gates do plano GEF 12 pela PORTA (`choose_library_image`), com a biblioteca de verdade: a fábrica
//! registada como o app a regista e a imagem do smoke trazida do ficheiro.

use super::*;
use ph2d_ecs::scene::{WorldSnapshot, snapshot_to_world, world_to_snapshot};
use ph2d_ecs::{ScrollFactor, Transform};
use ph2d_render::{BACKGROUND_TEXELS_PER_M, CLOUDS_TILE, HILLS_TILE};

const PPM: f32 = 100.0;
const MONTES_POR_DO_SOL: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/GameEngineFuncional/arte/montes_por_do_sol.png"
);

struct Biblioteca {
    db: AssetDb,
    atlas: BTreeMap<u32, AssetId>,
    por_do_sol: AssetId,
}

fn biblioteca() -> Biblioteca {
    let db = AssetDb::new();
    let mut atlas = BTreeMap::new();
    ph2d_app_components::factory_images::register(&db, &mut atlas);
    let por_do_sol = db
        .load_png_path(std::path::Path::new(MONTES_POR_DO_SOL))
        .expect("a arte do smoke abre");
    atlas.insert(ph2d_render::FIRST_IMPORT_KEY, por_do_sol);
    Biblioteca {
        db,
        atlas,
        por_do_sol,
    }
}

fn sem_placa(_: u32, _: u32, _: &[u8]) -> Result<u32, String> {
    Err("estes gates não sobem texturas próprias".into())
}

/// A camada como o Add a veste (o mesmo `veste_camada` do jogo, aqui escrito para não ciclar crates).
fn camada(sim: &mut SimWorld, nome: &str) -> Entity {
    let [lw, lh] = HILLS_TILE.native_m();
    sim.world_mut()
        .spawn((
            Transform::IDENTITY,
            ph2d_ecs::Name::new(nome),
            Sprite::atlas(HILLS_TILE.key, [lw * 4.0, lh], [1.0; 4]),
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

/// ⭐ O texel que a sprite manda amostrar no ponto `quv` do quad (`[0,1]²`, `v` para baixo): a imagem
/// referenciada (pela chave do atlas ou pelo `SpritePixels`), a célula da grelha, a repetição do UV.
fn texel(sim: &SimWorld, e: Entity, b: &Biblioteca, quv: [f32; 2]) -> [u8; 4] {
    let w = sim.world();
    let id = ph2d_app_components::asset_index_build::texture_of(
        w.get::<SpritePixels>(e),
        w.get::<Sprite>(e),
        &b.atlas,
    )
    .expect("a sprite mostra uma imagem da biblioteca");
    let asset = b.db.get(&id).expect("no db");
    let (iw, ih, px) = asset.image_rgba8().expect("imagem");
    let g = w.get::<SpriteGrid>(e).copied().unwrap_or(SpriteGrid::SINGLE);
    let uv = w.get::<UvTransform>(e).copied().unwrap_or(UvTransform::IDENTITY);
    let (cw, ch) = (iw / g.hframes, ih / g.vframes);
    let (cx, cy) = (g.frame % g.hframes, g.frame / g.hframes);
    let u = (quv[0] * uv.scale[0] + uv.offset[0]).fract();
    let v = (quv[1] * uv.scale[1] + uv.offset[1]).fract();
    let x = cx * cw + ((u * cw as f32) as u32).min(cw - 1);
    let y = cy * ch + ((v * ch as f32) as u32).min(ch - 1);
    let o = ((y * iw + x) * 4) as usize;
    [px[o], px[o + 1], px[o + 2], px[o + 3]]
}

fn perto(a: [u8; 4], b: [u8; 4]) -> bool {
    a.iter().zip(b).all(|(x, y)| x.abs_diff(y) <= 3)
}

/// O corpo dos montes de fábrica (`montes_escuro` `#ACD5DD`) e o dos montes ao pôr do sol (`arte_trocar.py`).
const MONTES_ESCURO: [u8; 4] = [0xAC, 0xD5, 0xDD, 0xFF];

fn instantaneo(sim: &mut SimWorld, r: &ph2d_ecs::scene::ComponentRegistry) -> WorldSnapshot {
    let mut prop = ph2d_ecs::TransformPropagationState::new(sim.world_mut());
    let mut work = ph2d_ecs::WorklistBuf::default();
    let mut snap = WorldSnapshot::default();
    world_to_snapshot(sim.world_mut(), &mut prop, &mut work, r, &mut snap).expect("captura");
    snap
}

fn porta(sim: &mut SimWorld, b: &Biblioteca, e: Entity, id: AssetId) {
    choose_library_image(sim, &mut sem_placa, &b.db, &b.atlas, PPM, e, id).expect("escolhe");
}

#[test]
fn trocar_a_imagem_muda_o_que_se_desenha() {
    let b = biblioteca();
    let mut sim = SimWorld::new();
    let hills = camada(&mut sim, "Hills");
    let controlo = camada(&mut sim, "Controlo");
    let fundo = [0.5, 0.95];
    assert!(perto(texel(&sim, hills, &b, fundo), MONTES_ESCURO));
    porta(&mut sim, &b, hills, b.por_do_sol);
    let depois = texel(&sim, hills, &b, fundo);
    assert!(
        !perto(depois, MONTES_ESCURO) && depois[0] > depois[2],
        "o fundo dos montes não passou à cor quente: {depois:?}"
    );
    // A imagem nova é a do ficheiro, no mesmo sítio dela.
    let asset = b.db.get(&b.por_do_sol).unwrap();
    let (iw, ih, px) = asset.image_rgba8().unwrap();
    let o = (((ih as f32 * 0.95) as u32 * iw + iw / 2) * 4) as usize;
    assert!(perto(depois, [px[o], px[o + 1], px[o + 2], px[o + 3]]));
    // CONTROLO: a camada que ninguém trocou desenha o mesmo.
    assert!(perto(texel(&sim, controlo, &b, fundo), MONTES_ESCURO));
}

#[test]
fn a_troca_sobrevive_a_gravar_e_abrir() {
    let b = biblioteca();
    let r = ph2d_app_components::test_support::registo();
    let mut sim = SimWorld::new();
    let hills = camada(&mut sim, "Hills");
    porta(&mut sim, &b, hills, b.por_do_sol);
    let snap = instantaneo(&mut sim, &r);
    let mut aberto = SimWorld::new();
    let rows = snapshot_to_world(aberto.world_mut(), &snap, &r).expect("abre");
    let e = rows[0];
    for (a, z) in [(hills, e)] {
        let (wa, wz) = (sim.world(), aberto.world());
        assert_eq!(wa.get::<Sprite>(a), wz.get::<Sprite>(z));
        assert_eq!(wa.get::<UvTransform>(a), wz.get::<UvTransform>(z));
        assert_eq!(wa.get::<ScrollRepeat>(a), wz.get::<ScrollRepeat>(z));
        assert_eq!(wa.get::<SpriteGrid>(a), wz.get::<SpriteGrid>(z));
    }
    assert!(perto(
        texel(&aberto, e, &b, [0.5, 0.95]),
        texel(&sim, hills, &b, [0.5, 0.95])
    ));
}

#[test]
fn desfazer_devolve_a_imagem_o_tamanho_a_grelha_e_o_ladrilho_num_passo() {
    let b = biblioteca();
    let r = ph2d_app_components::test_support::registo();
    let mut sim = SimWorld::new();
    let hills = camada(&mut sim, "Hills");
    let antes = instantaneo(&mut sim, &r);
    porta(&mut sim, &b, hills, b.por_do_sol);
    let depois = instantaneo(&mut sim, &r);
    assert_ne!(antes, depois, "a troca não mudou o documento");
    // O Ctrl+Z repõe o instantâneo de antes (o `ProjectState` é este mundo).
    let mut desfeito = SimWorld::new();
    let e = snapshot_to_world(desfeito.world_mut(), &antes, &r).expect("repõe")[0];
    assert!(perto(texel(&desfeito, e, &b, [0.5, 0.95]), MONTES_ESCURO));
    assert_eq!(
        instantaneo(&mut desfeito, &r),
        antes,
        "o passo de desfazer não devolveu o mundo de antes"
    );
}

/// O ladrilho do fundo continua o de antes do plano 12 (que o media em `BACKGROUND_TEXELS_PER_M`), bit a bit
/// — um projecto do nível 2 gravado antes abre com a mesma camada.
#[test]
fn o_ladrilho_de_fabrica_e_o_de_antes() {
    for art in [CLOUDS_TILE, HILLS_TILE] {
        let antes = art
            .cell_px()
            .map(|v| v as f32 / BACKGROUND_TEXELS_PER_M as f32);
        assert_eq!(art.native_m().map(f32::to_bits), antes.map(f32::to_bits));
    }
}

/// A fábrica está na biblioteca e o save não a grava: o mapa leva as 8 chaves, e só a importada é do ficheiro.
#[test]
fn a_fabrica_entra_na_biblioteca_e_fica_fora_do_ficheiro() {
    let b = biblioteca();
    let gravadas: Vec<u32> = b
        .atlas
        .keys()
        .copied()
        .filter(|k| !ph2d_app_components::factory_images::is_factory_key(*k))
        .collect();
    assert_eq!(gravadas, vec![ph2d_render::FIRST_IMPORT_KEY]);
    assert_eq!(b.atlas.len(), ph2d_render::FACTORY_ART.len() + 1);
    // Idempotente: registar outra vez não muda nada.
    let mut outra = b.atlas.clone();
    ph2d_app_components::factory_images::register(&b.db, &mut outra);
    assert_eq!(outra, b.atlas);
}

const HEROI_AZUL: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/GameEngineFuncional/arte/heroi_azul.png"
);

/// A folha nova do Player (a do smoke, 8 × 4 células de 100 px) mantém as animações por intervalo, o quadro
/// e o tamanho (1 m, como a de fábrica); e a de fábrica volta pela mesma porta.
#[test]
fn a_folha_nova_do_player_mantem_as_animacoes() {
    let b = biblioteca();
    let azul = b
        .db
        .load_png_path(std::path::Path::new(HEROI_AZUL))
        .expect("abre");
    let mut atlas = b.atlas.clone();
    atlas.insert(ph2d_render::FIRST_IMPORT_KEY + 1, azul);
    let b = Biblioteca { atlas, ..b };
    let mut sim = SimWorld::new();
    let mut tags = SpriteAnimations::new();
    for t in ph2d_render::HERO_SHEET_TAGS {
        tags.insert(ph2d_ecs::AnimationTag::new(t.name, t.from, t.to))
            .unwrap();
    }
    let e = sim
        .world_mut()
        .spawn((
            Sprite::atlas(ph2d_render::HERO_SHEET_KEY, [1.0, 1.0], [1.0; 4]),
            SpriteGrid {
                hframes: 8,
                vframes: 4,
                frame: 11,
            },
            tags.clone(),
        ))
        .id();
    porta(&mut sim, &b, e, azul);
    let w = sim.world();
    assert_eq!(w.get::<SpriteAnimations>(e), Some(&tags));
    assert_eq!(
        w.get::<SpriteGrid>(e).copied(),
        Some(SpriteGrid {
            hframes: 8,
            vframes: 4,
            frame: 11
        })
    );
    assert_eq!(w.get::<Sprite>(e).unwrap().size, [1.0, 1.0]);
    let fabrica = b.atlas[&ph2d_render::HERO_SHEET_KEY];
    porta(&mut sim, &b, e, fabrica);
    assert_eq!(
        sim.world().get::<Sprite>(e).unwrap().source,
        ph2d_render::SpriteSource::Atlas {
            key: ph2d_render::HERO_SHEET_KEY
        }
    );
    assert_eq!(sim.world().get::<Sprite>(e).unwrap().size, [1.0, 1.0]);
}

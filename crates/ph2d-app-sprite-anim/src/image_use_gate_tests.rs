//! Os gates do plano GEF 13 sobre o nível do jogo como o dono o viu: os três defeitos do doc 13 e a mesma
//! imagem pelas portas todas.

use super::*;
use crate::image_choice::{choose_library_image, image_facts, place_library_image};
use ph2d_app_components::test_support::registo;
use ph2d_asset::{AssetDb, AssetId};
use ph2d_ecs::scene::{WorldSnapshot, snapshot_to_world, world_to_snapshot};
use ph2d_ecs::{Name, SimWorld, SpriteAnimator, SpriteGrid, SpritePixels, UvTransform};
use ph2d_render::{FLAG_SHEET_KEY, PLATFORM_TILE, PLATFORM_TILE_KEY, SpriteSource, WHITE_TILE_KEY};
use std::collections::BTreeMap;

const PPM: f32 = 100.0;
const MONTES_POR_DO_SOL: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/GameEngineFuncional/arte/montes_por_do_sol.png"
);

struct Biblioteca {
    db: AssetDb,
    atlas: BTreeMap<u32, AssetId>,
}

impl Biblioteca {
    fn de(&self, key: u32) -> AssetId {
        self.atlas[&key]
    }

    fn factos(&self, id: AssetId) -> ImageFacts {
        image_facts(id, &self.db, &self.atlas, PPM).expect("imagem da biblioteca")
    }
}

fn biblioteca() -> Biblioteca {
    let db = AssetDb::new();
    let mut atlas = BTreeMap::new();
    ph2d_app_components::factory_images::register(&db, &mut atlas);
    let importada = db
        .load_png_path(std::path::Path::new(MONTES_POR_DO_SOL))
        .expect("a arte do smoke abre");
    atlas.insert(ph2d_render::FIRST_IMPORT_KEY, importada);
    Biblioteca { db, atlas }
}

fn sem_placa(_: u32, _: u32, _: &[u8]) -> Result<u32, String> {
    Err("uma imagem da biblioteca não sobe cópia".into())
}

/// O registo do produto (já leva os corpos do menu Add).
fn registo_do_jogo() -> ph2d_ecs::scene::ComponentRegistry {
    registo()
}

fn nivel(n: u32) -> SimWorld {
    let mut sim = SimWorld::new();
    ph2d_app_game::smoke::montar(&mut sim, &registo_do_jogo(), n);
    sim
}

fn por_nome(sim: &mut SimWorld, nome: &str) -> Entity {
    let w = sim.world_mut();
    let mut q = w.query::<(Entity, &Name)>();
    q.iter(w)
        .find(|(_, n)| n.as_str() == nome)
        .map(|(e, _)| e)
        .unwrap_or_else(|| panic!("{nome} no nível"))
}

fn celulas(g: Option<&SpriteGrid>) -> u32 {
    g.map_or(1, |g| g.hframes.max(1) * g.vframes.max(1))
}

/// Defeito (a): Platform High mostrava «H Frames 3» — o truque das cópias do azulejo vazava para o objecto e
/// para a imagem da biblioteca.
#[test]
fn o_bloco_de_fabrica_e_um_azulejo_sem_quadros() {
    let b = biblioteca();
    let plataforma = b.factos(b.de(PLATFORM_TILE_KEY));
    assert_eq!(
        plataforma.slicing.map_or([1, 1], |s| s.grid),
        [1, 1],
        "a imagem da plataforma tem quadros"
    );
    let dims =
        b.db.get(&b.de(PLATFORM_TILE_KEY))
            .and_then(|a| a.image_dimensions());
    assert_eq!(
        dims,
        Some((PLATFORM_TILE.cell_px()[0], PLATFORM_TILE.cell_px()[1])),
        "a miniatura da biblioteca mostra as cópias"
    );
    // O nível 1 tem blocos de cor, sem imagem; o 2 e o 4 vestem os azulejos de fábrica.
    for n in [2, 4] {
        let mut sim = nivel(n);
        let e = por_nome(&mut sim, "Platform High");
        let w = sim.world();
        assert_eq!(image_use(w, e), Some(ImageUse::Repeated), "nível {n}");
        assert_eq!(
            celulas(w.get::<SpriteGrid>(e)),
            1,
            "nível {n}: quadros no bloco"
        );
    }
}

/// Defeito (b): largar «Platform» dos Assets no canvas dava outra coisa (cópia Individual, 3 azulejos).
#[test]
fn largar_platform_no_canvas_e_a_mesma_imagem() {
    let b = biblioteca();
    let mut sim = SimWorld::new();
    let e = place_library_image(
        &mut sim,
        &mut sem_placa,
        &b.db,
        &b.atlas,
        PPM,
        [2.0, 1.0],
        "Platform",
        b.de(PLATFORM_TILE_KEY),
    )
    .expect("a queda dá uma sprite");
    let w = sim.world();
    let s = w.get::<Sprite>(e).expect("sprite");
    assert_eq!(
        s.source,
        SpriteSource::Atlas {
            key: PLATFORM_TILE_KEY
        }
    );
    assert_eq!(s.size, PLATFORM_TILE.native_m());
    assert!(w.get::<SpritePixels>(e).is_none(), "copiou os píxeis");
    assert_eq!(celulas(w.get::<SpriteGrid>(e)), 1);
    // Um azulejo nasce Tiled de um ladrilho: o mesmo tipo da Platform Low (plano GEF 14 D4).
    assert_eq!(
        w.get::<ph2d_ecs::TiledDraw>(e).map(|t| t.tile),
        Some(PLATFORM_TILE.native_m())
    );
    assert_eq!(image_use(w, e), Some(ImageUse::Repeated));
    assert_eq!(
        w.get::<Name>(e).map(Name::as_str),
        Some("Platform"),
        "o nome da imagem"
    );
}

fn instantaneo(sim: &mut SimWorld) -> WorldSnapshot {
    let r = registo_do_jogo();
    let mut prop = ph2d_ecs::TransformPropagationState::new(sim.world_mut());
    let mut work = ph2d_ecs::WorklistBuf::default();
    let mut snap = WorldSnapshot::default();
    world_to_snapshot(sim.world_mut(), &mut prop, &mut work, &r, &mut snap).expect("captura");
    snap
}

/// Defeito (c): Image de Platform High ← Flag guardava as proporções do bloco, cortava a bandeira parada e não
/// havia como a animar. Agora o seletor «Use» passa-a a «Animated» e a bandeira ondula (plano 15: o aviso saiu,
/// o conflito é irrepresentável).
#[test]
fn a_bandeira_num_bloco_fica_animada() {
    let b = biblioteca();
    let mut sim = nivel(2);
    let e = por_nome(&mut sim, "Platform High");
    let bandeira = b.de(FLAG_SHEET_KEY);
    choose_library_image(&mut sim, &mut sem_placa, &b.db, &b.atlas, PPM, e, bandeira)
        .expect("escolhe");
    let factos = b.factos(bandeira);
    assert_eq!(image_use(sim.world(), e), Some(ImageUse::Repeated));
    assert_eq!(cells(factos), 4);
    let antes = instantaneo(&mut sim);
    assert!(apply_edit(
        sim.world_mut(),
        e,
        &b.db,
        &b.atlas,
        PPM,
        ImageUseEdit::Use(InspectorImageUse::Animated)
    ));
    let w = sim.world();
    assert_eq!(image_use(w, e), Some(ImageUse::Animated));
    let tags = w.get::<SpriteAnimations>(e).expect("animações");
    let tag = tags.iter().next().expect("uma animação");
    assert_eq!((tag.from, tag.to), (0, 3));
    let anim = w.get::<SpriteAnimator>(e).expect("animador");
    assert!(anim.autoplay && anim.current == tag.name);
    assert!(
        w.get::<TextureRepeat>(e)
            .is_none_or(|r| r.0 == RepeatMode::Disabled)
    );
    assert!(w.get::<UvTransform>(e).is_none() && w.get::<ScrollRepeat>(e).is_none());
    // A bandeira inteira, de pé: uma célula (1 × 2 m) à escala do ladrilho do bloco (1 m de largo).
    assert_eq!(w.get::<Sprite>(e).unwrap().size, [1.0, 2.0]);
    // Um passo de undo devolve o bloco.
    let depois = instantaneo(&mut sim);
    assert_ne!(antes, depois);
    let mut desfeito = SimWorld::new();
    let r = registo_do_jogo();
    snapshot_to_world(desfeito.world_mut(), &antes, &r).expect("repõe");
    assert_eq!(instantaneo(&mut desfeito), antes);
    // E sobrevive a gravar e abrir.
    let mut aberto = SimWorld::new();
    snapshot_to_world(aberto.world_mut(), &depois, &r).expect("abre");
    let e2 = por_nome(&mut aberto, "Platform High");
    assert_eq!(image_use(aberto.world(), e2), Some(ImageUse::Animated));
}

/// Uma sprite parada nova, como o Add a dá (o quadrado branco a 1 m).
fn parada(sim: &mut SimWorld) -> Entity {
    sim.world_mut()
        .spawn((
            ph2d_ecs::Transform::IDENTITY,
            Sprite::atlas(WHITE_TILE_KEY, [1.0, 1.0], [1.0; 4]),
        ))
        .id()
}

/// ⭐ A mesma imagem dá o mesmo objecto por todas as portas: a linha Image, o seletor e a queda na ranhura
/// (`choose_library_image`) e a queda no canvas (`place_library_image`).
#[test]
fn a_mesma_imagem_pelas_portas_todas() {
    let b = biblioteca();
    for key in [
        PLATFORM_TILE_KEY,
        FLAG_SHEET_KEY,
        ph2d_render::HILLS_TILE_KEY,
        ph2d_render::FIRST_IMPORT_KEY,
    ] {
        let id = b.de(key);
        let mut sim = SimWorld::new();
        // O script decide sobre a sprite ANTES da troca (o override não toca no documento).
        let pelo_script = parada(&mut sim);
        let lei =
            crate::image_choice::override_law(sim.world(), pelo_script, id, &b.db, &b.atlas, PPM)
                .expect("script");
        let pela_linha = parada(&mut sim);
        choose_library_image(
            &mut sim,
            &mut sem_placa,
            &b.db,
            &b.atlas,
            PPM,
            pela_linha,
            id,
        )
        .expect("linha");
        let pelo_canvas = place_library_image(
            &mut sim,
            &mut sem_placa,
            &b.db,
            &b.atlas,
            PPM,
            [0.0; 2],
            "x",
            id,
        )
        .expect("canvas");
        let w = sim.world();
        let ver = |e| {
            (
                w.get::<Sprite>(e).map(|s| (s.source, s.size)),
                w.get::<SpriteGrid>(e).copied(),
                w.get::<SpritePixels>(e).copied(),
            )
        };
        assert_eq!(ver(pela_linha), ver(pelo_canvas), "chave {key}");
        let s = w.get::<Sprite>(pela_linha).unwrap();
        assert_eq!(
            (
                Some(SpriteSource::Atlas {
                    key: lei.atlas_key.expect("no atlas")
                }),
                lei.size,
                lei.grid
            ),
            (
                Some(s.source),
                s.size,
                w.get::<SpriteGrid>(pela_linha)
                    .copied()
                    .unwrap_or(SpriteGrid::SINGLE)
            ),
            "script, chave {key}"
        );
    }
}

/// ⭐ Ficheiro antigo igual (plano GEF 13 D2): um bloco gravado com a grelha das cópias abre sem ela e amostra o
/// MESMO texel (antes a célula do meio da tira, hoje a imagem da chave); a bandeira animada guarda a sua.
#[test]
fn um_bloco_gravado_antes_abre_com_o_mesmo_desenho() {
    let mut sim = nivel(2);
    let bloco = por_nome(&mut sim, "Platform High");
    let bandeira = por_nome(&mut sim, "Goal");
    let antiga = SpriteGrid {
        hframes: PLATFORM_TILE.grid[0],
        vframes: PLATFORM_TILE.grid[1],
        frame: PLATFORM_TILE.middle_frame(),
    };
    sim.world_mut().entity_mut(bloco).insert(antiga);
    let grelha_da_bandeira = sim.world().get::<SpriteGrid>(bandeira).copied();
    let gravado = instantaneo(&mut sim);
    let mut aberto = SimWorld::new();
    snapshot_to_world(aberto.world_mut(), &gravado, &registo_do_jogo()).expect("abre");
    assert_eq!(
        ph2d_app_components::factory_images::forget_extrusion_grids(aberto.world_mut()),
        1
    );
    let b2 = por_nome(&mut aberto, "Platform High");
    let g2 = por_nome(&mut aberto, "Goal");
    assert!(aberto.world().get::<SpriteGrid>(b2).is_none());
    assert_eq!(
        aberto.world().get::<SpriteGrid>(g2).copied(),
        grelha_da_bandeira
    );
    // O texel: a cópia do meio da tira de antes é a imagem de hoje, ponto a ponto.
    let tira = ph2d_render::factory_pixels(&PLATFORM_TILE);
    let imagem = ph2d_render::factory_image(&PLATFORM_TILE);
    let [cw, ch] = PLATFORM_TILE.cell_px();
    for y in (0..ch).step_by(7) {
        for x in (0..cw).step_by(5) {
            let o = ((y * PLATFORM_TILE.px[0] + cw + x) * 4) as usize;
            let n = ((y * cw + x) * 4) as usize;
            assert_eq!(tira[o..o + 4], imagem[n..n + 4], "({x},{y})");
        }
    }
}

/// ⭐ O corte é da IMAGEM (plano GEF 13 D3): editar colunas/linhas num objecto re-corta todos os que mostram a
/// mesma imagem — e a imagem nova que cai noutro objecto já vem cortada.
#[test]
fn re_cortar_um_re_corta_todos_os_que_mostram_a_imagem() {
    let b = biblioteca();
    let id = b.de(ph2d_render::FIRST_IMPORT_KEY);
    let mut sim = SimWorld::new();
    let um = parada(&mut sim);
    let outro = parada(&mut sim);
    let controlo = parada(&mut sim);
    for e in [um, outro] {
        choose_library_image(&mut sim, &mut sem_placa, &b.db, &b.atlas, PPM, e, id)
            .expect("escolhe");
    }
    let quatro = SpriteGrid {
        hframes: 4,
        vframes: 2,
        frame: 0,
    };
    sim.world_mut().entity_mut(um).insert(quatro);
    assert_eq!(recut_users(sim.world_mut(), um), 1);
    let w = sim.world();
    assert_eq!(w.get::<SpriteGrid>(outro).copied(), Some(quatro));
    assert!(
        w.get::<SpriteGrid>(controlo).is_none(),
        "o quadrado branco não mostra a imagem"
    );
    // A porta já sabe o corte: um terceiro objecto recebe a imagem cortada.
    let novo = place_library_image(
        &mut sim,
        &mut sem_placa,
        &b.db,
        &b.atlas,
        PPM,
        [0.0; 2],
        "n",
        id,
    )
    .expect("larga");
    let g = sim
        .world()
        .get::<SpriteGrid>(novo)
        .copied()
        .expect("cortada");
    assert_eq!((g.hframes, g.vframes), (4, 2));
    assert_eq!(
        b.factos(id).slicing,
        None,
        "a biblioteca sozinha não guarda o corte (D3)"
    );
    assert_eq!(
        crate::image_choice::library_facts(sim.world(), id, &b.db, &b.atlas, PPM)
            .and_then(|f| f.slicing)
            .map(|s| s.grid),
        Some([4, 2])
    );
}

//! Os gates do plano GEF 15 (o seletor «Use»): trocar o uso é reversível por inteiro, «Animated» numa folha sem
//! corte propõe a grelha pelas calhas transparentes (sem calhas: 1 × 1, nada inventado), e os níveis gravados
//! abrem com o uso certo.

use super::*;
use crate::image_choice::choose_library_image;
use ph2d_app_components::test_support::registo;
use ph2d_ecs::scene::{WorldSnapshot, snapshot_to_world, world_to_snapshot};
use ph2d_ecs::{Name, SimWorld};
use ph2d_physics_ecs::{AnimRules, CharacterAnimations, Collider, ColliderShape};
use ph2d_render::WHITE_TILE_KEY;

const PPM: f32 = 100.0;
const HEROI_KEY: u32 = ph2d_render::FIRST_IMPORT_KEY;
const MONTES_KEY: u32 = ph2d_render::FIRST_IMPORT_KEY + 1;

fn arte(nome: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/GameEngineFuncional/arte")
        .join(nome)
}

fn biblioteca() -> (AssetDb, BTreeMap<u32, AssetId>) {
    let db = AssetDb::new();
    let mut atlas = BTreeMap::new();
    ph2d_app_components::factory_images::register(&db, &mut atlas);
    for (key, nome) in [
        (HEROI_KEY, "heroi_azul.png"),
        (MONTES_KEY, "montes_por_do_sol.png"),
    ] {
        let id = db.load_png_path(&arte(nome)).expect("a arte abre");
        atlas.insert(key, id);
    }
    (db, atlas)
}

fn sem_placa(_: u32, _: u32, _: &[u8]) -> Result<u32, String> {
    Err("uma imagem da biblioteca não sobe cópia".into())
}

fn instantaneo(sim: &mut SimWorld) -> WorldSnapshot {
    let r = registo();
    let mut prop = ph2d_ecs::TransformPropagationState::new(sim.world_mut());
    let mut work = ph2d_ecs::WorklistBuf::default();
    let mut snap = WorldSnapshot::default();
    world_to_snapshot(sim.world_mut(), &mut prop, &mut work, &r, &mut snap).expect("captura");
    snap
}

fn reaberto(snap: &WorldSnapshot) -> SimWorld {
    let mut sim = SimWorld::new();
    snapshot_to_world(sim.world_mut(), snap, &registo()).expect("repõe");
    sim
}

fn por_nome(sim: &mut SimWorld, nome: &str) -> Entity {
    let w = sim.world_mut();
    let mut q = w.query::<(Entity, &Name)>();
    q.iter(w)
        .find(|(_, n)| n.as_str() == nome)
        .map(|(e, _)| e)
        .unwrap_or_else(|| panic!("{nome}"))
}

fn usa(sim: &mut SimWorld, e: Entity, to: InspectorImageUse) -> bool {
    let (db, atlas) = biblioteca();
    apply_edit(sim.world_mut(), e, &db, &atlas, PPM, ImageUseEdit::Use(to))
}

/// Uma sprite parada com a imagem da biblioteca `key`.
fn parada_com(sim: &mut SimWorld, key: u32) -> Entity {
    let (db, atlas) = biblioteca();
    let e = sim
        .world_mut()
        .spawn((
            ph2d_ecs::Transform::IDENTITY,
            Sprite::atlas(WHITE_TILE_KEY, [1.0, 1.0], [1.0; 4]),
        ))
        .id();
    choose_library_image(sim, &mut sem_placa, &db, &atlas, PPM, e, atlas[&key]).expect("escolhe");
    e
}

/// O que um passo de desfazer tem de devolver: as animações, o animador, as regras e o Tiled.
type Estado = (
    Option<ImageUse>,
    Option<SpriteAnimations>,
    Option<SpriteAnimator>,
    Option<CharacterAnimations>,
    Option<AnimRules>,
    Option<TiledDraw>,
    Option<Sprite>,
);

fn estado(sim: &SimWorld, e: Entity) -> Estado {
    let w = sim.world();
    (
        image_use(w, e),
        w.get::<SpriteAnimations>(e).cloned(),
        w.get::<SpriteAnimator>(e).cloned(),
        w.get::<CharacterAnimations>(e).cloned(),
        w.get::<AnimRules>(e).cloned(),
        w.get::<TiledDraw>(e).copied(),
        w.get::<Sprite>(e).copied(),
    )
}

/// ⭐ D3: trocar o uso do Player (animado, com regras) e de um bloco Tiled; o mundo gravado ANTES (a fila de
/// desfazer guarda-o) devolve tudo.
#[test]
fn trocar_o_uso_e_desfazer_devolve_tudo() {
    for (nome, idas) in [
        (
            "Player",
            [InspectorImageUse::Repeated, InspectorImageUse::Still],
        ),
        (
            "Platform Low",
            [InspectorImageUse::Animated, InspectorImageUse::Still],
        ),
    ] {
        for to in idas {
            let mut sim = SimWorld::new();
            ph2d_app_game::smoke::montar(&mut sim, &registo(), 4);
            let e = por_nome(&mut sim, nome);
            let antes = estado(&sim, e);
            assert!(
                antes.1.is_some() || antes.5.is_some(),
                "{nome}: nada a perder"
            );
            let gravado = instantaneo(&mut sim);
            assert!(usa(&mut sim, e, to), "{nome} → {to:?}");
            let depois = estado(&sim, e);
            assert_ne!(depois.0, antes.0, "{nome} → {to:?}: o uso não mudou");
            assert_eq!(
                depois.0,
                Some(match to {
                    InspectorImageUse::Still => ImageUse::Still,
                    InspectorImageUse::Repeated => ImageUse::Repeated,
                    InspectorImageUse::Animated => ImageUse::Animated,
                })
            );
            // As regras ficam (dormem sem animações) — voltar a animada devolve-as.
            assert_eq!(
                (&depois.3, &depois.4),
                (&antes.3, &antes.4),
                "{nome}: regras"
            );
            let mut desfeito = reaberto(&gravado);
            let e2 = por_nome(&mut desfeito, nome);
            assert_eq!(estado(&desfeito, e2), antes, "{nome} → {to:?}: o desfazer");
        }
    }
}

/// ⭐ D4: «Animated» numa folha importada sem corte propõe a grelha das calhas (heroi_azul: 8 × 4, medido) e
/// cria «loop» com todos os quadros; a célula fica à escala a que a imagem se via.
#[test]
fn animada_numa_folha_sem_corte_propoe_a_grelha() {
    let mut sim = SimWorld::new();
    let e = parada_com(&mut sim, HEROI_KEY);
    let inteira = sim
        .world()
        .get::<Sprite>(e)
        .map(|s| s.size)
        .expect("sprite");
    assert_eq!(image_use(sim.world(), e), Some(ImageUse::Still));
    assert!(usa(&mut sim, e, InspectorImageUse::Animated));
    let w = sim.world();
    let g = w.get::<SpriteGrid>(e).copied().expect("grelha");
    assert_eq!((g.hframes, g.vframes), (8, 4));
    let tag = w
        .get::<SpriteAnimations>(e)
        .and_then(|t| t.iter().next().cloned())
        .expect("loop");
    assert_eq!((tag.name.as_str(), tag.from, tag.to), (LOOP_TAG, 0, 31));
    assert_eq!(
        w.get::<Sprite>(e).map(|s| s.size),
        Some([inteira[0] / 8.0, inteira[1] / 4.0])
    );
}

/// ⭐ D4: sem calhas (os montes: uma figura só) não há proposta — 1 × 1, a lista vazia, o artista corta.
#[test]
fn animada_sem_calhas_fica_1x1_sem_animacao_inventada() {
    let mut sim = SimWorld::new();
    let e = parada_com(&mut sim, MONTES_KEY);
    assert!(usa(&mut sim, e, InspectorImageUse::Animated));
    let w = sim.world();
    assert_eq!(image_use(w, e), Some(ImageUse::Animated));
    assert_eq!(w.get::<SpriteGrid>(e).copied(), Some(SpriteGrid::SINGLE));
    assert_eq!(
        w.get::<SpriteAnimations>(e).map(|t| t.iter().count()),
        Some(0)
    );
    assert!(w.get::<SpriteAnimator>(e).is_some());
}

/// ⭐ D4, a premissa: a proposta só nasce de calhas inteiras.
#[test]
fn a_proposta_so_nasce_de_calhas() {
    let (db, atlas) = biblioteca();
    for (key, quer) in [(HEROI_KEY, Some([8, 4])), (MONTES_KEY, None)] {
        let a = db.get(&atlas[&key]).expect("imagem");
        let (w, h, px) = a.image_rgba8().expect("rgba");
        assert_eq!(proposed_grid(&px, w, h), quer, "{key}");
    }
}

/// ⭐ Gate 6: os níveis gravados abrem com o uso que tinham, sem migração.
#[test]
fn os_niveis_gravados_abrem_com_o_uso_certo() {
    for (n, nome, quer) in [
        (4, "Player", ImageUse::Animated),
        (4, "Platform Low", ImageUse::Repeated),
        (2, "Platform High", ImageUse::Repeated),
        (2, "Goal", ImageUse::Animated),
    ] {
        let mut sim = SimWorld::new();
        ph2d_app_game::smoke::montar(&mut sim, &registo(), n);
        let e = por_nome(&mut sim, nome);
        assert_eq!(image_use(sim.world(), e), Some(quer), "{n} {nome}");
        let mut aberto = reaberto(&instantaneo(&mut sim));
        let e2 = por_nome(&mut aberto, nome);
        assert_eq!(
            image_use(aberto.world(), e2),
            Some(quer),
            "{n} {nome} reaberto"
        );
    }
}

/// Uma camada de fundo repete sempre: o seletor não a muda.
#[test]
fn a_camada_de_fundo_nao_troca_de_uso() {
    let mut sim = SimWorld::new();
    ph2d_app_game::smoke::montar(&mut sim, &registo(), 2);
    let e = por_nome(&mut sim, "Sky");
    if sim.world().get::<ScrollRepeat>(e).is_none() {
        sim.world_mut()
            .entity_mut(e)
            .insert(ScrollRepeat { tile: [1.0, 0.0] });
    }
    for to in InspectorImageUse::ALL {
        assert!(!usa(&mut sim, e, to), "{to:?}");
    }
}

/// Smoke 26/10 (foto da bandeira parada num bloco): passar a parada um bloco que repetia mostra UM ladrilho,
/// à escala a que se via — a imagem nunca estica até ao bloco.
#[test]
fn parada_de_um_bloco_mostra_um_ladrilho() {
    let mut sim = SimWorld::new();
    ph2d_app_game::smoke::montar(&mut sim, &registo(), 4);
    let e = por_nome(&mut sim, "Platform Low");
    let tile = sim
        .world()
        .get::<TiledDraw>(e)
        .map(|t| t.tile)
        .expect("Tiled");
    assert!(usa(&mut sim, e, InspectorImageUse::Still));
    assert_eq!(sim.world().get::<Sprite>(e).map(|s| s.size), Some(tile));
}

/// Dono 27/10: «Still não deveria mostrar a imagem integral?» — passar a parada mostra a folha INTEIRA (sem corte,
/// sem quadro a escolher), à escala a que a célula se via.
#[test]
fn parada_mostra_a_imagem_inteira() {
    let mut sim = SimWorld::new();
    ph2d_app_game::smoke::montar(&mut sim, &registo(), 4);
    let e = por_nome(&mut sim, "Player");
    let g = sim.world().get::<SpriteGrid>(e).copied().expect("folha");
    let celula = sim
        .world()
        .get::<Sprite>(e)
        .map(|s| s.size)
        .expect("sprite");
    assert!(g.hframes * g.vframes > 1);
    assert!(usa(&mut sim, e, InspectorImageUse::Still));
    let w = sim.world();
    assert_eq!(w.get::<SpriteGrid>(e).copied(), Some(SpriteGrid::SINGLE));
    assert_eq!(
        w.get::<Sprite>(e).map(|s| s.size),
        Some([celula[0] * g.hframes as f32, celula[1] * g.vframes as f32])
    );
}

/// Dono 27/10: «por que em Repeated a imagem ficou quadrada e achatada?» — o ladrilho tem de ser o que o objecto
/// MOSTRA (a sua grelha), nunca a célula do corte da imagem: uma parada inteira que passa a repetida repete a
/// imagem inteira, com as proporções dela.
#[test]
fn repetida_de_uma_parada_inteira_nao_achata() {
    let (db, atlas) = biblioteca();
    let bandeira = atlas[&ph2d_render::FLAG_SHEET_KEY];
    let factos = crate::image_choice::image_facts(bandeira, &db, &atlas, PPM).expect("factos");
    let mut sim = SimWorld::new();
    ph2d_app_game::smoke::montar(&mut sim, &registo(), 4);
    let e = por_nome(&mut sim, "Platform High");
    choose_library_image(&mut sim, &mut sem_placa, &db, &atlas, PPM, e, bandeira).expect("escolhe");
    for to in [InspectorImageUse::Still, InspectorImageUse::Repeated] {
        usa(&mut sim, e, to);
    }
    let w = sim.world();
    let g = w
        .get::<SpriteGrid>(e)
        .copied()
        .unwrap_or(SpriteGrid::SINGLE);
    let tile = w.get::<TiledDraw>(e).map(|t| t.tile).expect("Tiled");
    let quer = [
        factos.native_m[0] / g.hframes.max(1) as f32,
        factos.native_m[1] / g.vframes.max(1) as f32,
    ];
    assert_eq!(
        tile, quer,
        "o ladrilho não é o que o objecto mostra (grelha {g:?})"
    );
}

/// ⭐⭐ **A auditoria das trocas** (dono 27/10: «mudando entre os modos a imagem ficou gigante… corrigir tudo de
/// uma vez»; e «cada modo guarda sua própria configuração»). Para cinco objectos e TODA sequência de três trocas:
/// a escala a que a imagem se vê nunca muda; um uso NOVO nasce com o tamanho dele (parada = a imagem inteira ·
/// animada = uma célula · repetida = UM ladrilho, 1 × 1); um uso que JÁ FOI usado volta exactamente como o
/// artista o deixou (o tamanho, o corte, o ladrilho, as animações, as regras).
#[test]
fn nenhuma_troca_de_uso_muda_a_escala() {
    let (db, atlas) = biblioteca();
    let perto = |a: f32, b: f32| (a - b).abs() <= 1e-4 * a.abs().max(b.abs()).max(1.0);
    let preparar: [(&str, u32, Option<u32>); 5] = [
        ("Player", 4, None),
        ("Platform Low", 4, None),
        ("Platform High", 4, Some(ph2d_render::FLAG_SHEET_KEY)),
        ("Platform High", 4, Some(HEROI_KEY)),
        ("Goal", 2, None),
    ];
    for (nome, nivel, imagem) in preparar {
        for a in InspectorImageUse::ALL {
            for b in InspectorImageUse::ALL {
                for c in InspectorImageUse::ALL {
                    let mut sim = SimWorld::new();
                    ph2d_app_game::smoke::montar(&mut sim, &registo(), nivel);
                    let e = por_nome(&mut sim, nome);
                    if let Some(key) = imagem {
                        choose_library_image(
                            &mut sim,
                            &mut sem_placa,
                            &db,
                            &atlas,
                            PPM,
                            e,
                            atlas[&key],
                        )
                        .expect("escolhe");
                    }
                    let factos = |sim: &SimWorld| {
                        shown_facts(sim.world(), e, &db, &atlas, PPM)
                            .or_else(|| own_facts(sim.world(), e))
                            .expect("factos")
                    };
                    let k0 = display_scale(sim.world(), e, factos(&sim));
                    let inicio = image_use(sim.world(), e).expect("uso");
                    let quem = format!("{nome} {imagem:?}: {inicio:?} → {a:?} → {b:?} → {c:?}");
                    // O que cada uso tinha quando foi deixado.
                    let mut deixados: Vec<(ImageUse, Completo)> = Vec::new();
                    for to in [a, b, c] {
                        let de = image_use(sim.world(), e).expect("uso");
                        let antes = completo(&sim, e);
                        // Sem UV nem ladrilho, o tamanho é exactamente o que se via.
                        let visto = (antes.2.is_none() && antes.0.5.is_none())
                            .then(|| antes.0.6.map(|s| s.size))
                            .flatten();
                        if !usa(&mut sim, e, to) {
                            continue;
                        }
                        deixados.retain(|(u, _)| *u != de);
                        deixados.push((de, antes));
                        let w = sim.world();
                        let agora = image_use(w, e).expect("uso");
                        let f = factos(&sim);
                        let k = display_scale(w, e, f);
                        assert!(
                            perto(k[0], k0[0]) && perto(k[1], k0[1]),
                            "{quem}: escala {k0:?} → {k:?} em {to:?}"
                        );
                        if let Some((_, guardado)) = deixados.iter().find(|(u, _)| *u == agora) {
                            assert_eq!(
                                &completo(&sim, e),
                                guardado,
                                "{quem}: {agora:?} não voltou como estava"
                            );
                            continue;
                        }
                        let g = w
                            .get::<SpriteGrid>(e)
                            .copied()
                            .unwrap_or(SpriteGrid::SINGLE);
                        let mostra = [
                            f.native_m[0] / g.hframes.max(1) as f32 * k0[0],
                            f.native_m[1] / g.vframes.max(1) as f32 * k0[1],
                        ];
                        let size = w.get::<Sprite>(e).map(|s| s.size).expect("sprite");
                        if agora == ImageUse::Still {
                            assert_eq!(g, SpriteGrid::SINGLE, "{quem}: parada nova com corte");
                        }
                        if agora == ImageUse::Repeated {
                            // Um ladrilho do que se via (dono 27/10): nada cresce nem encolhe.
                            if let Some(v) = visto {
                                assert!(
                                    perto(size[0], v[0]) && perto(size[1], v[1]),
                                    "{quem}: a repetida nova {size:?} ≠ o que se via {v:?}"
                                );
                            }
                            let tile = w.get::<TiledDraw>(e).map(|t| t.tile).expect("Tiled");
                            assert!(
                                perto(tile[0], mostra[0]) && perto(tile[1], mostra[1]),
                                "{quem}: ladrilho {tile:?} ≠ {mostra:?}"
                            );
                        }
                        // Os três usos novos: o tamanho é o que mostram (a repetida, UM ladrilho).
                        assert!(
                            perto(size[0], mostra[0]) && perto(size[1], mostra[1]),
                            "{quem}: {agora:?} nova {size:?} ≠ {mostra:?}"
                        );
                    }
                }
            }
        }
    }
}

/// Tudo o que um uso pode ter: [`Estado`], o corte, o UV, o *Repeat*, os quadros e o passo.
type Completo = (
    Estado,
    Option<SpriteGrid>,
    Option<UvTransform>,
    Option<TextureRepeat>,
    Option<SpriteAnimFrames>,
    Option<SpriteAnimCursor>,
);

fn completo(sim: &SimWorld, e: Entity) -> Completo {
    let w = sim.world();
    (
        estado(sim, e),
        w.get::<SpriteGrid>(e).copied(),
        w.get::<UvTransform>(e).copied(),
        w.get::<TextureRepeat>(e).copied(),
        w.get::<SpriteAnimFrames>(e).cloned(),
        w.get::<SpriteAnimCursor>(e).copied(),
    )
}

/// ⭐⭐ **Dono 27/10: «se voltar para Animated, não perde nada da configuração já realizada».** O Player: o artista
/// mexe na Animation (uma animação nova, o tamanho), passa a Repeated — que nasce com UM ladrilho, nunca o tamanho
/// da animada — e muda a Largura; volta a Animated: tudo o que tinha; volta a Repeated: a Largura dele. E a memória
/// viaja no projecto gravado (reaberto, a volta devolve o mesmo).
#[test]
fn cada_uso_guarda_a_sua_configuracao() {
    let mut sim = SimWorld::new();
    ph2d_app_game::smoke::montar(&mut sim, &registo(), 4);
    let e = por_nome(&mut sim, "Player");
    assert_eq!(image_use(sim.world(), e), Some(ImageUse::Animated));
    {
        let mut ent = sim.world_mut().entity_mut(e);
        let mut tags = ent.get::<SpriteAnimations>().cloned().expect("animações");
        tags.insert(AnimationTag::new("aceno", 1, 2)).expect("cabe");
        ent.insert(tags);
        let mut s = *ent.get::<Sprite>().expect("sprite");
        s.size = [s.size[0] * 1.5, s.size[1] * 1.5];
        ent.insert(s);
    }
    let animada = completo(&sim, e);
    assert!(usa(&mut sim, e, InspectorImageUse::Repeated));
    let w = sim.world();
    let tile = w.get::<TiledDraw>(e).map(|t| t.tile).expect("Tiled");
    assert_eq!(
        w.get::<Sprite>(e).map(|s| s.size),
        Some(tile),
        "Repeated nasce 1 × 1"
    );
    assert_eq!(
        Some(tile),
        animada.0.6.map(|s| s.size),
        "um ladrilho do boneco à vista"
    );
    let (db, atlas) = biblioteca();
    let largura = tile[0] * 4.0;
    assert!(apply_edit(
        sim.world_mut(),
        e,
        &db,
        &atlas,
        PPM,
        ImageUseEdit::Width(largura)
    ));
    let repetida = completo(&sim, e);
    assert!(usa(&mut sim, e, InspectorImageUse::Animated));
    assert_eq!(completo(&sim, e), animada, "a Animated voltou diferente");
    let gravado = instantaneo(&mut sim);
    let mut sim = reaberto(&gravado);
    let e = por_nome(&mut sim, "Player");
    assert_eq!(completo(&sim, e), animada, "reaberto");
    assert!(usa(&mut sim, e, InspectorImageUse::Repeated));
    assert_eq!(completo(&sim, e), repetida, "a Repeated voltou diferente");
    assert!(usa(&mut sim, e, InspectorImageUse::Still));
    assert!(usa(&mut sim, e, InspectorImageUse::Animated));
    assert_eq!(completo(&sim, e), animada, "depois de passar por Still");
}

/// Trocar a IMAGEM esquece o aspecto que os outros usos guardaram (descrevia a imagem antiga), mas nunca as
/// animações do artista: voltar a Animated corta a imagem nova e devolve a lista.
#[test]
fn trocar_a_imagem_guarda_as_animacoes_e_esquece_o_aspecto() {
    let (db, atlas) = biblioteca();
    let mut sim = SimWorld::new();
    ph2d_app_game::smoke::montar(&mut sim, &registo(), 4);
    let e = por_nome(&mut sim, "Player");
    let tags = sim.world().get::<SpriteAnimations>(e).cloned();
    assert!(usa(&mut sim, e, InspectorImageUse::Still));
    choose_library_image(
        &mut sim,
        &mut sem_placa,
        &db,
        &atlas,
        PPM,
        e,
        atlas[&HEROI_KEY],
    )
    .expect("escolhe");
    let memoria = sim
        .world()
        .get::<ImageUseMemory>(e)
        .cloned()
        .expect("memória");
    assert!(memoria.animated.as_ref().is_some_and(|m| m.look.is_none()));
    assert!(usa(&mut sim, e, InspectorImageUse::Animated));
    let w = sim.world();
    assert_eq!(w.get::<SpriteAnimations>(e).cloned(), tags);
    let g = w.get::<SpriteGrid>(e).copied().expect("grelha");
    assert_eq!((g.hframes, g.vframes), (8, 4), "o corte da imagem nova");
}

/// O collider automático (a caixa que media o desenho) segue o tamanho de cada uso; um mudado à mão fica.
#[test]
fn o_collider_automatico_segue_o_uso() {
    let mut sim = SimWorld::new();
    ph2d_app_game::smoke::montar(&mut sim, &registo(), 4);
    let e = por_nome(&mut sim, "Platform High");
    let caixa = |sim: &SimWorld| match sim.world().get::<Collider>(e).map(|c| c.shape) {
        Some(ColliderShape::Cuboid { half_x, half_y }) => [half_x * 2.0, half_y * 2.0],
        outro => panic!("{outro:?}"),
    };
    let size = |sim: &SimWorld| {
        sim.world()
            .get::<Sprite>(e)
            .map(|s| s.size)
            .expect("sprite")
    };
    assert_eq!(caixa(&sim), size(&sim), "a premissa: automático");
    for to in [
        InspectorImageUse::Still,
        InspectorImageUse::Animated,
        InspectorImageUse::Repeated,
    ] {
        usa(&mut sim, e, to);
        let (c, s) = (caixa(&sim), size(&sim));
        assert!(
            (c[0] - s[0]).abs() < 1e-4 && (c[1] - s[1]).abs() < 1e-4,
            "{to:?}: {c:?} ≠ {s:?}"
        );
    }
    sim.world_mut()
        .get_mut::<Collider>(e)
        .expect("collider")
        .shape = ColliderShape::Cuboid {
        half_x: 9.0,
        half_y: 9.0,
    };
    usa(&mut sim, e, InspectorImageUse::Still);
    assert_eq!(caixa(&sim), [18.0, 18.0], "à mão fica");
}

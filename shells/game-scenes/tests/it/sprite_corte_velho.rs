//! A costura da ficha do cartão da Sprite e do Animated: Keep old size, o vazio, o herói e o aviso de imagem trocada.

use ph2d_app_sprite_anim::sprite_card::{cut_notice, keep_size, try_sample_hero};
use ph2d_asset::{AssetDb, AssetId};
use ph2d_ecs::{Entity, SpriteAnimations, SpriteCutNotice, SpriteGrid, World};
use ph2d_editor_core::ImageCutNotice;
use ph2d_render::Sprite;
use ph2d_sprite_edit::vazio::sem_imagem;
use std::collections::BTreeMap;

#[test]
fn keep_size_devolve_o_tamanho_de_antes() {
    let mut w = World::new();
    let a = w
        .spawn(Sprite::atlas(
            ph2d_render::WHITE_TILE_KEY,
            [3.0, 3.0],
            [1.0; 4],
        ))
        .id();
    assert!(keep_size(&mut w, a, [8.0, 2.0]));
    assert_eq!(w.get::<Sprite>(a).unwrap().size, [8.0, 2.0]);
    assert!(!keep_size(&mut w, a, [8.0, 2.0]));
}

const PPM: f32 = 100.0;

/// Uma imagem de `w × h` px cujas células de `cw × ch` px têm um ponto opaco e o resto transparente.
fn folha(db: &AssetDb, w: u32, h: u32, cw: u32, ch: u32) -> AssetId {
    let mut px = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w {
            if x % cw < cw - 2 && y % ch < ch - 2 {
                px[((y * w + x) * 4 + 3) as usize] = 255;
            }
        }
    }
    db.insert_image_rgba8(w, h, px)
}

fn escolhe(sim: &mut ph2d_ecs::SimWorld, db: &AssetDb, e: Entity, id: AssetId) {
    ph2d_app_sprite_anim::image_choice::choose_library_image(
        sim,
        &mut |_, _, _| Ok(9),
        db,
        &BTreeMap::new(),
        PPM,
        e,
        id,
    )
    .expect("a imagem entra");
}

/// O Animated Sprite como `Add ▸ Animated Sprite` o deixa.
fn vazio(sim: &mut ph2d_ecs::SimWorld) -> Entity {
    let mut a = ph2d_ecs::SpriteAnimator::new("");
    a.autoplay = true;
    sim.world_mut()
        .spawn((
            Sprite::atlas(ph2d_render::WHITE_TILE_KEY, [1.0, 1.0], [1.0; 4]),
            SpriteAnimations::new(),
            a,
        ))
        .id()
}

fn grade(w: &World, e: Entity) -> [u32; 2] {
    let g = w.get::<SpriteGrid>(e).expect("a grelha");
    [g.hframes, g.vframes]
}

/// ⭐ Escolher a imagem no vazio já entra em Animated: cortada pela transparência, tocando todas as células.
/// *Mutação: tirar a proposta de `choose_library_image` ⇒ a folha fica numa célula só.*
#[test]
fn escolher_a_imagem_no_vazio_ja_entra_em_animated() {
    let (mut sim, db) = (ph2d_ecs::SimWorld::new(), AssetDb::new());
    let e = vazio(&mut sim);
    escolhe(&mut sim, &db, e, folha(&db, 128, 64, 32, 32));
    let w = sim.world();
    assert_eq!(grade(w, e), [4, 2]);
    let lib = w.get::<SpriteAnimations>(e).expect("a lista");
    let t = lib
        .get(ph2d_app_sprite_anim::image_use::LOOP_TAG)
        .expect("o laço");
    assert_eq!((t.from, t.to), (0, 7));
    let a = w.get::<ph2d_ecs::SpriteAnimator>(e).expect("o tocador");
    assert_eq!(a.current, ph2d_app_sprite_anim::image_use::LOOP_TAG);
    assert!(a.autoplay);
    assert!(!sem_imagem(w, e), "deixou de ser o convite");
    assert!(w.get::<SpriteCutNotice>(e).is_none(), "nada a avisar");
}

#[test]
fn o_heroi_de_amostra_entra_no_vazio_e_so_no_vazio() {
    let mut w = World::new();
    let e = vazio_no(&mut w);
    assert!(try_sample_hero(&mut w, e));
    assert_eq!(
        grade(&w, e),
        [ph2d_render::HERO_SHEET_COLS, ph2d_render::HERO_SHEET_ROWS]
    );
    assert_eq!(
        w.get::<SpriteAnimations>(e)
            .expect("a lista")
            .iter()
            .count(),
        ph2d_render::HERO_SHEET_TAGS.len()
    );
    assert!(!try_sample_hero(&mut w, e), "só o vazio o recebe");
}

fn vazio_no(w: &mut World) -> Entity {
    w.spawn((
        Sprite::atlas(ph2d_render::WHITE_TILE_KEY, [1.0, 1.0], [1.0; 4]),
        SpriteAnimations::new(),
        ph2d_ecs::SpriteAnimator::new(""),
    ))
    .id()
}

/// Uma animada de 8 × 4 com 6 animações, a mostrar `id`.
fn animada(sim: &mut ph2d_ecs::SimWorld, db: &AssetDb, id: AssetId) -> Entity {
    let e = vazio(sim);
    escolhe(sim, db, e, id);
    let w = sim.world_mut();
    let mut lib = SpriteAnimations::new();
    for i in 0..6 {
        lib.insert(ph2d_ecs::AnimationTag::new(format!("a{i}"), i, i))
            .unwrap();
    }
    w.entity_mut(e).insert(lib);
    e
}

/// ⭐ Trocar por imagem sem corte salvo acende o aviso com os números reais; refazer o corte à mão cala-o.
/// *Mutação: tirar `notice_of_the_swap` ⇒ a troca fica muda.*
#[test]
fn trocar_por_imagem_sem_corte_acende_o_aviso_com_os_numeros() {
    let (mut sim, db) = (ph2d_ecs::SimWorld::new(), AssetDb::new());
    let a = folha(&db, 800, 400, 100, 100);
    let e = animada(&mut sim, &db, a);
    assert_eq!(grade(sim.world(), e), [8, 4]);
    let atlas = BTreeMap::new();
    assert!(cut_notice(sim.world(), e, &db, &atlas, PPM).is_none());
    escolhe(&mut sim, &db, e, folha(&db, 256, 128, 64, 64));
    assert_eq!(grade(sim.world(), e), [8, 4], "o corte velho ficou");
    assert_eq!(
        cut_notice(sim.world(), e, &db, &atlas, PPM),
        Some(ImageCutNotice {
            grid: [8, 4],
            cell_px: [32, 32],
            animations: 6,
        })
    );
    let w = sim.world_mut();
    w.get_mut::<SpriteGrid>(e).expect("grelha").hframes = 2;
    assert!(
        cut_notice(sim.world(), e, &db, &atlas, PPM).is_none(),
        "outro corte, aviso calado"
    );
}

/// A imagem que já traz corte (outro objecto a usa cortada) não avisa e tira o aviso de antes.
#[test]
fn imagem_que_traz_corte_nao_avisa() {
    let (mut sim, db) = (ph2d_ecs::SimWorld::new(), AssetDb::new());
    let a = folha(&db, 800, 400, 100, 100);
    let cortada = folha(&db, 256, 128, 64, 64);
    let outro = vazio(&mut sim);
    escolhe(&mut sim, &db, outro, cortada);
    let e = animada(&mut sim, &db, a);
    escolhe(&mut sim, &db, e, folha(&db, 300, 100, 100, 100));
    assert!(sim.world().get::<SpriteCutNotice>(e).is_some());
    escolhe(&mut sim, &db, e, cortada);
    let w = sim.world();
    assert_eq!(grade(w, e), [4, 2], "o corte veio com a imagem");
    assert!(w.get::<SpriteCutNotice>(e).is_none());
}

/// ⭐ **Cut again** aplica a grade proposta da imagem nova; **Keep the cut** fica com a velha. Os dois fecham o aviso.
#[test]
fn as_duas_saidas_do_aviso() {
    use ph2d_app_sprite_anim::image_use::apply_edit;
    use ph2d_editor_core::ImageUseEdit as V;
    let atlas = BTreeMap::new();
    for (verbo, esperada) in [(V::CutAgain, [4, 2]), (V::KeepCut, [8, 4])] {
        let (mut sim, db) = (ph2d_ecs::SimWorld::new(), AssetDb::new());
        let a = folha(&db, 800, 400, 100, 100);
        let e = animada(&mut sim, &db, a);
        escolhe(&mut sim, &db, e, folha(&db, 256, 128, 64, 64));
        let w = sim.world_mut();
        assert!(apply_edit(w, e, &db, &atlas, PPM, verbo));
        assert_eq!(grade(w, e), esperada);
        assert!(w.get::<SpriteCutNotice>(e).is_none());
        assert!(
            !apply_edit(w, e, &db, &atlas, PPM, verbo),
            "sem aviso, nada"
        );
    }
}

/// Imagem sem grade a propor: o Cut again abre o editor, onde o corte se faz à mão.
#[test]
fn cut_again_sem_proposta_abre_o_editor() {
    use ph2d_app_sprite_anim::image_use::apply_edit;
    let atlas = BTreeMap::new();
    let (mut sim, db) = (ph2d_ecs::SimWorld::new(), AssetDb::new());
    let a = folha(&db, 800, 400, 100, 100);
    let e = animada(&mut sim, &db, a);
    let cheia = db.insert_image_rgba8(64, 64, vec![255u8; 64 * 64 * 4]);
    escolhe(&mut sim, &db, e, cheia);
    let w = sim.world_mut();
    assert!(apply_edit(
        w,
        e,
        &db,
        &atlas,
        PPM,
        ph2d_editor_core::ImageUseEdit::CutAgain
    ));
    assert_eq!(grade(w, e), [8, 4], "nada a propor, o corte fica");
    let pedidos = w.get_resource::<ph2d_app_sprite_anim::editor::AnimEditorRequests>();
    assert_eq!(pedidos.map(|r| r.0.clone()), Some(vec![e.to_bits()]));
    assert!(
        cut_notice(w, e, &db, &atlas, PPM).is_some(),
        "fechar o editor sem cortar não perde o aviso"
    );
    corta(&mut sim, e, 4, 2);
    assert!(sim.world().get::<SpriteCutNotice>(e).is_none(), "cortou");
}

fn corta(sim: &mut ph2d_ecs::SimWorld, e: Entity, h: u32, v: u32) {
    use ph2d_ecs::scene::{EditorCommandQueue, apply_editor_commands};
    let (q, r) = (
        EditorCommandQueue::new(),
        ph2d_app_components::test_support::registo(),
    );
    let edit = ph2d_editor_core::AnimFieldEdit::Grid(h, v);
    let _ = ph2d_app_sprite_anim::inspector::apply_anim_edit(sim, e.to_bits(), &edit, &q, &r);
    apply_editor_commands(sim.world_mut(), &q, &r).expect("o commit aplica");
}

/// ⭐ A grade mudou = aviso removido: voltar à grade antiga não o ressuscita.
/// *Mutação: só esconder pelo `aviso.grid != grade` ⇒ o aviso volta com o 8 × 4.*
#[test]
fn voltar_a_grade_antiga_nao_ressuscita_o_aviso() {
    let atlas = BTreeMap::new();
    let (mut sim, db) = (ph2d_ecs::SimWorld::new(), AssetDb::new());
    let a = folha(&db, 800, 400, 100, 100);
    let e = animada(&mut sim, &db, a);
    escolhe(&mut sim, &db, e, folha(&db, 256, 128, 64, 64));
    assert!(cut_notice(sim.world(), e, &db, &atlas, PPM).is_some());
    corta(&mut sim, e, 4, 2);
    corta(&mut sim, e, 8, 4);
    assert_eq!(grade(sim.world(), e), [8, 4]);
    assert!(sim.world().get::<SpriteCutNotice>(e).is_none());
    assert!(cut_notice(sim.world(), e, &db, &atlas, PPM).is_none());
}

/// O herói recusado não some em silêncio: a shell lê a recusa, uma vez, e diz ao artista.
/// *Mutação: engolir o `Err` ⇒ não há o que ler.*
#[test]
fn a_recusa_do_heroi_vira_aviso() {
    use ph2d_app_sprite_anim::sprite_card::{toma_a_recusa, veste_com};
    let mut w = World::new();
    let e = vazio_no(&mut w);
    assert!(!veste_com(&mut w, e, |_, _| Err("tabela".to_owned())));
    assert!(toma_a_recusa(&mut w).is_some_and(|m| m.contains("tabela")));
    assert_eq!(toma_a_recusa(&mut w), None, "lê-se uma vez");
    assert!(sem_imagem(&w, e), "o vazio fica");
}

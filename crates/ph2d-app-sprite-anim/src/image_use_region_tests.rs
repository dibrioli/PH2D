//! Os gates do uso **Region** ([plano GEF 16](../../../docs/GameEngineFuncional/16_uso_region_plano.md) D1): uma
//! Region nova é a imagem inteira e pede o editor; o OK põe a área à MESMA escala, dentro da imagem; o render
//! amostra só a área; trocar a imagem esquece a área. Tudo pela porta do clique (`apply_edit`).

use super::*;

fn regiao(sim: &mut SimWorld, e: Entity, rect: [u32; 4], grid: [u32; 2]) -> bool {
    let (db, atlas) = biblioteca();
    apply_edit(
        sim.world_mut(),
        e,
        &db,
        &atlas,
        PPM,
        ImageUseEdit::SetRegion { rect, grid },
    )
}

fn factos(sim: &SimWorld, e: Entity) -> ImageFacts {
    let (db, atlas) = biblioteca();
    shown_facts(sim.world(), e, &db, &atlas, PPM).expect("factos")
}

fn pedidos(sim: &mut SimWorld) -> Vec<u64> {
    sim.world_mut()
        .get_resource_mut::<RegionEditorRequests>()
        .map(|mut r| std::mem::take(&mut r.0))
        .unwrap_or_default()
}

fn perto(a: [f32; 2], b: [f32; 2]) -> bool {
    a.iter()
        .zip(b)
        .all(|(x, y)| (x - y).abs() <= 1e-4 * x.abs().max(1.0))
}

/// ⭐ O Player (folha 8 × 4): Region nova = a folha inteira, sem área, e o editor pede-se sozinho; a grelha de
/// snap nasce do corte da imagem. A célula (1, 0) escolhida: a sprite mostra UMA célula do mesmo tamanho a que a
/// animada a mostrava, o Inspector lê os píxeis dela, e o collider automático segue.
#[test]
fn a_area_escolhida_mostra_so_a_celula_a_mesma_escala() {
    let mut sim = SimWorld::new();
    ph2d_app_game::smoke::montar(&mut sim, &registo(), 4);
    let e = por_nome(&mut sim, "Player");
    let f = factos(&sim, e);
    let k0 = display_scale(sim.world(), e, f);
    let celula_animada = sim
        .world()
        .get::<Sprite>(e)
        .map(|s| s.size)
        .expect("sprite");
    pedidos(&mut sim);
    assert!(usa(&mut sim, e, InspectorImageUse::Region));
    assert_eq!(
        pedidos(&mut sim),
        vec![e.to_bits()],
        "Region nova abre o editor"
    );
    let w = sim.world();
    assert_eq!(w.get::<SpriteRegion>(e), None, "a imagem inteira");
    assert_eq!(w.get::<ImageRegion>(e), Some(&ImageRegion { grid: [8, 4] }));
    let inteira = w.get::<Sprite>(e).map(|s| s.size).expect("sprite");
    assert!(perto(
        inteira,
        [f.native_m[0] * k0[0], f.native_m[1] * k0[1]]
    ));

    let [pw, ph] = f.px;
    let (cw, ch) = (pw / 8, ph / 4);
    assert!(regiao(&mut sim, e, [cw, 0, cw, ch], [8, 4]));
    let w = sim.world();
    let size = w.get::<Sprite>(e).map(|s| s.size).expect("sprite");
    assert!(
        perto(size, celula_animada),
        "{size:?} ≠ a célula {celula_animada:?}"
    );
    let k = display_scale(w, e, f);
    assert!(perto(k, k0), "escala {k0:?} → {k:?}");
    assert_eq!(
        inspector_use(w, e, &biblioteca().0, &biblioteca().1, PPM).2,
        InspectorDraw::Region { px: [cw, ch] }
    );
    // A mesma área de novo: nada muda.
    assert!(!regiao(&mut sim, e, [cw, 0, cw, ch], [8, 4]));
}

/// O collider em caixa automático (Platform High) segue o tamanho da área.
#[test]
fn o_collider_automatico_segue_a_area() {
    let mut sim = SimWorld::new();
    ph2d_app_game::smoke::montar(&mut sim, &registo(), 4);
    let e = por_nome(&mut sim, "Platform High");
    assert!(usa(&mut sim, e, InspectorImageUse::Region));
    let [pw, ph] = factos(&sim, e).px;
    assert!(regiao(&mut sim, e, [0, 0, pw / 2, ph], [2, 1]));
    let w = sim.world();
    let size = w.get::<Sprite>(e).map(|s| s.size).expect("sprite");
    match w.get::<Collider>(e).map(|c| c.shape) {
        Some(ColliderShape::Cuboid { half_x, half_y }) => {
            assert!(
                perto([half_x * 2.0, half_y * 2.0], size),
                "o collider não seguiu"
            );
        }
        outro => panic!("{outro:?}"),
    }
}

/// ⭐ O render amostra SÓ a área: o `SpriteRegion` que o OK escreve, pela lei do extract (`region_subrect`), põe o
/// píxel do canto da área no canto do objecto.
#[test]
fn o_render_amostra_so_a_area() {
    let mut sim = SimWorld::new();
    ph2d_app_game::smoke::montar(&mut sim, &registo(), 4);
    let e = por_nome(&mut sim, "Player");
    assert!(usa(&mut sim, e, InspectorImageUse::Region));
    let [pw, ph] = factos(&sim, e).px;
    assert!(regiao(
        &mut sim,
        e,
        [pw / 8 * 3, ph / 4, pw / 8, ph / 4],
        [8, 4]
    ));
    let r = sim.world().get::<SpriteRegion>(e).copied().expect("a área");
    let uv = ph2d_render::sprite::region_subrect(
        [0.0, 0.0, 1.0, 1.0],
        r.rect,
        pw as f32,
        ph as f32,
        None,
    );
    let quase = |a: [f32; 4], b: [f32; 4]| a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-6);
    assert!(quase(uv, [0.375, 0.25, 0.5, 0.5]), "{uv:?}");
    assert!(r.filter_clip, "a borda não puxa a célula vizinha");
}

/// A área fica dentro da imagem e com 1 px pelo menos, qualquer que seja o retângulo pedido.
#[test]
fn a_area_nunca_sai_da_imagem() {
    let mut sim = SimWorld::new();
    ph2d_app_game::smoke::montar(&mut sim, &registo(), 4);
    let e = por_nome(&mut sim, "Player");
    assert!(usa(&mut sim, e, InspectorImageUse::Region));
    let [pw, ph] = factos(&sim, e).px;
    for (pedido, esperado) in [
        ([pw - 10, ph - 10, 500, 500], [pw - 10, ph - 10, 10, 10]),
        ([0, 0, 0, 0], [0, 0, 1, 1]),
        ([pw + 7, ph + 7, 3, 3], [pw - 1, ph - 1, 1, 1]),
    ] {
        regiao(&mut sim, e, pedido, [1, 1]);
        let r = sim
            .world()
            .get::<SpriteRegion>(e)
            .map(|r| r.rect)
            .expect("área");
        assert_eq!(r, esperado.map(|v| v as f32), "pedido {pedido:?}");
    }
}

/// «Edit Region» só pede o editor em Region; voltar a uma Region guardada não o pede (já tem área); o OK fora de
/// Region não escreve nada.
#[test]
fn o_editor_pede_se_so_em_region() {
    let (db, atlas) = biblioteca();
    let mut sim = SimWorld::new();
    ph2d_app_game::smoke::montar(&mut sim, &registo(), 4);
    let e = por_nome(&mut sim, "Player");
    let editar = |sim: &mut SimWorld| {
        apply_edit(
            sim.world_mut(),
            e,
            &db,
            &atlas,
            PPM,
            ImageUseEdit::EditRegion,
        )
    };
    assert!(!editar(&mut sim));
    assert!(
        !regiao(&mut sim, e, [0, 0, 10, 10], [1, 1]),
        "OK fora de Region"
    );
    assert_eq!(pedidos(&mut sim), Vec::<u64>::new());
    assert!(usa(&mut sim, e, InspectorImageUse::Region));
    pedidos(&mut sim);
    editar(&mut sim);
    assert_eq!(pedidos(&mut sim), vec![e.to_bits()]);
    assert!(usa(&mut sim, e, InspectorImageUse::Still));
    assert!(usa(&mut sim, e, InspectorImageUse::Region));
    assert_eq!(
        pedidos(&mut sim),
        Vec::<u64>::new(),
        "a Region guardada volta sem o editor"
    );
}

/// Trocar a imagem de uma Region: a área descrevia a imagem antiga e vai-se (a imagem nova inteira, à escala de
/// antes); o uso fica Region; a área guardada na memória também se esquece.
#[test]
fn trocar_a_imagem_esquece_a_area() {
    let (db, atlas) = biblioteca();
    let mut sim = SimWorld::new();
    ph2d_app_game::smoke::montar(&mut sim, &registo(), 4);
    let e = por_nome(&mut sim, "Player");
    assert!(usa(&mut sim, e, InspectorImageUse::Region));
    let [pw, ph] = factos(&sim, e).px;
    assert!(regiao(&mut sim, e, [0, 0, pw / 8, ph / 4], [8, 4]));
    let k0 = display_scale(sim.world(), e, factos(&sim, e));
    let flag = atlas[&ph2d_render::FLAG_SHEET_KEY];
    choose_library_image(&mut sim, &mut sem_placa, &db, &atlas, PPM, e, flag).expect("escolhe");
    let w = sim.world();
    assert_eq!(image_use(w, e), Some(ImageUse::Region));
    assert_eq!(w.get::<SpriteRegion>(e), None, "a área da folha antiga");
    let f = factos(&sim, e);
    let size = w.get::<Sprite>(e).map(|s| s.size).expect("sprite");
    assert!(
        perto(size, [f.native_m[0] * k0[0], f.native_m[1] * k0[1]]),
        "a bandeira inteira à escala de antes: {size:?}"
    );
    assert!(usa(&mut sim, e, InspectorImageUse::Still));
    let m = sim
        .world()
        .get::<ImageUseMemory>(e)
        .cloned()
        .expect("memória");
    assert!(m.region.is_some(), "a Region nova (da bandeira) guardou-se");
    assert_eq!(m.region.and_then(|r| r.area), None);
}

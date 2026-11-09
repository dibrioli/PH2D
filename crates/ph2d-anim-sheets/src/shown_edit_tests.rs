use super::*;
use ph2d_ecs::{ExtraSheet, ShownSheet};

/// Um objeto com a folha `0` e a folha `run` (12 × 4 px, corte 3 × 2, célula de 1,5 m), no ponto `(10, 20)`.
fn com_run(db: &AssetDb) -> (World, Entity, AssetId) {
    let run = db.insert_image_rgba8(12, 4, vec![200; 12 * 4 * 4]);
    let mut w = World::new();
    let e = w
        .spawn((
            {
                let mut t = Transform::default();
                t.translation.x = 10.0;
                t.translation.y = 20.0;
                t
            },
            ph2d_render::Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
            SpriteSheets(
                [ExtraSheet {
                    asset: run,
                    size: [1.5, 1.5],
                    anchor: [0.25, 0.0],
                    hframes: 3,
                    vframes: 2,
                    offsets: Default::default(),
                }]
                .into_iter()
                .collect(),
            ),
        ))
        .id();
    (w, e, run)
}

/// ⭐ Com `run` à vista o verbo lê os píxeis DELA, com o tamanho da imagem inteira (célula × corte); com a folha `0`
/// à vista, nada responde. *Mutações: ignorar a folha visível ⇒ lida com a 0 à vista; `whole` = a célula ⇒ 1,5.*
#[test]
fn o_verbo_le_a_folha_visivel_inteira() {
    let db = AssetDb::default();
    let (mut w, e, _) = com_run(&db);
    assert_eq!(read_shown(&w, e, &db), ShownRead::First);
    w.entity_mut(e).insert(ShownSheet(1));
    let ShownRead::Extra(r) = read_shown(&w, e, &db) else {
        panic!("a folha run")
    };
    assert_eq!((r.sheet, r.w, r.h, r.rgba[0]), (1, 12, 4, 200));
    assert_eq!(r.whole, [4.5, 3.0]);
    assert_eq!(r.anchor, [0.25, 0.0]);
}

/// ⭐ Gravar troca a imagem DA folha e devolve o tamanho à célula: o Trim que deixa a imagem com 9 × 4 px a 3 m
/// de largo dá células de 1 m. *Mutações: gravar sem trocar o nome ⇒ continua `run`; não dividir pelo corte ⇒ 3 m.*
#[test]
fn gravar_troca_a_imagem_da_folha_e_volta_a_celula() {
    let db = AssetDb::default();
    let (mut w, e, run) = com_run(&db);
    let novo = db.insert_image_rgba8(9, 4, vec![9; 9 * 4 * 4]);
    assert_ne!(novo, run);
    assert!(set_image(&mut w, e, 1, novo, [3.0, 3.0]));
    let s = w.get::<SpriteSheets>(e).unwrap().0[0].clone();
    assert_eq!(s.asset, novo);
    assert_eq!(s.size, [1.0, 1.5]);
    assert!(
        !set_image(&mut w, e, 2, novo, [1.0, 1.0]),
        "a folha 2 não existe"
    );
}

/// ⭐ Na folha extra o objeto NÃO anda (é de todas as folhas): o que ele andaria vai para o Pivot dela; na folha `0`
/// o objeto anda como sempre. *Mutações: mexer o `Transform` na folha extra ⇒ o objeto sai de (10, 20); somar com o
/// sinal trocado ⇒ o Pivot vai para −0,75.*
#[test]
fn na_folha_extra_quem_anda_e_o_pivot_dela() {
    let db = AssetDb::default();
    let (mut w, e, _) = com_run(&db);
    place(&mut w, e, 1, [[10.0, 20.0], [11.0, 19.5]], None);
    let t = w.get::<Transform>(e).unwrap().translation;
    assert_eq!((t.x, t.y), (10.0, 20.0), "o objeto fica");
    assert_eq!(w.get::<SpriteSheets>(e).unwrap().0[0].anchor, [1.25, -0.5]);
    // Com um Pivot dado pelo verbo (o Padding), é dele que se parte.
    place(&mut w, e, 1, [[10.0, 20.0], [10.5, 20.0]], Some([0.0, 0.0]));
    assert_eq!(w.get::<SpriteSheets>(e).unwrap().0[0].anchor, [0.5, 0.0]);
    place(&mut w, e, 0, [[10.0, 20.0], [12.0, 21.0]], Some([0.1, 0.2]));
    let t = w.get::<Transform>(e).unwrap().translation;
    assert_eq!((t.x, t.y), (12.0, 21.0));
    assert_eq!(w.get::<ph2d_render::Sprite>(e).unwrap().anchor, [0.1, 0.2]);
}

/// O objeto de `com_run` vira peça LIGADA (`Alt+D`) do mestre `m`, e `solta` uma cópia NÃO ligada do mesmo mestre.
fn mestre_e_copias(w: &mut World, e: Entity) -> (Entity, Entity) {
    let (t, s, sheets) = (
        *w.get::<Transform>(e).unwrap(),
        *w.get::<ph2d_render::Sprite>(e).unwrap(),
        w.get::<SpriteSheets>(e).unwrap().clone(),
    );
    let m = w.spawn((t, s, sheets.clone(), ph2d_ecs::StableId(77))).id();
    let solta = w
        .spawn((t, s, sheets, ph2d_ecs::InstanceOf { master: 77 }))
        .id();
    w.entity_mut(e)
        .insert((ph2d_ecs::InstanceOf { master: 77 }, ph2d_ecs::LinkedArt));
    (m, solta)
}

/// ⭐ **A arte LIGADA sobe ao mestre também na folha extra** (doc 27 §3 item 2), como os píxeis da folha 0
/// (`instance_paint_seam_tests::painting_one_copy_reaches_the_others`): a imagem e o Pivot da folha vão à cadeia
/// inteira, e o passe de sync lê «o mestre mexeu», não um override. A cópia não ligada fica com a dela, e na folha 0 o
/// `Transform` continua só do objeto. *Mutação: a cadeia devolve só o objeto ⇒ o mestre fica com `run` e reprova.*
#[test]
fn a_folha_extra_de_uma_copia_ligada_sobe_ao_mestre() {
    let db = AssetDb::default();
    let (mut w, e, run) = com_run(&db);
    let (m, solta) = mestre_e_copias(&mut w, e);
    let novo = db.insert_image_rgba8(9, 4, vec![9; 9 * 4 * 4]);
    assert!(set_image(&mut w, e, 1, novo, [3.0, 3.0]));
    let folha = |w: &World, x: Entity| w.get::<SpriteSheets>(x).unwrap().0[0].clone();
    assert_eq!(
        (folha(&w, m).asset, folha(&w, m).size),
        (novo, [1.0, 1.5]),
        "o mestre recebe a imagem"
    );
    assert_eq!(
        folha(&w, solta).asset,
        run,
        "a cópia não ligada fica com a dela"
    );
    place(&mut w, e, 1, [[10.0, 20.0], [11.0, 19.5]], None);
    assert_eq!(
        folha(&w, m).anchor,
        [1.25, -0.5],
        "o Pivot da folha sobe também"
    );
    assert_eq!(folha(&w, solta).anchor, [0.25, 0.0]);
    place(&mut w, e, 0, [[10.0, 20.0], [12.0, 21.0]], Some([0.1, 0.2]));
    let t = w.get::<Transform>(m).unwrap().translation;
    assert_eq!((t.x, t.y), (10.0, 20.0), "folha 0: o objeto anda sozinho");
    assert_eq!(w.get::<ph2d_render::Sprite>(m).unwrap().anchor, [0.0, 0.0]);
}

/// ⭐ Auditoria do fecho 08/11 (C): a folha extra à vista sem imagem legível é `Unreadable`, nunca a folha `0` que o
/// artista não vê. *Mutação: devolver `First` sem o recurso ⇒ o verbo editava a folha 0.*
#[test]
fn a_folha_extra_ilegivel_nao_cai_na_folha_zero() {
    let db = AssetDb::default();
    let (mut w, e, _) = com_run(&db);
    w.entity_mut(e).insert(ShownSheet(1));
    let mut sheets = w.get::<SpriteSheets>(e).unwrap().clone();
    sheets.0[0].asset = AssetId::from_digest([7; 32]);
    w.entity_mut(e).insert(sheets);
    assert_eq!(read_shown(&w, e, &db), ShownRead::Unreadable);
    // Há recurso, mas não é uma imagem que o verbo leia (textura comprimida): `Unreadable` também, nunca a folha 0.
    // *Mutação: `First` quando a imagem não descodifica ⇒ o verbo editava a folha 0.*
    let mut sheets = w.get::<SpriteSheets>(e).unwrap().clone();
    sheets.0[0].asset = db.insert_ktx2_bytes(ph2d_asset::TierIndex::DESKTOP, &[1, 2, 3]);
    w.entity_mut(e).insert(sheets);
    assert_eq!(read_shown(&w, e, &db), ShownRead::Unreadable);
}

/// ⭐ Auditoria do fecho 08/11 (I): uma folha de 16 bits diz-se, como a folha 0; a de 8 e a folha 0 não.
/// *Mutação: `false` sempre ⇒ a perda passa calada.*
#[test]
fn a_folha_de_16_bits_diz_que_perde_precisao() {
    let db = AssetDb::default();
    let (mut w, e, _) = com_run(&db);
    assert!(!sheet_is_16_bit(&w, e, 1, &db));
    let mut sheets = w.get::<SpriteSheets>(e).unwrap().clone();
    sheets.0[0].asset = db.insert_image_rgba16(12, 4, vec![9; 12 * 4 * 4]);
    w.entity_mut(e).insert(sheets);
    assert!(sheet_is_16_bit(&w, e, 1, &db));
    assert!(!sheet_is_16_bit(&w, e, 0, &db));
}

/// ⭐ Auditoria do fecho 08/11 (A): o Upscale numa folha extra só muda a densidade (a imagem inteira fica com o
/// tamanho da tela); na folha 0, `px / ppm` como sempre. *Mutação: `px / ppm` nas duas ⇒ a folha cresce.*
#[test]
fn o_upscale_da_folha_extra_so_muda_a_densidade() {
    assert_eq!(density_size(1, [4.5, 3.0], [48, 16], 4.0), [4.5, 3.0]);
    assert_eq!(density_size(0, [4.5, 3.0], [48, 16], 4.0), [12.0, 4.0]);
}

/// Onde o centro do píxel `(px, py)` da imagem `w × h` cai no mundo: `local = anchor + quad · size` com o flip do
/// `Sprite` dentro do quad (`sim_extract_sheet.rs`), depois a base do `Transform` (`sim_extract_emit.rs`).
fn no_mundo(
    t: Transform,
    flip: [bool; 2],
    anchor: [f32; 2],
    size: [f32; 2],
    img: [u32; 2],
    p: [u32; 2],
) -> [f32; 2] {
    let mut u = (p[0] as f32 + 0.5) / img[0] as f32;
    let mut v = (p[1] as f32 + 0.5) / img[1] as f32;
    if flip[0] {
        u = 1.0 - u;
    }
    if flip[1] {
        v = 1.0 - v;
    }
    let mut filho = Transform::default();
    filho.translation.x = anchor[0] + (u - 0.5) * size[0];
    filho.translation.y = anchor[1] + (0.5 - v) * size[1];
    let w = Transform::compose(t, filho).translation;
    [w.x, w.y]
}

/// ⭐ Auditoria do fecho 08/11 (B): o deslocamento do Trim é em metros LOCAIS (`recenter_after_crop` só vê os
/// píxeis), por isso somá-lo ao Pivot da folha está certo com escala negativa e rotação; o flip do `Sprite` vira os
/// píxeis DENTRO do quad e espelha-o. Na folha 0 o `Transform` anda o mesmo, passado pela base dele. O píxel opaco
/// `(5, 1)` de uma imagem `8 × 4` fica no mesmo ponto do mundo depois do Trim às caixas `x 4..8, y 0..2`.
/// *Mutações: sem o espelho do flip ⇒ a arte salta 1 m; somar sem a base na folha 0 ⇒ salta com a escala.*
#[test]
fn o_trim_deixa_a_arte_no_mundo_com_espelho_escala_e_flip() {
    use ph2d_editor_core::image_edit::{PixelBounds, recenter_after_crop};
    let caixa = PixelBounds {
        x: 4,
        y: 0,
        width: 4,
        height: 2,
    };
    for flip in [[false, false], [true, false], [false, true]] {
        for sheet in [0u8, 1] {
            let db = AssetDb::default();
            let (mut w, e, _) = com_run(&db);
            {
                let mut t = w.get_mut::<Transform>(e).unwrap();
                (t.scale.x, t.scale.y, t.rotation) = (-2.0, 3.0, 0.4);
            }
            {
                let mut s = w.get_mut::<ph2d_render::Sprite>(e).unwrap();
                (s.flip_x, s.flip_y, s.size, s.anchor) =
                    (flip[0], flip[1], [2.0, 1.0], [0.25, 0.0]);
            }
            let mut sheets = w.get::<SpriteSheets>(e).unwrap().clone();
            (sheets.0[0].size, sheets.0[0].hframes, sheets.0[0].vframes) = ([2.0, 1.0], 1, 1);
            w.entity_mut(e).insert((sheets, ShownSheet(sheet)));
            let t0 = *w.get::<Transform>(e).unwrap();
            let antes = no_mundo(t0, flip, [0.25, 0.0], [2.0, 1.0], [8, 4], [5, 1]);
            let old_t = [t0.translation.x, t0.translation.y];
            let new_t = recenter_after_crop(old_t, [2.0, 1.0], [8, 4], caixa);
            place(&mut w, e, sheet, [old_t, new_t], None);
            let t1 = *w.get::<Transform>(e).unwrap();
            let a1 = if sheet == 0 {
                w.get::<ph2d_render::Sprite>(e).unwrap().anchor
            } else {
                w.get::<SpriteSheets>(e).unwrap().0[0].anchor
            };
            let depois = no_mundo(t1, flip, a1, [1.0, 0.5], [4, 2], [1, 1]);
            let dist = ((antes[0] - depois[0]).powi(2) + (antes[1] - depois[1]).powi(2)).sqrt();
            assert!(
                dist < 1e-4,
                "folha {sheet}, flip {flip:?}: {antes:?} → {depois:?}"
            );
        }
    }
}

/// ⭐ Auditoria do fecho 08/11 (F): a prévia do Color equalization só grava de volta na folha que ainda mostra o
/// recurso dela; apagada ou trocada, nada. A folha 0 sempre. *Mutação: `true` sempre ⇒ o Cancel escreve na folha
/// que ocupou o índice.*
#[test]
fn a_previa_so_desfaz_o_que_ela_escreveu() {
    let db = AssetDb::default();
    let (mut w, e, run) = com_run(&db);
    assert_eq!(sheet_asset(&w, e, 1), Some(run));
    assert!(preview_still_owns(&w, e, 1, Some(run)));
    assert!(preview_still_owns(&w, e, 0, None));
    let outra = db.insert_image_rgba8(1, 1, vec![1; 4]);
    assert!(set_image(&mut w, e, 1, outra, [1.0, 1.0]));
    assert!(
        !preview_still_owns(&w, e, 1, Some(run)),
        "a folha mostra outra coisa"
    );
    w.entity_mut(e).remove::<SpriteSheets>();
    assert!(
        !preview_still_owns(&w, e, 1, Some(outra)),
        "a folha foi apagada"
    );
}

/// ⭐ Auditoria do fecho 08/11 (G) e doc 27 §3 item 6: a ilha separada nasce onde foi desenhada, com o Pivot e a POSE
/// da fonte. A caixa `[0, 0, 2, 2]` de uma imagem `8 × 4` a 4 px/m em `(10, 20)` com o Pivot `(0,5, −0,25)`.
/// *Mutações: sem o Pivot ⇒ `[9,25, 20,25]`; sem a pose (escala 1, sem flip) ⇒ a ilha virada e escalada sai do lugar
/// e do tamanho; o flip sem virar o centro ⇒ cai do outro lado.*
#[test]
fn a_ilha_separada_nasce_com_o_pivot_e_a_pose_da_fonte() {
    let c = pixel_box_centre([10.0, 20.0], [2.0, 1.0], 4.0, [0, 0, 2, 2]);
    assert_eq!(c, [9.25, 20.25]);
    let mut w = World::new();
    let mut t = Transform::default();
    (t.translation.x, t.translation.y) = (10.0, 20.0);
    let e = w
        .spawn((t, ph2d_render::Sprite::atlas(0, [2.0, 1.0], [1.0; 4])))
        .id();
    let pose = |w: &World| island_pose(w, e, [2.0, 1.0], 4.0, [0.5, -0.25], [0, 0, 2, 2]).unwrap();
    let (ilha, flip) = pose(&w);
    assert_eq!(
        ([ilha.translation.x, ilha.translation.y], flip),
        ([9.75, 20.0], [false, false])
    );
    // Escala 2 e flip horizontal: o centro da caixa, `(−0,75, 0,25)` desde o centro do quad, vira `(0,75, 0,25)`; com
    // o Pivot fica `(1,25, 0)`, e a escala dobra-o.
    t.scale.x = 2.0;
    t.scale.y = 2.0;
    w.entity_mut(e).insert(t);
    w.get_mut::<ph2d_render::Sprite>(e).unwrap().flip_x = true;
    let (ilha, flip) = pose(&w);
    assert_eq!([ilha.translation.x, ilha.translation.y], [12.5, 20.0]);
    assert_eq!(
        ([ilha.scale.x, ilha.scale.y], flip),
        ([2.0, 2.0], [true, false])
    );
}

/// O Rasterize recusa-se em qualquer folha de um objeto com folhas extra: assar o `Transform` partilhado numa só
/// imagem torceria as outras. *Mutação: `true` sempre ⇒ o objeto de uma folha recusa.*
#[test]
fn um_objeto_com_folhas_extra_partilha_o_transform() {
    let db = AssetDb::default();
    let (mut w, e, _) = com_run(&db);
    assert!(has_extra_sheets(&w, e));
    w.entity_mut(e).insert(SpriteSheets::default());
    assert!(!has_extra_sheets(&w, e), "sem folhas extra");
    let so = w.spawn(Transform::default()).id();
    assert!(!has_extra_sheets(&w, so));
}

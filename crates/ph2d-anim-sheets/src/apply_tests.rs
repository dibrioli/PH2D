use super::*;
use ph2d_ecs::{AnimationTag, FrameMarker, FrameRef};

const PPM: f32 = 32.0;

/// Uma imagem toda opaca `w × h`.
fn opaca(w: u32, h: u32) -> Vec<u8> {
    vec![255; (w * h * 4) as usize]
}

/// Um Animated de uma folha `128 × 32` (4 × 1) com a animação «idle» sobre ela.
fn animado(db: &AssetDb) -> (World, Entity) {
    let mut w = World::new();
    let id = db.insert_image_rgba8(128, 32, opaca(128, 32));
    let mut lib = SpriteAnimations::new();
    lib.insert(AnimationTag::new("idle", 0, 3)).unwrap();
    let e = w
        .spawn((
            Sprite {
                source: SpriteSource::Individual { texture_id: 1 },
                ..Sprite::atlas(0, [1.0, 1.0], [1.0; 4])
            },
            SpritePixels(id),
            SpriteGrid::new(4, 1),
            lib,
            SpriteAnimator::new("idle"),
        ))
        .id();
    (w, e)
}

fn folhas(w: &World, e: Entity) -> usize {
    sheet_count(w, e)
}

/// Junta a folha `run` (`64 × 32`, opaca).
fn com_run(db: &AssetDb, w: &mut World, e: Entity) -> AssetId {
    let run = db.insert_image_rgba8(64, 32, opaca(64, 32));
    assert!(add_sheet(w, e, db, &BTreeMap::new(), PPM, run));
    run
}

/// Load sheet põe a folha no FIM e ela fica a visível do painel.
/// Mata: não fazer `push` em `SpriteSheets` · não pôr `ShownSheet` na nova · aceitar num objeto sem imagem.
#[test]
fn load_sheet_acrescenta_e_fica_visivel() {
    let db = AssetDb::default();
    let (mut w, e) = animado(&db);
    let run = com_run(&db, &mut w, e);
    assert_eq!(folhas(&w, e), 2);
    assert_eq!(w.get::<SpriteSheets>(e).unwrap().0[0].asset, run);
    assert_eq!(shown_sheet(&w, e), 1);
    // O pixel tem o mesmo tamanho na tela: 64 px de largura = 2 m a 32 px por m da folha 0.
    let s = w.get::<SpriteSheets>(e).unwrap().0[0].clone();
    assert_eq!(
        (s.hframes * s.vframes, s.size[0] * s.hframes as f32),
        (s.cells(), 2.0)
    );
    // A segunda folha entra no fim.
    let mais = db.insert_image_rgba8(32, 32, opaca(32, 32));
    assert!(add_sheet(&mut w, e, &db, &BTreeMap::new(), PPM, mais));
    assert_eq!(w.get::<SpriteSheets>(e).unwrap().0[1].asset, mais);
    assert_eq!(shown_sheet(&w, e), 2);
    // Sem imagem na folha 0 a porta é a da 1.ª imagem, não esta.
    let mut vazio = World::new();
    let v = vazio
        .spawn((
            Sprite::atlas(ph2d_render::WHITE_TILE_KEY, [1.0, 1.0], [1.0; 4]),
            SpriteAnimator::new(""),
        ))
        .id();
    assert!(!add_sheet(&mut vazio, v, &db, &BTreeMap::new(), PPM, run));
    assert!(vazio.get::<SpriteSheets>(v).is_none());
}

/// Load sheet NÃO cria animação (decisão 7): a lista, as listas de quadros e o tocador ficam como estavam.
/// Mata: o Load sheet inserir uma animação nova sobre a folha (como o 1.º Load de um Animated vazio faz).
#[test]
fn load_sheet_nao_cria_animacao() {
    let db = AssetDb::default();
    let (mut w, e) = animado(&db);
    let antes = (
        w.get::<SpriteAnimations>(e).cloned(),
        w.get::<SpriteAnimFrames>(e).cloned(),
        w.get::<SpriteAnimator>(e).cloned(),
        *w.get::<SpriteGrid>(e).unwrap(),
    );
    com_run(&db, &mut w, e);
    let depois = (
        w.get::<SpriteAnimations>(e).cloned(),
        w.get::<SpriteAnimFrames>(e).cloned(),
        w.get::<SpriteAnimator>(e).cloned(),
        *w.get::<SpriteGrid>(e).unwrap(),
    );
    assert_eq!(antes, depois);
}

/// Apagar a folha `0` promove a `1` ao `Sprite`/grelha/píxeis — desenhada como já se desenhava — e os quadros da
/// `0` ficam vazios no lugar; os da `1` passam a apontar para a `0`.
/// Mata: apagar a `0` sem promover (o `Sprite` fica o da imagem apagada) · não tirar a `ExtraSheet` promovida.
#[test]
fn apagar_a_primeira_folha_promove_a_segunda() {
    let db = AssetDb::default();
    let (mut w, e) = animado(&db);
    let run = com_run(&db, &mut w, e);
    let x = w.get::<SpriteSheets>(e).unwrap().0[0].clone();
    let mut frames = SpriteAnimFrames::default();
    frames.set(
        "mix",
        vec![
            FrameRef::Cell { sheet: 0, cell: 1 },
            FrameRef::Cell { sheet: 1, cell: 0 },
        ],
    );
    let mut lib = w.get::<SpriteAnimations>(e).cloned().unwrap();
    lib.insert(AnimationTag::new("mix", 0, 0)).unwrap();
    let textura = SpriteSource::Individual { texture_id: 9 };
    w.entity_mut(e).insert((
        frames,
        lib,
        SheetTextures(vec![Some((run, textura))]),
        ShownSheet(0),
    ));
    assert!(delete_shown_sheet(&mut w, e, &BTreeMap::new()));
    let s = *w.get::<Sprite>(e).unwrap();
    assert_eq!((s.source, s.size, s.anchor), (textura, x.size, x.anchor));
    assert_eq!(w.get::<SpritePixels>(e).unwrap().0, run);
    let g = *w.get::<SpriteGrid>(e).unwrap();
    assert_eq!((g.hframes, g.vframes), (x.hframes, x.vframes));
    assert!(
        w.get::<SpriteSheets>(e).is_none(),
        "a folha promovida sai das extra"
    );
    let f = w.get::<SpriteAnimFrames>(e).unwrap();
    assert_eq!(
        f.get("mix").unwrap(),
        &[FrameRef::Empty, FrameRef::Cell { sheet: 0, cell: 0 }]
    );
    // O intervalo «idle» era da folha 0: vira quatro vazios.
    assert_eq!(f.get("idle").unwrap(), &[FrameRef::Empty; 4]);
    // O quadro na tela era da folha apagada.
    assert_eq!(g.shown(), FrameRef::Empty);
    // Com uma folha só já não se apaga.
    assert!(!delete_shown_sheet(&mut w, e, &BTreeMap::new()));
}

/// Apagar uma folha extra visível: os quadros dela ficam vazios, a `ExtraSheet` sai, a visível passa à anterior.
/// Mata: apagar `k` sem tirar a `ExtraSheet` · não esvaziar os quadros dela.
#[test]
fn apagar_uma_folha_extra_deixa_os_quadros_vazios() {
    let db = AssetDb::default();
    let (mut w, e) = animado(&db);
    com_run(&db, &mut w, e);
    let mut frames = SpriteAnimFrames::default();
    frames.set(
        "idle",
        vec![
            FrameRef::Cell { sheet: 1, cell: 0 },
            FrameRef::Cell { sheet: 0, cell: 2 },
        ],
    );
    w.entity_mut(e).insert(frames);
    assert!(delete_shown_sheet(&mut w, e, &BTreeMap::new()));
    assert!(w.get::<SpriteSheets>(e).is_none());
    assert_eq!(shown_sheet(&w, e), 0);
    assert_eq!(
        w.get::<SpriteAnimFrames>(e).unwrap().get("idle").unwrap(),
        &[FrameRef::Empty, FrameRef::Cell { sheet: 0, cell: 2 }]
    );
}

/// Remove empty frames tira os buracos de quem tem quadros de verdade.
/// Mata: o Remove empty frames não escrever nada no mundo.
#[test]
fn remove_empty_frames_tira_os_buracos_no_mundo() {
    let db = AssetDb::default();
    let (mut w, e) = animado(&db);
    let mut frames = SpriteAnimFrames::default();
    frames.set(
        "idle",
        vec![FrameRef::Empty, FrameRef::Cell { sheet: 0, cell: 2 }],
    );
    w.entity_mut(e).insert(frames);
    assert!(remove_empty_frames(&mut w, e));
    assert_eq!(
        w.get::<SpriteAnimFrames>(e).unwrap().get("idle").unwrap(),
        &[FrameRef::Cell { sheet: 0, cell: 2 }]
    );
    assert!(!remove_empty_frames(&mut w, e), "nada mais a tirar");
}

/// Delete empty animations leva as TODAS vazias com os marcadores, a lista e o tocador que estava nelas.
/// Mata: não tirar os marcadores · não tirar a lista · deixar o tocador numa animação apagada.
#[test]
fn delete_empty_animations_leva_os_marcadores() {
    let db = AssetDb::default();
    let (mut w, e) = animado(&db);
    let mut lib = w.get::<SpriteAnimations>(e).cloned().unwrap();
    lib.insert(AnimationTag::new("oca", 0, 0)).unwrap();
    let mut frames = SpriteAnimFrames::default();
    frames.set("oca", vec![FrameRef::Empty; 2]);
    let marca = |tag: &str| FrameMarker {
        tag: tag.into(),
        step: 0,
        signal: "pe".into(),
    };
    let mut p = SpriteAnimator::new("oca");
    p.start = Some("oca".into());
    w.entity_mut(e).insert((
        lib,
        frames,
        SpriteAnimCursor::default(),
        SpriteAnimMarkers(vec![marca("oca"), marca("idle")]),
        p,
    ));
    assert!(delete_empty_animations(&mut w, e));
    let lib = w.get::<SpriteAnimations>(e).unwrap();
    assert!(lib.get("oca").is_none() && lib.get("idle").is_some());
    assert_eq!(
        w.get::<SpriteAnimMarkers>(e).unwrap().0,
        vec![marca("idle")]
    );
    assert!(w.get::<SpriteAnimFrames>(e).is_none() && w.get::<SpriteAnimCursor>(e).is_none());
    let p = w.get::<SpriteAnimator>(e).unwrap();
    assert_eq!(
        (p.current.as_str(), p.start.as_deref()),
        ("idle", Some("idle"))
    );
    assert!(!delete_empty_animations(&mut w, e));
}

/// O dropdown só mostra folhas que existem; não toca no documento.
#[test]
fn show_sheet_so_aceita_folhas_que_existem() {
    let db = AssetDb::default();
    let (mut w, e) = animado(&db);
    com_run(&db, &mut w, e);
    show_sheet(&mut w, e, 0);
    assert_eq!(shown_sheet(&w, e), 0);
    show_sheet(&mut w, e, 5);
    assert_eq!(w.get::<ShownSheet>(e), Some(&ShownSheet(0)));
}

/// ⭐ Achado da auditoria 08/11: sem janela do editor o Load sheet não fazia nada e calava-se; cada recusa tem agora
/// uma frase. *Mutações: devolver `None` sem painel ⇒ vermelho; tirar a verificação da imagem ⇒ o vazio passa.*
#[test]
fn o_load_sheet_diz_porque_nao_abre() {
    let db = AssetDb::default();
    let (mut w, e) = animado(&db);
    assert_eq!(load_sheet_refusal(&w, e, true), None);
    assert_eq!(
        load_sheet_refusal(&w, e, false),
        Some("card.insp.sheet_load_no_assets")
    );
    let branco = Sprite::atlas(ph2d_render::WHITE_TILE_KEY, [1.0, 1.0], [1.0; 4]);
    let vazio = w.spawn(branco).id();
    assert_eq!(
        load_sheet_refusal(&w, vazio, true),
        Some("card.insp.sheet_load_no_image")
    );
    for _ in 1..SHEETS_MAX {
        let k = db.insert_image_rgba8(4, 4, opaca(4, 4));
        let mut s = w.get::<SpriteSheets>(e).cloned().unwrap_or_default();
        s.0.push(ph2d_ecs::ExtraSheet {
            asset: k,
            size: [1.0, 1.0],
            anchor: [0.0, 0.0],
            hframes: 1,
            vframes: 1,
            offsets: Default::default(),
        });
        w.entity_mut(e).insert(s);
    }
    assert_eq!(
        load_sheet_refusal(&w, e, true),
        Some("card.insp.sheet_load_full")
    );
}

/// ⭐ Auditoria do fecho 08/11 (D): o recurso que uma sprite mostra PRÉ-MULTIPLICADO (o BG removal) entra como folha
/// numa cópia recta, porque a folha extra desenha-se sempre recta; o mesmo recurso, sem ninguém pré-multiplicado,
/// entra pelo id de sempre. *Mutação: usar sempre o `id` escolhido ⇒ a folha fica com os bytes escuros.*
#[test]
fn load_sheet_de_um_recurso_premultiplicado_leva_a_copia_recta() {
    let db = AssetDb::default();
    let (mut w, e) = animado(&db);
    let pre = db.insert_image_rgba8(2, 1, [100, 50, 26, 128].repeat(2));
    w.spawn((
        Sprite {
            premultiplied: true,
            ..Sprite::atlas(0, [1.0, 1.0], [1.0; 4])
        },
        SpritePixels(pre),
    ));
    assert!(add_sheet(&mut w, e, &db, &BTreeMap::new(), PPM, pre));
    let s = w.get::<SpriteSheets>(e).unwrap().0[0].clone();
    assert_ne!(
        s.asset, pre,
        "a folha mostra os bytes pré-multiplicados como rectos"
    );
    let a = db.get(&s.asset).unwrap();
    let (_, _, px) = a.image_rgba8().unwrap();
    assert_eq!(&px[..4], &[199, 100, 52, 128]);
    let reta = db.insert_image_rgba8(64, 32, opaca(64, 32));
    assert!(add_sheet(&mut w, e, &db, &BTreeMap::new(), PPM, reta));
    assert_eq!(w.get::<SpriteSheets>(e).unwrap().0[1].asset, reta);
}

/// ⭐ **Escolher uma imagem com a folha extra à vista troca ESSA folha** (dono 09/11): a folha `0`, as animações e a
/// visível ficam; o pixel guarda o tamanho na tela; o corte fica quando a imagem nova se divide nele. Com a `0` à
/// vista a porta é a de sempre (`None`).
///
/// *Mutações: trocar a folha `0` ⇒ `SpritePixels` muda e reprova; recalcular o tamanho pelo `ppm` em vez do pixel da
/// folha velha ⇒ a largura na tela sai errada; perder o corte ⇒ `hframes` muda.*
#[test]
fn a_imagem_escolhida_troca_a_folha_a_vista() {
    let db = AssetDb::default();
    let (mut w, e) = animado(&db);
    let run = com_run(&db, &mut w, e);
    let velha = w.get::<SpriteSheets>(e).unwrap().0[0].clone();
    let pixels_da_0 = *w.get::<SpritePixels>(e).unwrap();
    let nova = db.insert_image_rgba8(128, 64, opaca(128, 64));
    assert_eq!(
        replace_shown_sheet(&mut w, e, &db, &BTreeMap::new(), nova),
        Some(ShownSwap::Kept([velha.hframes, velha.vframes]))
    );
    let s = w.get::<SpriteSheets>(e).unwrap().0[0].clone();
    assert_eq!(s.asset, nova);
    assert_eq!(
        (s.hframes, s.vframes),
        (velha.hframes, velha.vframes),
        "o corte fica"
    );
    // O mesmo metro por pixel: o dobro dos píxeis por célula, o dobro do tamanho na tela.
    assert_eq!(s.size, [velha.size[0] * 2.0, velha.size[1] * 2.0]);
    assert_eq!(
        *w.get::<SpritePixels>(e).unwrap(),
        pixels_da_0,
        "a folha 0 fica"
    );
    assert_eq!(shown_sheet(&w, e), 1);
    assert_eq!(
        replace_shown_sheet(&mut w, e, &db, &BTreeMap::new(), nova),
        Some(ShownSwap::Same),
        "a mesma"
    );
    // Uma imagem que não se divide no corte leva o corte que ela propõe — e o aviso di-lo.
    w.get_mut::<SpriteSheets>(e).unwrap().0[0].hframes = 2;
    let torta = db.insert_image_rgba8(33, 32, opaca(33, 32));
    assert_eq!(
        replace_shown_sheet(&mut w, e, &db, &BTreeMap::new(), torta),
        Some(ShownSwap::Recut {
            old: [2, velha.vframes],
            new: [1, 1]
        }),
        "o aviso diz que o corte mudou"
    );
    let s = w.get::<SpriteSheets>(e).unwrap().0[0].clone();
    assert_eq!((s.hframes, s.vframes), (1, 1));
    assert_ne!(s.asset, run);
    w.entity_mut(e).insert(ShownSheet(0));
    assert_eq!(
        replace_shown_sheet(&mut w, e, &db, &BTreeMap::new(), nova),
        None
    );
}

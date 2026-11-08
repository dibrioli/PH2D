use super::*;
use ph2d_ecs::{ExtraSheet, SpriteAnimations, TagFrames};

fn folha(asset: AssetId, h: u32, v: u32) -> ExtraSheet {
    ExtraSheet {
        asset,
        size: [1.0, 1.0],
        anchor: [0.0, 0.0],
        hframes: h,
        vframes: v,
    }
}

fn sprite() -> Sprite {
    Sprite {
        source: SpriteSource::Individual { texture_id: 1 },
        ..Sprite::atlas(0, [1.0, 1.0], [1.0; 4])
    }
}

fn lista(cells: Vec<FrameRef>) -> SpriteAnimFrames {
    SpriteAnimFrames(
        [TagFrames {
            tag: "mix".into(),
            cells,
        }]
        .into_iter()
        .collect(),
    )
}

fn idle() -> AssetId {
    AssetId::from_digest([1; 32])
}

fn run() -> AssetId {
    AssetId::from_digest([2; 32])
}

/// ⭐ Cortar a folha extra `run` de um objeto passa o corte à folha `0` de outro que mostra `run` e à folha extra de um
/// terceiro; quem mostra outra imagem fica. *Mutações: comparar só a folha `0` ⇒ o terceiro fica 6 × 2; esquecer o
/// `set_cut` das extra ⇒ idem; não excluir o próprio ⇒ inofensivo, mas o devolvido ganha `e`.*
#[test]
fn o_cut_again_passa_as_folhas_extra_que_mostram_a_mesma_imagem() {
    let mut w = World::new();
    let a = w
        .spawn((
            sprite(),
            SpritePixels(idle()),
            SpriteGrid::new(4, 1),
            SpriteSheets([folha(run(), 3, 1)].into_iter().collect()),
        ))
        .id();
    let b = w
        .spawn((sprite(), SpritePixels(run()), SpriteGrid::new(6, 2)))
        .id();
    let c = w
        .spawn((
            sprite(),
            SpritePixels(idle()),
            SpriteGrid::new(4, 1),
            SpriteSheets([folha(run(), 6, 2)].into_iter().collect()),
        ))
        .id();
    let outro = w
        .spawn((sprite(), SpritePixels(idle()), SpriteGrid::new(2, 2)))
        .id();
    let devolvidos = recut_users(&mut w, a, 1);
    assert_eq!(devolvidos, vec![b], "só a folha 0 de b mudou de corte");
    let g = w.get::<SpriteGrid>(b).copied().unwrap();
    assert_eq!((g.hframes, g.vframes), (3, 1));
    let s = w.get::<SpriteSheets>(c).unwrap().0[0];
    assert_eq!(
        (s.hframes, s.vframes),
        (3, 1),
        "a folha extra de c segue o corte"
    );
    let g = w.get::<SpriteGrid>(c).copied().unwrap();
    assert_eq!(
        (g.hframes, g.vframes),
        (4, 1),
        "a folha 0 de c é outra imagem"
    );
    let g = w.get::<SpriteGrid>(outro).copied().unwrap();
    assert_eq!(
        (g.hframes, g.vframes),
        (2, 2),
        "quem mostra outra imagem fica"
    );
}

/// ⭐ Um corte menor deixa BURACOS no lugar dos quadros que saíram da grelha, e a animação continua a tocar o resto
/// (antes, um quadro fora fazia `tag_cells` recusar a lista inteira). *Mutações: `holes_beyond` sem trocar ⇒ a lista
/// fica com a célula 5 e `tag_cells` devolve `None`; trocar todos ⇒ o quadro 1 some.*
#[test]
fn cols_rows_menores_deixam_buracos_e_a_animacao_toca_o_resto() {
    let mut w = World::new();
    let mut lib = SpriteAnimations::new();
    lib.insert(ph2d_ecs::AnimationTag::new("mix", 0, 0))
        .unwrap();
    let e = w
        .spawn((
            sprite(),
            SpritePixels(idle()),
            SpriteGrid::new(4, 1),
            SpriteSheets([folha(run(), 3, 1)].into_iter().collect()),
            lib,
            lista(vec![
                FrameRef::Cell { sheet: 0, cell: 1 },
                FrameRef::Cell { sheet: 1, cell: 2 },
                FrameRef::Cell { sheet: 0, cell: 3 },
            ]),
        ))
        .id();
    // Antes do corte: tudo cabe, nada muda.
    assert!(!hold_frames(&mut w, e));
    // A folha 0 passa a 2 × 1: a célula 3 sai.
    let mut g = *w.get::<SpriteGrid>(e).unwrap();
    g.hframes = 2;
    w.entity_mut(e).insert(g);
    assert!(hold_frames(&mut w, e));
    let f = w.get::<SpriteAnimFrames>(e).unwrap();
    assert_eq!(
        f.get("mix").unwrap(),
        [
            FrameRef::Cell { sheet: 0, cell: 1 },
            FrameRef::Cell { sheet: 1, cell: 2 },
            FrameRef::Empty,
        ]
    );
    let tag = w.get::<SpriteAnimations>(e).unwrap().0[0].clone();
    let cells = sheet_cells_of(&w, e);
    assert!(
        ph2d_ecs::tag_cells(&tag, f.get("mix"), &cells).is_some(),
        "a animação continua a tocar"
    );
}

/// A célula na tela volta para dentro da folha dela quando a grelha encolhe. *Mutação: sem o recuo ⇒ fica na 5.*
#[test]
fn a_celula_na_tela_volta_para_dentro_da_sua_folha() {
    let mut w = World::new();
    let mut g = SpriteGrid::new(4, 1);
    g.sheet = 1;
    g.frame = 5;
    let e = w
        .spawn((
            sprite(),
            SpritePixels(idle()),
            g,
            SpriteSheets([folha(run(), 3, 1)].into_iter().collect()),
        ))
        .id();
    assert!(hold_frames(&mut w, e));
    let g = w.get::<SpriteGrid>(e).copied().unwrap();
    assert_eq!((g.sheet, g.frame), (1, 2));
}

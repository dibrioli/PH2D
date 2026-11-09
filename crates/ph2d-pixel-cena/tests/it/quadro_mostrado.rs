//! `quadro_mostrado`: o editor mostra o seu quadro; em jogo, o do animador.

use ph2d_ecs::{PixelSprite, SpriteAnimator, SpriteGrid, World};
use ph2d_pixel_cena::fixture::heroi_p14;
use ph2d_pixel_cena::{clips_das_tags, quadro_mostrado};

#[test]
fn o_editor_mostra_o_seu_quadro_e_o_jogo_o_do_animador() {
    let s = heroi_p14();
    let ids: Vec<_> = s.frames().iter().map(|f| f.id).collect();
    let editor = ids[0];
    let mut grid = SpriteGrid::new(6, 1);
    grid.frame = 3;
    let mut w = World::new();
    let mut anim = SpriteAnimator::new("run");
    let e = w
        .spawn((
            PixelSprite(1),
            clips_das_tags(&s, "heroi"),
            grid,
            anim.clone(),
        ))
        .id();
    assert_eq!(
        quadro_mostrado(&w, e, &s, editor),
        editor,
        "parado: o do editor"
    );

    anim.playing = true;
    w.entity_mut(e).insert(anim.clone());
    assert_eq!(
        quadro_mostrado(&w, e, &s, editor),
        ids[3],
        "a tocar: o do animador"
    );

    // Uma tag que acabou fica no último quadro dela, não salta para o do editor.
    anim.playing = false;
    anim.loop_override = Some(false);
    anim.repeat_count = 1;
    w.entity_mut(e).insert(anim);
    assert_eq!(
        quadro_mostrado(&w, e, &s, editor),
        ids[3],
        "acabou: o último do animador"
    );

    let sem = w.spawn(PixelSprite(2)).id();
    assert_eq!(
        quadro_mostrado(&w, sem, &s, ids[2]),
        ids[2],
        "sem animador: o do editor"
    );
}

/// A costura do desenho (`ph2d-app-pixel` `draw.rs` → `tiles::compoe`): em Draw o do editor, fora o
/// do animador — e a família pinta ESSE quadro, não o `doc.frame`.
#[test]
fn o_play_mostra_o_quadro_do_animador() {
    use ph2d_pixel_cena::quadro_a_desenhar;
    let s = heroi_p14();
    let ids: Vec<_> = s.frames().iter().map(|f| f.id).collect();
    let mut grid = SpriteGrid::new(6, 1);
    grid.frame = 4;
    let mut anim = SpriteAnimator::new("run");
    anim.playing = true;
    let mut w = World::new();
    let e = w
        .spawn((PixelSprite(1), clips_das_tags(&s, "heroi"), grid, anim))
        .id();
    assert_eq!(quadro_a_desenhar(&w, e, &s, ids[0], false), ids[4], "Play");
    assert_eq!(quadro_a_desenhar(&w, e, &s, ids[0], true), ids[0], "Draw");

    let fonte = |f: &str| {
        let p = concat!(env!("CARGO_MANIFEST_DIR"), "/../ph2d-app-pixel/src/");
        std::fs::read_to_string(format!("{p}{f}")).expect("fonte da família")
    };
    let draw = fonte("draw.rs");
    let tiles = fonte("tiles.rs");
    let compoe = &tiles[tiles.find("fn compoe(").expect("compoe")..];
    let compoe = &compoe[..compoe.find("\n}\n").expect("fim")];
    assert!(
        draw.contains("quadro_a_desenhar(") && draw.contains("compoe(app, key, quadro,"),
        "o desenho não pinta o quadro mostrado"
    );
    assert!(
        compoe.contains("refresh_with(&d.sprite, quadro,") && !compoe.contains("d.frame"),
        "a composição ignora o quadro que o desenho lhe deu"
    );
}

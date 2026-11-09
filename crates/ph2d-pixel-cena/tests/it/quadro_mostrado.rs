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

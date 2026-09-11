//! **A BORDA INTEIRA REDIMENSIONA A COLUNA** — Enio, 2026-08-30: *«os painéis devem ser
//! redimensionáveis para esquerda e para direita e com setas bidirecionais no cursor; os
//! pontinhos de redimensionamento podem ser retirados. A borda inteira serve para
//! redimensionar»*.
//!
//! # As três leis que este ficheiro defende
//!
//! 1. **A costura vive DENTRO da coluna** — nos últimos `DOCK_SEAM_PX` px antes da área de
//!    desenho. ⚠️ Não a cavalo da fronteira, como uma divisória costuma ser: desde que a régua
//!    ficou **colada** à coluna, a faixa dela começa exactamente onde a coluna acaba, e uma
//!    costura centrada roubaria metade do agarre à régua.
//! 2. **A conta é do LADO** — à esquerda a coluna cresce com o `x`, à direita decresce. É a
//!    inversão que se escreve ao contrário sem o compilador reclamar, e o único gate que a apanha
//!    é este.
//! 3. **Uma coluna vazia não oferece costura** — sem painel não há borda, e um agarre sobre o nada
//!    seria chrome vivo e invisível (a espécie que este repo varre a cada wave).

use ph2d_editor_core::screens::layout::{
    ChromeBands, DOCK_SEAM_PX, DockSide, DockSides, HERO_VIEWPORT_H, HERO_VIEWPORT_W, HeroLayout,
};
use ph2d_editor_core::zones::Rect;

fn layout(mirrored: bool, docks: DockSides) -> HeroLayout {
    HeroLayout::for_viewport_bands(
        Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H),
        mirrored,
        ChromeBands::DEFAULT,
        ph2d_editor_core::screens::layout::CenterSplit::None,
        docks,
    )
}

/// **A costura é a borda INTERIOR, e está dentro da coluna.**
#[test]
fn the_seam_is_the_inner_border_and_lives_inside_the_column() {
    for mirrored in [false, true] {
        let l = layout(mirrored, DockSides::BOTH);
        let (left_col, right_col) = l.side_columns();
        for (side, col) in [(DockSide::Left, left_col), (DockSide::Right, right_col)] {
            let seam = l.dock_seam(side);
            assert!(
                (seam.w - DOCK_SEAM_PX).abs() < f32::EPSILON,
                "mirrored={mirrored} {side:?}: a costura tem de ter {DOCK_SEAM_PX} px ({seam:?})"
            );
            assert!(
                seam.x >= col.x && seam.x + seam.w <= col.x + col.w + f32::EPSILON,
                "mirrored={mirrored} {side:?}: a costura {seam:?} saiu da coluna {col:?} — ela \
                 roubaria o agarre a' regua, que comeca onde a coluna acaba"
            );
            // A borda INTERIOR: encostada ao lado que da' para a area de desenho.
            let touches_inner = match side {
                DockSide::Left => (seam.x + seam.w - (col.x + col.w)).abs() < f32::EPSILON,
                DockSide::Right => (seam.x - col.x).abs() < f32::EPSILON,
            };
            assert!(
                touches_inner,
                "mirrored={mirrored} {side:?}: a costura {seam:?} nao esta' na borda INTERIOR da \
                 coluna {col:?}"
            );
        }
    }
}

/// **A costura e o cursor perguntam a MESMA coisa** — `dock_seam_at` é a porta.
#[test]
fn a_point_on_the_seam_answers_that_side_and_nothing_else_does() {
    let l = layout(false, DockSides::BOTH);
    for side in [DockSide::Left, DockSide::Right] {
        let s = l.dock_seam(side);
        let inside = (s.x + s.w * 0.5, s.y + s.h * 0.5);
        assert_eq!(
            l.dock_seam_at(inside),
            Some(side),
            "{side:?}: o meio da costura tem de responder {side:?}"
        );
    }
    // O meio da area de desenho nao e' costura nenhuma.
    let a = l.draw_area;
    assert_eq!(l.dock_seam_at((a.x + a.w * 0.5, a.y + a.h * 0.5)), None);
    // Nem o meio de uma coluna.
    let (left_col, _) = l.side_columns();
    assert_eq!(
        l.dock_seam_at((left_col.x + left_col.w * 0.5, left_col.y + 50.0)),
        None,
        "o corpo do painel nao e' costura — senao todo clique nele redimensionava"
    );
}

/// ⭐ **A CONTA É DO LADO** — e é a metade que se escreve ao contrário.
#[test]
fn the_width_grows_with_x_on_the_left_and_shrinks_on_the_right() {
    let l = layout(false, DockSides::BOTH);
    let (left_col, right_col) = l.side_columns();

    // Puxar a borda da esquerda 40 px para a DIREITA aumenta a coluna em 40.
    let seam_l = l.dock_seam(DockSide::Left);
    let w_l = l.dock_width_for(DockSide::Left, seam_l.x + seam_l.w + 40.0);
    assert!(
        (w_l - (left_col.w + 40.0)).abs() < 0.5,
        "a coluna da esquerda tem de CRESCER 40 ({w_l} contra {})",
        left_col.w + 40.0
    );

    // Puxar a borda da direita 40 px para a ESQUERDA tambem a aumenta.
    let seam_r = l.dock_seam(DockSide::Right);
    let w_r = l.dock_width_for(DockSide::Right, seam_r.x - 40.0);
    assert!(
        (w_r - (right_col.w + 40.0)).abs() < 0.5,
        "a coluna da direita tem de CRESCER 40 ao ser puxada para a esquerda ({w_r} contra {})",
        right_col.w + 40.0
    );

    // ⚠️ O controlo que apanha a inversao: as duas contas NAO podem dar o mesmo sinal para o
    // mesmo deslocamento de `x`.
    let d_l = l.dock_width_for(DockSide::Left, seam_l.x + 10.0)
        - l.dock_width_for(DockSide::Left, seam_l.x - 10.0);
    let d_r = l.dock_width_for(DockSide::Right, seam_r.x + 10.0)
        - l.dock_width_for(DockSide::Right, seam_r.x - 10.0);
    assert!(
        d_l > 0.0 && d_r < 0.0,
        "os dois lados respondem ao `x` com o MESMO sinal ({d_l} e {d_r}) — a inversao caiu"
    );
}

/// **Uma coluna vazia não oferece costura.**
#[test]
fn a_column_nobody_occupies_offers_no_seam() {
    let l = layout(false, DockSides::NONE);
    for side in [DockSide::Left, DockSide::Right] {
        let s = l.dock_seam(side);
        assert!(
            s.w <= 0.0,
            "{side:?}: coluna vazia devolveu costura {s:?} — agarre sobre o nada"
        );
    }
    assert_eq!(l.dock_seam_at((5.0, 500.0)), None);
}

/// **A largura AUTORADA chega ao rect da coluna** — sem isto o arrasto escreve num campo que
/// ninguém lê, que é a forma exacta do controlo morto.
#[test]
fn an_authored_width_reaches_the_column_rect() {
    let wide = HeroLayout::for_viewport_bands(
        Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H),
        false,
        ChromeBands {
            left_dock_w: 420.0,
            ..ChromeBands::DEFAULT
        },
        ph2d_editor_core::screens::layout::CenterSplit::None,
        DockSides::BOTH,
    );
    let (left_col, _) = wide.side_columns();
    assert!(
        (left_col.w - 420.0).abs() < f32::EPSILON,
        "a largura autorada nao chegou a' coluna ({})",
        left_col.w
    );
    // E a area de desenho encolheu com ela — senao a coluna cresceria por cima do desenho.
    let base = layout(false, DockSides::BOTH);
    assert!(
        wide.draw_area.x > base.draw_area.x,
        "a area de desenho nao acompanhou a coluna mais larga"
    );
}

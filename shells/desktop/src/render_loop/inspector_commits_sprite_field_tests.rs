//! Os testes da lei de campo da SPRITE do Inspector — o módulo `inspector_commits::sprite_field_tests`,
//! num ficheiro irmão por `#[path]` (o padrão da casa) para o `inspector_commits.rs` caber no tecto de
//! 600 LOC quando os drenos das secções e dos nomes de sinal ganharam função própria.

use crate::render_loop::inspector_commits_sprite::{
    SpriteEditTarget, SpriteEditables, apply_sprite_field, clamp_frame,
};
use ph2d_ecs::{SpriteCornerTint, SpriteGrid};
use ph2d_editor_core::SpriteFieldEdit;
use ph2d_render::Sprite;

/// Os quatro editáveis no estado neutro — o que uma sprite sem nenhum dos três componentes
/// apresenta ao commit (ADR-0164 F1 passo 6).
fn editables() -> SpriteEditables {
    SpriteEditables {
        sprite: Sprite::atlas(0, [1.0, 1.0], [1.0, 1.0, 1.0, 1.0]),
        grid: SpriteGrid::SINGLE,
        region: None,
        corner_tint: SpriteCornerTint::IDENTITY,
    }
}

#[test]
fn flip_edits_set_the_flags() {
    let mut t = editables();
    assert_eq!(
        apply_sprite_field(&mut t, SpriteFieldEdit::FlipX(true)),
        SpriteEditTarget::Sprite
    );
    apply_sprite_field(&mut t, SpriteFieldEdit::FlipY(true));
    assert!(t.sprite.flip_x && t.sprite.flip_y);
    apply_sprite_field(&mut t, SpriteFieldEdit::FlipX(false));
    assert!(!t.sprite.flip_x && t.sprite.flip_y);
}

#[test]
fn opacity_is_clamped_to_unit() {
    let mut t = editables();
    apply_sprite_field(&mut t, SpriteFieldEdit::Opacity(2.5));
    assert_eq!(t.sprite.opacity, 1.0);
    apply_sprite_field(&mut t, SpriteFieldEdit::Opacity(-0.3));
    assert_eq!(t.sprite.opacity, 0.0);
}

#[test]
fn frame_count_floors_at_one_and_reclamps_frame() {
    let mut t = editables();
    assert_eq!(
        apply_sprite_field(&mut t, SpriteFieldEdit::Hframes(4)),
        SpriteEditTarget::Grid,
        "a grelha e' o alvo, nao a Sprite"
    );
    apply_sprite_field(&mut t, SpriteFieldEdit::Vframes(2));
    apply_sprite_field(&mut t, SpriteFieldEdit::Frame(7)); // last cell of 4*2
    assert_eq!(t.grid.frame, 7);
    // Shrinking the grid must drag the stale frame back in-range.
    apply_sprite_field(&mut t, SpriteFieldEdit::Vframes(1)); // now 4 cells
    assert_eq!(t.grid.frame, 3);
    // 0 is floored to 1 (never a zero-cell sheet).
    apply_sprite_field(&mut t, SpriteFieldEdit::Hframes(0));
    assert_eq!(t.grid.hframes, 1);
    assert_eq!(t.grid.frame, 0); // 1*1 = 1 cell → frame 0
}

#[test]
fn frame_set_past_grid_is_clamped_immediately() {
    let mut t = editables();
    // default hframes=vframes=1 → only cell is 0.
    apply_sprite_field(&mut t, SpriteFieldEdit::Frame(99));
    assert_eq!(t.grid.frame, 0);
}

#[test]
fn region_rect_clamps_extent_non_negative_but_keeps_origin() {
    let mut t = editables();
    apply_sprite_field(
        &mut t,
        SpriteFieldEdit::RegionRect([-4.0, -2.0, -10.0, 8.0]),
    );
    // x/y pass through (extract clamps into the source); w/h floor at 0.
    assert_eq!(t.region.expect("materializou").rect, [-4.0, -2.0, 0.0, 8.0]);
}

/// ⭐ **Ligar/desligar a janela é anexar/retirar o componente** (ADR-0164 F1 passo 6) — o
/// antigo `region_enabled`, dito da única maneira que ele hoje se diz.
#[test]
fn the_region_toggle_is_the_components_presence() {
    let mut t = editables();
    assert_eq!(
        apply_sprite_field(&mut t, SpriteFieldEdit::RegionEnabled(true)),
        SpriteEditTarget::Region
    );
    assert!(t.region.is_some());
    assert_eq!(
        apply_sprite_field(&mut t, SpriteFieldEdit::RegionEnabled(false)),
        SpriteEditTarget::RegionRemoved,
        "desligar tem de pedir a REMOCAO — um SetComponent nao sabe exprimir ausencia"
    );
    assert!(t.region.is_none());
}

/// ⚠️ **Uma edição de campo da região MATERIALIZA o componente**, em vez de ser um no-op: o
/// painel ainda mostra as linhas a toda sprite, e um campo que aceita o gesto e não faz nada
/// é o defeito que a DIRETIVA §2 proíbe.
#[test]
fn a_region_field_edit_materialises_the_component() {
    let mut t = editables();
    assert!(t.region.is_none());
    apply_sprite_field(&mut t, SpriteFieldEdit::RegionW(12.0));
    assert_eq!(t.region.expect("materializou").rect[2], 12.0);
    // E o `filter_clip` sai da FONTE dos pixels — esta e' uma sprite de Atlas.
    assert!(t.region.expect("regiao").filter_clip);
}

#[test]
fn per_axis_edits_preserve_the_other_components() {
    // BulkSelect D-1: editing one axis must NOT touch the siblings
    // (so a bulk edit of one axis can't stomp a diverging sibling).
    let mut t = editables();
    t.sprite.offset = [3.0, 5.0];
    apply_sprite_field(&mut t, SpriteFieldEdit::OffsetX(9.0));
    assert_eq!(t.sprite.offset, [9.0, 5.0], "OffsetX left Y untouched");

    t.region = Some(ph2d_ecs::SpriteRegion::for_atlas([1.0, 2.0, 3.0, 4.0]));
    apply_sprite_field(&mut t, SpriteFieldEdit::RegionY(8.0));
    assert_eq!(
        t.region.expect("regiao").rect,
        [1.0, 8.0, 3.0, 4.0],
        "RegionY left X/W/H"
    );
    // W/H still floor at 0 per-axis.
    apply_sprite_field(&mut t, SpriteFieldEdit::RegionW(-7.0));
    assert_eq!(
        t.region.expect("regiao").rect,
        [1.0, 8.0, 0.0, 4.0],
        "RegionW floored, rest kept"
    );
}

#[test]
fn clamp_frame_is_idempotent_in_range() {
    let mut g = SpriteGrid {
        hframes: 3,
        vframes: 3,
        frame: 4,
    };
    clamp_frame(&mut g);
    assert_eq!(g.frame, 4);
}

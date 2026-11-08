//! **O cartão «Collision Shape»** (ficha f1): um `Collider` sem corpo próprio. Só tem as linhas que a ponte
//! lê de uma peça (`physics_part.rs` diz o porquê); nada de zona. Irmão de [`super::physics_cartao`].

use super::physics_cartao::Tinta;
use super::physics_cartao_forma::{
    dobra_material, dobra_offset, dobra_sinais, dobra_superficie, medidas_e_estado, toque,
};
use super::physics_rows::paint_bounce_friction;
use super::rows::num_row;
use super::*;
use ph2d_editor_core::interaction::format_number;
use ph2d_editor_core::screens::hero::InspectorPhysicsInfo;
use ph2d_i18n::{TextKey, tr, tr_with};

pub(super) fn peca(t: &mut Tinta<'_>, y: f32, info: &InspectorPhysicsInfo) -> f32 {
    let mut y = de_quem_e(t, y, info);
    y = medidas_e_estado(t, y, info);
    y = toque(t, y, info);
    y = paint_bounce_friction(t.scene, t.text, t.theme, t.hit, t.store, t.x, t.w, y);
    // A densidade de uma peça é real: conta para a massa do corpo composto.
    let densidade = format_number(t.store.number_value(ids::INSP_PHYS_DENSITY).unwrap_or(0.0));
    let titulo = tr("card.insp.phys.density");
    y = t.dobra(
        y,
        ids::INSP_PHYS_FOLD_DENSITY,
        titulo,
        &densidade,
        |t, y| {
            num_row(
                t.scene,
                t.text,
                t.theme,
                t.hit,
                t.store,
                t.x,
                t.w,
                y,
                TextKey::new("component.field.collider.1"),
                ids::INSP_PHYS_DENSITY,
            )
        },
    );
    y = dobra_offset(t, y);
    y = dobra_material(t, y, info);
    y = dobra_superficie(t, y, info, !info.is_sensor);
    y = dobra_sinais(t, y, info);
    let (r, y) = t.botao(
        ids::INSP_PHYS_REMOVE,
        tr("panel.inspector.physics.remove_shape"),
        y,
    );
    t.hit.register(ids::INSP_PHYS_REMOVE, r);
    y - Spacing::Sm.px() + SECTION_BOTTOM_PAD_PX
}

/// Um corpo sem forma não bate em nada: o aviso da forma solta, ao contrário, e a caixa da imagem.
/// As duas faces (cartões e `PH2D_CARDS=0`) pintam isto; o handler gateia em `body_without_shape`.
pub(super) fn sem_forma(t: &mut Tinta<'_>, y: f32) -> f32 {
    let y = t.frase(y, tr("card.insp.phys.no_shape"), ColorToken::Warn);
    let (r, y) = t.botao(ids::INSP_PHYS_ADD_SHAPE, tr("card.insp.phys.add_shape"), y);
    t.hit.register(ids::INSP_PHYS_ADD_SHAPE, r);
    y - Spacing::Sm.px() + SECTION_BOTTOM_PAD_PX
}

/// A forma diz de quem é, e dá a saída: virar um corpo próprio.
fn de_quem_e(t: &mut Tinta<'_>, y: f32, info: &InspectorPhysicsInfo) -> f32 {
    let solta = info.part_owner.is_empty();
    let y = if solta {
        t.frase(y, tr("card.insp.phys.loose"), ColorToken::Warn)
    } else {
        let frase = tr_with("card.insp.phys.part_of", &[("owner", &info.part_owner)]);
        t.frase(y, &frase, ColorToken::Text2)
    };
    let (r, y) = t.botao(
        ids::INSP_PHYS_ADD,
        tr("panel.inspector.physics.make_independent_body"),
        y,
    );
    t.hit.register(ids::INSP_PHYS_ADD, r);
    if solta {
        t.frase(y, tr("card.insp.phys.drag_hint"), ColorToken::Text3)
    } else {
        y
    }
}

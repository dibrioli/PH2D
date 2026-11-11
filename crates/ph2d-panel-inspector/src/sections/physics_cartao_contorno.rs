//! **Os controlos do «Outline»** no cartão Shape (W4, plano `f2_contorno_e_formas_plano.md`): Detail,
//! Alpha cutoff, Ignore specks under, o aviso de quando o tecto de peças subiu o Detail, e o «Merge by distance».

use super::physics_cartao::Tinta;
use super::rows::num_row_unit;
use super::*;
use ph2d_editor_core::screens::hero::InspectorPhysicsInfo;
use ph2d_editor_core::widget::Unit;
use ph2d_i18n::{tr, tr_with};

/// As três linhas do traço, cada uma com o ponto de chave das outras linhas do cartão. Num Outline à mão
/// ([`travado`]) os três TRAVAM fora de «Edit shape on canvas» (dono 09/11): apagados, sem alvo nem losango,
/// e o bloco dá a razão; dentro da edição destravam e retraçam.
#[allow(clippy::too_many_arguments)]
pub(super) fn controlos(
    scene: &mut VectorScene,
    text: &mut TextSystem,
    theme: Theme,
    hit: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let topo = y;
    let mut y = y;
    let parado = crate::state::current_inspector_physics().is_some_and(|i| travado(&i));
    let tres = [
        ids::INSP_PHYS_OUTLINE_DETAIL,
        ids::INSP_PHYS_OUTLINE_ALPHA,
        ids::INSP_PHYS_OUTLINE_SPECKS,
    ];
    ph2d_editor_core::widget::card_field::parados(if parado { &tres } else { &[] });
    for ((rotulo, unidade), id) in [
        ("card.insp.phys.outline_detail", Some(Unit::Px)),
        ("card.insp.phys.outline_alpha", None),
        ("card.insp.phys.outline_specks", Some(Unit::Px)),
    ]
    .into_iter()
    .zip(tres)
    {
        // O campo parado não regista alvo nem nos cartões nem na face de referência (`num_row_unit`).
        y = num_row_unit(
            scene,
            text,
            theme,
            hit,
            store,
            x,
            w,
            y,
            tr(rotulo),
            id,
            unidade,
            None,
        );
    }
    if parado {
        let bloco = ph2d_editor_core::zones::Rect::new(x, topo, w, y - topo);
        hit.register(ids::INSP_PHYS_OUTLINE_LOCKED, bloco);
    }
    y
}

/// ⭐ **Detail · Alpha cutoff · Ignore specks under travados** (dono 09/11): o Outline tem pontos à mão
/// (`custom`) e «Edit shape on canvas» está desligado. O pintor e o handler leem isto.
#[must_use]
pub(crate) fn travado(info: &InspectorPhysicsInfo) -> bool {
    info.outline.is_some_and(|o| o.custom) && !info.shape_edit_on
}

/// «Too detailed — Detail raised to {x} px to stay under {n} pieces», só quando o tecto subiu.
pub(super) fn aviso(t: &mut Tinta<'_>, y: f32, info: &InspectorPhysicsInfo) -> f32 {
    let Some((usado, n)) = info.outline.and_then(|o| o.raised) else {
        return y;
    };
    let x = ph2d_editor_core::interaction::format_number(f64::from(usado));
    let frase = tr_with("card.insp.phys.outline_raised", &[("x", &x), ("n", &n)]);
    t.frase(y, &frase, ColorToken::Warn)
}

/// **«Merge by distance»** (dono 09/11): a distância, o botão e o que o último «Merge» fez. Só num «Outline».
pub(super) fn fundir(t: &mut Tinta<'_>, y: f32, info: &InspectorPhysicsInfo) -> f32 {
    let Some(o) = info.outline else {
        return y;
    };
    let y = num_row_unit(
        t.scene,
        t.text,
        t.theme,
        t.hit,
        t.store,
        t.x,
        t.w,
        y,
        tr("card.insp.phys.outline_merge_dist"),
        ids::INSP_PHYS_OUTLINE_MERGE_DIST,
        Some(Unit::Px),
        None,
    );
    let (r, mut y) = t.botao(
        ids::INSP_PHYS_OUTLINE_MERGE,
        tr("card.insp.phys.outline_merge"),
        y,
    );
    t.hit.register(ids::INSP_PHYS_OUTLINE_MERGE, r);
    if let Some((n, k)) = o.merged {
        let frase = tr_with("card.insp.phys.outline_merged", &[("n", &n)]);
        y = t.frase(y, &frase, ColorToken::Text3);
        if k > 0 {
            let frase = tr_with("card.insp.phys.outline_merge_kept", &[("k", &k)]);
            y = t.frase(y, &frase, ColorToken::Warn);
        }
    }
    y
}

//! **As quatro seções COMPARTILHADAS do Inspector** — §5 9-Slice, §7 Ordering, §9 Sampling e
//! §10 Material & Blend.
//!
//! ⚠️ **Irmão de [`super::paint_frame`] por CAP de FICHEIRO** (600): a família cabia lá dentro
//! logicamente, mas levava-o a 628. *Um cap de ficheiro e um cap de função medem grandezas
//! diferentes*, e extrair para o mesmo ficheiro curaria um e estouraria o outro — a lição que o
//! par de PRECISAO já pagou em 2026-08-20.
//!
//! Elas andam juntas porque partilham a mesma porta — **qualquer entidade com `Transform`**
//! (as notas deixaram de ter ranhura por posição em 2026-09-30: cada uma guarda a SECÇÃO a que pertence).

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetStore;

use crate::plano::{Plano, emoldurada};

/// ⭐⭐ **As TRÊS seções que TODO objecto tem** — §1 Name, §8 Visibility e §2 Transform.
///
/// ⚠️ **Elas andam juntas por uma PORTA, como as quatro compartilhadas abaixo:** são as únicas que
/// aparecem para *qualquer* entidade seleccionada.
///
/// ⚠️ **Saíram do orquestrador em 2026-09-09**, quando a secção SIGNAL ACTIONS o levou a `253`
/// contra uma catraca de `250`. ⛔ **A catraca só desce**, e levar só a secção nova devolveria o
/// número ao sítio — *ficar no mesmo sítio não é encolher*.
///
/// ⚠️ **Elas usam `begin_section`/`finish_section` directamente**, e não o `live_section!` do
/// orquestrador: aquele macro captura meia dúzia de locais do corpo dele, e é exactamente por
/// isso que este trio nunca tinha saído. Aqui a captura vira argumentos, como nas irmãs.
///
/// ⭐ **Desde 2026-09-29 elas EMPURRAM-SE para o [`Plano`]** em vez de se pintarem — o plano é que
/// as pinta, pela ordem que o artista escolheu. O Nome e a Visibilidade continuam no topo (são as
/// [`ph2d_editor_core::interaction::SECCOES_FIXAS`]).
#[allow(clippy::too_many_arguments)]
pub(crate) fn push_core_sections<'a>(
    plano: &mut Plano<'a>,
    store: &'a WidgetStore,
    inner_x: f32,
    inner_w: f32,
    row_h: f32,
    header_h: f32,
    name_present: bool,
    visibility: bool,
    transform: bool,
) {
    for (presente, id, banda) in [
        (name_present, ids::INSP_LIVE_NAME_SECTION, row_h),
        (visibility, ids::INSP_LIVE_VISIBILITY_SECTION, row_h),
        (transform, ids::INSP_LIVE_TRANSFORM_SECTION, header_h),
    ] {
        if !presente {
            continue;
        }
        // ⚠️ **O corpo sai de um `match` sobre o ID**, e não de três blocos copiados: as três
        // molduras são idênticas, e o que muda é UMA chamada.
        emoldurada(plano, id, store, inner_x, inner_w, banda, move |c, t, y| {
            if id == ids::INSP_LIVE_NAME_SECTION {
                crate::sections::paint_entity_name_row(
                    c.scene, c.text, t, c.hit, store, inner_x, inner_w, y,
                )
            } else if id == ids::INSP_LIVE_VISIBILITY_SECTION {
                crate::paint::visibility_body(c.scene, c.text, t, c.hit, store, inner_x, inner_w, y)
            } else {
                crate::sections::paint_transform_section(
                    c.scene, c.text, t, c.hit, store, inner_x, inner_w, y,
                )
            }
        });
    }
}

/// **§5 9-Slice + §7 Ordering + §9 Sampling + §10 Material & Blend**, moldura e tudo.
///
/// Levantadas do `paint_inspector` pelo mesmo motivo da família da física: aquele orquestrador
/// está numa tolerância de LOC que **só encolhe**, e a §5 (2026-08-21) empurrou-o para 436 contra
/// 414. As quatro vivem juntas porque partilham a mesma porta — qualquer entidade com `Transform`.
///
/// Devolve o novo `y`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn push_shared_sections<'a>(
    plano: &mut Plano<'a>,
    store: &'a WidgetStore,
    inner_x: f32,
    inner_w: f32,
    header_h: f32,
    slice: Option<&'a ph2d_editor_core::screens::hero::InspectorSliceInfo>,
    ordering: Option<&'a ph2d_editor_core::screens::hero::InspectorOrderingInfo>,
    sampling: Option<&'a ph2d_editor_core::screens::hero::InspectorSamplingInfo>,
    blend: Option<&'a ph2d_editor_core::screens::hero::InspectorBlendInfo>,
) {
    // §5 9-Slice — LOGO A SEGUIR à Sprite Sheet, que é a vizinhança que a explica. ⚠️ Aparece
    // para toda sprite, COM ou SEM o componente — sem ele mostra só o «+ Add 9-Slice».
    if let Some(sl) = slice {
        emoldurada(
            plano,
            ids::INSP_LIVE_SLICE_SECTION,
            store,
            inner_x,
            inner_w,
            header_h,
            move |c, t, y| {
                crate::sections::slice_nine::paint_slice_section(
                    c.scene, c.text, t, c.hit, store, inner_x, inner_w, y, sl,
                )
            },
        );
    }
    // §7 Ordering / Sorting — vale para qualquer entidade com Transform, não só sprites.
    if let Some(ord) = ordering {
        emoldurada(
            plano,
            ids::INSP_LIVE_ORDERING_SECTION,
            store,
            inner_x,
            inner_w,
            header_h,
            move |c, t, y| {
                crate::sections::paint_ordering_section(
                    c.scene, c.text, t, c.hit, store, inner_x, inner_w, y, ord,
                )
            },
        );
    }
    // §9 Sampling — irmã da §7.
    if let Some(samp) = sampling {
        emoldurada(
            plano,
            ids::INSP_LIVE_SAMPLING_SECTION,
            store,
            inner_x,
            inner_w,
            header_h,
            move |c, t, y| {
                crate::sections::paint_sampling_section(
                    c.scene, c.text, t, c.hit, store, inner_x, inner_w, y, samp,
                )
            },
        );
    }
    // §10 Material & Blend — irmã da §9.
    if let Some(bl) = blend {
        emoldurada(
            plano,
            ids::INSP_LIVE_BLEND_SECTION,
            store,
            inner_x,
            inner_w,
            header_h,
            move |c, t, y| {
                crate::sections::paint_material_blend_section(
                    c.scene, c.text, t, c.hit, store, inner_x, inner_w, y, bl,
                )
            },
        );
    }
}

/// **§12 Sockets / Named Anchors** (ADR-0072) — a última seção, e a única que precisa do
/// **estado do painel**: qual linha da lista está aberta.
///
/// ⚠️ O índice é **saturado aqui** contra o tamanho da lista. Apagar a última âncora deixa-o a
/// apontar para além do fim, e um editor aberto sobre uma linha que já não existe é a forma mais
/// direta de escrever na âncora errada.
#[allow(clippy::too_many_arguments)]
pub(crate) fn push_anchor_section<'a>(
    plano: &mut Plano<'a>,
    store: &'a WidgetStore,
    inner_x: f32,
    inner_w: f32,
    header_h: f32,
    anchor: Option<&'a ph2d_editor_core::screens::hero::InspectorAnchorInfo>,
    selected: &mut usize,
) {
    let Some(anch) = anchor else {
        // Sem snapshot não há ficha aberta — e o gizmo do canvas tem de saber disso, senão ele
        // continua a oferecer alças de uma âncora que a seção já não mostra.
        crate::state::set_open_anchor_row(None);
        return;
    };
    *selected = (*selected).min(anch.rows.len().saturating_sub(1));
    // ⚠️ **A linha aberta viaja para a SHELL aqui** — é o que dá alças ao gizmo do canvas. Sai da
    // PINTURA e não do despacho de propósito: a pintura corre todo o quadro e conhece o estado
    // final (já corrigido contra o tamanho da lista, na linha acima).
    crate::state::set_open_anchor_row((!anch.rows.is_empty()).then_some(*selected));
    let sel = *selected;
    emoldurada(
        plano,
        ids::INSP_LIVE_ANCHOR_SECTION,
        store,
        inner_x,
        inner_w,
        header_h,
        move |c, t, y| {
            crate::sections::anchors::paint_anchors_section(
                c.scene, c.text, t, c.hit, store, inner_x, inner_w, y, anch, sel,
            )
        },
    );
}

/// **§3 Render Source + §6 Color & Tint + §4 Sprite Sheet** — as três que só existem quando há
/// sprite. Moldura e tudo, como as irmãs deste ficheiro.
///
/// Saíram do `paint_inspector` pelo mesmo cap que levou lá as compartilhadas: a §12
/// Sockets/Anchors empurrou-o para 403 contra uma catraca de 387. *A cura de um teto estourado
/// é o corte.*
#[allow(clippy::too_many_arguments)]
pub(crate) fn push_sprite_sections<'a>(
    plano: &mut Plano<'a>,
    store: &'a WidgetStore,
    inner_x: f32,
    inner_w: f32,
    header_h: f32,
    sprite: Option<&'a ph2d_editor_core::screens::hero::InspectorSpriteInfo>,
) {
    let Some(info) = sprite else {
        return;
    };
    for (section_id, which) in [
        (ids::INSP_LIVE_RENDER_SECTION, 0u8),
        (ids::INSP_LIVE_COLOR_SECTION, 1),
        (ids::INSP_LIVE_SHEET_SECTION, 2),
    ] {
        emoldurada(
            plano,
            section_id,
            store,
            inner_x,
            inner_w,
            header_h,
            move |c, t, y| match which {
                0 => crate::sections::paint_render_source_section(
                    c.scene, c.text, t, c.hit, store, inner_x, inner_w, y, info,
                ),
                1 => crate::sections::paint_color_tint_section(
                    c.scene, c.text, t, c.hit, store, inner_x, inner_w, y,
                ),
                _ => crate::sections::paint_sprite_sheet_section(
                    c.scene, c.text, t, c.hit, store, inner_x, inner_w, y, info,
                ),
            },
        );
    }
}

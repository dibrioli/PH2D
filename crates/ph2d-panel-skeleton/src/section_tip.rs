//! ⭐⭐⭐ **QUEM MANDA NA PONTA DA CURVA** — a linha do *custom handle*, e a lista que ela abre.
//!
//! Ordem do dono (2026-09-16), depois de ver a cena da bifurcação: *«sim, escolher qual dos vários
//! filhos manda na curva. E quero que o modo atual (ninguém manda na curva) seja uma das opções»*.
//!
//! # ⚠️ Ela só existe onde há PERGUNTA
//!
//! Com as alças **autoradas** ninguém deriva tangente de vizinho nenhum, logo não há ponta para
//! escolher — e a lista nem é publicada ([`ph2d_app_skeleton::curve_tip::options`] devolve `None`).
//! *Um selector sem sujeito é a espécie de controlo morto que o `CLAUDE.md` §5.0 nomeia.*
//!
//! # ⛔ A lista vem da SHELL inteira, rótulos incluídos
//!
//! Quem a aplica lê exactamente a mesma lista, e a POSIÇÃO é a escolha — derivá-la outra vez aqui
//! seria a segunda resposta à mesma pergunta, e bastaria um filho nascer entre dois quadros para o
//! artista escolher um osso e outro obedecer.

use crate::state;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::paint::{paint_text, resolve};
use ph2d_editor_core::panel::{PaintCtx, RowCtx, label_col_w};
use ph2d_editor_core::widget::{
    DROPDOWN_SCROLLBAR_ID, Dropdown, DropdownOption, paint_dropdown_chip,
    paint_dropdown_popover_scrolled, scrollbar_is_needed, scrollbar_track_rect,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::{ColorToken, Spacing, Theme};

/// ⭐⭐⭐ **A LINHA** — o rótulo, e um chip que diz **quem manda agora**.
///
/// ⚠️ **Vem logo a seguir às alças**, e a ordem diz o que ela é: primeiro *de onde vêm as alças*,
/// depois *quem manda na ponta* — ler «qual filho» antes de «derivadas ou autoradas» é ler a
/// resposta antes da pergunta.
pub(crate) fn tip_row(r: &mut RowCtx, y: f32) -> f32 {
    let Some(v) = state::bone_tip() else {
        return y;
    };
    let Some(rotulo) = v.rotulos.get(v.ligado).cloned() else {
        return y;
    };
    let gap = Spacing::Xs.px();
    let id = crate::ids::VECTOR_BONE_TIP;
    // ⚠️ **Os filhos que não couberam no pool aparecem no RÓTULO** — uma lista truncada em silêncio
    // é um painel a esconder o que existe, e o pool é fixo porque o chrome não cunha ids em tempo
    // de execução. No caso normal (`escondidos == 0`) o rótulo é o de sempre.
    let etiqueta = if v.escondidos == 0 {
        tr("panel.vector.bone.tip").to_string()
    } else {
        format!("{} +{}", tr("panel.vector.bone.tip"), v.escondidos)
    };
    paint_text(
        r.text_system,
        r.scene,
        &etiqueta,
        r.inner_x,
        y + (r.row_h - r.font) * 0.5,
        r.font,
        label_col_w(r.inner_x, r.inner_w),
        resolve(ColorToken::Text1, r.theme),
    );
    let chip = Rect::new(
        r.inner_x + label_col_w(r.inner_x, r.inner_w) + gap,
        y,
        (r.inner_w - label_col_w(r.inner_x, r.inner_w) - gap).max(1.0),
        r.row_h,
    );
    let open = matches!(
        r.store.get(id),
        Some(InteractiveState::Dropdown { open: true, .. })
    );
    let dd = Dropdown::new(id, "", vec![DropdownOption::new(id, (), rotulo.as_str())])
        .selected(())
        .open(open)
        .visual(r.store.dropdown_visual(id));
    paint_dropdown_chip(&dd, chip, r.scene, r.text_system, r.theme);
    r.hit_index.register(id, chip);
    if open {
        state::set_pending_bone_tip_dd(Some(chip));
    }
    y + r.row_h + r.row_gap
}

/// ⭐⭐⭐ **A LISTA**, pintada no passe DIFERIDO por cima de todas as seções — espelho exacto do
/// [`crate::section_smart::paint_action_popover`], e pela mesma razão: a seção ROLA, e sem o passe
/// diferido a lista seria cortada na borda dela.
pub(crate) fn paint_tip_popover(ctx: &mut PaintCtx, chip: Rect, theme: Theme) {
    let id = crate::ids::VECTOR_BONE_TIP;
    let Some(v) = state::bone_tip() else {
        return;
    };
    let n = v.rotulos.len().min(ids::VECTOR_BONE_TIP_IDS.len());
    if n == 0 {
        return;
    }
    let options: Vec<DropdownOption<usize>> = v
        .rotulos
        .iter()
        .take(n)
        .enumerate()
        .map(|(i, nome)| DropdownOption::new(ids::VECTOR_BONE_TIP_IDS[i], i, nome.as_str()))
        .collect();
    let dd = Dropdown::new(id, "", options)
        .selected(v.ligado.min(n - 1))
        .open(true);

    let panel = dd.popover_rect_clamped(chip, ctx.layout.popover_region());
    let content_h = dd.content_height(chip.h);
    let visible_h = panel.h;
    let max_scroll = (content_h - visible_h).max(0.0);
    {
        let store = ctx.host.store_mut();
        store.set_dropdown_popover(id, panel);
        store.set_panel_content_h(id, content_h);
        store.set_panel_visible_h(id, visible_h);
        if store.panel_scroll(id) > max_scroll {
            store.set_panel_scroll(id, max_scroll);
        }
    }
    let scroll = ctx.host.store().panel_scroll(id).clamp(0.0, max_scroll); // CLAMP-OK: 0.0 literal; max_scroll is a non-negative px extent
    paint_dropdown_popover_scrolled(
        &dd,
        chip,
        panel,
        scroll,
        ctx.host
            .store()
            .scrollbar_visual_for(DROPDOWN_SCROLLBAR_ID, Some(id)),
        ctx.scene,
        ctx.text_system,
        theme,
    );

    // Hit-register só a parte VISÍVEL de cada linha (a barra de rolagem é o alvo do arrasto).
    let hit_index = ctx.host.hit_index_mut();
    for i in 0..n {
        let r = dd.option_rect_in_scrolled(chip, panel, i, scroll);
        let top = r.y.max(panel.y);
        let bot = (r.y + r.h).min(panel.y + panel.h);
        if bot - top >= 1.0 {
            hit_index.register(
                ids::VECTOR_BONE_TIP_IDS[i],
                Rect::new(r.x, top, r.w, bot - top),
            );
        }
    }
    if scrollbar_is_needed(content_h, visible_h) {
        ctx.host
            .hit_index_mut()
            .register(DROPDOWN_SCROLLBAR_ID, scrollbar_track_rect(panel));
    }
}

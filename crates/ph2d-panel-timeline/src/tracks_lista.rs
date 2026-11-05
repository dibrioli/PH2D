//! A lista de «+ Track»: pintada à mão, sobre tudo, e fechada pelo clique de fora.

use super::{ids, prop_label};
use ph2d_editor_core::paint::fill_rounded_rect;
use ph2d_editor_core::panel::{PaintCtx, camada_das_listas};
use ph2d_editor_core::widget::panel_chrome::clamp_menu_to_viewport;
use ph2d_editor_core::widget::{Button, paint_button};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, StrokeToken, Theme};

/// Paint the property list as an overlay when open, and register each option's hit. Call LAST so
/// it sits on top.
///
/// ⚠️ **Ela largava-se DIRECTO para baixo do botão, sem consciência nenhuma do viewport** — e a
/// tabela cresceu para **treze** linhas (as seis da pose · Time · Position · Morph · as quatro do
/// joint), o que são `13 × 28 = 364 px`. Numa janela baixa isso passa a borda de baixo e as
/// últimas cinco ficam **inalcançáveis**: Enio reportou-o com uma foto em que a lista acaba no
/// rodapé do ecrã (2026-08-15). Pior, elas eram **pintadas e HIT-REGISTADAS** lá fora — widgets
/// vivos em coordenadas onde o rato não chega, que é a forma silenciosa deste defeito.
///
/// A cura é a lei que o app já tem para MENUS ([`clamp_menu_to_viewport`], a que todo menu de
/// contexto usa): um menu não está preso a nada, então quando não cabe ele **DESLIZA** para
/// dentro. ⚠️ Não é a lei do popover de um chip (`Dropdown::popover_rect_clamped`), que VIRA para
/// cima e encolhe — virar não resolveria isto, porque aqui nem acima nem abaixo cabia (~239 e
/// ~235 px contra 364).
///
/// ⚠️ **E o resto é NOMEADO em vez de escondido:** numa janela mais baixa que a própria lista não
/// há deslize que a ponha inteira na tela. Ali ela encosta ao topo e o que sobra é **recusado** —
/// nem pintado nem registado —, porque uma linha que existe onde o ponteiro não chega é pior que
/// uma linha ausente: ela conta como oferta. A resposta completa para esse caso é a lista rolar,
/// e ela não foi construída aqui de propósito (a roda sobre esta região é consumida pela timeline
/// ANTES de qualquer popover, então dar-lhe scroll é mexer na ordem do despachante — wave própria,
/// com o seu próprio smoke).
/// Por baixo do chip; sem lugar, por cima, em vez de o apanhar (a lista empurrada para dentro da janela
/// tapava o «+ Track», e um 2.º clique nele carregava em «Max Length»).
pub(super) fn lista_y(chip: Rect, h: f32, janela: Rect) -> f32 {
    let baixo = chip.y + chip.h;
    if baixo + h > janela.y + janela.h && chip.y - h >= janela.y {
        chip.y - h
    } else {
        baixo
    }
}

pub(crate) fn paint_add_track_popover(ctx: &mut PaintCtx, theme: Theme, anchor: Rect, open: bool) {
    if !open {
        ctx.host
            .store_mut()
            .publish_own_list(ids::TIMELINE_ADD_TRACK, None);
        return;
    }
    let n = ids::ADDPROP_BUTTONS.len() as f32;
    let list = clamp_menu_to_viewport(
        anchor.x,
        lista_y(anchor, ROW_H_PX * n, ctx.viewport),
        anchor.w,
        ROW_H_PX * n,
        ctx.viewport,
    );
    ctx.host
        .store_mut()
        .publish_own_list(ids::TIMELINE_ADD_TRACK, Some(list));
    let viewport = ctx.viewport;
    let (host, text_system) = (&mut *ctx.host, &mut *ctx.text_system);
    // A lista desta tabela é pintada à mão: vai à camada das listas pela mesma porta das outras.
    camada_das_listas::por_cima(ctx.scene, |scene| {
        // ⭐ Raio e moldura pela porta do TEMA: a lista flutuante é plana num tema moderno.
        let list_radius = ph2d_editor_core::paint::frame_radius(theme, Radius::Sm.px());
        fill_rounded_rect(
            scene,
            list,
            list_radius,
            crate::folha_tinta::cor(ColorToken::BgElev, theme),
        );
        ph2d_editor_core::paint::stroke_frame(
            scene,
            list,
            list_radius,
            theme,
            ph2d_tokens::visuals::Feel::Rest,
            StrokeToken::Thin.px(),
            crate::folha_tinta::cor(ColorToken::Border, theme),
        );
        let mut y = list.y;
        for (id, prop) in ids::ADDPROP_BUTTONS {
            let r = Rect::new(list.x, y, list.w, ROW_H_PX);
            y += ROW_H_PX;
            // A linha tem de caber INTEIRA na tela. Meia linha desenhada na borda é uma oferta que o
            // artista lê e não consegue apertar.
            if r.y < viewport.y || r.y + r.h > viewport.y + viewport.h {
                continue;
            }
            let state = host.store().button_visual(id);
            let btn = Button::new(id, prop_label(prop)).visual(state);
            crate::folha_tinta::no_tom(theme, || {
                paint_button(&btn, r, scene, text_system, theme);
            });
            camada_das_listas::alvo(host.hit_index_mut(), id, r);
        }
    });
}

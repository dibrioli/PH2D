//! O quadro da janela: o âmbito, e os cartões Grid, Pixel Grid e Checkerboard, flutuando à esquerda
//! da coluna (a régua da janela UNDO HISTORY).

use ph2d_a11y::NodeId;
use ph2d_editor_core::panel::cards::banda::metrics as m;
use ph2d_editor_core::panel::cards::ferramenta::{caixa, cartao, opcoes};
use ph2d_editor_core::panel::cards::{CardCtx, CardDef, Cell, Widget, borda, paint_card_list};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::card_field::Glyph;
use ph2d_editor_core::widget::panel_chrome::panel_drag_handle_rect;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_pixel_core::PixelPrefs;
use ph2d_tokens::Spacing;

use crate::canal::{Caixa, Cor, Numero, Scope};
use crate::{PixelPrefsPanel, PixelPrefsPanelState, canal, event, ids};

fn solta(ctx: &mut PaintCtx) -> Rect {
    let area = ctx.layout.draw_area;
    let livre = ctx.layout.clear_of_left_list(area);
    let teto = {
        let r = ph2d_editor_core::ruler::top_band(area);
        r.y + r.h + Spacing::Sm.px()
    };
    let fundo = area.y + area.h - Spacing::Sm.px();
    let coluna = ctx.slot.x;
    let store = ctx.host.store_mut();
    let node = PixelPrefsPanel::NODE_ID;
    let esquerda = livre.x.max(area.x);
    let w = borda::largura(store, node, area, coluna)
        .min((coluna - Spacing::Lg.px() - esquerda).max(0.0));
    let base = (coluna - Spacing::Lg.px() - w).max(esquerda);
    let (dx, dy) = store.blender_picker_offset(node);
    let x = (base + dx).clamp(esquerda, base);
    let y = (teto + dy).clamp(teto, (fundo - m::LIST_HEAD_H).max(teto));
    if (x - base, y - teto) != (dx, dy) {
        store.set_blender_picker_offset(node, x - base, y - teto);
    }
    Rect::new(x, y, w, (fundo - y).max(0.0))
}

fn numero(n: Numero, chave: &'static str) -> Cell<'static> {
    Cell::new(Widget::Field {
        id: ids::numero(n),
        glyph: Glyph::Text(tr(chave)),
    })
}

fn hex(c: [u8; 4]) -> String {
    format!("#{:02X}{:02X}{:02X}", c[0], c[1], c[2])
}

fn amostra(
    c: &mut CardCtx<'_>,
    p: &PixelPrefs,
    cor: Cor,
    opacidade: Option<Numero>,
    y: f32,
) -> f32 {
    let rgba = canal::cor(p, cor);
    let codigo = hex(rgba);
    let s = Cell::new(Widget::Swatch {
        id: ids::cor(cor),
        rgba: Some(rgba),
        text: &codigo,
        edit: None,
        detach: None,
    })
    .label(tr(match cor {
        Cor::CheckerA => "panel.pixel_prefs.color_a",
        Cor::CheckerB => "panel.pixel_prefs.color_b",
        _ => "panel.pixel_prefs.color",
    }));
    match opacidade {
        Some(n) => c.row(
            &[
                s,
                numero(n, "panel.pixel_prefs.opacity_glyph").label(tr("panel.pixel_prefs.opacity")),
            ],
            y,
        ),
        None => c.row(&[s], y),
    }
}

fn marca(c: &mut CardCtx<'_>, p: &PixelPrefs, k: Caixa, chave: &'static str, y: f32) -> f32 {
    caixa(c, ids::caixa(k), chave, canal::caixa(p, k), y)
}

fn botao(id: NodeId, chave: &'static str) -> Cell<'static> {
    Cell::new(Widget::Button {
        id,
        text: tr(chave),
        primary: false,
    })
}

fn cartao_ambito(scope: Scope, tem: bool) -> CardDef<'static> {
    cartao(
        ids::SEC_SCOPE,
        tr("panel.pixel_prefs.sec_scope"),
        None,
        move |c, y| {
            let o = opcoes(
                &[
                    (ids::SCOPE_DOC, tr("panel.pixel_prefs.scope_doc")),
                    (ids::SCOPE_NEW, tr("panel.pixel_prefs.scope_new")),
                ],
                usize::from(scope == Scope::New),
            );
            let y = c.full(None, Widget::Choice(&o), y);
            if !tem {
                return c.hint(tr("panel.pixel_prefs.no_document"), y);
            }
            let mut linha = vec![botao(ids::RESET, "panel.pixel_prefs.reset")];
            if scope == Scope::Document {
                linha.insert(
                    0,
                    botao(ids::SET_DEFAULTS, "panel.pixel_prefs.set_defaults"),
                );
            }
            c.row(&linha, y)
        },
    )
}

fn cartao_grade(p: PixelPrefs) -> CardDef<'static> {
    cartao(
        ids::SEC_GRID,
        tr("panel.pixel_prefs.sec_grid"),
        None,
        move |c, y| {
            let y = marca(c, &p, Caixa::GridVisible, "panel.pixel_prefs.visible", y);
            let y = c.row(
                &[
                    numero(Numero::GridX, "panel.pixel_prefs.x")
                        .label(tr("panel.pixel_prefs.origin")),
                    numero(Numero::GridY, "panel.pixel_prefs.y"),
                ],
                y,
            );
            let y = c.row(
                &[
                    numero(Numero::GridW, "panel.pixel_prefs.w")
                        .label(tr("panel.pixel_prefs.size")),
                    numero(Numero::GridH, "panel.pixel_prefs.h"),
                ],
                y,
            );
            let y = amostra(c, &p, Cor::Grid, Some(Numero::GridOpacity), y);
            marca(c, &p, Caixa::GridAuto, "panel.pixel_prefs.auto", y)
        },
    )
}

fn cartao_pixel(p: PixelPrefs) -> CardDef<'static> {
    cartao(
        ids::SEC_PIXEL_GRID,
        tr("panel.pixel_prefs.sec_pixel_grid"),
        None,
        move |c, y| {
            let y = marca(c, &p, Caixa::PixelVisible, "panel.pixel_prefs.visible", y);
            let y = amostra(c, &p, Cor::PixelGrid, Some(Numero::PixelOpacity), y);
            marca(c, &p, Caixa::PixelAuto, "panel.pixel_prefs.auto", y)
        },
    )
}

fn cartao_xadrez(p: PixelPrefs) -> CardDef<'static> {
    cartao(
        ids::SEC_CHECKER,
        tr("panel.pixel_prefs.sec_checker"),
        None,
        move |c, y| {
            let y = c.row(
                &[
                    numero(Numero::CheckerW, "panel.pixel_prefs.w")
                        .label(tr("panel.pixel_prefs.size")),
                    numero(Numero::CheckerH, "panel.pixel_prefs.h"),
                ],
                y,
            );
            let y = amostra(c, &p, Cor::CheckerA, None, y);
            let y = amostra(c, &p, Cor::CheckerB, None, y);
            marca(c, &p, Caixa::CheckerZoom, "panel.pixel_prefs.apply_zoom", y)
        },
    )
}

pub(crate) fn paint(st: &mut PixelPrefsPanelState, ctx: &mut PaintCtx) {
    let node = PixelPrefsPanel::NODE_ID;
    if canal::take_abrir() {
        ctx.host.set_panel_visible(PixelPrefsPanel::ID, true);
    }
    if !ctx.host.panel_visible(PixelPrefsPanel::ID) {
        ctx.host.store_mut().clear_panel_rect(node);
        return;
    }
    event::semeia(st, ctx.host.store_mut());
    let p = canal::view().of(st.scope);
    let mut cartoes = vec![cartao_ambito(st.scope, p.is_some())];
    if let Some(p) = p {
        cartoes.extend([cartao_grade(p), cartao_pixel(p), cartao_xadrez(p)]);
    }
    let slot = solta(ctx);
    let encaixe = ctx.slot;
    ctx.slot = slot;
    let titulo = <PixelPrefsPanel as Panel>::TITLE.tr().to_owned();
    let lista = paint_card_list(ctx, node, &titulo, &cartoes);
    ctx.slot = encaixe;
    ctx.host.store_mut().set_panel_rect(node, lista);
    let reserva = (m::HEAD_ICON_BOX + m::LIST_HEAD_PAD_X) * 2.0;
    ctx.host.hit_index_mut().register(
        ids::DRAG,
        panel_drag_handle_rect(lista, m::LIST_HEAD_H, reserva),
    );
    borda::regista(ctx.host.store_mut(), ids::EDGE, node);
    borda::pega(ctx, ids::EDGE, lista);
}

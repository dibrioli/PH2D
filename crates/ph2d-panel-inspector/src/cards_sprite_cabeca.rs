//! O cartão da miniatura dentro do cartão Image ([ficha](../../../docs/GameEngineFuncional/componentes/01_sprite.md)):
//! a imagem, o nome, o tamanho e os verbos numa superfície própria; clicar na imagem é trocá-la.

use crate::ids;
use crate::slot_image::{self, SlotImage};
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::IconId;
use ph2d_editor_core::InspectorImageUse;
use ph2d_editor_core::paint::{
    fill_rounded_rect, paint_text_elided, paint_text_title_elided, push_rounded_clip, resolve,
    stroke_rounded_rect,
};
use ph2d_editor_core::panel::cards::CardCtx;
use ph2d_editor_core::panel::cards::verbos::{VERBO_H, Verbo};
use ph2d_editor_core::screens::hero::InspectorSpriteInfo;
use ph2d_editor_core::widget::ButtonState;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, TypeToken};

/// A miniatura é quadrada e mede três botões: a imagem é o que se lê primeiro no cartão (referência do dono, 05/11).
const LADOS: f32 = 3.0; // LITERAL-PX-OK: conta de botões, não geometria
/// Numa coluna estreita ela cede até dois botões.
const LADOS_MINIMOS: f32 = 2.0; // LITERAL-PX-OK: conta de botões, não geometria
const NOME_PX: f32 = TypeToken::Base.px();
const TAMANHO_PX: f32 = TypeToken::Xs.px();

thread_local! {
    static TROCA_PINTADA: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// O ícone de troca pintou no último quadro? Para o teste do hover.
#[doc(hidden)]
#[must_use]
pub fn troca_pintada() -> bool {
    TROCA_PINTADA.with(std::cell::Cell::get)
}

/// `256 × 128 px`, e em Repeated o tamanho no mundo entre parênteses.
fn linha_do_tamanho(info: &InspectorSpriteInfo) -> Option<String> {
    let (pw, ph) = info.source_pixels?;
    if info.image_use == InspectorImageUse::Repeated {
        return Some(tr_with(
            "card.insp.image_size_repeated",
            &[
                ("pw", &pw),
                ("ph", &ph),
                ("size", &crate::cards_objeto::tamanho(info)),
            ],
        ));
    }
    Some(tr_with(
        "panel.inspector.render_source.size_px",
        &[("pw", &pw), ("ph", &ph)],
    ))
}

fn nome_da_imagem(img: Option<&SlotImage>) -> String {
    img.map_or_else(
        || tr("panel.inspector.image.none").to_owned(),
        |i| {
            if i.name.is_empty() {
                tr("card.insp.image_unnamed").to_owned()
            } else {
                i.name.clone()
            }
        },
    )
}

/// A superfície da miniatura: a altura é sempre a mesma, seja qual for o degrau dos botões. Devolve o fundo.
pub(crate) fn cabeca(c: &mut CardCtx<'_>, y: f32, info: &InspectorSpriteInfo) -> f32 {
    let theme = c.ink.theme;
    let (x0, w0) = (c.x, c.w);
    let pad = Spacing::Xs.px();
    let gap = Spacing::Md.px();
    let img = slot_image::current_slot_image(info.entity_bits);
    let nome = nome_da_imagem(img.as_ref());
    let tamanho = linha_do_tamanho(info);
    // A linha do tamanho mostra-se inteira: a miniatura cede o que ela pede, quadrada e até ao mínimo.
    let pede_tamanho = tamanho
        .as_deref()
        .map_or(0.0, |t| c.text_system.prefix_width(t, TAMANHO_PX).ceil());
    let util = w0 - pad * 2.0;
    let lado = (util - gap - pede_tamanho).clamp(VERBO_H * LADOS_MINIMOS, VERBO_H * LADOS);
    let alto = VERBO_H * LADOS;
    let tx = x0 + pad + lado + gap;
    let tw = (util - lado - gap).max(0.0);
    let fundo = y + pad * 2.0 + alto;
    // O fundo da lista: um degrau abaixo do cartão e dos botões, que ficam legíveis por cima dele.
    let superficie = c.ink.color(ColorToken::KanbanList);
    fill_rounded_rect(
        c.scene,
        Rect::new(x0, y, w0, fundo - y),
        Radius::Sm.px(),
        superficie,
    );
    let topo = y + pad;
    let foto = Rect::new(x0 + pad, topo + (alto - lado) * 0.5, lado, lado);
    miniatura(c, foto, img.as_ref());
    c.hit_index.register(ids::INSP_RENDER_TEXTURE_SLOT, foto);
    troca_em_hover(c, foto);
    let nome_y = topo + pad;
    paint_text_title_elided(
        c.text_system,
        c.scene,
        &nome,
        tx,
        nome_y,
        NOME_PX,
        tw,
        resolve(ColorToken::Text1, theme),
    );
    if let Some(t) = tamanho.as_deref() {
        paint_text_elided(
            c.text_system,
            c.scene,
            t,
            tx,
            nome_y + NOME_PX + m::FIELD_ROW_GAP,
            TAMANHO_PX,
            tw,
            resolve(ColorToken::Text2, theme),
        );
    }
    (c.x, c.w) = (tx, tw);
    let _ = c.verbos(
        &[
            Verbo {
                id: ids::INSP_IMAGE_EDIT_PIXEL,
                seta: IconId::Export,
                icone: IconId::PixelSprite,
                nome: tr("card.insp.verb_pixel"),
            },
            Verbo {
                id: ids::INSP_IMAGE_EDIT_PAINTER,
                seta: IconId::Export,
                icone: IconId::Painter,
                nome: tr("card.insp.verb_painter"),
            },
        ],
        topo + alto - pad - VERBO_H,
    );
    (c.x, c.w) = (x0, w0);
    fundo
}

/// ⭐ Dono 05/11: em hover, a borda da miniatura acende e o ícone de troca entra a girar no canto dela.
/// O giro é o `hover_live`, o relógio da UI: com movimento reduzido ele já chega parado.
fn troca_em_hover(c: &mut CardCtx<'_>, foto: Rect) {
    TROCA_PINTADA.with(|p| p.set(false));
    let (estado, t) = c.store.button_visual(ids::INSP_RENDER_TEXTURE_SLOT);
    if !matches!(estado, ButtonState::Hovered | ButtonState::Pressed) {
        return;
    }
    let theme = c.ink.theme;
    stroke_rounded_rect(
        c.scene,
        foto,
        Radius::Sm.px(),
        StrokeToken::Thick.px(),
        resolve(ColorToken::Accent, theme),
    );
    let lado = m::HEAD_ICON_GLYPH;
    let inset = Spacing::Xxs.px();
    let canto = Rect::new(foto.x + foto.w - lado - inset, foto.y + inset, lado, lado);
    fill_rounded_rect(
        c.scene,
        canto,
        lado * 0.5,
        resolve(ColorToken::Accent, theme),
    );
    let giro = f64::from(1.0 - t.clamp(0.0, 1.0)) * -std::f64::consts::PI;
    TROCA_PINTADA.with(|p| p.set(true));
    ph2d_editor_core::paint::paint_icon_rotated(
        c.scene,
        IconId::RotateCw,
        canto,
        resolve(ColorToken::Text1, theme),
        StrokeToken::Default.px(),
        giro,
    );
}

/// A imagem sobre o xadrez da transparência, com o aspecto dela, as duas recortadas pelo raio da miniatura.
fn miniatura(c: &mut CardCtx<'_>, r: Rect, img: Option<&SlotImage>) {
    push_rounded_clip(c.scene, r, Radius::Sm.px());
    ph2d_editor_core::widget::paint_checker(c.scene, r, 0.0);
    if let Some(i) = img
        && let Some(s) = slot_image::stable(i)
    {
        let (tw, th) = (i.w.max(1) as f32, i.h.max(1) as f32);
        let k = (r.w / tw).min(r.h / th);
        let (dw, dh) = (tw * k, th * k);
        let x0 = f64::from(r.x + (r.w - dw) * 0.5);
        let y0 = f64::from(r.y + (r.h - dh) * 0.5);
        c.scene.draw_stable_image(
            &s,
            (x0, y0, x0 + f64::from(dw), y0 + f64::from(dh)),
            ph2d_vector::ImageQuality::Medium,
        );
    }
    c.scene.pop_layer();
}

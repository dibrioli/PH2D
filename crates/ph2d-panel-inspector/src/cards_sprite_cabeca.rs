//! O cartão da miniatura dentro do cartão Image ([ficha](../../../docs/GameEngineFuncional/componentes/01_sprite.md)):
//! a imagem, o nome, o tamanho e os verbos numa superfície própria; clicar na imagem é trocá-la.

use crate::ids;
use crate::slot_image::{self, SlotImage};
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::IconId;
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text, resolve, stroke_rounded_rect};
use ph2d_editor_core::panel::cards::{CardCtx, Cell, Widget};
use ph2d_editor_core::screens::hero::InspectorSpriteInfo;
use ph2d_editor_core::widget::ButtonState;
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{InspectorDraw, InspectorImageUse};
use ph2d_i18n::{tr, tr_with};
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, TypeToken};

/// A miniatura mede três campos: a imagem é o que se lê primeiro no cartão (Godot, GameMaker).
const LINHAS_DA_MINIATURA: f32 = 3.0; // LITERAL-PX-OK: conta de campos, não geometria
/// Numa coluna estreita ela cede até dois campos: continua maior que um botão.
const LINHAS_MINIMAS: f32 = 2.0; // LITERAL-PX-OK: conta de campos, não geometria

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

/// A superfície da miniatura; devolve o fundo dela.
pub(crate) fn cabeca(c: &mut CardCtx<'_>, y: f32, info: &InspectorSpriteInfo) -> f32 {
    let theme = c.ink.theme;
    let (x0, w0) = (c.x, c.w);
    let pad = Spacing::Sm.px();
    let gap = Spacing::Sm.px();
    let img = slot_image::current_slot_image(info.entity_bits);
    let nome = nome_da_imagem(img.as_ref());
    let tamanho = linha_do_tamanho(info);
    let (pixel, painter) = (
        tr("card.insp.edit_in_pixel"),
        tr("card.insp.edit_in_painter"),
    );
    let botoes = [
        Cell::new(Widget::Button {
            id: ids::INSP_IMAGE_EDIT_PIXEL,
            text: pixel,
            primary: false,
        }),
        Cell::new(Widget::Button {
            id: ids::INSP_IMAGE_EDIT_PAINTER,
            text: painter,
            primary: false,
        }),
    ];
    // A coluna da direita mostra inteiros o botão mais largo e o tamanho: a miniatura cede o que eles pedem.
    let pede_botao = botoes
        .iter()
        .map(|b| c.largura_pedida(b))
        .fold(0.0_f32, f32::max);
    let pede_tamanho = tamanho.as_deref().map_or(0.0, |t| {
        c.text_system.prefix_width(t, TypeToken::Sm.px()).ceil()
    });
    let util = w0 - pad * 2.0;
    let lado = (util - gap - pede_botao.max(pede_tamanho)).clamp(
        m::FIELD_H * LINHAS_MINIMAS,
        m::FIELD_H * LINHAS_DA_MINIATURA,
    );
    let tx = x0 + pad + lado + gap;
    let tw = (util - lado - gap).max(0.0);
    (c.x, c.w) = (tx, tw);
    let lado_a_lado = c.cabem_lado_a_lado(&botoes);
    let botoes_h = if lado_a_lado {
        m::FIELD_H
    } else {
        m::FIELD_H * 2.0 + m::FIELD_ROW_GAP
    };
    let texto_h = TypeToken::Md.px() + m::FIELD_ROW_GAP + TypeToken::Sm.px() + Spacing::Sm.px();
    let alto = (texto_h + botoes_h).max(m::FIELD_H * LINHAS_DA_MINIATURA);
    let fundo = y + pad * 2.0 + alto;
    // O fundo da lista: um degrau abaixo do cartão e dos botões, que ficam legíveis por cima dele.
    let superficie = c.ink.color(ColorToken::KanbanList);
    fill_rounded_rect(
        c.scene,
        Rect::new(x0, y, w0, fundo - y),
        Radius::Md.px(),
        superficie,
    );
    let topo = y + pad;
    let foto = Rect::new(x0 + pad, topo, lado, alto);
    miniatura(c, foto, img.as_ref());
    c.hit_index.register(ids::INSP_RENDER_TEXTURE_SLOT, foto);
    troca_em_hover(c, foto);
    paint_text(
        c.text_system,
        c.scene,
        &nome,
        tx,
        topo,
        TypeToken::Md.px(),
        tw,
        resolve(ColorToken::Text1, theme),
    );
    if let Some(t) = tamanho.as_deref() {
        paint_text(
            c.text_system,
            c.scene,
            t,
            tx,
            topo + TypeToken::Md.px() + m::FIELD_ROW_GAP,
            TypeToken::Sm.px(),
            tw,
            resolve(ColorToken::Text2, theme),
        );
    }
    let _ = c.compact(&botoes, fundo - pad - botoes_h);
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
        0.0,
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

/// A imagem sobre o xadrez da transparência, com o aspecto dela.
fn miniatura(c: &mut CardCtx<'_>, r: Rect, img: Option<&SlotImage>) {
    // Sem raio: o xadrez encolhe pelo raio que recebe, e a imagem por cima não encolheria com ele.
    ph2d_editor_core::widget::paint_checker(c.scene, r, 0.0);
    let Some(i) = img else {
        return;
    };
    let (tw, th) = (i.w.max(1) as f32, i.h.max(1) as f32);
    let k = (r.w / tw).min(r.h / th);
    let (dw, dh) = (tw * k, th * k);
    desenha(
        c,
        Rect::new(r.x + (r.w - dw) * 0.5, r.y + (r.h - dh) * 0.5, dw, dh),
        i,
    );
}

fn desenha(c: &mut CardCtx<'_>, r: Rect, img: &SlotImage) {
    let Some(s) = slot_image::stable(img) else {
        return;
    };
    let (x0, y0) = (f64::from(r.x), f64::from(r.y));
    c.scene.draw_stable_image(
        &s,
        (x0, y0, x0 + f64::from(r.w), y0 + f64::from(r.h)),
        ph2d_vector::ImageQuality::Medium,
    );
}

/// As cópias que a tira mostra: as de uma fileira, enquanto cada uma tiver um campo de largura.
fn copias_na_tira(largura: f32, pedidas: f32) -> usize {
    let cabem = (largura / m::FIELD_H).floor().max(1.0);
    pedidas.ceil().clamp(1.0, cabem) as usize
}

/// ⭐ A tira de prévia do Repeated: as cópias de uma fileira coladas, na largura do corpo e no aspecto da imagem.
pub(crate) fn tira_do_repeated(c: &mut CardCtx<'_>, y: f32, info: &InspectorSpriteInfo) -> f32 {
    let InspectorDraw::Tiled { copies } = info.draw else {
        return y;
    };
    let Some(img) = slot_image::current_slot_image(info.entity_bits) else {
        return y;
    };
    let n = copias_na_tira(c.w, copies[0]);
    let aspecto = img.h.max(1) as f32 / img.w.max(1) as f32;
    // Uma imagem alta numa fileira curta não abre o cartão: a altura pára na da miniatura.
    let alto = (c.w / n as f32 * aspecto).min(m::FIELD_H * LINHAS_DA_MINIATURA);
    let largo = alto / aspecto;
    let y = y + m::FIELD_ROW_GAP;
    // Colunas em píxel inteiro: uma costura fracionária mostraria o fundo entre as cópias.
    let coluna = |i: usize| (c.x + i as f32 * largo).round();
    let colunas: Vec<f32> = (0..=n).map(coluna).collect();
    for par in colunas.windows(2) {
        desenha(c, Rect::new(par[0], y, par[1] - par[0], alto), &img);
    }
    y + alto
}

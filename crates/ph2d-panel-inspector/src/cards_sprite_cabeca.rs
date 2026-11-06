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
/// O registo para o teste não cresce sem fim: quem não o lê paga só esta moldura.
const PINTADAS_GUARDADAS: usize = 8;
/// A opacidade do véu sobre o xadrez, medida na referência do dono: células perto de `#3a4150` e `#4a5262`.
const VEU_DO_XADREZ: f32 = 0.7; // LITERAL-PX-OK: opacidade, não geometria
const NOME_PX: f32 = TypeToken::Base.px();
const TAMANHO_PX: f32 = TypeToken::Sm.px();

thread_local! {
    static TROCA_PINTADA: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static CABECA_PINTADA: std::cell::Cell<Option<CabecaPintada>> = const { std::cell::Cell::new(None) };
    static IMAGENS_PINTADAS: std::cell::RefCell<Vec<(u64, String)>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// `(objecto, nome da imagem)` de cada moldura pintada desde a última leitura. Para o teste do cartão preso.
#[doc(hidden)]
#[must_use]
pub fn imagens_pintadas() -> Vec<(u64, String)> {
    IMAGENS_PINTADAS.with(std::cell::RefCell::take)
}

/// O que a última moldura pintou. Para o teste dos degraus.
#[doc(hidden)]
#[derive(Copy, Clone, Debug)]
pub struct CabecaPintada {
    pub moldura: Rect,
    /// A faixa do nome.
    pub nome: Rect,
    /// A linha do tamanho dentro da moldura (degrau largo).
    pub tamanho: Option<Rect>,
    /// A linha do tamanho no título do cartão (degrau estreito).
    pub tamanho_no_titulo: Option<Rect>,
}

/// A última [`CabecaPintada`].
#[doc(hidden)]
#[must_use]
pub fn cabeca_pintada() -> Option<CabecaPintada> {
    CABECA_PINTADA.with(std::cell::Cell::get)
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
    Some(tamanho_em_px((pw, ph)))
}

fn tamanho_em_px((pw, ph): (u32, u32)) -> String {
    tr_with(
        "panel.inspector.render_source.size_px",
        &[("pw", &pw), ("ph", &ph)],
    )
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

/// A superfície da miniatura: a altura é sempre a mesma, seja qual for o degrau. Devolve o fundo.
///
/// Degrau largo (o tamanho e os dois botões inteiros, lado a lado, cabem ao lado da miniatura): nome, tamanho e
/// botões. Degrau estreito: o tamanho sobe para o título do cartão e os botões empilham-se na coluna.
pub(crate) fn cabeca(c: &mut CardCtx<'_>, y: f32, info: &InspectorSpriteInfo) -> f32 {
    let theme = c.ink.theme;
    let (x0, w0) = (c.x, c.w);
    let pad = Spacing::Xs.px();
    let gap = Spacing::Md.px();
    let img = slot_image::current_slot_image(info.entity_bits);
    let nome = nome_da_imagem(img.as_ref());
    let tamanho = linha_do_tamanho(info);
    IMAGENS_PINTADAS.with(|v| {
        let mut v = v.borrow_mut();
        if v.len() < PINTADAS_GUARDADAS {
            v.push((info.entity_bits, nome.clone()));
        }
    });
    let verbos = [
        Verbo {
            id: ids::INSP_IMAGE_EDIT_PIXEL,
            seta: IconId::Export,
            nome: tr("card.insp.verb_pixel"),
        },
        Verbo {
            id: ids::INSP_IMAGE_EDIT_PAINTER,
            seta: IconId::Export,
            nome: tr("card.insp.verb_painter"),
        },
    ];
    // A linha do tamanho mostra-se inteira: a miniatura cede o que ela pede, quadrada e até ao mínimo.
    let pede_tamanho = tamanho
        .as_deref()
        .map_or(0.0, |t| c.text_system.prefix_width(t, TAMANHO_PX).ceil());
    let util = w0 - pad * 2.0;
    let lado_largo = (util - gap - pede_tamanho).clamp(VERBO_H * LADOS_MINIMOS, VERBO_H * LADOS);
    let tw_largo = (util - lado_largo - gap).max(0.0);
    let largo = pede_tamanho <= tw_largo && c.verbos_cabem_lado_a_lado(&verbos, tw_largo);
    // Na pilha é a coluna dos botões que pede: a miniatura cede até ao mínimo, nunca os botões.
    let pede_coluna = verbos
        .iter()
        .map(|v| c.verbo_pede(v))
        .fold(0.0_f32, f32::max);
    let lado = if largo {
        lado_largo
    } else {
        (util - gap - pede_coluna).clamp(VERBO_H * LADOS_MINIMOS, VERBO_H * LADOS)
    };
    let alto = VERBO_H * LADOS;
    let tx = x0 + pad + lado + gap;
    let tw = (util - lado - gap).max(0.0);
    let fundo = y + pad * 2.0 + alto;
    // Um degrau abaixo do cartão, e os botões um degrau acima dela (a referência do dono).
    let superficie = c.ink.color(ColorToken::CardField);
    let moldura = Rect::new(x0, y, w0, fundo - y);
    fill_rounded_rect(c.scene, moldura, Radius::Sm.px(), superficie);
    let topo = y + pad;
    let foto = Rect::new(x0 + pad, topo + (alto - lado) * 0.5, lado, lado);
    miniatura(c, foto, img.as_ref());
    c.hit_index.register(ids::INSP_RENDER_TEXTURE_SLOT, foto);
    troca_em_hover(c, foto);
    // Na pilha os botões tomam o fundo da moldura e o nome sobe à margem dela.
    let nome_y = if largo { topo + pad } else { topo };
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
    let mut pintada = CabecaPintada {
        moldura,
        nome: Rect::new(tx, nome_y, tw, NOME_PX),
        tamanho: None,
        tamanho_no_titulo: None,
    };
    if let Some(t) = tamanho.as_deref() {
        if largo {
            let ty = nome_y + NOME_PX + m::FIELD_ROW_GAP;
            paint_text_elided(
                c.text_system,
                c.scene,
                t,
                tx,
                ty,
                TAMANHO_PX,
                tw,
                c.ink.color(ColorToken::CardLabel),
            );
            pintada.tamanho = Some(Rect::new(tx, ty, pede_tamanho, TAMANHO_PX));
        } else {
            // Inteiro, ou sem os parênteses do Repeated (os campos Width/Height mostram-nos).
            pintada.tamanho_no_titulo = c.resumo_no_titulo(t).or_else(|| {
                info.source_pixels
                    .and_then(|px| c.resumo_no_titulo(&tamanho_em_px(px)))
            });
        }
    }
    (c.x, c.w) = (tx, tw);
    if largo {
        c.verbos_lado_a_lado(&verbos, topo + alto - pad - VERBO_H);
    } else {
        c.verbos_em_pilha(&verbos, topo + alto);
    }
    (c.x, c.w) = (x0, w0);
    CABECA_PINTADA.with(|p| p.set(Some(pintada)));
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
    // O xadrez da referência é escuro: claro, ele brilha mais que o desenho.
    let veu = c
        .ink
        .color(ColorToken::KanbanList)
        .with_alpha(VEU_DO_XADREZ);
    c.scene
        .fill_rect(ph2d_editor_core::paint::rect_to_vello(r), veu);
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

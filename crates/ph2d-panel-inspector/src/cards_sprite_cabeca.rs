//! O cartão da miniatura dentro do cartão Image ([ficha](../../../docs/GameEngineFuncional/componentes/01_sprite.md)):
//! a imagem, o nome, o tamanho e os verbos numa superfície própria; clicar na imagem é trocá-la.

use crate::ids;
use crate::slot_image::{self, SlotImage};
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::IconId;
use ph2d_editor_core::InspectorImageUse;
use ph2d_editor_core::paint::{
    fill_rounded_rect, frame_radius, paint_text_elided, paint_text_title_elided, push_rounded_clip,
    resolve, stroke_frame,
};
use ph2d_editor_core::panel::cards::CardCtx;
use ph2d_editor_core::panel::cards::verbos::{VERBO_H, Verbo};
use ph2d_editor_core::pixel_panel::PixelSpriteCard;
use ph2d_editor_core::screens::hero::InspectorSpriteInfo;
use ph2d_editor_core::widget::ButtonState;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_tokens::visuals::Feel;
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, TypeToken};
use ph2d_vector::ImageQuality;

/// A miniatura é quadrada e mede três botões: a imagem é o que se lê primeiro no cartão (referência do dono, 05/11).
const LADOS: f32 = 3.0; // LITERAL-PX-OK: conta de botões, não geometria
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
    static MINIATURA_PINTADA: std::cell::RefCell<Option<std::sync::Arc<Vec<u8>>>> =
        const { std::cell::RefCell::new(None) };
}

/// `(objecto, nome da imagem)` de cada moldura pintada desde a última leitura. Para o teste do cartão preso.
#[doc(hidden)]
#[must_use]
pub fn imagens_pintadas() -> Vec<(u64, String)> {
    IMAGENS_PINTADAS.with(std::cell::RefCell::take)
}

/// Os píxeis da última miniatura. Para o teste que a vê mudar depois de pintar.
#[doc(hidden)]
#[must_use]
pub fn miniatura_pintada() -> Option<std::sync::Arc<Vec<u8>>> {
    MINIATURA_PINTADA.with(|p| p.borrow().clone())
}

/// O que a última moldura pintou. Para o teste dos degraus.
#[doc(hidden)]
#[derive(Copy, Clone, Debug)]
pub struct CabecaPintada {
    pub moldura: Rect,
    /// A faixa do nome.
    pub nome: Rect,
    /// A linha do tamanho, sempre dentro da moldura (dono, 06/11).
    pub tamanho: Option<Rect>,
    /// A miniatura.
    pub foto: Rect,
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

/// Em Animated a linha diz o corte (`1536 × 768 px · 8 × 4`): a grade mora no editor. A forma mais curta é só o corte.
fn formas_com_o_corte(px: Option<(u32, u32)>, [c, l]: [u32; 2]) -> Vec<String> {
    let mut formas: Vec<String> = px
        .into_iter()
        .flat_map(|(pw, ph)| {
            [
                "card.insp.image_size_cut",
                "card.insp.image_size_cut_bare",
                "card.insp.image_size_cut_tight",
            ]
            .map(|k| tr_with(k, &[("pw", &pw), ("ph", &ph), ("c", &c), ("l", &l)]))
        })
        .collect();
    formas.push(tamanho_nu((c, l)));
    formas
}

fn tamanho_em_px((pw, ph): (u32, u32)) -> String {
    tr_with(
        "panel.inspector.render_source.size_px",
        &[("pw", &pw), ("ph", &ph)],
    )
}

fn tamanho_nu((pw, ph): (u32, u32)) -> String {
    tr_with("card.insp.image_size_bare", &[("pw", &pw), ("ph", &ph)])
}

fn tamanho_junto((pw, ph): (u32, u32)) -> String {
    tr_with("card.insp.image_size_tight", &[("pw", &pw), ("ph", &ph)])
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

/// O que muda entre a moldura da imagem e a do Pixel Sprite; a lei da moldura é uma só ([`moldura`]).
struct Moldura<'a> {
    bits: u64,
    img: Option<&'a SlotImage>,
    nome: &'a str,
    /// As formas da linha do tamanho, da mais longa à mais curta: fica a primeira que cabe.
    tamanhos: Vec<String>,
    verbos: &'a [Verbo<'a>],
    /// Clicar na miniatura troca a imagem?
    trocavel: bool,
    /// Com duas ou mais folhas, a miniatura abre o dropdown delas em vez de trocar a imagem.
    folhas: Option<&'a crate::slot_sheets::SheetCard>,
    qualidade: ImageQuality,
}

/// A moldura da imagem: o único verbo é o do Painter (dono 06/11).
pub(crate) fn cabeca(c: &mut CardCtx<'_>, y: f32, info: &InspectorSpriteInfo) -> f32 {
    // Em Animated a moldura mostra a folha VISÍVEL (plano GEF 26 §3.3).
    let folhas = crate::cards_sprite_folhas::folhas(info);
    let img = folhas
        .as_ref()
        .and_then(|f| f.visible())
        .map(|f| f.image.clone())
        .or_else(|| slot_image::current_slot_image(info.entity_bits));
    let nome = nome_da_imagem(img.as_ref());
    let tamanhos = if let Some(card) = &folhas {
        crate::cards_sprite_folhas::formas_da_folha(card, |px, grade| {
            formas_com_o_corte(Some(px), grade)
        })
    } else if info.image_use == InspectorImageUse::Animated {
        formas_com_o_corte(info.source_pixels, [info.hframes, info.vframes])
    } else {
        linha_do_tamanho(info)
            .into_iter()
            .chain(
                info.source_pixels
                    .into_iter()
                    .flat_map(|px| [tamanho_em_px(px), tamanho_nu(px)]),
            )
            .collect()
    };
    let verbos = [Verbo {
        id: ids::INSP_IMAGE_EDIT_PAINTER,
        seta: IconId::Export,
        nome: tr("card.insp.verb_painter"),
    }];
    let fundo = moldura(
        c,
        y,
        &Moldura {
            bits: info.entity_bits,
            img: img.as_ref(),
            nome: &nome,
            tamanhos,
            verbos: &verbos,
            trocavel: true,
            folhas: crate::cards_sprite_folhas::varias(folhas.as_ref()),
            qualidade: ph2d_editor_core::ui_image_quality(),
        },
    );
    crate::cards_sprite_folhas::verbos_das_folhas(c, fundo, folhas.as_ref())
}

/// A moldura do Pixel Sprite: o único verbo é o do Pixel (dono 06/11); a miniatura não troca nada, e sai nítida.
pub(crate) fn cabeca_do_pixel(
    c: &mut CardCtx<'_>,
    y: f32,
    card: &PixelSpriteCard,
    nome: &str,
) -> f32 {
    let img = SlotImage {
        entity_bits: card.entity_bits,
        name: nome.to_owned(),
        rgba: card.rgba.clone(),
        w: card.thumb[0],
        h: card.thumb[1],
    };
    let nome = nome_da_imagem(Some(&img));
    let px = (card.size[0], card.size[1]);
    let verbos = [Verbo {
        id: ids::INSP_IMAGE_EDIT_PIXEL,
        seta: IconId::Export,
        nome: tr("card.insp.verb_pixel"),
    }];
    moldura(
        c,
        y,
        &Moldura {
            bits: card.entity_bits,
            img: Some(&img),
            nome: &nome,
            // O documento vai a 65535 de lado: no piso do painel, cinco dígitos só cabem juntos.
            tamanhos: vec![tamanho_em_px(px), tamanho_nu(px), tamanho_junto(px)],
            verbos: &verbos,
            trocavel: false,
            folhas: None,
            qualidade: ImageQuality::Low,
        },
    )
}

/// A superfície da miniatura. Devolve o fundo. Em qualquer largura: a miniatura não muda, e ao lado dela
/// ficam o nome, o tamanho e o verbo.
fn moldura(c: &mut CardCtx<'_>, y: f32, m: &Moldura<'_>) -> f32 {
    let theme = c.ink.theme;
    let (x0, w0) = (c.x, c.w);
    let pad = Spacing::Xs.px();
    let gap = Spacing::Sm.px();
    IMAGENS_PINTADAS.with(|v| {
        let mut v = v.borrow_mut();
        if v.len() < PINTADAS_GUARDADAS {
            v.push((m.bits, m.nome.to_owned()));
        }
    });
    let util = w0 - pad * 2.0;
    let lado = VERBO_H * LADOS;
    let alto = lado;
    let tx = x0 + pad + lado + gap;
    let tw = (util - lado - gap).max(0.0);
    let fundo = y + pad * 2.0 + alto;
    // Um degrau abaixo do cartão, e os botões um degrau acima dela (a referência do dono).
    let superficie = c.ink.color(ColorToken::CardField);
    let caixa = Rect::new(x0, y, w0, fundo - y);
    let raio = frame_radius(c.ink.theme, Radius::Sm.px());
    fill_rounded_rect(c.scene, caixa, raio, superficie);
    let topo = y + pad;
    let foto = Rect::new(x0 + pad, topo, lado, lado);
    miniatura(c, foto, m.img, m.qualidade);
    TROCA_PINTADA.with(|p| p.set(false));
    if let Some(card) = m.folhas {
        crate::cards_sprite_folhas::miniatura_escolhe(c, foto, card);
    } else if m.trocavel {
        c.hit_index.register(ids::INSP_RENDER_TEXTURE_SLOT, foto);
        troca_em_hover(c, foto);
    }
    let nome_y = topo + pad;
    paint_text_title_elided(
        c.text_system,
        c.scene,
        m.nome,
        tx,
        nome_y,
        NOME_PX,
        tw,
        resolve(ColorToken::Text1, theme),
    );
    let mut pintada = CabecaPintada {
        moldura: caixa,
        nome: Rect::new(tx, nome_y, tw, NOME_PX),
        tamanho: None,
        foto,
    };
    // Inteiro; sem os parênteses do Repeated (os campos Width/Height mostram-nos); no piso, sem a unidade.
    let pede = |c: &mut CardCtx<'_>, t: &str| c.text_system.prefix_width(t, TAMANHO_PX).ceil();
    let mut linha = None;
    for t in &m.tamanhos {
        let cabe = pede(c, t) <= tw;
        linha = Some(t.as_str());
        if cabe {
            break;
        }
    }
    if let Some(t) = linha {
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
        pintada.tamanho = Some(Rect::new(tx, ty, pede(c, t), TAMANHO_PX));
    }
    (c.x, c.w) = (tx, tw);
    c.verbos_lado_a_lado(m.verbos, topo + alto - pad - VERBO_H);
    (c.x, c.w) = (x0, w0);
    CABECA_PINTADA.with(|p| p.set(Some(pintada)));
    fundo
}

/// ⭐ Dono 05/11: em hover, a borda da miniatura acende e o ícone de troca entra a girar no canto dela.
/// O giro é o `hover_live`, o relógio da UI: com movimento reduzido ele já chega parado.
fn troca_em_hover(c: &mut CardCtx<'_>, foto: Rect) {
    let (estado, t) = c.store.button_visual(ids::INSP_RENDER_TEXTURE_SLOT);
    if !matches!(estado, ButtonState::Hovered | ButtonState::Pressed) {
        return;
    }
    let theme = c.ink.theme;
    stroke_frame(
        c.scene,
        foto,
        frame_radius(theme, Radius::Sm.px()),
        theme,
        Feel::Hovered,
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
fn miniatura(c: &mut CardCtx<'_>, r: Rect, img: Option<&SlotImage>, qualidade: ImageQuality) {
    MINIATURA_PINTADA.with(|p| *p.borrow_mut() = img.map(|i| i.rgba.clone()));
    push_rounded_clip(c.scene, r, frame_radius(c.ink.theme, Radius::Sm.px()));
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
            qualidade,
        );
    }
    c.scene.pop_layer();
}

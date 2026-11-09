//! ⭐ **As folhas de um Animated Sprite no cartão Image** (plano GEF 26 §3.3): o dropdown da miniatura (com duas ou
//! mais), **Load sheet** · **Delete sheet** sob a moldura, e no cartão Animation a fila que limpa os quadros vazios.
//! As folhas chegam por [`crate::slot_sheets`]; o painel só pede `ImageUseEdit`.

use crate::ids;
use crate::slot_image::{self, SlotImage};
use crate::slot_sheets::{SheetCard, current_sheets};
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{HitIndex, InteractiveState, WidgetEvent, WidgetStore};
use ph2d_editor_core::paint::{
    fill_rounded_rect, frame_radius, paint_icon, paint_text_elided, paint_text_title_elided,
    push_rounded_clip, resolve,
};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::panel::cards::verbos::{VERBO_H, Verbo};
use ph2d_editor_core::panel::cards::{CardCtx, Cell, Widget};
use ph2d_editor_core::screens::hero::{InspectorAnimInfo, InspectorAnimRow, InspectorSpriteInfo};
use ph2d_editor_core::widget::ButtonState;
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{IconId, ImageUseEdit, InspectorImageUse};
use ph2d_i18n::{tr, tr_with};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::{ImageQuality, VectorScene};
use std::cell::{Cell as Celula, RefCell};

/// A miniatura de um item do dropdown mede dois botões: a folha inteira lê-se, e cinco itens cabem num ecrã baixo.
const LADOS_DO_ITEM: f32 = 2.0; // LITERAL-PX-OK: conta de botões, não geometria
/// O dropdown tem pelo menos a largura de três miniaturas do cartão (a maquete 26, painel 2).
const LARGURA_EM_FOTOS: f32 = 3.0; // LITERAL-PX-OK: conta de miniaturas, não geometria

thread_local! {
    /// O objecto cujo dropdown das folhas está aberto.
    static ABERTO: Celula<Option<u64>> = const { Celula::new(None) };
    /// A miniatura e as folhas do dropdown a pintar por cima de tudo neste quadro.
    static PENDENTE: RefCell<Option<(Rect, SheetCard)>> = const { RefCell::new(None) };
    /// As linhas do cartão Animation pintadas em alerta neste quadro (limpas no `prepara`).
    static ALERTAS: RefCell<Vec<ph2d_a11y::NodeId>> = const { RefCell::new(Vec::new()) };
}

/// A linha `row_id` pintou em alerta.
pub(crate) fn marca_alerta(row_id: ph2d_a11y::NodeId) {
    ALERTAS.with(|a| a.borrow_mut().push(row_id));
}

/// As linhas do cartão Animation em alerta no último quadro. Para o teste.
#[doc(hidden)]
#[must_use]
pub fn linhas_em_alerta() -> Vec<ph2d_a11y::NodeId> {
    ALERTAS.with(|a| a.borrow().clone())
}

/// O dropdown de `bits` está aberto? Para o teste.
#[doc(hidden)]
#[must_use]
pub fn dropdown_aberto(bits: u64) -> bool {
    ABERTO.with(Celula::get) == Some(bits)
}

/// As folhas de `info` quando ele é um Animated com imagem.
pub(crate) fn folhas(info: &InspectorSpriteInfo) -> Option<SheetCard> {
    (info.image_use == InspectorImageUse::Animated)
        .then(|| current_sheets(info.entity_bits))
        .flatten()
}

/// Com duas ou mais folhas a miniatura escolhe a folha, nunca troca a imagem.
pub(crate) fn varias(card: Option<&SheetCard>) -> Option<&SheetCard> {
    card.filter(|c| c.sheets.len() >= 2)
}

/// As formas da linha do tamanho da folha visível, da mais longa à mais curta: com duas ou mais, a mais longa diz
/// também «sheet i of n».
pub(crate) fn formas_da_folha(
    card: &SheetCard,
    sem_folha: impl Fn((u32, u32), [u32; 2]) -> Vec<String>,
) -> Vec<String> {
    let Some(f) = card.visible() else {
        return Vec::new();
    };
    let formas = sem_folha(f.px, f.grid);
    if card.sheets.len() < 2 {
        return formas;
    }
    let com = formas.first().map(|base| {
        tr_with(
            "card.insp.image_sheet_of",
            &[
                ("size", base),
                ("i", &(usize::from(card.shown) + 1)),
                ("n", &card.sheets.len()),
            ],
        )
    });
    com.into_iter().chain(formas).collect()
}

/// A moldura, com duas ou mais folhas: a miniatura é a caixa do dropdown, com uma faixa translúcida no pé e a ▾ ao
/// centro (dono 08/11: o quadradinho no canto lia-se como uma caixa de marcar).
pub(crate) fn miniatura_escolhe(c: &mut CardCtx<'_>, foto: Rect, card: &SheetCard) {
    c.hit_index.register(ids::INSP_IMAGE_SHEET_PICK, foto);
    let theme = c.ink.theme;
    let lado = m::HEAD_ICON_GLYPH;
    let inset = Spacing::Xxs.px();
    let alto = lado + inset;
    let faixa = Rect::new(foto.x, foto.y + foto.h - alto, foto.w, alto);
    fill_rounded_rect(
        c.scene,
        faixa,
        frame_radius(theme, Radius::Sm.px()),
        c.ink.color(ColorToken::RailBg),
    );
    let seta = Rect::new(
        faixa.x + (faixa.w - lado) * 0.5,
        faixa.y + inset * 0.5,
        lado,
        lado,
    );
    paint_icon(
        c.scene,
        IconId::ChevronDown,
        seta,
        resolve(ColorToken::Text1, theme),
        StrokeToken::Default.px(),
    );
    if dropdown_aberto(card.entity_bits) {
        PENDENTE.with(|p| *p.borrow_mut() = Some((foto, card.clone())));
    }
}

/// **+ Load sheet** sempre em Animated; **Delete sheet** só com duas ou mais (apagar a última é o convite do vazio).
pub(crate) fn verbos_das_folhas(c: &mut CardCtx<'_>, y: f32, card: Option<&SheetCard>) -> f32 {
    let Some(card) = card else {
        return y;
    };
    let mut verbos = vec![Verbo {
        id: ids::INSP_IMAGE_SHEET_LOAD,
        seta: IconId::Plus,
        nome: tr("card.insp.sheet_load"),
    }];
    if card.sheets.len() >= 2 {
        verbos.push(Verbo {
            id: ids::INSP_IMAGE_SHEET_DELETE,
            seta: IconId::Trash,
            nome: tr("card.insp.sheet_delete"),
        });
    }
    let topo = y + m::FIELD_ROW_GAP;
    c.verbos_lado_a_lado(&verbos, topo);
    topo + VERBO_H
}

/// Quantos quadros vazios (de uma folha apagada) a animação tem.
#[must_use]
pub(crate) fn vazios(row: &InspectorAnimRow) -> usize {
    row.frames.iter().filter(|f| f.is_empty()).count()
}

/// A animação é TODA vazia (nenhum quadro de verdade)?
fn toda_vazia(row: &InspectorAnimRow) -> bool {
    !row.frames.is_empty() && row.frames.iter().all(|f| f.is_empty())
}

/// No cartão Animation, com vazios em alguma animação: **Remove empty frames**, e **Delete empty animations** só
/// quando alguma é toda vazia.
pub(crate) fn limpeza(c: &mut CardCtx<'_>, y: f32, info: &InspectorAnimInfo) -> f32 {
    if !info.rows.iter().any(|r| vazios(r) > 0) {
        return y;
    }
    let mut celulas = vec![Cell::new(Widget::Button {
        id: ids::INSP_ANIM_REMOVE_EMPTY_FRAMES,
        text: tr("panel.inspector.anim_card.remove_empty_frames"),
        primary: false,
    })];
    if info.rows.iter().any(toda_vazia) {
        celulas.push(Cell::new(Widget::Button {
            id: ids::INSP_ANIM_DELETE_EMPTY_ANIMS,
            text: tr("panel.inspector.anim_card.delete_empty_anims"),
            primary: false,
        }));
    }
    c.row(&celulas, y + m::FIELD_ROW_GAP)
}

fn botao(store: &mut WidgetStore, id: ph2d_a11y::NodeId, dica: Option<&str>) {
    store.register_if_absent(
        id,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
    if let Some(dica) = dica
        && store.tooltip_for(id).is_none()
    {
        store.set_tooltip(id, tr(dica));
    }
}

/// Os botões das folhas: uma vez por quadro, antes da pintura.
pub(crate) fn prepara(store: &mut WidgetStore) {
    botao(
        store,
        ids::INSP_IMAGE_SHEET_PICK,
        Some("card.insp.sheet_pick_tip"),
    );
    botao(
        store,
        ids::INSP_IMAGE_SHEET_LOAD,
        Some("card.insp.sheet_load_tip"),
    );
    botao(
        store,
        ids::INSP_IMAGE_SHEET_DELETE,
        Some("card.insp.sheet_delete_tip"),
    );
    botao(store, ids::INSP_IMAGE_SHEET_SCRIM, None);
    for id in ids::INSP_IMAGE_SHEET_ITEM {
        botao(store, id, None);
    }
    botao(
        store,
        ids::INSP_ANIM_REMOVE_EMPTY_FRAMES,
        Some("panel.inspector.anim_card.remove_empty_frames_tip"),
    );
    botao(
        store,
        ids::INSP_ANIM_DELETE_EMPTY_ANIMS,
        Some("panel.inspector.anim_card.delete_empty_anims_tip"),
    );
    botao(
        store,
        ids::INSP_ANIM_PREVIEW_SYNC,
        Some("panel.inspector.anim_card.preview_sync_tip"),
    );
    ALERTAS.with(|a| a.borrow_mut().clear());
    // O dropdown de um objecto que saiu de vista fecha-se.
    let selecionada = crate::state::current_inspector_sprite().map(|s| s.entity_bits);
    if ABERTO
        .with(Celula::get)
        .is_some_and(|b| Some(b) != selecionada)
    {
        ABERTO.with(|a| a.set(None));
    }
}

/// Os cliques das folhas. `true` = consumido. Qualquer outro clique fecha o dropdown e segue.
pub(crate) fn evento(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = ev else {
        return false;
    };
    let aberto = ABERTO.with(|a| a.replace(None));
    let bits = crate::state::current_inspector_sprite().map(|s| s.entity_bits);
    let Some(bits) = bits else {
        return false;
    };
    let pede = |host: &mut dyn PanelHostInternal, edit| {
        host.bus_mut().push(EditorAction::ImageUseEdit {
            entity_bits: bits,
            edit,
        });
        true
    };
    if id == ids::INSP_IMAGE_SHEET_PICK {
        if aberto != Some(bits) {
            ABERTO.with(|a| a.set(Some(bits)));
        }
        return true;
    }
    if id == ids::INSP_IMAGE_SHEET_SCRIM {
        return true;
    }
    if let Some(k) = ids::INSP_IMAGE_SHEET_ITEM.iter().position(|&i| i == id) {
        return match u8::try_from(k) {
            Ok(k) => pede(host, ImageUseEdit::ShowSheet(k)),
            Err(_) => true,
        };
    }
    let edit = if id == ids::INSP_IMAGE_SHEET_LOAD {
        ImageUseEdit::LoadSheet
    } else if id == ids::INSP_IMAGE_SHEET_DELETE {
        ImageUseEdit::DeleteSheet
    } else if id == ids::INSP_ANIM_REMOVE_EMPTY_FRAMES {
        ImageUseEdit::RemoveEmptyFrames
    } else if id == ids::INSP_ANIM_DELETE_EMPTY_ANIMS {
        ImageUseEdit::DeleteEmptyAnimations
    } else {
        return false;
    };
    pede(host, edit)
}

/// ⭐ O dropdown aberto, por cima de tudo (o fecho do corpo): um item por folha, a moldura com a imagem inteira,
/// o nome e `C × R`; a visível marcada com ✓. Um véu na região toda: o clique fora fecha sem chegar a ninguém.
pub(crate) fn paint_deferred(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    region: Rect,
) {
    let Some((foto, card)) = PENDENTE.with(|p| p.borrow_mut().take()) else {
        return;
    };
    hit_index.register(ids::INSP_IMAGE_SHEET_SCRIM, region);
    let pad = m::SEG_PAD;
    let item_h = VERBO_H * LADOS_DO_ITEM + pad * 2.0;
    let n = card.sheets.len().min(ids::INSP_IMAGE_SHEET_ITEM.len());
    let largura = (foto.w * LARGURA_EM_FOTOS).min(region.w);
    let alto = (item_h * n as f32 + pad * 2.0).min(region.h);
    let caixa = Rect::new(
        foto.x.min(region.x + region.w - largura).max(region.x),
        foto.y.min(region.y + region.h - alto).max(region.y),
        largura,
        alto,
    );
    ph2d_editor_core::paint_flight::shadow(
        scene,
        caixa,
        m::FIELD_RADIUS,
        resolve(ColorToken::KanbanListShadow, theme),
        m::LIST_SHADOW_FAR,
    );
    fill_rounded_rect(
        scene,
        caixa,
        m::FIELD_RADIUS,
        resolve(ColorToken::KanbanCardRaised, theme),
    );
    let nome_px = TypeToken::Base.px();
    let grade_px = TypeToken::Sm.px();
    for (k, (face, &id)) in card
        .sheets
        .iter()
        .zip(ids::INSP_IMAGE_SHEET_ITEM.iter())
        .enumerate()
        .take(n)
    {
        let lr = Rect::new(
            caixa.x + pad,
            caixa.y + pad + item_h * k as f32,
            caixa.w - pad * 2.0,
            item_h,
        );
        if lr.y + lr.h > caixa.y + caixa.h {
            break;
        }
        hit_index.register(id, lr);
        let mostrada = k == usize::from(card.shown);
        if mostrada || store.hover_live(id) > 0.0 {
            let fundo = if mostrada {
                ColorToken::Selection
            } else {
                ColorToken::CardFieldHover
            };
            fill_rounded_rect(scene, lr, m::SEG_RADIUS, resolve(fundo, theme));
        }
        let lado = lr.h - pad * 2.0;
        let quadro = Rect::new(lr.x + pad, lr.y + pad, lado, lado);
        moldura_inteira(scene, theme, quadro, &face.image);
        let tx = quadro.x + quadro.w + Spacing::Sm.px();
        let tw = (lr.x + lr.w - tx - pad).max(0.0);
        let nome = if face.image.name.is_empty() {
            tr("card.insp.image_unnamed").to_owned()
        } else {
            face.image.name.clone()
        };
        let ny = quadro.y + Spacing::Xs.px();
        let tw_nome = if mostrada {
            (tw - nome_px).max(0.0)
        } else {
            tw
        };
        paint_text_title_elided(
            text_system,
            scene,
            &nome,
            tx,
            ny,
            nome_px,
            tw_nome,
            resolve(ColorToken::Text1, theme),
        );
        if mostrada {
            let nw = text_system.prefix_width(&nome, nome_px).min(tw_nome);
            paint_icon(
                scene,
                IconId::Check,
                Rect::new(tx + nw + Spacing::Xxs.px(), ny, nome_px, nome_px),
                resolve(ColorToken::Text1, theme),
                StrokeToken::Default.px(),
            );
        }
        paint_text_elided(
            text_system,
            scene,
            &tr_with(
                "card.insp.image_size_bare",
                &[("pw", &face.grid[0]), ("ph", &face.grid[1])],
            ),
            tx,
            ny + nome_px + m::FIELD_ROW_GAP,
            grade_px,
            tw,
            resolve(ColorToken::CardLabel, theme),
        );
    }
}

/// A imagem inteira da folha, com o aspecto dela, numa moldura escura.
fn moldura_inteira(scene: &mut VectorScene, theme: Theme, r: Rect, img: &SlotImage) {
    push_rounded_clip(scene, r, frame_radius(theme, Radius::Sm.px()));
    scene.fill_rect(
        ph2d_editor_core::paint::rect_to_vello(r),
        resolve(ColorToken::KanbanList, theme),
    );
    if let Some(s) = slot_image::stable(img) {
        let (tw, th) = (img.w.max(1) as f32, img.h.max(1) as f32);
        let k = (r.w / tw).min(r.h / th);
        let (dw, dh) = (tw * k, th * k);
        let x0 = f64::from(r.x + (r.w - dw) * 0.5);
        let y0 = f64::from(r.y + (r.h - dh) * 0.5);
        scene.draw_stable_image(
            &s,
            (x0, y0, x0 + f64::from(dw), y0 + f64::from(dh)),
            ImageQuality::Medium,
        );
    }
    scene.pop_layer();
}

//! **A biblioteca no navegador de presets** (23/10): a fileira «Preset library» (Import… · Export… ·
//! Delete · Reset positions, cada um só quando serve), a confirmação do Delete e o ARRASTO de um pincel
//! dentro do popover aberto (a família `PainterPreset` da máquina de linhas do despacho; a queda vai à
//! ponte como `PAINTER_WETPAINT_PRESET_MOVE`, que reordena o `estado.json`).
//!
//! O Delete abre uma JANELA DE CONFIRMAÇÃO (dono 23/10: «melhor um modal») — o [`Modal`] da fundação, com
//! o nome do pincel no título; só o «Delete» dela chega ao tool. Cancel, o X e o clique fora fecham-na.
//! Sem desfazer: o ficheiro é do artista e a janela diz o tamanho do estrago.

use crate::paint_wetpaint_presets::{escolher_preset, preset_escolhido};
use crate::state_wet_presets::{VistaPresets, current_wet_presets};
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{PanelRowDrop, PanelRowFamily, WidgetEvent};
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text_block, resolve};
use ph2d_editor_core::panel::{
    PaintCtx, PanelHostInternal, paint_dropdown_list, visible_option_bands,
};
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::widget::{Button, Dropdown, DropdownOption, Modal, paint_modal};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_tokens::{ColorToken, Radius};
use ph2d_tool_painter::ids;
use std::cell::RefCell;

thread_local! {
    /// O pincel cuja janela de confirmação do Delete está aberta.
    static ARMADO: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// A fileira «Preset library»; devolve o `y` seguinte.
pub(crate) fn paint_biblioteca(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    v: &VistaPresets,
) -> f32 {
    let ativo = v.ativo.as_deref().and_then(|r| v.procurar(r));
    let mut botoes = vec![(
        ids::PAINTER_WETPAINT_PRESET_IMPORT,
        tr("panel.painter_layers.wetpaint.preset_import").to_owned(),
    )];
    if let Some(a) = ativo {
        botoes.push((
            ids::PAINTER_WETPAINT_PRESET_EXPORT,
            tr("panel.painter_layers.wetpaint.preset_export").to_owned(),
        ));
        if a.apagavel {
            botoes.push((
                ids::PAINTER_WETPAINT_PRESET_DELETE,
                tr("panel.painter_layers.wetpaint.preset_delete").to_owned(),
            ));
        }
    }
    if v.ordem_manual {
        botoes.push((
            ids::PAINTER_WETPAINT_PRESET_RESET_ORDER,
            tr("panel.painter_layers.wetpaint.preset_reset_order").to_owned(),
        ));
    }
    crate::paint_seg_row::seg_row_owned(
        ctx,
        theme,
        x,
        content_w,
        y,
        ids::PAINTER_WETPAINT_PRESET_IMPORT,
        tr("panel.painter_layers.wetpaint.preset_library"),
        &botoes,
        usize::MAX,
    )
}

/// O popover do Preset aberto: pinta, publica as linhas arrastáveis (as faixas VISÍVEIS, as mesmas que
/// recebem o clique) e, durante um arrasto, a barra onde o pincel vai cair — a partição 50/50 do
/// despacho para uma família que não aninha.
pub(crate) fn paint_popover(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    opcoes: Vec<DropdownOption<usize>>,
    chip: Rect,
    atual: usize,
) {
    let dd = Dropdown::new(ids::PAINTER_WETPAINT_PRESET, "", opcoes)
        .selected(atual)
        .open(true);
    let viewport = ctx.viewport;
    let painted = paint_dropdown_list(ctx, &dd, chip, viewport, theme, None);
    let bandas: Vec<_> = visible_option_bands(&dd, chip, painted.panel, painted.scroll).collect();
    let store = ctx.host.store_mut();
    store.set_panel_row_ids(
        PanelRowFamily::PainterPreset,
        bandas.iter().map(|(id, _)| *id).collect(),
    );
    let Some((PanelRowFamily::PainterPreset, d)) = store.panel_row_drag() else {
        return;
    };
    if !d.active {
        return;
    }
    let alvo = bandas
        .iter()
        .find(|(id, r)| *id != d.dragged && d.cursor_y >= r.y && d.cursor_y < r.y + r.h);
    if let Some((_, r)) = alvo {
        let y = if d.cursor_y < r.y + r.h * 0.5 {
            r.y
        } else {
            r.y + r.h
        };
        let h = crate::paint_rows::DROP_BAR_H;
        fill_rounded_rect(
            ctx.scene,
            Rect::new(r.x, y - h * 0.5, r.w, h),
            ph2d_editor_core::paint::frame_radius(theme, Radius::Sm.px()),
            resolve(ColorToken::Accent, theme),
        );
    }
}

/// O popover não se pintou neste quadro: nenhuma linha dele fica arrastável.
pub(crate) fn sem_popover(ctx: &mut PaintCtx) {
    ctx.host
        .store_mut()
        .set_panel_row_ids(PanelRowFamily::PainterPreset, Default::default());
}

const DIALOGO_W: f32 = 360.0; // LITERAL-PX-OK: the confirm dialog's card width (fits title + two buttons)
const DIALOGO_H: f32 = 168.0; // LITERAL-PX-OK: header + one body line + footer of the Modal

/// [confirm, cancel, close, card, scrim] — os ids da janela, pela ordem de [`ids::PAINTER_WETPAINT_PRESET_DELETE_DIALOG`].
const CONFIRMA: ph2d_a11y::NodeId = ids::PAINTER_WETPAINT_PRESET_DELETE_DIALOG[0];
const CANCELA: ph2d_a11y::NodeId = ids::PAINTER_WETPAINT_PRESET_DELETE_DIALOG[1];
const FECHA: ph2d_a11y::NodeId = ids::PAINTER_WETPAINT_PRESET_DELETE_DIALOG[2];
const CARTAO: ph2d_a11y::NodeId = ids::PAINTER_WETPAINT_PRESET_DELETE_DIALOG[3];
const FORA: ph2d_a11y::NodeId = ids::PAINTER_WETPAINT_PRESET_DELETE_DIALOG[4];

/// A janela de confirmação do Delete, por cima de tudo (o passe dos popovers), enquanto está aberta.
pub(crate) fn paint_dialogo(ctx: &mut PaintCtx, theme: ph2d_tokens::Theme) {
    let Some(rel) = ARMADO.with(|c| c.borrow().clone()) else {
        return;
    };
    let Some(nome) = current_wet_presets().and_then(|v| v.procurar(&rel).map(|e| e.nome.clone()))
    else {
        desarmar();
        return;
    };
    let vp = ctx.viewport;
    let w = DIALOGO_W.min(vp.w);
    let card = Rect::new(
        vp.x + (vp.w - w) * 0.5,
        vp.y + (vp.h - DIALOGO_H) * 0.5,
        w,
        DIALOGO_H,
    );
    let modal = Modal::new(
        FECHA,
        tr_with(
            "panel.painter_layers.wetpaint.preset_delete_confirm",
            &[("name", &nome)],
        ),
        Button::new(CANCELA, tr("panel.painter_layers.wetpaint.preset_cancel")),
        Button::new(CONFIRMA, tr("panel.painter_layers.wetpaint.preset_delete")).danger(),
    );
    paint_modal(&modal, vp, card, ctx.scene, ctx.text_system, theme);
    let corpo = modal.body_rect(card);
    let _ = paint_text_block(
        ctx.text_system,
        ctx.scene,
        tr("panel.painter_layers.wetpaint.preset_delete_body"),
        corpo.x,
        corpo.y,
        ph2d_tokens::TypeToken::Base.px(),
        corpo.w,
        resolve(ColorToken::Text2, theme),
    );
    let (cancel, confirm) = modal.button_rects(card);
    // Por ordem: o último registado ganha o clique.
    let alvos = [
        (FORA, vp),
        (CARTAO, card),
        (FECHA, modal.close_rect(card)),
        (CANCELA, cancel),
        (CONFIRMA, confirm),
    ];
    let (_, hit) = ctx.host.store_and_hit_index_mut();
    for (id, r) in alvos {
        hit.register(id, r);
    }
}

/// Os eventos da biblioteca (antes do resto do painel). `None` = não é daqui.
pub(crate) fn try_event(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> Option<bool> {
    match ev {
        WidgetEvent::Click(id) if id == ids::PAINTER_WETPAINT_PRESET_DELETE => {
            let ativo = current_wet_presets().and_then(|v| v.ativo.clone());
            ARMADO.with(|c| *c.borrow_mut() = ativo);
            Some(true)
        }
        WidgetEvent::Click(id) if id == CONFIRMA => {
            confirmar_apagar(host);
            Some(true)
        }
        WidgetEvent::Click(id) if [CANCELA, FECHA, FORA].contains(&id) => {
            desarmar();
            Some(true)
        }
        WidgetEvent::Click(id) if id == CARTAO => Some(true),
        // Um preset do navegador (o valor é o caminho na biblioteca, não um u8).
        WidgetEvent::Click(id) if preset_escolhido(id).is_some() => {
            desarmar();
            escolher_preset(host, preset_escolhido(id).unwrap_or_default());
            Some(true)
        }
        WidgetEvent::PanelRowReparent {
            family: PanelRowFamily::PainterPreset,
            dragged,
            drop,
        } => {
            largar(host, dragged, drop);
            Some(true)
        }
        WidgetEvent::Click(_) => {
            desarmar();
            None
        }
        _ => None,
    }
}

fn desarmar() {
    ARMADO.with(|c| c.borrow_mut().take());
}

/// O «Delete» da janela: só apaga se o pincel dela ainda for o activo (o artista não trocou por baixo).
fn confirmar_apagar(host: &mut dyn PanelHostInternal) {
    let aberto = ARMADO.with(|c| c.borrow_mut().take());
    let ativo = current_wet_presets().and_then(|v| v.ativo.clone());
    if aberto.is_some() && aberto == ativo {
        host.bus_mut()
            .push(EditorAction::ToolPanelEvent(PanelEvent::Click(
                ids::PAINTER_WETPAINT_PRESET_DELETE,
            )));
    }
}

fn largar(host: &mut dyn PanelHostInternal, dragged: ph2d_a11y::NodeId, drop: PanelRowDrop) {
    let (alvo, lado) = match drop {
        PanelRowDrop::Before(id) | PanelRowDrop::Inside(id) => (id, "before"),
        PanelRowDrop::After(id) => (id, "after"),
        PanelRowDrop::End => return,
    };
    let (Some(a), Some(b)) = (preset_escolhido(dragged), preset_escolhido(alvo)) else {
        return;
    };
    let sep = ids::PRESET_MOVE_SEP;
    host.bus_mut()
        .push(EditorAction::ToolPanelEvent(PanelEvent::SelectOption(
            ids::PAINTER_WETPAINT_PRESET_MOVE,
            format!("{a}{sep}{b}{sep}{lado}"),
        )));
}

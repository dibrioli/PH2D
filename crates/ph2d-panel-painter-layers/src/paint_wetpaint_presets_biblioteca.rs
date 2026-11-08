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
use ph2d_editor_core::paint::{paint_text_block, resolve};
use ph2d_editor_core::panel::{
    DropdownRowInHand, PaintCtx, PanelHostInternal, camada_das_listas, dropdown_list_bands,
    paint_dropdown_list, paint_dropdown_list_reordering, visible_option_bands,
};
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::widget::{
    Button, Dropdown, DropdownOption, Modal, PaintedPopover, paint_modal,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_tokens::ColorToken;
use ph2d_tool_painter::ids;
use std::cell::RefCell;

thread_local! {
    /// O pincel cuja janela de confirmação do Delete está aberta.
    static ARMADO: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// Save as New · Save as Default · Reset. Sem preset activo não há o que guardar por cima nem o que repor.
pub(crate) fn botoes_de_guardar(v: &VistaPresets) -> Vec<(ph2d_a11y::NodeId, String)> {
    let mut botoes = vec![(
        ids::PAINTER_WETPAINT_PRESET_SAVE_NEW,
        tr("panel.painter_layers.wetpaint.preset_save_new").to_owned(),
    )];
    if v.ativo.as_deref().and_then(|r| v.procurar(r)).is_some() {
        botoes.push((
            ids::PAINTER_WETPAINT_PRESET_SAVE_DEFAULT,
            tr("panel.painter_layers.wetpaint.preset_save_default").to_owned(),
        ));
        botoes.push((
            ids::PAINTER_WETPAINT_PRESET_RESET,
            tr("panel.painter_layers.wetpaint.preset_reset").to_owned(),
        ));
    }
    botoes
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
    crate::paint_seg_row::seg_row_owned(
        ctx,
        theme,
        x,
        content_w,
        y,
        ids::PAINTER_WETPAINT_PRESET_IMPORT,
        tr("panel.painter_layers.wetpaint.preset_library"),
        &botoes_da_biblioteca(v),
        usize::MAX,
    )
}

/// Import · Export · Delete · Reset positions, cada um só quando serve.
pub(crate) fn botoes_da_biblioteca(v: &VistaPresets) -> Vec<(ph2d_a11y::NodeId, String)> {
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
    botoes
}

/// O popover do Preset aberto: pinta e publica as linhas arrastáveis (as faixas VISÍVEIS, as mesmas que
/// recebem o clique). Durante um arrasto, a lei do cartão do Kanban: a linha voa, as outras abrem o vão
/// onde ela vai cair e a marca de acento diz o sítio; os alvos ficam os de agora.
pub(crate) fn paint_popover(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    opcoes: Vec<DropdownOption<usize>>,
    chip: Rect,
    atual: usize,
    arrasta: bool,
) -> PaintedPopover {
    let dd = Dropdown::new(ids::PAINTER_WETPAINT_PRESET, "", opcoes)
        .selected(atual)
        .open(true);
    let viewport = ctx.viewport;
    let na_mao = match ctx.host.store().panel_row_drag() {
        Some((PanelRowFamily::PainterPreset, d)) if d.active => Some(d),
        _ => None,
    };
    let previsto = na_mao.and_then(|d| {
        let bandas = dropdown_list_bands(ctx, &dd, chip, viewport);
        let alvo = bandas
            .iter()
            .find(|(id, r)| *id != d.dragged && d.cursor_y >= r.y && d.cursor_y < r.y + r.h)
            .map(|(id, r)| (*id, d.cursor_y >= r.y + r.h * 0.5));
        let voa = dd.options.iter().position(|o| o.id == d.dragged)?;
        let (origem, largura, altura) = {
            let r = bandas.iter().find(|(id, _)| *id == d.dragged);
            r.map_or((chip.x, chip.w, chip.h), |(_, r)| (r.x, r.w, chip.h))
        };
        let (linhas, vao, cai) = ordem_prevista(&dd.options, voa, alvo);
        let at = Rect::new(
            origem + d.cursor_x - d.down_x,
            d.cursor_y - altura * 0.5,
            largura,
            altura,
        );
        Some((linhas, vao, cai, voa, at))
    });
    let painted = match previsto {
        Some((linhas, gap, lands, flying, at)) => {
            let preview = Dropdown::new(ids::PAINTER_WETPAINT_PRESET, "", linhas)
                .selected(atual)
                .open(true);
            let mao = DropdownRowInHand {
                preview: &preview,
                gap,
                lands,
                flying,
                at,
            };
            paint_dropdown_list_reordering(ctx, &dd, &mao, chip, viewport, theme)
        }
        None => paint_dropdown_list(ctx, &dd, chip, viewport, theme, None),
    };
    // Com a busca a filtrar, nada se arrasta: a ordem é a da lista inteira.
    let linhas = if arrasta {
        visible_option_bands(&dd, chip, painted.panel, painted.scroll)
            .map(|(id, _)| id)
            .collect()
    } else {
        Default::default()
    };
    ctx.host
        .store_mut()
        .set_panel_row_ids(PanelRowFamily::PainterPreset, linhas);
    painted
}

/// O valor do vão: nenhum preset o tem, nem o «sem preset» do popover (`usize::MAX`).
const VALOR_DO_VAO: usize = usize::MAX - 1;

/// As linhas com `voa` a sair e o vão onde ela cai. `alvo` é a linha sob o cursor e se a queda é depois
/// dela. Devolve as linhas, o índice do vão e se a queda muda alguma coisa: fora da secção de `voa` (os
/// títulos e o aviso separam-nas) a ponte recusa, e então o vão fica no sítio dela, sem marca.
fn ordem_prevista(
    opcoes: &[DropdownOption<usize>],
    voa: usize,
    alvo: Option<(ph2d_a11y::NodeId, bool)>,
) -> (Vec<DropdownOption<usize>>, usize, bool) {
    let separa = |o: &DropdownOption<usize>| o.header || o.disabled;
    let secao = |i: usize| opcoes[..i].iter().filter(|o| separa(o)).count();
    let mut linhas = opcoes.to_vec();
    linhas[voa] = DropdownOption::new(opcoes[voa].id, VALOR_DO_VAO, String::new());
    let lugar = alvo.and_then(|(id, depois)| {
        let t = opcoes.iter().position(|o| o.id == id)?;
        (secao(t) == secao(voa)).then_some((t, depois))
    });
    let Some((t, depois)) = lugar else {
        return (linhas, voa, false);
    };
    let vao = linhas.remove(voa);
    // O índice do alvo depois de `voa` sair, e a seguir a ele se a queda é depois.
    let i = if t > voa { t - 1 } else { t } + usize::from(depois);
    linhas.insert(i, vao);
    (linhas, i, true)
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
    let corpo = modal.body_rect(card);
    let texto = &mut *ctx.text_system;
    camada_das_listas::por_cima(ctx.scene, |cena| {
        paint_modal(&modal, vp, card, cena, texto, theme);
        let _ = paint_text_block(
            texto,
            cena,
            tr("panel.painter_layers.wetpaint.preset_delete_body"),
            corpo.x,
            corpo.y,
            ph2d_tokens::TypeToken::Base.px(),
            corpo.w,
            resolve(ColorToken::Text2, theme),
        );
    });
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
        camada_das_listas::alvo(hit, id, r);
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

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_a11y::NodeId;

    /// Título · A B C D · título · E: os ids são 10 + o índice.
    fn lista() -> Vec<DropdownOption<usize>> {
        (0..7)
            .map(|i| {
                let o = DropdownOption::new(NodeId(10 + i as u64), i, format!("{i}"));
                if i == 0 || i == 5 { o.header(true) } else { o }
            })
            .collect()
    }

    /// As linhas pela ordem prevista: o rótulo de cada uma, o vão vazio.
    fn ordem(voa: usize, alvo: Option<(u64, bool)>) -> (Vec<String>, usize, bool) {
        let alvo = alvo.map(|(i, d)| (NodeId(10 + i), d));
        let (l, vao, cai) = ordem_prevista(&lista(), voa, alvo);
        (l.into_iter().map(|o| o.label).collect(), vao, cai)
    }

    #[test]
    fn the_gap_opens_where_the_row_will_land_inside_its_section() {
        let v = |s: &[&str]| s.iter().map(|x| (*x).to_owned()).collect::<Vec<_>>();
        // Para baixo, antes e depois do alvo; para cima, antes e depois do alvo.
        let casos = [
            ((1, (3, false)), (["0", "2", "", "3", "4", "5", "6"], 2)),
            ((1, (3, true)), (["0", "2", "3", "", "4", "5", "6"], 3)),
            ((4, (2, false)), (["0", "1", "", "2", "3", "5", "6"], 2)),
            ((4, (2, true)), (["0", "1", "2", "", "3", "5", "6"], 3)),
        ];
        for ((voa, alvo), (linhas, vao)) in casos {
            assert_eq!(
                ordem(voa, Some(alvo)),
                (v(&linhas), vao, true),
                "{voa} {alvo:?}"
            );
        }
    }

    /// Sem alvo, ou noutra secção: a ordem é a de agora, com o vão no sítio dela.
    #[test]
    fn without_a_place_to_land_the_gap_stays_home() {
        let parada = ["0", "", "2", "3", "4", "5", "6"]
            .map(str::to_owned)
            .to_vec();
        assert_eq!(ordem(1, None), (parada.clone(), 1, false));
        assert_eq!(ordem(1, Some((6, false))), (parada, 1, false));
    }
}

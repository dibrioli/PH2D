//! **O ponto fixo do spray** (dono 08/11): a caixa «Fixed dab size» e, ligada, o «Dab size (px)». Moram no Jitter,
//! logo depois do Count e do Scatter de que dependem (ordem do dono 08/11), nos dois painéis e em todo meio que lê a
//! Forma. Sempre à vista; num pincel que não espalha (Count 1, Scatter 0) a inércia `PontoFixoSemEspalhar` esmaece-os
//! e a dica dela toma o lugar da explicação; senão a dica de hover explica o que fazem.

use crate::cards_linhas::{Item, numero};
use crate::paint_wetpaint::Numero;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::property_row::Seccao;
use ph2d_i18n::tr;
use ph2d_tool_painter::ids;
use ph2d_tool_painter::{BrushSettings, LinhaDaFerramenta};

/// O rótulo da caixa.
pub(crate) const CAIXA: &str = "panel.painter_layers.stroke.jitter.fixed_dab";
/// O rótulo do campo.
pub(crate) const CAMPO: &str = "panel.painter_layers.stroke.jitter.dab_px";
/// A explicação de cada controlo, ao passar o rato (dono 08/11: sempre, não só esmaecido).
pub(crate) const DICAS: [(ph2d_a11y::NodeId, &str); 2] = [
    (
        ids::PAINTER_BRUSH_DAB_FIXO,
        "panel.painter_layers.stroke.jitter.fixed_dab_tip",
    ),
    (
        ids::PAINTER_BRUSH_DAB_PX,
        "panel.painter_layers.stroke.jitter.dab_px_tip",
    ),
];

/// O pincel lê o ponto fixo: o mesmo filtro que o das fileiras da Forma.
pub(crate) fn presente(b: &BrushSettings) -> bool {
    b.le(LinhaDaFerramenta::Forma)
}

/// A dica de inércia, ou (sem ela) a explicação. O corpo de um cartão não escreve no store: lá isto corre no `prepare`.
pub(crate) fn poe_dicas(store: &mut WidgetStore) {
    for (id, dica) in DICAS {
        if crate::esmaecer::poe_dica(store, &[id]).is_none() {
            store.set_tooltip(id, tr(dica));
        }
    }
}

fn faixa() -> (f32, f32) {
    (ids::PAINTER_BRUSH_DAB_PX_MIN, ids::PAINTER_BRUSH_DAB_PX_MAX)
}

/// O número do «Dab size (px)», à vista só com a caixa ligada.
pub(crate) fn numeros(l: &BrushSettings, v: &BrushSettings) -> Vec<Numero> {
    if !presente(l) || !l.dab_fixo {
        return Vec::new();
    }
    vec![numero(
        CAMPO,
        ids::PAINTER_BRUSH_DAB_PX,
        v.dab_px,
        faixa(),
        crate::number_field::SIZE_STEP,
        1,
    )]
}

/// As linhas do cartão: a caixa e, ligada, o campo ao lado.
pub(crate) fn itens(l: &BrushSettings, v: &BrushSettings) -> Vec<Item> {
    if !presente(l) {
        return Vec::new();
    }
    let mut n = vec![Item::Caixa {
        id: ids::PAINTER_BRUSH_DAB_FIXO,
        chave: CAIXA,
        on: l.dab_fixo,
    }];
    n.extend(numeros(l, v).into_iter().map(Item::Num));
    n
}

fn seccao(ctx: &mut PaintCtx) -> Seccao {
    crate::seccoes::seccao_da_chave(ctx.text_system, CAMPO)
}

/// A altura que [`paint`] ocupa — o cartão do Jitter soma-a antes de pintar o fundo.
pub(crate) fn altura(ctx: &mut PaintCtx, x: f32, w: f32, b: &BrushSettings) -> f32 {
    let passo = ph2d_tokens::row_pitch_px();
    let cb = ph2d_editor_core::widget::Checkbox::new(ids::PAINTER_BRUSH_DAB_FIXO, tr(CAIXA))
        .seccao(crate::seccoes::seccao_da_chave(ctx.text_system, CAIXA));
    let rect = ph2d_editor_core::zones::Rect::new(x, 0.0, w, ph2d_tokens::ROW_H_PX);
    let mut h = passo * cb.linhas_do_nome(rect, ctx.text_system) as f32;
    if b.dab_fixo {
        let sec = seccao(ctx);
        let linhas = ph2d_editor_core::property_row::linhas_do_nome(
            ctx.text_system,
            x,
            w,
            tr(CAMPO),
            1,
            sec,
        );
        h += passo * linhas as f32;
    }
    h
}

/// O painel de antes: a caixa e, ligada, o campo. Devolve o próximo `y`.
pub(crate) fn paint(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    w: f32,
    mut y: f32,
    b: &BrushSettings,
) -> f32 {
    y = crate::paint_brush_top::paint_checkbox_row(
        ctx,
        theme,
        x,
        w,
        y,
        ids::PAINTER_BRUSH_DAB_FIXO,
        CAIXA,
        b.dab_fixo,
    );
    if b.dab_fixo {
        let sec = seccao(ctx);
        let (min, max) = faixa();
        y = crate::number_field::paint_num_row(
            ctx,
            theme,
            x,
            w,
            y,
            tr(CAMPO),
            ids::PAINTER_BRUSH_DAB_PX,
            b.dab_px,
            min,
            max,
            crate::number_field::SIZE_STEP,
            1,
            sec,
        );
    }
    poe_dicas(ctx.host.store_mut());
    y
}

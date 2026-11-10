//! O quadro da janela: o cartão Rules (uma caixa por regra, Color limit) e o cartão Issues (um grupo
//! por regra com a contagem e a severidade; por baixo, um aviso por linha), flutuando à esquerda da
//! coluna (a régua da janela UNDO HISTORY).

use ph2d_editor_core::panel::cards::banda::metrics as m;
use ph2d_editor_core::panel::cards::ferramenta::{caixas_aos_pares, cartao};
use ph2d_editor_core::panel::cards::linhas_extra::ListRow;
use ph2d_editor_core::panel::cards::{CardDef, Cell, Widget, borda, paint_card_list};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::card_field::Glyph;
use ph2d_editor_core::widget::panel_chrome::panel_drag_handle_rect;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_pixel_lint::{Regra, Regras, Severidade};
use ph2d_tokens::Spacing;

use crate::canal::{self, LintView};
use crate::{PixelLintPanel, event, ids};

fn solta(ctx: &mut PaintCtx) -> Rect {
    let area = ctx.layout.draw_area;
    let teto = {
        let r = ph2d_editor_core::ruler::top_band(area);
        r.y + r.h + Spacing::Sm.px()
    };
    let fundo = borda::fundo(ctx);
    let (esquerda, direita) = borda::faixa_x(ctx);
    let coluna = ctx.slot.x;
    let store = ctx.host.store_mut();
    let node = PixelLintPanel::NODE_ID;
    let w = borda::largura(store, node, area, coluna).min(direita - esquerda);
    let base = (direita - w).max(esquerda);
    borda::coloca(store, node, w, (esquerda, base), (teto, fundo))
}

/// A chave i18n do nome da regra.
#[must_use]
pub fn chave_da_regra(r: Regra) -> &'static str {
    match r {
        Regra::Orfao => "pixel.lint.rule.orphan",
        Regra::CantoDuplo => "pixel.lint.rule.doubles",
        Regra::Degrau => "pixel.lint.rule.jaggies",
        Regra::Faixa => "pixel.lint.rule.banding",
        Regra::Almofada => "pixel.lint.rule.pillow",
        Regra::CoresAMais => "pixel.lint.rule.too_many_colors",
    }
}

fn chave_da_severidade(s: Severidade) -> &'static str {
    match s {
        Severidade::Baixa => "pixel.lint.severity.low",
        Severidade::Media => "pixel.lint.severity.medium",
        Severidade::Alta => "pixel.lint.severity.high",
    }
}

fn cartao_regras(r: Regras) -> CardDef<'static> {
    cartao(
        ids::SEC_RULES,
        tr("pixel.lint.sec_rules"),
        None,
        move |c, mut y| {
            let caixas: Vec<_> = Regra::TODAS
                .iter()
                .map(|&regra| (ids::regra(regra), chave_da_regra(regra), r.ligada(regra)))
                .collect();
            y = caixas_aos_pares(c, &caixas, y);
            let limite = Cell::new(Widget::Field {
                id: ids::COLOR_LIMIT,
                glyph: Glyph::Icon(ph2d_editor_core::icons::IconId::Info),
            })
            .label(tr("pixel.lint.color_limit"));
            c.row(&[limite], y)
        },
    )
}

fn cartao_avisos(v: LintView) -> CardDef<'static> {
    cartao(
        ids::SEC_ISSUES,
        tr("pixel.lint.sec_issues"),
        None,
        move |c, mut y| {
            if !v.tem_documento {
                return c.hint(tr("pixel.lint.no_document"), y);
            }
            if v.grupos.is_empty() {
                return c.hint(tr("pixel.lint.none"), y);
            }
            for g in &v.grupos {
                let facto = format!(
                    "{} {} · {}",
                    g.avisos.len(),
                    tr("pixel.lint.count"),
                    tr(chave_da_severidade(g.severidade))
                );
                y = c.list_row(
                    &ListRow {
                        trailing: Some(&facto),
                        selected: v.escolhido.is_some_and(|(r, _)| r == g.regra),
                        ..ListRow::new(ids::grupo(g.regra), tr(chave_da_regra(g.regra)))
                    },
                    y,
                );
                for (i, cels) in g.avisos.iter().enumerate() {
                    let Some([x, yy]) = cels.first() else {
                        continue;
                    };
                    let onde = format!("{x}, {yy}");
                    y = c.list_row(
                        &ListRow {
                            depth: 1,
                            selected: v.escolhido == Some((g.regra, i)),
                            ..ListRow::new(ids::aviso(g.regra, i), &onde)
                        },
                        y,
                    );
                }
            }
            y
        },
    )
}

pub(crate) fn paint(ctx: &mut PaintCtx) {
    let node = PixelLintPanel::NODE_ID;
    let aberto = ctx.host.panel_visible(PixelLintPanel::ID);
    canal::poe_aberto(aberto);
    if !aberto {
        ctx.host.store_mut().clear_panel_rect(node);
        return;
    }
    let v = canal::view();
    event::regista_avisos(ctx.host.store_mut(), &v);
    let titulo = <PixelLintPanel as Panel>::TITLE.tr();
    let cartoes = [cartao_regras(canal::regras()), cartao_avisos(v)];
    let slot = solta(ctx);
    let encaixe = ctx.slot;
    ctx.slot = slot;
    borda::declara_fecho(ctx.host.store_mut(), node);
    let lista = paint_card_list(ctx, node, titulo, &cartoes);
    ctx.slot = encaixe;
    ctx.host.store_mut().set_panel_rect(node, lista);
    let reserva = borda::reserva_do_titulo();
    ctx.host.hit_index_mut().register(
        ids::LINT_DRAG,
        panel_drag_handle_rect(lista, m::LIST_HEAD_H, reserva),
    );
    borda::regista(ctx.host.store_mut(), ids::LINT_EDGE, node);
    borda::pega(ctx, ids::LINT_EDGE, lista);
}

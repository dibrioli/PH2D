//! ⭐⭐ **O painel Painter em CARTÕES** (`docs/CartoesUI/13_PAINTER_PLANO.md`): uma lista, com Brush/Layers numa
//! faixa presa debaixo do título. Mesmos ids de secção e de controlo; `populate` e `event` não mudam.
//!
//! Secção convertida = corpo em [`crate::cards_nativos`]. Secção por converter = o corpo de hoje, capturado
//! ([`crate::cards_captura`]). Em ambos o pintor antigo decide se a secção está à vista e dá o título.

use crate::cards_captura::{Captura, coluna, y_de};
use crate::{
    cards_impasto, cards_nativos, cards_shape, cards_shape_tom, cards_topo, cards_watercolor,
    cards_wet, paint, paint_brush, paint_brush_top, preview_pino, state,
};
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::IconId;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::interaction::PanelRowFamily;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::panel::cards::{
    CardBody, CardCtx, CardDef, HeadAction, ListTop, Seg, Widget, paint_card_list_top,
};
use ph2d_editor_core::panel::section_plan_ctx::TarefaCtx;
use ph2d_editor_core::widget::card_ink::CardInk;
use ph2d_editor_core::widget::panel_chrome::PANEL_HEAD_PAD;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::{ROW_H_PX, Spacing, Theme};
use ph2d_tool_painter::ids as pids;
use std::collections::BTreeSet;

pub(crate) use crate::cards_captura::declara_reset;

/// Os cartões dos blocos sem título do corpo do pincel, pela ordem de `plano_do_corpo`.
const BLOCOS: [NodeId; 3] = [
    pids::PAINTER_CARD_TOPO,
    pids::PAINTER_CARD_BASICOS,
    pids::PAINTER_CARD_MIXING,
];

thread_local! {
    static FORCADO: std::cell::Cell<Option<bool>> = const { std::cell::Cell::new(None) };
}

/// O painel pinta-se em cartões? `PH2D_CARDS=0` = o painel de antes.
#[must_use]
pub fn cards_on() -> bool {
    FORCADO
        .with(std::cell::Cell::get)
        .unwrap_or_else(|| !cfg!(test) && std::env::var("PH2D_CARDS").as_deref() != Ok("0"))
}

/// Força o modo NESTA thread: os testes de um processo correm em paralelo. `None` volta ao ambiente.
#[doc(hidden)]
pub fn force_cards_on_this_thread(on: Option<bool>) {
    FORCADO.with(|f| f.set(on));
}

/// Um cartão à vista neste quadro.
struct Presente {
    id: NodeId,
    titulo: String,
    conteudo: Conteudo,
}

/// O corpo de hoje, capturado; as linhas do topo e dos básicos; o Wet Paint, com o visor do Tilt capturado.
enum Conteudo {
    Captura(Captura),
    Linhas(Vec<cards_topo::Linha>),
    Wet(Captura),
    /// A prévia do Flow.
    Watercolor(Captura),
    /// Os knobs do verbo Sculpt na mão.
    Impasto(Option<Captura>),
    /// Os gizmos, a prévia, as linhas de cada camada.
    Forma(cards_shape::Pecas),
    /// O editor de cada rampa.
    Tom(cards_shape_tom::Pecas),
}

impl Presente {
    fn reset(&self) -> Option<NodeId> {
        match &self.conteudo {
            Conteudo::Captura(c) => c.reset,
            Conteudo::Linhas(_) => None,
            Conteudo::Wet(_) => Some(pids::PAINTER_WETPAINT_RESET),
            Conteudo::Watercolor(_) => Some(pids::PAINTER_WATERCOLOR_RESET),
            Conteudo::Impasto(_) => Some(pids::PAINTER_IMPASTO_RESET),
            Conteudo::Forma(_) => Some(pids::PAINTER_SHAPE_RESET),
            Conteudo::Tom(_) => Some(cards_shape_tom::reset()),
        }
    }
}

/// Corre as secções do pincel, e fica com as que estão à vista.
fn secoes_do_pincel(ctx: &mut PaintCtx, theme: Theme) -> Vec<Presente> {
    let brush = state::current_brush().unwrap_or(paint_brush::FALLBACK_BRUSH);
    let (x, w) = coluna(ctx.slot);
    // Deform e Selection são donos do corpo inteiro, como no painel de antes.
    let entradas: Vec<(NodeId, bool, TarefaCtx<'_>)> = if brush.is_deform {
        vec![(
            pids::PAINTER_CARD_EXCLUSIVO,
            false,
            Box::new(move |ctx, tema, y| {
                crate::paint_deform::paint_deform_section(ctx, tema, x, w, y, brush)
            }),
        )]
    } else if brush.is_selection {
        vec![(
            pids::PAINTER_CARD_EXCLUSIVO,
            false,
            Box::new(move |ctx, tema, y| {
                crate::paint_selection::paint_selection_section(ctx, tema, x, w, y, brush)
            }),
        )]
    } else {
        paint_brush::prepara_a_cor(ctx, brush);
        let mut blocos = BLOCOS.iter();
        paint_brush::plano_do_corpo(x, w, brush)
            .entradas()
            .into_iter()
            .filter_map(|(id, tarefa)| match id {
                Some(id) => Some((id, true, tarefa)),
                None => blocos.next().map(|b| (*b, false, tarefa)),
            })
            .collect()
    };
    let topo = ctx.slot.y;
    let mut presentes = Vec::with_capacity(entradas.len());
    for (id, absorve, tarefa) in entradas {
        let ink = CardInk::of_card(ctx.host.store().section_theme(id), theme);
        if id == pids::PAINTER_CARD_TOPO || id == pids::PAINTER_CARD_BASICOS {
            let linhas = if id == pids::PAINTER_CARD_TOPO {
                cards_topo::do_topo(ctx, ink, (x, w), brush)
            } else {
                cards_topo::dos_basicos(ctx, ink, (x, w), brush)
            };
            if !linhas.is_empty() {
                presentes.push(Presente {
                    id,
                    titulo: titulo_do_bloco(id, &brush),
                    conteudo: Conteudo::Linhas(linhas),
                });
            }
            continue;
        }
        if id == pids::PAINTER_WETPAINT_SECTION {
            if cards_wet::presente(&brush) {
                presentes.push(Presente {
                    id,
                    titulo: cards_wet::titulo().to_owned(),
                    conteudo: Conteudo::Wet(cards_wet::visor(ctx, ink, (x, w), brush)),
                });
            }
            continue;
        }
        if id == pids::PAINTER_WATERCOLOR_SECTION {
            if cards_watercolor::presente(&brush) {
                presentes.push(Presente {
                    id,
                    titulo: cards_watercolor::titulo().to_owned(),
                    conteudo: Conteudo::Watercolor(cards_watercolor::previa(
                        ctx,
                        ink,
                        (x, w),
                        brush,
                    )),
                });
            }
            continue;
        }
        if id == pids::PAINTER_IMPASTO_SECTION {
            if cards_impasto::presente(&brush) {
                presentes.push(Presente {
                    id,
                    titulo: cards_impasto::titulo().to_owned(),
                    conteudo: Conteudo::Impasto(cards_impasto::escultura(ctx, ink, (x, w), brush)),
                });
            }
            continue;
        }
        if id == pids::PAINTER_SHAPE_SECTION {
            if cards_shape::presente(&brush) {
                presentes.push(Presente {
                    id,
                    titulo: cards_shape::titulo().to_owned(),
                    conteudo: Conteudo::Forma(cards_shape::pecas(ctx, ink, (x, w), brush)),
                });
            }
            continue;
        }
        if id == pids::PAINTER_SHAPE_RAMP_SECTION {
            if cards_shape_tom::presente(&brush) {
                presentes.push(Presente {
                    id,
                    titulo: cards_shape_tom::titulo().to_owned(),
                    conteudo: Conteudo::Tom(cards_shape_tom::pecas(ctx, ink, (x, w), brush)),
                });
            }
            continue;
        }
        let tema = ink.theme;
        let posicao = (x, y_de(id, topo));
        let captura = Captura::de(ctx, id, ink, posicao, absorve, |ctx, y| {
            tarefa(ctx, tema, y)
        });
        let titulo = if absorve {
            captura.titulo.clone()
        } else {
            captura.pintou().then(|| titulo_do_bloco(id, &brush))
        };
        if let Some(titulo) = titulo {
            presentes.push(Presente {
                id,
                titulo,
                conteudo: Conteudo::Captura(captura),
            });
        }
    }
    presentes
}

/// O topo não tem título, como o cartão Object: o nome da ferramenta já está na lista.
fn titulo_do_bloco(id: NodeId, brush: &ph2d_tool_painter::BrushSettings) -> String {
    let chave = if id == pids::PAINTER_CARD_TOPO {
        return String::new();
    } else if id == pids::PAINTER_CARD_BASICOS {
        "panel.painter_layers.brush.basics"
    } else if id == pids::PAINTER_CARD_MIXING {
        "panel.painter_layers.brush.mixing"
    } else {
        return paint_brush_top::header_title(false, Some(*brush)).to_owned();
    };
    tr(chave).to_owned()
}

/// A vista Layers: um cartão, por agora com a cara de antes. Devolve-o com as linhas arrastáveis e o fantasma.
fn cartao_das_camadas(ctx: &mut PaintCtx, theme: Theme) -> (Presente, Fileiras) {
    let (x, w) = coluna(ctx.slot);
    let id = pids::PAINTER_CARD_CAMADAS;
    let ink = CardInk::of_card(ctx.host.store().section_theme(id), theme);
    let tema = ink.theme;
    let mut linhas = BTreeSet::new();
    let mut fantasma = None;
    let posicao = (x, y_de(id, ctx.slot.y));
    let captura = Captura::de(ctx, id, ink, posicao, false, |ctx, y| {
        let fora = x - PANEL_HEAD_PAD;
        let largo = w + 2.0 * PANEL_HEAD_PAD;
        paint::paint_action_toolbar(ctx, Rect::new(fora, y, largo, paint::TOOLBAR_H), tema);
        let y = y + paint::TOOLBAR_H;
        paint::paint_modifier_toolbar(ctx, Rect::new(fora, y, largo, paint::TOOLBAR_H), tema);
        let y = y + paint::TOOLBAR_H + Spacing::Xs.px();
        let area = Rect::new(fora, y, largo, 0.0);
        let (fim, ids, f) = crate::paint_layer_list::paint_layer_rows(ctx, tema, area, area, w, y);
        linhas = ids;
        fantasma = f;
        let fim = fim + Spacing::Xs.px();
        let apply = Rect::new(
            x + w - paint::APPLY_BTN_W,
            fim,
            paint::APPLY_BTN_W,
            ROW_H_PX,
        );
        paint::paint_apply_button(ctx, apply, tema);
        fim + ROW_H_PX
    });
    (
        Presente {
            id,
            titulo: tr("panel.painter_layers.layers.layers").to_owned(),
            conteudo: Conteudo::Captura(captura),
        },
        (linhas, fantasma),
    )
}

/// As linhas arrastáveis e o fantasma da que voa: `(nome, x, y)`.
type Fileiras = (BTreeSet<NodeId>, Option<(String, f32, f32)>);

/// A prévia presa pelo pino, na faixa: `None` onde o painel de antes também não a põe.
fn previa_fixada(ctx: &mut PaintCtx, theme: Theme) -> Option<Captura> {
    preview_pino::esquece();
    let brush = state::current_brush().unwrap_or(paint_brush::FALLBACK_BRUSH);
    if !preview_pino::preview_fixada(ctx.host.store())
        || brush.is_deform
        || brush.is_selection
        || !preview_pino::ha_preview()
    {
        return None;
    }
    let (x, w) = coluna(ctx.slot);
    let id = core_ids::PAINTER_LAYERS_PANEL;
    let posicao = (x, y_de(id, ctx.slot.y));
    let c = Captura::de(
        ctx,
        id,
        CardInk::of_list(theme),
        posicao,
        false,
        |ctx, y| preview_pino::pinta(ctx, theme, x, w, y).unwrap_or(y),
    );
    c.pintou().then_some(c)
}

fn cartoes<'a>(presentes: &'a [Presente]) -> Vec<CardDef<'a>> {
    presentes
        .iter()
        .map(|p| {
            let body: CardBody<'a> = match (cards_nativos::corpo(p.id), &p.conteudo) {
                (Some(f), _) => Box::new(f),
                (None, Conteudo::Captura(cap)) => {
                    Box::new(move |c: &mut CardCtx<'_>, y| cap.repete(c, y))
                }
                (None, Conteudo::Linhas(l)) => {
                    Box::new(move |c: &mut CardCtx<'_>, y| cards_topo::corpo(c, l, y))
                }
                (None, Conteudo::Wet(visor)) => {
                    Box::new(move |c: &mut CardCtx<'_>, y| cards_wet::corpo(c, visor, y))
                }
                (None, Conteudo::Watercolor(previa)) => {
                    Box::new(move |c: &mut CardCtx<'_>, y| cards_watercolor::corpo(c, previa, y))
                }
                (None, Conteudo::Impasto(esculpe)) => Box::new(move |c: &mut CardCtx<'_>, y| {
                    cards_impasto::corpo(c, esculpe.as_ref(), y)
                }),
                (None, Conteudo::Forma(pecas)) => {
                    Box::new(move |c: &mut CardCtx<'_>, y| cards_shape::corpo(c, pecas, y))
                }
                (None, Conteudo::Tom(pecas)) => {
                    Box::new(move |c: &mut CardCtx<'_>, y| cards_shape_tom::corpo(c, pecas, y))
                }
            };
            CardDef {
                section: p.id,
                title: &p.titulo,
                actions: p
                    .reset()
                    .map(|id| HeadAction {
                        id,
                        icon: IconId::Reset,
                        on: false,
                    })
                    .into_iter()
                    .collect(),
                add: None,
                summary: None,
                body: Some(body),
            }
        })
        .collect()
}

/// O painel inteiro, depois de `set_panel_rect`.
pub(crate) fn paint(ctx: &mut PaintCtx, theme: Theme) {
    let panel = core_ids::PAINTER_LAYERS_PANEL;
    ctx.host
        .store_mut()
        .mark_card_list_close(panel, pids::PAINTER_LAYERS_CLOSE);
    let camadas = state::current_dock_shows_layers();
    if camadas && ctx.host.store().picker_target() == Some(core_ids::PAINTER_COLOR_THUMB) {
        ctx.host.store_mut().set_picker_target(None);
    }
    let previa = if camadas {
        None
    } else {
        previa_fixada(ctx, theme)
    };
    let (presentes, fileiras) = if camadas {
        let (p, f) = cartao_das_camadas(ctx, theme);
        (vec![p], Some(f))
    } else {
        (secoes_do_pincel(ctx, theme), None)
    };
    let pincel = state::current_brush().unwrap_or(paint_brush::FALLBACK_BRUSH);
    cards_nativos::prepare(ctx.host.store_mut());
    if !camadas {
        cards_topo::prepare(ctx.host.store_mut(), &pincel);
        cards_wet::prepare(ctx.host.store_mut(), &pincel);
        cards_watercolor::prepare(ctx.host.store_mut(), &pincel);
        cards_impasto::prepare(ctx, &pincel);
        cards_shape::prepare(ctx, &pincel);
        cards_shape_tom::prepare(ctx, &pincel);
    }
    for p in &presentes {
        if let Some(id) = p.reset()
            && ctx.host.store().tooltip_for(id).is_none()
        {
            ctx.host
                .store_mut()
                .set_tooltip(id, tr("panel.painter_layers.brush.reset_section"));
        }
    }

    let titulo = paint_brush_top::header_title(camadas, state::current_brush());
    let modos = [
        Seg {
            id: pids::PAINTER_SIDEBAR_TOGGLE_DOCK,
            icon: None,
            text: tr("panel.painter_layers.layers.brush"),
            on: !camadas,
        },
        Seg {
            id: pids::PAINTER_LAYERS_TOGGLE_DOCK,
            icon: None,
            text: tr("panel.painter_layers.layers.layers"),
            on: camadas,
        },
    ];
    let previa_h = previa
        .as_ref()
        .map_or(0.0, |p| m::FIELD_ROW_GAP + p.altura());
    let faixa = |c: &mut CardCtx<'_>, r: Rect| {
        let y = c.full(None, Widget::Choice(&modos), r.y);
        if let Some(p) = &previa {
            let _ = p.repete(c, y + m::FIELD_ROW_GAP);
        }
    };
    let preso = ListTop {
        h: m::FIELD_H + previa_h,
        body: &faixa,
        add: None,
    };
    {
        let lista = cartoes(&presentes);
        paint_card_list_top(ctx, panel, titulo, &lista, None, &[], Some(preso));
    }

    let store = ctx.host.store();
    let conteudo = store.panel_content_h(panel).unwrap_or(0.0);
    let visivel = store.panel_visible_h(panel).unwrap_or(0.0);
    state::set_last_content_h(conteudo);
    state::set_last_visible_h(visivel);
    match fileiras {
        Some((linhas, fantasma)) => {
            ctx.host
                .store_mut()
                .set_panel_row_ids(PanelRowFamily::PainterLayer, linhas);
            paint::paint_layers_popovers(ctx, theme, fantasma);
        }
        None => {
            cards_topo::listas_abertas(ctx, &pincel);
            cards_wet::listas_abertas(ctx, &pincel);
            cards_watercolor::listas_abertas(ctx, &pincel);
            cards_shape::listas_abertas(ctx, &pincel);
            cards_shape_tom::listas_abertas(ctx, &pincel);
            paint::paint_brush_view_popovers(ctx, theme);
        }
    }
}

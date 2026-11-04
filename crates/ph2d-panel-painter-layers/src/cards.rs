//! ⭐⭐ **O painel Painter em CARTÕES** (`docs/CartoesUI/13_PAINTER_PLANO.md`): uma lista, com Brush/Layers numa
//! faixa presa debaixo do título. Mesmos ids de secção e de controlo; `populate` e `event` não mudam.
//!
//! Secção convertida = corpo em [`crate::cards_nativos`]. Secção por converter = o corpo de hoje, capturado
//! ([`crate::cards_captura`]). Em ambos o pintor antigo decide se a secção está à vista e dá o título.

use crate::cards_captura::{Captura, coluna, y_de};
use crate::{
    cards_aleatorio, cards_camadas, cards_caneta, cards_composto, cards_exclusivos, cards_ferramenta, cards_grao,
    cards_impasto, cards_nativos, cards_papel, cards_shape, cards_shape_tom, cards_topo,
    cards_traco, cards_watercolor, cards_wet, paint, paint_brush, paint_brush_top, preview_pino,
    state,
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
use ph2d_editor_core::widget::card_ink::CardInk;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::Theme;
use ph2d_tool_painter::ids as pids;

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
    /// Os ajustes e o editor da camada Texture.
    Camadas(cards_camadas::Pecas),
    /// Um corpo de `cards_nativos`, com o Reset dele.
    Nativo(Option<NodeId>),
    Linhas(Vec<cards_topo::Linha>),
    Wet(Captura),
    /// A prévia do Flow.
    Watercolor(Captura),
    Impasto,
    /// Os gizmos, a prévia, as linhas de cada camada.
    Forma(cards_shape::Pecas),
    /// O editor de cada rampa.
    Tom(cards_shape_tom::Pecas),
    /// A prévia do padrão.
    Papel(Option<Captura>),
    /// A prévia, o editor da rampa, os dois títulos que dobram dentro do cartão.
    Grao(cards_grao::Pecas),
    Aleatorio,
    /// O título do Jitter, que dobra dentro do cartão.
    Traco(Option<Captura>),
    Caneta,
    Mascara,
    Deform,
    Selecao,
}

impl Presente {
    fn reset(&self) -> Option<NodeId> {
        match &self.conteudo {
            Conteudo::Nativo(r) => *r,
            Conteudo::Linhas(_)
            | Conteudo::Camadas(_)
            | Conteudo::Mascara
            | Conteudo::Deform
            | Conteudo::Selecao => None,
            Conteudo::Wet(_) => Some(pids::PAINTER_WETPAINT_RESET),
            Conteudo::Watercolor(_) => Some(pids::PAINTER_WATERCOLOR_RESET),
            Conteudo::Impasto => Some(pids::PAINTER_IMPASTO_RESET),
            Conteudo::Forma(_) => Some(pids::PAINTER_SHAPE_RESET),
            Conteudo::Tom(_) => Some(cards_shape_tom::reset()),
            Conteudo::Papel(_) => Some(pids::PAINTER_WATERCOLOR_PAPER_RESET),
            Conteudo::Grao(_) => Some(pids::PAINTER_BRUSH_TEXTURE_RESET),
            Conteudo::Aleatorio => Some(pids::PAINTER_BRUSH_RANDOMIZE_RESET),
            Conteudo::Traco(_) => Some(pids::PAINTER_BRUSH_STROKE_RESET),
            Conteudo::Caneta => Some(pids::PAINTER_PEN_RESET),
        }
    }
}

/// Corre as secções do pincel, e fica com as que estão à vista.
fn secoes_do_pincel(ctx: &mut PaintCtx, theme: Theme) -> Vec<Presente> {
    let brush = state::current_brush().unwrap_or(paint_brush::FALLBACK_BRUSH);
    let (x, w) = coluna(ctx.slot);
    // Deform e Selection são donos do corpo inteiro, como no painel de antes.
    if brush.is_deform || brush.is_selection {
        return vec![Presente {
            id: pids::PAINTER_CARD_EXCLUSIVO,
            titulo: paint_brush_top::header_title(false, Some(brush)).to_owned(),
            conteudo: if brush.is_deform {
                Conteudo::Deform
            } else {
                Conteudo::Selecao
            },
        }];
    }
    paint_brush::prepara_a_cor(ctx, brush);
    let mut blocos = BLOCOS.iter();
    let ids: Vec<NodeId> = paint_brush::plano_do_corpo(x, w, brush)
        .entradas()
        .into_iter()
        .filter_map(|(id, _)| id.or_else(|| blocos.next().copied()))
        .collect();
    let mut presentes = Vec::with_capacity(ids.len());
    for id in ids {
        let ink = CardInk::of_card(ctx.host.store().section_theme(id), theme);
        if id == pids::PAINTER_CARD_TOPO || id == pids::PAINTER_CARD_BASICOS {
            let linhas = if id == pids::PAINTER_CARD_TOPO {
                cards_topo::do_topo(ctx, ink, (x, w), brush)
            } else {
                cards_topo::dos_basicos(brush)
            };
            if !linhas.is_empty() {
                presentes.push(Presente {
                    id,
                    titulo: titulo_do_bloco(id),
                    conteudo: Conteudo::Linhas(linhas),
                });
            }
            continue;
        }
        if id == pids::PAINTER_MASK_SECTION {
            if brush.is_mask {
                presentes.push(Presente {
                    id,
                    titulo: tr("panel.painter_layers.mask.mask").to_owned(),
                    conteudo: Conteudo::Mascara,
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
                    conteudo: Conteudo::Impasto,
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
        if id == pids::PAINTER_WATERCOLOR_PAPER_SECTION {
            if cards_papel::presente(&brush) {
                presentes.push(Presente {
                    id,
                    titulo: cards_papel::titulo().to_owned(),
                    conteudo: Conteudo::Papel(cards_papel::previa(ctx, ink, (x, w), brush)),
                });
            }
            continue;
        }
        if id == pids::PAINTER_BRUSH_TEXTURE_SECTION {
            if cards_grao::presente(&brush) {
                presentes.push(Presente {
                    id,
                    titulo: cards_grao::titulo().to_owned(),
                    conteudo: Conteudo::Grao(cards_grao::pecas(ctx, ink, (x, w), brush)),
                });
            }
            continue;
        }
        if id == pids::PAINTER_BRUSH_RANDOMIZE_SECTION {
            if cards_aleatorio::presente(&brush) {
                presentes.push(Presente {
                    id,
                    titulo: cards_aleatorio::titulo().to_owned(),
                    conteudo: Conteudo::Aleatorio,
                });
            }
            continue;
        }
        if id == pids::PAINTER_BRUSH_STROKE_SECTION {
            if cards_traco::presente(&brush) {
                presentes.push(Presente {
                    id,
                    titulo: cards_traco::titulo().to_owned(),
                    conteudo: Conteudo::Traco(cards_traco::titulo_do_jitter(
                        ctx,
                        ink,
                        (x, w),
                        &brush,
                    )),
                });
            }
            continue;
        }
        if id == pids::PAINTER_PEN_SECTION {
            if cards_caneta::presente(&brush) {
                presentes.push(Presente {
                    id,
                    titulo: cards_caneta::titulo().to_owned(),
                    conteudo: Conteudo::Caneta,
                });
            }
            continue;
        }
        if let Some((titulo, reset)) = cards_nativos::presente(id, &brush) {
            presentes.push(Presente {
                id,
                titulo,
                conteudo: Conteudo::Nativo(reset),
            });
        }
    }
    presentes
}

/// O topo não tem título, como o cartão Object: o nome da ferramenta já está na lista.
fn titulo_do_bloco(id: NodeId) -> String {
    if id == pids::PAINTER_CARD_TOPO {
        String::new()
    } else {
        tr("panel.painter_layers.brush.basics").to_owned()
    }
}

/// A vista Layers: um cartão, com os ajustes e o editor da Texture capturados.
fn cartao_das_camadas(ctx: &mut PaintCtx, theme: Theme) -> Presente {
    let id = pids::PAINTER_CARD_CAMADAS;
    let ink = CardInk::of_card(ctx.host.store().section_theme(id), theme);
    Presente {
        id,
        titulo: tr("panel.painter_layers.layers.layers").to_owned(),
        conteudo: Conteudo::Camadas(cards_camadas::pecas(ctx, ink, coluna(ctx.slot))),
    }
}

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
        |ctx, y| preview_pino::pinta(ctx, theme, x, w, y).unwrap_or(y),
    );
    c.pintou().then_some(c)
}

fn pincel_atual() -> ph2d_tool_painter::BrushSettings {
    state::current_brush().unwrap_or(paint_brush::FALLBACK_BRUSH)
}

fn cartoes<'a>(presentes: &'a [Presente]) -> Vec<CardDef<'a>> {
    presentes
        .iter()
        .map(|p| {
            let body: CardBody<'a> = match (cards_nativos::corpo(p.id), &p.conteudo) {
                (Some(f), _) => Box::new(f),
                (None, Conteudo::Nativo(_)) => Box::new(|_: &mut CardCtx<'_>, y| y),
                (None, Conteudo::Camadas(pecas)) => {
                    Box::new(move |c: &mut CardCtx<'_>, y| cards_camadas::corpo(c, pecas, y))
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
                (None, Conteudo::Impasto) => Box::new(cards_impasto::corpo),
                (None, Conteudo::Forma(pecas)) => {
                    Box::new(move |c: &mut CardCtx<'_>, y| cards_shape::corpo(c, pecas, y))
                }
                (None, Conteudo::Tom(pecas)) => {
                    Box::new(move |c: &mut CardCtx<'_>, y| cards_shape_tom::corpo(c, pecas, y))
                }
                (None, Conteudo::Papel(previa)) => Box::new(move |c: &mut CardCtx<'_>, y| {
                    cards_papel::corpo(c, previa.as_ref(), y)
                }),
                (None, Conteudo::Grao(pecas)) => {
                    Box::new(move |c: &mut CardCtx<'_>, y| cards_grao::corpo(c, pecas, y))
                }
                (None, Conteudo::Aleatorio) => Box::new(cards_aleatorio::corpo),
                (None, Conteudo::Traco(jitter)) => Box::new(move |c: &mut CardCtx<'_>, y| {
                    cards_traco::corpo(c, jitter.as_ref(), y)
                }),
                (None, Conteudo::Caneta) => Box::new(cards_caneta::corpo),
                (None, Conteudo::Mascara) => Box::new(|c: &mut CardCtx<'_>, y| {
                    cards_exclusivos::mascara(c, &pincel_atual(), y)
                }),
                (None, Conteudo::Deform) => Box::new(|c: &mut CardCtx<'_>, y| {
                    cards_exclusivos::deform(c, &pincel_atual(), y)
                }),
                (None, Conteudo::Selecao) => Box::new(|c: &mut CardCtx<'_>, y| {
                    cards_exclusivos::selecao(c, &pincel_atual(), y)
                }),
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
    let presentes = if camadas {
        vec![cartao_das_camadas(ctx, theme)]
    } else {
        secoes_do_pincel(ctx, theme)
    };
    let pincel = state::current_brush().unwrap_or(paint_brush::FALLBACK_BRUSH);
    cards_nativos::prepare(ctx.host.store_mut());
    if camadas {
        cards_camadas::prepare(ctx);
    } else {
        cards_topo::prepare(ctx.host.store_mut(), &pincel);
        cards_wet::prepare(ctx.host.store_mut(), &pincel);
        cards_watercolor::prepare(ctx.host.store_mut(), &pincel);
        cards_impasto::prepare(ctx, &pincel);
        cards_shape::prepare(ctx, &pincel);
        cards_shape_tom::prepare(ctx, &pincel);
        cards_papel::prepare(ctx, &pincel);
        cards_grao::prepare(ctx, &pincel);
        cards_aleatorio::prepare(ctx, &pincel);
        cards_traco::prepare(ctx, &pincel);
        cards_caneta::prepare(ctx, &pincel);
        cards_ferramenta::prepare(ctx, &pincel);
        cards_composto::prepare(ctx, &pincel);
        cards_exclusivos::prepare(ctx.host.store_mut(), &pincel);
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
    match camadas {
        true => {
            ctx.host.store_mut().set_panel_row_ids(
                PanelRowFamily::PainterLayer,
                cards_camadas::linhas_arrastaveis(),
            );
            cards_camadas::listas_abertas(ctx, coluna(ctx.slot));
            paint::paint_layers_popovers(ctx, theme, None);
        }
        false => {
            cards_topo::listas_abertas(ctx, &pincel);
            cards_ferramenta::listas_abertas(ctx, &pincel);
            cards_composto::listas_abertas(ctx, &pincel);
            cards_wet::listas_abertas(ctx, &pincel);
            cards_watercolor::listas_abertas(ctx, &pincel);
            cards_shape::listas_abertas(ctx, &pincel);
            cards_shape_tom::listas_abertas(ctx, &pincel);
            cards_papel::listas_abertas(ctx, &pincel);
            cards_grao::listas_abertas(ctx, &pincel);
            cards_traco::listas_abertas(ctx, &pincel);
            cards_caneta::listas_abertas(ctx, &pincel, coluna(ctx.slot));
            paint::paint_brush_view_popovers(ctx, theme);
        }
    }
}

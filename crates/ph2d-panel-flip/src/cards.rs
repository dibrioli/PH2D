//! ⭐ **O painel Flip em CARTÕES** (`docs/CartoesUI/11_RECEITA_PAINEL_EM_CARTOES.md`): as mesmas opções
//! por modo e os MESMOS ids do corpo de antes; `populate` e `event.rs` não mudam. Cada modo mostra só
//! os seus cartões. O campo é o chip ligado à barra, e é a barra que avisa a ferramenta.

use crate::ids::{self, FlipLayerWidget};
use crate::paint_sections::tool_button;
use crate::state::{FlipLayerRow, FlipLayersSnapshot, FlipPanelState};
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::IconId;
use ph2d_editor_core::interaction::{
    HierarchyDragState, InteractiveState, PanelRowFamily, WidgetStore,
};
use ph2d_editor_core::paint::{fill_rounded_rect, paint_icon};
use ph2d_editor_core::panel::cards::ferramenta::{
    Numero, aos_pares, caixas_aos_pares, campo, cartao, opcoes, prepara_numeros,
};
use ph2d_editor_core::panel::cards::linhas_extra::ListRow;
use ph2d_editor_core::panel::cards::{CardCtx, CardDef, Cell, Seg, Widget, dropdown};
use ph2d_editor_core::widget::card_field::paint_key_mark;
use ph2d_editor_core::widget::{ButtonState, SliderOrientation, SliderState, TextInputState};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_painter_effects::{BlendMode, MAX_BLEND_MODES};
use ph2d_tokens::{ColorToken, StrokeToken};
use ph2d_tool_flip::ids as t;
use ph2d_tool_flip::{
    Cap, EditDomain, EraseMode, FillMode, FlipMode, FlipStyleSnapshot, ReshapeKind, StrokeTip,
};

#[path = "cards_camadas.rs"]
mod camadas;
use camadas::cartao_das_camadas;
pub(crate) use camadas::toma_a_queda;
#[path = "cards_pincel.rs"]
mod pincel;
use pincel::cartao_do_pincel;

/// Os passos do domínio de cada grandeza (os mesmos do `populate`).
const INTEIRO: f64 = 1.0;
const CENTESIMO: f64 = 0.01; // LITERAL-PX-OK: fração 0..1
const VIGESIMO: f64 = 0.05; // LITERAL-PX-OK: unidades de mundo do Gap
const DECIMO: f64 = 0.1; // LITERAL-PX-OK: múltiplo do diâmetro, Precision
const PARA_PORCENTO: f64 = 100.0; // LITERAL-PX-OK: fração 0..1 mostrada em %

fn numeros(s: &FlipStyleSnapshot) -> Vec<Numero> {
    let f = FlipStyleSnapshot::default();
    let n = |chip, step, fabrica: f64, valor: f64| Numero {
        chip,
        step,
        scrub: step,
        fabrica,
        valor,
    };
    let pct = |v: f32| (f64::from(v) * PARA_PORCENTO).round();
    vec![
        n(ids::FLIP_SIZE_NUM, INTEIRO, f.width_px, s.width_px),
        n(ids::FLIP_ERASE_SIZE_NUM, INTEIRO, f.erase_px, s.erase_px),
        n(
            ids::FLIP_HARDNESS_NUM,
            CENTESIMO,
            f64::from(f.hardness),
            f64::from(s.hardness),
        ),
        n(
            ids::FLIP_OPACITY_NUM,
            INTEIRO,
            pct(f.opacity),
            pct(s.opacity),
        ),
        n(
            ids::FLIP_ERASE_STRENGTH_NUM,
            INTEIRO,
            pct(f.erase_strength),
            pct(s.erase_strength),
        ),
        n(
            ids::FLIP_SMOOTHING_NUM,
            CENTESIMO,
            f64::from(f.smoothing),
            f64::from(s.smoothing),
        ),
        n(
            ids::FLIP_PRESSURE_MIN_NUM,
            INTEIRO,
            pct(f.pressure_min_width),
            pct(s.pressure_min_width),
        ),
        n(
            ids::FLIP_PRESSURE_RESPONSE_NUM,
            INTEIRO,
            pct(f.pressure_response),
            pct(s.pressure_response),
        ),
        n(
            ids::FLIP_DOT_SPACING_NUM,
            DECIMO,
            f.dot_spacing,
            s.dot_spacing,
        ),
        n(ids::FLIP_GAP_NUM, VIGESIMO, f.gap, s.gap),
        n(ids::FLIP_TRAP_NUM, INTEIRO, f.trap, s.trap),
        n(ids::FLIP_GROW_NUM, INTEIRO, f.grow, s.grow),
        n(ids::FLIP_PRECISION_NUM, DECIMO, f.precision, s.precision),
        n(
            ids::FLIP_COLORIZE_BLEED_NUM,
            INTEIRO,
            f.colorize_bleed * PARA_PORCENTO,
            (s.colorize_bleed * PARA_PORCENTO).round(),
        ),
    ]
}

/// Antes de pintar, com o store mutável: os campos, as amostras, o fecho, o renomear e, por camada, o
/// que o corpo de antes registava a pintar.
pub(crate) fn prepare(
    state: &mut FlipPanelState,
    store: &mut WidgetStore,
    snap: &FlipStyleSnapshot,
    layers: &FlipLayersSnapshot,
) {
    store.mark_card_list_close(ph2d_editor_core::ids::FLIP_PANEL, ids::FLIP_CLOSE);
    prepara_numeros(store, &numeros(snap));
    for id in [
        ids::FLIP_STROKE_SWATCH,
        ids::FLIP_FILL_SWATCH,
        ids::FLIP_COLORIZE_SWATCH,
    ] {
        store.register_picker_swatch(id);
    }
    dica(store, t::FLIP_LINK_SIZE, "card.flip.link_size");
    dica(store, t::FLIP_LINK_STRENGTH, "card.flip.link_strength");
    crate::paint_layers::seed_rename(state, store, layers);
    for row in &layers.rows {
        prepara_camada(store, row);
    }
    store.set_panel_row_ids(
        PanelRowFamily::FlipLayer,
        layers
            .rows
            .iter()
            .map(|r| ids::flip_layer_widget_id(r.id, FlipLayerWidget::Row))
            .collect(),
    );
}

fn dica(store: &mut WidgetStore, id: NodeId, chave: &'static str) {
    if store.tooltip_for(id).is_none() {
        store.set_tooltip(id, tr(chave).to_string());
    }
}

fn prepara_camada(store: &mut WidgetStore, row: &FlipLayerRow) {
    let id = |k| ids::flip_layer_widget_id(row.id, k);
    // A caixa da mistura é `Button` nos cartões: a lista abre pelo `Click`.
    for k in [
        FlipLayerWidget::Row,
        FlipLayerWidget::Visibility,
        FlipLayerWidget::Lock,
        FlipLayerWidget::MoveUp,
        FlipLayerWidget::MoveDown,
        FlipLayerWidget::Blend,
    ] {
        store.register_if_absent(
            id(k),
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    dropdown::declare(id(FlipLayerWidget::Blend));
    if dropdown::open() == Some(id(FlipLayerWidget::Blend)) {
        for modo in 0..MAX_BLEND_MODES {
            store.register_if_absent(
                ids::flip_layer_blend_option_id(row.id, modo),
                InteractiveState::Button {
                    state: ButtonState::Normal,
                },
            );
        }
    }
    dica(
        store,
        id(FlipLayerWidget::Visibility),
        "card.flip.layer_eye",
    );
    dica(store, id(FlipLayerWidget::Lock), "card.flip.layer_lock");
    dica(store, id(FlipLayerWidget::MoveUp), "card.flip.layer_up");
    dica(store, id(FlipLayerWidget::MoveDown), "card.flip.layer_down");
    numero_da_camada(
        store,
        id(FlipLayerWidget::Opacity),
        id(FlipLayerWidget::OpacityNum),
        row.opacity,
    );
    numero_da_camada(
        store,
        id(FlipLayerWidget::Depth),
        id(FlipLayerWidget::DepthNum),
        row.depth,
    );
}

fn numero_da_camada(store: &mut WidgetStore, barra: NodeId, chip: NodeId, valor: f32) {
    let pct = (f64::from(valor) * PARA_PORCENTO).round();
    store.register_if_absent(
        barra,
        InteractiveState::Slider {
            state: SliderState::Normal,
            value: valor,
            orientation: SliderOrientation::Horizontal,
        },
    );
    store.register_if_absent(
        chip,
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value: pct,
            buffer: String::new(),
            caret: 0,
            last_committed: pct,
            selection_anchor: None,
        },
    );
    store.link_slider_number_mapped_integer(barra, chip, PARA_PORCENTO as f32, 0.0);
    store.set_number_range(chip, 0.0, PARA_PORCENTO, INTEIRO);
    prepara_numeros(
        store,
        &[Numero {
            chip,
            step: INTEIRO,
            scrub: INTEIRO,
            fabrica: PARA_PORCENTO,
            valor: pct,
        }],
    );
}

fn hex(c: [u8; 4]) -> String {
    format!("{:02X}{:02X}{:02X}", c[0], c[1], c[2])
}

fn amostra<'a>(id: NodeId, rgba: [u8; 4], codigo: &'a str, rotulo: &'a str) -> Cell<'a> {
    Cell::new(Widget::Swatch {
        id,
        rgba: Some(rgba),
        text: codigo,
        edit: None,
        detach: None,
    })
    .label(rotulo)
}

fn escolha(
    c: &mut CardCtx<'_>,
    rotulo: Option<&str>,
    pares: &[(NodeId, &str, bool)],
    y: f32,
) -> f32 {
    let segs: Vec<Seg<'_>> = pares
        .iter()
        .map(|&(id, text, on)| Seg {
            id,
            icon: None,
            text,
            on,
        })
        .collect();
    c.full(rotulo, Widget::Choice(&segs), y)
}

/// ⭐ Os cartões do modo em curso, pela ordem do corpo de antes.
pub(crate) fn cards(
    snap: &FlipStyleSnapshot,
    layers: &FlipLayersSnapshot,
    renomeia: Option<u64>,
) -> Vec<CardDef<'static>> {
    let s = *snap;
    let mut out = vec![cartao_da_ferramenta(s)];
    if matches!(
        s.mode,
        FlipMode::Draw | FlipMode::Erase | FlipMode::Reshape | FlipMode::Edit
    ) {
        out.push(cartao_do_pincel(s));
    }
    if matches!(s.mode, FlipMode::Draw | FlipMode::Edit) {
        out.push(cartao_da_cor(s));
    }
    if s.mode == FlipMode::Fill {
        out.push(cartao_do_balde(s));
    }
    if s.mode == FlipMode::Colorize {
        out.push(cartao_do_colorize(s));
    }
    out.push(cartao_das_camadas(layers.clone(), renomeia));
    out
}

fn cartao_da_ferramenta(s: FlipStyleSnapshot) -> CardDef<'static> {
    cartao(
        ids::FLIP_SEC_TOOL,
        tr("panel.flip.tool.mode"),
        Some(tr(tool_button(s.mode).1).to_string()),
        move |c, y| {
            // No máximo três por fileira: o «Colorize» não cabe num quinto da coluna.
            let todas: Vec<(NodeId, &str, bool)> = FlipMode::tools_of(s.mode.object_mode())
                .iter()
                .map(|m| {
                    let (id, chave) = tool_button(*m);
                    (id, tr(chave), s.mode == *m)
                })
                .collect();
            let y = todas.chunks(3).fold(y, |y, fila| escolha(c, None, fila, y));
            opcoes_do_modo(c, s, y)
        },
    )
}

fn opcoes_do_modo(c: &mut CardCtx<'_>, s: FlipStyleSnapshot, y: f32) -> f32 {
    match s.mode {
        FlipMode::Draw => escolha(
            c,
            Some(tr("panel.flip.tool.shape")),
            &[
                (
                    t::FLIP_SHAPE_LINE,
                    tr("panel.flip.tool.line"),
                    !s.draw_filled,
                ),
                (
                    t::FLIP_SHAPE_FILLED,
                    tr("panel.flip.tool.filled"),
                    s.draw_filled,
                ),
            ],
            y,
        ),
        FlipMode::Erase => escolha(
            c,
            Some(tr("panel.flip.tool.erase")),
            &[
                (
                    t::FLIP_ERASE_SOFT,
                    tr("panel.flip.tool.soft"),
                    s.erase == EraseMode::Soft,
                ),
                (
                    t::FLIP_ERASE_HARD,
                    tr("panel.flip.tool.hard"),
                    s.erase == EraseMode::Hard,
                ),
                (
                    t::FLIP_ERASE_STROKE,
                    tr("panel.flip.tool.stroke"),
                    s.erase == EraseMode::Stroke,
                ),
            ],
            y,
        ),
        FlipMode::Reshape => {
            let todos: Vec<(NodeId, &str, bool)> = ReshapeKind::ALL
                .iter()
                .zip(t::FLIP_RESHAPE_KIND_IDS)
                .map(|(k, id)| (id, tr(k.label_key()), s.reshape == *k))
                .collect();
            let (a, b) = todos.split_at(todos.len() / 2);
            let y = escolha(c, Some(tr("panel.flip.tool.sculpt_brush")), a, y);
            escolha(c, None, b, y)
        }
        FlipMode::Trace => c.full(
            None,
            Widget::Button {
                id: ids::FLIP_TRACE_RESET,
                text: tr("panel.flip.tool.reset_shifts"),
                primary: false,
            },
            y,
        ),
        FlipMode::Edit => {
            let y = escolha(
                c,
                Some(tr("panel.flip.tool.select")),
                &[
                    (
                        t::FLIP_EDIT_DOM_STROKE,
                        tr("panel.flip.tool.stroke"),
                        s.edit_domain == EditDomain::Stroke,
                    ),
                    (
                        t::FLIP_EDIT_DOM_POINT,
                        tr("panel.flip.tool.point"),
                        s.edit_domain == EditDomain::Point,
                    ),
                    (
                        t::FLIP_EDIT_DOM_SEGMENT,
                        tr("panel.flip.tool.segment"),
                        s.edit_domain == EditDomain::Segment,
                    ),
                ],
                y,
            );
            let y = c.row(
                &[
                    Cell::new(Widget::Button {
                        id: ids::FLIP_EDIT_SELECT_ALL,
                        text: tr("panel.flip.tool.all"),
                        primary: false,
                    })
                    .label(tr("panel.flip.tool.selection")),
                    Cell::new(Widget::Button {
                        id: ids::FLIP_EDIT_DESELECT,
                        text: tr("panel.flip.tool.none"),
                        primary: false,
                    }),
                ],
                y,
            );
            c.danger_button(ids::FLIP_EDIT_DELETE, tr("panel.flip.tool.delete"), y)
        }
        FlipMode::Fill | FlipMode::Colorize => y,
    }
}

fn cartao_da_cor(s: FlipStyleSnapshot) -> CardDef<'static> {
    cartao(
        ids::FLIP_SEC_COLOR,
        tr("panel.flip.tool.color"),
        Some(hex(s.stroke)),
        move |c, y| {
            let (traco, miolo) = (hex(s.stroke), hex(s.fill_color));
            let linha = amostra(
                ids::FLIP_STROKE_SWATCH,
                s.stroke,
                &traco,
                tr("panel.flip.tool.stroke"),
            );
            if s.draw_filled || s.mode == FlipMode::Edit {
                let fill = amostra(
                    ids::FLIP_FILL_SWATCH,
                    s.fill_color,
                    &miolo,
                    tr("panel.flip.tool.fill"),
                );
                c.row(&[fill, linha], y)
            } else {
                c.row(&[linha], y)
            }
        },
    )
}

fn cartao_do_balde(s: FlipStyleSnapshot) -> CardDef<'static> {
    cartao(
        ids::FLIP_SEC_FILL,
        tr("panel.flip.tool.fill"),
        Some(hex(s.fill_color)),
        move |c, y| {
            let codigo = hex(s.fill_color);
            let y = c.row(
                &[amostra(
                    ids::FLIP_FILL_SWATCH,
                    s.fill_color,
                    &codigo,
                    tr("panel.flip.tool.color"),
                )],
                y,
            );
            let y = escolha(
                c,
                Some(tr("panel.flip.tool.bucket")),
                &[
                    (
                        t::FLIP_FILL_PAINT,
                        tr("panel.flip.tool.paint"),
                        s.fill_mode == FillMode::Paint,
                    ),
                    (
                        t::FLIP_FILL_BEHIND,
                        tr("panel.flip.tool.behind"),
                        s.fill_mode == FillMode::PaintBehind,
                    ),
                    (
                        t::FLIP_FILL_UNPAINT,
                        tr("panel.flip.tool.unpaint"),
                        s.fill_mode == FillMode::Unpaint,
                    ),
                ],
                y,
            );
            aos_pares(
                c,
                &[
                    campo(ids::FLIP_GAP_NUM, tr("panel.flip.tool.gap")),
                    campo(ids::FLIP_TRAP_NUM, tr("card.flip.trap")),
                    campo(ids::FLIP_GROW_NUM, tr("card.flip.grow")),
                    campo(ids::FLIP_PRECISION_NUM, tr("panel.flip.tool.precision")),
                ],
                y,
            )
        },
    )
}

fn cartao_do_colorize(s: FlipStyleSnapshot) -> CardDef<'static> {
    cartao(
        ids::FLIP_SEC_COLORIZE,
        tr("panel.flip.colorize.colorize"),
        Some(hex(s.colorize_color)),
        move |c, y| {
            let codigo = hex(s.colorize_color);
            // Cor + Size, Trap + Bleed: a sobra de 3 números na linha da cor (lei 1).
            let y = aos_pares(
                c,
                &[
                    amostra(
                        ids::FLIP_COLORIZE_SWATCH,
                        s.colorize_color,
                        &codigo,
                        tr("panel.flip.colorize.color"),
                    ),
                    campo(ids::FLIP_SIZE_NUM, tr("card.flip.size")),
                    campo(ids::FLIP_TRAP_NUM, tr("card.flip.trap")),
                    campo(ids::FLIP_COLORIZE_BLEED_NUM, tr("card.flip.bleed")),
                ],
                y,
            );
            c.row(
                &[
                    Cell::new(Widget::Button {
                        id: ids::FLIP_COLORIZE_APPLY,
                        text: tr("panel.flip.colorize.apply"),
                        primary: false,
                    })
                    .label(tr("panel.flip.colorize.scribbles")),
                    Cell::new(Widget::Button {
                        id: ids::FLIP_COLORIZE_CLEAR,
                        text: tr("panel.flip.colorize.clear"),
                        primary: false,
                    }),
                ],
                y,
            )
        },
    )
}

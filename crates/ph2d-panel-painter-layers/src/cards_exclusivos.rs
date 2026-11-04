//! Máscara, Deform e Selection em linhas de cartão (fatia H8, `docs/CartoesUI/13_PAINTER_PLANO.md`): os três donos do
//! corpo do painel. Mesmos ids, mesmos eventos. As caixas de antes viram grupos; o que escolhe é escolha, o que age é
//! botão.

use crate::cards_linhas::{Item, Pista, botoes, escolha, fila, filete, grupo};
use crate::paint_mask::OVERLAY_COLORS;
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::paint::{frame_radius, resolve, stroke_frame};
use ph2d_editor_core::panel::cards::{CardCtx, Cell, Seg, Widget, censo, ferramenta};
use ph2d_editor_core::widget::{ColorSwatch, SwatchSize, SwatchState, paint_color_swatch};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::{ColorToken, Radius, StrokeToken};
use ph2d_tool_painter::ids as pids;
use ph2d_tool_painter::{BrushSettings, DEFORM_TEMPERAMENT_RESHAPE, DEFORM_TEMPERAMENT_TRANSFORM};

/// O sub-modo Reconstruct: Distortion, Momentum e Strength não se aplicam.
const RECONSTROI: u8 = 5;

const DEFORM_BASE: [Pista; 2] = [
    Pista {
        chave: "panel.painter_layers.deform.size",
        ids: [
            pids::PAINTER_DEFORM_SIZE_SLIDER,
            pids::PAINTER_DEFORM_SIZE_CHIP,
        ],
        le: |b| b.deform_size_norm,
        casas: crate::cards_topo::CASAS_DO_PINCEL,
    },
    Pista {
        chave: "panel.painter_layers.deform.pressure",
        ids: [
            pids::PAINTER_DEFORM_PRESSURE_SLIDER,
            pids::PAINTER_DEFORM_PRESSURE_CHIP,
        ],
        le: |b| b.deform_pressure,
        casas: crate::cards_topo::CASAS_DO_PINCEL,
    },
];
const DEFORM_CAMPO: [Pista; 3] = [
    Pista {
        chave: "panel.painter_layers.deform.distortion",
        ids: [
            pids::PAINTER_DEFORM_DISTORTION_SLIDER,
            pids::PAINTER_DEFORM_DISTORTION_CHIP,
        ],
        le: |b| b.deform_distortion,
        casas: crate::cards_topo::CASAS_DO_PINCEL,
    },
    Pista {
        chave: "panel.painter_layers.deform.momentum",
        ids: [
            pids::PAINTER_DEFORM_MOMENTUM_SLIDER,
            pids::PAINTER_DEFORM_MOMENTUM_CHIP,
        ],
        le: |b| b.deform_momentum,
        casas: crate::cards_topo::CASAS_DO_PINCEL,
    },
    Pista {
        chave: "panel.painter_layers.deform.strength",
        ids: [
            pids::PAINTER_DEFORM_STRENGTH_SLIDER,
            pids::PAINTER_DEFORM_STRENGTH_CHIP,
        ],
        le: |b| b.deform_strength,
        casas: crate::cards_topo::CASAS_DO_PINCEL,
    },
];

const SEL_LIMIAR: Pista = Pista {
    chave: "panel.painter_layers.selection.threshold",
    ids: [
        pids::PAINTER_SEL_THRESHOLD_SLIDER,
        pids::PAINTER_SEL_THRESHOLD_CHIP,
    ],
    le: |b| b.selection_threshold,
    casas: crate::cards_topo::CASAS_DO_PINCEL,
};
const SEL_ESTABILIZA: Pista = Pista {
    chave: "panel.painter_layers.selection.stabilization",
    ids: [
        pids::PAINTER_SEL_STABILIZE_SLIDER,
        pids::PAINTER_SEL_STABILIZE_CHIP,
    ],
    le: |b| b.selection_stabilizer,
    casas: crate::cards_topo::CASAS_DO_PINCEL,
};
const SEL_FIXAS: [Pista; 2] = [
    Pista {
        chave: "panel.painter_layers.selection.feather",
        ids: [
            pids::PAINTER_SEL_FEATHER_SLIDER,
            pids::PAINTER_SEL_FEATHER_CHIP,
        ],
        le: |b| b.selection_feather,
        casas: crate::cards_topo::CASAS_DO_PINCEL,
    },
    Pista {
        chave: "panel.painter_layers.selection.overlay",
        ids: [
            pids::PAINTER_SEL_OPACITY_SLIDER,
            pids::PAINTER_SEL_OPACITY_CHIP,
        ],
        le: |b| b.selection_overlay_opacity,
        casas: crate::cards_topo::CASAS_DO_PINCEL,
    },
];
const SEL_OFFSET: Pista = Pista {
    chave: "panel.painter_layers.selection.offset",
    ids: [
        pids::PAINTER_SEL_OFFSET_SLIDER,
        pids::PAINTER_SEL_OFFSET_CHIP,
    ],
    le: |b| b.selection_offset,
    casas: crate::cards_topo::CASAS_DO_PINCEL,
};

/// Com o store MUTÁVEL, antes de pintar.
pub(crate) fn prepare(store: &mut WidgetStore, b: &BrushSettings) {
    if b.is_mask {
        for id in pids::PAINTER_MASK_COLOR
            .iter()
            .chain(&pids::PAINTER_MASK_BRUSH)
            .chain(&pids::PAINTER_MASK_OP)
            .chain(std::iter::once(&pids::PAINTER_MASK_APPLY))
        {
            crate::paint::register_button(store, *id);
        }
    }
    if b.is_deform {
        DEFORM_BASE
            .iter()
            .chain(&DEFORM_CAMPO)
            .for_each(|p| p.arma(store, b));
    }
    if b.is_selection {
        for p in SEL_FIXAS
            .iter()
            .chain([&SEL_LIMIAR, &SEL_ESTABILIZA, &SEL_OFFSET])
        {
            p.arma(store, b);
        }
    }
}

fn seg(id: NodeId, chave: &str, on: bool) -> Seg<'static> {
    Seg {
        id,
        icon: None,
        text: tr(chave),
        on,
    }
}

/// As cinco cores da tinta sobre a máscara: quadrados, porque a cor não tem código a mostrar aqui.
fn cores_da_mascara(c: &mut CardCtx<'_>, escolhida: usize, y: f32) -> f32 {
    let lado = m::FIELD_H;
    let topo = c.hint(tr("panel.painter_layers.mask.overlay_color"), y) + m::FIELD_ROW_GAP;
    let tema = c.ink.theme;
    for (i, rgba) in OVERLAY_COLORS.iter().enumerate() {
        let id = pids::PAINTER_MASK_COLOR[i];
        let r = Rect::new(c.x + (lado + m::FIELD_GAP) * i as f32, topo, lado, lado);
        let amostra = ColorSwatch::new(id, "", *rgba)
            .size(SwatchSize::Sm)
            .state(SwatchState::Normal);
        paint_color_swatch(&amostra, r, c.scene, tema);
        if escolhida == i {
            stroke_frame(
                c.scene,
                r,
                frame_radius(tema, Radius::Sm.px()),
                tema,
                ph2d_tokens::visuals::Feel::Selected,
                StrokeToken::Thick.px(),
                resolve(ColorToken::Accent, tema),
            );
        }
        c.hit_index.register(id, r);
    }
    censo::frase(
        (c.x, c.w),
        topo,
        topo + lado,
        tr("panel.painter_layers.mask.overlay_color"),
    );
    topo + lado
}

pub(crate) fn mascara(c: &mut CardCtx<'_>, b: &BrushSettings, y: f32) -> f32 {
    let chaves = [
        "panel.painter_layers.mask.paint",
        "panel.painter_layers.mask.erase",
        "panel.painter_layers.mask.blur",
        "panel.painter_layers.mask.smear",
    ];
    let pinceis: Vec<Seg<'_>> = chaves
        .iter()
        .enumerate()
        .map(|(i, k)| {
            seg(
                pids::PAINTER_MASK_BRUSH[i],
                k,
                usize::from(b.mask_brush) == i,
            )
        })
        .collect();
    let y = escolha(c, tr("panel.painter_layers.mask.brushes"), &pinceis, y);
    let y = grupo(c, "panel.painter_layers.mask.modifiers", y);
    let ops = [
        "panel.painter_layers.mask.expand",
        "panel.painter_layers.mask.contract",
        "panel.painter_layers.mask.blur",
        "panel.painter_layers.mask.sharpen",
        "panel.painter_layers.mask.invert",
        "panel.painter_layers.mask.clear",
    ];
    let acoes: Vec<(NodeId, &str)> = ops
        .iter()
        .enumerate()
        .map(|(i, k)| (pids::PAINTER_MASK_OP[i], tr(k)))
        .collect();
    let (uma, outra) = acoes.split_at(acoes.len() / 2);
    let y = botoes(c, uma, y);
    let y = botoes(c, outra, y);
    let y = cores_da_mascara(c, usize::from(b.mask_overlay_color), y);
    c.row(
        &[Cell::new(Widget::Button {
            id: pids::PAINTER_MASK_APPLY,
            text: tr("panel.painter_layers.mask.apply_mask"),
            primary: true,
        })],
        y,
    )
}

fn acoes_do_deform(c: &mut CardCtx<'_>, y: f32) -> f32 {
    let y = filete(c, y);
    botoes(
        c,
        &[
            (
                pids::PAINTER_DEFORM_RESET,
                tr("panel.painter_layers.deform.reset"),
            ),
            (
                pids::PAINTER_DEFORM_APPLY,
                tr("panel.painter_layers.deform.apply"),
            ),
            (
                pids::PAINTER_DEFORM_APPLY_KEEP,
                tr("panel.painter_layers.deform.apply_and_keep"),
            ),
        ],
        y,
    )
}

fn liquify(c: &mut CardCtx<'_>, b: &BrushSettings, y: f32) -> f32 {
    let modos = [
        (
            pids::PAINTER_DEFORM_MODE_PUSH,
            "panel.painter_layers.deform.push",
        ),
        (
            pids::PAINTER_DEFORM_MODE_TWIST,
            "panel.painter_layers.deform.twist",
        ),
        (
            pids::PAINTER_DEFORM_MODE_PINCH,
            "panel.painter_layers.deform.pinch",
        ),
        (
            pids::PAINTER_DEFORM_MODE_WRINKLE,
            "panel.painter_layers.deform.wrinkle",
        ),
        (
            pids::PAINTER_DEFORM_MODE_FOLD,
            "panel.painter_layers.deform.fold",
        ),
        (
            pids::PAINTER_DEFORM_MODE_RECONSTRUCT,
            "panel.painter_layers.deform.reconstruct",
        ),
    ];
    let segs: Vec<Seg<'_>> = modos
        .iter()
        .enumerate()
        .map(|(i, (id, k))| seg(*id, k, usize::from(b.deform_mode) == i))
        .collect();
    let y = escolha(c, tr("panel.painter_layers.deform.deform_mode"), &segs, y);
    let mut itens: Vec<_> = DEFORM_BASE.iter().map(Pista::item).collect();
    if b.deform_mode != RECONSTROI {
        itens.extend(DEFORM_CAMPO.iter().map(Pista::item));
    }
    if b.deform_layer_has_relief {
        itens.push(Item::Caixa {
            id: pids::PAINTER_DEFORM_RELIEF,
            chave: "panel.painter_layers.deform.affect_relief",
            on: b.deform_affect_relief,
        });
    }
    fila(c, &itens, y)
}

pub(crate) fn deform(c: &mut CardCtx<'_>, b: &BrushSettings, y: f32) -> f32 {
    let t = b.deform_temperament;
    let temperamento = [
        seg(
            pids::PAINTER_DEFORM_TEMPERAMENT_RESHAPE,
            "panel.painter_layers.deform.liquify",
            t == DEFORM_TEMPERAMENT_RESHAPE,
        ),
        seg(
            pids::PAINTER_DEFORM_TEMPERAMENT_TRANSFORM,
            "panel.painter_layers.deform.transform",
            t == DEFORM_TEMPERAMENT_TRANSFORM,
        ),
    ];
    let y = escolha(
        c,
        tr("panel.painter_layers.deform.warp_tool"),
        &temperamento,
        y,
    );
    if t == DEFORM_TEMPERAMENT_RESHAPE {
        let y = liquify(c, b, y);
        return acoes_do_deform(c, y);
    }
    if t != DEFORM_TEMPERAMENT_TRANSFORM {
        return y;
    }
    let chaves = [
        (
            pids::PAINTER_DEFORM_TRANSFORM_MODE_UNIFORM,
            "panel.painter_layers.deform.uniform",
        ),
        (
            pids::PAINTER_DEFORM_TRANSFORM_MODE_FREE,
            "panel.painter_layers.deform.free",
        ),
        (
            pids::PAINTER_DEFORM_TRANSFORM_MODE_DISTORT,
            "panel.painter_layers.deform.distort",
        ),
        (
            pids::PAINTER_DEFORM_TRANSFORM_MODE_WARP,
            "panel.painter_layers.deform.warp",
        ),
    ];
    let modos: Vec<Seg<'_>> = chaves
        .iter()
        .enumerate()
        .map(|(i, (id, k))| seg(*id, k, usize::from(b.deform_transform_mode) == i))
        .collect();
    let y = escolha(
        c,
        tr("panel.painter_layers.deform.transform_mode"),
        &modos,
        y,
    );
    acoes_do_deform(c, y)
}

fn selecao_acoes(c: &mut CardCtx<'_>, y: f32) -> f32 {
    let y = filete(c, y);
    let y = botoes(
        c,
        &[
            (
                pids::PAINTER_SEL_ALL,
                tr("panel.painter_layers.selection.all"),
            ),
            (
                pids::PAINTER_SEL_INVERT,
                tr("panel.painter_layers.selection.invert"),
            ),
            (
                pids::PAINTER_SEL_CLEAR,
                tr("panel.painter_layers.selection.clear"),
            ),
        ],
        y,
    );
    botoes(
        c,
        &[
            (
                pids::PAINTER_SEL_LAYER_CONTENTS,
                tr("panel.painter_layers.selection.layer"),
            ),
            (
                pids::PAINTER_SEL_FILL,
                tr("panel.painter_layers.selection.fill"),
            ),
            (
                pids::PAINTER_SEL_CUT,
                tr("panel.painter_layers.selection.cut"),
            ),
            (
                pids::PAINTER_SEL_COPY,
                tr("panel.painter_layers.selection.copy"),
            ),
            (
                pids::PAINTER_SEL_PASTE,
                tr("panel.painter_layers.selection.paste"),
            ),
        ],
        y,
    )
}

pub(crate) fn selecao(c: &mut CardCtx<'_>, b: &BrushSettings, y: f32) -> f32 {
    let op = usize::from(b.selection_op);
    let ops = [
        seg(
            pids::PAINTER_SEL_OP_NEW,
            "panel.painter_layers.selection.new",
            op == 0,
        ),
        seg(
            pids::PAINTER_SEL_OP_ADD,
            "panel.painter_layers.selection.add",
            op == 1,
        ),
        seg(
            pids::PAINTER_SEL_OP_REMOVE,
            "panel.painter_layers.selection.remove",
            op == 2,
        ),
        seg(
            pids::PAINTER_SEL_OP_INTERSECT,
            "panel.painter_layers.selection.intersect",
            op == 3,
        ),
    ];
    let y = escolha(c, tr("card.painter.selection.operation"), &ops, y);
    let modo = usize::from(b.selection_mode);
    let modos = [
        seg(
            pids::PAINTER_SEL_MODE_AUTO,
            "panel.painter_layers.selection.auto",
            modo == 0,
        ),
        seg(
            pids::PAINTER_SEL_MODE_FREEHAND,
            "panel.painter_layers.selection.free",
            modo == 1,
        ),
        seg(
            pids::PAINTER_SEL_MODE_RECT,
            "panel.painter_layers.selection.rect",
            modo == 2,
        ),
        seg(
            pids::PAINTER_SEL_MODE_ELLIPSE,
            "panel.painter_layers.selection.ellipse",
            modo == 3,
        ),
        seg(
            pids::PAINTER_SEL_MODE_PEN,
            "panel.painter_layers.selection.pen",
            modo == 4,
        ),
    ];
    let y = escolha(
        c,
        tr("panel.painter_layers.selection.selection_mode"),
        &modos,
        y,
    );
    let mut itens = Vec::new();
    if modo == 0 {
        itens.push(SEL_LIMIAR.item());
    }
    if modo == 1 {
        itens.push(SEL_ESTABILIZA.item());
    }
    itens.extend(SEL_FIXAS.iter().map(Pista::item));
    let y = fila(c, &itens, y);
    let y = grupo(c, "card.painter.selection.expand_contract", y);
    let y = fila(c, &[SEL_OFFSET.item()], y);
    let y = botoes(
        c,
        &[
            (
                pids::PAINTER_SEL_OFFSET_APPLY,
                tr("panel.painter_layers.selection.apply"),
            ),
            (
                pids::PAINTER_SEL_OFFSET_APPLY_KEEP,
                tr("panel.painter_layers.selection.apply_and_keep"),
            ),
        ],
        y,
    );
    let y = filete(c, y);
    let mut y = ferramenta::caixa(
        c,
        pids::PAINTER_SEL_EDIT,
        "panel.painter_layers.selection.edit_gizmos",
        b.selection_edit,
        y,
    );
    if b.selection_edit {
        y = botoes(
            c,
            &[
                (
                    pids::PAINTER_SEL_CONVERT,
                    tr("panel.painter_layers.selection.convert_to_curve"),
                ),
                (
                    pids::PAINTER_SEL_MERGE,
                    tr("panel.painter_layers.selection.merge_curves"),
                ),
                (
                    pids::PAINTER_SEL_SIMPLIFY,
                    tr("panel.painter_layers.selection.simplify_curve"),
                ),
            ],
            y,
        );
    }
    selecao_acoes(c, y)
}

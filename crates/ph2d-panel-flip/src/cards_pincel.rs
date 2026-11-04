//! O cartão do PINCEL do Flip: cada modo mostra só o que a ponta dele usa (a doutrina de `paint_brush`).

use super::*;

/// O número com a corrente da borracha: ligada, o campo é o do pincel (os ids de sempre).
fn com_corrente(
    c: &mut CardCtx<'_>,
    chip: NodeId,
    rotulo: &str,
    link: NodeId,
    on: bool,
    y: f32,
) -> f32 {
    let corrente = [Seg {
        id: link,
        icon: Some(IconId::Link),
        text: "",
        on,
    }];
    c.row(
        &[campo(chip, rotulo), Cell::new(Widget::Bare(&corrente))],
        y,
    )
}

pub(super) fn cartao_do_pincel(s: FlipStyleSnapshot) -> CardDef<'static> {
    let titulo = match s.mode {
        FlipMode::Erase => "panel.flip.brush.eraser",
        FlipMode::Reshape => "panel.flip.brush.sculpt",
        FlipMode::Edit => "panel.flip.brush.stroke",
        _ => "panel.flip.brush.brush",
    };
    let resumo = format!("{:.0}", s.width_px);
    cartao(
        ids::FLIP_SEC_BRUSH,
        tr(titulo),
        Some(resumo),
        move |c, y| match s.mode {
            FlipMode::Erase => borracha(c, s, y),
            FlipMode::Reshape => c.row(
                &[
                    campo(ids::FLIP_SIZE_NUM, tr("card.flip.size")),
                    campo(ids::FLIP_OPACITY_NUM, tr("card.flip.strength")),
                ],
                y,
            ),
            FlipMode::Edit => aos_pares(
                c,
                &[
                    campo(ids::FLIP_SIZE_NUM, tr("card.flip.size")),
                    campo(ids::FLIP_HARDNESS_NUM, tr("panel.flip.brush.hardness")),
                    campo(ids::FLIP_OPACITY_NUM, tr("card.flip.opacity")),
                ],
                y,
            ),
            _ => pincel_de_desenho(c, s, y),
        },
    )
}

fn borracha(c: &mut CardCtx<'_>, s: FlipStyleSnapshot, y: f32) -> f32 {
    let tamanho = if s.link_size {
        ids::FLIP_SIZE_NUM
    } else {
        ids::FLIP_ERASE_SIZE_NUM
    };
    let y = com_corrente(
        c,
        tamanho,
        tr("card.flip.size"),
        t::FLIP_LINK_SIZE,
        s.link_size,
        y,
    );
    // Hard e Stroke são binárias: só o Soft doseia.
    if s.erase != EraseMode::Soft {
        return y;
    }
    let forca = if s.link_strength {
        ids::FLIP_OPACITY_NUM
    } else {
        ids::FLIP_ERASE_STRENGTH_NUM
    };
    com_corrente(
        c,
        forca,
        tr("card.flip.strength"),
        t::FLIP_LINK_STRENGTH,
        s.link_strength,
        y,
    )
}

fn pincel_de_desenho(c: &mut CardCtx<'_>, s: FlipStyleSnapshot, y: f32) -> f32 {
    let y = aos_pares(
        c,
        &[
            campo(ids::FLIP_SIZE_NUM, tr("card.flip.size")),
            campo(ids::FLIP_HARDNESS_NUM, tr("panel.flip.brush.hardness")),
            campo(ids::FLIP_OPACITY_NUM, tr("card.flip.opacity")),
            campo(ids::FLIP_SMOOTHING_NUM, tr("panel.flip.brush.smoothing")),
            campo(ids::FLIP_PRESSURE_MIN_NUM, tr("card.flip.min_width")),
            campo(ids::FLIP_PRESSURE_RESPONSE_NUM, tr("card.flip.response")),
        ],
        y,
    );
    let mut y = escolha(
        c,
        Some(tr("panel.flip.tip.tip")),
        &[
            (
                t::FLIP_TIP_LINE,
                tr("panel.flip.tip.line"),
                s.tip == StrokeTip::Continuous,
            ),
            (
                t::FLIP_TIP_DOTS,
                tr("panel.flip.tip.dots"),
                s.tip == StrokeTip::Dots,
            ),
            (
                t::FLIP_TIP_SQUARES,
                tr("panel.flip.tip.squares"),
                s.tip == StrokeTip::Squares,
            ),
        ],
        y,
    );
    if s.tip != StrokeTip::Continuous {
        y = c.row(
            &[campo(ids::FLIP_DOT_SPACING_NUM, tr("card.flip.spacing"))],
            y,
        );
    }
    let y = escolha(
        c,
        Some(tr("panel.flip.tip.cap")),
        &[
            (
                t::FLIP_CAP_ROUND,
                tr("panel.flip.tip.round"),
                s.cap == Cap::Round,
            ),
            (
                t::FLIP_CAP_FLAT,
                tr("panel.flip.tip.flat"),
                s.cap == Cap::Flat,
            ),
            (
                t::FLIP_CAP_SQUARE,
                tr("panel.flip.tip.square"),
                s.cap == Cap::Square,
            ),
        ],
        y,
    );
    caixas_aos_pares(
        c,
        &[
            (
                t::FLIP_SELF_OVERLAP,
                "panel.flip.tip.self_overlap",
                s.self_overlap,
            ),
            (t::FLIP_AIRBRUSH, "panel.flip.tip.airbrush", s.airbrush),
        ],
        y,
    )
}

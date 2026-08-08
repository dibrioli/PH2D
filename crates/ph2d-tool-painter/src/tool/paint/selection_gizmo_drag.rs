//! **O que um arrasto FAZ com a forma** — a metade transformacional do gizmo de seleção, cortada do
//! irmão [`super`] (que fica com *o que o gizmo É e como se agarra nele*) quando o arquivo cruzou o teto
//! de LOC. Módulo FILHO, e não irmão, de propósito: o `use super::*` alcança o `Frame`, os ids de alça e
//! os helpers privados sem abrir nada de novo na crate.

use super::*;

/// O SINAL de cada alça nos eixos do frame — a mesma tabela que [`Frame::handles`] usa para desenhá-las,
/// lida ao contrário. Uma alça de aresta tem `0` no eixo que ela não mexe, e é isso que faz a âncora dela
/// cair no MEIO do lado oposto sem um caso especial.
fn handle_sign(handle: u8) -> [f32; 2] {
    match handle {
        0 => [-1.0, 1.0],
        1 => [1.0, 1.0],
        2 => [1.0, -1.0],
        3 => [-1.0, -1.0],
        4 => [1.0, 0.0],
        5 => [0.0, 1.0],
        6 => [-1.0, 0.0],
        _ => [0.0, -1.0],
    }
}

pub(super) fn apply_gizmo_drag(
    initial: &SelectionShape,
    handle: u8,
    start: [f32; 2],
    pos: [f32; 2],
    tol: f32,
    mods: ph2d_editor_core::GizmoModifiers,
) -> SelectionShape {
    let Some(f) = shape_frame(initial) else {
        return initial.clone();
    };
    if handle == H_SIDES {
        return drag_sides(initial, &f, pos, tol);
    }
    if handle == H_MOVE {
        let d = [pos[0] - start[0], pos[1] - start[1]];
        return transform_shape(initial, |p| [p[0] + d[0], p[1] + d[1]], f.u, f.u);
    }
    if handle == H_ROTATE {
        let v0 = unit_or([start[0] - f.center[0], start[1] - f.center[1]], [1.0, 0.0]);
        let v1 = unit_or([pos[0] - f.center[0], pos[1] - f.center[1]], [1.0, 0.0]);
        let cos = dot(v0, v1);
        let sin = v0[0] * v1[1] - v0[1] * v1[0];
        let c = f.center;
        let nu = [f.u[0] * cos - f.u[1] * sin, f.u[0] * sin + f.u[1] * cos];
        return transform_shape(
            initial,
            move |p| {
                let r = [p[0] - c[0], p[1] - c[1]];
                [
                    c[0] + r[0] * cos - r[1] * sin,
                    c[1] + r[0] * sin + r[1] * cos,
                ]
            },
            f.u,
            nu,
        );
    }
    // Scale (handles 0..8): map the pointer onto the frame axes → new half-extents. The FREEHAND box is
    // drawn inflated (`gizmo_frame`), so subtract the margin from the pointer distance — the drag then
    // measures the TRUE extents and the inflated corner tracks the cursor exactly.
    if handle < H_SCALE_END {
        let m = if matches!(initial, SelectionShape::Freehand { .. }) {
            tol * FREEHAND_BOX_MARGIN
        } else {
            0.0
        };
        let v = f.v();
        let sgn = handle_sign(handle);
        // **A ÂNCORA.** Por default é a quina OPOSTA (ou o meio do lado oposto, numa alça de aresta): o
        // canto que a mão segura acompanha o cursor e o outro fica onde está — o que todo editor faz, e o
        // que a `GizmoModifiers` do gizmo de sprite já declara como a lei desta casa (*"Ctrl/Cmd … flip
        // Scale gizmo to center-anchor (default = opposite-corner)"*). Este gizmo escalava SEMPRE pelo
        // centro, então a tecla que o Enio pediu para escalar pelo centro não teria o que ligar (2026-08-07:
        // *"as seleções ainda não têm aquele sistema de teclas acessórias (shift e ctrl) para escalonar a
        // partir do centro e manter as proporções"*).
        //
        // Com o centro ancorado a medida sai do CENTRO e a metade é a distância; com a quina ancorada ela
        // sai da ÂNCORA e a metade é a distância dividida por dois. É a mesma frase escrita nas duas
        // réguas, e é por isso que o `m` (a margem inflada do Freehand) entra antes da divisão nas duas.
        let anchor = if mods.ctrl {
            f.center
        } else {
            [
                f.center[0] - sgn[0] * f.hx * f.u[0] - sgn[1] * f.hy * v[0],
                f.center[1] - sgn[0] * f.hx * f.u[1] - sgn[1] * f.hy * v[1],
            ]
        };
        let half = if mods.ctrl { 1.0 } else { 0.5 };
        let rel = [pos[0] - anchor[0], pos[1] - anchor[1]];
        let du = ((dot(rel, f.u).abs() - m) * half).max(MIN_AXIS_PX);
        let dv = ((dot(rel, v).abs() - m) * half).max(MIN_AXIS_PX);
        // Corners (0..3) scale both axes; edge R/L (4/6) scale hx; edge T/B (5/7) scale hy.
        let (mut nhx, mut nhy) = match handle {
            0..=3 => (du, dv),
            4 | 6 => (du, f.hy),
            _ => (f.hx, dv),
        };
        // **Shift TRAVA A PROPORÇÃO**, e só numa quina — numa aresta há um eixo só, e travar a razão ali
        // faria a alça mexer no eixo que ela não segura. O fator vem do eixo que o cursor puxou MAIS, que
        // é o que mantém a quina sob o dedo em vez de a deixar para trás.
        if mods.shift && handle <= 3 {
            let s = (nhx / f.hx.max(0.001)).max(nhy / f.hy.max(0.001));
            nhx = (f.hx * s).max(MIN_AXIS_PX);
            nhy = (f.hy * s).max(MIN_AXIS_PX);
        }
        let scaled = scale_shape(initial, &f, nhx, nhy);
        if mods.ctrl {
            return scaled; // âncora no centro ⇒ o `scale_shape`, que escala em torno dele, já basta
        }
        // O `scale_shape` escala em torno do CENTRO, então pinar a âncora é uma translação — e ela é
        // exatamente o quanto a âncora andou. Numa alça de aresta o `sgn` do outro eixo é `0` e o termo
        // some sozinho, sem um segundo caminho para manter em dia.
        let d = [
            sgn[0] * (nhx - f.hx) * f.u[0] + sgn[1] * (nhy - f.hy) * v[0],
            sgn[0] * (nhx - f.hx) * f.u[1] + sgn[1] * (nhy - f.hy) * v[1],
        ];
        return transform_shape(&scaled, |p| [p[0] + d[0], p[1] + d[1]], f.u, f.u);
    }
    initial.clone()
}

/// Rewrite a shape under a positional transform `xf` (applied to every geometric point), plus the new axis
/// `new_u` for the parametric shapes (ellipse/polygon) whose orientation is stored explicitly.
fn transform_shape(
    shape: &SelectionShape,
    xf: impl Fn([f32; 2]) -> [f32; 2],
    _old_u: [f32; 2],
    new_u: [f32; 2],
) -> SelectionShape {
    match shape {
        SelectionShape::Ellipse { rx, ry, center, .. } => SelectionShape::Ellipse {
            center: xf(*center),
            u: new_u,
            rx: *rx,
            ry: *ry,
        },
        SelectionShape::Polygon {
            rx,
            ry,
            sides,
            center,
            ..
        } => SelectionShape::Polygon {
            center: xf(*center),
            u: new_u,
            rx: *rx,
            ry: *ry,
            sides: *sides,
        },
        SelectionShape::Freehand { model, .. } => SelectionShape::Freehand {
            model: CurveModel {
                points: model.points.iter().map(|&p| xf(p)).collect(),
                handles: model.handles.iter().map(|h| [xf(h[0]), xf(h[1])]).collect(),
                kinds: model.kinds.clone(),
                selected: model.selected,
                closed: model.closed,
            },
            // The box orientation follows the shape (identity for move; rotated for rotate).
            u: new_u,
        },
        SelectionShape::Raster { .. } => shape.clone(),
    }
}

/// Scale a shape about its frame centre to new half-extents `nhx`/`nhy` (parametric → set rx/ry; freehand →
/// scale every point/handle in the frame's u/v axes).
fn scale_shape(shape: &SelectionShape, f: &Frame, nhx: f32, nhy: f32) -> SelectionShape {
    match shape {
        SelectionShape::Ellipse { center, u, .. } => SelectionShape::Ellipse {
            center: *center,
            u: *u,
            rx: nhx,
            ry: nhy,
        },
        SelectionShape::Polygon {
            center, u, sides, ..
        } => SelectionShape::Polygon {
            center: *center,
            u: *u,
            rx: nhx,
            ry: nhy,
            sides: *sides,
        },
        SelectionShape::Freehand { model, .. } => {
            let (c, u, v) = (f.center, f.u, f.v());
            let sx = nhx / f.hx.max(0.001);
            let sy = nhy / f.hy.max(0.001);
            let scale = |p: [f32; 2]| {
                let d = [p[0] - c[0], p[1] - c[1]];
                let du = dot(d, u) * sx;
                let dv = dot(d, v) * sy;
                [c[0] + du * u[0] + dv * v[0], c[1] + du * u[1] + dv * v[1]]
            };
            SelectionShape::Freehand {
                model: CurveModel {
                    points: model.points.iter().map(|&p| scale(p)).collect(),
                    handles: model
                        .handles
                        .iter()
                        .map(|h| [scale(h[0]), scale(h[1])])
                        .collect(),
                    kinds: model.kinds.clone(),
                    selected: model.selected,
                    closed: model.closed,
                },
                u: f.u, // scaling keeps the box orientation
            }
        }
        SelectionShape::Raster { .. } => shape.clone(),
    }
}

/// Drag the polygon **sides** diamond → new side count from the pointer's projection along `u`.
fn drag_sides(shape: &SelectionShape, f: &Frame, pos: [f32; 2], tol: f32) -> SelectionShape {
    let SelectionShape::Polygon {
        center, u, rx, ry, ..
    } = shape
    else {
        return shape.clone();
    };
    let rel = [pos[0] - f.center[0], pos[1] - f.center[1]];
    let proj = dot(rel, *u);
    let base = *rx + tol * SIDES_GAP;
    let step = (tol * SIDES_STEP).max(1.0);
    let raw = MIN_SIDES as f32 + ((proj - base) / step).round();
    let sides = (raw as i32).clamp(MIN_SIDES as i32, MAX_SIDES as i32) as u32;
    SelectionShape::Polygon {
        center: *center,
        u: *u,
        rx: *rx,
        ry: *ry,
        sides,
    }
}

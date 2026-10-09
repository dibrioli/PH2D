//! ② **A fila de cima do vetor** — o que cada modo põe nela (irmão do [`super`] pelo tecto de 600
//! LOC dos painéis; o corte é por LUGAR: aqui a fila e os pulldowns, lá a coluna, a ④ e a ordem).

use ph2d_a11y::NodeId;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::interaction::AreaMenu;
use ph2d_editor_core::widget::ToolRailEntry;
use ph2d_i18n::tr;
use ph2d_tool_vector::ids;
use ph2d_tool_vector::params::{VertexSel, VertexType};
use ph2d_tool_vector::{DrawMode, VectorStyleSnapshot};

pub(super) fn chip(id: NodeId, key: &str, active: bool) -> ToolRailEntry {
    let e = ToolRailEntry::compound(id, tr(key), tr(key), "");
    if active { e.active() } else { e }
}

pub(super) fn icon_chip(id: NodeId, key: &str, icon: IconId, active: bool) -> ToolRailEntry {
    let e = ToolRailEntry::icon(id, tr(key), icon);
    if active { e.active() } else { e }
}

/// ② a cor e a largura — o padrão nº 1 do Affinity (a amostra abre a cor ali mesmo).
pub(super) fn style_chips(snap: &VectorStyleSnapshot) -> Vec<ToolRailEntry> {
    vec![
        ToolRailEntry::swatch(
            ids::VECTOR_FILL_SWATCH,
            tr("panel.vector.bar.fill"),
            snap.fill,
        )
        .with_sub(tr("panel.vector.bar.sub.fill")),
        ToolRailEntry::swatch(
            ids::VECTOR_STROKE_SWATCH,
            tr("panel.vector.bar.stroke"),
            snap.stroke,
        )
        .with_sub(tr("panel.vector.bar.sub.stroke")),
        ToolRailEntry::number(
            ids::VECTOR_WIDTH_NUM,
            tr("panel.vector.bar.width"),
            tr("panel.vector.bar.sub.width"),
            None,
        ),
    ]
}

/// ② as opções do GESTO que está na mão — à frente dos chips do gizmo (prioridade).
pub(super) fn gesture_chips(
    snap: &VectorStyleSnapshot,
    sel: usize,
    verts: usize,
) -> Vec<ToolRailEntry> {
    let mut v = Vec::new();
    match snap.mode {
        DrawMode::Node => {
            use ph2d_tool_vector::params::MarqueeShape;
            v.push(icon_chip(
                ids::VECTOR_MARQUEE_BOX,
                "panel.vector.marquee.box",
                IconId::MarqueeBox,
                snap.marquee == MarqueeShape::Box,
            ));
            v.push(icon_chip(
                ids::VECTOR_MARQUEE_LASSO,
                "panel.vector.marquee.lasso",
                IconId::MarqueeLasso,
                snap.marquee == MarqueeShape::Lasso,
            ));
            if verts > 0 {
                let ty = crate::state::current_vertex_type();
                let is = |k| ty == Some(VertexSel::Uniform(k));
                v.push(ToolRailEntry::Divider);
                v.push(icon_chip(
                    ids::VECTOR_VERT_CORNER,
                    "panel.vector.vertex.corner",
                    IconId::VertexCorner,
                    is(VertexType::Corner),
                ));
                v.push(icon_chip(
                    ids::VECTOR_VERT_SMOOTH,
                    "panel.vector.vertex.smooth",
                    IconId::VertexSmooth,
                    is(VertexType::Smooth),
                ));
                v.push(icon_chip(
                    ids::VECTOR_VERT_SYMMETRIC,
                    "panel.vector.vertex.symm",
                    IconId::VertexSymmetric,
                    is(VertexType::Symmetric),
                ));
            }
            if sel > 0 {
                v.push(ToolRailEntry::Divider);
                if sel >= 2 {
                    v.push(icon_chip(
                        crate::ids::VECTOR_PATH_JOIN,
                        "panel.vector.arrange.join",
                        IconId::PathJoin,
                        false,
                    ));
                }
                let closed = crate::state::current_path_closed() == Some(true);
                v.push(icon_chip(
                    crate::ids::VECTOR_PATH_CLOSE,
                    "panel.vector.arrange.close_path",
                    IconId::PathClose,
                    closed,
                ));
                v.push(icon_chip(
                    crate::ids::VECTOR_PATH_REVERSE,
                    "panel.vector.arrange.reverse",
                    IconId::PathReverse,
                    false,
                ));
                v.push(icon_chip(
                    crate::ids::VECTOR_PATH_WELD,
                    "panel.vector.arrange.weld",
                    IconId::PathWeld,
                    false,
                ));
            }
        }
        DrawMode::Pencil => {
            use ph2d_vec_edit::pencil_width::WidthSource as Ws;
            v.push(ToolRailEntry::number(
                ids::VECTOR_PENCIL_FIDELITY_NUM,
                tr("panel.vector.pencil.fidelity"),
                tr("panel.vector.bar.sub.fidelity"),
                None,
            ));
            v.push(ToolRailEntry::number(
                ids::VECTOR_PENCIL_STABILIZER_NUM,
                tr("panel.vector.pencil.stabilizer"),
                tr("panel.vector.bar.sub.stabilizer"),
                None,
            ));
            let src = snap.pencil_width_source;
            v.push(chip(
                ids::VECTOR_PENCIL_W_UNIFORM,
                "panel.vector.pencil.width.uniform",
                src == Ws::Uniform,
            ));
            v.push(chip(
                ids::VECTOR_PENCIL_W_SPEED,
                "panel.vector.pencil.width.speed",
                src == Ws::Speed,
            ));
            v.push(chip(
                ids::VECTOR_PENCIL_W_PRESSURE,
                "panel.vector.pencil.width.pressure",
                src == Ws::Pressure,
            ));
            let quick = if snap.pencil_quickshape {
                ids::VECTOR_PENCIL_QUICKSHAPE_OFF
            } else {
                ids::VECTOR_PENCIL_QUICKSHAPE_ON
            };
            v.push(chip(quick, "panel.vector.pencil.quickshape", snap.pencil_quickshape));
        }
        DrawMode::Cut if crate::state::cut_line_exists() => {
            v.push(chip(
                crate::ids::VECTOR_CUT_APPLY,
                "panel.vector.cut.apply",
                false,
            ));
            v.push(chip(
                crate::ids::VECTOR_CUT_DISCARD,
                "panel.vector.cut.discard",
                false,
            ));
        }
        _ => {}
    }
    v
}

/// ② a SIMETRIA — opção das ferramentas que desenham (cauda: sai antes do gesto e da cor).
pub(super) fn symmetry_chips(snap: &VectorStyleSnapshot) -> Vec<ToolRailEntry> {
    use ph2d_symmetry::SymmetryKind;
    if !crate::section_scope::AUTHORS_GEOMETRY.contains(&snap.mode) && !snap.symmetry.on {
        return Vec::new();
    }
    let sym = snap.symmetry;
    let toggle = if sym.on {
        ids::VECTOR_SYM_OFF
    } else {
        ids::VECTOR_SYM_ON
    };
    let mut v = vec![chip(toggle, "panel.vector.bar.symmetry", sym.on)];
    if sym.on {
        for k in SymmetryKind::ALL {
            v.push(chip(
                ph2d_tool_vector::params::symmetry_kind_id(*k),
                k.label_key(),
                *k == sym.kind,
            ));
        }
        if sym.kind == SymmetryKind::Radial {
            v.push(ToolRailEntry::number(
                ids::VECTOR_SYM_SEGMENTS_NUM,
                tr("panel.vector.symmetry.segments"),
                tr("panel.vector.bar.sub.segments"),
                None,
            ));
        } else {
            let fuse = if sym.fuse {
                ids::VECTOR_SYM_FUSE_OFF
            } else {
                ids::VECTOR_SYM_FUSE_ON
            };
            v.push(chip(fuse, "panel.vector.symmetry.fuse", sym.fuse));
        }
        if crate::state_symmetry::symmetry_live_count() > 0 {
            v.push(chip(
                ids::VECTOR_SYM_APPLY,
                "panel.vector.symmetry.apply",
                false,
            ));
        }
    }
    v
}

/// ② os pulldowns do módulo: o ENCAIXE (sempre) e o ⚙ do CAMINHO (com seleção, no Select/Node).
pub(super) fn area_menus(mode: DrawMode, sel: usize) -> Vec<AreaMenu> {
    let toggle = |on: bool, on_id: NodeId, off_id: NodeId, key: &str| {
        let e = ToolRailEntry::icon(if on { off_id } else { on_id }, tr(key), IconId::SnapMagnet);
        if on { e.active() } else { e }
    };
    let snap_on = crate::state::current_snap();
    let mut menus = vec![AreaMenu {
        label: tr("panel.vector.bar.snap").to_owned(),
        face: tr(if snap_on {
            "panel.vector.bar.snap_on"
        } else {
            "panel.vector.bar.snap_off"
        })
        .to_owned(),
        faces: vec![
            tr("panel.vector.bar.snap_on").to_owned(),
            tr("panel.vector.bar.snap_off").to_owned(),
        ],
        rows: vec![
            toggle(
                snap_on,
                ph2d_editor_core::ids::VECTOR_SNAP_ON,
                crate::ids::VECTOR_SNAP_OFF,
                "panel.vector.bar.snap.shapes",
            ),
            toggle(
                crate::state::current_snap_path(),
                crate::ids::VECTOR_SNAP_PATH_ON,
                crate::ids::VECTOR_SNAP_PATH_OFF,
                "panel.vector.bar.snap.path",
            ),
            toggle(
                crate::state::current_snap_crossings(),
                crate::ids::VECTOR_SNAP_CROSS_ON,
                crate::ids::VECTOR_SNAP_CROSS_OFF,
                "panel.vector.bar.snap.crossings",
            ),
            toggle(
                crate::state::current_snap_guides(),
                crate::ids::VECTOR_SNAP_GUIDES_ON,
                crate::ids::VECTOR_SNAP_GUIDES_OFF,
                "panel.vector.bar.snap.guides",
            ),
        ],
    }];
    if sel > 0 && matches!(mode, DrawMode::Select | DrawMode::Node) {
        let row = |id, key: &str| ToolRailEntry::compound(id, tr(key), tr(key), "");
        let mut rows = vec![
            row(ids::VECTOR_PATH_SMOOTH, "panel.vector.arrange.smooth"),
            row(ids::VECTOR_PATH_SHARPEN, "panel.vector.arrange.sharpen"),
            row(ids::VECTOR_PATH_SIMPLIFY, "panel.vector.arrange.simplify"),
            row(ids::VECTOR_PATH_SUBDIVIDE, "panel.vector.arrange.subdivide"),
        ];
        // No Select o Fechar e o Inverter não estão na ② (são do Node): moram aqui.
        if mode == DrawMode::Select {
            rows.push(row(
                crate::ids::VECTOR_PATH_CLOSE,
                "panel.vector.arrange.close_path",
            ));
            rows.push(row(
                crate::ids::VECTOR_PATH_REVERSE,
                "panel.vector.arrange.reverse",
            ));
        }
        menus.push(AreaMenu {
            label: tr("panel.vector.bar.path").to_owned(),
            face: tr("panel.vector.bar.path").to_owned(),
            faces: Vec::new(),
            rows,
        });
    }
    menus
}

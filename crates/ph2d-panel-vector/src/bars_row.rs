//! ② **A fila de cima do vetor** — o que cada modo põe nela (irmão do [`super`] pelo tecto de 600
//! LOC dos painéis; o corte é por LUGAR: aqui o gesto, a forma e a simetria da fila, no
//! `bars_control` a cabeça dela — a barra de controle, F16 —, lá a coluna, a ④ e a ordem).

use ph2d_a11y::NodeId;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::interaction::WidgetStore;
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

/// ② os PARÂMETROS da forma em foco (F9, 11/10 — a Context Toolbar do Affinity): a secção do
/// Inspector saiu; o número exacto mora aqui, a alça no canvas. O foco é a MESMA porta do clique
/// (`shape_focus::resolved`: a forma viva, senão a armada, senão nada) e os ids são os do store
/// que a shell semeia — um campo `Choice` é um chip que cicla, no id gémeo do slot numérico.
pub(super) fn shape_chips(snap: &VectorStyleSnapshot, store: &WidgetStore) -> Vec<ToolRailEntry> {
    let Some(kind) = crate::shape_focus::resolved(snap) else {
        return Vec::new();
    };
    ph2d_tool_vector::shapes::shown_fields(kind)
        .filter_map(|i| shape_field_chip(kind, i, store))
        .collect()
}

/// O chip do campo `i` de `kind` na fila ②: um número no slot `vector_shape_field_id(i)`, ou uma
/// escolha que cicla no gémeo `_choice_id(i)`.
fn shape_field_chip(
    kind: ph2d_vec_scene::ShapeKind,
    i: usize,
    store: &WidgetStore,
) -> Option<ToolRailEntry> {
    use ph2d_tool_vector::shapes::{self, FieldUnit};
    let f = shapes::desc(kind).fields.get(i)?;
    // ⚠️ Os nomes entram pela porta da tradução — o gate `every_door_the_names_enter_by_translates_them`
    // lê esta chamada à letra.
    let name = crate::nomes_do_motor::campo(f.label);
    Some(match f.unit {
        FieldUnit::Choice(names) => {
            let cur = store
                .number_value(ids::vector_shape_field_id(i))
                .unwrap_or(0.0);
            let opcao = shapes::choice_label(kind, i, cur).unwrap_or("");
            let face = crate::nomes_do_motor::campo(opcao);
            ToolRailEntry::compound(ids::vector_shape_choice_id(i), name, face, name)
                .showing(names.iter().copied().map(crate::nomes_do_motor::campo))
        }
        _ => ToolRailEntry::number(ids::vector_shape_field_id(i), name, name, None),
    })
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
            v.push(chip(
                quick,
                "panel.vector.pencil.quickshape",
                snap.pencil_quickshape,
            ));
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

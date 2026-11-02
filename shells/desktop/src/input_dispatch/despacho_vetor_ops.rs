//! **As funções livres do vetor: booleana, vértices, duplicar, ordem, espelho, rotação, fecho e tipo de tinta**
//! — movidas VERBATIM do índice ([`super`], `line/input-dispatch`, 2026-09-13), com os caminhos
//! `crate::input_dispatch::..` preservados por re-exportação.

/// Set the selected path's fill rule (the panel's Non-Zero / Even-Odd segmented
/// row, shown only for compound paths). One undo step; no-op if unchanged.
pub(crate) fn apply_vec_fill_rule(
    scene: &mut ph2d_vec_scene::VecScene,
    pen: &ph2d_vec_edit::PenTool,
    even_odd: bool,
) {
    let rule = if even_odd {
        ph2d_vec_scene::FillRule::EvenOdd
    } else {
        ph2d_vec_scene::FillRule::NonZero
    };
    let Some(path) = pen.selected().and_then(|id| scene.path_mut(id)) else {
        return;
    };
    if path.fill_rule == rule {
        return;
    }
    path.fill_rule = rule;
}

/// Map a Vector-panel Boolean button `NodeId` to its op (`None` for any other
/// id). Pure — unit-tested; called from the render_loop drain to turn a
/// `ToolPanelEvent::Click` into a document boolean.
pub(crate) fn vec_bool_op_for_id(
    id: ph2d_editor_core::NodeId,
) -> Option<ph2d_vec_boolean::PathfinderOp> {
    use ph2d_vec_boolean::PathfinderOp as P;
    // Uma TABELA, e não uma cadeia de `else if`: o 5º comando entra numa linha, e quem esquecer a
    // linha vê o botão morto no gate de seam — em vez de o ver a cair no `None` em silêncio.
    [
        (ph2d_panel_vector::ids::VECTOR_BOOL_UNION, P::Union),
        (ph2d_panel_vector::ids::VECTOR_BOOL_SUBTRACT, P::Subtract),
        (ph2d_panel_vector::ids::VECTOR_BOOL_INTERSECT, P::Intersect),
        (ph2d_panel_vector::ids::VECTOR_BOOL_EXCLUDE, P::Exclude),
    ]
    .into_iter()
    .find(|(k, _)| *k == id)
    .map(|(_, op)| op)
}

/// Retype the Pen's SELECTED vertex (panel Vertex buttons). The undo step is the global one,
/// by diff (`App::post_frame_undo`), so it exists iff the kind changed. Free fn (mirror of
/// [`apply_vec_fill_rule`]) so the render_loop drain can call it with the destructured shell refs.
pub(crate) fn apply_vec_vertex_kind(
    scene: &mut ph2d_vec_scene::VecScene,
    pen: &mut ph2d_vec_edit::PenTool,
    kind: ph2d_vec_scene::VertexKind,
) {
    pen.set_selected_vertex_kind(scene, kind);
}

// NOTA: `apply_vec_corner_chamfer` (o toggle Chamfer da seção Vertex) foi REMOVIDO — o estilo de
// quina virou o par de ferramentas Fillet / Chamfer (o gesto de clicar-e-arrastar de
// `on_press_corner`, roteado no `on_mouse_input` acima). O SINAL do `corner_radius` agora é
// escrito pelo arrasto, não por um botão.

/// Delete the Pen's SELECTED vertex (panel "Delete Node" button / Delete key). The
/// undo step is the global one, by diff (`App::post_frame_undo`), so it exists iff
/// something was removed. Free fn (mirror of [`apply_vec_fill_rule`]) so the
/// render_loop drain can call it with destructured refs. Returns whether anything
/// was deleted.
pub(crate) fn apply_vec_delete_vertex(
    scene: &mut ph2d_vec_scene::VecScene,
    pen: &mut ph2d_vec_edit::PenTool,
) -> bool {
    pen.delete_selected_vertex(scene)
}

/// Map a Vector-panel Vertex-type button `NodeId` to its `VertexKind` (`None` for
/// any other id). Pure — unit-tested; called from the render_loop drain.
pub(crate) fn vec_vertex_kind_for_id(
    id: ph2d_editor_core::NodeId,
) -> Option<ph2d_vec_scene::VertexKind> {
    use ph2d_vec_scene::VertexKind;
    if id == ph2d_tool_vector::ids::VECTOR_VERT_CORNER {
        Some(VertexKind::Corner)
    } else if id == ph2d_tool_vector::ids::VECTOR_VERT_SMOOTH {
        Some(VertexKind::Smooth)
    } else if id == ph2d_tool_vector::ids::VECTOR_VERT_SYMMETRIC {
        Some(VertexKind::Symmetric)
    } else {
        None
    }
}

/// **Quanto vale, em MUNDO, um deslocamento de `px` pixels de tela** — a porta única.
///
/// ⚠️ Porta única e não conveniência: quem coloca uma cópia (paste · Ctrl+D · o **Place** de
/// instância) faz a MESMA pergunta, e uma segunda resposta é uma folga que muda de tamanho
/// conforme o botão. Ela já divergiu uma vez — ver [`crate::vec_component_edit::cascade_offset`].
///
/// O sinal de `y` vem do afim da câmara, nunca de um palpite: em tela `+y` desce, e é a conversão
/// que sabe se isso é `+` ou `−` no mundo.
pub(crate) fn screen_offset_world(
    camera: &ph2d_render::Camera2d,
    win: ph2d_host::WindowSize,
    px: f64,
) -> (f64, f64) {
    let base = camera.screen_to_world((0.0, 0.0), win);
    let moved = camera.screen_to_world((px as f32, px as f32), win);
    (f64::from(moved[0] - base[0]), f64::from(moved[1] - base[1]))
}

/// Duplicate the SELECTED path (panel "Duplicate" button), offsetting the clone
/// by `(dx, dy)` world-units so it's visible, and select the copy. Records ONE
/// undo step iff a path was cloned. Free fn (mirror of [`apply_vec_fill_rule`]) so
/// the render_loop drain can call it with the destructured shell refs.
pub(crate) fn apply_vec_duplicate(
    scene: &mut ph2d_vec_scene::VecScene,
    pen: &mut ph2d_vec_edit::PenTool,
    dx: f64,
    dy: f64,
) {
    let sel = pen.selected_paths().to_vec();
    if sel.is_empty() {
        eprintln!("[ph2d-vec] duplicate: nenhum path selecionado");
        return;
    }
    duplicate_vec_paths(scene, pen, &sel, dx, dy);
}

/// **A porta de DUPLICAR uma forma** — ela recebe *quais* paths, e quem pergunta responde isso de
/// maneiras diferentes: o botão Duplicate do painel dá a seleção da caneta, e a row **Duplicate**
/// da Hierarchy dá o path da linha clicada com o botão direito.
///
/// ⚠️ Ela é UMA porque as duas rotas têm de produzir a mesma coisa. Uma segunda implementação
/// para a Hierarchy divergiria no primeiro detalhe que só uma delas aprendesse — a estrutura de
/// grupo, o passo de undo, ou qual cópia fica selecionada no fim.
///
/// Duplicar **É** copiar-e-colar: um caminho só, então a estrutura de grupo vem junto.
/// Grava **UM** passo de undo sse alguma cópia nasceu, e devolve se nasceu.
pub(crate) fn duplicate_vec_paths(
    scene: &mut ph2d_vec_scene::VecScene,
    pen: &mut ph2d_vec_edit::PenTool,
    ids: &[ph2d_vec_scene::VecPathId],
    dx: f64,
    dy: f64,
) -> bool {
    if ids.is_empty() {
        return false;
    }
    let clip = scene.copy_paths(ids);
    let new_ids = scene.paste_clip(&clip, dx, dy);
    if new_ids.is_empty() {
        return false;
    }
    pen.select_many(&new_ids);
    eprintln!("[ph2d-vec] duplicate: {} path(s)", new_ids.len());
    true
}

/// Map a Vector-panel Arrange z-order button `NodeId` to its [`ph2d_vec_scene::ZOrder`]
/// (`None` for any other id, incl. Duplicate). Pure — unit-tested; called from
/// the render_loop drain.
pub(crate) fn vec_reorder_for_id(id: ph2d_editor_core::NodeId) -> Option<ph2d_vec_scene::ZOrder> {
    use ph2d_vec_scene::ZOrder;
    if id == ph2d_tool_vector::ids::VECTOR_ARRANGE_TO_BACK {
        Some(ZOrder::ToBack)
    } else if id == ph2d_tool_vector::ids::VECTOR_ARRANGE_BACKWARD {
        Some(ZOrder::Lower)
    } else if id == ph2d_tool_vector::ids::VECTOR_ARRANGE_FORWARD {
        Some(ZOrder::Raise)
    } else if id == ph2d_tool_vector::ids::VECTOR_ARRANGE_TO_FRONT {
        Some(ZOrder::ToFront)
    } else {
        None
    }
}

/// Mirror the SELECTED path (panel Arrange Flip buttons), recording ONE undo step
/// iff it flipped. Free fn (mirror of [`apply_vec_fill_rule`]).
pub(crate) fn apply_vec_flip(
    scene: &mut ph2d_vec_scene::VecScene,
    pen: &ph2d_vec_edit::PenTool,
    axis: ph2d_vec_scene::FlipAxis,
) {
    let Some(sel) = pen.selected() else {
        eprintln!("[ph2d-vec] flip: nenhum path selecionado");
        return;
    };
    scene.flip_path(sel, axis);
}

/// Map a Vector-panel Arrange Flip button `NodeId` to its [`ph2d_vec_scene::FlipAxis`]
/// (`None` for any other id). Pure — unit-tested; called from the render_loop drain.
pub(crate) fn vec_flip_for_id(id: ph2d_editor_core::NodeId) -> Option<ph2d_vec_scene::FlipAxis> {
    use ph2d_vec_scene::FlipAxis;
    if id == ph2d_tool_vector::ids::VECTOR_ARRANGE_FLIP_H {
        Some(FlipAxis::Horizontal)
    } else if id == ph2d_tool_vector::ids::VECTOR_ARRANGE_FLIP_V {
        Some(FlipAxis::Vertical)
    } else {
        None
    }
}

/// Rotate the SELECTED path 90° (panel Arrange Rotate buttons), recording ONE undo
/// step iff it rotated. Free fn (mirror of [`apply_vec_fill_rule`]).
pub(crate) fn apply_vec_rotate(
    scene: &mut ph2d_vec_scene::VecScene,
    pen: &ph2d_vec_edit::PenTool,
    dir: ph2d_vec_scene::Rotate90,
) {
    let Some(sel) = pen.selected() else {
        eprintln!("[ph2d-vec] rotate: nenhum path selecionado");
        return;
    };
    scene.rotate_path(sel, dir);
}

/// Map a Vector-panel Arrange Rotate button `NodeId` to its [`ph2d_vec_scene::Rotate90`]
/// (`None` for any other id). Pure — unit-tested; called from the render_loop drain.
pub(crate) fn vec_rotate_for_id(id: ph2d_editor_core::NodeId) -> Option<ph2d_vec_scene::Rotate90> {
    use ph2d_vec_scene::Rotate90;
    if id == ph2d_tool_vector::ids::VECTOR_ARRANGE_ROTATE_CW {
        Some(Rotate90::Cw)
    } else if id == ph2d_tool_vector::ids::VECTOR_ARRANGE_ROTATE_CCW {
        Some(Rotate90::Ccw)
    } else {
        None
    }
}

/// Toggle the SELECTED path between closed (loop) and open (ribbon) — panel
/// Close/Open button — recording ONE undo step iff it flipped. Free fn (mirror of
/// [`apply_vec_flip`]).
pub(crate) fn apply_vec_toggle_closed(
    scene: &mut ph2d_vec_scene::VecScene,
    pen: &mut ph2d_vec_edit::PenTool,
) {
    let Some(sel) = pen.selected() else {
        eprintln!("[ph2d-vec] close-toggle: nenhum path selecionado");
        return;
    };
    let Some(cur) = scene.paths().iter().find(|p| p.id == sel).map(|p| p.closed) else {
        return;
    };
    // ⚠️ **Fechar passa pela porta que SOLDA** (`close_path`, W4): antes isto era um
    // `set_path_closed(true)` cru, então fechar um laço cujas pontas o artista tinha acabado de
    // encostar deixava DOIS vértices sobrepostos no mesmo lugar — invisível no desenho e presente
    // em todo Delete/Average/Simplify seguinte. Abrir continua a ser só o flag: não há nada a
    // soldar ao abrir, e um `close_path(false)` seria uma porta que não existe.
    let did = if cur {
        scene.set_path_closed(sel, false)
    } else {
        scene.close_path(sel, ph2d_vec_edit::WELD_TOL)
    };
    if did {
        // Closing a never-filled path seeds its fill from the current Style — so it
        // paints IMMEDIATELY, matching the pen's auto-close (click the start point).
        // An existing fill is preserved across open→close cycles.
        if !cur
            && let Some(path) = scene.path_mut(sel)
            && path.base_fill().is_none()
        {
            path.fill = Some(ph2d_vec_scene::Paint::solid(pen.style().fill));
        }
        // A costura mudou de sítio (e, num fecho soldado, um vértice inteiro sumiu), então todo
        // índice plano guardado descreve outro nó — ou nó nenhum.
        pen.select(Some(sel));
    }
}

/// Fill kind a Vector-panel Fill-type button targets on the selected path.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum VecFillKind {
    Solid,
    Linear,
    Radial,
    MultiPoint,
    /// Padrão de textura (plano 33).
    Pattern,
}

/// Map a Fill-type button `NodeId` to its [`VecFillKind`] (`None` otherwise).
pub(crate) fn vec_fill_kind_for_id(id: ph2d_editor_core::NodeId) -> Option<VecFillKind> {
    if id == ph2d_tool_vector::ids::VECTOR_FILL_KIND_SOLID {
        Some(VecFillKind::Solid)
    } else if id == ph2d_tool_vector::ids::VECTOR_FILL_KIND_LINEAR {
        Some(VecFillKind::Linear)
    } else if id == ph2d_tool_vector::ids::VECTOR_FILL_KIND_RADIAL {
        Some(VecFillKind::Radial)
    } else if id == ph2d_tool_vector::ids::VECTOR_FILL_KIND_MULTI {
        Some(VecFillKind::MultiPoint)
    } else if id == ph2d_tool_vector::ids::VECTOR_FILL_KIND_PATTERN {
        Some(VecFillKind::Pattern)
    } else {
        None
    }
}

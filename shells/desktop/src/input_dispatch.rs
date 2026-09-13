// Tecto de LOC NUMERADO em `tests/it/file_loc_caps.rs` — Onda 2C multi-select dispatch + hit_map routing +
// click-vs-drag + group-translate snapshot capture grew this file
// past the HR-18 600-LOC cap (currently ~900 LOC). The MouseInput
// Down/Up arms are the bulk; the natural decomposition is to move
// each Down sub-path (modifier override / pivot tool / gizmo handle /
// canvas pick) and the Up resolver into siblings under
// `input_dispatch/`, parallel to the existing eyedropper / gizmo_drag
// / keyboard / protect_brush splits. That refactor lands as a
// follow-up to Onda 2 once the gizmo polish is locked.
//! Window-event dispatch — one method per `WindowEvent` variant.
//!
//! PR 9b of `docs/Migracao/2026-05-convention-by-discovery.md`:
//! `window_event()` in `main.rs` used to inline ~700 LOC across 13
//! `WindowEvent` arms, with single arms (CursorMoved 166 LOC,
//! MouseInput 325 LOC, KeyboardInput 83 LOC) violating HR-18's 200-LOC
//! per-function cap. This module hosts each arm as a `pub(crate) fn
//! on_<arm>(&mut self, …)` method on `App` — bodies are verbatim
//! former arms (no behaviour change), so smoke parity is
//! byte-for-byte.
//!
//! `window_event()` in `main.rs` becomes a 13-line dispatch table.
//! Adding a new arm: one method here + one line in the table.
//!
//! Rust allows `impl App` to be split across files within the same
//! crate as long as both files are reachable via `mod`. `App` is
//! `private` to `main.rs`, but submodules see their parent's private
//! items — so this `impl` block compiles without exposing any field
//! visibility upstream.

// ⭐ **W2: os ganchos de entrada da janela 3D são um trait de extensão sobre o `AppHost`.**
// Os sítios de chamada abaixo ficaram **byte a byte iguais** — o que mudou foi só esta linha.
use ph2d_app_field3d::input::Field3dInput;

use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, MouseButton, MouseScrollDelta};
use winit::event_loop::ActiveEventLoop;

use ph2d_editor_core::Toast;
use ph2d_host::{
    CloseAction, HostHandler, Lifecycle, PlatformHost, PointerEvent, PointerKind, PointerSource,
    WindowSize,
};

use crate::App;
use crate::Transform;
use crate::forwarding::{
    cursor_over_hero_panel, forward_blur_to_hero, forward_text_to_hero, forward_to_hero,
    forward_wheel_to_hero, resolve_live_entry,
};
use ph2d_tool_vector::params::MarqueeShape;

// `impl App` is split across sibling modules (see the eyedropper /
// keyboard handlers) to keep this file under the HR-18 LOC cap.
mod eyedropper;
pub(crate) mod fill_drag;
mod gizmo_drag;
mod keyboard;
/// **A escuta do Input Map no topo do teclado** — irmão de [`keyboard`], cortado por teto de LOC.
mod keyboard_bind_capture;

/// **As teclas que ENCERRAM um gesto em curso** (Esc cancela, Enter confirma) — irmão do
/// `keyboard`, cortado dele pelo cap de LOC. A ORDEM entre elas é a lei, e é por isso que
/// viajam juntas em vez de por dono.
mod keyboard_escapes;
/// ⭐ As teclas do modelador 3D, numa porta só — ver [`keyboard_field3d`].
mod keyboard_field3d;
/// ⭐⭐⭐ **Que MODO é dono do teclado neste quadro** — irmão do `keyboard`, cortado dele pelo cap
/// de LOC. *Um modo em curso é dono da entrada dele.*
mod keyboard_modal;
mod keyboard_palette; // as teclas do palette de nos -- MODAL, ver o doc do modulo

/// O gizmo do botão primário (modificador, alvo, pivô, âncoras, alça) — ramos do `on_mouse_input`.
mod despacho_clique_gizmo;
/// O largar do botão primário (borracha, colapso da multi-seleção, fim do arrasto) — ramos do `on_mouse_input`.
mod despacho_clique_largar;
/// O pick de canvas do botão primário (hits, ciclo, seleção, arrasto) — ramos do `on_mouse_input`.
mod despacho_clique_pick;
/// Os reclamantes do fim do clique (painter, modais, pan, barra lateral) — ramos do `on_mouse_input`.
mod despacho_clique_reclamantes;
/// A ferramenta vetorial no clique (o guarda do ADR-0112, o Shift, o direito) — ramos do `on_mouse_input`.
mod despacho_clique_vetor;
/// **Os acordes de ARQUIVO** — irmão do `keyboard`, cortado dele pelo cap de LOC.
mod keyboard_files;
mod keyboard_hierarchy; // Delete/Ctrl+D sobre a linha da Hierarquia -- ver o doc do modulo
mod keyboard_painter; // a cadeia do Delete do Painter: ancora -> figura -> falloff
mod keyboard_timeline;
pub(crate) mod painter_canvas_input;
mod painter_canvas_mods;
/// ⭐ O menu de alça de um ponto da CURVA no canvas — irmão do `painter_canvas_input` pelo teto
/// de LOC, cortado por assunto.
mod painter_curve_input;
pub(crate) mod painter_falloff_input;
mod painter_grid_erase; // os modificadores que o CanvasPointer nao carrega
pub(crate) mod protect_brush;

/// Deslocamento diagonal de um paste/duplicate, em pixels de tela (o zoom converte
/// para world) — a cópia não nasce exatamente sob o original.
///
/// ⚠️ **Em pixels de TELA, e é isso que o torna a resposta certa a *"onde nasce uma cópia?"***:
/// ele lê o mesmo em qualquer zoom. O `Place` de instância (plano UI/UX W5) nasceu com um número
/// próprio em unidades de MUNDO, e a conta que isso deu está escrita no
/// [`crate::vec_component_edit::cascade_offset`].
pub(crate) const PASTE_OFFSET_PX: f64 = 12.0;

/// O raio de acerto de uma alça de canvas, em **pixels de tela**. O MESMO número que as
/// ferramentas de quina usam para agarrar um vértice — uma alça é uma alça, e dois raios
/// diferentes fariam o artista aprender duas mãos.
const HANDLE_HIT_PX: f64 = 12.0;

/// The z-ordered (back → front) indices of the closed paths in the pen's OBJECT
/// selection. Boolean and Make Compound both need this: the back-most is the base
/// and the front-most donates the style (Illustrator's Pathfinder).
fn selected_closed_z(scene: &ph2d_vec_scene::VecScene, pen: &ph2d_vec_edit::PenTool) -> Vec<usize> {
    let mut zs: Vec<usize> = pen
        .selected_paths()
        .iter()
        .filter_map(|id| scene.paths().iter().position(|p| p.id == *id && p.closed))
        .collect();
    zs.sort_unstable();
    zs.dedup();
    zs
}

/// ADR-0108 Fase 1: apply a boolean `op` to the SELECTED closed regions of `scene`
/// (destructive — consumes the operands, inserts the result where the base sat,
/// records ONE undo step + selects the result). N-ary: `Subtract` keeps the
/// back-most and removes every path above it. A free fn so both the U/I/D hotkeys
/// ([`App::vec_boolean`]) and the panel's Boolean buttons (the render_loop drain,
/// where `AppGfx` is already destructured and the method isn't callable) can
/// invoke it with the decomposed refs. Logs + no-ops on < 2 selected closed
/// regions / empty result.
pub(crate) fn apply_vec_boolean(
    scene: &mut ph2d_vec_scene::VecScene,
    pen: &mut ph2d_vec_edit::PenTool,
    xforms: &ph2d_vec_scene::VecXforms,
    op: ph2d_vec_boolean::PathfinderOp,
) {
    let zs = selected_closed_z(scene, pen);
    if zs.len() < 2 {
        eprintln!("[ph2d-vec] boolean: selecione >= 2 regioes FECHADAS (Shift+clique)");
        return;
    }
    // ADR-0111: os operandos podem ter poses diferentes, e um resultado só vive num
    // frame. Assa cada um no MUNDO; o path novo nasce world-space e a entidade dele
    // nasce na identidade — a forma aparece exatamente onde as originais estavam.
    let operands: Vec<ph2d_vec_scene::VecPath> = zs
        .iter()
        .map(|&z| {
            let mut p = scene.paths()[z].clone();
            let x = ph2d_vec_scene::xform_of(xforms, p.id);
            ph2d_vec_scene::bake_xform(&mut p, &x);
            p
        })
        .collect();
    let refs: Vec<&ph2d_vec_scene::VecPath> = operands.iter().collect();
    // ⚠️ **`Ok(vazio)` e `Err` sao coisas DIFERENTES, e ate' a W5 eram indistinguiveis:** o motor
    // engolia a falha e o artista via o mesmo nada nos dois casos. A mensagem separa-os.
    let results = match ph2d_vec_boolean::pathfinder(&refs, op) {
        Ok(r) if r.is_empty() => {
            eprintln!("[ph2d-vec] pathfinder {op:?}: sem resultado (as formas nao se cruzam?)");
            return;
        }
        Ok(r) => r,
        Err(e) => {
            eprintln!("[ph2d-vec] pathfinder {op:?}: o motor recusou -- {e}");
            return;
        }
    };
    // A base é a de trás: o resultado ocupa a fatia de z dela (não salta pro topo).
    // Os operandos removidos estão todos em z >= `at`, então o índice segue válido.
    let at = zs[0];
    for p in &operands {
        scene.remove_path(p.id);
    }
    let new_ids: Vec<u64> = results
        .into_iter()
        .enumerate()
        .map(|(k, r)| scene.insert_path(at + k, r))
        .collect();
    pen.select_many(&new_ids);
    eprintln!(
        "[ph2d-vec] pathfinder {op:?}: ok ({} path[s])",
        new_ids.len()
    );
}

/// Make / Release Compound sobre a seleção. **Make** funde os paths fechados
/// selecionados num só (`EvenOdd` ⇒ um contorno dentro de outro vira buraco na
/// hora); **Release** devolve cada subpath do(s) selecionado(s) a path próprio.
/// Um passo de undo; re-seleciona o resultado. Free fn pelo mesmo motivo de
/// [`apply_vec_boolean`].
pub(crate) fn apply_vec_compound(
    scene: &mut ph2d_vec_scene::VecScene,
    pen: &mut ph2d_vec_edit::PenTool,
    make: bool,
) {
    if make {
        let ids: Vec<u64> = selected_closed_z(scene, pen)
            .iter()
            .map(|&z| scene.paths()[z].id)
            .collect();
        let Some(base) = scene.make_compound(&ids) else {
            eprintln!("[ph2d-vec] compound: selecione >= 2 regioes FECHADAS");
            return;
        };
        pen.select(Some(base));
        return;
    }
    let selected: Vec<u64> = pen.selected_paths().to_vec();
    let mut all: Vec<u64> = Vec::new();
    for id in &selected {
        let freed = scene.release_compound(*id);
        if !freed.is_empty() {
            all.push(*id);
            all.extend(freed);
        }
    }
    if all.is_empty() {
        eprintln!("[ph2d-vec] release: a selecao nao tem compound path");
        return;
    }
    pen.select_many(&all);
}

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
    // Uma TABELA, e não uma cadeia de `else if`: o 9º comando entra numa linha, e quem esquecer a
    // linha vê o botão morto no gate de seam — em vez de o ver a cair no `None` em silêncio.
    [
        (ph2d_panel_vector::ids::VECTOR_BOOL_UNION, P::Union),
        (ph2d_panel_vector::ids::VECTOR_BOOL_SUBTRACT, P::Subtract),
        (ph2d_panel_vector::ids::VECTOR_BOOL_INTERSECT, P::Intersect),
        (ph2d_panel_vector::ids::VECTOR_BOOL_EXCLUDE, P::Exclude),
        (ph2d_tool_vector::ids::VECTOR_BOOL_MINUS_BACK, P::MinusBack),
        (ph2d_tool_vector::ids::VECTOR_BOOL_TRIM, P::Trim),
        (ph2d_tool_vector::ids::VECTOR_BOOL_CROP, P::Crop),
        (ph2d_tool_vector::ids::VECTOR_BOOL_MERGE, P::Merge),
    ]
    .into_iter()
    .find(|(k, _)| *k == id)
    .map(|(_, op)| op)
}

/// Retype the Pen's SELECTED vertex (panel Vertex buttons). The undo step is the global one,
/// by diff (`App::post_frame_undo`), so it exists iff the kind changed. Free fn (mirror of
/// [`apply_vec_boolean`]) so the render_loop drain can call it with the destructured shell refs.
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
/// something was removed. Free fn (mirror of [`apply_vec_boolean`]) so the
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
/// undo step iff a path was cloned. Free fn (mirror of [`apply_vec_boolean`]) so
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
/// iff it flipped. Free fn (mirror of [`apply_vec_boolean`]).
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
/// step iff it rotated. Free fn (mirror of [`apply_vec_boolean`]).
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
            && path.fill.is_none()
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

/// Component-wise average of two colours (alpha too).
fn avg_color(a: ph2d_vec_scene::Rgba8, b: ph2d_vec_scene::Rgba8) -> ph2d_vec_scene::Rgba8 {
    let m = |x: u8, y: u8| ((u16::from(x) + u16::from(y)) / 2) as u8;
    ph2d_vec_scene::Rgba8::new(m(a.r, b.r), m(a.g, b.g), m(a.b, b.b), m(a.a, b.a))
}

/// Multi-point set to use when switching to a freeform fill: reuse existing points,
/// else seed 3 spread points `[fill, contrast, average]` across the bbox `(lo,hi)`.
fn gradient_points_from(
    fill: &Option<ph2d_vec_scene::Paint>,
    lo: [f64; 2],
    hi: [f64; 2],
) -> Vec<ph2d_vec_scene::GradientPoint> {
    use ph2d_vec_scene::{GradientPoint, Paint};
    if let Some(Paint::MultiPoint { points }) = fill
        && !points.is_empty()
    {
        return points.clone();
    }
    let base = fill
        .as_ref()
        .map_or(ph2d_vec_scene::Rgba8::new(255, 255, 255, 255), |p| {
            p.primary_color()
        });
    let contrast = contrast_color(base);
    let (w, h) = (hi[0] - lo[0], hi[1] - lo[1]);
    let at = |fx: f64, fy: f64| [lo[0] + w * fx, lo[1] + h * fy];
    vec![
        GradientPoint::new(at(0.25, 0.25), base, 1.0),
        GradientPoint::new(at(0.75, 0.75), contrast, 1.0),
        GradientPoint::new(at(0.75, 0.25), avg_color(base, contrast), 1.0),
    ]
}

/// A luminance-opposite (black/white, alpha preserved) — the second stop seeded
/// when a solid fill first becomes a gradient, so the ramp is visibly a gradient.
fn contrast_color(c: ph2d_vec_scene::Rgba8) -> ph2d_vec_scene::Rgba8 {
    let lum = 0.2126 * f64::from(c.r) + 0.7152 * f64::from(c.g) + 0.0722 * f64::from(c.b);
    if lum > 128.0 {
        ph2d_vec_scene::Rgba8::new(0, 0, 0, c.a)
    } else {
        ph2d_vec_scene::Rgba8::new(255, 255, 255, c.a)
    }
}

/// Gradient stops to use when switching to a gradient: reuse the existing gradient's
/// stops (Linear↔Radial keep them), else seed a 2-stop ramp `[fill → contrast]`.
fn gradient_stops_from(fill: &Option<ph2d_vec_scene::Paint>) -> Vec<ph2d_vec_scene::GradientStop> {
    use ph2d_vec_scene::{GradientStop, Paint};
    match fill {
        Some(Paint::Linear { stops, .. }) | Some(Paint::Radial { stops, .. })
            if stops.len() >= 2 =>
        {
            stops.clone()
        }
        _ => {
            let base = fill
                .as_ref()
                .map_or(ph2d_vec_scene::Rgba8::new(255, 255, 255, 255), |p| {
                    p.primary_color()
                });
            vec![
                GradientStop::new(0.0, base),
                GradientStop::new(1.0, contrast_color(base)),
            ]
        }
    }
}

/// Linear ramp endpoints spanning the bbox `(lo,hi)` along `degrees` (0° = →),
/// centered on the bbox — the world-space geometry a linear gradient stores.
fn linear_span(lo: [f64; 2], hi: [f64; 2], degrees: f64) -> ([f64; 2], [f64; 2]) {
    let (cx, cy) = ((lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5);
    let (w, h) = (hi[0] - lo[0], hi[1] - lo[1]);
    let r = degrees.to_radians();
    let (dx, dy) = (r.cos(), r.sin());
    let reach = 0.5 * ((w * dx).abs() + (h * dy).abs());
    (
        [cx - dx * reach, cy - dy * reach],
        [cx + dx * reach, cy + dy * reach],
    )
}

/// Switch the SELECTED path's fill kind (Solid/Linear/Radial), preserving colour(s)
/// and existing gradient geometry when the kind is unchanged; when entering a
/// gradient from Solid/other, the geometry is seeded to fit the path's bbox. One
/// undo step iff it changed.
/// ⚠️ **`pattern` é a FONTE já resolvida** para o caso `Pattern`, e vem de fora de propósito: ela
/// pode exigir um diálogo de ficheiro, que congela o laço e por isso pertence à shell (a porta
/// `ph2d_app_host::modal`), não a esta função pura de documento.
///
/// ⚠️ **`None` com `kind == Pattern` é DESISTÊNCIA e não muda nada** — o artista fechou o diálogo,
/// e apagar o gradiente dele por isso seria o pior dos dois mundos.
pub(crate) fn apply_vec_set_fill_kind(
    scene: &mut ph2d_vec_scene::VecScene,
    pen: &ph2d_vec_edit::PenTool,
    kind: VecFillKind,
    pattern: Option<(ph2d_vec_scene::PatternSource, [f64; 2], [f64; 2])>,
) {
    use ph2d_vec_scene::Paint;
    let Some(sel) = pen.selected() else {
        eprintln!("[ph2d-vec] fill-kind: nenhum path selecionado");
        return;
    };
    let Some(cur) = scene
        .paths()
        .iter()
        .find(|p| p.id == sel)
        .map(|p| p.fill.clone())
    else {
        return;
    };
    let (lo, hi) = scene.path_bbox(sel).unwrap_or(([0.0, 0.0], [1.0, 1.0]));
    let (cx, cy) = ((lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5);
    let new_fill = match kind {
        VecFillKind::Solid => Paint::Solid(
            cur.as_ref()
                .map_or(ph2d_vec_scene::Rgba8::new(255, 255, 255, 255), |p| {
                    p.primary_color()
                }),
        ),
        // Already this kind → keep its geometry; else seed to fit the bbox.
        VecFillKind::Linear => match &cur {
            Some(p @ Paint::Linear { .. }) => p.clone(),
            _ => {
                let (start, end) = linear_span(lo, hi, 0.0);
                Paint::Linear {
                    stops: gradient_stops_from(&cur),
                    start,
                    end,
                }
            }
        },
        VecFillKind::Radial => match &cur {
            Some(p @ Paint::Radial { .. }) => p.clone(),
            _ => Paint::Radial {
                stops: gradient_stops_from(&cur),
                center: [cx, cy],
                radius: 0.5 * (hi[0] - lo[0]).hypot(hi[1] - lo[1]),
            },
        },
        VecFillKind::MultiPoint => match &cur {
            Some(p @ Paint::MultiPoint { .. }) => p.clone(),
            _ => Paint::MultiPoint {
                points: gradient_points_from(&cur, lo, hi),
            },
        },
        VecFillKind::Pattern => match (&cur, pattern) {
            // Já é padrão: preserva a lei inteira (trocar de chip e voltar não perde a arte, nem o
            // reticulado, nem a colocação).
            (Some(p @ Paint::Pattern(_)), _) => p.clone(),
            (_, Some((source, size, origin))) => {
                let mut f = ph2d_vec_scene::PatternFill::new(
                    source,
                    size,
                    crate::texture_pattern_pick::fallback_of(cur.as_ref()),
                );
                // ⛔ O canto é o da FORMA, não a origem do mundo (ver `default_placement`).
                f.origin = origin;
                Paint::Pattern(Box::new(f))
            }
            // ⚠️ Desistiu do diálogo: NÃO mexe no preenchimento.
            (_, None) => return,
        },
    };
    if cur.as_ref() == Some(&new_fill) {
        return;
    }
    if let Some(path) = scene.path_mut(sel) {
        path.fill = Some(new_fill);
    }
}

/// Set the SELECTED path's Linear-gradient angle (degrees; from the Angle slider's
/// `track·360`) by re-fitting the ramp endpoints across the bbox at that angle.
/// No-op unless the fill is Linear. One undo step iff it changed.
pub(crate) fn apply_vec_set_grad_angle(
    scene: &mut ph2d_vec_scene::VecScene,
    pen: &ph2d_vec_edit::PenTool,
    degrees: f64,
) {
    use ph2d_vec_scene::Paint;
    let Some(sel) = pen.selected() else {
        return;
    };
    let is_linear = scene
        .paths()
        .iter()
        .find(|p| p.id == sel)
        .is_some_and(|p| matches!(p.fill, Some(Paint::Linear { .. })));
    if !is_linear {
        return;
    }
    let (lo, hi) = scene.path_bbox(sel).unwrap_or(([0.0, 0.0], [1.0, 1.0]));
    let (start, end) = linear_span(lo, hi, degrees);
    if let Some(Paint::Linear {
        start: s, end: e, ..
    }) = scene.path_mut(sel).and_then(|p| p.fill.as_mut())
    {
        if *s == start && *e == end {
            return;
        }
        *s = start;
        *e = end;
    }
}

/// Add a multi-point gradient point at the selected path's bbox center (colour =
/// the first existing point). No-op unless the fill is MultiPoint. One undo step.
pub(crate) fn apply_vec_grad_add_point(
    scene: &mut ph2d_vec_scene::VecScene,
    pen: &ph2d_vec_edit::PenTool,
) {
    use ph2d_vec_scene::{GradientPoint, Paint};
    let Some(sel) = pen.selected() else {
        return;
    };
    let Some((lo, hi)) = scene.path_bbox(sel) else {
        return;
    };
    let center = [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5];
    if let Some(Paint::MultiPoint { points }) = scene.path_mut(sel).and_then(|p| p.fill.as_mut()) {
        let col = points
            .first()
            .map_or(ph2d_vec_scene::Rgba8::new(255, 255, 255, 255), |p| p.color);
        points.push(GradientPoint::new(center, col, 1.0));
    }
}

/// Remove a multi-point gradient point (`selected`, else the last), keeping at
/// least one. Returns the new selection (`None`). One undo step iff it removed.
pub(crate) fn apply_vec_grad_remove_point(
    scene: &mut ph2d_vec_scene::VecScene,
    pen: &ph2d_vec_edit::PenTool,
    selected: Option<usize>,
) -> Option<usize> {
    use ph2d_vec_scene::Paint;
    let Some(sel) = pen.selected() else {
        return selected;
    };
    if let Some(Paint::MultiPoint { points }) = scene.path_mut(sel).and_then(|p| p.fill.as_mut())
        && points.len() > 1
    {
        let idx = selected
            .filter(|&i| i < points.len())
            .unwrap_or(points.len() - 1);
        points.remove(idx);
        return None;
    }
    selected
}

/// Set the SELECTED multi-point gradient point's influence (`value` from the
/// Influence slider's `track·4`). No-op unless the fill is MultiPoint and `point`
/// is valid. One undo step iff it changed.
pub(crate) fn apply_vec_grad_influence(
    scene: &mut ph2d_vec_scene::VecScene,
    pen: &ph2d_vec_edit::PenTool,
    point: Option<usize>,
    value: f64,
) {
    use ph2d_vec_scene::Paint;
    let Some(sel) = pen.selected() else {
        return;
    };
    let Some(i) = point else {
        return;
    };
    if let Some(Paint::MultiPoint { points }) = scene.path_mut(sel).and_then(|p| p.fill.as_mut())
        && let Some(gp) = points.get_mut(i)
        && (gp.influence - value).abs() > 1e-9
    {
        gp.influence = value;
    }
}

/// Set the SELECTED multi-point gradient point's jitter (`value` 0..1, from the
/// Jitter slider's track). No-op unless the fill is MultiPoint and `point` is valid.
/// One undo step iff it changed.
pub(crate) fn apply_vec_grad_jitter(
    scene: &mut ph2d_vec_scene::VecScene,
    pen: &ph2d_vec_edit::PenTool,
    point: Option<usize>,
    value: f64,
) {
    use ph2d_vec_scene::Paint;
    let Some(sel) = pen.selected() else {
        return;
    };
    let Some(i) = point else {
        return;
    };
    if let Some(Paint::MultiPoint { points }) = scene.path_mut(sel).and_then(|p| p.fill.as_mut())
        && let Some(gp) = points.get_mut(i)
        && (gp.jitter - value).abs() > 1e-9
    {
        gp.jitter = value;
    }
}

/// Component-wise linear blend of two colours at `t ∈ [0,1]`.
fn lerp_color(a: ph2d_vec_scene::Rgba8, b: ph2d_vec_scene::Rgba8, t: f64) -> ph2d_vec_scene::Rgba8 {
    let m = |x: u8, y: u8| (f64::from(x) + (f64::from(y) - f64::from(x)) * t).round() as u8;
    ph2d_vec_scene::Rgba8::new(m(a.r, b.r), m(a.g, b.g), m(a.b, b.b), m(a.a, b.a))
}

/// The Linear/Radial gradient stops of the selected path (`None` for other fills).
fn selected_ramp_stops<'a>(
    scene: &'a ph2d_vec_scene::VecScene,
    pen: &ph2d_vec_edit::PenTool,
) -> Option<&'a [ph2d_vec_scene::GradientStop]> {
    use ph2d_vec_scene::Paint;
    let sel = pen.selected()?;
    match &scene.paths().iter().find(|p| p.id == sel)?.fill {
        Some(Paint::Linear { stops, .. }) | Some(Paint::Radial { stops, .. }) => Some(stops),
        _ => None,
    }
}

/// Add an interior ramp stop to the SELECTED Linear/Radial gradient, at the midpoint
/// of the widest gap (colour = the blend there). Returns the new stop's index (to
/// select), or `None` if the fill isn't a ramp. One undo step.
pub(crate) fn apply_vec_grad_add_stop(
    scene: &mut ph2d_vec_scene::VecScene,
    pen: &ph2d_vec_edit::PenTool,
) -> Option<usize> {
    use ph2d_vec_scene::{GradientStop, Paint};
    let sel = pen.selected()?;
    // Interior stops may cross, so the Vec isn't sorted — find the widest gap on a
    // sorted (offset, colour) view; the new stop's colour is the blend across it.
    let stops = selected_ramp_stops(scene, pen)?;
    if stops.len() < 2 {
        return None;
    }
    let mut sorted: Vec<(f64, ph2d_vec_scene::Rgba8)> =
        stops.iter().map(|s| (s.offset, s.color)).collect();
    sorted.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    let mut best = (0usize, f64::NEG_INFINITY);
    for k in 0..sorted.len() - 1 {
        let gap = sorted[k + 1].0 - sorted[k].0;
        if gap > best.1 {
            best = (k, gap);
        }
    }
    let k = best.0;
    let off = (sorted[k].0 + sorted[k + 1].0) * 0.5;
    let col = lerp_color(sorted[k].1, sorted[k + 1].1, 0.5);
    if let Some(Paint::Linear { stops, .. }) | Some(Paint::Radial { stops, .. }) =
        scene.path_mut(sel).and_then(|p| p.fill.as_mut())
    {
        // Insert as an INTERIOR stop (just before the last end stop) so the two ends
        // stay at index 0 / last; return its index to select it.
        let idx = stops.len() - 1;
        stops.insert(idx, GradientStop::new(off, col));
        return Some(idx);
    }
    None
}

/// Remove the SELECTED interior ramp stop (`selected` index) from the Linear/Radial
/// gradient, keeping the two end stops (≥2 total). Returns the new selection
/// (`None`). One undo step iff it removed.
pub(crate) fn apply_vec_grad_remove_stop(
    scene: &mut ph2d_vec_scene::VecScene,
    pen: &ph2d_vec_edit::PenTool,
    selected: Option<usize>,
) -> Option<usize> {
    use ph2d_vec_scene::Paint;
    let Some(sel) = pen.selected() else {
        return selected;
    };
    if let Some(Paint::Linear { stops, .. }) | Some(Paint::Radial { stops, .. }) =
        scene.path_mut(sel).and_then(|p| p.fill.as_mut())
        && let Some(i) = selected
        && i > 0
        && i + 1 < stops.len()
    {
        stops.remove(i);
        return None;
    }
    selected
}

/// An alignment edge/center the Align buttons snap the selected paths' bboxes to
/// (within the selection's union bbox).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum VecAlign {
    Left,
    HCenter,
    Right,
    Top,
    VCenter,
    Bottom,
}

/// Map an Align button `NodeId` to its [`VecAlign`] (`None` otherwise).
pub(crate) fn vec_align_for_id(id: ph2d_editor_core::NodeId) -> Option<VecAlign> {
    Some(match id {
        x if x == ph2d_tool_vector::ids::VECTOR_ALIGN_LEFT => VecAlign::Left,
        x if x == ph2d_tool_vector::ids::VECTOR_ALIGN_HCENTER => VecAlign::HCenter,
        x if x == ph2d_tool_vector::ids::VECTOR_ALIGN_RIGHT => VecAlign::Right,
        x if x == ph2d_tool_vector::ids::VECTOR_ALIGN_TOP => VecAlign::Top,
        x if x == ph2d_tool_vector::ids::VECTOR_ALIGN_VCENTER => VecAlign::VCenter,
        x if x == ph2d_tool_vector::ids::VECTOR_ALIGN_BOTTOM => VecAlign::Bottom,
        _ => return None,
    })
}

/// Distribute axis: even the selected paths' center spacing horizontally / vertically.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum VecDistribute {
    Horizontal,
    Vertical,
}

/// Map a Distribute button `NodeId` to its [`VecDistribute`] (`None` otherwise).
pub(crate) fn vec_distribute_for_id(id: ph2d_editor_core::NodeId) -> Option<VecDistribute> {
    if id == ph2d_tool_vector::ids::VECTOR_DISTRIBUTE_H {
        Some(VecDistribute::Horizontal)
    } else if id == ph2d_tool_vector::ids::VECTOR_DISTRIBUTE_V {
        Some(VecDistribute::Vertical)
    } else {
        None
    }
}

/// Align every selected path's bbox to the selection's union bbox per [`VecAlign`]
/// (needs ≥2 selected). One undo step iff anything moved.
pub(crate) fn apply_vec_align(
    scene: &mut ph2d_vec_scene::VecScene,
    pen: &ph2d_vec_edit::PenTool,
    xforms: &ph2d_vec_scene::VecXforms,
    kind: VecAlign,
) {
    // Bbox de CURVA (a caixa que o gizmo desenha), não a de âncoras: alinhar tem de
    // casar com o que o usuário vê — uma curva que abaula para fora das âncoras
    // encostaria errado.
    // Bboxes de MUNDO: alinhar compara formas ENTRE SI, e a bbox local de toda forma
    // assentada está centrada na origem (ADR-0112).
    let boxes: Vec<(u64, [f64; 2], [f64; 2])> = pen
        .selected_paths()
        .iter()
        .filter_map(|&id| {
            scene
                .path_world_curve_bbox(xforms, id)
                .map(|(lo, hi)| (id, lo, hi))
        })
        .collect();
    if boxes.len() < 2 {
        return;
    }
    // Union bbox of the selection.
    let mut ulo = [f64::INFINITY; 2];
    let mut uhi = [f64::NEG_INFINITY; 2];
    for &(_, lo, hi) in &boxes {
        ulo[0] = ulo[0].min(lo[0]);
        ulo[1] = ulo[1].min(lo[1]);
        uhi[0] = uhi[0].max(hi[0]);
        uhi[1] = uhi[1].max(hi[1]);
    }
    for (id, lo, hi) in boxes {
        let (mut dx, mut dy) = (0.0, 0.0);
        match kind {
            VecAlign::Left => dx = ulo[0] - lo[0],
            VecAlign::Right => dx = uhi[0] - hi[0],
            VecAlign::HCenter => dx = (ulo[0] + uhi[0]) * 0.5 - (lo[0] + hi[0]) * 0.5,
            // World Y is UP here, so "Top" = the selection's MAX y and "Bottom" =
            // its MIN y (matches what the user sees on-canvas).
            VecAlign::Top => dy = uhi[1] - hi[1],
            VecAlign::Bottom => dy = ulo[1] - lo[1],
            VecAlign::VCenter => dy = (ulo[1] + uhi[1]) * 0.5 - (lo[1] + hi[1]) * 0.5,
        }
        if dx.abs() > 1e-9 || dy.abs() > 1e-9 {
            scene.translate_path_world(xforms, id, dx, dy);
        }
    }
}

/// Evenly space the selected paths' bbox CENTERS along `axis`, keeping the two
/// extremes fixed (needs ≥3 selected). One undo step iff anything moved.
pub(crate) fn apply_vec_distribute(
    scene: &mut ph2d_vec_scene::VecScene,
    pen: &ph2d_vec_edit::PenTool,
    xforms: &ph2d_vec_scene::VecXforms,
    axis: VecDistribute,
) {
    // (id, center-on-axis) for each selected path.
    let mut items: Vec<(u64, f64)> = pen
        .selected_paths()
        .iter()
        .filter_map(|&id| {
            scene.path_world_curve_bbox(xforms, id).map(|(lo, hi)| {
                let c = match axis {
                    VecDistribute::Horizontal => (lo[0] + hi[0]) * 0.5,
                    VecDistribute::Vertical => (lo[1] + hi[1]) * 0.5,
                };
                (id, c)
            })
        })
        .collect();
    if items.len() < 3 {
        return;
    }
    items.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    let n = items.len();
    let (first, last) = (items[0].1, items[n - 1].1);
    let step = (last - first) / (n - 1) as f64;
    for (k, &(id, c)) in items.iter().enumerate().take(n - 1).skip(1) {
        let target = first + step * k as f64;
        let d = target - c;
        if d.abs() > 1e-9 {
            let (dx, dy) = match axis {
                VecDistribute::Horizontal => (d, 0.0),
                VecDistribute::Vertical => (0.0, d),
            };
            scene.translate_path(id, dx, dy);
        }
    }
}

/// Rotate the SELECTED path by `degrees` (panel Transform Angle field — a
/// relative scrub) about its bbox center, recording ONE undo step iff it turned.
/// A zero delta is a no-op (no undo). Free fn (mirror of [`apply_vec_rotate`]).
pub(crate) fn apply_vec_rotate_by(
    scene: &mut ph2d_vec_scene::VecScene,
    pen: &ph2d_vec_edit::PenTool,
    degrees: f64,
) {
    if degrees.abs() < 1e-9 {
        return;
    }
    let Some(sel) = pen.selected() else {
        eprintln!("[ph2d-vec] rotate-by: nenhum path selecionado");
        return;
    };
    let Some((lo, hi)) = scene.path_bbox(sel) else {
        return;
    };
    let pivot = [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5];
    scene.rotate_path_by(sel, degrees.to_radians(), pivot);
}

/// Whole-path reshape op (panel "Smooth" / "Sharpen" / "Simplify" buttons).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum VecPathShapeOp {
    Smooth,
    Sharpen,
    Simplify,
    Subdivide,
}

/// Smooth / sharpen / simplify ALL vertices of the SELECTED path (panel Path
/// buttons), recording ONE undo step iff it changed. Free fn (mirror of
/// [`apply_vec_flip`]).
pub(crate) fn apply_vec_path_shape(
    scene: &mut ph2d_vec_scene::VecScene,
    pen: &ph2d_vec_edit::PenTool,
    op: VecPathShapeOp,
) {
    let Some(sel) = pen.selected() else {
        eprintln!("[ph2d-vec] path-shape: nenhum path selecionado");
        return;
    };
    match op {
        VecPathShapeOp::Smooth => scene.smooth_path(sel),
        VecPathShapeOp::Sharpen => scene.sharpen_path(sel),
        VecPathShapeOp::Simplify => scene.simplify_path(sel),
        VecPathShapeOp::Subdivide => scene.subdivide_path(sel),
    };
}

/// Map a Vector-panel Path button `NodeId` to its [`VecPathShapeOp`] (`None`
/// otherwise). Pure — unit-tested; called from the render_loop drain.
pub(crate) fn vec_path_shape_for_id(id: ph2d_editor_core::NodeId) -> Option<VecPathShapeOp> {
    if id == ph2d_tool_vector::ids::VECTOR_PATH_SMOOTH {
        Some(VecPathShapeOp::Smooth)
    } else if id == ph2d_tool_vector::ids::VECTOR_PATH_SHARPEN {
        Some(VecPathShapeOp::Sharpen)
    } else if id == ph2d_tool_vector::ids::VECTOR_PATH_SIMPLIFY {
        Some(VecPathShapeOp::Simplify)
    } else if id == ph2d_tool_vector::ids::VECTOR_PATH_SUBDIVIDE {
        Some(VecPathShapeOp::Subdivide)
    } else {
        None
    }
}

/// Which numeric-transform field a Vector-panel edit targets on the selected
/// path's anchor bbox (X/Y = top-left, W/H = size).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum VecTransformField {
    X,
    Y,
    W,
    H,
}

/// Map a Vector-panel Transform field `NodeId` to its [`VecTransformField`]
/// (`None` for any other id). Pure — unit-tested; called from the render_loop drain.
pub(crate) fn vec_transform_field_for_id(
    id: ph2d_editor_core::NodeId,
) -> Option<VecTransformField> {
    if id == ph2d_tool_vector::ids::VECTOR_TRANSFORM_X {
        Some(VecTransformField::X)
    } else if id == ph2d_tool_vector::ids::VECTOR_TRANSFORM_Y {
        Some(VecTransformField::Y)
    } else if id == ph2d_tool_vector::ids::VECTOR_TRANSFORM_W {
        Some(VecTransformField::W)
    } else if id == ph2d_tool_vector::ids::VECTOR_TRANSFORM_H {
        Some(VecTransformField::H)
    } else {
        None
    }
}

/// Apply a numeric transform edit to the SELECTED path: X/Y translate the anchor
/// bbox top-left to `target`; W/H scale (about the bbox min) so that dimension
/// becomes `target` (clamped > 0; a degenerate dimension can't be resized).
/// Records ONE undo step iff it changed. Free fn (mirror of [`apply_vec_boolean`]).
///
/// ⚠️ **O `sim`/`map` estão aqui pela RECEITA, e não por conveniência.** Um `W`/`H` escala a
/// GEOMETRIA (`scale_path`), e a geometria de uma forma VIVA é derivada do `w`/`h` guardado no
/// `VecShape::Param`. Escrever só os `verts` deixa a receita a descrever a caixa antiga, e a
/// primeira edição de qualquer parâmetro re-cozinha a partir dela — medido, `100 → 50 → 100`:
/// **o redimensionamento evapora em silêncio**. Este defeito é ANTERIOR à alça do gizmo (este
/// caminho era o único que escalava geometria de forma viva), e a alça só o tornou fácil de
/// alcançar. A porta é a MESMA que ela usa ([`crate::vec_shape_live::resize_recipe`]).
#[allow(clippy::too_many_arguments)] // a receita mora no ECS; a geometria, na cena
pub(crate) fn apply_vec_transform(
    sim: &mut ph2d_ecs::SimWorld,
    map: &ph2d_vec_entities::entities::VecEntityMap,
    scene: &mut ph2d_vec_scene::VecScene,
    pen: &ph2d_vec_edit::PenTool,
    xforms: &ph2d_vec_scene::VecXforms,
    field: VecTransformField,
    target: f64,
) {
    let Some(sel) = pen.selected() else {
        return;
    };
    // Os campos do painel são números de MUNDO (ADR-0112).
    let Some((lo, hi)) = scene.path_world_curve_bbox(xforms, sel) else {
        return;
    };
    // O escalonamento acontece na geometria local, em torno do canto local que
    // corresponde ao `lo` de mundo — a razão de escala é a mesma nos dois espaços.
    let local_lo = scene.path_curve_bbox(sel).map_or([0.0, 0.0], |(l, _)| l);
    match field {
        VecTransformField::X => {
            let dx = target - lo[0];
            dx.abs() > 1e-9 && scene.translate_path_world(xforms, sel, dx, 0.0)
        }
        VecTransformField::Y => {
            let dy = target - lo[1];
            dy.abs() > 1e-9 && scene.translate_path_world(xforms, sel, 0.0, dy)
        }
        VecTransformField::W => {
            let w = hi[0] - lo[0];
            if w <= 1e-6 {
                false
            } else {
                let sx = target.max(1e-4) / w;
                (sx - 1.0).abs() > 1e-9
                    && scene.scale_path(sel, sx, 1.0, local_lo)
                    && keep_recipe_in_step(sim, map, sel, sx, 1.0)
            }
        }
        VecTransformField::H => {
            let h = hi[1] - lo[1];
            if h <= 1e-6 {
                false
            } else {
                let sy = target.max(1e-4) / h;
                (sy - 1.0).abs() > 1e-9
                    && scene.scale_path(sel, 1.0, sy, local_lo)
                    && keep_recipe_in_step(sim, map, sel, 1.0, sy)
            }
        }
    };
}

/// A receita da forma VIVA de `id` passa a medir a caixa escalada por `(sx, sy)`. Devolve
/// **sempre `true`** — a geometria já mudou, e um path CRU (sem receita) não é falha nenhuma.
///
/// ⚠️ Ela vive dentro do `&&` do braço de propósito: pendurada ali, um braço de escala novo que
/// esqueça de a chamar **não conta como escala** — o `changed` fica `false` e o undo grita.
/// Escrita depois do `match`, ela seria uma linha que o próximo braço não sabe que existe.
fn keep_recipe_in_step(
    sim: &mut ph2d_ecs::SimWorld,
    map: &ph2d_vec_entities::entities::VecEntityMap,
    id: ph2d_vec_scene::VecPathId,
    sx: f64,
    sy: f64,
) -> bool {
    if let Some(&bits) = map.get(&id) {
        let e = ph2d_ecs::Entity::from_bits(bits);
        if let Some(ph2d_ecs::VecShape::Param { w, h, .. }) =
            sim.world().get::<ph2d_ecs::VecShape>(e).cloned()
        {
            crate::vec_shape_live::resize_recipe(sim, e, w * sx, h * sy);
        }
    }
    true
}

/// The shape kind a Vector draw-mode maps to (`None` = Pen, the non-shape
/// gesture). Lets the canvas dispatch route Down/Move/Up to the pen or the
/// A forma que o gesto de canvas desenha: só no modo **Shape**, e é a forma ATIVA do
/// catálogo. Antes cada forma era um modo (e este `match` crescia com o catálogo).
#[must_use]
fn shape_kind_for_mode(
    cfg: &ph2d_tool_vector::VectorDrawConfig,
) -> Option<ph2d_vec_scene::ShapeKind> {
    // ⚠️ **A tabela mora na TOOL** (`DrawMode::shape_kind`), e esta função é o adaptador que a
    // lê. Não é cerimónia: o `VectorTool::draw_config` precisa da MESMA resposta para escolher
    // de que slot saem os `values`, e uma segunda tabela aqui divergiria dela em silêncio — que
    // é exactamente o defeito que a moldura arredondável expôs (o gesto cozinhava um kind e lia
    // os parâmetros de outro).
    cfg.mode.shape_kind(cfg.shape)
}

/// As restrições do gesto de forma a partir do teclado — as de todo editor vetorial:
/// **Shift** trava a proporção **1:1** (quadrado / círculo; numa reta, snap de 45°) e
/// **Alt** desenha **a partir do centro**. Combinam.
#[must_use]
fn shape_constraint(mods: winit::keyboard::ModifiersState) -> ph2d_vec_edit::ShapeConstraint {
    ph2d_vec_edit::ShapeConstraint {
        uniform: mods.shift_key(),
        from_center: mods.alt_key(),
    }
}

/// Whether a primary pointer-Up should be **consumed** by the shape tool (and
/// NOT fall through to the chrome dispatch). Critical: in a shape mode the Up is
/// consumed ONLY while a shape drag is actually live — otherwise releasing over
/// a panel button (mode switch / boolean / close) while in a shape mode would
/// silently swallow the click, leaving every button dead.
fn shape_up_consumes(mode: ph2d_tool_vector::DrawMode, shape_active: bool) -> bool {
    matches!(
        mode,
        ph2d_tool_vector::DrawMode::Shape | ph2d_tool_vector::DrawMode::Frame
    ) && shape_active
}

/// O `anchor` e o meio-tamanho **intrínsecos** de um objeto do canvas, na linguagem
/// do gizmo de sprite: do `Sprite`, se houver; da bbox local da curva, se for uma
/// forma vetorial (ADR-0111). `([0,0], [0,0])` para o que não é nem um nem outro —
/// um grupo, que não tem geometria própria.
fn gizmo_anchor_half(
    sim: &ph2d_ecs::SimWorld,
    vec_scene: &ph2d_vec_scene::VecScene,
    flip: &ph2d_flip::FlipDoc,
    entity: ph2d_ecs::Entity,
) -> ([f32; 2], [f32; 2]) {
    if let Some(s) = sim.world().get::<ph2d_render::Sprite>(entity) {
        return (s.anchor, [s.size[0] * 0.5, s.size[1] * 0.5]);
    }
    // `sim`/`vec_scene`/`flip` chegam separados (e não via `AppGfx`) porque
    // `hero_screen` está emprestado mutável no Down — campos irmãos, borrows
    // disjuntos. Vetor OU objeto Flip (ADR-0111): a bbox local + `Transform`.
    if let Some(ah) = crate::vec_gizmo_view::anchor_half(sim, vec_scene, entity) {
        return ah;
    }
    ph2d_app_flip::gizmo_view::anchor_half(sim, flip, entity).unwrap_or(([0.0, 0.0], [0.0, 0.0]))
}

impl App {
    /// **Pick Shapes** (ADR-0128 C2b): alterna a forma FECHADA sob `world` na lista de escolhidas
    /// ([`ph2d_app_vec::state::VecState::blend_picks`]), na ordem de clique. Já escolhida → removida (corrigir a
    /// ordem sem recomeçar); nova → anexada, até o teto de [`crate::blend_live::MAX_BLEND_SOURCES`].
    /// Só FECHADAS entram — uma curva aberta não tem interior para interpolar. Marca
    /// `any_input_this_frame` para a prévia do spine redesenhar.
    pub(crate) fn blend_pick_at(&mut self, world: [f64; 2]) {
        let px = self.vec_px_to_world();
        let hit = {
            let Some(gfx) = self.gfx.as_ref() else { return };
            self.vec
                .pen
                .path_at(&gfx.vec_scene, world, 10.0 * px)
                .filter(|id| {
                    gfx.vec_scene
                        .paths()
                        .iter()
                        .any(|p| p.id == *id && p.closed)
                })
        };
        let Some(id) = hit else { return };
        if let Some(pos) = self.vec.blend_picks.iter().position(|&p| p == id) {
            self.vec.blend_picks.remove(pos);
        } else if self.vec.blend_picks.len() < crate::blend_live::MAX_BLEND_SOURCES {
            self.vec.blend_picks.push(id);
        }
        self.any_input_this_frame = true;
    }

    pub(crate) fn on_close_request(&mut self, event_loop: &ActiveEventLoop) {
        match self.handler.on_close_request() {
            CloseAction::Close => {
                self.handler.on_lifecycle(Lifecycle::WillTerminate);
                // Tear the audio system down HERE, deterministically, while the
                // main thread is quiescent — instead of letting the `cpal::Stream`
                // (ALSA/PipeWire, `!Send`) drop LAST in the `App` field cascade at
                // the end of `main`, where `snd_pcm_close` on the pipewire-alsa
                // plugin segfaults on teardown (benign — fires after "exited
                // cleanly" — but returns 139, which pollutes exit-code checks).
                #[cfg(feature = "panel-audio-editor")]
                {
                    self.audio = None;
                }
                // **E a GPU pela MESMA razão, medida no mesmo lugar.** O `EventLoop` é CONSUMIDO por
                // `run_app`, então ele — e com ele a conexão Wayland — morre quando `run_app` retorna,
                // e só DEPOIS o `App` desenrola seus campos. A `SurfaceContext` do `AppGfx` cai nesse
                // rabo, e destruir uma superfície EGL sobre um `wl_display` que já se foi marshala num
                // proxy morto: `wl_proxy_marshal_array_flags` <- libnvidia-egl-wayland2 <- libEGL_nvidia,
                // dentro do epílogo do `main` (stack de 217 coredumps desde 2026-07-22, idêntica nas
                // SEIS worktrees — é da shell, não de linha nenhuma). Benigno, porque dispara depois do
                // "exited cleanly"; mas devolve 139 e some com todo `$status` que um smoke checaria.
                //
                // A ordem aqui é a INVERSA da construção, e cada passo é uma dependência real: a
                // superfície/dispositivo primeiro (o que fala EGL), o host depois, a janela por último —
                // ela é quem possui o `wl_surface` que os outros dois referenciam.
                self.gfx = None;
                self.host = None;
                self.window = None;
                // Todo frame daqui em diante é um frame sem dispositivo (`render_frame` desiste).
                self.exiting = true;
                event_loop.exit();
            }
            CloseAction::Cancel => {}
        }
    }

    /// **O oráculo do teardown: feche a janela sozinho depois de `n` frames.**
    /// `PH2D_EXIT_AFTER_FRAMES=<n>` (não-setado = inerte, zero custo além de um `Relaxed` load).
    ///
    /// O defeito que ele mede vive no DESLIGAMENTO, então nenhum teste headless o alcança: só um app
    /// de verdade, com janela de verdade e superfície EGL de verdade, tem o que destruir na ordem
    /// errada. Com este gancho o oráculo passa a ser o `$?` do processo — **139 = a superfície morreu
    /// depois do `wl_display`; 0 = a ordem está certa** —, o que qualquer smoke pode checar sem olho
    /// humano nenhum.
    ///
    /// ⚠️ Ele passa pela **MESMA porta** que o X da janela (`on_close_request`), nunca por um
    /// `exit()` próprio. Um caminho de saída paralelo provaria a ordem de destruição de um caminho
    /// que o artista nunca toma — verde sobre nada.
    pub(crate) fn exit_after_frames_tick(&mut self, event_loop: &ActiveEventLoop) {
        use std::sync::OnceLock;
        use std::sync::atomic::{AtomicU32, Ordering};
        static LIMIT: OnceLock<Option<u32>> = OnceLock::new();
        static SEEN: AtomicU32 = AtomicU32::new(0);
        let Some(limit) = *LIMIT.get_or_init(|| {
            std::env::var("PH2D_EXIT_AFTER_FRAMES")
                .ok()
                .and_then(|v| v.parse::<u32>().ok())
        }) else {
            return;
        };
        if SEEN.fetch_add(1, Ordering::Relaxed) + 1 >= limit && !self.exiting {
            println!(
                "PH2D_EXIT_AFTER_FRAMES={limit} atingido — fechando pela porta do X da janela."
            );
            self.on_close_request(event_loop);
        }
    }

    pub(crate) fn on_resized(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        self.pending_resize = Some(WindowSize::new(size.width, size.height));
    }

    /// M14.4e drag-and-drop. winit emits one HoveredFile per path when
    /// multiple files are dragged together. Buffer paths into
    /// `self.hovered_files` and push to the hero (for the overlay) on
    /// every HoveredFile event.
    pub(crate) fn on_hovered_file(&mut self, path: std::path::PathBuf) {
        self.hovered_files.push(path);
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.dragging_files = Some((self.hovered_files.clone(), self.last_cursor));
        }
        self.handler.on_file_hover(&self.hovered_files);
        if let Some(host) = self.host.as_ref() {
            host.request_redraw();
        }
    }

    pub(crate) fn on_hovered_file_cancelled(&mut self) {
        self.hovered_files.clear();
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.dragging_files = None;
        }
        self.handler.on_file_hover_cancel();
        if let Some(host) = self.host.as_ref() {
            host.request_redraw();
        }
    }

    pub(crate) fn on_dropped_file(&mut self, path: std::path::PathBuf) {
        // M14.7 polish (7.3 fix): winit fires `DroppedFile` once PER
        // FILE on macOS but the events arrive across multiple loop
        // iterations. Importing inline on each event was racy — some
        // imports silently dropped when an event came in mid-render.
        // Buffer the path here; `render_frame` drains `pending_drops`
        // atomically.
        self.pending_drops.push(path);
        if let Some(host) = self.host.as_ref() {
            host.request_redraw();
        }
    }

    pub(crate) fn on_scale_factor_changed(&mut self, scale_factor: f64) {
        if let Some(host) = &self.host {
            host.scale().set(scale_factor as f32);
            if let Some(gfx) = self.gfx.as_ref() {
                self.pending_resize = Some(gfx.surface.size());
            }
        }
    }

    pub(crate) fn on_modifiers_changed(&mut self, mods: winit::event::Modifiers) {
        self.modifiers = mods.state();
        // M14.A: push the Shift state to the hero's WidgetStore so
        // `dispatch_pointer` Move can scale the NumberInput drag delta
        // correctly (Shift = fine adjustment). The ph2d-host
        // `PointerEvent` schema doesn't carry modifiers natively — the
        // store cache is the canonical bridge for now.
        // Fase 0c: also push the Cmd (macOS super) / Ctrl modifier
        // OR'd together — used by hierarchy + canvas multi-select to
        // map a click into `SelectModifier::Toggle`.
        if let Some(gfx) = self.gfx.as_mut()
            && let Some(hero) = gfx.hero_screen.as_mut()
        {
            hero.store.set_shift_held(self.modifiers.shift_key());
            hero.store
                .set_cmd_held(self.modifiers.super_key() || self.modifiers.control_key());
            // Motion Nodes M0.T3 — Alt cache, folded into `GestureMods.alt` for
            // graph gestures (mirror of shift/cmd; pointer events carry no mods).
            hero.store.set_alt_held(self.modifiers.alt_key());
        }
        // Shift (1:1) / Alt (do centro) durante um gesto de FORMA: reconstrói o preview
        // na hora, sem esperar o próximo Move — apertar a tecla e a forma não reagir é
        // o comportamento errado (o usuário costuma apertar com o mouse parado).
        let c = shape_constraint(self.modifiers);
        if let Some(gfx) = self.gfx.as_mut() {
            self.vec.shape.set_constraint(&mut gfx.vec_scene, c);
        }
    }

    /// IME composition commits — PT-BR / Spanish / French accent
    /// dead-key sequences arrive here on macOS, NOT in `KeyEvent::text`
    /// (the system text-input service swallows the dead-key keystroke
    /// and emits the composed char via `Ime::Commit`).
    pub(crate) fn on_ime_commit(&mut self, text: String) {
        for ch in text.chars() {
            if !ch.is_control() {
                forward_text_to_hero(self.gfx.as_mut(), ch);
            }
        }
        // `Preedit` (in-progress composition) is ignored for now — no
        // visible preedit caret yet. Future: render the preedit text
        // in italics at the caret.
    }

    /// Reflect the current hover context in the OS cursor. Called each
    /// CursorMoved (winit dedups the icon). Priority: an armed colour-picker
    /// eyedropper wins (a crosshair "target"), else the Motion graph's split
    /// divider shows a double-arrow resize cursor (`NsResize` ↕ for a horizontal
    /// divider, `EwResize` ↔ for a vertical one), else the 3D canvas split seam
    /// (same law, plus `Move` on the crossing where both seams travel together),
    /// else a timeline grab band
    /// (panel edge, label splitter, graph-height grip), else the default arrow.
    fn update_eyedropper_cursor(&self) {
        let Some(win) = self.window.as_ref() else {
            return;
        };
        use winit::window::CursorIcon;
        let cursor = self
            .gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .map(|h| {
                if h.store.eyedropper_pending().is_some() {
                    CursorIcon::Crosshair
                } else if self.over_motion_split_divider(h) {
                    if h.view.center_split.is_vertical() {
                        CursorIcon::EwResize
                    } else {
                        CursorIcon::NsResize
                    }
                } else if let Some(icon) = ph2d_app_field3d::smoke::with_smoke(|s| {
                    ph2d_app_field3d::smoke::divider_cursor(s, self.last_pointer)
                })
                .flatten()
                {
                    // ⭐ A costura da divisão do canvas 3D (W93) — a mesma fonte que o arrasto lê.
                    icon
                } else if let Some(icon) = self.sculpt3d_seam_cursor() {
                    // ⭐ **A costura da divisão da ESCULTURA** (2026-09-08) — a mesma lei do vizinho
                    // acima, com a divisão da outra cena 3D. ⚠️ Ela sai da MESMA porta que o
                    // arrasto pergunta (`Sculpt3dScene::seam_grab`), senão a seta aparece um pixel
                    // ao lado de onde o gesto pega — o que se lê como *«às vezes não agarra»*.
                    icon
                } else if let Some(icon) = self.timeline_resize_cursor(h) {
                    icon
                } else if let Some(icon) =
                    self.dock_seam_cursor(self.last_pointer.0, self.last_pointer.1)
                {
                    // ⭐ A borda de uma coluna docada — a MESMA porta que o arrasto pergunta.
                    icon
                } else {
                    CursorIcon::Default
                }
            })
            .unwrap_or(CursorIcon::Default);
        win.set_cursor(cursor);
    }

    /// The double-arrow cursor for the timeline grab band under the pointer, if
    /// any. Resolves the last pointer through the hit index to a `TimelineSurface`
    /// hit — the same channel the drag uses, so the cursor and the gesture always
    /// agree on where the band is (mirror of `over_motion_split_divider`).
    fn timeline_resize_cursor(
        &self,
        hero: &ph2d_editor_core::HeroScreen,
    ) -> Option<winit::window::CursorIcon> {
        use ph2d_editor_core::interaction::TimelineHitKind;
        use winit::window::CursorIcon;
        let (x, y) = self.last_pointer;
        let (_, kind) = hero
            .hit_index
            .hit(x, y)
            .and_then(|id| hero.store.timeline_surface_at_id(id))?;
        Some(match kind {
            // The names column widens sideways; the graph band grows downward.
            TimelineHitKind::LabelSplitter => CursorIcon::EwResize,
            TimelineHitKind::GraphResize => CursorIcon::NsResize,
            TimelineHitKind::ResizeEdge { edges } => resize_cursor_for_edges(edges),
            // The veil's duration grip resizes the composition sideways (Enio,
            // 2026-07-23: the ↔ over the ruler at the veil edge).
            TimelineHitKind::DurationHandle => CursorIcon::EwResize,
            _ => return None,
        })
    }

    /// Is the cursor over the Motion graph's draggable split divider? Resolves
    /// the last-pointer position through the hit index to a `GraphSurface` hit
    /// and checks its kind — the same channel the divider drag uses, so the
    /// cursor and the gesture agree on the grab band.
    fn over_motion_split_divider(&self, hero: &ph2d_editor_core::HeroScreen) -> bool {
        let (x, y) = self.last_pointer;
        hero.hit_index
            .hit(x, y)
            .and_then(|id| hero.store.graph_surface_at_id(id))
            .is_some_and(|(_, kind)| {
                matches!(
                    kind,
                    ph2d_editor_core::interaction::GraphHitKind::SplitDivider
                )
            })
    }

    /// ADR-0108 Fase 1: booleana N-ária sobre as regiões fechadas SELECIONADAS
    /// (hotkeys U/I/D/X). Delega ao livre [`apply_vec_boolean`] com os refs
    /// decompostos — o mesmo caminho usado pelos botões Boolean do painel (drain
    /// do render_loop, onde `self.gfx` já está destruturado e o método não é
    /// chamável).
    fn vec_boolean(&mut self, op: ph2d_vec_boolean::PathfinderOp) {
        if let Some(gfx) = self.gfx.as_mut() {
            let xf = ph2d_vec_entities::transform::build(&gfx.sim, &self.vec.entities);
            apply_vec_boolean(&mut gfx.vec_scene, &mut self.vec.pen, &xf, op);
        }
    }

    /// Arrow-key nudge: move the selection by a SCREEN delta (px), converted to
    /// world (honours zoom + orientation). Returns whether anything moved.
    ///
    /// (Até 2026-09-12 levava `record_undo`, que agrupava as auto-repetições da seta num passo da
    /// `History` do vetor. A pilha morreu sem leitor; o Ctrl+Z é o da fila global.)
    pub(crate) fn vec_nudge_selected(&mut self, dx_px: f64, dy_px: f64) -> bool {
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let win = gfx.surface.size();
        let base = gfx.camera.screen_to_world((0.0, 0.0), win);
        let moved = gfx
            .camera
            .screen_to_world((dx_px as f32, dy_px as f32), win);
        let (dx, dy) = ((moved[0] - base[0]) as f64, (moved[1] - base[1]) as f64);
        self.vec.pen.nudge(&mut gfx.vec_scene, dx, dy)
    }

    /// ADR-0108 Fase 1: Delete/Backspace no modo vetorial — prioriza apagar o
    /// VÉRTICE selecionado (edição de nó); sem vértice selecionado (ex.: resultado
    /// de booleana), apaga o PATH inteiro.
    pub(crate) fn vec_delete_selected_vertex_or_path(&mut self) -> bool {
        if self.vec.pen.selected_vert().is_some()
            && let Some(gfx) = self.gfx.as_mut()
            && apply_vec_delete_vertex(&mut gfx.vec_scene, &mut self.vec.pen)
        {
            eprintln!("[ph2d-vec] vértice apagado");
            return true;
        }
        self.vec_delete_selected()
    }

    /// ADR-0108 Fase 1: apaga o path selecionado (fallback do Delete sem vértice).
    /// World-space offset for a `px` screen-space diagonal shift (paste / dup
    /// placement), honouring the current zoom. `(0, 0)` when the gfx isn't ready.
    fn vec_screen_offset(&self, px: f64) -> (f64, f64) {
        let Some(gfx) = self.gfx.as_ref() else {
            return (0.0, 0.0);
        };
        screen_offset_world(&gfx.camera, gfx.surface.size(), px)
    }

    /// **Um campo de entrada de TEXTO tem o foco do teclado?**
    ///
    /// Pergunta ao STORE, não ao painel: qualquer `TextInput` / `NumberInput` /
    /// `Combobox` conta, venha ele do rename da Hierarquia, de um chip numérico do
    /// Inspector ou de um campo do painel do vetor. ⚠️ O nome antigo era
    /// `text_entry_focused`, e ele **mentia desde o dia em que o bloco do
    /// Flip passou a chamá-lo** — isto nunca foi uma pergunta do Vector.
    ///
    /// Quem quer *"as minhas teclas estão vivas?"* pergunta a
    /// [`Self::vector_keys_live`] / [`Self::motion_keys_live`], não a esta —
    /// compor `tool_active && !text_entry_focused` em cada braço é a enumeração
    /// que o BUGS #25 documenta apodrecendo.
    pub(crate) fn text_entry_focused(&self) -> bool {
        let Some(h) = self.gfx.as_ref().and_then(|g| g.hero_screen.as_ref()) else {
            return false;
        };
        let Some(id) = h.store.focus_id() else {
            return false;
        };
        matches!(
            h.store.get(id),
            Some(
                ph2d_editor_core::InteractiveState::TextInput { .. }
                    | ph2d_editor_core::InteractiveState::NumberInput { .. }
                    | ph2d_editor_core::InteractiveState::Combobox { .. }
            )
        )
    }

    /// Vector Ctrl+C: copy the object selection into the in-app clipboard. O
    /// recorte leva os GRUPOS inteiramente selecionados junto (ver `VecScene::
    /// copy_paths`), então colar reconstrói a estrutura. No-op sem seleção.
    fn vec_copy(&mut self) {
        let sel = self.vec.pen.selected_paths().to_vec();
        if sel.is_empty() {
            return;
        }
        if let Some(gfx) = self.gfx.as_ref() {
            let clip = gfx.vec_scene.copy_paths(&sel);
            if !clip.is_empty() {
                self.vec.clipboard = Some(clip);
            }
        }
    }

    /// Vector Ctrl+X: copy the object selection, then delete it.
    fn vec_cut(&mut self) {
        self.vec_copy();
        self.vec_delete_selected();
    }

    /// Vector Ctrl+V: paste the clipboard, offset ~12 px (screen→world), e seleciona
    /// o resultado. Ctrl+Shift+V cola **no lugar** (sem deslocar). ONE undo step.
    fn vec_paste(&mut self, in_place: bool) {
        let Some(clip) = self.vec.clipboard.clone() else {
            return;
        };
        let (dx, dy) = if in_place {
            (0.0, 0.0)
        } else {
            self.vec_screen_offset(PASTE_OFFSET_PX)
        };
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let new_ids = gfx.vec_scene.paste_clip(&clip, dx, dy);
        if new_ids.is_empty() {
            return;
        }
        self.vec.pen.select_many(&new_ids);
    }

    /// A seleção de objeto que tocar `path` produz — o grupo inteiro, se houver.
    /// A árvore é a Hierarquia (ADR-0110), então quem sabe disso é o ECS.
    pub(crate) fn vec_object_selection_for(&self, path: u64) -> Vec<u64> {
        let Some(gfx) = self.gfx.as_ref() else {
            return vec![path];
        };
        ph2d_vec_entities::entities::object_selection_for(
            &gfx.sim,
            &gfx.vec_scene,
            &self.vec.entities,
            path,
        )
    }

    /// Ctrl+G / Ctrl+Shift+G: agrupa / desagrupa a seleção. O grupo é uma entidade
    /// comum, então ele aceita sprite e path vetorial no mesmo saco.
    fn vec_group(&mut self, group: bool) {
        let sel: Vec<u64> = self
            .vec
            .pen
            .selected_paths()
            .iter()
            .filter_map(|id| self.vec.entities.get(id).copied())
            .collect();
        if sel.is_empty() {
            return;
        }
        let Some(gfx) = self.gfx.as_mut() else { return };
        let sim = &mut gfx.sim;
        // ⛔⛔ **A RECUSA FALA NA TELA, e não no terminal** (auditoria de 2026-09-06). As duas
        // recusas deste atalho saíam só por `eprintln!`, que o artista **não vê** — e o gémeo
        // deste gesto, o item *Group* do menu da Hierarquia, já respondia com toast. *Um gesto
        // que não faz nada e não diz porquê ensina que a feature está partida*, e foi assim que
        // um smoke desta linha mandou o dono agrupar um objecto só e ficar a olhar para o nada.
        if group {
            let name = format!("Group {}", sel.len());
            if ph2d_vec_entities::entities::group_entities(sim, &sel, name).is_none() {
                gfx.toasts.push(ph2d_editor_core::Toast::warning(
                    "Select two or more objects to group",
                ));
            }
        } else if ph2d_vec_entities::entities::ungroup_entities(sim, &sel) == 0 {
            gfx.toasts.push(ph2d_editor_core::Toast::warning(
                "That selection is not inside a group",
            ));
        }
    }

    /// Vector Ctrl+D: duplicate the object selection (offset ~12 px) — o irmão de
    /// teclado do botão Arrange "Duplicate". Preserva os grupos, como o paste.
    fn vec_duplicate_shortcut(&mut self) {
        let (dx, dy) = self.vec_screen_offset(PASTE_OFFSET_PX);
        if let Some(gfx) = self.gfx.as_mut() {
            apply_vec_duplicate(&mut gfx.vec_scene, &mut self.vec.pen, dx, dy);
        }
    }

    /// Apaga TODA a seleção de objeto (um grupo some inteiro) e limpa os grupos
    /// que ficaram sem membro. ONE undo step.
    fn vec_delete_selected(&mut self) -> bool {
        let sel = self.vec.pen.selected_paths().to_vec();
        if sel.is_empty() {
            return false;
        }
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let mut any = false;
        for id in &sel {
            any |= gfx.vec_scene.remove_path(*id);
        }
        if !any {
            return false;
        }
        self.vec.pen.clear();
        eprintln!("[ph2d-vec] {} path(s) apagado(s)", sel.len());
        true
    }

    /// ADR-0108 cutover: is the Vector drawing tool the active tool? Gates the
    /// Pen input hooks (replaces the retired `PH2D_VEC_PEN` test flag).
    /// Põe a ORIGEM (o pivô) do path selecionado sob o cursor, sem mover a forma.
    /// `false` (e não consome o clique) se não há forma selecionada.
    fn vec_set_origin_to_cursor(&mut self, x: f32, y: f32) -> bool {
        let Some(sel) = self.vec.pen.selected() else {
            return false;
        };
        let Some(&bits) = self.vec.entities.get(&sel) else {
            return false;
        };
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let win = gfx.surface.size();
        let target = gfx.camera.screen_to_world((x, y), win);
        let moved = ph2d_vec_entities::transform::move_origin_to(
            &mut gfx.sim,
            &mut gfx.vec_scene,
            ph2d_ecs::Entity::from_bits(bits),
            sel,
            target,
        );
        if moved {
            self.title_dirty = true;
        }
        moved
    }

    pub(crate) fn vector_tool_active(&self) -> bool {
        self.gfx.as_ref().is_some_and(|g| {
            g.tools
                .active()
                .is_some_and(|t| t.id() == ph2d_editor_core::ToolId::new("vector"))
        })
    }

    /// **O barro está na tela?** — a pergunta que o PONTEIRO da cena 3D já fazia.
    ///
    /// Delega ao papel da forma (`FormRole::draws_clay`), que é o mesmo fato que
    /// decide o passe de cor e a posse do clique no canvas. Sem cena (ou sem a
    /// feature) ela é `false`, e o resto do app nem sabe que este módulo existe.
    #[cfg(feature = "sculpt3d")]
    pub(crate) fn sculpt3d_clay_on_screen(&self) -> bool {
        self.gfx
            .as_ref()
            .and_then(|g| g.sculpt3d.as_ref())
            .is_some_and(ph2d_app_sculpt3d::Sculpt3dScene::clay_on_screen)
    }

    /// **As teclas da ESCULTURA estão vivas?** — o irmão de [`Self::motion_keys_live`]
    /// e [`Self::vector_keys_live`], e a cura do report do Enio (2026-08-17:
    /// *"depois de abrir outros módulos como Sculpt, o Motion não consegue usar os
    /// atalhos nem digitar um número"*).
    ///
    /// ⚠️ **A pergunta ERRADA era *«existe uma cena?»***. O teclado do 3D consome
    /// os **dez dígitos** e ~26 letras (o próprio módulo o escreve: *"com uma cena
    /// armada este teclado consome quase toda letra"*), e o portão dele era
    /// `sculpt3d_scene_mut().is_some()` — que fica verdadeiro **para sempre** depois
    /// do primeiro clique no pill, porque **sair do modo nunca destrói a cena**. A
    /// partir dali toda letra e todo dígito eram comidos ANTES do `handler.on_key`,
    /// então o painel do Motion não via nem atalho nem texto. Medido, o que morria
    /// era exatamente o conjunto NU do grafo (`F` Fit · `A` Add · `H` Bypass ·
    /// `K` Knife · `P` Probe) mais os dígitos — os `Ctrl+…` já passavam, porque o
    /// braço de `ctrl` só reclama o `Ctrl+Z`.
    ///
    /// ⚠️ **É uma assimetria entre duas portas que respondem à MESMA pergunta:** o
    /// ponteiro já cedia (ele pergunta `draws_clay`), o teclado não. Agora os dois
    /// perguntam o mesmo — *o barro está na tela?* —, que é o que "estou esculpindo"
    /// significa neste módulo, do mesmo jeito que *ter a ferramenta em mãos* é o que
    /// significa nos irmãos.
    /// ⚠️ **E o barro sozinho não bastava**, porque *sair do modo* não é o único jeito de
    /// ir trabalhar noutro lugar: pegar a ferramenta **Motion** no rail deixa o barro na
    /// tela e põe o artista num PAINEL, que foi o segundo caso do report (*"faça com que
    /// os atalhos motion funcionem no painel motion logo que ele for aberto"*). Uma
    /// ferramenta EM MÃOS ganha as teclas nuas — ver [`Self::a_tool_owns_the_bare_keys`].
    #[cfg(feature = "sculpt3d")]
    pub(crate) fn sculpt3d_keys_live(&self) -> bool {
        self.sculpt3d_keys_dead_reason().is_empty()
    }

    /// ⭐⭐⭐ **POR QUE as teclas da escultura estão mortas** — vazio quando estão vivas.
    ///
    /// ⛔⛔ **Ela existe por um report** (Enio, 2026-09-04: *«corrija o deletar com a tecla
    /// del»*): a tecla morria num de três guardas e **nenhum deles dizia nada**. O diagnóstico
    /// custou uma sessão de leitura de código para chegar a uma frase que esta função imprime.
    ///
    /// ⚠️ **Ela é a FONTE do [`Self::sculpt3d_keys_live`]**, e não uma segunda opinião: duas
    /// respostas à mesma pergunta divergem no dia em que alguém acrescenta um quarto guarda a
    /// só uma delas.
    #[cfg(feature = "sculpt3d")]
    pub(crate) fn sculpt3d_keys_dead_reason(&self) -> &'static str {
        if !self.sculpt3d_clay_on_screen() {
            return "nao ha' barro na tela (o pill SCULPT esta' fora, ou a forma nao e' barro)";
        }
        if self.text_entry_focused() {
            return "um campo de texto esta' FOCADO -- clique fora dele e tente outra vez";
        }
        if self.a_tool_owns_the_bare_keys() {
            return "a ferramenta Motion/Vector esta' EM MAOS e reivindica as teclas nuas";
        }
        ""
    }

    /// **Uma FERRAMENTA está em mãos reivindicando as teclas NUAS?**
    ///
    /// ⚠️ **Por que uma lista, e onde ela apodrece — dito na cara:** a cena 3D **não é uma
    /// `Tool`** (ADR-0150: a navegação orbital mora no shell justamente para manter a
    /// superfície congelada `Tool=12` fora do caminho), então ela **não participa** da
    /// arbitragem normal de ferramenta ativa — não há `ToolId` dela para o rail comparar. E
    /// o `Tool` é contrato CONGELADO (§6), logo não dá para perguntar à ferramenta *"você
    /// quer o teclado?"* sem um ADR.
    ///
    /// Enquanto isso, a precedência é expressa contra as ferramentas que de facto reclamam
    /// tecla NUA — hoje o Motion (`F`/`A`/`H`/`K`/`P`) e o Vector. ⚠️ **Uma terceira nasce
    /// fora desta lista**, e o sintoma será o mesmo report: atalhos mudos com o barro na
    /// tela. A cura definitiva é o 3D virar camada do documento (`docs/3D/05.2`), quando a
    /// pergunta deixa de precisar de lista.
    #[cfg(feature = "sculpt3d")]
    fn a_tool_owns_the_bare_keys(&self) -> bool {
        self.motion_tool_active() || self.vector_tool_active()
    }

    /// Motion Nodes M1: is the Motion Nodes tool the active tool? Gates the graph
    /// undo/redo chord (mirror of `vector_tool_active`).
    pub(crate) fn motion_tool_active(&self) -> bool {
        self.gfx.as_ref().is_some_and(|g| {
            g.tools
                .active()
                .is_some_and(|t| t.id() == ph2d_editor_core::ToolId::new("motion"))
        })
    }

    /// **As teclas do Vector estão vivas?** (BUGS #25)
    ///
    /// ⚠️ **Não é a mesma pergunta que [`Self::vector_tool_active`]**, e é essa
    /// distinção que o bug era: *ter a ferramenta em mãos* governa o PONTEIRO
    /// (clicar o canvas com um campo focado é justamente como se sai dele), mas
    /// uma TECLA pertence a quem tem o foco do teclado. Digitar `Update` no rename
    /// da Hierarquia disparava Union · Difference · modo Texto, e `Backspace`
    /// apagava um vértice em vez de uma letra.
    ///
    /// ⚠️ **A guarda sempre existiu e sempre esteve certa** — o que apodreceu foi
    /// o modo de aplicá-la: ela era composta **à mão** em três dos oito blocos de
    /// tecla, e os outros cinco nasceram sem. *Uma condição que enumera os seus
    /// leitores apodrece*; por isso ela é uma PORTA, e o arch-gate
    /// `the_vector_key_blocks_ask_whether_the_keys_are_live` recusa
    /// `vector_tool_active()` cru na família `keyboard*.rs`.
    pub(crate) fn vector_keys_live(&self) -> bool {
        self.vector_tool_active() && !self.text_entry_focused()
    }

    /// O espelho do Motion — mesma lei, mesmo motivo (o acorde Ctrl+Z do grafo
    /// roubava o undo de um campo de texto focado). Duas portas e não uma porque
    /// *qual ferramenta está em mãos* é a metade que difere; a metade do foco é a
    /// MESMA função, então não há duas respostas para *"há texto sob o cursor de
    /// teclado?"*.
    pub(crate) fn motion_keys_live(&self) -> bool {
        self.motion_tool_active() && !self.text_entry_focused()
    }

    /// Motion Nodes M1 Phase 1b-3: undo the last graph edit (Ctrl/Cmd+Z). The
    /// `MotionHistory` stack is populated by the graph-edit intents (add / delete
    /// / connect / disconnect = one step each; a node drag is one bracketed step).
    /// Restoring the doc changes the cook, so re-cook via `mark_dirty`.
    fn motion_undo(&mut self) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let m = &mut gfx.motion;
        if let Some(prev) = m.history.undo(&m.doc) {
            m.doc = prev;
            m.pump.mark_dirty();
        }
    }

    /// Motion Nodes M1 Phase 1b-3: redo (Ctrl/Cmd+Shift+Z / Ctrl+Y). Mirror of
    /// [`Self::motion_undo`].
    fn motion_redo(&mut self) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let m = &mut gfx.motion;
        if let Some(next) = m.history.redo(&m.doc) {
            m.doc = next;
            m.pump.mark_dirty();
        }
    }

    /// ADR-0108: enquanto o Pen arrasta um handle, projeta o cursor pra world e
    /// puxa os handles Bézier do último vértice. No-op barato quando não há
    /// arrasto — chamado a cada CursorMoved.
    ///
    /// O snap é entregue como closure porque o Pen sabe o que é ÂNCORA (encaixa) e
    /// o que é handle (não encaixa); a shell só sabe a posição do cursor.
    /// Arrasta o canto da gaiola do Envelope agarrado no press (ADR-0129 Fatia 1) para a
    /// posição do cursor, respeitando a convexidade (o canto para na fronteira, não sai
    /// dela). No-op (false) sem um arrasto vivo — a mesma disciplina do `vec_pen_drag_move`.
    fn vec_textpath_handle_move(&mut self, x: f32, y: f32) -> bool {
        if !self.vec.textpath_handle_drag {
            return false;
        }
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let win = gfx.surface.size();
        let w = gfx.camera.screen_to_world((x, y), win);
        crate::vec_text_ride::handle::drag(
            &mut gfx.sim,
            &mut gfx.vec_scene,
            &self.vec.entities,
            self.vec.pen.selected_paths(),
            [f64::from(w[0]), f64::from(w[1])],
            self.vec.textpath_handle_drag,
        )
    }

    /// **Pressão no modo Select sobre a alça do texto em caminho** (W5): se a alça está sob o
    /// cursor, arma o arrasto e devolve `true` — o host então PULA o picking/gizmo. Irmã do
    /// `conn_handle_down`: no Select a tool não captura o canvas, e o gizmo é inócuo sobre um
    /// texto vinculado (identidade), então a alça precisa deste arm para o dedo a pegar.
    fn vec_textpath_handle_down(&mut self, world: [f64; 2]) -> bool {
        let Some(gfx) = self.gfx.as_ref() else {
            return false;
        };
        let radius = self.vec_px_to_world() * crate::vec_text_ride::HANDLE_R_PX;
        crate::vec_text_ride::handle::press(
            &gfx.sim,
            &gfx.vec_scene,
            &self.vec.entities,
            self.vec.pen.selected_paths(),
            world,
            radius,
            &mut self.vec.textpath_handle_drag,
        )
    }

    /// **Pressão sobre uma ÂNCORA do motion path** (ADR-0141, Fatia 3): se há uma sob o
    /// cursor, arma o arrasto, abre UM passo de undo e devolve `true` — o host então PULA
    /// o picking/gizmo.
    ///
    /// ⚠️ **Não é gateada por ferramenta**, ao contrário das alças do vetor: a trajetória
    /// é do documento de ANIMAÇÃO, e o artista a molda com qualquer ferramenta na mão. O
    /// que a gateia é ela estar VISÍVEL — a mesma pergunta que o desenho faz, e ela só é
    /// verdadeira para o objeto selecionado com um binding Position.
    ///
    /// ⚠️ **Consequência honesta:** a âncora do primeiro key costuma cair EM CIMA do
    /// sprite, onde o gizmo mora, então apertar exatamente ali agarra a âncora e não o
    /// objeto. O alvo tem 7 px de raio e nada mais muda — é o mesmo trade que toda alça
    /// deste app faz, e o desenho (um quadrado, a forma universal de "isto se arrasta")
    /// é o que o anuncia.
    ///
    /// ⚠️ **Agarra a âncora OU uma alça de tangente** (`motion_path_hit`) — as duas são a
    /// mesma pergunta ("o que está sob o cursor?"), e o gesto de move despacha sobre o
    /// que veio.
    fn motion_path_anchor_down(&mut self, x: f32, y: f32) -> bool {
        let Some(gfx) = self.gfx.as_ref() else {
            return false;
        };
        let selected = gfx
            .hero_screen
            .as_ref()
            .and_then(|h| h.gizmo.iter_selected().next());
        let Some(hit) = ph2d_app_motion::motion_path_overlay::motion_path_hit(
            self.timeline.keys_mode,
            &self.timeline.doc,
            selected,
            &gfx.camera,
            gfx.surface.size(),
            x,
            y,
        ) else {
            return false;
        };
        // UM passo de undo por GESTO, não por frame de arrasto: o `commit_if_changed` do
        // release fecha o que este `begin` abriu.
        self.timeline.history.begin(&self.timeline.doc);
        self.motion_shell.path_drag = Some(hit);
        true
    }

    /// Secondary Down sobre uma **âncora** da trajetória: abre o menu de tipo de alça
    /// (Corner / Smooth / Symmetric) no cursor (ADR-0141) — o espelho do menu de ponto de
    /// curva do Painter. Devolve `true` (consumindo) só quando uma âncora foi atingida; uma
    /// ponta de tangente ou tela vazia cai fora (pan / outros handlers).
    ///
    /// ⚠️ A identidade da âncora VIAJA no `ContextMenuKind` (`{target, i}`), porque uma
    /// âncora de caminho não tem seleção persistente que o shell possa recuperar depois — o
    /// chrome a lê de volta do `last_context_menu` e o drain (`render_loop`) a converte.
    fn motion_path_open_anchor_menu(&mut self, x: f32, y: f32) -> bool {
        use ph2d_app_motion::motion_path_overlay::{MotionPathGrab, motion_path_hit};
        let Some(gfx) = self.gfx.as_ref() else {
            return false;
        };
        let selected = gfx
            .hero_screen
            .as_ref()
            .and_then(|h| h.gizmo.iter_selected().next());
        // Só a ÂNCORA (o quadrado) abre o menu — a ponta de tangente é para arrastar, e o
        // tipo de alça é propriedade do NÓ, não da alça.
        let Some(MotionPathGrab::Anchor { target, i }) = motion_path_hit(
            self.timeline.keys_mode,
            &self.timeline.doc,
            selected,
            &gfx.camera,
            gfx.surface.size(),
            x,
            y,
        ) else {
            return false;
        };
        let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) else {
            return false;
        };
        hero.store
            .open_context_menu(ph2d_editor_core::interaction::ContextMenuRequest {
                x,
                y,
                kind: ph2d_editor_core::interaction::ContextMenuKind::MotionPathAnchor {
                    target: target.get(),
                    i: i as u32,
                },
            });
        true
    }

    /// Janela do duplo-clique no canvas (a mesma do texto) e a folga de posição.
    const MOTION_PATH_DCLICK_MS: u128 = 350;
    const MOTION_PATH_DCLICK_SLOP_PX: f32 = 5.0;

    /// **Duplo-clique no CAMINHO insere um ponto ali** (ADR-0141) — o "adicionar ponto" de
    /// um editor de vetor. Um clique SIMPLES cairia toda hora perto da trajetória (que fica
    /// sempre visível para o objeto selecionado); o duplo é deliberado. A forma da curva e o
    /// compasso do objeto são PRESERVADOS (`insert_path_anchor_at` divide por de Casteljau e
    /// keya no tempo exato em que o objeto passa ali). `true` = inseriu (o chamador consome).
    ///
    /// ⚠️ O canvas não emite `DoubleClick` (é evento por-widget do chrome), então o par
    /// (instante, posição) é rastreado aqui — o mesmo recurso do `vec_text_double_click`.
    fn motion_path_curve_double_click(&mut self, x: f32, y: f32) -> bool {
        let now = std::time::Instant::now();
        let is_double = self
            .motion_shell
            .path_last_click
            .is_some_and(|(t, (px, py))| {
                now.duration_since(t).as_millis() <= Self::MOTION_PATH_DCLICK_MS
                    && (x - px).abs() <= Self::MOTION_PATH_DCLICK_SLOP_PX
                    && (y - py).abs() <= Self::MOTION_PATH_DCLICK_SLOP_PX
            });
        self.motion_shell.path_last_click = Some((now, (x, y)));
        if !is_double {
            return false;
        }
        let Some(gfx) = self.gfx.as_ref() else {
            return false;
        };
        let selected = gfx
            .hero_screen
            .as_ref()
            .and_then(|h| h.gizmo.iter_selected().next());
        let Some((target, d)) = ph2d_app_motion::motion_path_overlay::motion_path_curve_hit(
            self.timeline.keys_mode,
            &self.timeline.doc,
            selected,
            &gfx.camera,
            gfx.surface.size(),
            x,
            y,
        ) else {
            return false;
        };
        // Um passo de undo próprio, como o arrasto de âncora.
        self.timeline.history.begin(&self.timeline.doc);
        let ok = self.timeline.doc.insert_path_anchor_at(target, d);
        self.timeline.history.commit_if_changed(&self.timeline.doc);
        ok
    }

    /// Leva o que foi agarrado (âncora ou alça) para o cursor. No-op (`false`) sem um
    /// arrasto vivo — a mesma disciplina de early-return das alças do vetor.
    ///
    /// Escreve pela porta ÚNICA de cada gesto: `move_path_anchor` translada a curva (e
    /// re-suaviza as âncoras `auto`, senão a curva quebra), `move_path_tangent` a molda.
    /// As duas reescrevem as distâncias que as keys guardam na MESMA operação.
    fn motion_path_anchor_move(&mut self, x: f32, y: f32) -> bool {
        use ph2d_app_motion::motion_path_overlay::MotionPathGrab;
        let Some(grab) = self.motion_shell.path_drag else {
            return false;
        };
        let Some(gfx) = self.gfx.as_ref() else {
            return false;
        };
        let w = gfx.camera.screen_to_world((x, y), gfx.surface.size());
        match grab {
            MotionPathGrab::Anchor { target, i } => {
                let Some(mut a) = self.timeline.doc.path_anchor(target, i) else {
                    return false;
                };
                a.anchor = [w[0], w[1]];
                self.timeline.doc.move_path_anchor(target, i, a)
            }
            MotionPathGrab::Tangent { target, i, out } => {
                self.timeline
                    .doc
                    .move_path_tangent(target, i, out, [w[0], w[1]])
            }
        }
    }

    /// Arrasta a alça do PATTERN (Start/End, W4) armada para o cursor — no-op sem uma armada.
    /// Irmã do `vec_textpath_handle_move`, mesma disciplina de early-return.
    fn vec_patternpath_handle_move(&mut self, x: f32, y: f32) -> bool {
        if self.vec.patternpath_handle.is_none() {
            return false;
        }
        let armed = self.vec.patternpath_handle;
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let win = gfx.surface.size();
        let w = gfx.camera.screen_to_world((x, y), win);
        crate::pattern_live::handle::drag(
            &mut gfx.sim,
            &gfx.vec_scene,
            &self.vec.entities,
            self.vec.pen.selected_paths(),
            [f64::from(w[0]), f64::from(w[1])],
            armed,
        )
    }

    /// **Pressão no modo Select sobre uma alça do PATTERN** (W4): se uma está sob o cursor, arma o
    /// arrasto DELA e devolve `true` (o host então PULA o picking/gizmo). Irmã do
    /// `vec_textpath_handle_down`; o raio é o MESMO da ficha do texto (as duas são a mesma ficha).
    fn vec_patternpath_handle_down(&mut self, world: [f64; 2]) -> bool {
        let radius = self.vec_px_to_world() * crate::vec_text_ride::HANDLE_R_PX;
        let Some(gfx) = self.gfx.as_ref() else {
            return false;
        };
        crate::pattern_live::handle::press(
            &gfx.sim,
            &gfx.vec_scene,
            &self.vec.entities,
            self.vec.pen.selected_paths(),
            world,
            radius,
            &mut self.vec.patternpath_handle,
        )
    }

    /// **O clique do Picker de guia** (Enio 2026-07-23): com um pick armado, resolve o caminho sob o
    /// cursor e PRENDE — o motivo/texto capturado à fonte, o clicado ao guia. Clique no vazio desiste;
    /// clicar a própria fonte é ignorado (fica armado). Consome sempre o press (o guard já filtrou por
    /// `vec_path_pick.is_some()`), então o clique nunca cai no picking/gizmo enquanto o pick corre.
    fn vec_path_pick_click(&mut self, world: [f64; 2]) {
        let Some(pick) = self.vec.path_pick else {
            return;
        };
        // LITERAL-PX-OK: o MESMO raio/resolvedor do realce do hover, para o que se clica ser o que se vê.
        let hit_r = 10.0 * self.vec_px_to_world();
        self.any_input_this_frame = true;
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let Some(guide) = self.vec.pen.path_at(&gfx.vec_scene, world, hit_r) else {
            self.vec.path_pick = None; // clique no vazio = desiste
            return;
        };
        if guide == pick.source() {
            return; // clicou a própria fonte — não é um guia; continua armado
        }
        let done = match pick {
            crate::vec_pick::PathPick::PatternMotif(motif) => {
                crate::pattern_live::link(&mut gfx.sim, &self.vec.entities, motif, guide)
            }
            crate::vec_pick::PathPick::TextObject(text) => crate::vec_text_ride::link_explicit(
                &mut gfx.sim,
                &mut gfx.vec_scene,
                &self.vec.entities,
                text,
                guide,
            ),
            // ⭐⭐⭐ **O SEGUNDO clique do conta-gotas do *Swap Prefab*** — a cópia `inst` passa a
            // ser uma cópia do prefab que o clique apontou.
            //
            // ⚠️ **O clicado NÃO tem de ser o prefab** — no modelo geral a receita está escondida
            // do canvas, então o alvo é *uma cópia dele* (ou a receita, quando aberta). Quem
            // resolve é a mesma porta que os outros verbos usam.
            //
            // ⚠️ **Falhar deixa o pick ARMADO de propósito:** desarmar aqui faria um clique fora
            // do alvo parecer que a troca aconteceu.
            //
            // ⛔ Aqui viveu um `if armed() { … } else { … }` (F4.6c): a resolução pelo motor
            // `VecInstance` morreu com ele. *Armar por um motor e resolver pelo outro trocava o
            // prefab pela porta errada, e o sintoma era uma cópia que muda de desenho e mantém o
            // elo antigo* — hoje só há uma porta, e a classe inteira do defeito com ela.
            crate::vec_pick::PathPick::InstanceMain(inst) => self
                .vec
                .entities
                .get(&inst)
                .copied()
                .zip(self.vec.entities.get(&guide).copied())
                .is_some_and(|(src, dst)| {
                    crate::vec_component_general::swap_by_pick(
                        &mut gfx.sim,
                        &mut self.instance_echo,
                        &mut gfx.toasts,
                        ph2d_ecs::Entity::from_bits(src),
                        ph2d_ecs::Entity::from_bits(dst),
                    )
                }),
            // ⭐ **A ARTE de um padrão** (plano 33 W7): a fonte é a forma COM o padrão, o clicado
            // é a forma que passa a ser o desenho que se repete. ⚠️ O `guide == pick.source()` logo
            // acima já barra o ciclo — e a `source_shape` do memo barra-o outra vez, porque o
            // documento pode chegar lá por outro caminho (um save, um replay).
            crate::vec_pick::PathPick::TexturePatternArt(host, slot) => {
                // ⭐⭐⭐ **A FORMA ESCOLHIDA TRAZ O TAMANHO DELA** (report do Enio, 2026-08-30: um
                // grupo alto virava um padrão achatado). O padrão nasceu sem arte, logo com um
                // `size` QUADRADO — e um quadrado não é uma escolha, é um marcador.
                //
                // ⚠️ Pela porta que ASSA (`art_dims` -> `bake_dims`) e com a MESMA expansão de
                // objecto, senão o ladrilho tem um aspecto e a colocação tem outro.
                let fonte = ph2d_vec_scene::PatternSource::Shape(guide);
                let xf = ph2d_vec_entities::transform::build(&gfx.sim, &self.vec.entities);
                let arte = crate::texture_pattern_pick::art_dims(
                    &gfx.asset_db,
                    &gfx.vec_scene,
                    &xf,
                    &self.vec.live_drawn,
                    host,
                    &fonte,
                    &|id| {
                        ph2d_vec_entities::entities::object_selection_for(
                            &gfx.sim,
                            &gfx.vec_scene,
                            &self.vec.entities,
                            id,
                        )
                    },
                );
                let (size, _) =
                    crate::texture_pattern_pick::default_placement(&gfx.vec_scene, host, arte);
                crate::texture_pattern_edit::set_source(&mut gfx.vec_scene, host, slot, fonte, size)
            }
            // ⭐⭐⭐ **A ARTE de um PINCEL** (plano 36, W4): a fonte é a forma COM o pincel, o clicado
            // é a forma que passa a ser o motivo repetido ao longo do contorno dela.
            //
            // ⚠️ O `guide == pick.source()` logo acima já barra o ciclo — e a `brush_live::art_of`
            // barra-o outra vez, porque o documento pode chegar lá por outro caminho (um save, um
            // replay). *Duas metades porque as duas portas existem.*
            crate::vec_pick::PathPick::BrushArt(host) => {
                // ⚠️ A recusa é sobre PERTENÇA (a arte pode ser um GRUPO), e por isso a porta
                // precisa da expansão de objecto — a MESMA que a resolução usa.
                //
                // ⚠️ A expansão é medida ANTES do empréstimo mutável — a porta só pergunta pelo
                // `guide` (é ele a arte), e o `&mut scene` da escrita não coexiste com o `&scene`
                // que a expansão lê.
                let membros = ph2d_vec_entities::entities::object_selection_for(
                    &gfx.sim,
                    &gfx.vec_scene,
                    &self.vec.entities,
                    guide,
                );
                crate::vec_stroke_paint::set_art(&mut gfx.vec_scene, host, guide, &|_| {
                    membros.clone()
                })
            }
            // **O vínculo da row** (W8b.3): a fonte é o WIDGET, o clicado é a forma dirigida.
            crate::vec_pick::PathPick::WidgetBind(widget) => {
                crate::vec_widget_edit::bind(&mut gfx.sim, &self.vec.entities, widget, guide)
            }
        };
        if done {
            self.vec.path_pick = None;
            eprintln!("[ph2d-vec] pick: preso ao caminho-guia");
        }
    }

    /// **O press das ferramentas de PONTO** (W-Hand: explosão e atração).
    ///
    /// `true` = consumiu o gesto. A decisão inteira (relógio andando · física
    /// armada · a ferramenta é de ponto) mora em `body_grab::poke_at`, que é
    /// testável sem janela; aqui fica só a projeção tela→mundo e a marca que o
    /// overlay desenha.
    ///
    /// ⚠️ **Recusa quando não há mundo sob o cursor** (`vec_world_at` = `None`,
    /// que é o caso fora do canvas): estourar num ponto que não existe é o gesto
    /// caindo em silêncio, e sem esta linha ele seria consumido de qualquer jeito.
    fn poke_press(&mut self, sx: f32, sy: f32) -> bool {
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let window = gfx.surface.size();
        let world = gfx.camera.screen_to_world((sx, sy), window);
        let playing = self.playhead.is_playing();
        let simulating = self.timeline.flags.simulate_physics;
        let Some(hit) = ph2d_app_physics::body_grab::poke_at(
            &mut gfx.physics,
            &gfx.sim,
            &self.physics.interaction,
            world,
            playing,
            simulating,
        ) else {
            return false;
        };
        // A metade VISÍVEL. A explosão é instantânea, então a marca é o único
        // vestígio dela que não é "corpos que se moveram"; a atração é sustentada
        // e o overlay lê o campo VIVO da ponte (`attract_marks`), sem cópia aqui.
        if self.physics.interaction.tool == ph2d_physics_ecs::InteractionTool::Explode {
            let radius = self.physics.interaction.clamped().blast_radius;
            self.blast_flash = Some((
                world,
                radius,
                ph2d_app_physics::body_grab::BLAST_FLASH_TICKS,
            ));
            if hit > 0
                && let Some(gfx) = self.gfx.as_mut()
            {
                gfx.toasts.push(ph2d_editor_core::Toast::info(format!(
                    "Blast: {hit} bodies"
                )));
            }
        }
        true
    }

    /// **O clique do eyedropper de corpo do joint** (§12): com um pick armado,
    /// resolve o CORPO sob o cursor e religa aquela ponta do joint. Clique no
    /// vazio (ou num não-corpo) desiste; clicar o corpo que já está na outra
    /// ponta é RECUSADO e mantém o pick armado (um self-joint fica dormente,
    /// `set_joint_body`). Consome o press (o guard já filtrou por
    /// `joint_body_pick.is_some()`), então nunca cai no picking/gizmo.
    fn joint_body_pick_click(&mut self, sx: f32, sy: f32) {
        let Some((joint, slot_b)) = self.joint_body_pick else {
            return;
        };
        self.any_input_this_frame = true;
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let window_size = gfx.surface.size();
        let world_pos = gfx.camera.screen_to_world((sx, sy), window_size);
        // O sprite mais ao topo sob o cursor que é um CORPO físico e não é a
        // própria entidade-joint.
        let target = ph2d_render::pick_sprites_at_world(gfx.present.world_mut(), world_pos)
            .into_iter()
            .find(|&bits| {
                bits != joint
                    && gfx
                        .sim
                        .world()
                        .get::<ph2d_physics_ecs::RigidBody>(ph2d_ecs::Entity::from_bits(bits))
                        .is_some()
            })
            .map(ph2d_ecs::Entity::from_bits);
        match target {
            Some(t) => {
                if ph2d_app_physics::joint::set_joint_body(&mut gfx.sim, joint, slot_b, t) {
                    self.joint_body_pick = None; // religado — pronto
                }
                // senão: self-joint recusado, segue armado para outro clique
            }
            None => self.joint_body_pick = None, // vazio / não-corpo = desiste
        }
    }

    /// ⭐⭐⭐ **O clique que escolhe o ALVO de um osso inteligente.** Consome o press — é isso que
    /// impede a ferramenta Bone de criar um osso por baixo do gesto (report do dono, 2026-09-08).
    ///
    /// ⚠️ **Ele procura DUAS coisas, e a ordem é a do desenho:** primeiro uma FORMA vectorial (é o
    /// que o artista aponta num editor de vector, e o que a cena de smoke tem), depois uma SPRITE.
    /// Um alvo pode ser qualquer objecto que a timeline anime, e as duas famílias respondem a
    /// *«o que está debaixo do cursor»* de maneiras diferentes.
    ///
    /// ⛔ **Clique no vazio NÃO desiste** — ao contrário dos eyedroppers de física, e a razão é o
    /// alvo: falhar a forma por três píxeis é comum, e um pick que se perde nisso faz o artista
    /// repetir o botão sem saber porquê. Quem desiste é o `Escape`.
    fn smart_pick_click(&mut self, sx: f32, sy: f32) {
        let Some(bits_osso) = self.skeleton.smart_pick else {
            return;
        };
        self.any_input_this_frame = true;
        let hit_r = 10.0 * self.vec_px_to_world(); // LITERAL-PX-OK: o MESMO raio do irmão `vec_path_pick_click`
        let Some(world) = self.vec_world_at((sx, sy)) else {
            return;
        };
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let por_forma = self
            .vec
            .pen
            .path_at(&gfx.vec_scene, world, hit_r)
            .and_then(|pid| self.vec.entities.get(&pid).copied());
        let alvo = por_forma.or_else(|| {
            // ⚠️ O mundo do documento é `f64` e o do render `f32` — a conversão vive aqui, na porta
            // entre os dois, e não numa das pontas.
            #[expect(
                clippy::cast_possible_truncation,
                reason = "o picking de sprite fala f32; o documento vectorial fala f64"
            )]
            let p = [world[0] as f32, world[1] as f32];
            ph2d_render::pick_sprites_at_world(gfx.present.world_mut(), p)
                .into_iter()
                .find(|&bits| bits != bits_osso)
        });
        let Some(alvo) = alvo else {
            return; // clique no vazio — segue armado
        };
        if alvo != bits_osso
            && crate::skeleton_smart::set_target(
                &mut gfx.sim,
                ph2d_ecs::Entity::from_bits(bits_osso),
                ph2d_ecs::Entity::from_bits(alvo),
            )
        {
            self.skeleton.smart_pick = None;
        }
    }

    /// **O clique do eyedropper de MONTAGEM** (§13, W3): com um pick armado,
    /// resolve o CORPO sob o cursor e monta o eixo daquela roldana nele. Clique
    /// no vazio (ou num não-corpo) desiste. Consome o press, então nunca cai no
    /// picking/gizmo.
    fn wheel_mount_pick_click(&mut self, sx: f32, sy: f32) {
        let Some(wheel) = self.wheel_body_pick else {
            return;
        };
        self.any_input_this_frame = true;
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let window_size = gfx.surface.size();
        let world_pos = gfx.camera.screen_to_world((sx, sy), window_size);
        // O sprite mais ao topo sob o cursor que é um CORPO físico e não é a
        // própria roldana — montar uma roldana nela mesma não descreve nada.
        let target = ph2d_render::pick_sprites_at_world(gfx.present.world_mut(), world_pos)
            .into_iter()
            .find(|&bits| {
                bits != wheel
                    && gfx
                        .sim
                        .world()
                        .get::<ph2d_physics_ecs::RigidBody>(ph2d_ecs::Entity::from_bits(bits))
                        .is_some()
            })
            .map(ph2d_ecs::Entity::from_bits);
        if let Some(t) = target {
            ph2d_app_physics::joint_wheel::set_wheel_mount(&mut gfx.sim, wheel, t);
        }
        self.wheel_body_pick = None;
    }

    /// **O clique do eyedropper de CORDA** (§13, W1): com um pick armado, resolve a
    /// corda cuja ROTA passa sob o cursor e religa aquela roldana a ela.
    ///
    /// ⚠️ **A tolerância é a MESMA `SNAP_PX` do ímã de âncora**, convertida em
    /// mundo pelo zoom vigente — um app onde dois alvos de canvas respondem a
    /// distâncias diferentes é um app que se aprende duas vezes. Medido: 14 px valem
    /// 0,052 m a `height_world` 4 e 0,207 m a 16.
    ///
    /// Clique longe de toda corda desiste; um alvo que não é polia é RECUSADO e o
    /// pick segue armado (`set_wheel_rope`).
    fn wheel_rope_pick_click(&mut self, sx: f32, sy: f32) {
        let Some(wheel) = self.wheel_rope_pick else {
            return;
        };
        self.any_input_this_frame = true;
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let window_size = gfx.surface.size();
        let world_pos = gfx.camera.screen_to_world((sx, sy), window_size);
        let tol = ph2d_app_physics::joint_anchor_drag::SNAP_PX * gfx.camera.height_world
            / window_size.height as f32;
        match gfx.physics.rope_at_world(world_pos, tol) {
            Some(rope) => {
                if ph2d_app_physics::joint_wheel::set_wheel_rope(&mut gfx.sim, wheel, rope) {
                    self.wheel_rope_pick = None;
                }
                // senão: o alvo não é uma polia, segue armado para outro clique
            }
            None => self.wheel_rope_pick = None, // longe de toda corda = desiste
        }
    }

    /// Um quadro de POSE de osso — no-op sem osso agarrado.
    fn vec_bone_pose_move(&mut self) -> bool {
        let Some((bits, parte)) = self.skeleton.bone_pose else {
            return false;
        };
        let Some(world) = self.vec_world_at(self.last_pointer) else {
            return false;
        };
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        crate::bone_pose::pose(
            &mut gfx.sim,
            ph2d_ecs::Entity::from_bits(bits),
            world,
            parte,
        )
    }

    fn vec_envelope_corner_move(&mut self, x: f32, y: f32) -> bool {
        let Some(active) = self.vec.envelope_drag else {
            return false;
        };
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let win = gfx.surface.size();
        let w = gfx.camera.screen_to_world((x, y), win);
        crate::envelope_gesture::drag(
            &mut gfx.sim,
            Some(active),
            [f64::from(w[0]), f64::from(w[1])],
        )
    }

    fn vec_pen_drag_move(&mut self, x: f32, y: f32) -> bool {
        if !self.vector_tool_active() || !self.vec.pen.is_dragging() {
            return false;
        }
        let cfg = self.vec_snap_cfg(self.vec_px_to_world());
        // `take` evita emprestar `self` duas vezes: a closure fica com os alvos e as
        // guias, `self.vec.pen`/`self.gfx` seguem livres. Devolvidos logo abaixo.
        let targets = std::mem::take(&mut self.vec.snap_targets);
        let mut guides = Vec::new();
        let Some(gfx) = self.gfx.as_mut() else {
            self.vec.snap_targets = targets;
            return false;
        };
        let win = gfx.surface.size();
        let w = gfx.camera.screen_to_world((x, y), win);
        // `hero_screen` e `vec_scene` são campos IRMÃOS de `AppGfx`: a grade pode ser
        // consultada enquanto o Pen muta a cena.
        let mut hero = gfx.hero_screen.as_mut();
        let mut snap = |p: [f64; 2]| {
            let mut grid = |q: [f64; 2]| {
                let h = hero.as_mut()?;
                crate::vec_snap::ask_grid(&mut h.grid.snap_state, q)
            };
            let r = ph2d_vec_edit::snap::snap(&[p], &targets, cfg, Some(&mut grid));
            guides = crate::vec_snap::guides_of(&r);
            r.apply(p)
        };
        let consumed =
            self.vec
                .pen
                .on_drag(&mut gfx.vec_scene, [w[0] as f64, w[1] as f64], &mut snap);
        self.vec.snap_targets = targets;
        self.vec.snap_guides = guides;
        consumed
    }

    /// Gradient group: hit-test the selected path's gradient handles (screen `pos`)
    /// → the handle within ~9 px (world-scaled): a multi-point point, or a
    /// linear/radial endpoint. `None` unless the Vector tool is active and the
    /// selected path has a gradient fill.
    fn vec_grad_hit(&self, pos: (f32, f32)) -> Option<ph2d_vec_render::GradHandle> {
        if !self.vector_tool_active() {
            return None;
        }
        let gfx = self.gfx.as_ref()?;
        let sel = self.vec.pen.selected()?;
        let path = gfx.vec_scene.paths().iter().find(|p| p.id == sel)?;
        let win = gfx.surface.size();
        let w = gfx.camera.screen_to_world(pos, win);
        let (wx, wy) = (w[0] as f64, w[1] as f64);
        let w0 = gfx.camera.screen_to_world((0.0, 0.0), win);
        let w1 = gfx.camera.screen_to_world((1.0, 0.0), win);
        let px = (((w1[0] - w0[0]).powi(2) + (w1[1] - w0[1]).powi(2)).sqrt()) as f64;
        // ADR-0111: a geometria do gradiente é LOCAL, como a do path. O cursor desce
        // pelo afim, e o raio de captura com ele (a forma pode estar escalada).
        let x = ph2d_vec_entities::transform::xform_of_transform(
            ph2d_vec_entities::transform::world_transform(
                &gfx.sim,
                ph2d_ecs::Entity::from_bits(*self.vec.entities.get(&sel)?),
            ),
        );
        let inv = x.inverse()?;
        let l = inv.apply([wx, wy]);
        ph2d_vec_render::hit_gradient_handle(path, l[0], l[1], 9.0 * px / x.mean_scale())
    }

    /// Gradient group: while a gradient handle is grabbed, move it to the cursor's
    /// world position (a radial edge sets the radius). No-op unless a grad drag is
    /// live. Reuses the pure `drag_gradient_handle` geometry helper.
    fn vec_grad_drag_move(&mut self, x: f32, y: f32) -> bool {
        let Some(handle) = self.vec.grad_drag else {
            return false;
        };
        let Some(sel) = self.vec.pen.selected() else {
            return false;
        };
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let win = gfx.surface.size();
        let w = gfx.camera.screen_to_world((x, y), win);
        // O ponto do gradiente é guardado no espaço local do path (ADR-0111).
        let w = match self.vec.entities.get(&sel).and_then(|&b| {
            ph2d_vec_entities::transform::xform_of_transform(
                ph2d_vec_entities::transform::world_transform(
                    &gfx.sim,
                    ph2d_ecs::Entity::from_bits(b),
                ),
            )
            .inverse()
        }) {
            Some(inv) => {
                let l = inv.apply([f64::from(w[0]), f64::from(w[1])]);
                [l[0] as f32, l[1] as f32]
            }
            None => w,
        };
        if let Some(path) = gfx.vec_scene.path_mut(sel) {
            return ph2d_vec_render::drag_gradient_handle(path, handle, w[0] as f64, w[1] as f64);
        }
        false
    }

    /// Motion Nodes M1: is the cursor over the docked graph panel? Drives the
    /// cursor-gated graph keyboard focus + middle-pan routing (Blender-style F
    /// acts on the hovered area, graph vs scene).
    pub(crate) fn cursor_over_motion_graph(&self) -> bool {
        self.gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .and_then(|h| {
                h.store
                    .panel_rect(ph2d_editor_core::ids::MOTION_GRAPH_PANEL)
            })
            .is_some_and(|r| r.contains(self.last_pointer.0, self.last_pointer.1))
    }

    /// W2.E6: is the cursor over the general timeline dock? Mirrors
    /// [`Self::cursor_over_motion_graph`] — a middle-drag there pans the
    /// dope-sheet (via its `TimelineSurface` gesture), not the camera behind it.
    /// Blender-style: the hovered component owns the zoom/pan.
    pub(crate) fn cursor_over_timeline(&self) -> bool {
        self.gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .and_then(|h| h.store.panel_rect(ph2d_editor_core::ids::TIMELINE_PANEL))
            .is_some_and(|r| r.contains(self.last_pointer.0, self.last_pointer.1))
    }
    /// ADR-0108 Fase 1: while a shape drag is live, resize it to the cursor.
    /// No-op unless the Vector tool is active AND a shape gesture is in progress.
    /// A ferramenta de forma não faz hit-test, então o canto é encaixado direto.
    fn vec_shape_drag_move(&mut self, x: f32, y: f32) -> bool {
        if !self.vector_tool_active() || !self.vec.shape.is_active() {
            return false;
        }
        let Some(w) = self
            .gfx
            .as_ref()
            .map(|gfx| gfx.camera.screen_to_world((x, y), gfx.surface.size()))
        else {
            return false;
        };
        let cfg = self.vec_snap_cfg(self.vec_px_to_world());
        let p = self.vec_snap_point([f64::from(w[0]), f64::from(w[1])], cfg);
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        self.vec
            .shape
            .on_drag(&mut gfx.vec_scene, p, shape_constraint(self.modifiers))
    }

    /// **O LÁPIS, enquanto a mão anda**: a amostra entra (se andou o passo mínimo) e a curva é
    /// re-ajustada AO VIVO. No-op sem gesto — a mesma disciplina de early-return do pen.
    ///
    /// ⚠️ **Sem snap, de propósito.** A caneta encaixa porque cada clique é uma DECISÃO; encaixar
    /// cada amostra de um arrasto contínuo quantizaria a mão inteira numa grade, que é o oposto
    /// de desenhar à mão livre. (O snap a caminho/interseção é a W6 do plano, e a pergunta lá é
    /// sobre as PONTAS.)
    /// **O move do Width Tool**: a alça agarrada segue o cursor — a distância à curva vira o
    /// multiplicador e a projeção nela vira a posição. Mesma disciplina de early-return do lápis;
    /// no-op sem alça agarrada.
    fn vec_width_drag_move(&mut self, x: f32, y: f32) -> bool {
        let Some(grab) = self.vec.width_grab else {
            return false;
        };
        let Some(w) = self
            .gfx
            .as_ref()
            .map(|gfx| gfx.camera.screen_to_world((x, y), gfx.surface.size()))
        else {
            return false;
        };
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let scene = &gfx.vec_scene;
        crate::width_handles::drag(
            &mut gfx.sim,
            scene,
            &self.vec.entities,
            grab,
            [f64::from(w[0]), f64::from(w[1])],
        );
        // O dedo MOVEU: a parada deixa de ser "nascida num clique" e o release não a desfaz.
        self.vec.width_grab = Some(ph2d_app_vec::width_grab::Grab {
            created: false,
            ..grab
        });
        // Consome o move: o gesto É do Width enquanto a alça está agarrada, e deixar cair viraria
        // pan da câmera no meio do arrasto.
        true
    }

    fn vec_pencil_drag_move(&mut self, x: f32, y: f32) -> bool {
        if !self.vector_tool_active() || !self.vec.pencil.is_active() {
            return false;
        }
        // **O ESTABILIZADOR corre aqui, em px de TELA, antes da conversão para mundo** — o tremor
        // é um fato da mão sobre a mesa, e é em px que ele tem tamanho. Com o slider no mínimo o
        // `lazy_mouse_step` devolve o ponteiro cru, ao bit.
        let (x, y) = self
            .vec
            .pencil_hand
            .filter((x, y), self.vec.draw_config.pencil_stabilizer);
        let Some(w) = self
            .gfx
            .as_ref()
            .map(|gfx| gfx.camera.screen_to_world((x, y), gfx.surface.size()))
        else {
            return false;
        };
        let dyn_in = self.pointer_dynamics();
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        self.vec.pencil.on_drag(
            &mut gfx.vec_scene,
            [f64::from(w[0]), f64::from(w[1])],
            dyn_in,
        );
        // Consome o move mesmo quando a amostra foi recusada pelo passo mínimo: o gesto É do
        // lápis enquanto ele está vivo, e deixar cair viraria pan da câmera no meio do traço.
        true
    }

    /// The clip frame under `(x, y)` if it's inside the overlay waveform — for
    /// starting a selection drag. `None` if the overlay is hidden or the point is
    /// outside the waveform area.
    #[cfg(feature = "panel-audio-editor")]
    fn audio_wave_frame_at(&self, x: f32, y: f32) -> Option<(u64, f32)> {
        let view = ph2d_app_audio::wave_view()?;
        self.audio.as_ref()?.editor_clip()?;
        let r = view.rect;
        (x >= r.x && x <= r.x + r.w && y >= r.y && y <= r.y + r.h)
            .then(|| (ph2d_app_audio::frame_at_x(&view, x), freq_at_y(&view, y)))
    }

    /// Extend the active selection to the cursor. Returns `true` if a selection drag is
    /// live (the caller early-returns so it doesn't also pan).
    ///
    /// In the **spectrogram** the gesture sets a box — a time range AND a frequency band,
    /// which is the only thing spectral repair can act on (W5). In the **waveform** there is
    /// no frequency axis, so the same gesture sets a time range and explicitly CLEARS the
    /// band.
    ///
    /// Clearing it matters. The doc used to promise the waveform "sets a time range and
    /// nothing else" while the code wrote the band on every drag, using the y of a gesture
    /// the user made along x — so a horizontal drag in the waveform overwrote a carefully
    /// drawn box with a degenerate one, and Repair stayed lit and did nothing visible
    /// (audit 2026-07-12).
    #[cfg(feature = "panel-audio-editor")]
    fn audio_sel_drag_move(&mut self, x: f32, y: f32) -> bool {
        let Some((anchor, anchor_hz)) = self.audio_sel_drag else {
            return false;
        };
        let spectral = ph2d_panel_audio_editor::spectral_state::view();
        if let Some(view) = ph2d_app_audio::wave_view() {
            let cur = ph2d_app_audio::frame_at_x(&view, x);
            let cur_hz = freq_at_y(&view, y);
            if let Some(a) = self.audio.as_mut() {
                a.editor_set_selection(anchor, cur);
                if spectral {
                    a.editor_set_spectral_band(anchor_hz, cur_hz);
                } else {
                    a.editor_clear_spectral_band();
                }
            }
        }
        true
    }

    /// Update the piece drag (Move / Scale) to the cursor. Returns `true` while one is live, so
    /// the caller early-returns and the drag does not also pan the camera.
    ///
    /// Nothing is committed here — the overlay draws an outline and the release does the edit
    /// (`audio/editor/pieces.rs`). A per-frame WSOLA stretch of a three-minute piece is not a
    /// thing to attempt sixty times a second.
    #[cfg(feature = "panel-audio-editor")]
    fn audio_piece_drag_move(&mut self, x: f32) -> bool {
        let Some(view) = ph2d_app_audio::wave_view() else {
            return false;
        };
        let frame = ph2d_app_audio::frame_at_x(&view, x) as usize;
        self.audio
            .as_mut()
            .is_some_and(|a| a.editor_piece_drag_to(frame))
    }

    /// The clip frame under `(x, y)` if it's inside the overlay's time RULER — for
    /// starting a playhead scrub (seek). The ruler is the strip below the waveform;
    /// dragging it scrubs, while the wave body above it makes a selection.
    #[cfg(feature = "panel-audio-editor")]
    fn audio_ruler_frame_at(&self, x: f32, y: f32) -> Option<u64> {
        let view = ph2d_app_audio::wave_view()?;
        self.audio.as_ref()?.editor_clip()?;
        let r = view.ruler;
        (x >= r.x && x <= r.x + r.w && y >= r.y && y <= r.y + r.h)
            .then(|| ph2d_app_audio::frame_at_x(&view, x))
    }

    /// Seek the preview to the cursor `x` while a scrub is live. Returns `true` if
    /// scrubbing (the caller early-returns so the drag doesn't also pan).
    #[cfg(feature = "panel-audio-editor")]
    fn audio_scrub_move(&mut self, x: f32) -> bool {
        if !self.audio_scrub_drag {
            return false;
        }
        if let Some(view) = ph2d_app_audio::wave_view() {
            let frame = ph2d_app_audio::frame_at_x(&view, x);
            if let Some(a) = self.audio.as_mut() {
                a.editor_scrub_to_frame(frame);
            }
        }
        true
    }

    pub(crate) fn on_cursor_moved(&mut self, position: PhysicalPosition<f64>) {
        // Diagnostics: count every raw winit move (input rate), paired with `paint_stamps_this_frame`
        // in the HUD so the coalescing is visible (high events → 1 stamp).
        self.input_events_this_frame = self.input_events_this_frame.saturating_add(1);
        let prev = self.last_pointer;
        self.last_pointer = (position.x as f32, position.y as f32);
        // (Graph keyboard focus is set every frame by `motion_bridge` from the
        // cursor — reliable even when the cursor stops before the panel rect is
        // published; see its `over_graph` gate.)
        // M14.4e: cache the latest cursor for DroppedFile — winit's
        // DroppedFile carries no position, so we project the most-
        // recently-seen cursor to world.
        self.last_cursor = self.last_pointer;
        // ⚠️ **A ORDEM destes dois foi decidida na integracao de 2026-09-04**, e nao
        // e' arbitraria: o da BORDA tem `return` e uma guarda estreita (so' responde com
        // `dock_seam_drag` armado), logo so' toma o quadro quando ha' um redimensionamento
        // a serio -- e nesse quadro nao pode existir arrasto de biblioteca. Invertidos, o
        // da biblioteca correria em todo quadro de um resize.
        // ⭐ **O arrasto da BORDA de uma coluna** — dono do ponteiro até o Up, como todo arrasto
        // desta shell. Vem cedo pelo mesmo motivo que o Down dele.
        if self.dock_seam_move(self.last_pointer.0) {
            return;
        }

        // ⭐ **O arrasto da biblioteca anda AQUI**, e não no `on_mouse_input` — é a mesma doutrina
        // das guias logo abaixo: um arrasto em curso responde ao movimento, não ao botão.
        {
            let (x, y) = self.last_pointer;
            self.asset_drag_move(x, y);
        }
        // ⭐ **O PIE MENU acende pela DIRECÇÃO** (estudo de UI viva, E4) — aqui, no movimento, e não
        // no frame: o menu tem de responder ao gesto em curso, e um acender que espera o quadro
        // seguinte é um menu que a mão sente como pesado. No-op sem menu aberto.
        self.radial_point();
        // Reflect the colour-picker eyedropper in the OS cursor (a crosshair "target" while armed).
        self.update_eyedropper_cursor();
        // **AS GUIAS** (plano 25 §9, a W6.2): um arrasto de guia em curso é DONO do ponteiro,
        // então ele vem antes de todo o resto — a mesma doutrina dos `*_move` abaixo.
        //
        // ⚠️ **É AQUI que um arrasto de guia anda, e não no `on_mouse_input`.** A primeira
        // versão desta wave pôs o braço `PointerKind::Move` junto do Down/Up, num handler que
        // só produz Down e Up: o braço era **inalcançável**, `guide_pointer_move` ficou sem
        // chamador nenhum, e o produto criava a guia e a deixava onde nasceu — o que se lê
        // como *"criar funciona, mover não"*, dois sintomas de um defeito só.
        if self.guide_pointer_move(self.last_pointer.0, self.last_pointer.1) {
            return;
        }
        // **O MODO DE PREVIEW** (plano UI/UX W7r): o cursor decide que hospedeiro está aceso.
        //
        // ⚠️ **Ele NÃO consome, e a assimetria com o Down/Up é deliberada.** Um `Down` primário
        // abriria um arrasto de edição e por isso é da preview; um movimento não abre nada, e
        // consumi-lo mataria o pan e o zoom — que o Figma mantém vivos no modo de apresentação
        // dele, pela mesma razão: olhar de perto não é editar.
        //
        // ⚠️ E ele corre **antes** dos `*_move` abaixo de propósito: um arrasto de gizmo não pode
        // existir aqui dentro (o `Down` que o abriria foi consumido), então nada a jusante tem
        // opinião sobre este movimento — mas se um dia tiver, o hover da UI é quem manda.
        if self.ui_preview.is_on() {
            // ⚠️ **`Primary`, e não `is_some()`**: o `held_button` guarda QUALQUER botão entre o
            // Down e o Up, então um pan de botão do meio sobre um controle o mostraria `Pressed` —
            // o papel errado por um gesto que nem é dele.
            let pressed = self.held_button == Some(ph2d_host::PointerButton::Primary);
            self.ui_preview_point(self.last_pointer.0, self.last_pointer.1, pressed);
        }
        // BgRemoval eyedropper drag (SHELL-only): while the primary
        // button is held with the eyedropper armed, every motion
        // samples another colour. Early-return so the move does not
        // also drive a gizmo drag / panel slider.
        if self.eyedropper_dragging {
            self.try_eyedropper_sample(self.last_pointer.0, self.last_pointer.1);
            return;
        }
        // ADR-0150 W1/M2: a órbita da cena 3D. Só consome com um arrasto EM
        // CURSO — a porta devolve `false` sem cena armada e sem botão preso, e
        // é por isso que ela não rouba o hover do app 2D.
        // ⭐ A shell procura a cena; a lei do gesto é função livre da família (W2/L3-A2).
        // ⚠️ **O `false` sem cena armada continua a ser a resposta** — ele mudou de sítio
        // (era o `else` do `let Some` lá dentro), não de valor: sem cena esta porta não rouba
        // o hover do app 2D, e é essa a promessa que a linha de cima descreve.
        #[cfg(feature = "sculpt3d")]
        {
            let (px, py) = self.last_pointer;
            if self
                .sculpt3d_scene_mut()
                .is_some_and(|scene| ph2d_app_sculpt3d::pointer_move(scene, px, py))
            {
                return;
            }
        }
        // ADR-0161 W4: a órbita da janela 3D de MODELAGEM (irmã da de cima, e com
        // a mesma lei: só consome com um arrasto EM CURSO).
        if self.field3d_pointer_move(self.last_pointer.0, self.last_pointer.1) {
            return;
        }
        // Audio Editor piece drag (SHELL-only): Move / Scale own the pointer while they are live.
        #[cfg(feature = "panel-audio-editor")]
        if self.audio_piece_drag_move(self.last_pointer.0) {
            return;
        }
        // Audio Editor waveform selection drag (SHELL-only): while a selection is
        // being dragged over the overlay waveform, every motion extends it. Early-
        // return so it doesn't also pan / drive a gizmo.
        #[cfg(feature = "panel-audio-editor")]
        if self.audio_sel_drag_move(self.last_pointer.0, self.last_pointer.1) {
            return;
        }
        // Audio Editor playhead scrub drag (SHELL-only): dragging the time ruler seeks
        // the preview. Early-return so it doesn't also pan.
        #[cfg(feature = "panel-audio-editor")]
        if self.audio_scrub_move(self.last_pointer.0) {
            return;
        }
        // Keep the brush-size ring gizmo following the cursor while the
        // protection brush is armed (published for the on-canvas overlay).
        self.update_protect_brush_cursor(self.last_pointer.0, self.last_pointer.1);
        // BgRemoval protection brush drag (SHELL-only): while a dab is in
        // progress, every motion paints/erases another disc into the keep
        // mask. Early-return so it doesn't also drive a gizmo drag / slider.
        if self.protect_drag_move(self.last_pointer.0, self.last_pointer.1) {
            return;
        }
        // Painter Falloff add-drag (SHELL-only): while a freshly click-added
        // control point is grabbed, motion drags it. Early-return so it doesn't
        // pan / drive a gizmo. No-ops unless an add-drag is live.
        if self.painter_falloff_drag(self.last_pointer.0, self.last_pointer.1) {
            return;
        }
        // Painter brush stroke (SHELL-only): while a canvas stroke is open, every
        // motion feeds another `CanvasPointer` to the active PainterTool. Early-
        // return so it doesn't also drive a gizmo drag / pan / slider.
        if self.painter_canvas_move(self.last_pointer.0, self.last_pointer.1) {
            return;
        }
        // Flip stroke (ADR-0114 W2, SHELL-only): while a Flip canvas stroke is open,
        // every motion adds a world sample. Early-return so it doesn't also drive a
        // gizmo drag / pan. No-op unless a Flip stroke is in progress.
        if self.flip_canvas_move(self.last_pointer.0, self.last_pointer.1) {
            return;
        }
        // Flip eraser (ADR-0114 W2 T2.9): while an erase gesture is open, every
        // motion erases under the cursor. Early-return like the stroke. No-op
        // unless a Flip erase is in progress.
        if self.flip_erase_canvas_move(self.last_pointer.0, self.last_pointer.1) {
            return;
        }
        // Flip sculpt (ADR-0114 W5): while a reshape gesture is open, every motion is
        // ONE more brush sample (a dose é por amostra — mover devagar aplica mais).
        // No-op unless a sculpt gesture is in progress.
        if self.flip_reshape_canvas_move(self.last_pointer.0, self.last_pointer.1) {
            return;
        }
        // ADR-0114 C2: enquanto um rabisco do Colorize está aberto, cada movimento é mais
        // uma amostra da polilinha. No-op sem gesto.
        if self.flip_colorize_canvas_move(self.last_pointer.0, self.last_pointer.1) {
            return;
        }
        // Flip Edit Mode (ADR-0114 W6.1): enquanto um gesto de seleção está aberto, cada
        // movimento arrasta a CAIXA do marquee ou TRANSLADA a seleção. No-op sem gesto.
        if self.flip_edit_canvas_move(self.last_pointer.0, self.last_pointer.1) {
            return;
        }
        // Shift & Trace: enquanto um arrasto de trace está aberto, cada movimento
        // desloca (ou gira) a folha do fantasma pego. No-op sem gesto.
        if self.flip_trace_canvas_move(self.last_pointer.0, self.last_pointer.1) {
            return;
        }
        // Flip W7.5: arrasto do gizmo de POSE em curso — cada movimento recomputa a
        // pose da chave (rotate/scale) a partir do snapshot do Down. No-op sem gesto.
        if self.flip_pose_gizmo_move(self.last_pointer.0, self.last_pointer.1) {
            return;
        }
        // Flip §4.A: arrasto do gizmo de SELEÇÃO em curso — cada movimento recomputa os
        // pontos selecionados (rotate/scale) a partir do snapshot do Down. No-op sem gesto.
        if self.flip_selection_gizmo_move(self.last_pointer.0, self.last_pointer.1) {
            return;
        }
        // Motion Nodes: arrasto do gizmo de FIELD em curso — cada movimento recomputa o TRS
        // (do snapshot do Down) e escreve os params do NÓ. No-op sem gesto.
        if self.warp_gizmo_move(self.last_pointer.0, self.last_pointer.1) {
            return;
        }
        if self.field_gizmo_move(self.last_pointer.0, self.last_pointer.1) {
            return;
        }
        // Fill (Bucket) ColorDrop drag (SHELL-only): while a colour is being dragged from the Fill rail
        // button onto the canvas, deliver it to the painter's Fill. Early-return so it doesn't pan.
        if self.fill_drag_move(self.last_pointer.0, self.last_pointer.1) {
            return;
        }
        // Fill "Fill adjust" modal title-band drag (SHELL-only): while the card is grabbed, motion moves
        // it. Early-return so it doesn't pan / drive a gizmo. No-ops unless a modal drag is armed.
        // ⚠️ A janela do Input Map ANTES do Fill: as duas são cartões flutuantes, e quem está a
        // arrastar um não pode ver o outro reclamar o movimento.
        if self.input_map_drag_move(self.last_pointer.0, self.last_pointer.1) {
            return;
        }
        if self.fill_modal_drag_move(self.last_pointer.0, self.last_pointer.1) {
            return;
        }
        // Onion settings modal title-band drag (ADR-0142 W3b) — same shape as the Fill modal's.
        if self.onion_modal_drag_move(self.last_pointer.0, self.last_pointer.1) {
            return;
        }
        // ADR-0108 Fase 1: o gesto de REGIÃO do modo Node — o canto vivo segue, e o LAÇO grava
        // mais um ponto se andou o bastante. Early-return para não panar / desenhar. No-op parado.
        if let Some(m) = self.vec.marquee.as_mut() {
            m.advance(self.last_pointer);
            return;
        }
        // Shape Builder: o realce segue o cursor mesmo SEM botão apertado (é o que
        // deixa o artista ver as regiões antes de escolher uma), e com o botão ele PINTA.
        if self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::Build
            && let Some(w) = self.vec_world_at(self.last_pointer)
            && self.build_move(w)
        {
            return;
        }
        // ⭐⭐⭐ **POSAR um osso** (estudo 42 item 5): a mesma disciplina de early-return dos irmãos,
        // e no-op sem osso agarrado.
        if self.vec_bone_pose_move() {
            return;
        }
        // ADR-0129 Fatia 1: arrastar um canto da gaiola do Envelope (modo Node).
        // Mesma disciplina de early-return do pen; no-op sem um canto agarrado.
        if self.vec_envelope_corner_move(self.last_pointer.0, self.last_pointer.1) {
            return;
        }
        // ADR-0108 Fase 1.2: Pen NOVO — arrastar após a âncora puxa os handles
        // Bézier (simétricos). Early-return: não pan/gizmo. No-op sem drag ativo.
        if self.vec_pen_drag_move(self.last_pointer.0, self.last_pointer.1) {
            return;
        }
        // ADR-0111: não há gizmo vetorial próprio. O gizmo de sprite move o
        // `Transform` da entidade do path, pelo mesmo caminho de qualquer objeto.
        // Gradient group 3b: dragging a multi-point gradient handle. Same
        // early-return discipline; no-op unless a grad drag is live.
        if self.vec_grad_drag_move(self.last_pointer.0, self.last_pointer.1) {
            return;
        }
        // ADR-0108 Fase 1: shape drag-to-size (Rectangle/Ellipse/Polygon). Same
        // early-return discipline as the pen; no-op unless a shape drag is live.
        if self.vec_shape_drag_move(self.last_pointer.0, self.last_pointer.1) {
            return;
        }
        // **O LÁPIS**: a mão livre acumula amostras e re-ajusta a curva. Mesma disciplina.
        if self.vec_pencil_drag_move(self.last_pointer.0, self.last_pointer.1) {
            return;
        }
        // **O Width Tool**: a alça de largura agarrada segue o cursor. Mesma disciplina.
        if self.vec_width_drag_move(self.last_pointer.0, self.last_pointer.1) {
            return;
        }
        // Conector (modo Connect): a 2ª ponta segue o cursor e GRUDA na forma sob ele — o
        // que se vê é o conector de verdade, re-cozido pela mesma `route`. Mesma disciplina
        // de early-return; no-op sem gesto vivo.
        if self.connector_drag_move(self.last_pointer.0, self.last_pointer.1) {
            return;
        }
        // Alça de ponta (modo Select): a ponta agarrada segue o cursor. Durante o arrasto ela é
        // uma ponta SOLTA no ponteiro, e o re-cook do frame já desenha a linha inteira seguindo
        // a mão — não há caminho de preview separado, o preview é o conector.
        if self.conn_handle_move(self.last_pointer.0, self.last_pointer.1) {
            return;
        }
        // W5: arrastar a alça do texto em caminho (modo Select) — irmã da alça do conector
        // acima, mesma disciplina de early-return; no-op sem a alça agarrada.
        if self.vec_textpath_handle_move(self.last_pointer.0, self.last_pointer.1) {
            return;
        }
        // W4 do pattern: arrastar a ficha de Start/End (modo Select) — irmã da do texto.
        if self.vec_patternpath_handle_move(self.last_pointer.0, self.last_pointer.1) {
            return;
        }
        // A ÂNCORA do motion path (ADR-0141): leva a trajetória para o cursor. Irmã das
        // alças acima na disciplina, e sem gate de ferramenta — a trajetória é do
        // documento de animação, não de uma tool.
        if self.motion_path_anchor_move(self.last_pointer.0, self.last_pointer.1) {
            return;
        }
        // M14.4b.bis: middle-drag camera pan. Applied BEFORE pointer
        // forwarding so widgets receive the move event but the camera
        // also follows.
        if let Some(anchor) = self.pan_anchor
            && let Some(gfx) = self.gfx.as_mut()
        {
            let dx = self.last_pointer.0 - anchor.0;
            let dy = self.last_pointer.1 - anchor.1;
            let size = gfx.surface.size();
            // ⚠️ **O mundo-por-pixel é o da CENA, não o da JANELA** (report do Enio,
            // 2026-08-25: *«no modo motion a imagem de referência sofre um drift no pan
            // com o mouse»*). Sob o split da tool Motion a cena renderiza num
            // sub-retângulo e a projeção dela MUDA — `pan_screen_delta` com a janela
            // cheia movia o mundo `t` vezes o que o cursor andava, e a imagem ficava
            // para trás do rato. Ver `field_gizmo::pan_scene_camera`.
            let split = gfx
                .hero_screen
                .as_ref()
                .map_or(ph2d_editor_core::screens::layout::CenterSplit::None, |h| {
                    h.view.center_split
                });
            ph2d_app_motion::field_gizmo::pan_scene_camera(&mut gfx.camera, split, size, dx, dy);
            self.pan_anchor = Some(self.last_pointer);
            let _ = prev; // silence unused warning when feature shifts
        }
        // Fase 0f: extend the active rubber-band rect, if any.
        if let Some(rb) = self.rubber_band.as_mut() {
            rb.current_screen = self.last_pointer;
        }
        let evt = PointerEvent {
            x: self.last_pointer.0,
            y: self.last_pointer.1,
            pressure: 1.0,
            kind: PointerKind::Move,
            source: PointerSource::Mouse,
            // Motion Nodes M0.T1: carry the REAL held button (winit's Move has
            // none). A middle/right drag now reaches editor-core with its
            // identity intact — the graph channel needs it (pan/box-select).
            button: self
                .held_button
                .unwrap_or(ph2d_host::PointerButton::Primary),
            timestamp_ns: Self::timestamp_ns(),
        };
        self.handler.on_pointer(evt);
        // A reparent only fires on pointer-Up (handled in on_mouse_input);
        // Move never emits one.
        let _ = forward_to_hero(self.gfx.as_mut(), evt);
        // M14.7 C: advance an open gizmo drag against the latest cursor
        // (MovePivot / scale / rotate / translate). Extracted to the
        // `gizmo_drag` sibling to keep this dispatch hub readable.
        self.advance_gizmo_drag();
        // W-J2: and the joint-anchor drag, which is NOT a gizmo drag — it writes
        // a body-local anchor through the bridge's door, not a `Transform`.
        self.advance_joint_anchor_drag();
        // **§12** — e a alça do gizmo de âncora, que também não é arrasto de gizmo: ela publica
        // `InspectorAnchorEdit` no barramento, a MESMA porta por onde o painel escreve.
        self.advance_anchor_gizmo_drag();
        // W-Grab: e a MÃO, que também não é arrasto de gizmo — ela move a âncora
        // de uma mola no solver, e o `Transform` chega pelo readback do dispatch.
        //
        // ⚠️⚠️ **Os três eram `impl App` e passaram a funções da crate da família** (W2/L2 Fase B).
        // A conversão `ecrã → mundo` fica AQUI, e é o ponto inteiro: a câmera e o tamanho da
        // janela são da shell, e o gesto só quer um ponto em mundo. ⭐ *Nenhum sexto método do
        // `AppHost` foi preciso* — o que parecia «precisar da `App`» eram três tipos de crate de
        // módulo que a shell por acaso segurava.
        if let Some(gfx) = self.gfx.as_mut() {
            let window = gfx.surface.size();
            let world = gfx.camera.screen_to_world(self.last_pointer, window);
            let opts = self.physics.interaction.ik_options();
            ph2d_app_physics::body_grab::advance_body_grab(&mut gfx.physics, world);
            // ⚠️ Os dois de POSE devolvem «autorou» e a shell é que marca o quadro: o gesto sabe
            // *que* autorou, e *quando* um quadro conta para o diff de undo é decisão daqui.
            let autorou = ph2d_app_physics::body_pose::advance_body_pose(
                &mut gfx.physics,
                &mut gfx.sim,
                world,
                opts,
            ) | ph2d_app_physics::body_fk::advance_body_fk(
                &mut gfx.physics,
                &mut gfx.sim,
                world,
            );
            if autorou {
                self.any_input_this_frame = true;
            }
        }
        // Enio 2026-07-10: snap vetorial em TEMPO REAL — depois de o advance seguir o
        // cursor, gruda a forma arrastada no vizinho mais próximo (ponta p/ aberta,
        // vértice p/ fechada). Roda todo Move, então a forma prende/solta ao vivo.
        self.snap_dragged_vec_during_drag();
        // Drag-in-progress: forward pointer to active tool panel
        // hit-test → updates slider value continuously.
        if self.dragging.is_some() {
            self.dispatch_panel_pointer(self.last_pointer.0, self.last_pointer.1, false);
        }
    }

    pub(crate) fn on_mouse_wheel(&mut self, delta: MouseScrollDelta) {
        let (dx, dy) = match delta {
            MouseScrollDelta::LineDelta(x, y) => (x * 16.0, y * 16.0),
            MouseScrollDelta::PixelDelta(p) => (p.x as f32, p.y as f32),
        };
        // M14.4b.bis: wheel over the canvas zooms the camera. Wheel
        // over a hero panel keeps the existing panel-scroll behavior
        // (forward to hero).
        // ⭐ **A roda pertence À PALETA enquanto ela estiver aberta** (F3 / ADR-0166), e vem antes
        // de tudo — inclusive do Input Map: ela é um modal de TELA CHEIA com scrim, então não há
        // «por fora dela». Sem esta linha a lista transbordava o ecrã e o que sobrava era
        // inalcançável, com a roda a dar zoom no canvas por baixo do scrim (report do Enio, 25/08).
        if self.command_palette_wheel(dy) {
            return;
        }
        // ⭐ **A roda sobre a janela do Input Map é dela** — e vem ANTES do resto, pelo motivo do
        // arrasto: a roda que atravessasse o cartão daria zoom no canvas por baixo dele.
        if self.input_map_wheel(dy) {
            return;
        }
        let over_panel =
            cursor_over_hero_panel(self.gfx.as_ref(), self.last_pointer.0, self.last_pointer.1);
        // ADR-0150 W1/M2: fora de painel, a roda aproxima a câmera 3D. Um
        // "passo" é uma linha de roda (os 16 px acima são a régua do zoom 2D).
        #[cfg(feature = "sculpt3d")]
        if !over_panel && self.sculpt3d_wheel(dy / 16.0) {
            return;
        }
        // ADR-0161 W4: o mesmo para a janela 3D de modelagem.
        if !over_panel && self.field3d_wheel(dy / 16.0) {
            return;
        }
        // **O ajuste modal do Gap Closure** (doc 06 §8): em modo Fill, Ctrl+roda sobre o
        // canvas ajusta o alcance — e os helpers no canvas mostram, ao vivo, quais vãos
        // o valor atual fecha (`flip_gap_live`). A roda CRUA continua sendo zoom
        // (inspecionar o line-art é load-bearing); o GP toma a roda inteira durante o
        // fill, e esta é a divergência deliberada, documentada em vez de silenciosa.
        if !over_panel
            && (self.modifiers.control_key() || self.modifiers.super_key())
            && let Some(track) = ph2d_app_flip::gap_live::gap_wheel_track(
                self.flip_state.active,
                self.flip_state.style,
                dy / 16.0,
            )
        {
            if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
                // As DUAS metades do que o próprio slider faz num arrasto (ver
                // `panel-flip/event.rs`): o valor do widget no store (o knob que o
                // artista vê — sem isto ele pinta o valor velho por cima do novo) e o
                // `SetValue` pro tool (o valor autorado, clampado pelo MESMO braço).
                hero.store
                    .set_slider_value(ph2d_tool_flip::ids::FLIP_GAP, track as f32);
                hero.bus
                    .push(ph2d_editor_core::action_bus::EditorAction::ToolPanelEvent(
                        ph2d_editor_core::tool::PanelEvent::SetValue(
                            ph2d_tool_flip::ids::FLIP_GAP,
                            track,
                        ),
                    ));
                self.any_input_this_frame = true;
            }
            return;
        }
        // **A ROLAGEM de uma moldura** (o item 3 do estudo dos contêineres): a roda sobre uma
        // moldura que RECORTA e cujo conteúdo NÃO CABE rola essa moldura, em vez de dar zoom.
        //
        // ⚠️ **Ela não rouba o zoom, e é isso que a torna aceitável sem modo nem modificador:** as
        // duas condições juntas só valem numa lista que o artista fez deliberadamente transbordar,
        // e ali rolar é a única coisa que a roda pode querer dizer. Em todo o resto da tela — que é
        // 99% dela — a roda continua a ser o zoom, que é o gesto do dia inteiro. É o precedente do
        // Gap Closure, uma dúzia de linhas acima, com a mesma frase: *a roda crua é load-bearing*.
        //
        // ⚠️ E ela vem **antes** do zoom pela razão de sempre: quem consome tem de decidir primeiro,
        // senão a câmera já se mexeu quando a moldura for perguntada.
        if !over_panel && self.wheel_scrolls_a_frame(dx, dy) {
            return;
        }
        if !over_panel && let Some(gfx) = self.gfx.as_mut() {
            // Wheel up (positive dy) zooms IN (smaller height_world).
            let factor = 0.9_f32.powf(dy / 16.0);
            // ⚠️ **A roda escreve o DESTINO; quem move a câmera é o quadro** (`crate::canvas_zoom`).
            // O `camera.zoom(factor)` que morava aqui fazia do gesto mais repetido do app o único
            // movimento da tela que salta.
            gfx.canvas_zoom.wheel(gfx.camera.height_world, factor);
            self.any_input_this_frame = true;
        } else {
            let evt = ph2d_host::WheelEvent {
                x: self.last_pointer.0,
                y: self.last_pointer.1,
                delta_x: dx,
                delta_y: dy,
                modifiers: Self::convert_modifiers(self.modifiers),
                timestamp_ns: Self::timestamp_ns(),
            };
            forward_wheel_to_hero(self.gfx.as_mut(), evt);
        }
    }

    pub(crate) fn on_mouse_input(&mut self, state: ElementState, button: MouseButton) {
        self.any_input_this_frame = true;
        // W-Grab: **soltar a mão vem ANTES de tudo.** Este handler tem muitos
        // early-returns e uma mão que sobrevive ao release fica colada no cursor
        // para sempre; e vale para qualquer botão, porque uma mão não é um
        // modificador (ver `ph2d_app_physics::body_grab::release_body_grab`).
        if state == ElementState::Released {
            if let Some(gfx) = self.gfx.as_mut() {
                ph2d_app_physics::body_grab::release_body_grab(&mut gfx.physics);
                ph2d_app_physics::body_pose::release_body_pose(&mut gfx.physics);
                ph2d_app_physics::body_fk::release_body_fk(&mut gfx.physics);
            }
            // **§12** — e a alça do gizmo de âncora, pela MESMA razão escrita acima: este handler
            // tem muitos early-returns, e uma alça que sobrevive ao release fica colada ao cursor.
            self.end_anchor_gizmo_drag();
        }
        // ⭐⭐⭐ **O ARRASTO DA BIBLIOTECA** (plano `docs/Components/07`, etapa B).
        //
        // ⛔⛔ **E ele vem DEPOIS da soltura das mãos, não antes — a 1.ª versão tinha-o antes e o
        // comentário logo acima descreve exactamente o defeito que isso cria:** este handler tem
        // muitos early-returns, e uma mão que sobrevive ao release fica colada ao cursor para
        // sempre. Eu acrescentei um `return` **à frente** da própria linha que existe para o
        // evitar. *Ler a regra não é o mesmo que estar do lado certo dela.*
        //
        // ⚠️ **O `Down` NÃO consome**: enquanto o limiar não for passado isto ainda é um clique, e
        // o clique do cartão tem de chegar ao painel como sempre (ele escolhe; o duplo-clique
        // instancia).
        //
        // ⚠️ **O `Up` consome, e só quando o gesto foi de facto um arrasto.** Sem isso o mesmo
        // gesto largaria o asset na tela **e** contaria como clique no cartão — o `forward_to_hero`
        // que emite o `Click` corre mais abaixo neste mesmo handler.
        if button == MouseButton::Left {
            match state {
                ElementState::Pressed => {
                    let (x, y) = self.last_pointer;
                    self.asset_drag_down(x, y);
                }
                ElementState::Released => {
                    let (x, y) = self.last_pointer;
                    // ⛔⛔ **E ele NÃO consome, e a 1.ª versão consumia.** Um `return` aqui salta o
                    // resto deste handler — e com ele o `held_button = None` e, mais abaixo, o
                    // `forward_to_hero` que é o **único** sítio do app que faz `set_active(None)`.
                    // Consequências medidas na auditoria: o cartão fica preso em `Pressed`, o
                    // widget activo aponta para ele para sempre, e o `post_frame_undo` recusa-se a
                    // registar um passo enquanto `held_button.is_some()` ⇒ **a queda não era
                    // desfazível** até ao clique seguinte.
                    //
                    // ⚠️ **E não há nada a suprimir:** o `Click` que o despachante emite a seguir
                    // cai num cartão, e o `apply_event` do navegador **não tem braço para
                    // `Click(cartão)`** — só para `DoubleClick`. *Suprimir um evento inofensivo
                    // custou quatro fugas de estado.*
                    self.asset_drag_up(x, y);
                }
            }
        }
        // ⭐⭐⭐ **UM APERTO NO CANVAS SOLTA O TECLADO QUE UM CAMPO DO PAINEL SEGURAVA.**
        //
        // ⛔ **Ele vem ANTES dos três consumidores abaixo, e é aí que está a cura.** Os três
        // — a cena de escultura, a janela de modelagem, a alça do gizmo de âncora — TOMAM o
        // aperto e devolvem `return` antes do `forward_to_hero`, que é o único sítio onde a
        // partida de foco corre. Sem esta linha, tocar num chip numérico de painel e voltar
        // ao canvas deixava `focus_id` preso naquele chip **para o resto da sessão**, e com
        // ele morriam `Delete`, `Ctrl+Z` e todo atalho do módulo que tomou o gesto (Enio,
        // 2026-09-07: *"a tecla del parou de funcionar e não temos undo/redo para Cloth"* —
        // dois relatos, um defeito, nenhum deles do pincel de tecido).
        //
        // ⚠️ **A guarda é a MESMA que os consumidores usam** (`pointer_over_chrome`): um
        // aperto SOBRE o chrome não é um aperto no canvas, e para esse o despachante lá
        // abaixo continua a decidir sozinho — inclusive quando ele cai em espaço morto de
        // painel, que é blur pela lei dele.
        //
        // ⚠️ **E ela não presume que alguém vá consumir**: a porta é idempotente, então
        // quando ninguém toma o gesto o `dispatch_down` a seguir não tem o que refazer.
        // *Condicionar a soltura a QUEM tomou seria uma lista de consumidores a apodrecer no
        // dia em que nasce o quarto.*
        if state == ElementState::Pressed
            && !crate::chrome_hit::pointer_over_chrome(
                self.gfx.as_ref(),
                self.last_pointer.0,
                self.last_pointer.1,
            )
        {
            forward_blur_to_hero(self.gfx.as_mut());
        }
        // ADR-0150 W1/M2: a cena 3D toma o botão para navegar. Inerte (e
        // portanto invisível) sem cena armada.
        #[cfg(feature = "sculpt3d")]
        {
            let taken = match state {
                ElementState::Pressed => self.sculpt3d_pointer_down(button),
                // ⚠️ **Os dois lados do `match` deixaram de ter a mesma FORMA** (W2/L3-A2), e
                // é mensagem, não descuido: o pen-up só precisa da CENA e é função livre; o
                // pen-down arbitra quem fica com o gesto e para isso lê `gfx`, `last_pointer`
                // e `modifiers` — logo continua em `impl App`, com a razão escrita lá.
                ElementState::Released => self
                    .sculpt3d_scene_mut()
                    .is_some_and(ph2d_app_sculpt3d::pointer_up),
            };
            if taken {
                return;
            }
        }
        // ADR-0161 W4: a janela 3D de modelagem toma o botão para navegar. Inerte
        // (e portanto invisível) sem o smoke armado, e ela só reclama o gesto que
        // começa DENTRO da área que ela desenhou.
        {
            let taken = match state {
                ElementState::Pressed => self.field3d_pointer_down(button),
                ElementState::Released => self.field3d_pointer_up(),
            };
            if taken {
                return;
            }
        }
        // **§12 — a alça do gizmo de âncora toma o botão** (ADR-0072 §2.3).
        //
        // ⚠️ Antes do resto do `Down`, e com `return`: agarrar uma alça **não** é selecionar um
        // sprite, não é começar um marquee e não é entregar o ponteiro à ferramenta. O
        // `try_open_…` já recusou tudo o que não é canvas (painel por cima, seção fechada, nenhuma
        // linha aberta), então chegar aqui e devolver `true` significa que o gesto é este.
        if state == ElementState::Pressed
            && button == MouseButton::Left
            && let (px, py) = self.last_pointer
            && self.try_open_anchor_gizmo_drag(px, py)
        {
            return;
        }
        let kind = match state {
            ElementState::Pressed => PointerKind::Down,
            ElementState::Released => PointerKind::Up,
        };
        let mapped_button = match button {
            MouseButton::Left => ph2d_host::PointerButton::Primary,
            MouseButton::Right => ph2d_host::PointerButton::Secondary,
            MouseButton::Middle => ph2d_host::PointerButton::Middle,
            _ => ph2d_host::PointerButton::Primary,
        };
        // Motion Nodes M0.T1: track the held button so `CursorMoved` can carry
        // its identity (winit Move events don't). Held between Down and Up.
        self.held_button = match kind {
            PointerKind::Down => Some(mapped_button),
            PointerKind::Up => None,
            PointerKind::Move => self.held_button,
        };
        let evt = PointerEvent {
            x: self.last_pointer.0,
            y: self.last_pointer.1,
            pressure: 1.0,
            kind,
            source: PointerSource::Mouse,
            button: mapped_button,
            timestamp_ns: Self::timestamp_ns(),
        };
        self.handler.on_pointer(evt);
        // Audio Editor waveform selection (SHELL-only): a primary press INSIDE the
        // overlay waveform starts a selection (cleared to a point); release ends
        // it. Early-return so the press doesn't drive the canvas/gizmo underneath.
        // Presses on the overlay's title-bar / resize handles fall through (they're
        // outside the waveform rect) to the shared BlenderHit dispatch.
        #[cfg(feature = "panel-audio-editor")]
        match kind {
            // Press on the RULER strip → grab the playhead and scrub (seek).
            PointerKind::Down
                if let Some(frame) =
                    self.audio_ruler_frame_at(self.last_pointer.0, self.last_pointer.1) =>
            {
                self.audio_scrub_drag = true;
                if let Some(a) = self.audio.as_mut() {
                    a.editor_scrub_to_frame(frame);
                }
                return;
            }
            // Press on the WAVE body → what it means depends on the armed tool (the Edit
            // section's toolbar). Select drags a time range, which is what the waveform has
            // always done; Move drags a piece onto another seam; Scale drags a piece's edge.
            PointerKind::Down
                if let Some(hit) =
                    self.audio_wave_frame_at(self.last_pointer.0, self.last_pointer.1) =>
            {
                use ph2d_panel_audio_editor::tool_state::{EditTool, tool};
                let frame = hit.0 as usize;
                match tool() {
                    EditTool::Move => {
                        if let Some(a) = self.audio.as_mut() {
                            a.editor_piece_grab(frame);
                        }
                    }
                    EditTool::Scale => {
                        if let Some(a) = self.audio.as_mut() {
                            a.editor_piece_scale_grab(frame);
                        }
                    }
                    EditTool::Select => {
                        self.audio_sel_drag = Some(hit);
                        if let Some(a) = self.audio.as_mut() {
                            a.editor_clear_selection();
                        }
                    }
                }
                return;
            }
            // Let go of a piece: THIS is where the reorder / stretch lands, as one undo step.
            PointerKind::Up
                if self
                    .audio
                    .as_ref()
                    .is_some_and(|a| a.editor_piece_drag().is_some()) =>
            {
                if let Some(a) = self.audio.as_mut() {
                    a.editor_piece_release();
                }
                return;
            }
            PointerKind::Up if self.audio_scrub_drag => {
                self.audio_scrub_drag = false;
                // Hand the playhead back to playback if it's advancing; else the
                // manual position stays where it was dropped.
                if let Some(a) = self.audio.as_mut() {
                    a.editor_end_scrub();
                }
                return;
            }
            PointerKind::Up if self.audio_sel_drag.take().is_some() => return,
            _ => {}
        }
        // Was a right-click context menu (or the Fill "Fill adjust" modal) open when this click
        // arrived? If so the click belongs to that overlay (its slider/buttons/items) — chrome dispatch
        // in `forward_to_hero` handles it, so the canvas-consume arms below (paint / gizmo / select /
        // pan) must NOT also fire on a click LANDING on the overlay (which sits over the canvas). The
        // Fill modal counts as a modal exactly like the new-image dialog — without this, clicking its
        // threshold slider started a fresh flood-fill on the canvas underneath (mirror of the
        // new-image-modal "leaked a dab" fix). Captured now because `forward_to_hero` may close it.
        let menu_open_before = self
            .gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .is_some_and(|h| {
                h.store.context_menu().is_some() || h.store.fill_modal_pos().is_some()
            });
        // Colour-picker eyedropper armed when this click arrived? `forward_to_hero` services the pick
        // (sampling the pixel) AND clears the pending flag, so by the time the consume arms below run
        // it reads as disarmed. Capture it now so the Painter brush does NOT also paint where the user
        // sampled — the eyedropper must inhibit the brush (the sampled click is consumed, not painted).
        let eyedropper_armed_before = self
            .gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .is_some_and(|h| h.store.eyedropper_pending().is_some());

        // ADR-0108 cutover: the Vector tool's Pen draws ONLY on empty canvas.
        // A press over ANY UI — a docked panel body, a topbar pill, an open
        // menu, or this tool's own Style panel controls — MUST fall through to
        // the chrome dispatch below, never the pen; otherwise the whole UI is
        // unclickable while drawing (can't even deactivate the tool). Guard
        // mirrors the sprite-pick path: no panel under the cursor AND no
        // interactive widget hit (`hit_index` covers pills / menus / panel
        // controls; `panel_at` covers panel bodies incl. the vector panel).
        let on_canvas = self
            .gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .map(|h| {
                h.store.panel_at(evt.x, evt.y).is_none() && h.hit_index.hit(evt.x, evt.y).is_none()
            })
            .unwrap_or(false);
        // **O MODO DE PREVIEW** (plano UI/UX W7r) — a UI desenhada a responder ao rato.
        //
        // ⚠️ Vem antes de TODA ferramenta, e não só das do Vector: enquanto ele corre não existe
        // pincel, traço do Flip, seleção, gizmo nem caneta — o clique é da interface que o artista
        // desenhou. É a mesma doutrina dos picks armados (*um modo em curso é dono do clique*),
        // uma família adiante: aqueles são modais sobre o Vector, este é modal sobre o editor.
        //
        // ⚠️ **A guarda é `over_canvas_or_gizmo`, e o `on_canvas` estava ERRADO — reportado pelo
        // Enio (2026-08-07: *"em preview permite que tanto o pai como o filho fossem
        // selecionados"*).** O `on_canvas` exige o `hit_index` **VAZIO**, e o gizmo registra as
        // alças (e o interior de translação) NELE — o doc do `over_canvas_or_gizmo` diz isto
        // literalmente, e eu escolhi o outro justificando-o num comentário. Entrar na preview
        // **exige o hospedeiro SELECIONADO** (a seção States só existe assim), logo o gizmo está
        // sempre lá, logo a guarda **nunca disparava na configuração em que a feature roda** — o
        // clique caía no picking e selecionava o filho. E ficava ERRÁTICO quando um estado movia
        // a forma para longe: fora da caixa do gizmo o `hit_index` volta a estar vazio e a guarda
        // acordava, então o mesmo gesto funcionava ou não conforme ONDE a forma estava.
        //
        // O `over_canvas_or_gizmo` aceita o gizmo por cima e **continua a barrar painel** — é
        // isso que mantém o próprio botão *Preview* clicável, que é a porta de saída visível.
        if self.ui_preview.is_on()
            && self.over_canvas_or_gizmo(evt.x, evt.y)
            && !menu_open_before
            && mapped_button == ph2d_host::PointerButton::Primary
            && matches!(kind, PointerKind::Down | PointerKind::Up)
        {
            self.ui_preview_point(evt.x, evt.y, kind == PointerKind::Down);
            return;
        }
        // ADR-0114 W2: desenho do Flip. O pen-UP sempre encerra um traço em curso
        // (consome), mesmo que o modo tenha mudado no meio. O pen-DOWN começa um
        // traço só no modo Draw, em canvas vazio — em Select cai no gizmo/pick.
        if kind == PointerKind::Up
            && mapped_button == ph2d_host::PointerButton::Primary
            && self.flip_state.draw.is_active()
            && self.flip_canvas_up()
        {
            return;
        }
        // Flip eraser (T2.9): the pen-UP ends an erase gesture (+ Soft cleanup).
        if kind == PointerKind::Up
            && mapped_button == ph2d_host::PointerButton::Primary
            && self.flip_state.erasing
            && self.flip_erase_canvas_up()
        {
            return;
        }
        // **AS GUIAS** (plano 25 §9, a W6.2). O gesto da régua vem ANTES de toda ferramenta,
        // e não por prioridade inventada: a faixa da régua está VISÍVEL com qualquer ferramenta
        // na mão, então um press nela que caísse no picking/gizmo moveria um objeto em vez de
        // puxar uma guia — chrome desenhado e morto sob o mouse, que é o defeito que esta
        // codebase varre a cada wave.
        //
        // ⚠️ Uma vez começado, o arrasto é DONO do ponteiro até o Up (o padrão do `joint_draw`).
        // O passo do MEIO — o Move que leva a guia — mora no `on_cursor_moved`, que é o
        // handler que o winit usa para movimento; **este só recebe Down e Up**.
        //
        // ⚠️ E o Up **não é gateado no Primary**, pelo mesmo motivo que abre o `on_mouse_input`
        // com o release da mão: um arrasto que sobrevive ao release fica colado no cursor para
        // sempre, e um botão secundário não é um modificador de gesto.
        // ⭐ **A BORDA DA COLUNA redimensiona** (Enio, 2026-08-30). Vem antes de tudo pelo mesmo
        // motivo que o gesto da guia: a costura vive DENTRO da coluna, por cima do corpo do
        // painel — sem a precedência, o painel come o press e a borda fica inerte.
        if kind == PointerKind::Up && self.dock_seam_up() {
            return;
        }
        if kind == PointerKind::Down
            && mapped_button == ph2d_host::PointerButton::Primary
            && !menu_open_before
            && self.dock_seam_down(evt.x, evt.y)
        {
            return;
        }
        if kind == PointerKind::Up && self.guide_pointer_up(evt.x, evt.y) {
            return;
        }
        if kind == PointerKind::Down
            && mapped_button == ph2d_host::PointerButton::Primary
            && !menu_open_before
            && self.guide_pointer_down(evt.x, evt.y)
        {
            return;
        }
        // Flip sculpt (W5): o pen-UP encerra o gesto (a máscara congelada morre com
        // ele; o passo de undo sai do diff pós-frame).
        if kind == PointerKind::Up
            && mapped_button == ph2d_host::PointerButton::Primary
            && self.flip_reshape_canvas_up()
        {
            return;
        }
        // Flip Edit Mode (W6.1): o pen-UP fecha o marquee (aplicando a seleção) ou o
        // move. Como os outros UPs, ele vem ANTES dos DOWNs e não depende do modo atual
        // (o gesto pode ter começado antes de uma troca de modo).
        if kind == PointerKind::Up
            && mapped_button == ph2d_host::PointerButton::Primary
            && self.flip_edit_canvas_up()
        {
            return;
        }
        // ADR-0114 C2: o pen-UP fecha um RABISCO do Colorize e o acumula no buffer (as
        // regiões só nascem no Apply). Como os outros UPs, vem antes dos DOWNs.
        if kind == PointerKind::Up
            && mapped_button == ph2d_host::PointerButton::Primary
            && self.flip_colorize_canvas_up()
        {
            return;
        }
        // Shift & Trace: o pen-UP fecha o arrasto de trace (o deslocamento já está
        // aplicado — é exibição, não documento). Como os outros UPs, vem antes dos DOWNs.
        if kind == PointerKind::Up
            && mapped_button == ph2d_host::PointerButton::Primary
            && self.flip_trace_canvas_up()
        {
            return;
        }
        // Flip W7.5: o pen-UP fecha um arrasto do gizmo de pose (o passo de undo sai
        // do diff pós-frame, como os outros gestos).
        if kind == PointerKind::Up
            && mapped_button == ph2d_host::PointerButton::Primary
            && self.flip_pose_gizmo_up()
        {
            return;
        }
        // Flip §4.A: o pen-UP fecha um arrasto do gizmo de seleção (idem — undo pós-frame).
        if kind == PointerKind::Up
            && mapped_button == ph2d_host::PointerButton::Primary
            && self.flip_selection_gizmo_up()
        {
            return;
        }
        // Motion Nodes: o pen-UP fecha um arrasto do gizmo de field e commita o passo de
        // undo (um arrasto = um passo, como um drag de nó).
        if kind == PointerKind::Up
            && mapped_button == ph2d_host::PointerButton::Primary
            && (self.warp_gizmo_up() || self.field_gizmo_up())
        {
            return;
        }
        if self.flip_wants_canvas()
            && kind == PointerKind::Down
            && mapped_button == ph2d_host::PointerButton::Primary
            && on_canvas
            && !menu_open_before
            && self.flip_canvas_down(self.last_pointer.0, self.last_pointer.1)
        {
            return;
        }
        // Tween v2 — correção de pares: enquanto a sessão Pairs está aberta, o clique do
        // canvas RE-PAREIA (sobrepõe o modo atual — Draw/Erase/etc — porque é um sub-modo do
        // fluxo de tween, não um modo de desenho). Uma chamada faz tudo (não é arrasto).
        if self.flip_wants_tween_pairs()
            && kind == PointerKind::Down
            && mapped_button == ph2d_host::PointerButton::Primary
            && on_canvas
            && !menu_open_before
            && self.flip_tween_pairs_canvas_down(self.last_pointer.0, self.last_pointer.1)
        {
            return;
        }
        // Flip bucket (W4): um CLIQUE no modo Fill preenche a região sob o cursor.
        // Não é um gesto de arrasto — uma chamada faz tudo.
        if self.flip_wants_fill()
            && kind == PointerKind::Down
            && mapped_button == ph2d_host::PointerButton::Primary
            && on_canvas
            && !menu_open_before
            && self.flip_fill_canvas_down(self.last_pointer.0, self.last_pointer.1)
        {
            return;
        }
        // ADR-0114 C2: a pen-DOWN no modo Colorize começa um RABISCO (arrasto), como o Draw.
        if self.flip_wants_colorize()
            && kind == PointerKind::Down
            && mapped_button == ph2d_host::PointerButton::Primary
            && on_canvas
            && !menu_open_before
            && self.flip_colorize_canvas_down(self.last_pointer.0, self.last_pointer.1)
        {
            return;
        }
        // Shift & Trace: a pen-DOWN no modo Trace pega o fantasma sob o cursor (Ctrl =
        // girar) e CONSOME mesmo errando — o Trace é dono do canvas (cair no gizmo
        // moveria o objeto no meio do posicionamento da referência).
        if self.flip_wants_trace()
            && kind == PointerKind::Down
            && mapped_button == ph2d_host::PointerButton::Primary
            && on_canvas
            && !menu_open_before
            && self.flip_trace_canvas_down(self.last_pointer.0, self.last_pointer.1)
        {
            return;
        }
        // Flip eraser (T2.9): a pen-DOWN in Erase mode on the canvas begins an
        // erase gesture (Select falls through to gizmo/pick, like Draw).
        if self.flip_wants_erase()
            && kind == PointerKind::Down
            && mapped_button == ph2d_host::PointerButton::Primary
            && on_canvas
            && !menu_open_before
            && self.flip_erase_canvas_down(self.last_pointer.0, self.last_pointer.1)
        {
            return;
        }
        // Flip sculpt (W5): a pen-DOWN no modo Reshape começa um gesto de escultura
        // (Select cai no gizmo/pick, como o Draw e a borracha).
        if self.flip_wants_reshape()
            && kind == PointerKind::Down
            && mapped_button == ph2d_host::PointerButton::Primary
            && on_canvas
            && !menu_open_before
            && self.flip_reshape_canvas_down(self.last_pointer.0, self.last_pointer.1)
        {
            return;
        }
        // Flip W7.5: um handle do gizmo de POSE sob o cursor abre o arrasto de pose
        // (rotate/scale da instância). Vem ANTES do arm de canvas do Edit — um handle
        // registrado no hit-index torna `on_canvas` falso, então sem este arm o clique
        // cairia no caminho genérico de gizmo (que escreve o `Transform` do OBJETO).
        // O método só consome quando o hit é `GizmoTarget::FlipPose`.
        if kind == PointerKind::Down
            && mapped_button == ph2d_host::PointerButton::Primary
            && !menu_open_before
            && self.flip_pose_gizmo_down(self.last_pointer.0, self.last_pointer.1)
        {
            return;
        }
        // Flip §4.A: um handle do gizmo de SELEÇÃO sob o cursor abre o arrasto de
        // seleção (rotate/scale assado nos pontos). Como o da pose, vem ANTES do arm de
        // canvas do Edit — um handle no hit-index torna `on_canvas` falso. O método só
        // consome quando o hit é `GizmoTarget::FlipSelection` (mutuamente exclusivo com
        // o da pose, então nunca disputam o mesmo clique).
        if kind == PointerKind::Down
            && mapped_button == ph2d_host::PointerButton::Primary
            && !menu_open_before
            && self.flip_selection_gizmo_down(self.last_pointer.0, self.last_pointer.1)
        {
            return;
        }
        // Motion Nodes: um handle do gizmo de FIELD sob o cursor abre o arrasto (rotate/
        // scale/translate escrito nos params do NÓ). Como os do Flip, vem ANTES do caminho
        // genérico de gizmo — um handle no hit-index o alcançaria e escreveria um
        // `Transform`. O método só consome quando o hit é `GizmoTarget::MotionField`, que
        // só existe com a tool Motion ativa + um field espacial selecionado no grafo.
        // Motion Nodes: uma ALÇA do gizmo dos deformadores de quadrilátero (Corner Pin +
        // Bezier Warp) sob o cursor abre o arrasto, que escreve os params do NÓ. Vem antes
        // do gizmo de field e do genérico pela mesma razão que aquele: uma alça alcançada
        // pelo caminho genérico escreveria um `Transform` de entidade. O método só consome
        // quando há retrato publicado — ou seja com a tool Motion activa e um dos dois nós
        // seleccionado.
        //
        // ⚠️ **E ele exige `on_canvas`, ao contrário dos irmãos** — Enio, 2026-08-23:
        // *"se colocar transform antes, não é possível conectar transform em Bezier
        // Warp"*. Os gizmos acima consomem pelo HIT-INDEX (`GizmoTarget::…`), que já
        // sabe das regiões; este faz o seu próprio hit-test em coordenadas de MUNDO, e
        // sem guarda ele convertia um clique **no painel do grafo** para o mundo, calhava
        // de cair sobre uma alça, e ENGOLIA o gesto de ligar um fio. *Um consumidor que
        // decide sozinho tem de saber sozinho onde ele vale.*
        if kind == PointerKind::Down
            && mapped_button == ph2d_host::PointerButton::Primary
            && !menu_open_before
            && on_canvas
            && self.warp_gizmo_down(self.last_pointer.0, self.last_pointer.1)
        {
            return;
        }
        if kind == PointerKind::Down
            && mapped_button == ph2d_host::PointerButton::Primary
            && !menu_open_before
            && self.field_gizmo_down(self.last_pointer.0, self.last_pointer.1)
        {
            return;
        }
        // Flip Edit Mode (W6): um CLIQUE no modo Edit seleciona o TRAÇO sob o cursor
        // (Shift alterna; no vazio, desmarca). Consome mesmo errando o traço — no Edit o
        // gizmo de objeto não manda, senão o arrasto seguinte moveria o objeto inteiro.
        if self.flip_wants_edit()
            && kind == PointerKind::Down
            && mapped_button == ph2d_host::PointerButton::Primary
            && on_canvas
            && !menu_open_before
            && self.flip_edit_canvas_down(self.last_pointer.0, self.last_pointer.1)
        {
            return;
        }
        // "Set Center" armado (ADR-0112): a pressão põe a ORIGEM da forma selecionada
        // sob o cursor e desarma. Vale em QUALQUER modo — inclusive Select, onde o
        // pivô do gizmo é o que se está ajustando.
        if self.vec.pivot_edit
            && self.vector_tool_active()
            && mapped_button == ph2d_host::PointerButton::Primary
            && kind == PointerKind::Down
            && on_canvas
            && !menu_open_before
        {
            self.vec.pivot_edit = false;
            if self.vec_set_origin_to_cursor(evt.x, evt.y) {
                return;
            }
        }
        // Duplo-clique num TEXTO no modo Select ⇒ entra na edição dele (o gesto padrão
        // de todo editor vetorial). Antes do bloco abaixo porque no Select a tool NÃO
        // captura o canvas — sem isto a pressão iria para o gizmo e arrastaria a forma.
        //
        // **NÃO use `on_canvas` aqui**: ele exige o `hit_index` VAZIO sob o cursor, e o
        // gizmo REGISTRA os hits dele no `hit_index` (as alças + o interior "Translate").
        // Como o 1º clique do par SELECIONA o objeto, o gizmo passa a cobrir a forma —
        // então no 2º clique `on_canvas` é falso e o duplo-clique nunca dispararia (o bug
        // do 1º smoke). O que vale aqui é: fora de painel, e o único widget sob o cursor
        // pode ser o gizmo — que é exatamente o que está por cima do texto.
        if self.vector_tool_active()
            && self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::Select
            && mapped_button == ph2d_host::PointerButton::Primary
            && kind == PointerKind::Down
            && !menu_open_before
            && self.over_canvas_or_gizmo(evt.x, evt.y)
            && self.vec_text_double_click(evt.x, evt.y)
        {
            return;
        }
        // **As alças de ponta do conector** (os dois círculos), no modo Select. Mesmo lugar e
        // mesma razão do duplo-clique de texto acima: no Select a tool não captura o canvas, e
        // sem este arm a pressão iria para o picking/gizmo — que selecionaria a forma ATRÁS da
        // alça em vez de arrastá-la.
        //
        // O `conn_handle_down` só devolve `true` quando o cursor está mesmo sobre uma alça de
        // um conector SELECIONADO; em qualquer outro caso o clique segue o caminho de sempre.
        // É esse contrato que mantém o resto do editor intacto.
        if self.vector_tool_active()
            && self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::Select
            && mapped_button == ph2d_host::PointerButton::Primary
            && kind == PointerKind::Down
            && !menu_open_before
            && self.over_canvas_or_gizmo(evt.x, evt.y)
            && let Some(w) = self.vec_world_at((evt.x, evt.y))
            && self.conn_handle_down(w)
        {
            return;
        }
        // **O Picker de caminho-guia ARMADO** (Enio 2026-07-23): enquanto se escolhe um guia, o
        // clique no canvas PRENDE (ou desiste no vazio) — nunca seleciona nem arrasta ficha. Por
        // isso precede as alças e o picking/gizmo: um pick em curso é modal, e o clique é dele.
        if self.vector_tool_active()
            && self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::Select
            && mapped_button == ph2d_host::PointerButton::Primary
            && kind == PointerKind::Down
            && !menu_open_before
            && self.vec.path_pick.is_some()
            && self.over_canvas_or_gizmo(evt.x, evt.y)
            && let Some(w) = self.vec_world_at((evt.x, evt.y))
        {
            self.vec_path_pick_click(w);
            return;
        }
        // ⭐⭐⭐ **O PICK DO ALVO de um osso inteligente** — a mesma classe modal, e **independente
        // de ferramenta** de propósito.
        //
        // ⛔⛔ **Report do dono (2026-09-08): *«Pick object deve inibir a criação de bones. Ao tentar
        // fazer o pick no canvas criou um osso indesejado»*.** Ele arma o pick a partir da secção
        // Skeleton, logo está na ferramenta **Bone** — onde um `Down` no canvas **cria um osso**. A
        // 1.ª versão deste pick não consumia o press: ela esperava que a SELECÇÃO mudasse, e no modo
        // *Criar* o clique não selecciona coisa nenhuma, **desenha**.
        //
        // ⇒ *um pick modal que não consome o press herda o gesto da ferramenta em que foi armado*, e
        // a ferramenta em que este é armado é a única que CRIA no clique. Precede as alças e o
        // picking/gizmo, como os irmãos.
        if self.skeleton.smart_pick.is_some()
            && mapped_button == ph2d_host::PointerButton::Primary
            && kind == PointerKind::Down
            && !menu_open_before
            && self.over_canvas_or_gizmo(evt.x, evt.y)
        {
            self.smart_pick_click(evt.x, evt.y);
            return;
        }
        // ⭐⭐⭐ **AS ALÇAS DO OSSO PEGAM EM TODO MODO DE VECTOR, e não só no modo Osso.**
        //
        // ⛔⛔ **Achado da auditoria de 2026-09-08:** o arco de limite — e a alça da força, e a
        // ponta da corrente — é **pintado e ACENDE sob o rato nos 14 modos** (o
        // `refresh_bone_hover` não se gateia pelo modo, de propósito, porque o `vec_overlay::bones`
        // também não) e o `Down` só era lido dentro do `DrawMode::Bone`. *Um controlo que acende
        // debaixo do dedo e não responde é a espécie de morto que este repo já pagou três vezes.*
        //
        // ⚠️ **QUAIS alças é a porta [`crate::bone_pick::grabbable_outside_bone_mode`]**, e a
        // linha é o VERBO: entram as quatro que nenhuma outra ferramenta sabe exprimir; girar e
        // deslocar ficam com o gizmo de sprite, que já os faz.
        //
        // ⛔ **Dentro do modo Osso este arm NÃO corre** — lá a `bone_gesture::press` decide, e ela
        // distingue *Criar* de *Transformar*: em *Criar*, pousar sobre uma alça só ACENDE o osso,
        // que é o desenho e não um esquecimento.
        //
        // ⛔ **Consome o press**, como os picks modais acima e pela mesma razão: sem o `return;` o
        // gesto cai na cadeia de baixo e o modo Select começa um marquee por cima do arrasto.
        if mapped_button == ph2d_host::PointerButton::Primary
            && kind == PointerKind::Down
            && !menu_open_before
            && self.vector_tool_active()
            && self.vec.draw_config.mode != ph2d_tool_vector::DrawMode::Bone
            && self.over_canvas_or_gizmo(evt.x, evt.y)
            && let Some(h) = self.bone_handle_at((evt.x, evt.y))
        {
            self.skeleton.bone_pose = Some((h.bone, h.part));
            return;
        }
        // **O eyedropper de corpo do joint** (§12) — mesma classe de pick modal do
        // acima, mas independente de ferramenta: armado, o próximo Down no canvas
        // escolhe o corpo sob o cursor e religa aquela ponta. Precede o
        // picking/gizmo; nenhum outro objeto precisa estar pré-selecionado.
        if self.joint_body_pick.is_some()
            && mapped_button == ph2d_host::PointerButton::Primary
            && kind == PointerKind::Down
            && !menu_open_before
            && self.over_canvas_or_gizmo(evt.x, evt.y)
        {
            self.joint_body_pick_click(evt.x, evt.y);
            return;
        }
        // **O eyedropper de MONTAGEM da roldana** (§13, W-Pulley W3) — a mesma
        // classe de pick modal, uma família adiante: armado, o próximo Down
        // escolhe o CORPO em que o eixo daquela roldana se monta.
        if self.wheel_body_pick.is_some()
            && mapped_button == ph2d_host::PointerButton::Primary
            && kind == PointerKind::Down
            && !menu_open_before
            && self.over_canvas_or_gizmo(evt.x, evt.y)
        {
            self.wheel_mount_pick_click(evt.x, evt.y);
            return;
        }
        // **O eyedropper de CORDA da roldana** (§13, W-Pulley W1) — a mesma classe
        // de pick modal, e o único cujo alvo NÃO é um sprite: uma corda é uma
        // linha, e ela é apontada pela ROTA que o overlay desenha.
        if self.wheel_rope_pick.is_some()
            && mapped_button == ph2d_host::PointerButton::Primary
            && kind == PointerKind::Down
            && !menu_open_before
            && self.over_canvas_or_gizmo(evt.x, evt.y)
        {
            self.wheel_rope_pick_click(evt.x, evt.y);
            return;
        }
        // **A EXPLOSÃO e a ATRAÇÃO** (W-Hand) — modal como os picks acima e pela
        // MESMA razão estrutural: elas precisam só de um PONTO, então não podem
        // pendurar no pick de canvas (que só dispara quando há algo sob o cursor).
        // A MÃO fica onde estava, dentro do pick, para a seleção seguir acontecendo;
        // quem decide de qual família a ferramenta é é `needs_a_body`, uma porta só.
        if mapped_button == ph2d_host::PointerButton::Primary
            && kind == PointerKind::Down
            && !menu_open_before
            && !self.physics.interaction.tool.needs_a_body()
            && self.over_canvas_or_gizmo(evt.x, evt.y)
            && self.poke_press(evt.x, evt.y)
        {
            return;
        }
        // **O gesto de DESENHAR um joint** (W-J4) — a 2ª rota de criação, e a única
        // em que as âncoras nascem NOS pontos que a mão indicou. Modal como o
        // eyedropper acima (precede picking/gizmo, independe de ferramenta): o
        // press começa a banda elástica, o Move a estica, o release cria.
        if self.physics.joint_draw_armed
            && mapped_button == ph2d_host::PointerButton::Primary
            && kind == PointerKind::Down
            && !menu_open_before
            && self.over_canvas_or_gizmo(evt.x, evt.y)
            && self.joint_draw_press(evt.x, evt.y)
        {
            return;
        }
        if self.physics.joint_draw.is_some() {
            match kind {
                PointerKind::Move => {
                    self.joint_draw_move(evt.x, evt.y);
                    return;
                }
                PointerKind::Up => {
                    self.joint_draw_release(evt.x, evt.y);
                    return;
                }
                _ => {}
            }
        }
        // **A alça do TEXTO EM CAMINHO** (W5), no modo Select — irmã da do conector, mesmo lugar
        // e mesma razão: no Select a tool não captura o canvas, e sem este arm a pressão iria
        // para o picking/gizmo. O gizmo é inócuo sobre um texto vinculado (vive na identidade),
        // então o Select é a casa natural da alça — e sem as âncoras do Node ela não se confunde
        // com ponto de objeto nenhum (Enio, smoke). Só devolve `true` sobre a alça de um texto
        // vinculado SELECIONADO; qualquer outro caso segue o caminho de sempre.
        if self.vector_tool_active()
            && self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::Select
            && mapped_button == ph2d_host::PointerButton::Primary
            && kind == PointerKind::Down
            && !menu_open_before
            && self.over_canvas_or_gizmo(evt.x, evt.y)
            && let Some(w) = self.vec_world_at((evt.x, evt.y))
            && self.vec_textpath_handle_down(w)
        {
            return;
        }
        // O MESMO guard para as alças do PATTERN (W4): no Select a tool não captura o canvas e o
        // gizmo é inócuo sobre um motivo vinculado, então a ficha precisa deste arm antes do
        // picking/gizmo. Irmão do `vec_textpath_handle_down` logo acima.
        if self.vector_tool_active()
            && self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::Select
            && mapped_button == ph2d_host::PointerButton::Primary
            && kind == PointerKind::Down
            && !menu_open_before
            && self.over_canvas_or_gizmo(evt.x, evt.y)
            && let Some(w) = self.vec_world_at((evt.x, evt.y))
            && self.vec_patternpath_handle_down(w)
        {
            return;
        }
        // **DUPLO-clique no CAMINHO insere um ponto** (ADR-0141), ANTES do arrasto de âncora:
        // um duplo-clique sobre a curva é "adicionar ponto", não "arrastar". O 1º clique do
        // par devolve `false` (não é duplo) e cai adiante como um clique normal; só o 2º sobre
        // a curva consome.
        if mapped_button == ph2d_host::PointerButton::Primary
            && kind == PointerKind::Down
            && !menu_open_before
            && self.over_canvas_or_gizmo(evt.x, evt.y)
            && self.motion_path_curve_double_click(evt.x, evt.y)
        {
            return;
        }
        // **A ÂNCORA do MOTION PATH** (ADR-0141), antes do picking/gizmo pela mesma razão
        // que as alças acima: a âncora do primeiro key cai em cima do sprite, e sem este
        // arm a pressão iria para o gizmo — que moveria o OBJETO onde o dedo pediu a
        // CURVA. Sem gate de ferramenta de propósito (ver `motion_path_anchor_down`).
        if mapped_button == ph2d_host::PointerButton::Primary
            && kind == PointerKind::Down
            && !menu_open_before
            && self.over_canvas_or_gizmo(evt.x, evt.y)
            && self.motion_path_anchor_down(evt.x, evt.y)
        {
            return;
        }
        // O Up que FECHA o arrasto de alça (ele nasceu no Select, e é lá que morre).
        if self.vec.conn_handle.is_some()
            && mapped_button == ph2d_host::PointerButton::Primary
            && kind == PointerKind::Up
        {
            if let Some(w) = self.vec_world_at((evt.x, evt.y)) {
                self.conn_handle_up(w);
            } else {
                self.conn_handle_cancel();
            }
            return;
        }
        // O Up que fecha o arrasto da alça do texto — nasceu no Select, morre no Select.
        if self.vec.textpath_handle_drag
            && mapped_button == ph2d_host::PointerButton::Primary
            && kind == PointerKind::Up
        {
            self.vec.textpath_handle_drag = false;
            return;
        }
        // ⭐⭐⭐ **O Up que fecha o arrasto de uma ALÇA DE OSSO.**
        //
        // ⚠️ Ela CONSOME o gesto: sem isto, soltar depois de girar um osso cai na cadeia de baixo e
        // a forma sob o cursor é seleccionada.
        //
        // ⛔⛔ **Ele vivia DENTRO do bloco `vector_tool_active() && modo != Select`** e a alça passou
        // a poder ser agarrada em todo modo (auditoria de 2026-09-08) ⇒ no modo **Select** o slot
        // era armado e **nunca** libertado: o osso seguia o rato para sempre, sem botão nenhum
        // apertado. *Um slot de arrasto tem de ser largado onde quer que possa ser agarrado* — e é
        // por isso que ele passou para esta família, que é a dos irmãos independentes de modo.
        if self.skeleton.bone_pose.is_some()
            && mapped_button == ph2d_host::PointerButton::Primary
            && kind == PointerKind::Up
        {
            self.skeleton.bone_pose = None;
            return;
        }
        // O Up que fecha o arrasto de uma ficha do PATTERN (W4) — mesma vida da do texto.
        if self.vec.patternpath_handle.is_some()
            && mapped_button == ph2d_host::PointerButton::Primary
            && kind == PointerKind::Up
        {
            self.vec.patternpath_handle = None;
            return;
        }
        // O Up que fecha o arrasto de uma ÂNCORA do motion path — e que FECHA o passo de
        // undo que o press abriu. Sem este `commit_if_changed` o `begin` fica pendurado e
        // o próximo gesto o herda: um Ctrl+Z desfaria os dois de uma vez.
        if self.motion_shell.path_drag.is_some()
            && mapped_button == ph2d_host::PointerButton::Primary
            && kind == PointerKind::Up
        {
            self.motion_shell.path_drag = None;
            self.timeline.history.commit_if_changed(&self.timeline.doc);
            return;
        }
        // ADR-0112: no modo **Select** a ferramenta não captura o canvas — o clique
        // cai no caminho de sempre (picking de sprite + gizmo), e é assim que uma
        // forma vetorial se transforma. Só Node e os modos de desenho entram aqui.
        if self.vector_tool_active()
            && self.vec.draw_config.mode != ph2d_tool_vector::DrawMode::Select
            && !menu_open_before
        {
            // A canvas press while a text field is focused must blur it (commit the
            // edit) — the pen/shape arms below consume the press and bypass the
            // chrome dispatch that normally does this. Route it explicitly (only a
            // primary press with a field actually focused, so normal draw clicks
            // don't churn dispatch and a right-click can't open a menu here).
            if mapped_button == ph2d_host::PointerButton::Primary
                && kind == PointerKind::Down
                && on_canvas
                && self.text_entry_focused()
            {
                let _ = forward_to_hero(self.gfx.as_mut(), evt);
            }
            // A canvas press dismisses an open colour picker (click-outside closes
            // it, mirroring the chrome light-dismiss). `on_canvas` already excludes
            // the picker rect, so any press reaching here is genuinely outside it —
            // done BEFORE the grad/pen/shape arms so the picker's colour is never
            // applied to the handle the press then selects (Enio 2026-07-08).
            if mapped_button == ph2d_host::PointerButton::Primary
                && kind == PointerKind::Down
                && on_canvas
                && let Some(gfx) = self.gfx.as_mut()
                && let Some(hero) = gfx.hero_screen.as_mut()
                && hero.store.picker_target().is_some()
            {
                hero.store.set_picker_target(None);
            }
            match (mapped_button, kind) {
                // Shift+Down on a PATH → toggle it in the object multi-selection
                // (Align/Distribute); Shift+Down on empty canvas → vertex marquee.
                // Tried first so Shift diverts the press from the pen/shape draw.
                (ph2d_host::PointerButton::Primary, PointerKind::Down)
                    if on_canvas && self.modifiers.shift_key() =>
                {
                    // **Modo Node: Shift+clique num PONTO alterna-o na multi-seleção de pontos**
                    // (Enio 2026-07-15). Tentado ANTES do toggle de OBJETO: no Node é no ponto que
                    // se mexe, e o Shift sobre a forma (que cobre o ponto) alternava o objeto —
                    // somar pontos a dedo era impossível (só o retângulo). Mesmo raio do grab do
                    // Node (`10 px`), então o que se agarra é o que se alterna. Sem ponto sob o
                    // cursor, cai no comportamento de sempre (objeto / marquee).
                    if self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::Node
                        && let Some(wp) = self.vec_world_at(self.last_pointer)
                    {
                        let hit_r = 10.0 * self.vec_px_to_world();
                        // `gfx.vec_scene` e `vec_pen` são campos DISJUNTOS de `self`.
                        if let Some(gfx) = self.gfx.as_ref()
                            && self.vec.pen.toggle_vert_at(&gfx.vec_scene, wp, hit_r)
                        {
                            return;
                        }
                    }
                    let hit = self.gfx.as_ref().and_then(|gfx| {
                        let win = gfx.surface.size();
                        let w = gfx.camera.screen_to_world(self.last_pointer, win);
                        let w0 = gfx.camera.screen_to_world((0.0, 0.0), win);
                        let w1 = gfx.camera.screen_to_world((1.0, 0.0), win);
                        let px =
                            (((w1[0] - w0[0]).powi(2) + (w1[1] - w0[1]).powi(2)).sqrt()) as f64;
                        self.vec
                            .pen
                            .path_at(&gfx.vec_scene, [w[0] as f64, w[1] as f64], 10.0 * px)
                    });
                    if let Some(id) = hit {
                        // Um grupo entra e sai da seleção INTEIRO (a árvore é a
                        // Hierarquia — o ancestral de topo diz quem vem junto).
                        let members = self.vec_object_selection_for(id);
                        self.vec.pen.toggle_object_members(&members);
                        // Object selection changed → drop any gradient-handle selection.
                        self.vec.grad_selected = None;
                        self.vec.grad_drag = None;
                        return;
                    }
                    self.vec.marquee = Some(crate::vec_marquee::VecMarquee::open(
                        self.marquee_shape_for_press(),
                        self.last_pointer,
                    ));
                    return;
                }
                (ph2d_host::PointerButton::Primary, PointerKind::Down) if on_canvas => {
                    // Canvas press priority (most specific first):
                    //   1. "Set Center" armed mode (positions the gizmo pivot).
                    //   2. Gradient handles — tiny (~9 px) and only present when the
                    //      selected path has a gradient fill, so they must outrank the
                    //      gizmo, whose bbox interior otherwise swallows every dot.
                    //   3. Transform gizmo handles (scale / rotate / interior move).
                    //   4. Pen / shape drawing + vertex editing.
                    // **Modo Node: o press no VAZIO abre o retângulo — sem Shift** (plano 25 §6).
                    //
                    // ⚠️ Ele exigia Shift, e o Shift é o modificador de ADIÇÃO em todo app de
                    // desenho: quem quisesse somar nós não tinha tecla, e quem quisesse só o
                    // retângulo tinha de descobrir uma. Agora o gesto é o de todo mundo — arrastar
                    // do vazio desenha a caixa, e o Shift SOMA.
                    //
                    // A pergunta *"o press acerta alguma coisa?"* é feita à porta que já existe
                    // (`node_edit_hit_at` + `path_at`), e **antes** do `on_press_node`: ele
                    // desseleciona quando não acerta nada, e um marquee aditivo aberto depois disso
                    // somaria a uma seleção que acabou de ser apagada.
                    if self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::Node
                        && let Some(w) = self.vec_world_at(self.last_pointer)
                        && let Some(gfx) = self.gfx.as_ref()
                    {
                        let px = self.vec_px_to_world();
                        let empty = self
                            .vec
                            .pen
                            .node_edit_hit_at(&gfx.vec_scene, w, px)
                            .is_none()
                            && self
                                .vec
                                .pen
                                .path_at(&gfx.vec_scene, w, HANDLE_HIT_PX * px)
                                .is_none();
                        if empty {
                            self.vec.marquee = Some(crate::vec_marquee::VecMarquee::open(
                                self.marquee_shape_for_press(),
                                self.last_pointer,
                            ));
                            return;
                        }
                    }
                    // Gradient group 3b: a Down on a gradient handle starts dragging it.
                    if let Some(i) = self.vec_grad_hit(self.last_pointer) {
                        self.vec.grad_selected = Some(i);
                        self.vec.grad_drag = Some(i);
                        return;
                    }
                    // Modo Text: o clique põe/reposiciona o cursor de texto no ponto
                    // clicado (finalizando a edição anterior). A digitação vem pelo
                    // teclado; nada de shape/pen aqui.
                    if self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::Text {
                        let w = self.gfx.as_ref().map(|gfx| {
                            gfx.camera
                                .screen_to_world(self.last_pointer, gfx.surface.size())
                        });
                        if let Some(w) = w {
                            self.vec_text_click([f64::from(w[0]), f64::from(w[1])]);
                        }
                        return;
                    }
                    // Modo Build (Shape Builder): a pressão começa a PINTAR faces do
                    // arranjo. Captura o canvas inteiro — não há pen, shape nem gizmo aqui;
                    // o que se manipula não é a forma, é a região.
                    if self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::Build {
                        if let Some(w) = self.vec_world_at(self.last_pointer) {
                            let alt = self.modifiers.alt_key();
                            let shift = self.modifiers.shift_key();
                            self.build_down(w, alt, shift);
                        }
                        return;
                    }
                    // **Modo Lápis**: a pressão abre um traço de mão livre. O gesto é INTEIRO
                    // dele (press/move/release), como o Build e o Connect — não há hit-test a
                    // fazer: um lápis desenha onde você encostou.
                    if self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::Pencil {
                        let px_to_world = self.vec_px_to_world();
                        let dyn_in = self.pointer_dynamics();
                        if let Some(w) = self.vec_world_at(self.last_pointer)
                            && let Some(gfx) = self.gfx.as_mut()
                        {
                            self.vec
                                .pencil
                                .on_press(&mut gfx.vec_scene, w, px_to_world, dyn_in);
                        }
                        // O estabilizador começa ONDE A MÃO ENCOSTOU. Sem esta semente o 1º move
                        // mistura a partir de onde o gesto ANTERIOR acabou, e o traço nasce com um
                        // salto vindo do outro lado da tela. Fora do `if let` de propósito: ele
                        // depende só do ponteiro, e semear a mão nunca pode ficar refém de a cena
                        // estar pronta — o move consome esta posição sem perguntar mais nada.
                        self.vec.pencil_hand.begin(self.last_pointer);
                        return;
                    }
                    // Modo Connect: a pressão abre o gesto do CONECTOR (sobre uma forma, a
                    // ponta nasce presa a ela; no vazio, solta ali). Nada de pen/shape —
                    // a linha de um conector não é autorada, é derivada.
                    if self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::Connect {
                        if let Some(w) = self.vec_world_at(self.last_pointer) {
                            self.connector_down(w);
                        }
                        return;
                    }
                    // Modo Pick Shapes (Blend): a pressão coleta a forma FECHADA sob o
                    // cursor na ordem de clique (ADR-0128 C2b). Não há pen/shape/gizmo — o
                    // que se escolhe é a LISTA de formas, e o botão Blend a liga. Clicar de
                    // novo numa já escolhida a remove (corrigir sem recomeçar).
                    if self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::PickBlend {
                        if let Some(w) = self.vec_world_at(self.last_pointer) {
                            self.blend_pick_at(w);
                        }
                        return;
                    }
                    // Modos **Fillet / Chamfer**: a pressão agarra a QUINA sob o cursor e arma o
                    // arrasto de raio (o dedo dita a MAGNITUDE, a ferramenta o ESTILO). Só o press
                    // é próprio — move e release reusam o caminho do pen (o arrasto é guiado pelo
                    // `grab`, o release comita um passo). "Basta clicar numa quina", e um ponto
                    // SUAVE é primeiro transformado em quina (`on_press_corner`).
                    // **Modo Width**: a pressão agarra a alça de largura sob o cursor, ou
                    // ACRESCENTA uma parada se o cursor está sobre a curva (plano 25 §5). O gesto
                    // é inteiro dele — o `Grab` armado dita o move, e o release comita um passo.
                    if self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::Width {
                        let px_to_world = self.vec_px_to_world();
                        if let Some(world) = self.vec_world_at(self.last_pointer) {
                            let hit_r = HANDLE_HIT_PX * px_to_world;
                            // (Re)seleciona o caminho sob o cursor — o gesto vale sem
                            // pré-selecionar, como o das ferramentas de quina.
                            if let Some(gfx) = self.gfx.as_mut()
                                && let Some(pid) =
                                    self.vec.pen.path_at(&gfx.vec_scene, world, hit_r)
                            {
                                self.vec.pen.select(Some(pid));
                            }
                            if let Some(pid) = self.vec.pen.selected()
                                && let Some(gfx) = self.gfx.as_mut()
                            {
                                let scene = &gfx.vec_scene;
                                self.vec.width_grab = crate::width_handles::press(
                                    &mut gfx.sim,
                                    scene,
                                    &self.vec.entities,
                                    pid,
                                    world,
                                    hit_r,
                                );
                            }
                        }
                        return;
                    }
                    // **Modo Corte** (W4): NÃO há early return aqui, e é o desenho inteiro — a
                    // linha de corte é desenhada pela CANETA, que é o caminho por onde este press
                    // cai adiante. Uma rota própria seria uma segunda resposta a *"como se desenha
                    // uma curva?"*, e ela divergiria da caneta no primeiro refino (handles,
                    // fechamento, snap, continuar por um endpoint — tudo isto sai de graça).
                    //
                    // O que o modo muda é só o que a caneta PRODUZ: o caminho nasce marcado como
                    // lâmina (`vec_cut_line::adopt_new_path`, depois do `sync`).
                    // ⭐⭐⭐ **APARAR** (plano 38): o clique apaga o pedaço que o realce está a
                    // mostrar. ⚠️ **O pedaço vem do estado do QUADRO** (`vec_trim_hit`), e não de
                    // um cálculo feito aqui: o que o artista vê a vermelho é literalmente o que
                    // some. Recalcular no clique abriria a porta para o cursor ter andado um pixel
                    // entre o desenho e o gesto — e numa ferramenta destrutiva isso é apagar outra
                    // coisa.
                    //
                    // ⚠️ **A forma VIVA congela a receita AQUI**, como no Fillet/Chamfer: um corte
                    // não sobrevive ao `recook_into`, então sem isto o pedaço voltaria no quadro
                    // seguinte e a ferramenta leria como *"não funciona"*.
                    if self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::Trim {
                        if let Some(hit) = self.vec.trim_hit
                            && let Some(gfx) = self.gfx.as_mut()
                        {
                            crate::vec_convert::freeze_shape_recipe(
                                &mut gfx.sim,
                                &self.vec.entities,
                                hit.path,
                            );
                            if crate::vec_trim::apply(&mut gfx.vec_scene, &hit) {
                                // A selecção pode ter deixado de existir (a peça toda saiu).
                                if gfx.vec_scene.path(hit.path).is_none() {
                                    self.vec.pen.select(None);
                                }
                            }
                            self.vec.trim_hit = None;
                            self.vec.trim_piece.clear();
                        }
                        // ⛔ Consome o press SEMPRE que a ferramenta está na mão: um clique no
                        // vazio não pode cair na cadeia de baixo e começar a desenhar uma forma.
                        return;
                    }
                    // ⭐⭐⭐ **O BALDE** (plano 40): o clique deposita a face que o realce está a
                    // mostrar. ⚠️ **A geometria vem do estado do QUADRO** (`vec_bucket_face`), como
                    // no Trim: o que o artista vê aceso é literalmente o que fica.
                    //
                    // ⛔ Consome o press SEMPRE, pela razão do Trim: um clique no vazio não pode
                    // cair na cadeia de baixo e começar a desenhar uma forma.
                    if self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::Bucket {
                        // ⚠️ A guarda de fora é a de sempre: o `apply_bucket` pergunta a tinta ANTES das guardas
                        // dele (e avisa se ela for transparente), então chamá-lo sem face nem `gfx` imprimiria um
                        // aviso que este clique nunca imprimiu.
                        if self.vec.bucket_face.is_some() && self.gfx.is_some() {
                            self.apply_bucket();
                        }
                        return;
                    }
                    // ⭐⭐⭐ **O OSSO** (estudo 42 item 5, doc 47 §2.6): apontar um osso
                    // SELECCIONA-o (é assim que se ramifica); o vazio marca a ORIGEM, e o `release`
                    // faz o osso dali até onde a mão soltou.
                    //
                    // ⛔ Consome o press SEMPRE que a ferramenta está na mão, pela razão do Trim e
                    // do Balde: um clique no vazio não pode cair na cadeia de baixo e começar a
                    // desenhar uma forma.
                    // ⭐⭐⭐ **O OSSO** (estudo 42 item 5, doc 47 §2.6): a DECISÃO vive na porta
                    // única `bone_gesture::press` — aqui ficam só os efeitos. ⚠️ Foi tê-la dentro
                    // deste ficheiro que escondeu a metade que faltava (report do Enio,
                    // 2026-09-06: *"o bind não funciona"* — apontar uma forma nunca a
                    // seleccionava, e o botão só sabia recusar).
                    //
                    // ⛔ Consome o press SEMPRE que a ferramenta está na mão, pela razão do Trim e
                    // do Balde: um clique no vazio não pode cair na cadeia de baixo e começar a
                    // desenhar uma forma.
                    if self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::Bone {
                        if let Some(world) = self.vec_world_at(self.last_pointer) {
                            let px = self.vec_px_to_world();
                            let sel = self.selected_bone_bits();
                            // ⭐ O VERBO do arrasto, que o grupo alternável da seção SKELETON diz.
                            let acao = self.vec.draw_config.bone_action;
                            let decisao = self.gfx.as_ref().map(|g| {
                                crate::bone_gesture::press(
                                    &g.sim,
                                    &g.vec_scene,
                                    &self.vec.pen,
                                    world,
                                    px,
                                    sel,
                                    acao,
                                )
                            });
                            match decisao {
                                Some(crate::bone_gesture::BonePress::Grab { bone, part }) => {
                                    // Agarrar o osso é o gesto de o POSAR (o gizmo de sprite não
                                    // serve — ver `bone_pose::pose`), e também o que o
                                    // selecciona: o pai do próximo osso é o que está aceso.
                                    self.skeleton.bone_pose = Some((bone, part));
                                    if let Some(gfx) = self.gfx.as_mut()
                                        && let Some(hero) = gfx.hero_screen.as_mut()
                                    {
                                        hero.gizmo.selection = Some(bone);
                                        hero.gizmo.extra_selection.clear();
                                    }
                                }
                                // ⭐ Em *Transformar*, um press fora de osso aponta a forma e mais
                                // nada — o *Bind* precisa do sujeito, e nenhum osso nasce aqui.
                                Some(crate::bone_gesture::BonePress::Pick { path: Some(pid) }) => {
                                    self.vec.pen.select(Some(pid));
                                }
                                // ⛔ Sem forma sob o cursor, um press em *Transformar* não faz
                                // NADA — nem cria, nem DESMARCA: desmarcar tiraria o sujeito do
                                // `Bind` a cada clique no vazio, e o artista clica no vazio o tempo
                                // todo.
                                Some(crate::bone_gesture::BonePress::Pick { path: None }) => {}
                                Some(crate::bone_gesture::BonePress::Start { birth, pick }) => {
                                    self.skeleton.bone_drag = Some(birth);
                                    // ⚠️ **O clique que SELECCIONA e o arrasto que faz osso são o
                                    // MESMO press**, e é de propósito: um clique curto (< 12 px)
                                    // não faz osso nenhum, então apontar uma forma é só apontar —
                                    // e é assim que o *Bind* passa a ter sujeito.
                                    if let Some(pid) = pick {
                                        self.vec.pen.select(Some(pid));
                                    }
                                }
                                None => {}
                            }
                        }
                        return;
                    }
                    if self.vec.draw_config.mode.is_corner_tool() {
                        let chamfer = self.vec.draw_config.mode.corner_is_chamfer();
                        let px_to_world = self.vec_px_to_world();
                        if let Some(world) = self.vec_world_at(self.last_pointer)
                            && let Some(gfx) = self.gfx.as_mut()
                        {
                            let hit_r = 12.0 * px_to_world;
                            // (re)seleciona o path sob o cursor num acerto — o gesto vale sem
                            // pré-selecionar; num erro mantém a seleção (uma quina do path já
                            // selecionado ainda pega).
                            if let Some(pid) = self.vec.pen.path_at(&gfx.vec_scene, world, hit_r) {
                                self.vec.pen.select(Some(pid));
                            }
                            // **A forma VIVA congela a receita AQUI** (Enio: *"fillet e chanfer
                            // nao funciona diretamente nos vertex das shapes"*). Um raio
                            // por-vértice não sobrevive ao `recook_into`, então antes a
                            // ferramenta RECUSAVA a forma — o que lê como "não funciona". Agora
                            // ela faz, dentro do gesto, o "Convert to Curves" que o artista faria
                            // à mão. Só com a quina de fato ACERTADA: congelar num clique que
                            // erra expandiria a forma sem ninguém pedir.
                            if let Some(pid) = self.vec.pen.selected()
                                && self
                                    .vec
                                    .pen
                                    .corner_hit_at(&gfx.vec_scene, world, px_to_world)
                            {
                                crate::vec_convert::freeze_shape_recipe(
                                    &mut gfx.sim,
                                    &self.vec.entities,
                                    pid,
                                );
                            }
                            // DIAGNÓSTICO (`PH2D_CORNER_LOG=1`): os raios do path NO INSTANTE do
                            // press. Serve para partir em dois o report *"a 1ª operação é
                            // apagada"*: se os raios anteriores já vêm ZERADOS aqui, quem apagou
                            // foi algo ENTRE os gestos (um passe por-frame); se vêm inteiros e
                            // somem depois, foi o gesto. O motor e o recook da forma viva já
                            // estão provados limpos por gate, então o eraser está fora deles.
                            if std::env::var_os("PH2D_CORNER_LOG").is_some()
                                && let Some(pid) = self.vec.pen.selected()
                            {
                                // `shape` = a receita ainda esta' pendurada? Se ela reaparece
                                // entre gestos, o `recook_into` reescreve `verts` e zera TODOS
                                // os raios de uma vez -- o unico mecanismo que casa com "so' um
                                // raio vivo por vez, com a contagem de vertices intacta".
                                let shape = self.vec.entities.get(&pid).is_some_and(|&b| {
                                    gfx.sim
                                        .world()
                                        .get::<ph2d_ecs::VecShape>(ph2d_ecs::Entity::from_bits(b))
                                        .is_some()
                                });
                                let radii: Vec<f64> = gfx
                                    .vec_scene
                                    .path(pid)
                                    .map(|p| p.verts_all().map(|v| v.corner_radius).collect())
                                    .unwrap_or_default();
                                eprintln!(
                                    "[corner] PRESS chamfer={chamfer} shape={shape} radii={radii:?}"
                                );
                            }
                            // Os hosts de RELAÇÃO (conector, morph, blend, envelope) seguem
                            // recusados: ali a geometria é uma relação, e soltá-la sem o artista
                            // pedir destruiria o que ele construiu. A forma viva já saiu acima.
                            let derived = self.vec.pen.selected().is_some_and(|pid| {
                                crate::corner_handles::has_derived_verts(
                                    &gfx.sim,
                                    &self.vec.entities,
                                    pid,
                                )
                            });
                            if !derived {
                                self.vec.pen.on_press_corner(
                                    &mut gfx.vec_scene,
                                    world,
                                    px_to_world,
                                    chamfer,
                                );
                            }
                        }
                        return;
                    }
                    let shape_kind = shape_kind_for_mode(&self.vec.draw_config);
                    // Alt held → the Pen breaks the tangent when grabbing a handle.
                    let alt = self.modifiers.alt_key();
                    // Snap targets for THIS gesture: the whole scene as it stands.
                    // Rebuilt right after the press, once we know what got grabbed.
                    self.vec_rebuild_snap_targets(&[], &[]);
                    let cfg = self.vec_snap_cfg(self.vec_px_to_world());
                    let targets = std::mem::take(&mut self.vec.snap_targets);
                    if let Some(gfx) = self.gfx.as_mut() {
                        let win = gfx.surface.size();
                        let w = gfx.camera.screen_to_world(self.last_pointer, win);
                        // world-units por pixel (delta de 1px) → limiar/traço em px.
                        let w0 = gfx.camera.screen_to_world((0.0, 0.0), win);
                        let w1 = gfx.camera.screen_to_world((1.0, 0.0), win);
                        let px_to_world =
                            (((w1[0] - w0[0]).powi(2) + (w1[1] - w0[1]).powi(2)).sqrt()) as f64;
                        // ADR-0129 Fatia 3: o alvo do gesto de gaiola é o CONTAINER do envelope (sem
                        // path), não a forma selecionada — os bits dele estão na seleção do gizmo
                        // (regra seleciona-só-o-container). Copy, lido ANTES do borrow mutável de
                        // `hero_screen` abaixo. `press` devolve `false` se não for um `VecEnvelope`.
                        let env_container =
                            gfx.hero_screen.as_ref().and_then(|h| h.gizmo.selection);
                        // `hero_screen` e `vec_scene` são campos IRMÃOS de `AppGfx`: a
                        // grade pode ser consultada enquanto o Pen muta a cena.
                        let mut hero = gfx.hero_screen.as_mut();
                        let mut snap = |p: [f64; 2]| {
                            let mut grid = |q: [f64; 2]| {
                                let h = hero.as_mut()?;
                                crate::vec_snap::ask_grid(&mut h.grid.snap_state, q)
                            };
                            ph2d_vec_edit::snap::snap(&[p], &targets, cfg, Some(&mut grid)).apply(p)
                        };
                        let node_mode =
                            self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::Node;
                        match shape_kind {
                            // Node edita nós e NUNCA cria (ADR-0112). Não encaixa
                            // tampouco: o snap serve a quem POSICIONA um ponto novo.
                            None if node_mode => {
                                // ADR-0129 Fatia 1: os cantos da gaiola do Envelope são
                                // alças PRÓPRIAS no modo Node (§3.3). Hit-testa-os
                                // PRIMEIRO; um acerto arma o arrasto do canto e PULA o pen
                                // — que agarraria uma âncora da forma COZIDA, revertida
                                // pelo recook do frame seguinte. Um erro cai no
                                // `on_press_node` de sempre (seleção / edição de âncora).
                                //
                                // A alça do TEXTO EM CAMINHO (W5) NÃO vive aqui — ela é do
                                // modo **Select** (a bolinha se perdia no meio das âncoras
                                // do Node; Enio, smoke). Vive ao lado das alças do conector,
                                // mais acima neste arquivo.
                                if crate::envelope_gesture::press(
                                    &mut gfx.sim,
                                    &gfx.vec_scene,
                                    &self.vec.live_drawn,
                                    &self.vec.view_derived,
                                    env_container,
                                    [w[0] as f64, w[1] as f64],
                                    px_to_world,
                                    self.modifiers.alt_key(),
                                    &mut self.vec.envelope_drag,
                                ) {
                                    // canto agarrado — o pen fica de fora
                                } else {
                                    // **A forma VIVA congela a receita AQUI**, exatamente como no
                                    // par Fillet/Chamfer acima — e pelo mesmo motivo: o
                                    // `recook_into` reescreve `path.verts` INTEIRO, então um nó
                                    // arrastado numa Live Shape sobrevive até o instante em que o
                                    // artista encosta num slider de parâmetro, e some **sem erro
                                    // nenhum** (o modo de falha que o `corner_handles` descreve;
                                    // medido em `vec_node_freeze_tests`).
                                    //
                                    // ⚠️ Só quando o press vai de fato EDITAR geometria: o
                                    // `on_press_node` devolve `Grabbed` tanto ao agarrar um vértice
                                    // como ao apenas SELECIONAR a forma pelo preenchimento, então a
                                    // pergunta é feita ANTES, à porta que faz a MESMA busca
                                    // (`node_edit_hit_at`) — congelar num clique que só seleciona
                                    // expandiria a forma sem ninguém pedir.
                                    if let Some(pid) = self.vec.pen.node_edit_hit_at(
                                        &gfx.vec_scene,
                                        [w[0] as f64, w[1] as f64],
                                        px_to_world,
                                    ) {
                                        crate::vec_convert::freeze_shape_recipe(
                                            &mut gfx.sim,
                                            &self.vec.entities,
                                            pid,
                                        );
                                    }
                                    // Node edita âncoras/handles. Arredondar/chanfrar quina não é
                                    // mais deste modo — virou o par Fillet/Chamfer (o hit-test aqui
                                    // não agarra alça de raio nenhuma).
                                    self.vec.pen.on_press_node(
                                        &mut gfx.vec_scene,
                                        [w[0] as f64, w[1] as f64],
                                        px_to_world,
                                        alt,
                                    );
                                }
                            }
                            None => {
                                let click = self.vec.pen.on_press(
                                    &mut gfx.vec_scene,
                                    [w[0] as f64, w[1] as f64],
                                    px_to_world,
                                    alt,
                                    &mut snap,
                                );
                                // **O modo Corte só muda o que a caneta PRODUZ.** Um caminho
                                // começado aqui em modo Cut é a LÂMINA, e não desenho: fica
                                // pendente até o `sync` lhe dar entidade, e aí recebe o
                                // `VecCutPath` (o padrão exato do conector e do blend).
                                if self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::Cut
                                    && click == ph2d_vec_edit::PenClick::Started
                                    && let Some(id) = self.vec.pen.selected()
                                {
                                    self.vec.cut_pending = Some(id);
                                }
                            }
                            Some(kind) => {
                                // A ferramenta de forma não faz hit-test: o canto pode
                                // ser encaixado antes de entrar.
                                let p = snap([w[0] as f64, w[1] as f64]);
                                // Os parâmetros cruzam a fronteira de unidade AQUI: a
                                // tool os guarda como o usuário os digita (px nos
                                // raios), a geometria só fala mundo.
                                let values = ph2d_tool_vector::shapes::to_world(
                                    kind,
                                    &self.vec.draw_config.values,
                                    px_to_world,
                                );
                                self.vec.shape.on_press(
                                    &mut gfx.vec_scene,
                                    kind,
                                    values,
                                    p,
                                    px_to_world,
                                    shape_constraint(self.modifiers),
                                );
                            }
                        }
                        self.vec.snap_targets = targets;
                        // Tocar um filho seleciona o GRUPO (a árvore é a Hierarquia).
                        // Depois do press, porque só agora sabemos o que foi agarrado.
                        if let Some(primary) = self.vec.pen.selected() {
                            let members = self.vec_object_selection_for(primary);
                            self.vec.pen.set_object_selection(&members);
                        }
                        // Agora sabemos o que o press agarrou: o que se move sai dos
                        // alvos (uma âncora não pode encaixar em si mesma; a forma em
                        // desenho não é referência de nada).
                        match (self.vec.pen.dragging_anchors(), self.vec.shape.selected()) {
                            // ⚠️ Os pares vêm PRONTOS do pen: ele passou a guardar o dono de cada
                            // nó, então a re-montagem que morava aqui (`map(|&v| (pid, v))`) some
                            // — e com ela o pressuposto de que todas as âncoras em movimento
                            // pertencem à MESMA forma, que um arrasto multi-forma quebra.
                            (Some(moving), _) => self.vec_rebuild_snap_targets(&[], &moving),
                            (None, Some(sid)) => self.vec_rebuild_snap_targets(&[sid], &[]),
                            (None, None) => {}
                        }
                        return;
                    }
                    self.vec.snap_targets = targets;
                }
                (ph2d_host::PointerButton::Primary, PointerKind::Up) => {
                    // Fim de gesto: as guias de snap não sobrevivem ao Up.
                    self.vec_clear_snap_guides();
                    // ⭐⭐⭐ **O OSSO nasce aqui** (estudo 42 item 5): origem no press, comprimento e
                    // ângulo no arrasto, PAI = o osso seleccionado — e o novo fica seleccionado, que
                    // é o que faz arrasto-arrasto-arrasto ser uma cadeia.
                    //
                    // ⚠️ **Consome SÓ com o gesto VIVO** (a origem marcada), pela lei que o
                    // `shape_up_consumes` documenta: soltar sobre um botão do painel neste modo não
                    // pode engolir o clique.
                    if let Some(nascimento) = self.skeleton.bone_drag.take() {
                        let px = self.vec_px_to_world();
                        let mut nasceu = None;
                        if let Some(solto) = self.vec_world_at(self.last_pointer) {
                            // ⭐⭐⭐ **A MESMA leitura que a pré-visualização desenhou**
                            // ([`crate::bone_gesture::drag_now`]): a emenda, a ponta encaixada e o
                            // limiar. O artista viu o osso saltar para aquela bolinha, e é
                            // exactamente ali que ele nasce.
                            //
                            // ⚠️ **A EMENDA** (ordem do dono, 2026-09-09): se o arrasto acaba na
                            // BASE de uma corrente solta, a ponta do osso novo encaixa nela e essa
                            // corrente passa a pendurar-se nele — duas correntes viram uma.
                            let Some(agora) = self.gfx.as_ref().map(|g| {
                                crate::bone_gesture::drag_now(&g.sim, nascimento, solto, px)
                            }) else {
                                return;
                            };
                            let (ponta, emenda) = (agora.tip, agora.splice);
                            if agora.armed
                                && let Some(gfx) = self.gfx.as_mut()
                            {
                                // ⭐⭐⭐ **O PAI é o que o PRESS apontou** (ordem do dono,
                                // 2026-09-09) — ⛔ nunca a selecção, que era a lei que ele mandou
                                // tirar. Ele ainda é filtrado porque um osso pode ter sido apagado
                                // entre o press e o release, e um pai morto não tem espaço local.
                                let pai = nascimento
                                    .parent
                                    .and_then(ph2d_ecs::Entity::try_from_bits)
                                    .filter(|e| {
                                        gfx.sim.world().get::<ph2d_skeleton_ecs::Bone>(*e).is_some()
                                    });
                                nasceu = crate::bone_gesture::create(
                                    &mut gfx.sim,
                                    pai,
                                    nascimento.origin,
                                    ponta,
                                );
                                // ⭐⭐⭐ **E a corrente solta passa a pendurar-se no osso novo.**
                                //
                                // ⚠️ **Depois do `create`, nunca antes:** o pai só existe agora, e
                                // adoptar antes dele nascer não tem onde pendurar. ⚠️ E o `connect`
                                // preserva a pose de MUNDO do adoptado — sem isso o esqueleto
                                // inteiro saltaria pela pose do osso novo.
                                if let (Some(novo), Some((alvo, _))) = (nasceu, emenda) {
                                    crate::bone_gesture::connect(&mut gfx.sim, alvo, novo);
                                }
                                if let Some(bits) = nasceu
                                    && let Some(hero) = gfx.hero_screen.as_mut()
                                {
                                    hero.gizmo.selection = Some(bits);
                                    hero.gizmo.extra_selection.clear();
                                }
                            }
                        }
                        // ⭐⭐⭐ **UM OSSO ACABADO DE NASCER NÃO É UM OSSO ESCOLHIDO** (report do
                        // dono, 2026-09-09: *«cada vez que se cria um osso o modo Transform é
                        // selecionado»*).
                        //
                        // ⛔ O osso novo fica aceso — é assim que o artista vê qual é — e no quadro
                        // seguinte a aresta do foco lia isso como *«o artista escolheu um osso»* e
                        // armava *Transform*, arrancando-o do verbo em que ele estava. A memória
                        // absorve-o AQUI, onde se sabe que ele nasceu de um arrasto e não de uma
                        // escolha. *A aresta continua a valer; o que mudou é quem a alimenta.*
                        if let Some(bits) = nasceu {
                            crate::skeleton_reveal::on_birth(
                                &mut self.skeleton.osso_revelado,
                                bits,
                            );
                        }
                        return;
                    }
                    // Shape Builder: o Up materializa as faces pintadas. Consome SÓ com o
                    // arrasto VIVO, pela mesma razão que o conector documenta abaixo —
                    // um Up sobre um botão do painel não pode ser engolido pelo modo.
                    if self.vec.build.as_ref().is_some_and(|s| s.dragging) {
                        self.build_up();
                        return;
                    }
                    // Conector: o Up prende a 2ª ponta (na forma sob o cursor, ou solta ali).
                    // Consome SÓ com gesto vivo — senão soltar sobre um botão do painel no
                    // modo Connect engoliria o clique (a armadilha do `shape_up_consumes`).
                    if self.vec.connect.is_some() {
                        let w = self.vec_world_at(self.last_pointer);
                        if let Some(w) = w {
                            self.connector_up(w);
                        } else {
                            self.connector_cancel();
                        }
                        return;
                    }
                    // Gradient group 3b: end a gradient-handle drag.
                    if self.vec.grad_drag.take().is_some() {
                        return;
                    }
                    // ADR-0129 Fatia 1: fim de um arrasto de canto da gaiola. O
                    // `VecEnvelope` alterado vira UM passo no diff global do undo ao
                    // soltar — o `held_button` suprimiu os frames intermediários
                    // (`post_frame_undo`), então não há `commit_if_changed` a chamar aqui
                    // (esse é o histórico do PEN; o envelope viaja no `WorldSnapshot`).
                    // Consome só quando havia um canto vivo.
                    if self.vec.envelope_drag.take().is_some() {
                        return;
                    }
                    // (A alça do texto em caminho é do modo Select — o Up dela mora lá em cima,
                    // ao lado do Up do conector; não aqui, que é o caminho de Node.)
                    // Fim do gesto de REGIÃO → selecciona as âncoras dentro dela.
                    if let Some(m) = self.vec.marquee.take() {
                        // **Shift SOMA** (o retângulo de todo app); sem ele, substitui. E uma
                        // região de tamanho zero é um CLIQUE no vazio: ela desseleciona, em vez
                        // de fazer um select que não apanha nada e deixa a seleção intacta.
                        let additive = self.modifiers.shift_key();
                        let (start, cur) = (m.start, m.cur);
                        let moved = (start.0 - cur.0).abs() > 1.0 || (start.1 - cur.1).abs() > 1.0;
                        if let Some(gfx) = self.gfx.as_mut() {
                            if moved {
                                let win = gfx.surface.size();
                                let to_world = |p: (f32, f32)| {
                                    let w = gfx.camera.screen_to_world(p, win);
                                    [w[0] as f64, w[1] as f64]
                                };
                                match m.shape {
                                    MarqueeShape::Box => self.vec.pen.box_select_with(
                                        &gfx.vec_scene,
                                        to_world(start),
                                        to_world(cur),
                                        additive,
                                    ),
                                    // ⚠️ O polígono é convertido a MUNDO ponto a ponto, e é aqui
                                    // que o LAÇO tem de o ser: as âncoras que ele julga sobem
                                    // pelo afim de cada forma (ADR-0111), então a pergunta só faz
                                    // sentido no espaço que as duas partilham.
                                    MarqueeShape::Lasso => {
                                        let poly: Vec<[f64; 2]> =
                                            m.closed_path().into_iter().map(to_world).collect();
                                        self.vec.pen.lasso_select_with(
                                            &gfx.vec_scene,
                                            &poly,
                                            additive,
                                        );
                                    }
                                }
                            } else if !additive {
                                self.vec.pen.select(None);
                            }
                        }
                        return;
                    }
                    // **O WIDTH TOOL solta.** A alça é largada e o passo de undo fecha — o
                    // MESMO par begin/commit do lápis e das ferramentas de quina.
                    //
                    // ⚠️ **Arm próprio, e ANTES da cadeia de modo**, pela razão que o lápis
                    // pagou logo abaixo: `shape_kind_for_mode(..).is_none()` é verdadeiro no modo
                    // Width, então um ramo posto no `else` dele seria código morto no único modo
                    // capaz de o alcançar — e a alça ficaria agarrada ao dedo depois de solta.
                    if let Some(grab) = self.vec.width_grab.take() {
                        if let Some(gfx) = self.gfx.as_mut() {
                            // Um clique que não moveu nada não pediu nada: a parada que o press
                            // criou é desfeita, e o desenho fica como estava (ver `Grab::created`
                            // — os 13,1% da re-parametrização nunca chegam à tela).
                            crate::width_handles::discard_if_untouched(
                                &mut gfx.sim,
                                &self.vec.entities,
                                grab,
                            );
                        }
                        return;
                    }
                    // **O LÁPIS solta.** O traço vira documento (ou desaparece, se o gesto
                    // foi um clique perdido) e a forma nova fica SELECIONADA — o artista
                    // acabou de a desenhar, então é nela que ele vai mexer.
                    //
                    // ⚠️ **Arm PRÓPRIO, e antes da cadeia de modo.** Ele nasceu no `else` de
                    // `shape_kind_for_mode(..).is_none()`, que é **verdadeiro em modo Pencil** (o
                    // lápis não é um `ShapeKind`) ⇒ a primeira metade ganhava sempre e este
                    // ramo era **código morto no único modo capaz de o alcançar**. O preço eram
                    // dois defeitos que o Enio viu como um: o `active` nunca era limpo, então o
                    // lápis **continuava a desenhar com o botão em cima** (todo move seguinte
                    // entrava no traço) e o press seguinte **apagava o traço anterior**
                    // (`on_press` remove o path que encontra vivo).
                    //
                    // ⚠️ O guard é `is_active()`, não "o modo é Pencil": soltar sobre um botão
                    // do painel enquanto o lápis está armado mas ocioso TEM de cair no chrome,
                    // senão todo clique de painel morre em silêncio (a lição que o
                    // `shape_up_consumes` documenta ao lado).
                    if self.vec.pencil.is_active() {
                        let committed = if let Some(gfx) = self.gfx.as_mut() {
                            self.vec.pencil.on_release(&mut gfx.vec_scene)
                        } else {
                            false
                        };
                        if committed {
                            let sel = self.vec.pencil.selected();
                            self.vec.pen.select(sel);
                        }
                        return;
                    }
                    if shape_kind_for_mode(&self.vec.draw_config).is_none() {
                        // Pen: the release ends a handle drag / grab.
                        let consumed = self.vec.pen.on_release();
                        // DIAGNÓSTICO (`PH2D_CORNER_LOG=1`): os raios LOGO APÓS o gesto. Com o
                        // log do press, parte o report em dois — se aqui os raios anteriores já
                        // sumiram, foi o GESTO; se estão inteiros e somem até o press seguinte,
                        // foi um passe POR-FRAME entre os dois.
                        if std::env::var_os("PH2D_CORNER_LOG").is_some()
                            && self.vec.draw_config.mode.is_corner_tool()
                            && let Some(gfx) = self.gfx.as_ref()
                            && let Some(pid) = self.vec.pen.selected()
                        {
                            let shape = self.vec.entities.get(&pid).is_some_and(|&b| {
                                gfx.sim
                                    .world()
                                    .get::<ph2d_ecs::VecShape>(ph2d_ecs::Entity::from_bits(b))
                                    .is_some()
                            });
                            let radii: Vec<f64> = gfx
                                .vec_scene
                                .path(pid)
                                .map(|p| p.verts_all().map(|v| v.corner_radius).collect())
                                .unwrap_or_default();
                            eprintln!("[corner] RELEASE shape={shape} radii={radii:?}");
                        }
                        if consumed {
                            return;
                        }
                    } else if shape_up_consumes(
                        self.vec.draw_config.mode,
                        self.vec.shape.is_active(),
                    ) {
                        // A shape drag is in progress → finalize it. Commit if the
                        // drag spanned a real size, else discard the stray click
                        // (cancel the pending undo so it doesn't record a spurious
                        // `next_id`-only step). ONLY consume the Up when a shape is
                        // actually being drawn — otherwise (e.g. releasing over a
                        // panel button while in a shape mode) the Up MUST fall
                        // through to the chrome dispatch, else every panel click
                        // (mode switch, boolean, close) is silently swallowed.
                        let committed = if let Some(gfx) = self.gfx.as_mut() {
                            let c = self.vec.shape.on_release(&mut gfx.vec_scene);
                            if c {
                                // Solda os endpoints da forma recém-criada com nós
                                // vizinhos: basta ficarem próximos para se fundirem, e
                                // várias linhas/arcos fecham numa forma (Enio
                                // 2026-07-09). A forma nova ainda não tem entidade
                                // (o sync roda depois), então está na identidade; a
                                // geometria PRÉ-existente nunca se mexe (só a nova
                                // snapa nela). Ao fechar num laço, recebe o fill do
                                // estilo atual — como uma região desenhada pela pen.
                                if let Some(new_id) = self.vec.shape.selected() {
                                    let fill = self.vec.pen.style().fill;
                                    let fill_on_close =
                                        (fill.a != 0).then(|| ph2d_vec_scene::Paint::solid(fill));
                                    let xforms = ph2d_vec_entities::transform::build(
                                        &gfx.sim,
                                        &self.vec.entities,
                                    );
                                    let win = gfx.surface.size();
                                    let tol =
                                        crate::vec_gizmo_view::stroke_hit_r(&gfx.camera, win) * 1.5;
                                    gfx.vec_scene.weld_new_shape(
                                        new_id,
                                        &xforms,
                                        tol,
                                        fill_on_close,
                                    );
                                }
                            }
                            c
                        } else {
                            false
                        };
                        if committed {
                            // Seleciona a forma nova para edição imediata — a menos que
                            // o weld a tenha fundido noutro objeto (o id sumiu).
                            let sel = self.vec.shape.selected().filter(|id| {
                                self.gfx.as_ref().is_some_and(|g| {
                                    g.vec_scene.paths().iter().any(|p| p.id == *id)
                                })
                            });
                            self.vec.pen.select(sel);
                        }
                        return;
                    }
                    // Shape mode but no active drag → fall through to chrome so the
                    // panel buttons receive their Up.
                }
                (ph2d_host::PointerButton::Secondary, PointerKind::Down) if on_canvas => {
                    self.ramo_vetor_direito_premido();
                    return;
                }
                _ => {}
            }
        }

        // Painter layers drag-reparent (W3 T3.8): the dispatch emits a
        // PainterLayerReparent on Up of an active layer-row drag; route it to
        // the active PainterTool, which reverses NodeId→LayerId and applies
        // move_into_group / reorder. The concrete-tool downcast lives in the
        // allowlisted painter bridge so central dispatch stays downcast-free
        // (architecture_no_downcast_to_concrete_tool_in_shell gate).
        if let Some((dragged, drop)) = forward_to_hero(self.gfx.as_mut(), evt)
            && let Some(gfx) = self.gfx.as_mut()
        {
            ph2d_app_painter::painter_bridge_queries::apply_layer_reparent(
                &mut gfx.tools,
                dragged,
                drop,
            );
        }

        if self.ramo_reclamantes(
            mapped_button,
            kind,
            evt,
            menu_open_before,
            eyedropper_armed_before,
        ) {
            return;
        }

        // M14.7 C: gizmo drag begin/end. A Primary Down that lands on
        // a gizmo handle starts a drag (snapshot Transform + cursor
        // world pos); Up clears it. Move handling lives in CursorMoved
        // so every motion event gets the live cursor.
        if mapped_button == ph2d_host::PointerButton::Primary {
            match kind {
                PointerKind::Down => {
                    if self.ramo_gizmo_premido(evt, menu_open_before) {
                        return;
                    }
                }
                PointerKind::Up => {
                    self.ramo_gizmo_largar(evt);
                }
                _ => {}
            }
        }
        self.ramo_pan_e_barra_lateral(state, button);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        VecPathShapeOp, VecTransformField, apply_vec_path_shape, apply_vec_transform,
        shape_kind_for_mode, shape_up_consumes, vec_bool_op_for_id, vec_flip_for_id,
        vec_path_shape_for_id, vec_reorder_for_id, vec_rotate_for_id, vec_transform_field_for_id,
        vec_vertex_kind_for_id,
    };
    use ph2d_tool_vector::DrawMode;
    use ph2d_vec_scene::ShapeKind;
    use ph2d_vec_scene::{FlipAxis, Rotate90, VertexKind, ZOrder};

    #[test]
    fn vertex_button_ids_map_to_their_kinds() {
        assert_eq!(
            vec_vertex_kind_for_id(ph2d_tool_vector::ids::VECTOR_VERT_CORNER),
            Some(VertexKind::Corner)
        );
        assert_eq!(
            vec_vertex_kind_for_id(ph2d_tool_vector::ids::VECTOR_VERT_SMOOTH),
            Some(VertexKind::Smooth)
        );
        assert_eq!(
            vec_vertex_kind_for_id(ph2d_tool_vector::ids::VECTOR_VERT_SYMMETRIC),
            Some(VertexKind::Symmetric)
        );
        assert_eq!(
            vec_vertex_kind_for_id(ph2d_panel_vector::ids::VECTOR_BOOL_UNION),
            None
        );
    }

    #[test]
    fn arrange_button_ids_map_to_their_zorder() {
        assert_eq!(
            vec_reorder_for_id(ph2d_tool_vector::ids::VECTOR_ARRANGE_TO_BACK),
            Some(ZOrder::ToBack)
        );
        assert_eq!(
            vec_reorder_for_id(ph2d_tool_vector::ids::VECTOR_ARRANGE_BACKWARD),
            Some(ZOrder::Lower)
        );
        assert_eq!(
            vec_reorder_for_id(ph2d_tool_vector::ids::VECTOR_ARRANGE_FORWARD),
            Some(ZOrder::Raise)
        );
        assert_eq!(
            vec_reorder_for_id(ph2d_tool_vector::ids::VECTOR_ARRANGE_TO_FRONT),
            Some(ZOrder::ToFront)
        );
        // Duplicate is NOT a reorder (handled separately), nor any non-Arrange id.
        assert_eq!(
            vec_reorder_for_id(ph2d_panel_vector::ids::VECTOR_ARRANGE_DUPLICATE),
            None
        );
        assert_eq!(
            vec_reorder_for_id(ph2d_panel_vector::ids::VECTOR_BOOL_UNION),
            None
        );
    }

    #[test]
    fn flip_button_ids_map_to_their_axis() {
        assert_eq!(
            vec_flip_for_id(ph2d_tool_vector::ids::VECTOR_ARRANGE_FLIP_H),
            Some(FlipAxis::Horizontal)
        );
        assert_eq!(
            vec_flip_for_id(ph2d_tool_vector::ids::VECTOR_ARRANGE_FLIP_V),
            Some(FlipAxis::Vertical)
        );
        // Flip is NOT a reorder and vice-versa.
        assert_eq!(
            vec_flip_for_id(ph2d_tool_vector::ids::VECTOR_ARRANGE_TO_BACK),
            None
        );
        assert_eq!(
            vec_reorder_for_id(ph2d_tool_vector::ids::VECTOR_ARRANGE_FLIP_H),
            None
        );
    }

    #[test]
    fn rotate_button_ids_map_to_their_direction() {
        assert_eq!(
            vec_rotate_for_id(ph2d_tool_vector::ids::VECTOR_ARRANGE_ROTATE_CW),
            Some(Rotate90::Cw)
        );
        assert_eq!(
            vec_rotate_for_id(ph2d_tool_vector::ids::VECTOR_ARRANGE_ROTATE_CCW),
            Some(Rotate90::Ccw)
        );
        assert_eq!(
            vec_rotate_for_id(ph2d_tool_vector::ids::VECTOR_ARRANGE_FLIP_H),
            None
        );
        assert_eq!(
            vec_flip_for_id(ph2d_tool_vector::ids::VECTOR_ARRANGE_ROTATE_CW),
            None
        );
    }

    #[test]
    fn transform_fields_map_and_apply_translates_and_scales() {
        use ph2d_vec_scene::{VecScene, rectangle};
        assert_eq!(
            vec_transform_field_for_id(ph2d_tool_vector::ids::VECTOR_TRANSFORM_X),
            Some(VecTransformField::X)
        );
        assert_eq!(
            vec_transform_field_for_id(ph2d_tool_vector::ids::VECTOR_TRANSFORM_H),
            Some(VecTransformField::H)
        );
        assert_eq!(
            vec_transform_field_for_id(ph2d_tool_vector::ids::VECTOR_ARRANGE_FLIP_H),
            None
        );

        let mut scene = VecScene::new();
        let id = scene.push_path(rectangle([0.0, 0.0], [10.0, 4.0]));
        // Um path CRU: sem receita a manter em passo (o `keep_recipe_in_step` sai calado).
        let mut sim = ph2d_ecs::SimWorld::default();
        let map = ph2d_vec_entities::entities::VecEntityMap::new();
        let mut pen = ph2d_vec_edit::PenTool::new();
        pen.select(Some(id));

        // X → 5 moves the bbox min; W → 20 doubles the width.
        apply_vec_transform(
            &mut sim,
            &map,
            &mut scene,
            &pen,
            &ph2d_vec_scene::VecXforms::new(),
            VecTransformField::X,
            5.0,
        );
        assert!((scene.path_bbox(id).unwrap().0[0] - 5.0).abs() < 1e-9);
        apply_vec_transform(
            &mut sim,
            &map,
            &mut scene,
            &pen,
            &ph2d_vec_scene::VecXforms::new(),
            VecTransformField::W,
            20.0,
        );
        let (lo, hi) = scene.path_bbox(id).unwrap();
        assert!((hi[0] - lo[0] - 20.0).abs() < 1e-9, "W set to 20");
        assert!((lo[0] - 5.0).abs() < 1e-9, "min x pinned during scale");
    }

    #[test]
    fn path_shape_ids_map_and_apply_smooths_then_sharpens() {
        use ph2d_vec_scene::{VertexKind, regular_polygon};
        assert_eq!(
            vec_path_shape_for_id(ph2d_tool_vector::ids::VECTOR_PATH_SMOOTH),
            Some(VecPathShapeOp::Smooth)
        );
        assert_eq!(
            vec_path_shape_for_id(ph2d_tool_vector::ids::VECTOR_PATH_SHARPEN),
            Some(VecPathShapeOp::Sharpen)
        );
        assert_eq!(
            vec_path_shape_for_id(ph2d_tool_vector::ids::VECTOR_PATH_SIMPLIFY),
            Some(VecPathShapeOp::Simplify)
        );
        assert_eq!(
            vec_path_shape_for_id(ph2d_tool_vector::ids::VECTOR_PATH_SUBDIVIDE),
            Some(VecPathShapeOp::Subdivide)
        );
        assert_eq!(
            vec_path_shape_for_id(ph2d_tool_vector::ids::VECTOR_ARRANGE_FLIP_H),
            None
        );

        let mut scene = ph2d_vec_scene::VecScene::new();
        let id = scene.push_path(regular_polygon([0.0, 0.0], 5.0, 5.0, 5));
        let mut pen = ph2d_vec_edit::PenTool::new();
        pen.select(Some(id));

        apply_vec_path_shape(&mut scene, &pen, VecPathShapeOp::Smooth);
        assert!(
            scene.paths()[0]
                .verts
                .iter()
                .all(|v| v.kind == VertexKind::Smooth),
            "smooth button curves every vertex"
        );
        apply_vec_path_shape(&mut scene, &pen, VecPathShapeOp::Sharpen);
        assert!(
            scene.paths()[0]
                .verts
                .iter()
                .all(|v| v.kind == VertexKind::Corner && v.in_handle == v.anchor),
            "sharpen button flattens every vertex"
        );

        // Simplify: a closed square with a redundant midpoint on one edge drops it.
        let sq = scene.push_path(ph2d_vec_scene::VecPath {
            verts: vec![
                ph2d_vec_scene::VecVertex::corner([0.0, 0.0]),
                ph2d_vec_scene::VecVertex::corner([5.0, 0.0]), // redundant midpoint
                ph2d_vec_scene::VecVertex::corner([10.0, 0.0]),
                ph2d_vec_scene::VecVertex::corner([10.0, 10.0]),
                ph2d_vec_scene::VecVertex::corner([0.0, 10.0]),
            ],
            closed: true,
            ..ph2d_vec_scene::VecPath::default()
        });
        pen.select(Some(sq));
        let before = scene
            .paths()
            .iter()
            .find(|p| p.id == sq)
            .unwrap()
            .verts
            .len();
        apply_vec_path_shape(&mut scene, &pen, VecPathShapeOp::Simplify);
        let after = scene
            .paths()
            .iter()
            .find(|p| p.id == sq)
            .unwrap()
            .verts
            .len();
        assert_eq!(after, before - 1, "simplify drops the one redundant point");

        // Subdivide: one midpoint per segment (closed ⇒ doubles the vertex count).
        let n = scene
            .paths()
            .iter()
            .find(|p| p.id == sq)
            .unwrap()
            .verts
            .len();
        apply_vec_path_shape(&mut scene, &pen, VecPathShapeOp::Subdivide);
        let n2 = scene
            .paths()
            .iter()
            .find(|p| p.id == sq)
            .unwrap()
            .verts
            .len();
        assert_eq!(n2, n * 2, "subdivide doubles a closed path's vertices");

        // Close/Open toggle flips the selected path's `closed` flag each click.
        let was = scene.paths().iter().find(|p| p.id == sq).unwrap().closed;
        super::apply_vec_toggle_closed(&mut scene, &mut pen);
        assert_eq!(
            scene.paths().iter().find(|p| p.id == sq).unwrap().closed,
            !was,
            "toggle flips closed"
        );
        super::apply_vec_toggle_closed(&mut scene, &mut pen);
        assert_eq!(
            scene.paths().iter().find(|p| p.id == sq).unwrap().closed,
            was,
            "toggle flips back"
        );
        // Closing a never-filled path seeds a fill so it paints immediately.
        assert!(
            scene
                .paths()
                .iter()
                .find(|p| p.id == sq)
                .unwrap()
                .fill
                .is_some(),
            "closing seeds the Style fill (immediate paint)"
        );
    }

    #[test]
    fn shape_up_only_consumed_while_a_drag_is_live() {
        // Pen mode never consumes via the shape path (the pen path handles it).
        assert!(!shape_up_consumes(DrawMode::Pen, false));
        assert!(!shape_up_consumes(DrawMode::Pen, true));
        // In a shape mode, a live drag consumes the Up (finalize the shape)...
        assert!(shape_up_consumes(DrawMode::Shape, true));
        // ...but with NO active drag the Up must fall through so a panel-button
        // click (mode switch / boolean / close) is not swallowed. This is the
        // exact regression that made every button dead after entering Rect mode.
        assert!(!shape_up_consumes(DrawMode::Shape, false));
        assert!(
            !shape_up_consumes(DrawMode::Pen, true),
            "a caneta nao e forma"
        );
    }

    /// **Os OITO ids do Pathfinder mapeiam para as suas ops** (plano 25 §8, W5).
    ///
    /// ⚠️ A lista é enumerada aqui à mão de propósito: derivá-la da tabela do produto tornaria o
    /// gate um espelho (encolher a tabela encolheria a lista percorrida, e ele seguiria verde) —
    /// o oráculo auto-referente que esta linha já pagou na varredura dos pills.
    #[test]
    fn pathfinder_button_ids_map_to_their_ops() {
        use ph2d_vec_boolean::PathfinderOp as P;
        for (id, want) in [
            (ph2d_panel_vector::ids::VECTOR_BOOL_UNION, P::Union),
            (ph2d_panel_vector::ids::VECTOR_BOOL_SUBTRACT, P::Subtract),
            (ph2d_panel_vector::ids::VECTOR_BOOL_INTERSECT, P::Intersect),
            (ph2d_panel_vector::ids::VECTOR_BOOL_EXCLUDE, P::Exclude),
            (ph2d_tool_vector::ids::VECTOR_BOOL_MINUS_BACK, P::MinusBack),
            (ph2d_tool_vector::ids::VECTOR_BOOL_TRIM, P::Trim),
            (ph2d_tool_vector::ids::VECTOR_BOOL_CROP, P::Crop),
            (ph2d_tool_vector::ids::VECTOR_BOOL_MERGE, P::Merge),
        ] {
            assert_eq!(vec_bool_op_for_id(id), Some(want), "{want:?}");
        }
        // A non-boolean id (a mode button) is not a boolean op.
        assert_eq!(
            vec_bool_op_for_id(ph2d_tool_vector::ids::VECTOR_MODE_PEN),
            None
        );
    }

    /// O gesto de canvas só desenha no modo **Shape**, e o que ele desenha é a forma
    /// ATIVA do catálogo — não há mais um modo por forma. Com vinte e cinco formas, o
    /// `match` antigo (um braço por forma) seria o pior lugar para esquecer uma.
    #[test]
    fn only_shape_mode_draws_and_it_draws_the_active_shape() {
        use ph2d_tool_vector::VectorDrawConfig;
        let mut cfg = VectorDrawConfig::default();
        for m in [
            DrawMode::Select,
            DrawMode::Node,
            DrawMode::Pen,
            DrawMode::Text,
        ] {
            cfg.mode = m;
            assert_eq!(shape_kind_for_mode(&cfg), None, "{m:?} nao desenha forma");
        }
        cfg.mode = DrawMode::Shape;
        for k in [ShapeKind::Rectangle, ShapeKind::Star, ShapeKind::Arc] {
            cfg.shape = k;
            assert_eq!(shape_kind_for_mode(&cfg), Some(k));
        }
    }

    /// **A MOLDURA desenha um retângulo ARREDONDÁVEL** (Enio, 2026-08-21: *"o Frame é criado como
    /// retângulo de quinas sem a possibilidade de arredondamento"*).
    ///
    /// ⚠️ **E o kind dela NÃO segue o catálogo** — é o par de asserções que importa. A moldura
    /// tem de dar `RoundRect` mesmo com a estrela ativa, senão o gesto herdaria a forma do botão
    /// aceso e a ferramenta Moldura deixaria de desenhar molduras.
    #[test]
    fn the_frame_draws_a_roundable_rectangle_whatever_the_catalogue_says() {
        use ph2d_tool_vector::VectorDrawConfig;
        let mut cfg = VectorDrawConfig {
            mode: DrawMode::Frame,
            ..Default::default()
        };
        for catalogue in [ShapeKind::Rectangle, ShapeKind::Star, ShapeKind::Heart] {
            cfg.shape = catalogue;
            assert_eq!(
                shape_kind_for_mode(&cfg),
                Some(ShapeKind::RoundRect),
                "a moldura tem de ser arredondavel, e o catalogo ({catalogue:?}) nao manda nela"
            );
        }
    }

    // ─── boolean/compound: a costura shell ↔ documento ────────────────────────

    /// Cena com um quadrado externo e outro DENTRO dele, ambos selecionados
    /// (z: externo atrás, interno na frente). Devolve `(scene, pen, ids)`.
    fn nested_selection() -> (ph2d_vec_scene::VecScene, ph2d_vec_edit::PenTool, [u64; 2]) {
        let mut scene = ph2d_vec_scene::VecScene::new();
        let outer = scene.push_path(ph2d_vec_scene::rectangle([0.0, 0.0], [10.0, 10.0]));
        let inner = scene.push_path(ph2d_vec_scene::rectangle([3.0, 3.0], [7.0, 7.0]));
        let mut pen = ph2d_vec_edit::PenTool::default();
        pen.select_many(&[outer, inner]);
        (scene, pen, [outer, inner])
    }

    /// A regressão que motivou o bloco: Subtract agia nas duas últimas regiões
    /// fechadas do DOCUMENTO, ignorando a seleção — e devolvia dois discos
    /// sólidos em vez de uma rosquinha.
    #[test]
    fn boolean_subtract_uses_the_selection_and_makes_a_real_hole() {
        let (mut scene, mut pen, _) = nested_selection();
        // Um terceiro path, NÃO selecionado, bem longe: a booleana antiga o teria
        // agarrado (é uma das duas últimas fechadas); a nova tem de ignorá-lo.
        let bystander = scene.push_path(ph2d_vec_scene::rectangle([90.0, 90.0], [95.0, 95.0]));

        super::apply_vec_boolean(
            &mut scene,
            &mut pen,
            &ph2d_vec_scene::VecXforms::new(),
            ph2d_vec_boolean::PathfinderOp::Subtract,
        );

        assert_eq!(scene.paths().len(), 2, "resultado + o bystander intacto");
        assert!(scene.paths().iter().any(|p| p.id == bystander));
        let donut = scene.paths().iter().find(|p| p.id != bystander).unwrap();
        assert!(donut.is_compound(), "o furo vive num subpath");
        let id = donut.id;
        assert!(scene.path_contains_point(id, [1.0, 5.0]), "o anel é sólido");
        assert!(
            !scene.path_contains_point(id, [5.0, 5.0]),
            "o centro é vazado"
        );
        // O resultado entra na fatia de z da BASE (não salta pro topo).
        assert_eq!(scene.paths()[0].id, id);
        assert_eq!(pen.selected(), Some(id), "a booleana seleciona o resultado");
    }

    #[test]
    fn boolean_needs_two_selected_closed_regions() {
        let (mut scene, mut pen, ids) = nested_selection();
        pen.select(Some(ids[0])); // só um selecionado
        super::apply_vec_boolean(
            &mut scene,
            &mut pen,
            &ph2d_vec_scene::VecXforms::new(),
            ph2d_vec_boolean::PathfinderOp::Union,
        );
        assert_eq!(scene.paths().len(), 2, "no-op");
    }

    /// Make Compound é como o usuário desenha um buraco à mão; Release desfaz.
    #[test]
    fn make_and_release_compound_from_the_selection() {
        let (mut scene, mut pen, ids) = nested_selection();

        super::apply_vec_compound(&mut scene, &mut pen, true);
        assert_eq!(scene.paths().len(), 1);
        assert!(
            !scene.path_contains_point(ids[0], [5.0, 5.0]),
            "virou buraco"
        );
        assert_eq!(pen.selected(), Some(ids[0]));

        super::apply_vec_compound(&mut scene, &mut pen, false);
        assert_eq!(scene.paths().len(), 2);
        assert!(
            scene.path_contains_point(ids[0], [5.0, 5.0]),
            "sólido de novo"
        );
        assert_eq!(pen.selected_paths().len(), 2, "base + liberado");
    }

    /// A regra de preenchimento troca o buraco por região sólida, sem tocar a geometria.
    #[test]
    fn fill_rule_toggle_vacates_or_fills_the_hole() {
        let (mut scene, mut pen, ids) = nested_selection();
        super::apply_vec_compound(&mut scene, &mut pen, true);
        assert!(!scene.path_contains_point(ids[0], [5.0, 5.0]));

        super::apply_vec_fill_rule(&mut scene, &pen, false); // Non-Zero
        assert!(
            scene.path_contains_point(ids[0], [5.0, 5.0]),
            "NonZero preenche"
        );
        super::apply_vec_fill_rule(&mut scene, &pen, true); // Even-Odd
        assert!(
            !scene.path_contains_point(ids[0], [5.0, 5.0]),
            "EvenOdd vaza"
        );
    }
}

/// The double-arrow cursor for a panel-border grip, given its edge bitmask
/// (`TIMELINE_EDGE_*`; a corner sets two bits). Corners point along their own
/// diagonal: the top-left / bottom-right pair is `Nwse` (↖↘), the other `Nesw`.
fn resize_cursor_for_edges(edges: u8) -> winit::window::CursorIcon {
    use ph2d_editor_core::interaction::{
        TIMELINE_EDGE_B, TIMELINE_EDGE_L, TIMELINE_EDGE_R, TIMELINE_EDGE_T,
    };
    use winit::window::CursorIcon;
    let (l, r) = (edges & TIMELINE_EDGE_L != 0, edges & TIMELINE_EDGE_R != 0);
    let (t, b) = (edges & TIMELINE_EDGE_T != 0, edges & TIMELINE_EDGE_B != 0);
    match (l, r, t, b) {
        (true, _, true, _) | (_, true, _, true) => CursorIcon::NwseResize,
        (_, true, true, _) | (true, _, _, true) => CursorIcon::NeswResize,
        (_, _, true, _) | (_, _, _, true) => CursorIcon::NsResize,
        _ => CursorIcon::EwResize,
    }
}

#[cfg(test)]
mod cursor_tests {
    use super::resize_cursor_for_edges;
    use ph2d_editor_core::interaction::{
        TIMELINE_EDGE_B, TIMELINE_EDGE_L, TIMELINE_EDGE_R, TIMELINE_EDGE_T,
    };
    use winit::window::CursorIcon;

    #[test]
    fn each_edge_points_across_the_side_it_moves() {
        assert_eq!(
            resize_cursor_for_edges(TIMELINE_EDGE_L),
            CursorIcon::EwResize
        );
        assert_eq!(
            resize_cursor_for_edges(TIMELINE_EDGE_R),
            CursorIcon::EwResize
        );
        assert_eq!(
            resize_cursor_for_edges(TIMELINE_EDGE_T),
            CursorIcon::NsResize
        );
        assert_eq!(
            resize_cursor_for_edges(TIMELINE_EDGE_B),
            CursorIcon::NsResize
        );
    }

    #[test]
    fn each_corner_points_along_its_own_diagonal() {
        let tl = TIMELINE_EDGE_T | TIMELINE_EDGE_L;
        let br = TIMELINE_EDGE_B | TIMELINE_EDGE_R;
        let tr = TIMELINE_EDGE_T | TIMELINE_EDGE_R;
        let bl = TIMELINE_EDGE_B | TIMELINE_EDGE_L;
        assert_eq!(resize_cursor_for_edges(tl), CursorIcon::NwseResize);
        assert_eq!(resize_cursor_for_edges(br), CursorIcon::NwseResize);
        assert_eq!(resize_cursor_for_edges(tr), CursorIcon::NeswResize);
        assert_eq!(resize_cursor_for_edges(bl), CursorIcon::NeswResize);
    }

    #[test]
    fn an_empty_mask_never_shows_a_vertical_arrow() {
        // Defensive: a mask with no bits is a horizontal edge by fallback, not a
        // panic and not a misleading up-down arrow on a left/right grip.
        assert_eq!(resize_cursor_for_edges(0), CursorIcon::EwResize);
    }
}

/// Map a screen `y` inside the overlay to a **frequency**, as a fraction of Nyquist.
///
/// Frequency runs UP the spectrogram (DC at the bottom, Nyquist at the top) — the DAW
/// convention, and the one the picture is drawn in. Screen y runs DOWN. Getting this
/// inversion wrong would select a band and repair its mirror image: a fix that removes the
/// wrong sound and swears it did what you asked.
#[cfg(feature = "panel-audio-editor")]
fn freq_at_y(view: &ph2d_app_audio::WaveView, y: f32) -> f32 {
    let r = view.rect;
    (1.0 - (y - r.y) / r.h.max(1.0)).clamp(0.0, 1.0)
}

#[cfg(all(test, feature = "panel-audio-editor"))]
mod spectral_axis_tests {
    use super::freq_at_y;
    use ph2d_app_audio::WaveView;
    use ph2d_editor_core::zones::Rect;

    fn view() -> WaveView {
        WaveView {
            rect: Rect::new(100.0, 200.0, 400.0, 300.0),
            ruler: Rect::new(100.0, 180.0, 400.0, 20.0),
            frames: 48_000,
        }
    }

    /// **Frequency runs UP the spectrogram; screen y runs DOWN.**
    ///
    /// This inversion is the whole of the mapping, and getting it backwards is the worst
    /// kind of bug this feature can have: the user drags a box around a 5 kHz beep, and the
    /// repair confidently rebuilds the *mirror* band instead — removing a sound they wanted
    /// and leaving the one they pointed at. Nothing crashes; nothing looks wrong; the beep
    /// is still there and something else is gone.
    ///
    /// Red if the `1.0 -` is dropped: the top of the view would report DC.
    #[test]
    fn the_top_of_the_view_is_the_highest_frequency() {
        let v = view();
        let top = freq_at_y(&v, v.rect.y);
        let bottom = freq_at_y(&v, v.rect.y + v.rect.h);
        assert!(
            top > 0.99,
            "the TOP of the spectrogram should be Nyquist, got {top}"
        );
        assert!(
            bottom < 0.01,
            "the BOTTOM of the spectrogram should be DC, got {bottom}"
        );
        // …and the middle is the middle: a linear axis, which is what the picture draws.
        let mid = freq_at_y(&v, v.rect.y + v.rect.h * 0.5);
        assert!((mid - 0.5).abs() < 0.01, "the axis is not linear: {mid}");
    }

    /// A drag that leaves the view still names a frequency inside it — the band is clamped,
    /// not wrapped. (A wrap would jump the selection to the other end of the spectrum.)
    #[test]
    fn dragging_out_of_the_view_clamps() {
        let v = view();
        assert_eq!(freq_at_y(&v, v.rect.y - 500.0), 1.0);
        assert_eq!(freq_at_y(&v, v.rect.y + v.rect.h + 500.0), 0.0);
    }
}

/// **A roldana sob o cursor vira a SELEÇÃO** (W-RopeStop) — o pedido do Enio
/// *"permita selecionar as polias com mouse no canvas"*.
///
/// Até aqui uma roldana só era alcançável pela Hierarquia: ela não tem sprite,
/// então o `pick_sprites_at_world` não a vê, e as alças dela (centro/aro) só
/// nascem DEPOIS de ela estar selecionada — o laço em que a única porta de
/// entrada era uma lista de nomes.
///
/// ⚠️ **A tolerância é a MESMA `SNAP_PX` do ímã de âncora e do conta-gotas de
/// corda**, convertida em mundo pelo zoom: um app onde dois alvos de canvas
/// respondem a distâncias diferentes é um app que se aprende duas vezes.
///
/// Devolve se alguma roldana foi de fato selecionada — o chamador usa isso para
/// consumir o Down, como faz com o pivô e com a âncora.
/// ⚠️ Toma os TRÊS pedaços do `AppGfx` de que precisa, e não `&AppGfx`: quem
/// chama já segura um `&mut` em `gfx.hero_screen` — empréstimos por CAMPO são
/// disjuntos, um reborrow da struct inteira não é. A mesma assinatura, pelo mesmo
/// motivo, que o `joint_anchor_drag::open_drag`.
fn select_wheel_at(
    physics: &ph2d_physics_ecs::PhysicsBridge,
    camera: &ph2d_render::Camera2d,
    win: ph2d_host::WindowSize,
    hero: &mut ph2d_editor_core::HeroScreen,
    at: (f32, f32),
) -> bool {
    let w = camera.screen_to_world(at, win);
    let tol =
        ph2d_app_physics::joint_anchor_drag::SNAP_PX * camera.height_world / win.height as f32;
    let Some(wheel) = physics.wheel_at_world(w, tol) else {
        return false;
    };
    hero.gizmo.selection = Some(wheel.to_bits());
    hero.gizmo.extra_selection.clear();
    true
}

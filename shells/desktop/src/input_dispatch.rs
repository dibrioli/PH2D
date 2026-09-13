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

/// A preview, os fechos do Up e os press do Flip e dos gizmos de nó — ramos do `on_mouse_input`.
mod despacho_clique_flip;
/// O gizmo do botão primário (modificador, alvo, pivô, âncoras, alça) — ramos do `on_mouse_input`.
mod despacho_clique_gizmo;
/// O largar do botão primário (borracha, colapso da multi-seleção, fim do arrasto) — ramos do `on_mouse_input`.
mod despacho_clique_largar;
/// O pick de canvas do botão primário (hits, ciclo, seleção, arrasto) — ramos do `on_mouse_input`.
mod despacho_clique_pick;
/// O prólogo do clique (biblioteca, teclado do painel, 3D, âncora, áudio) — ramos do `on_mouse_input`.
mod despacho_clique_prologo;
/// Os reclamantes do fim do clique (painter, modais, pan, barra lateral) — ramos do `on_mouse_input`.
mod despacho_clique_reclamantes;
/// A roldana sob o cursor vira a seleção (a porta do ramo do pivô e da âncora).
mod despacho_clique_roldana;
/// Os picks modais e as alças do Select, antes da ferramenta vetorial — ramos do `on_mouse_input`.
mod despacho_clique_select;
/// A ferramenta vetorial no clique (o guarda do ADR-0112, o Shift, o direito) — ramos do `on_mouse_input`.
mod despacho_clique_vetor;
/// O premir da ferramenta vetorial (modos, corte/balde/osso, quinas, caneta/forma) — ramos do `on_mouse_input`.
mod despacho_clique_vetor_premido;
/// O soltar da ferramenta vetorial (osso, gestos, caneta/forma) — ramos do `on_mouse_input`.
mod despacho_clique_vetor_solto;
/// Os métodos auxiliares do despacho: janela e vetor (movidos do índice).
mod despacho_metodos_janela_e_vetor;
/// Os métodos auxiliares do despacho: modos e alças (movidos do índice).
mod despacho_metodos_modos_e_alcas;
/// Os métodos auxiliares do despacho: picks e arrastos (movidos do índice).
mod despacho_metodos_picks_e_arrastos;
/// Os arrastos em curso do movimento do cursor — ramos do `on_cursor_moved`.
mod despacho_mover;
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
use despacho_clique_roldana::select_wheel_at;

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
        if self.ramo_mover_arrastos_de_topo() {
            return;
        }
        if self.ramo_mover_pintura_flip_e_modais() {
            return;
        }
        if self.ramo_mover_vetor() {
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
        self.ramo_arrasto_biblioteca(state, button);
        self.ramo_aperto_solta_teclado(state);
        if self.ramo_navegacao_3d_e_ancora(state, button) {
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
        #[cfg(feature = "panel-audio-editor")]
        if self.ramo_editor_audio(kind) {
            return;
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
        if self.ramo_preview_e_fechos(kind, mapped_button, evt, menu_open_before) {
            return;
        }
        if self.ramo_flip_premidos(kind, mapped_button, menu_open_before, on_canvas) {
            return;
        }
        if self.ramo_select_modais(kind, mapped_button, evt, menu_open_before, on_canvas) {
            return;
        }
        if self.ramo_picks_de_fisica_e_alcas(kind, mapped_button, evt, menu_open_before) {
            return;
        }
        if self.ramo_alcas_soltas(kind, mapped_button, evt) {
            return;
        }
        if self.ramo_ferramenta_vetorial(mapped_button, kind, on_canvas, evt, menu_open_before) {
            return;
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
#[path = "input_dispatch/despacho_testes.rs"]
mod tests;

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
#[path = "input_dispatch/despacho_testes_cursor.rs"]
mod cursor_tests;

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
#[path = "input_dispatch/despacho_testes_espectro.rs"]
mod spectral_axis_tests;

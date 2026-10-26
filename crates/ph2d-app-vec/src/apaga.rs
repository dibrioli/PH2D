//! **O Delete do vetor** (ADR-0108 Fase 1; mudou-se da shell na W4.6): o mais ESPECÍFICO vence — a regra escolhida
//! com a tool Regras na mão, depois o vértice seleccionado, depois os paths seleccionados. Undo: o passo global, por
//! diff, existe sse algo saiu.

use ph2d_vec_scene::VecScene;

use crate::state::VecState;

/// Delete/Backspace sobre o canvas. `true` = algo saiu (consumido).
pub fn delete(vec: &mut VecState, scene: &mut VecScene) -> bool {
    if crate::cad_tool::delete(vec, scene) {
        return true;
    }
    if vec.pen.selected_vert().is_some() && vec.pen.delete_selected_vertex(scene) {
        eprintln!("[ph2d-vec] vértice apagado");
        return true;
    }
    paths(vec, scene)
}

/// Apaga os paths seleccionados (o Delete sem vértice; o «Delete» do menu).
pub fn paths(vec: &mut VecState, scene: &mut VecScene) -> bool {
    let sel = vec.pen.selected_paths().to_vec();
    let mut any = false;
    for id in &sel {
        any |= scene.remove_path(*id);
    }
    if any {
        vec.pen.clear();
        eprintln!("[ph2d-vec] {} path(s) apagado(s)", sel.len());
    }
    any
}

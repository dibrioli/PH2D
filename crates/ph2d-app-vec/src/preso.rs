//! ⭐ **C2 — Fill e Stroke de um cartão PRESO** (`docs/CartoesUI/09_alfinete_nos_cartoes_PLANO.md`): o que
//! o objecto preso mostra e por onde a escrita do cartão chega a ele. A tool é da seleção: o gesto do
//! cartão preso passa por ela para traduzir o id em campos, e ela volta ao molde no mesmo quadro.

use ph2d_ecs::SimWorld;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_panel_vector::state as painel;
use ph2d_tool_vector::VectorStyleSnapshot;
use ph2d_tool_vector::appearance;
use ph2d_vec_edit::PenTool;
use ph2d_vec_scene::{Rgba8, VecPathId, VecScene};
use std::cell::Cell;

thread_local! {
    /// O estilo da seleção no fim do último quadro: o que a tool devolve depois de um gesto no preso,
    /// e os campos de sessão (modo, forma) do retrato de cada preso.
    static MOLDE: Cell<Option<VectorStyleSnapshot>> = const { Cell::new(None) };
}

pub fn guarda_molde(s: VectorStyleSnapshot) {
    MOLDE.with(|m| m.set(Some(s)));
}

#[must_use]
pub fn molde() -> Option<VectorStyleSnapshot> {
    MOLDE.with(Cell::get)
}

/// A forma de um objecto (`None` = não é uma forma, ou já saiu).
#[must_use]
pub fn forma(sim: &SimWorld, bits: u64) -> Option<VecPathId> {
    let e = ph2d_ecs::Entity::try_from_bits(bits)?;
    Some(sim.world().get::<ph2d_ecs::VecPathRef>(e)?.0)
}

/// A forma do cartão preso sob o gesto deste quadro (`None` = o gesto é da seleção).
#[must_use]
pub fn alvo_do_gesto(store: &WidgetStore, sim: &SimWorld) -> Option<VecPathId> {
    forma(sim, store.card_edit_subject()?)
}

/// O retrato de UM objecto preso: tudo o que os cartões Fill e Stroke leem, pelas portas da seleção.
pub fn publica(
    bits: u64,
    sim: &SimWorld,
    scene: &VecScene,
    map: &ph2d_vec_entities::entities::VecEntityMap,
    store: &WidgetStore,
    doc: ph2d_editor_core::DocPx,
) {
    let id = forma(sim, bits);
    let p = id.and_then(|i| scene.path(i));
    painel::set_current_appearance(crate::paint_stack::aparencia_do_preso(
        bits, sim, scene, map, store, doc,
    ));
    crate::stroke_present::publica_de(p);
    appearance::set_current_vector_style(p.map(|p| {
        let fill = crate::vector_bridge::seed_fill_from_paint(p.base_fill()).or_else(|| {
            p.base_fill()
                .and_then(crate::stroke_present::ramp)
                .map(|r| r[0])
        });
        let traco = p.base_stroke();
        molde().unwrap_or_default().da_forma(
            fill,
            traco,
            traco.map_or(0.0, |s| doc.of_world(s.width)),
        )
    }));
    ph2d_editor_core::panel::cards::estilos::set_ligados(id.map_or_else(Vec::new, |i| {
        crate::paint_stack::estilos::ligados_de(scene, i)
    }));
}

/// **O seletor de cor aberto por um cartão preso**: a cor vai à tinta de base do objecto dele, sem
/// passar pela tool. Devolve se o seletor é de um preso (a ponte não o lê como da seleção). Com o
/// objecto apagado, fecha.
pub fn seletor_na_base(
    store: &mut WidgetStore,
    scene: &mut VecScene,
    sim: &SimWorld,
    tocou: bool,
) -> bool {
    let Some(bits) = store.picker_subject() else {
        return false;
    };
    let Some(id) = forma(sim, bits) else {
        store.set_picker_target(None);
        return true;
    };
    let alvo = store.picker_target();
    let fill = alvo == Some(ph2d_tool_vector::ids::VECTOR_FILL_SWATCH);
    if !fill && alvo != Some(ph2d_tool_vector::ids::VECTOR_STROKE_SWATCH) {
        return true; // a amostra de uma camada: é do `apply_picker`
    }
    let picker = ph2d_editor_core::ids::INSP_BLENDER_PICKER;
    let Some((value, ..)) = store.blender_picker(picker) else {
        return true;
    };
    let mut c = value.rgba;
    if tocou && c[3] == 0 {
        c[3] = u8::MAX;
        store.set_blender_value(
            picker,
            ph2d_tokens::ColorValue::from_rgba8(c[0], c[1], c[2], c[3]),
        );
    }
    let nova = Rgba8::new(c[0], c[1], c[2], c[3]);
    if let Some(path) = scene.path_mut(id)
        && crate::vector_bridge::pick_base(path, fill, nova, tocou)
    {
        crate::bindings::note_authored(if fill {
            ph2d_ecs::BoundProp::Fill
        } else {
            ph2d_ecs::BoundProp::StrokeColor
        });
        crate::bindings::note_alvo(id);
    }
    true
}

/// Os pedidos da pilha de aparência que o dreno recolheu neste quadro.
#[derive(Clone, Copy, Debug, Default)]
pub struct PedidosDaPilha {
    pub vec_opacity: Option<f64>,
    pub vec_blend: Option<u8>,
    pub verb: Option<crate::paint_stack::StackVerb>,
    pub width: Option<f64>,
    pub dx: Option<f64>,
    pub dy: Option<f64>,
    pub dilate: Option<f64>,
    pub join: Option<u8>,
    pub opacity: Option<f64>,
    pub blend: Option<u8>,
}

/// ⭐ **A APARÊNCIA do objecto entra no DOCUMENTO** (estudo 42 item 2 e 4). O verbo primeiro, as
/// propriedades depois: escrever numa camada que o verbo acabou de remover seria um `None` silencioso.
/// O passo de undo sai de graça: o registo é por DIFF e um arrasto suprime a captura.
pub fn aplica_pedidos(
    store: &mut WidgetStore,
    scene: &mut VecScene,
    pen: &PenTool,
    sim: &mut SimWorld,
    map: &ph2d_vec_entities::entities::VecEntityMap,
    doc: ph2d_editor_core::DocPx,
    p: PedidosDaPilha,
) {
    use crate::paint_stack as pilha;
    let sel = pilha::alvos(store, pen, sim);
    if let Some(track) = p.vec_opacity {
        #[allow(clippy::cast_possible_truncation)]
        crate::appearance::set_opacity(scene, &sel, track as f32);
        // Numa forma que a timeline conduz, o valor que a TELA mostra (plano 34 D3): o auto-key vê a
        // mão: armado grava, desarmado segura até o tempo mudar.
        #[allow(clippy::cast_possible_truncation)]
        crate::driven_style::follow_the_hand(sim, map, &sel, track as f32);
    }
    if let Some(code) = p.vec_blend {
        crate::appearance::set_blend(scene, &sel, code);
    }
    if let Some(v) = p.verb {
        pilha::apply(scene, &sel, v);
    }
    pilha::apply_no_quadro(store, scene, pen, sim);
    if let Some(i) = camada_do_gesto(store) {
        if let Some(w) = p.width {
            pilha::set_width(scene, &sel, i, w, doc);
        }
        // ONDE ela desenha (v21): só o eixo comitado, o outro fica o de cada forma.
        pilha::set_offset_axes(scene, &sel, i, p.dx, p.dy);
        if let Some(t) = p.opacity {
            #[allow(clippy::cast_possible_truncation)]
            pilha::set_opacity(scene, &sel, i, t as f32);
        }
        if let Some(code) = p.blend {
            pilha::set_blend(scene, &sel, i, code);
        }
        if let Some(d) = p.dilate {
            pilha::set_dilate(scene, &sel, i, d);
        }
    }
    if let Some(j) = p.join {
        pilha::set_chip(scene, &sel, camada_do_gesto(store), j);
    }
}

/// A camada que as propriedades deste quadro endereçam: a aberta no painel, da seleção; num cartão
/// preso, a linha que o gesto apontou, porque a aberta é de outro objecto.
#[must_use]
pub fn camada_do_gesto(store: &WidgetStore) -> Option<usize> {
    if store.card_edit_subject().is_some() {
        appearance::pinned_row()
    } else {
        painel::open_layer_index()
    }
}

#[cfg(test)]
#[path = "preso_tests.rs"]
mod tests;

//! ⭐ **C2 — Fill e Stroke de um cartão PRESO** (`docs/CartoesUI/09_alfinete_nos_cartoes_PLANO.md`): o que
//! o objecto preso mostra e por onde a escrita do cartão chega a ele. A tool é da seleção: o gesto do
//! cartão preso passa por ela para traduzir o id em campos, e ela volta ao molde no mesmo quadro
//! (`ph2d_app_vec::vector_bridge::dispatch`).

pub mod losango;

use ph2d_app_vec::paint_stack::{self as pilha, alvo_do_gesto, forma};
use ph2d_app_vec::stroke_present::{self, BaseVerb};
use ph2d_app_vec::vector_bridge as ponte;
use ph2d_ecs::SimWorld;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_panel_vector::state as painel;
use ph2d_tool_vector::StyleFields as F;
use ph2d_tool_vector::appearance;
use ph2d_vec_edit::PenTool;
use ph2d_vec_scene::{PaintSlot, Rgba8, VecPath, VecPathId, VecScene};
use std::cell::Cell;

/// A aparência de UM objecto preso por um cartão solto (`None` = não é uma forma): a mesma porta da
/// seleção, sem misto (um só).
fn aparencia_do_preso(
    id: VecPathId,
    sim: &SimWorld,
    scene: &VecScene,
    map: &ph2d_vec_entities::entities::VecEntityMap,
    store: &WidgetStore,
    doc: ph2d_editor_core::DocPx,
) -> Option<painel::Appearance> {
    let mut vivo = ph2d_vec_scene::VecViewState::default();
    ph2d_app_vec::driven_style::live_layers(sim, map, store, &mut vivo);
    pilha::published(scene, &[id], &vivo, doc)
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
    painel::set_current_appearance(
        id.and_then(|i| aparencia_do_preso(i, sim, scene, map, store, doc)),
    );
    stroke_present::publica_de(p);
    appearance::set_current_vector_style(p.map(|p| {
        let fill = ponte::seed_fill_from_paint(p.base_fill())
            .or_else(|| p.base_fill().and_then(stroke_present::ramp).map(|r| r[0]));
        let traco = p.base_stroke();
        appearance::molde_do_estilo().unwrap_or_default().da_forma(
            fill,
            traco,
            traco.map_or(0.0, |s| doc.of_world(s.width)),
        )
    }));
    ph2d_editor_core::panel::cards::estilos::set_ligados(
        id.map_or_else(Vec::new, |i| pilha::estilos::ligados_de(scene, i)),
    );
}

/// O que só se lê ANTES do respawn: a chave durável dos objectos com proporção do Scale guardada.
pub struct AntesDoRestauro(Vec<(u64, ph2d_ecs::StableId)>);

fn anota_travas(world: &ph2d_ecs::World) {
    ph2d_panel_inspector::anota_chaves_de_trava(
        |bits| ph2d_ecs::Entity::try_from_bits(bits).is_some_and(|e| world.get_entity(e).is_ok()),
        |bits| ph2d_card_rope::chave::chave_de(world, bits),
    );
}

/// ⭐ O quadro: os alfinetes e a proporção do Scale ligam os bits de agora à chave durável do objecto,
/// nos dois sentidos, enquanto ele está no mundo para a dar.
pub fn chaves_do_quadro(store: &mut WidgetStore, world: &ph2d_ecs::World) {
    ph2d_card_rope::chave::resolve(store, world);
    anota_travas(world);
    ph2d_panel_inspector::devolve_travas(|(id, nome)| {
        ph2d_card_rope::chave::achar(world, *id, nome)
    });
}

/// Chame antes de o restauro apagar o mundo: depois dele os bits de agora não dizem quem eram.
#[must_use]
pub fn antes_do_restauro(world: &ph2d_ecs::World) -> AntesDoRestauro {
    anota_travas(world);
    let chave = |bits: u64| {
        let e = ph2d_ecs::Entity::try_from_bits(bits)?;
        let id = world
            .get::<ph2d_ecs::StableId>(e)
            .filter(|s| !s.is_none())?;
        Some((bits, *id))
    };
    AntesDoRestauro(
        ph2d_panel_inspector::travas_de_escala()
            .into_iter()
            .filter_map(chave)
            .collect(),
    )
}

/// ⭐ **Logo depois de um restauro** (Ctrl+Z, Ctrl+Y, abrir): o gizmo, os alfinetes e a proporção do
/// Scale passam já aos bits novos. O quadro seguinte publica o Inspector antes do `sync_selection` e do
/// `chave::resolve` dele, e um quadro sem seleção fechava os cartões todos. Medido na app:
/// `doc CartoesUI/10`, «Smoke 3».
#[allow(clippy::too_many_arguments)] // a composição: cada parte é de uma família
pub fn depois_do_restauro(
    hero: &mut ph2d_editor_core::screens::hero::HeroScreen,
    sim: &SimWorld,
    scene: &VecScene,
    map: &ph2d_vec_entities::entities::VecEntityMap,
    pen: &mut PenTool,
    sel: &mut ph2d_app_vec::selection_sync::VecSelSync,
    tools: &ph2d_editor_core::tool::ToolRegistry,
    antes: &AntesDoRestauro,
) {
    ph2d_panel_inspector::reata_travas_de_escala(|bits| {
        let (_, id) = antes.0.iter().find(|(b, _)| *b == bits)?;
        ph2d_ecs::entity_with_stable_id(sim.world(), *id).map(ph2d_ecs::Entity::to_bits)
    });
    let vector_active = !hero.store.family_owns_canvas()
        && tools
            .active()
            .is_some_and(|t| t.id() == ph2d_editor_core::ToolId::new("vector"));
    ph2d_app_vec::vec_selection::sync_selection(
        &mut hero.gizmo,
        sim,
        scene,
        map,
        pen,
        sel,
        vector_active,
    );
    chaves_do_quadro(&mut hero.store, sim.world());
}

/// O gesto de base vai à forma do cartão preso sob ele; sem cartão, à selecionada.
pub fn apply_no_alvo(
    scene: &mut VecScene,
    pen: &PenTool,
    store: &WidgetStore,
    sim: &SimWorld,
    verb: BaseVerb,
) -> bool {
    match alvo_do_gesto(store, sim) {
        Some(id) => stroke_present::apply_em(scene, &[id], pen, verb),
        None => stroke_present::apply(scene, pen, verb),
    }
}

/// A cor do seletor na tinta de base de `path` — a lei de [`ponte::pick_merge`], cada canal só se o
/// gesto o tocou. Uma base escondida é editada escondida; um degradê fica quieto. Devolve se mudou.
#[must_use]
pub fn pick_base(path: &mut VecPath, fill: bool, nova: Rgba8, tocou: bool) -> bool {
    let campos = |cur: Rgba8| {
        let mut f = F::NONE;
        if tocou || (cur.r, cur.g, cur.b) != (nova.r, nova.g, nova.b) {
            f |= F::FILL_RGB;
        }
        if cur.a != nova.a {
            f |= F::FILL_A;
        }
        f
    };
    if fill {
        let Some(c) = ponte::seed_fill_from_paint(path.base_fill()) else {
            return false;
        };
        let cur = Rgba8::new(c[0], c[1], c[2], c[3]);
        let m = ponte::pick_merge(cur, nova, campos(cur));
        if m == cur {
            return false;
        }
        let closed = path.closed;
        let alvo = if path.fill_is_hidden() {
            &mut path.fill_hidden
        } else {
            &mut path.fill
        };
        return ponte::apply_fill_colour(alvo, closed, m);
    }
    let Some(s) = path.slot_stroke_mut(PaintSlot::Base) else {
        return false;
    };
    let cur = s.color();
    let m = ponte::pick_merge(cur, nova, campos(cur));
    if m == cur {
        return false;
    }
    s.paint = ponte::recolour_stroke_paint(&s.paint, m);
    true
}

thread_local! {
    /// O cursor desta escrita no contador de escolhas do seletor partilhado.
    static PICKS_SEEN: Cell<u64> = const { Cell::new(0) };
}

fn existe(sim: &SimWorld, bits: u64) -> bool {
    ph2d_ecs::Entity::try_from_bits(bits).is_some_and(|e| sim.world().get_entity(e).is_ok())
}

/// **O seletor de cor aberto por um cartão preso**: a cor vai à tinta de base do objecto dele, sem
/// passar pela tool (a ponte não o lê). Com o objecto apagado, fecha. Uma vez por quadro.
fn seletor_na_base(store: &mut WidgetStore, scene: &mut VecScene, sim: &SimWorld) {
    let picker = ph2d_editor_core::ids::INSP_BLENDER_PICKER;
    let n = store.colour_picks(picker);
    let tocou = PICKS_SEEN.with(|c| c.replace(n)) != n;
    let Some(bits) = store.picker_subject() else {
        return;
    };
    let Some(id) = forma(sim, bits) else {
        // O objecto saiu. Uma sprite não é uma forma, e a amostra dela é do Inspector, que a escreve.
        if !existe(sim, bits) {
            store.set_picker_target(None);
        }
        return;
    };
    let alvo = store.picker_target();
    let fill = alvo == Some(ph2d_tool_vector::ids::VECTOR_FILL_SWATCH);
    if !fill && alvo != Some(ph2d_tool_vector::ids::VECTOR_STROKE_SWATCH) {
        return; // a amostra de uma camada: é do `apply_no_quadro`
    }
    let Some((value, ..)) = store.blender_picker(picker) else {
        return;
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
        && pick_base(path, fill, nova, tocou)
    {
        ph2d_app_vec::bindings::note_authored(if fill {
            ph2d_ecs::BoundProp::Fill
        } else {
            ph2d_ecs::BoundProp::StrokeColor
        });
        ph2d_app_vec::bindings::note_alvo(id);
    }
}

/// O quadro do seletor e dos estilos. Os pedidos da lista são do gesto, a vista dela é da seleção e
/// o seletor de cor é de quem o abriu: um cartão preso continua no objecto dele.
fn apply_no_quadro(
    store: &mut WidgetStore,
    scene: &mut VecScene,
    pen: &PenTool,
    sim: &SimWorld,
) -> bool {
    let (sel, mostrada) = (pilha::alvos(store, pen, sim), pilha::ordered(pen));
    let seletor = store
        .picker_subject()
        .and_then(|b| forma(sim, b))
        .map_or_else(|| mostrada.clone(), |id| vec![id]);
    let estilos = pilha::estilos::drena_de(store, scene, &sel, &mostrada, &seletor);
    pilha::pica(store, scene, &seletor) || estilos
}

/// Os pedidos da pilha de aparência que o dreno recolheu neste quadro.
#[derive(Clone, Copy, Debug, Default)]
pub struct PedidosDaPilha {
    pub vec_opacity: Option<f64>,
    pub vec_blend: Option<u8>,
    pub verb: Option<pilha::StackVerb>,
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
    let sel = pilha::alvos(store, pen, sim);
    if let Some(track) = p.vec_opacity {
        #[allow(clippy::cast_possible_truncation)]
        ph2d_app_vec::appearance::set_opacity(scene, &sel, track as f32);
        // Numa forma que a timeline conduz, o valor que a TELA mostra (plano 34 D3): o auto-key vê a
        // mão: armado grava, desarmado segura até o tempo mudar.
        #[allow(clippy::cast_possible_truncation)]
        ph2d_app_vec::driven_style::follow_the_hand(sim, map, &sel, track as f32);
    }
    if let Some(code) = p.vec_blend {
        ph2d_app_vec::appearance::set_blend(scene, &sel, code);
    }
    if let Some(v) = p.verb {
        pilha::apply(scene, &sel, v);
    }
    apply_no_quadro(store, scene, pen, sim);
    seletor_na_base(store, scene, sim);
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
mod tests {
    use super::pick_base;
    use ph2d_vec_scene::{Paint, Rgba8, StrokeSpec, VecPath, rectangle};

    fn forma() -> VecPath {
        let mut p = rectangle([0.0, 0.0], [2.0, 2.0]);
        p.fill = Some(Paint::Solid(Rgba8::new(10, 20, 30, 128)));
        p.stroke = Some(StrokeSpec::new(Rgba8::new(40, 50, 60, 200), 1.0));
        p
    }

    #[test]
    fn the_picker_keeps_the_alpha_it_did_not_touch_and_an_open_picker_writes_nothing() {
        let mut p = forma();
        assert!(!pick_base(&mut p, true, Rgba8::new(10, 20, 30, 128), false));
        let nova = Rgba8::new(1, 2, 3, 128);
        assert!(pick_base(&mut p, true, nova, true));
        assert_eq!(p.fill, Some(Paint::Solid(nova)));
        let traco = Rgba8::new(7, 8, 9, 200);
        assert!(pick_base(&mut p, false, traco, true));
        assert_eq!(p.base_stroke().map(StrokeSpec::color), Some(traco));
        assert_eq!(p.fill, Some(Paint::Solid(nova)), "o traço mexeu no fill");
    }

    #[test]
    fn the_picker_on_a_hidden_base_edits_it_hidden() {
        let mut p = forma();
        assert!(p.set_fill_hidden(true));
        let nova = Rgba8::new(1, 2, 3, 128);
        assert!(pick_base(&mut p, true, nova, true));
        assert!(p.fill_is_hidden(), "a escolha destapou a tinta");
        assert_eq!(p.base_fill(), Some(&Paint::Solid(nova)));
    }
}

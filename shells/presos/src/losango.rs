//! ⭐ **C3 — o losango de um campo PRESO** (`docs/CartoesUI/09`): o estado e o clique são do objecto do
//! cartão, com o contexto DELE (a pilha dele; a camada aberta no painel é da seleção). As famílias que
//! dependem de «quem» passam todas por aqui, para a seleção e para cada preso.

use ph2d_app_vec::marcas::Contexto;
use ph2d_ecs::canal_vetor_ids::IdsDoVetor;
use ph2d_ecs::{FieldKey, SimWorld};
use ph2d_editor_core::NodeId;
use ph2d_editor_core::ids::{INSP_PANEL, SKELETON_PANEL};
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::widget::key_mark::{self, KeyMark};
use ph2d_panel_inspector::ids::INSP_SPRITE_PIVOT_XY;
use ph2d_timeline::{Instante, PropKind, TimelineDoc, TimelineIntent};
use ph2d_vec_scene::VecScene;
use std::collections::BTreeMap;

/// Os objectos presos pelos cartões soltos do Inspector e do Bones, sem repetidos.
#[must_use]
pub fn pinos(store: &WidgetStore) -> Vec<u64> {
    let mut v = store.loose_pins_of(INSP_PANEL);
    v.extend(store.loose_pins_of(SKELETON_PANEL));
    v.sort_unstable();
    v.dedup();
    v
}

/// O contexto de um preso: a pilha e os ids dele, sem camada aberta nem cota escolhida (são da seleção).
#[must_use]
pub fn contexto<'a>(sim: &'a SimWorld, scene: &VecScene, bits: u64) -> Contexto<'a> {
    let forma = ph2d_app_vec::paint_stack::forma(sim, bits).and_then(|id| scene.path(id));
    Contexto {
        aberta: None,
        camadas: forma.map_or(0, |p| p.paints.len()),
        regra: None,
        ids: sim
            .world()
            .get_resource::<IdsDoVetor>()
            .and_then(|m| m.0.get(&bits)),
    }
}

/// De quem é um clique num losango: do objecto do cartão preso, com o contexto dele; senão, da seleção.
#[must_use]
pub fn alvo_do_clique<'a>(
    c: &key_mark::KeyMarkClick,
    sim: &'a SimWorld,
    scene: &VecScene,
    da_selecao: Contexto<'a>,
    sel: &[u64],
) -> (Contexto<'a>, Vec<u64>) {
    match c.sujeito {
        Some(p) => (contexto(sim, scene, p), vec![p]),
        None => (da_selecao, sel.to_vec()),
    }
}

/// O mapa das famílias que dependem do objecto, para `sel`.
#[must_use]
pub fn marcas(
    tl: &TimelineDoc,
    sim: &SimWorld,
    ctx: Contexto<'_>,
    sel: &[u64],
    inst: Instante,
    valor: impl Fn(u64, PropKind) -> Option<f32> + Copy,
) -> BTreeMap<NodeId, KeyMark> {
    let mut m = ph2d_app_vec::marcas::marcas(tl, ctx, sel, inst, valor);
    m.extend(ph2d_app_components::marcas_inspector::marcas(
        tl, sel, inst, valor,
    ));
    m.extend(ph2d_app_constraints::marcas::marcas(
        tl, sim, sel, inst, valor,
    ));
    m.extend(ph2d_app_skeleton::canais_osso::marcas(
        tl, sim, sel, inst, valor,
    ));
    m
}

/// Publica o mapa de cada preso, para a pintura deste quadro.
pub fn publica_marcas(
    tl: &TimelineDoc,
    sim: &SimWorld,
    scene: &VecScene,
    pins: &[u64],
    inst: Instante,
    valor: impl Fn(u64, PropKind) -> Option<f32> + Copy,
) {
    key_mark::set_key_marks_dos_presos(
        pins.iter()
            .map(|&b| {
                let ctx = contexto(sim, scene, b);
                (b, marcas(tl, sim, ctx, &[b], inst, valor))
            })
            .collect(),
    );
}

/// O que o clique num losango escreve em `sel`: a primeira família que conhece o campo.
#[must_use]
#[allow(clippy::too_many_arguments)] // o irmão dos `clique` de cada família
pub fn clique(
    tl: &TimelineDoc,
    sim: &SimWorld,
    ctx: Contexto<'_>,
    campos: &[NodeId],
    sel: &[u64],
    inst: Instante,
    lidos: impl Fn(u64) -> Vec<FieldKey> + Copy,
    valor: impl Fn(u64, PropKind) -> Option<f32> + Copy,
    k: impl Fn(u64, PropKind) -> Option<TimelineIntent> + Copy,
) -> Vec<TimelineIntent> {
    use ph2d_app_components::marcas_inspector as comp;
    use ph2d_app_constraints::marcas as cons;
    use ph2d_app_skeleton::canais_osso as osso;
    let mut i = ph2d_app_vec::marcas::clique(tl, ctx, campos, sel, inst, lidos, valor, k);
    if i.is_empty() {
        i = comp::clique(tl, campos, sel, inst, lidos, valor, k);
        if campos.iter().any(|c| INSP_SPRITE_PIVOT_XY.contains(c)) {
            use ph2d_sprite_edit::pivot_chave as pivo;
            let filhos = pivo::filhos_do_pivo(sim.world(), sel.iter().copied());
            i = pivo::clique_leva_os_filhos(i, tl, &filhos, inst, valor, k);
            pivo::iguala_o_ritmo(&mut i, tl);
        }
    }
    if i.is_empty() {
        i = cons::clique(tl, campos, sim, sel, inst, lidos, valor, k);
    }
    if i.is_empty() {
        i = osso::clique(tl, campos, sim, sel, inst, lidos, valor, k);
    }
    i
}

thread_local! {
    /// Cena 12: quantos cartões já receberam o clique.
    static CENA: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// `PH2D_VEC_CARTOES_SMOKE=12`: o despacho real recebe um clique no 1.º losango do Stroke preso a A (a
/// `fase_autokey` grava-o); o da cópia presa a B fica no ponto. Sem a env, nada.
pub fn cena(hero: &mut ph2d_editor_core::screens::hero::HeroScreen) {
    use ph2d_editor_core::interaction::{dispatch_pointer, loose_rect_of};
    use ph2d_editor_core::registry::hash_node_id as h;
    use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
    if std::env::var("PH2D_VEC_CARTOES_SMOKE").as_deref() != Ok("12") {
        return;
    }
    let cartoes = [h("cards.insp.stroke")];
    let n = CENA.with(std::cell::Cell::get);
    let Some(&sec) = cartoes.get(n) else {
        return;
    };
    let Some(r) = hero.store.panel_rect(loose_rect_of(sec)) else {
        return;
    };
    if hero.store.loose_card(sec).and_then(|c| c.pin).is_none() {
        return;
    }
    let mut pontos = (0..)
        .map(|i| r.y + 2.0 * i as f32)
        .take_while(|y| *y < r.y + r.h);
    let achado = pontos.find_map(|y| {
        (0..)
            .map(|i| r.x + 2.0 * i as f32)
            .take_while(|x| *x < r.x + r.w)
            .find(|&x| {
                hero.hit_index
                    .hit(x, y)
                    .is_some_and(|id| key_mark::key_mark_target(id).is_some())
            })
            .map(|x| (x, y))
    });
    let Some((x, y)) = achado else { return };
    let arena = bumpalo::Bump::new();
    for kind in [PointerKind::Down, PointerKind::Up] {
        let ev = PointerEvent {
            x,
            y,
            pressure: 1.0,
            kind,
            source: PointerSource::Mouse,
            button: PointerButton::Primary,
            timestamp_ns: 0,
        };
        let _ = dispatch_pointer(&mut hero.store, &hero.hit_index, ev, &arena);
    }
    CENA.with(|c| c.set(n + 1));
    eprintln!("[cartoes-smoke] cena 12: clique no losango do cartão {n} em ({x}, {y})");
}

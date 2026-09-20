//! **The Add Node catalog + the full-screen palette model** — split from `motion_bridge` for the shell
//! LOC cap. `super` is `render_loop::motion_bridge`. Gated by the parent's `#[cfg(feature =
//! "panel-motion-graph")]` mod declaration, so nothing here needs its own cfg.
//!
//! Two concerns, one place: `build_catalog` is the flat list the graph menu publishes; `build_palette_model`
//! groups that same list into the coloured, sub-clustered model the shell's "Add Node" palette renders.
//! Both derive from the registry — the single source of what a node is called and which category it wears.
//! Plus the two handshake ends: `open_pending_palette` (a gesture asked → open, filtered) and
//! `route_palette_pick` (last frame's pick → the right graph edit).

use crate::motion_state::MotionState;
use ph2d_editor_core::HeroScreen;
use ph2d_i18n::TextKey;

/// Route LAST frame's palette pick into a graph edit — mapping the picked id back to its canonical
/// `type_name`, then turning it (WITH the gesture's wire context, drained from `library_open`) into the
/// right intent via `library_pick`: a plain add, a smart-connect, or a splice. Call BEFORE draining
/// intents so the edit lands this frame. A click and an Enter on the palette both arrive here, so they
/// route identically.
pub(super) fn route_palette_pick(hero: &mut HeroScreen, motion: &mut MotionState) {
    // ⚠️ CONDICIONAL, e é o que torna a ordem dos drenos irrelevante. O canal do pick tem DOIS
    // consumidores desde a paleta de comandos global (`hero::global_palette`); um `take`
    // incondicional aqui engoliria um comando do chrome e devolveria `None` a quem o soubesse
    // executar, com o sintoma a ser *«às vezes não faz nada»*.
    let known = |id| {
        motion
            .registry
            .manifests()
            .any(|m| ph2d_tool_registry::hash_node_id(m.name) == id)
    };
    let Some(id) = hero.store.take_command_pick_if(known) else {
        return;
    };
    let type_name = motion
        .registry
        .manifests()
        .map(|m| m.name)
        .find(|name| ph2d_tool_registry::hash_node_id(name) == id);
    if let Some(type_name) = type_name {
        let open = motion.library_open.take().unwrap_or_default();
        ph2d_panel_motion_graph::push_intent(ph2d_panel_motion_graph::library_pick(
            open.connect_from,
            open.connect_to,
            open.splice,
            type_name,
            open.spawn,
        ));
    }
}

/// If a gesture asked (`OpenLibrary`, stashed in `open_library`), open the full-screen palette on the
/// live catalog — FILTERED to the compatible types for a smart-connect — and remember the wire context
/// in `library_open` for [`route_palette_pick`].
pub(super) fn open_pending_palette(hero: &mut HeroScreen, motion: &mut MotionState) {
    if let Some(open) = motion.open_library.take() {
        hero.store
            .open_command_palette(build_palette_model(&motion.registry, &open.compatible));
        motion.library_open = Some(open);
    }
}

/// Build the addable-node catalog from the registry (canonical name + English display label + category),
/// sorted by category then label so the menu groups by colour (the palette teaches the library map,
/// plan §2.4).
pub(super) fn build_catalog(
    registry: &ph2d_node_registry::NodeRegistry,
) -> Vec<ph2d_panel_motion_graph::NodeChoice> {
    use ph2d_node_registry::NodeUiCategory;
    use ph2d_panel_motion_graph::NodeChoice;
    let mut v: Vec<NodeChoice> = registry
        .manifests()
        // ⚠️ **As FIXTURAS não são oferecidas** (doc 89, folha 17): o `debug.const` (o "1º
        // nó" do W1.T3) e o `debug.wave` (o template de fan-out) existem para o motor e
        // apareciam na paleta com o nome cru do tipo. Quem responde é o REGISTO, por
        // opt-in — uma lista escrita aqui seria a segunda resposta à mesma pergunta, e a
        // que envelhece é sempre a que o artista vê.
        .filter(|m| !registry.is_fixture(m.id))
        // ⚠️ **E os RETIRADOS também não** (doc 115 W6) — outra bandeira, de propósito.
        // Uma fixtura NUNCA foi para o artista; um retirado ERA, e o dono fechou-lhe a
        // porta enquanto o motor fica vivo para outro consumidor. Fundir as duas numa
        // só apagaria essa diferença, e é ela que diz à próxima pessoa se o que falta
        // é um fio (morto) ou uma decisão (retirado).
        .filter(|m| !registry.is_out_of_catalogue(m.id))
        .map(|m| {
            let ui = registry.ui_manifest(m.id);
            NodeChoice {
                type_name: m.name,
                display: ui.map(|u| ph2d_i18n::tr(u.display_key)).unwrap_or(m.name),
                category: ui.map(|u| u.category).unwrap_or(NodeUiCategory::Utility),
                // Straight off the manifest (`&'static`), so the panel can filter
                // the smart-connect menu by what each type can actually take.
                inputs: m.inputs,
                // O espelho, para a paleta aberta de uma ENTRADA — ver o campo.
                outputs: m.outputs,
            }
        })
        .collect();
    v.sort_by(|a, b| (a.category as u8, a.display).cmp(&(b.category as u8, b.display)));
    v
}

/// Build the full-screen "Add Node" palette model from the live catalog: the 7 categories in display
/// order, each with the `node-cat-*` colour from the panel's single `cat_token`, and the two overloaded
/// categories (Transform, Utility) split into named sub-clusters so the layout stays scannable. Every
/// item id is `hash_node_id(type_name)` — the same hash the pick round-trips through.
///
/// `compatible` is the smart-connect allow-list: when non-empty, only those node types are shown (a wire
/// dropped in empty space opens the palette filtered to what it can feed, replacing the old dropdown's
/// filter). Empty = the whole catalog (plain `A` / R-click).
pub(super) fn build_palette_model(
    registry: &ph2d_node_registry::NodeRegistry,
    compatible: &[&'static str],
) -> ph2d_editor_core::widget::command_palette::PaletteModel {
    use ph2d_editor_core::widget::command_palette::{
        PaletteGroup, PaletteItem, PaletteModel, PaletteSub,
    };
    use ph2d_node_registry::NodeUiCategory;

    let mut cat = build_catalog(registry); // already sorted by (category, display)
    if !compatible.is_empty() {
        cat.retain(|nc| compatible.contains(&nc.type_name));
    }
    const ORDER: [(NodeUiCategory, TextKey); 7] = [
        (
            NodeUiCategory::Source,
            TextKey::new("panel.motion_graph.library.source"),
        ),
        (
            NodeUiCategory::Distribute,
            TextKey::new("panel.motion_graph.library.distribute"),
        ),
        (
            NodeUiCategory::Transform,
            TextKey::new("panel.motion_graph.library.transform"),
        ),
        (
            NodeUiCategory::Focus,
            TextKey::new("panel.motion_graph.library.focus"),
        ),
        (
            NodeUiCategory::Fx,
            TextKey::new("panel.motion_graph.library.fx"),
        ),
        (
            NodeUiCategory::Output,
            TextKey::new("panel.motion_graph.library.output"),
        ),
        (
            NodeUiCategory::Utility,
            TextKey::new("panel.motion_graph.library.utility"),
        ),
    ];
    let make_item = |nc: &ph2d_panel_motion_graph::NodeChoice| PaletteItem {
        label: nc.display.to_string(),
        id: ph2d_tool_registry::hash_node_id(nc.type_name),
    };
    let mut groups = Vec::new();
    for (c, title) in ORDER {
        let in_cat: Vec<&ph2d_panel_motion_graph::NodeChoice> = cat
            .iter()
            .filter(|nc| nc.category as u8 == c as u8)
            .collect();
        if in_cat.is_empty() {
            continue;
        }
        let sub_titles = palette_subgroups(c);
        let subs = if sub_titles.is_empty() {
            vec![PaletteSub {
                title: None,
                items: in_cat.iter().map(|nc| make_item(nc)).collect(),
            }]
        } else {
            sub_titles
                .iter()
                .filter_map(|&st| {
                    let items: Vec<PaletteItem> = in_cat
                        .iter()
                        .filter(|nc| palette_subgroup_of(c, nc.display) == Some(st))
                        .map(|nc| make_item(nc))
                        .collect();
                    (!items.is_empty()).then_some(PaletteSub {
                        title: Some(st.tr().to_string()),
                        items,
                    })
                })
                .collect()
        };
        groups.push(PaletteGroup {
            title: title.tr().to_string(),
            color: ph2d_panel_motion_graph::cat_token(c),
            subs,
        });
    }
    PaletteModel {
        title: ph2d_i18n::tr("panel.motion_graph.library.add_node").to_string(),
        groups,
        // A biblioteca de nós não tem caixa nenhuma — ver `PaletteModel::toggle`.
        toggle: None,
    }
}

/// The named sub-clusters for the two overloaded categories (empty = a flat category). Order is the
/// display order in the palette. ⚠️ KEYS (HR-15): the sub-cluster is also the GROUPING identity
/// (`palette_subgroup_of` returns one of these), so it compares as a key and is translated only
/// where the title is written.
fn palette_subgroups(c: ph2d_node_registry::NodeUiCategory) -> &'static [TextKey] {
    use ph2d_node_registry::NodeUiCategory;
    match c {
        NodeUiCategory::Transform => &[BASIC, DEFORMERS, FORCES, RIGGING, BEHAVIORS],
        NodeUiCategory::Utility => &[VALUES, TIME_SIGNAL, DATA],
        _ => &[],
    }
}

const BASIC: TextKey = TextKey::new("panel.motion_graph.library.basic_transforms");
const DEFORMERS: TextKey = TextKey::new("panel.motion_graph.library.deformers");
const FORCES: TextKey = TextKey::new("panel.motion_graph.library.forces_and_physics");
const RIGGING: TextKey = TextKey::new("panel.motion_graph.library.rigging");
const BEHAVIORS: TextKey = TextKey::new("panel.motion_graph.library.behaviors_and_timing");
const VALUES: TextKey = TextKey::new("panel.motion_graph.library.values_and_math");
const TIME_SIGNAL: TextKey = TextKey::new("panel.motion_graph.library.time_and_signal");
const DATA: TextKey = TextKey::new("panel.motion_graph.library.data_and_adapters");

/// Which sub-cluster a node belongs to, by display name. The last arm is a CATCH-ALL, so a node added to
/// the registry later lands in a sensible cluster instead of vanishing from the palette.
fn palette_subgroup_of(c: ph2d_node_registry::NodeUiCategory, display: &str) -> Option<TextKey> {
    use ph2d_node_registry::NodeUiCategory;
    match c {
        NodeUiCategory::Transform => Some(match display {
            "Move" | "Rotate" | "Scale" | "Transform" | "Mirror" | "Orbit" | "Look At" => BASIC,
            "Bend" | "Twist" | "Spherize" | "Four Point Warp" | "Kaleidoscope" | "Spline Wrap" => {
                DEFORMERS
            }
            "Attractor" | "Vortex" | "Wind" | "Drag" | "Curl Noise" | "Noise" | "Spring"
            | "Integrate" | "Collide" | "Collider" | "Buoyancy" | "Simulation Step"
            | "Simulation Zone" => FORCES,
            "FABRIK" | "FK" | "IK 2-Bone" | "Rubber Hose" | "Skin" => RIGGING,
            _ => BEHAVIORS,
        }),
        NodeUiCategory::Utility => Some(match display {
            "Math" | "Unary" | "Compare" | "Gain" | "Mix" | "Normalize" | "Quantize"
            | "Threshold" | "Step" | "Wrap" | "Slope" | "Smooth" | "Median" | "Percentile"
            | "Reduce" | "Sort" | "Cull" => VALUES,
            "Time" | "Time Remap" | "LFO" | "Beat" | "Counter" | "On Change" | "Sample & Hold" => {
                TIME_SIGNAL
            }
            _ => DATA,
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::build_catalog;

    /// ⭐ **A PALETA NÃO OFERECE FIXTURA** — medido no CATÁLOGO, não no registo.
    ///
    /// ⚠️ **É a metade que o gate do censo não vê.** O censo
    /// (`every_offered_node_has_a_name_and_every_fixture_has_none`) afirma que os dois
    /// `debug.*` se DECLARAM fixturas; este afirma que o catálogo **honra** a declaração.
    /// Tirar o `.filter(...)` daqui deixaria o censo verde e devolveria os dois nós à
    /// tecla `A` — *declarar e ser honrado são duas coisas, e só a segunda é o produto.*
    #[test]
    fn the_palette_never_offers_a_fixture() {
        let mut reg = ph2d_node_registry::NodeRegistry::new();
        ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registram");
        let cat = build_catalog(&reg);
        assert!(!cat.is_empty(), "CONTROLE: o catalogo nao esta' vazio");
        for nc in &cat {
            assert!(
                !nc.type_name.starts_with("debug."),
                "o catalogo oferece `{}` — o filtro de fixturas caiu",
                nc.type_name
            );
        }
        // E o REGISTO tem-nos: o catálogo é que os esconde, não o registo que os perdeu.
        //
        // ⚠️⚠️ **A PREMISSA DESTA CONTA MORREU na W6 do doc 115, e o número não se
        // corrige — a conta é que passa a ter DUAS parcelas.** Ela dizia *«exactamente
        // as duas fixturas ficam de fora»* e lia `manifests − catálogo == 2`, o que era
        // verdade enquanto **ser fixtura** fosse a única razão para não ser oferecido.
        // Hoje há uma segunda, e oposta: um nó RETIRADO (`is_out_of_catalogue`), que o
        // artista já teve e o dono fechou. *Somar as duas num literal apagaria a
        // diferença que as duas bandeiras existem para guardar* — e a próxima pessoa
        // leria «faltam 3 fixturas».
        let fixturas = reg.manifests().filter(|m| reg.is_fixture(m.id)).count();
        let retirados = reg
            .manifests()
            .filter(|m| reg.is_out_of_catalogue(m.id))
            .count();
        assert_eq!(fixturas, 2, "exactamente as duas fixturas se declaram");
        assert_eq!(retirados, 1, "exactamente um tipo foi RETIRADO (o colisor)");
        // ⭐ E o total é DERIVADO das duas parcelas, nunca escrito à mão: quem
        // acrescentar uma terceira razão de esconder tem de a somar aqui, e um nó que
        // se esconda sem bandeira nenhuma faz esta linha reprovar por diferença.
        assert_eq!(
            reg.manifests().count() - cat.len(),
            fixturas + retirados,
            "o catalogo esconde EXACTAMENTE quem se declarou escondido — nem mais um"
        );
    }
}

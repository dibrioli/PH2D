//! ⭐⭐ **As linhas que a cena não tem** — a raiz CENA e o CONTEÚDO de cada objecto, desdobrados
//! sobre a lista que a shell publica (doc 01 §4–§7, [`ph2d_editor_core::hier_rows`]).
//!
//! ⚠️ **Entram na MESMA lista** que os objectos, e é de propósito: o galo, o recolher, as linhas de
//! parentesco, a listra e a busca valem para elas sem uma segunda porta. O que as separa é o
//! [`RowRole`] — só as [`RowRole::Object`] são publicadas como `hierarchy_row_ids`, e é isso que as
//! deixa fora do arrasto, do menu de objecto e do renomear.

use std::collections::{BTreeMap, BTreeSet};

use ph2d_a11y::NodeId;
use ph2d_editor_core::hier_rows::{RowRole, SCENE_COMPONENTS};
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::screens::hero::fixture::HierarchyEntity;

use crate::ids::{HIER_SCENE_ROW, hier_content_row};

/// A lista da shell, com a Cena à cabeça (tudo um nível abaixo dela) e o conteúdo de cada objecto
/// logo a seguir à linha dele — antes dos filhos, que são PARTES e não conteúdo.
pub(crate) fn expand(
    ordered: &[NodeId],
    mut entries: BTreeMap<NodeId, HierarchyEntity>,
) -> (Vec<NodeId>, BTreeMap<NodeId, HierarchyEntity>) {
    let mut order = Vec::with_capacity(ordered.len() + 1);
    let mut out = BTreeMap::new();
    let anything_selected = entries.values().any(|e| e.selected);
    order.push(HIER_SCENE_ROW);
    out.insert(HIER_SCENE_ROW, scene_row(!anything_selected));
    for &id in ordered {
        let Some(mut owner) = entries.remove(&id) else {
            continue;
        };
        owner.indent = owner.indent.saturating_add(1);
        let content = std::mem::take(&mut owner.content);
        let base = owner.indent;
        let owner_selected = owner.selected;
        order.push(id);
        out.insert(id, owner);
        for (g, group) in content.iter().enumerate() {
            let Ok(g) = u8::try_from(g) else { break };
            let head = hier_content_row(id, g, None);
            order.push(head);
            out.insert(
                head,
                synthetic(
                    ph2d_i18n::tr_with(group.label_key, &[("n", &group.items.len())]),
                    group.icon,
                    base.saturating_add(1),
                    RowRole::ContentGroup {
                        owner: id,
                        group: g,
                    },
                    false,
                ),
            );
            for item in &group.items {
                let row = hier_content_row(id, g, Some(item.key));
                order.push(row);
                out.insert(
                    row,
                    synthetic(
                        item.name.clone(),
                        group.item_icon,
                        base.saturating_add(2).saturating_add(item.depth),
                        RowRole::ContentItem {
                            owner: id,
                            group: g,
                            key: item.key,
                            mode: group.mode,
                        },
                        owner_selected && item.active,
                    ),
                );
            }
        }
    }
    (order, out)
}

fn scene_row(selected: bool) -> HierarchyEntity {
    let mut row = synthetic(
        ph2d_i18n::tr("hier_rows.scene").to_string(),
        IconId::Scene,
        0,
        RowRole::Scene,
        selected,
    );
    row.components = SCENE_COMPONENTS.iter().map(|(c, _)| *c).collect();
    row
}

fn synthetic(
    name: String,
    icon: IconId,
    indent: u8,
    role: RowRole,
    selected: bool,
) -> HierarchyEntity {
    HierarchyEntity {
        name,
        icon,
        indent,
        badge: None,
        swatch: None,
        visible: true,
        selected,
        hovered: false,
        muted: false,
        locked: false,
        group_locked: false,
        role,
        components: Vec::new(),
        content: Vec::new(),
    }
}

/// A linha `id` está recolhida? O conteúdo nasce FECHADO; o resto, aberto.
pub(crate) fn is_collapsed(store: &WidgetStore, id: NodeId, row: &HierarchyEntity) -> bool {
    match row.role {
        RowRole::ContentGroup { .. } => !store.is_hierarchy_content_open(id),
        _ => store.is_hierarchy_collapsed(id),
    }
}

/// Só os OBJECTOS — o conjunto que o arrasto, o menu e o renomear reconhecem como linha.
pub(crate) fn object_rows(
    order: &[NodeId],
    entries: &BTreeMap<NodeId, HierarchyEntity>,
) -> BTreeSet<NodeId> {
    order
        .iter()
        .copied()
        .filter(|id| entries.get(id).is_none_or(|e| e.role == RowRole::Object))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_editor_core::hier_rows::{ContentGroup, ContentItem};
    use ph2d_editor_core::object_mode::ObjectMode;

    fn object(name: &str, indent: u8) -> HierarchyEntity {
        synthetic(name.into(), IconId::Sprite, indent, RowRole::Object, false)
    }

    #[test]
    fn a_cena_e_a_raiz_e_o_conteudo_vem_antes_dos_filhos() {
        let (a, b) = (NodeId(100_000), NodeId(100_001));
        let mut img = object("Herói", 0);
        img.content = vec![ContentGroup {
            label_key: "hier_rows.content.layers",
            icon: IconId::Layers,
            item_icon: IconId::Layer,
            mode: ObjectMode::Paint,
            items: vec![ContentItem {
                key: 7,
                name: "Tinta".into(),
                depth: 0,
                active: true,
            }],
        }];
        let entries = BTreeMap::from([(a, img), (b, object("Espada", 1))]);
        let (order, rows) = expand(&[a, b], entries);
        let head = hier_content_row(a, 0, None);
        let item = hier_content_row(a, 0, Some(7));
        assert_eq!(order, vec![HIER_SCENE_ROW, a, head, item, b]);
        let indents: Vec<u8> = order.iter().map(|id| rows[id].indent).collect();
        assert_eq!(indents, vec![0, 1, 2, 3, 2]);
        assert_eq!(rows[&head].name, "Layers (1)");
        assert!(rows[&HIER_SCENE_ROW].selected, "nada seleccionado = a Cena");
        assert_eq!(object_rows(&order, &rows), BTreeSet::from([a, b]));
    }
}

//! ⭐⭐ **A Animação na Hierarquia** (doc ObjectsAndComponents 01 §6): o ícone 🎞 na linha do
//! objecto que tem clips, e os clips como CONTEÚDO dele — *Clips (3)* ▸ andar · pular — que se
//! editam na Timeline. A Timeline declara; a shell só passa o documento (a lei do §8 F3: «a
//! família declara, a shell não escreve a lista»).

use ph2d_component_desc::ComponentCategory;
use ph2d_editor_core::hier_rows::{ContentGroup, ContentItem, ContentOpens, row_categories};
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::screens::hero::PANEL_TIMELINE;
use ph2d_timeline::{SCENE, TimelineState};

/// O grupo *Clips (n)* da animação de `owner` ([`SCENE`] = a da Cena), ou `None` sem clips. A
/// `key` de cada peça é o índice do clip no documento; a activa é a que a Timeline edita.
#[must_use]
pub fn clips_content(timeline: &TimelineState, owner: u64) -> Option<ContentGroup> {
    let doc = &timeline.doc;
    let in_view = timeline.view.owner == owner;
    let items: Vec<ContentItem> = doc
        .clips_of(owner)
        .map(|i| ContentItem {
            key: i as u64,
            name: doc.clips()[i].name.clone(),
            depth: 0,
            active: in_view && i == doc.active_index(),
        })
        .collect();
    (!items.is_empty()).then_some(ContentGroup {
        label_key: "hier_rows.content.clips",
        icon: IconId::Film,
        item_icon: IconId::Layer,
        opens: ContentOpens::Panel(PANEL_TIMELINE),
        items,
    })
}

/// **Declara a Animação na linha de `bits`**: o ícone da família e o grupo de clips — nada, se o
/// objecto não tem Animação.
pub fn declare_in_row(
    timeline: &TimelineState,
    bits: u64,
    components: &mut Vec<ComponentCategory>,
    content: &mut Vec<ContentGroup>,
) {
    if bits == SCENE {
        return;
    }
    let Some(group) = clips_content(timeline, bits) else {
        return;
    };
    *components = row_categories(components.iter().copied().chain([ComponentCategory::Animation]));
    content.push(group);
}

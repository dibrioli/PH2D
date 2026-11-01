//! A cor do ícone de cada TIPO de objecto na linha (F42) — irmão do `row.rs` pelo tecto de 600 LOC.

use super::*;
use ph2d_editor_core::hier_rows::RowRole;
use ph2d_editor_core::hier_types::ObjectType;

fn linha(icon: IconId, role: RowRole, selected: bool, muted: bool) -> fixture::HierarchyEntity {
    fixture::HierarchyEntity {
        name: String::new(),
        icon,
        indent: 0,
        badge: None,
        swatch: None,
        visible: true,
        selected,
        hovered: false,
        muted,
        locked: false,
        group_locked: false,
        role,
        components: Vec::new(),
        content: Vec::new(),
        has: Default::default(),
        entity: None,
    }
}

/// ⭐ F42 — o ícone de um objecto pinta a cor do TIPO, seleccionado ou não; escondido fica
/// cinzento; a Cena e o conteúdo não ganham cor de tipo. (Mutação: devolver `Text3` em vez da
/// cor do tipo ⇒ RED; tirar o filtro do papel ⇒ a Cena com glifo de tipo pintaria a cor — RED.)
#[test]
fn o_icone_do_objecto_pinta_a_cor_do_tipo() {
    for t in ObjectType::ALL {
        assert_eq!(
            icon_color(&linha(t.icon(), RowRole::Object, false, false)),
            t.color()
        );
        assert_eq!(
            icon_color(&linha(t.icon(), RowRole::Object, true, false)),
            t.color()
        );
        assert_eq!(
            icon_color(&linha(t.icon(), RowRole::Object, false, true)),
            ColorToken::TextDisabled
        );
    }
    let cena = linha(ObjectType::Image.icon(), RowRole::Scene, false, false);
    assert_eq!(icon_color(&cena), ColorToken::Text3);
    let pasta = linha(IconId::Folder, RowRole::Object, true, false);
    assert_eq!(icon_color(&pasta), ColorToken::Accent);
}

use super::*;
use crate::action_bus::{EditorAction, HierRequest};
use crate::screens::hero::global_palette::{build_global_model, route_global_pick};
use crate::widget::command_palette::split_shortcut;
use ph2d_input::Key;

fn hero() -> HeroScreen {
    HeroScreen::new(NodeId(1))
}

fn item<'m>(
    m: &'m crate::widget::command_palette::PaletteModel,
    id: NodeId,
) -> Option<(&'m str, &'m PaletteItem)> {
    m.groups.iter().find_map(|g| {
        g.subs
            .iter()
            .flat_map(|s| &s.items)
            .find(|it| it.id == id)
            .map(|it| (g.title.as_str(), it))
    })
}

/// ⭐ **As ações do Input Map entram com as teclas; escolher abre a aba nelas.**
/// Mutação: `route` sem o ramo das ações ⇒ o pick cai no `Click` e a aba não abre, sangra.
#[test]
fn a_acao_do_input_map_vem_com_as_teclas_e_abre_a_aba() {
    let mut h = hero();
    let jump = h.input_map.create("wave_hello");
    if let Some(a) = h.input_map.get_mut(jump) {
        a.bindings.push(Binding::Key(Key(0x20)));
        a.bindings.push(Binding::Key(Key(0x57)));
    }
    let m = build_global_model(&h);
    let (group, it) = item(&m, action_item(jump)).expect("a ação não está na busca");
    assert_eq!(group, tr("chrome.palette.input_map"));
    let (name, keys) = split_shortcut(&it.label);
    assert_eq!(name, "wave_hello");
    assert_eq!(keys, Some(format!("{}, W", Key(0x20).label()).as_str()));
    assert!(route_global_pick(&mut h, action_item(jump)));
    assert!(h.store.take_input_map_tab_request(), "a aba não foi pedida");
    assert_eq!(h.store.take_input_map_action(), Some(jump));
}

/// ⭐ **Os objetos da cena entram pelo nome; escolher seleciona a linha e pisca-a.**
#[test]
fn escolher_um_objeto_seleciona_o() {
    let mut h = hero();
    let row = NodeId(0x0B1E_C7);
    h.store.set_scene_objects(vec![(row, "Player".into())]);
    let m = build_global_model(&h);
    let (group, it) = item(&m, row).expect("o objeto não está na busca");
    assert_eq!(
        (group, it.label.as_str()),
        (tr("chrome.palette.scene"), "Player")
    );
    assert!(route_global_pick(&mut h, row));
    assert!(
        h.bus.iter().any(|a| matches!(
            a,
            EditorAction::Hierarchy(HierRequest::SelectRow { row: r, .. }) if *r == row
        )),
        "a escolha não pediu a seleção da linha"
    );
    assert_eq!(h.store.ring_anywhere(true).map(|(id, _)| id), Some(row));
}

/// ⭐ **Os campos que os cartões pintaram entram com o cartão no caminho; escolher dá-lhes o
/// teclado e pisca-os; o cartão dobrado entra pelo título e desdobra.**
#[test]
fn os_campos_dos_cartoes_entram_pelo_rotulo_e_o_cartao_dobrado_pelo_titulo() {
    let mut h = hero();
    let (transform, pos, physics) = (NodeId(0x51), NodeId(0x52), NodeId(0x53));
    h.store.begin_card_directory_frame();
    h.store.card_painting(transform, "Transform");
    h.store.card_field_painted("Position", pos, 0.0);
    h.store.card_painted();
    h.store.card_painting(physics, "Physics");
    h.store.card_painted();
    h.store.card_field_painted("Fora", NodeId(0x54), 0.0);
    let m = build_global_model(&h);
    let (group, it) = item(&m, pos).expect("o campo não está na busca");
    assert_eq!(
        (group, it.label.as_str()),
        (tr("chrome.palette.inspector"), "Position")
    );
    assert!(item(&m, physics).is_some(), "o cartão dobrado não entrou");
    assert!(
        item(&m, NodeId(0x54)).is_none(),
        "um campo fora de cartão entrou"
    );
    assert!(route_global_pick(&mut h, pos));
    assert_eq!(h.store.focus_id(), Some(pos));
    h.store.set_collapsed(physics, true);
    assert!(route_global_pick(&mut h, physics));
    assert!(
        !h.store.is_collapsed(physics),
        "escolher o cartão não o desdobrou"
    );
}

/// O que os cartões pintaram há dois quadros já não é deste objeto.
#[test]
fn os_campos_de_um_quadro_velho_saem() {
    let mut h = hero();
    h.store.begin_card_directory_frame();
    h.store.card_painting(NodeId(0x61), "Transform");
    h.store.card_field_painted("Position", NodeId(0x62), 0.0);
    h.store.begin_card_directory_frame();
    h.store.begin_card_directory_frame();
    assert!(item(&build_global_model(&h), NodeId(0x62)).is_none());
}

/// ⭐ **Na busca vazia os comandos dos menus vêm antes das ferramentas** (dono 12/11).
#[test]
fn na_busca_vazia_os_menus_vem_primeiro() {
    let m = build_global_model(&hero());
    let titles: Vec<&str> = m.groups.iter().map(|g| g.title.as_str()).collect();
    let file = titles.iter().position(|t| *t == tr("chrome.menu_bar.file"));
    let tools = titles.iter().position(|t| *t == tr("chrome.palette.tools"));
    assert_eq!(file, Some(0), "o File não é o primeiro grupo: {titles:?}");
    assert!(
        tools.is_none_or(|t| t > 4),
        "ferramentas antes dos menus: {titles:?}"
    );
}

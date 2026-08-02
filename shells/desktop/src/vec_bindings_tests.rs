//! Os gates do resolvedor de tokens.

use super::*;
use crate::vec_entities::{VecEntityMap, sync};
use ph2d_vec_scene::{VecPathId, VecScene, rectangle};

/// Uma cena com uma forma, e o mapa `VecPathId → entidade`.
fn scene() -> (SimWorld, VecScene, VecEntityMap, VecPathId) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let id = scene.push_path(rectangle([0.0, 0.0], [2.0, 1.0]));
    sync(&mut sim, &mut scene, &mut map);
    (sim, scene, map, id)
}

fn bind(sim: &mut SimWorld, map: &VecEntityMap, id: VecPathId, prop: BoundProp, token: &str) {
    let e = Entity::from_bits(map[&id]);
    let mut b = sim
        .world()
        .get::<VecBindings>(e)
        .cloned()
        .unwrap_or_default();
    b.set(prop, token);
    sim.world_mut().entity_mut(e).insert(b);
}

/// **Sem binding, a tabela sai VAZIA** — e é isso que faz todo documento que já existe desenhar
/// byte-idêntico ao mundo pré-token.
#[test]
fn a_document_with_no_bindings_publishes_nothing() {
    let (sim, _scene, map, _id) = scene();
    assert!(resolve(&sim, &map, Theme::Forge).is_empty());
}

/// **O MODO decide a cor.** É a entrega inteira da wave: a mesma arte, dois modos, duas cores — e
/// sem tocar no documento.
#[test]
fn the_same_binding_resolves_to_a_different_colour_in_another_mode() {
    let (mut sim, _scene, map, id) = scene();
    bind(&mut sim, &map, id, BoundProp::Fill, "accent");

    let forge = resolve(&sim, &map, Theme::Forge);
    let sunstone = resolve(&sim, &map, Theme::Sunstone);
    assert_eq!(forge.len(), 1);
    assert_eq!(sunstone.len(), 1);
    assert_eq!(forge[0].path, id);
    assert_ne!(
        forge[0].fill, sunstone[0].fill,
        "trocar de modo tem de re-vestir a arte; iguais, o binding nao serve para nada"
    );
    assert_eq!(
        forge[0].fill,
        token_color("accent", Theme::Forge),
        "a cor vem da porta unica, nao de uma segunda tabela"
    );
}

/// **Um token que não existe deixa o LITERAL valer.**
///
/// A alternativa (uma cor de emergência) pintaria de errado uma forma que estava certa, e o
/// artista veria a arte mudar sem ter mexido nela.
#[test]
fn an_unknown_token_falls_back_to_the_literal() {
    let (mut sim, _scene, map, id) = scene();
    bind(&mut sim, &map, id, BoundProp::Fill, "no-such-token");
    assert!(token_color("no-such-token", Theme::Forge).is_none());
    assert!(
        resolve(&sim, &map, Theme::Forge).is_empty(),
        "nada resolvido ⇒ nada publicado ⇒ o desenho usa o literal do documento"
    );
}

/// As duas propriedades chegam juntas na mesma entrada.
#[test]
fn fill_and_stroke_ride_the_same_entry() {
    let (mut sim, _scene, map, id) = scene();
    bind(&mut sim, &map, id, BoundProp::Fill, "accent");
    bind(&mut sim, &map, id, BoundProp::StrokeColor, "border");
    let out = resolve(&sim, &map, Theme::Forge);
    assert_eq!(out.len(), 1, "uma forma, uma entrada");
    assert_eq!(out[0].fill, token_color("accent", Theme::Forge));
    assert_eq!(out[0].stroke, token_color("border", Theme::Forge));
}

/// ⚠️ **A chave do documento tem de casar com a chave que o token EMITE.**
///
/// O gate percorre a lista inteira e afirma o round-trip. Sem ele, um token cuja chave o
/// `from_key` não reconhecesse ficaria para sempre no fallback: o artista o escolheria no picker e
/// a arte não mudaria, em silêncio.
#[test]
fn every_token_the_picker_offers_resolves_by_its_own_key() {
    for &t in ColorToken::ALL {
        assert_eq!(
            ColorToken::from_key(t.key()),
            Some(t),
            "token {:?} nao volta pela propria chave",
            t
        );
        assert!(token_color(t.key(), Theme::Forge).is_some());
    }
    assert!(
        ColorToken::ALL.len() >= 60,
        "a lista encolheu — o picker perdeu tokens"
    );
}

/// **A SEQUÊNCIA leva a algum lugar** — a 4ª condição de UI, e a que nenhuma das outras três
/// implica: todo edit pode ter gate, todo widget pode estar registado e clicável, e o gesto ainda
/// não chegar a lado nenhum.
///
/// A corrente inteira: **o id da opção → a propriedade + o token → o componente no ECS → a tinta
/// resolvida → o `VecPath` que o renderer recebe** — e trocar de modo re-veste.
#[test]
fn the_whole_chain_from_the_click_to_the_drawn_paint() {
    use ph2d_vec_scene::{Paint, VecViewState};

    let (mut sim, mut scene, map, id) = scene();
    let literal = ph2d_vec_scene::Rgba8::new(9, 9, 9, 255);
    if let Some(p) = scene.path_mut(id) {
        p.fill = Some(Paint::Solid(literal));
    }

    // 1. O id que o picker pinta para "accent" — decodificado pela porta do PRODUTO.
    let row = 1 + ColorToken::ALL
        .iter()
        .position(|t| t.key() == "accent")
        .expect("o token accent existe na tabela");
    let opt = ph2d_editor::ids::vector_token_option_id(0, row);
    let (prop, token) = token_choice(opt).expect("o id e' uma escolha do picker");
    assert_eq!(prop, BoundProp::Fill);
    assert_eq!(token, Some("accent"));

    // 2. A shell escreve no ECS.
    set_selected_binding(&mut sim, &map, &[id], prop, token);

    // 3+4. O resolvedor produz a tinta, o desenho a usa — e ela DIFERE entre dois modos.
    let mut view = VecViewState::default();
    for theme in [Theme::Forge, Theme::Sunstone] {
        view.bound = resolve(&sim, &map, theme);
        let path = scene.path(id).expect("a forma existe");
        assert_eq!(
            path.painted(view.bound_paint(id)).fill,
            token_color("accent", theme).map(Paint::Solid),
            "o desenho tem de mostrar o token resolvido no modo {theme:?}"
        );
        assert_ne!(
            path.painted(view.bound_paint(id)).fill,
            Some(Paint::Solid(literal)),
            "e nao o literal"
        );
    }

    // 5. Soltar devolve o LITERAL — bindar nunca o apagou.
    set_selected_binding(&mut sim, &map, &[id], prop, None);
    view.bound = resolve(&sim, &map, Theme::Forge);
    let path = scene.path(id).expect("a forma existe");
    assert_eq!(
        path.painted(view.bound_paint(id)).fill,
        Some(Paint::Solid(literal)),
        "soltar tem de devolver a cor que o artista escreveu"
    );
}

/// **O componente DESANEXA quando fica vazio.**
///
/// Um `VecBindings` sem entradas viaja no save e entra no diff do undo — e então duas cenas
/// logicamente iguais comparam diferente, que é o passo espúrio que o `canonicalize` do undo
/// global existe para matar.
#[test]
fn unbinding_the_last_property_detaches_the_component() {
    let (mut sim, _scene, map, id) = scene();
    set_selected_binding(&mut sim, &map, &[id], BoundProp::Fill, Some("accent"));
    let e = Entity::from_bits(map[&id]);
    assert!(sim.world().get::<VecBindings>(e).is_some());
    set_selected_binding(&mut sim, &map, &[id], BoundProp::Fill, None);
    assert!(
        sim.world().get::<VecBindings>(e).is_none(),
        "um componente VAZIO ficou anexado — ele entra no diff do undo e no save"
    );
}

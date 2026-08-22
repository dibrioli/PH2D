//! Os gates do RECORTE — a metade que decide QUEM pode recortar, e o que ligar o chip faz.
//!
//! ⚠️ A propriedade que estes gates existem para defender não é *"o recorte funciona"* (isso o
//! `vec_frame_spans_tests` já provava quando só a moldura recortava): é que ele passou a alcançar
//! **qualquer forma fechada** — e que alcançá-la **não a transforma numa moldura**. As duas
//! metades importam. Sem a primeira o pedido do Enio não foi entregue; sem a segunda a estrela
//! ganha um rótulo flutuante com nome, alças de contêiner e presets de telefone.

use super::*;
use crate::vec_entities::VecEntityMap;
use ph2d_ecs::{Transform, VecFrame, VecPathRef};
use ph2d_vec_scene::{VecScene, line, rectangle};

/// Um mundo com UMA forma — fechada (retângulo) ou aberta (reta) — e o mapa `VecPathId → entidade`.
fn world(closed: bool) -> (SimWorld, VecScene, VecEntityMap, VecPathId) {
    let mut sim = SimWorld::new();
    let mut scene = VecScene::new();
    let id = if closed {
        scene.push_path(rectangle([0.0, 0.0], [4.0, 3.0]))
    } else {
        scene.push_path(line([0.0, 0.0], [4.0, 3.0]))
    };
    let e = sim
        .world_mut()
        .spawn((Transform::default(), VecPathRef(id)))
        .id();
    let mut map = VecEntityMap::new();
    map.insert(id, e.to_bits());
    (sim, scene, map, id)
}

/// **Uma forma fechada QUALQUER oferece o recorte** — o pedido, em uma asserção (Enio, 2026-08-21:
/// *"coloque a feature Clip Content para qualquer forma vetorial fechada"*).
///
/// Nasceu vermelho: `selected_frame_clip` exigia `VecFrame`, então um retângulo comum respondia
/// `None` e a seção não era pintada.
#[test]
fn any_closed_shape_offers_the_clip_even_without_being_a_frame() {
    let (sim, scene, map, id) = world(true);
    // Controlo positivo: sem ele, o teste passaria mesmo que a fixture tivesse posto um
    // `VecFrame` — e estaria a provar o comportamento ANTIGO com o nome do novo.
    assert!(
        sim.world()
            .get::<VecFrame>(ph2d_ecs::Entity::from_bits(map[&id]))
            .is_none(),
        "a fixture NAO pode ser moldura, senao o gate nao prova nada"
    );
    assert_eq!(
        selected_clip(&sim, &scene, &map, &[id]),
        Some(false),
        "uma forma fechada tem de OFERECER o recorte, desligado"
    );
}

/// **Uma forma ABERTA não oferece.** Um caminho sem interior não tem "dentro": o Vello recortaria
/// por uma silhueta implícita que o artista nunca desenhou, e arte sumiria atrás de uma fronteira
/// invisível.
#[test]
fn an_open_path_does_not_offer_the_clip() {
    let (sim, scene, map, id) = world(false);
    assert_eq!(
        selected_clip(&sim, &scene, &map, &[id]),
        None,
        "uma reta nao tem interior -- o controlo nao pode ser oferecido"
    );
}

/// **Ligar o recorte NÃO faz da forma uma moldura** — a propriedade que motivou a separação dos
/// dois componentes, e a única razão de o `VecClipContent` existir em vez de reusar o `VecFrame`.
///
/// Se um dia alguém "simplificar" isto pendurando o `VecFrame`, este gate fica vermelho — e o
/// sintoma no canvas seria um nome flutuante sobre a estrela, alças de contêiner e a seção Frame
/// com presets de telefone.
#[test]
fn turning_the_clip_on_does_not_turn_the_shape_into_a_frame() {
    let (mut sim, scene, map, id) = world(true);
    assert!(set_selected_clip(&mut sim, &scene, &map, &[id], true));

    let e = ph2d_ecs::Entity::from_bits(map[&id]);
    assert!(
        sim.world().get::<ph2d_ecs::VecClipContent>(e).is_some(),
        "o recorte tem de estar ligado"
    );
    assert!(
        sim.world().get::<VecFrame>(e).is_none(),
        "e a forma tem de continuar NAO sendo moldura"
    );
}

/// **Desligar REMOVE o componente, e um no-op não mente.** As duas metades: a ausência é o estado
/// "não recorta" (um documento sem recortes fica byte-idêntico ao de antes da feature), e o
/// `false` de retorno é o que impede um clique repetido de custar um passo de undo — o
/// `post_frame_undo` captura por diff, mas quem sabe que nada mudou é este retorno.
#[test]
fn switching_off_removes_the_component_and_a_noop_reports_no_change() {
    let (mut sim, scene, map, id) = world(true);
    let e = ph2d_ecs::Entity::from_bits(map[&id]);

    assert!(set_selected_clip(&mut sim, &scene, &map, &[id], true));
    assert!(
        !set_selected_clip(&mut sim, &scene, &map, &[id], true),
        "ligar o que ja' esta' ligado nao mudou nada"
    );
    assert!(set_selected_clip(&mut sim, &scene, &map, &[id], false));
    assert!(
        sim.world().get::<ph2d_ecs::VecClipContent>(e).is_none(),
        "desligar REMOVE -- nao deixa um componente inerte para o undo tropecar"
    );
    assert!(
        !set_selected_clip(&mut sim, &scene, &map, &[id], false),
        "e desligar o que ja' esta' desligado tambem nao"
    );
}

/// **O chip alcança o objeto ATRAVÉS da seleção expandida** — migrado do `vec_frame_edit_tests`
/// junto com o que ele testa.
///
/// ⚠️ É a metade que um gate de leitura não pega: sem ela o chip mostraria o estado certo e não
/// editaria nada. Selecionar um contêiner costuma trazer os filhos junto (box-select, Ctrl+A, o
/// contêiner dentro de um grupo), e é sobre ESSA lista que a escrita tem de acertar o pai.
#[test]
fn the_chip_writes_through_a_selection_that_carries_the_children() {
    let mut sim = SimWorld::new();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();

    let parent = scene.push_path(rectangle([0.0, 0.0], [8.0, 6.0]));
    let kids: Vec<VecPathId> = (0..2)
        .map(|k| scene.push_path(rectangle([k as f64, 0.0], [k as f64 + 0.5, 0.5])))
        .collect();
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);

    let pe = ph2d_ecs::Entity::from_bits(map[&parent]);
    for id in &kids {
        let e = ph2d_ecs::Entity::from_bits(map[id]);
        sim.world_mut().entity_mut(e).insert(ph2d_ecs::ChildOf(pe));
    }

    let sel: Vec<VecPathId> = std::iter::once(parent)
        .chain(kids.iter().copied())
        .collect();
    assert_eq!(sel.len(), 3, "premissa: o pai chega acompanhado dos filhos");
    assert!(set_selected_clip(&mut sim, &scene, &map, &sel, true));
    assert!(
        sim.world().get::<ph2d_ecs::VecClipContent>(pe).is_some(),
        "o recorte tem de aterrar no PAI, nao num filho"
    );
    assert_eq!(selected_clip(&sim, &scene, &map, &sel), Some(true));
}

/// **Uma seleção sem entidade viva não oferece nada** — a herança literal do `frame_of_selection`:
/// ali *"contém tudo o que está selecionado"* deixaria de significar o que diz.
#[test]
fn a_selection_whose_world_is_gone_offers_nothing() {
    let (sim, scene, map, id) = world(true);
    let ghost: VecPathId = id + 999;
    assert_eq!(selected_clip(&sim, &scene, &map, &[id, ghost]), None);
    assert_eq!(
        selected_clip(&sim, &scene, &map, &[]),
        None,
        "nada selecionado"
    );
}

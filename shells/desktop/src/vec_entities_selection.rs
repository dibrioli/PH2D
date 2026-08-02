//! **A ancestralidade da árvore, e o que uma SELEÇÃO significa.**
//!
//! Irmão do [`super`] pelo teto de 600 LOC da shell, cortado por ASSUNTO: lá mora o que a ponte
//! path ⟺ entidade **mantém**; aqui, o que a árvore **responde** — *quem é o objeto que este
//! clique nomeia?* e *o que selecionar esta entidade significa?*
//!
//! As duas perguntas têm UMA resposta cada, e é isso que impede a Hierarquia e o canvas de
//! selecionarem coisas diferentes ao apontar para o mesmo objeto.

use super::{MAX_DEPTH, VecEntityMap};
use ph2d_ecs::{ChildOf, Entity, SimWorld, VecPathRef};
use ph2d_vec_scene::{VecPathId, VecScene};

/// O ancestral de topo de `entity` (ele mesmo, se já é raiz).
#[must_use]
pub(crate) fn top_ancestor(sim: &SimWorld, entity: Entity) -> Entity {
    let w = sim.world();
    let mut cur = entity;
    for _ in 0..MAX_DEPTH {
        match w.get::<ChildOf>(cur) {
            Some(c) => cur = c.parent(),
            None => break,
        }
    }
    cur
}

/// Todos os paths vetoriais na sub-árvore de `entity` (ele mesmo, se for um path),
/// **na ordem de z do documento** — a seleção fica estável entre frames.
#[must_use]
pub(crate) fn subtree_paths(sim: &SimWorld, scene: &VecScene, entity: Entity) -> Vec<VecPathId> {
    let w = sim.world();
    let mut found: Vec<VecPathId> = Vec::new();
    let mut stack = vec![entity];
    while let Some(e) = stack.pop() {
        if let Some(vp) = w.get::<VecPathRef>(e) {
            found.push(vp.0);
        }
        if let Some(kids) = w.get::<ph2d_ecs::Children>(e) {
            stack.extend(kids.iter().copied());
        }
    }
    scene
        .paths()
        .iter()
        .map(|p| p.id)
        .filter(|id| found.contains(id))
        .collect()
}

/// **O que SELECIONAR esta entidade SIGNIFICA** — a porta única da expansão.
///
/// # A lei, numa frase
///
/// *Uma entidade que é ela própria uma FORMA responde por si; só um grupo PURO empresta a
/// sub-árvore.*
///
/// Um grupo não tem geometria nenhuma: se ele não emprestasse os filhos, selecioná-lo daria
/// zero caminhos e o grupo seria inestilizável, ineditável e invisível para a booleana. Uma
/// moldura, ao contrário, **É** um retângulo desenhado — e emprestar os filhos dela faz o
/// artista mudar a cor de UMA forma e ver SEIS mudarem (report do Enio, 2026-08-02).
///
/// ⚠️ Não confundir com [`subtree_paths`], que responde outra pergunta — *o que há LÁ DENTRO?*
/// — e continua certa para quem a faz (o recorte do layout, a booleana de grupo, e o
/// `fully_selected_ancestors`, que quer saber se um ancestral está INTEIRAMENTE coberto).
#[must_use]
pub(crate) fn selection_paths(sim: &SimWorld, scene: &VecScene, entity: Entity) -> Vec<VecPathId> {
    if let Some(vp) = sim.world().get::<VecPathRef>(entity) {
        let own = vp.0;
        // ⚠️ A entidade pode apontar para um path que o documento já não tem (o `sync` do frame
        // seguinte é que reconcilia): devolver o id na mesma poria um fantasma na seleção.
        return if scene.paths().iter().any(|p| p.id == own) {
            vec![own]
        } else {
            Vec::new()
        };
    }
    subtree_paths(sim, scene, entity)
}

/// **O ancestral que um CLIQUE nomeia**: sobe por grupos PUROS e para na primeira forma.
///
/// A subida existe porque um grupo é *"estas coisas como uma"* — tocar um membro pega o grupo,
/// a convenção de qualquer editor. Ela **para** numa moldura porque uma moldura não é isso: ela
/// é um objeto, e os filhos dela são objetos (o Figma, e a única leitura que deixa o artista
/// escolher UM filho para lhe dar `Grow`).
///
/// ⚠️ Antes desta parada, tocar um filho de moldura publicava a sub-árvore INTEIRA — e como
/// `item_of_selection` pede exactamente UMA forma, **as linhas Grow/Shrink eram inalcançáveis
/// pelo canvas**: um controlo que nunca aparecia, sem nada na tela a dizer porquê.
#[must_use]
pub(crate) fn selection_root(sim: &SimWorld, entity: Entity) -> Entity {
    let w = sim.world();
    let mut cur = entity;
    for _ in 0..MAX_DEPTH {
        let Some(parent) = w.get::<ChildOf>(cur).map(|c| c.parent()) else {
            break;
        };
        if w.get::<VecPathRef>(parent).is_some() {
            break; // o pai é uma FORMA (uma moldura): a fronteira do objeto é aqui.
        }
        cur = parent;
    }
    cur
}

/// A seleção de objeto que tocar `path` produz: o que [`selection_root`] nomeia, lido pela lei
/// de [`selection_paths`]. Fora de qualquer grupo isso é só ele — o comportamento de sempre.
#[must_use]
pub(crate) fn object_selection_for(
    sim: &SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    path: VecPathId,
) -> Vec<VecPathId> {
    let Some(&bits) = map.get(&path) else {
        return vec![path];
    };
    let root = selection_root(sim, Entity::from_bits(bits));
    let leaves = selection_paths(sim, scene, root);
    if leaves.is_empty() {
        vec![path]
    } else {
        leaves
    }
}

#[cfg(test)]
mod tests {
    use super::super::{bits, group_entities, setup, sync};
    use super::*;
    use ph2d_vec_scene::rectangle;

    /// Uma moldura com dois filhos, tudo já sincronizado. Devolve `(sim, scene, map, frame, kids)`.
    fn frame_with_kids() -> (SimWorld, VecScene, VecEntityMap, VecPathId, [VecPathId; 2]) {
        let (mut sim, mut scene, mut map) = setup();
        let frame = scene.push_path(rectangle([0.0, 0.0], [9.0, 9.0]));
        let a = scene.push_path(rectangle([1.0, 1.0], [2.0, 2.0]));
        let b = scene.push_path(rectangle([3.0, 1.0], [4.0, 2.0]));
        sync(&mut sim, &mut scene, &mut map);
        let fe = bits(&map, frame);
        for id in [a, b] {
            sim.world_mut()
                .entity_mut(bits(&map, id))
                .insert(ChildOf(fe));
        }
        (sim, scene, map, frame, [a, b])
    }

    /// **UMA FORMA RESPONDE POR SI** (Enio 2026-08-02: *"se mudo as cores ou espessura do stroke
    /// do pai, os filhos também mudam mesmo sem que estejam selecionados"*).
    ///
    /// Nasceu VERMELHO com três caminhos: a moldura emprestava a sub-árvore, e quem lê a seleção
    /// para restilizar via a moldura **e os dois filhos**.
    #[test]
    fn a_shape_answers_for_itself_never_for_its_children() {
        let (sim, scene, map, frame, kids) = frame_with_kids();
        assert_eq!(
            selection_paths(&sim, &scene, bits(&map, frame)),
            vec![frame],
            "a moldura emprestou os filhos — mudar a cor dela muda a deles"
        );
        assert_eq!(
            selection_paths(&sim, &scene, bits(&map, kids[0])),
            vec![kids[0]],
            "e uma folha continua a ser ela própria"
        );
    }

    /// **Um GRUPO PURO continua a emprestar** — sem isto, selecionar um grupo daria ZERO
    /// caminhos e ele ficaria inestilizável e ineditável. É a metade que impede a cura de virar
    /// uma regressão.
    #[test]
    fn a_pure_group_still_lends_its_subtree() {
        let (mut sim, mut scene, mut map) = setup();
        let a = scene.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
        let b = scene.push_path(rectangle([2.0, 0.0], [3.0, 1.0]));
        sync(&mut sim, &mut scene, &mut map);
        let g = group_entities(&mut sim, &[map[&a], map[&b]], "G".into()).unwrap();
        assert_eq!(
            selection_paths(&sim, &scene, Entity::from_bits(g)),
            vec![a, b]
        );
    }

    /// **O clique PARA na moldura.** Tocar um filho nomeia o filho — e não a moldura inteira,
    /// que era o que tornava as linhas Grow/Shrink inalcançáveis pelo canvas (elas pedem
    /// exactamente UMA forma selecionada).
    #[test]
    fn clicking_a_child_of_a_frame_names_the_child() {
        let (sim, scene, map, frame, kids) = frame_with_kids();
        assert_eq!(
            selection_root(&sim, bits(&map, kids[0])),
            bits(&map, kids[0]),
            "a subida atravessou a moldura"
        );
        assert_eq!(
            object_selection_for(&sim, &scene, &map, kids[0]),
            vec![kids[0]]
        );
        assert_eq!(
            object_selection_for(&sim, &scene, &map, frame),
            vec![frame],
            "e tocar a própria moldura também não a expande"
        );
    }

    /// Clicar num filho de um GRUPO seleciona o grupo inteiro; fora de grupo, só ele.
    #[test]
    fn object_selection_expands_to_the_top_group() {
        let (mut sim, mut scene, mut map) = setup();
        let a = scene.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
        let b = scene.push_path(rectangle([2.0, 0.0], [3.0, 1.0]));
        let c = scene.push_path(rectangle([4.0, 0.0], [5.0, 1.0]));
        sync(&mut sim, &mut scene, &mut map);
        assert_eq!(object_selection_for(&sim, &scene, &map, a), vec![a]);

        group_entities(&mut sim, &[map[&a], map[&b]], "G".into()).unwrap();
        assert_eq!(object_selection_for(&sim, &scene, &map, a), vec![a, b]);
        assert_eq!(
            object_selection_for(&sim, &scene, &map, c),
            vec![c],
            "solto"
        );
    }
}

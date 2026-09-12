//! **Os gates da ponte documento ↔ árvore** — módulo irmão de [`ph2d_vec_entities::entities`], pelo
//! teto de 600 LOC por ficheiro da shell (HR-18).
//!
//! ⚠️ **O corte é por RESPONSABILIDADE, não por tamanho**: ali mora o passe que reconcilia,
//! aqui as réguas que dizem o que ele tem de fazer. A fixtura (`setup`/`bits`) vem com eles,
//! porque ela só existe para os gates — e uma fixtura partilhada com o produto seria um
//! oráculo que usa a função sob teste.

use ph2d_ecs::{ChildOf, Entity, Name, RootOrder, SimWorld, Transform, VecPathRef};
use ph2d_vec_entities::entities::*;
use ph2d_vec_entities::entity_map::VecEntityMap;
use ph2d_vec_scene::VecViewState;
// ⭐ **A FIXTURA mudou-se para a crate** e chega por `test-support`: o irmão da SELECÇÃO usa-a
// de dentro dela, este ficheiro de fora, e *duas fixturas para a mesma ponte seriam duas
// respostas a «como nasce uma cena de teste?»* — a razão que esta secção já dava, agora a
// atravessar uma fronteira (HOWTO §2.5).
use ph2d_vec_entities::entities::{bits, initial_name, setup};

mod tests {
    use super::*;
    // ⚠️ Só os gates a usam desde que a porta de «está na cena?» passou a ser o `off_canvas`.
    use ph2d_ecs::Visibility;
    use ph2d_vec_scene::rectangle;

    /// ⭐⭐⭐ **A receita ABERTA entra na lista de EXEMPTAS do isolamento** (Enio, 2026-09-07).
    ///
    /// ⚠️ **A pergunta é o `MasterEditing`, a mesma marca que a faz voltar a ser visível** — e é
    /// isso que impede o isolamento de ser uma segunda resposta a *«qual é a receita aberta?»*.
    /// Sem receita aberta a lista é vazia, e o desenho é o de sempre.
    ///
    /// **Mutação que deve sangrar:** apagar o `view.isolated.push(id)`.
    #[test]
    fn the_open_prefab_is_isolated_in_the_view() {
        let mut sim = SimWorld::new();
        let mut map = VecEntityMap::default();
        let mut scene = ph2d_vec_scene::VecScene::new();
        let normal = scene.push_path(rectangle([0.0, 0.0], [10.0, 10.0]));
        let receita = scene.push_path(rectangle([20.0, 0.0], [10.0, 10.0]));
        sync(&mut sim, &mut scene, &mut map);
        let aberta = bits(&map, receita);
        sim.world_mut()
            .entity_mut(aberta)
            .insert(ph2d_ecs::MasterEditing);

        let view = view_state(&sim, &map);

        assert!(
            view.is_isolated(receita),
            "a receita aberta nao entrou nas exemptas — ela desenharia esbatida com o resto"
        );
        assert!(
            !view.is_isolated(normal),
            "uma forma comum entrou nas exemptas — o recuo ficaria com buracos nitidos"
        );
    }

    /// ⭐⭐⭐ **A arte VETORIAL de uma receita também sai da cena** (F4.6, o §14).
    ///
    /// ⛔ Enquanto a porta de *«está na cena?»* era só do extract de SPRITES, a regra tinha duas
    /// respostas: a forma de um mestre continuava a desenhar **por baixo da cópia** que o *Criar
    /// componente* deixa no lugar, e o artista não distinguia uma da outra.
    ///
    /// ⚠️ E a outra metade: enquanto a receita está a ser **editada**, ela volta.
    ///
    /// (Mutação: a cadeia voltar a ler só `Visibility` ⇒ RED.)
    #[test]
    fn the_vector_art_of_a_recipe_leaves_the_canvas_too() {
        let (mut sim, mut scene, mut map) = setup();
        let id = scene.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
        sync(&mut sim, &mut scene, &mut map);
        let e = bits(&map, id);
        assert!(
            !view_state(&sim, &map).is_hidden(id),
            "a forma nasceu escondida"
        );

        sim.world_mut().entity_mut(e).insert(ph2d_ecs::MasterRoot);
        ph2d_ecs::assign_master_pieces(sim.world_mut());
        assert!(
            view_state(&sim, &map).is_hidden(id),
            "a arte da receita continua a desenhar — dois objetos empilhados"
        );

        // E volta enquanto está a ser editada.
        crate::render_loop::master_editing_mark_for_tests(&mut sim, Some(e.to_bits()));
        assert!(
            !view_state(&sim, &map).is_hidden(id),
            "a receita nao volta ao ser editada — a forma do mestre fica inalcancavel"
        );
    }

    /// **Duplicar uma forma dá ao clone o PRÓPRIO path, e o `sync` cunha UMA entidade para ele.**
    ///
    /// ⚠️ É o invariante que a row **Duplicate** da Hierarchy violava: ela clonava a ENTIDADE
    /// (Transform + Name + ChildOf) e não o path, então o clone nascia sem `VecPathRef` — uma
    /// linha na Hierarchy sobre geometria nenhuma. E copiar o `VecPathRef` teria sido pior: duas
    /// entidades a apontar para o MESMO path, num mapa que é um-para-um.
    #[test]
    fn duplicating_a_shape_gives_the_copy_its_own_path_and_its_own_entity() {
        let (mut sim, mut scene, mut map) = setup();
        let a = scene.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
        sync(&mut sim, &mut scene, &mut map);
        assert_eq!(
            map.len(),
            1,
            "o CONTROLE falhou: o sync nao cunhou a origem"
        );

        let mut pen = ph2d_vec_edit::PenTool::default();
        assert!(
            crate::input_dispatch::duplicate_vec_paths(&mut scene, &mut pen, &[a], 5.0, 5.0),
            "a porta recusou duplicar um path que existe"
        );
        sync(&mut sim, &mut scene, &mut map);

        assert_eq!(scene.paths().len(), 2, "o documento nao ganhou a copia");
        assert_eq!(map.len(), 2, "o sync nao cunhou UMA entidade para a copia");
        let copy = scene
            .paths()
            .iter()
            .map(|p| p.id)
            .find(|id| *id != a)
            .expect("a copia tem id proprio");
        // As duas entidades apontam para paths DIFERENTES — nenhuma aliasing.
        let (ea, ec) = (bits(&map, a), bits(&map, copy));
        assert_ne!(ea, ec, "a copia herdou a entidade da origem");
        let ra = sim
            .world()
            .get::<VecPathRef>(ea)
            .copied()
            .expect("origem sem ref");
        let rc = sim
            .world()
            .get::<VecPathRef>(ec)
            .copied()
            .expect("copia sem ref");
        assert_ne!(ra.0, rc.0, "as duas entidades apontam para o MESMO path");

        // ⛔ A metade «UM Ctrl+Z desfaz a cópia inteira» MORREU em 2026-09-12 com a `History` do
        // vetor (`line/render-loop`, A9). Ela lia o `push_undo` desta porta — e o Ctrl+Z do produto
        // NUNCA leu aquela pilha: é a fila global, que regista o quadro por diff, e a cópia entra
        // nela por ser uma mudança do documento. O que a metade dizia medir já não tinha leitor.
    }

    /// O invariante da ponte: um path ⟺ uma entidade. Nas duas direções, e o
    /// sync é idempotente (rodar de novo não spawna fantasma).
    #[test]
    fn sync_keeps_one_entity_per_path_in_both_directions() {
        let (mut sim, mut scene, mut map) = setup();
        let a = scene.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
        let b = scene.push_path(rectangle([2.0, 0.0], [3.0, 1.0]));

        sync(&mut sim, &mut scene, &mut map);
        assert_eq!(map.len(), 2);
        let ea = bits(&map, a);
        assert!(sim.world().get::<VecPathRef>(ea).is_some_and(|v| v.0 == a));
        assert!(sim.world().get::<Name>(ea).is_some());

        // Idempotente.
        sync(&mut sim, &mut scene, &mut map);
        assert_eq!(map.len(), 2);
        assert_eq!(bits(&map, a), ea, "não respawnou");

        // Path apagado no canvas ⇒ entidade despawnada.
        scene.remove_path(b);
        sync(&mut sim, &mut scene, &mut map);
        assert_eq!(map.len(), 1);
        assert!(!map.contains_key(&b));

        // Entidade apagada pela Hierarquia ⇒ path removido do documento.
        sim.world_mut().despawn(ea);
        sync(&mut sim, &mut scene, &mut map);
        assert!(map.is_empty());
        assert!(scene.paths().is_empty(), "o path foi junto");
    }

    /// **A forma nova nasce com nome ÚNICO no MUNDO** — não só entre formas.
    ///
    /// `initial_name(id)` é único entre paths (o id é), mas o mundo tem sprites e objetos Flip
    /// junto: basta o artista ter renomeado um sprite para "Path 1" e a próxima forma nasceria
    /// homônima dele. Desde o W4.T6 isso não é cosmético — a animação reencontra o objeto **pelo
    /// nome** (`wire_id` = hash do `Name`), então dois homônimos são dois donos para a mesma
    /// track. O nome é IDENTIDADE agora, e passa pela mesma porta que o import e o rename usam.
    #[test]
    fn a_new_shape_never_takes_a_name_the_world_already_uses() {
        let (mut sim, mut scene, mut map) = setup();
        // O artista renomeou um sprite exatamente com o nome que a próxima forma pediria.
        let first = scene.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
        let squatter = initial_name(first);
        let mut scene2 = ph2d_vec_scene::VecScene::new();
        std::mem::swap(&mut scene, &mut scene2); // a forma ainda não entrou no mundo
        sim.world_mut()
            .spawn((Transform::default(), Name::new(squatter.clone())));
        std::mem::swap(&mut scene, &mut scene2);

        sync(&mut sim, &mut scene, &mut map);

        let names: Vec<String> = {
            let mut q = sim.world_mut().query::<&Name>();
            q.iter(sim.world()).map(|n| n.as_str().to_owned()).collect()
        };
        assert_eq!(names.len(), 2);
        assert_ne!(
            names[0], names[1],
            "a forma nova pegou o nome do sprite — duas tracks colariam no mesmo objeto: {names:?}"
        );
        assert!(
            names.contains(&squatter),
            "e o nome do sprite continua o dele"
        );
    }

    /// Grupo é uma ENTIDADE COMUM: aceita path vetorial e sprite no mesmo saco, e
    /// agrupar normaliza para os ancestrais de topo (aninha, não reparenta o filho).
    #[test]
    fn group_entities_nests_and_accepts_mixed_types() {
        let (mut sim, mut scene, mut map) = setup();
        let a = scene.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
        let b = scene.push_path(rectangle([2.0, 0.0], [3.0, 1.0]));
        sync(&mut sim, &mut scene, &mut map);
        // Um "sprite": entidade sem VecPathRef (o que importa é não ter geometria vetorial).
        let sprite = sim
            .world_mut()
            .spawn((Transform::default(), Name::new("Spr")))
            .id();

        let inner = group_entities(&mut sim, &[map[&a], map[&b]], "in".into()).unwrap();
        assert_eq!(
            subtree_paths(&sim, &scene, Entity::from_bits(inner)),
            vec![a, b]
        );

        // Agrupa o path `a` (que já está em `inner`) com o SPRITE → `inner` aninha.
        let outer = group_entities(&mut sim, &[map[&a], sprite.to_bits()], "out".into())
            .expect("`a` traz o grupo `inner` junto");
        assert_eq!(
            top_ancestor(&sim, bits(&map, a)).to_bits(),
            outer,
            "o topo agora é o grupo de fora"
        );
        assert_eq!(
            sim.world()
                .get::<ChildOf>(Entity::from_bits(inner))
                .unwrap()
                .parent()
                .to_bits(),
            outer
        );
        assert_eq!(
            top_ancestor(&sim, sprite).to_bits(),
            outer,
            "o sprite entrou junto"
        );
        assert_eq!(
            subtree_paths(&sim, &scene, Entity::from_bits(outer)),
            vec![a, b]
        );

        // Menos de 2 topos distintos = no-op.
        assert_eq!(
            group_entities(&mut sim, &[map[&a], map[&b]], "x".into()),
            None
        );
    }

    /// Desagrupar dissolve só GRUPOS PUROS: um sprite (ou path) com filhos é um pai,
    /// não um grupo — dissolvê-lo apagaria um objeto.
    #[test]
    fn ungroup_dissolves_plain_groups_but_never_a_parent_object() {
        let (mut sim, mut scene, mut map) = setup();
        let a = scene.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
        let b = scene.push_path(rectangle([2.0, 0.0], [3.0, 1.0]));
        sync(&mut sim, &mut scene, &mut map);

        let g = group_entities(&mut sim, &[map[&a], map[&b]], "G".into()).unwrap();
        assert_eq!(ungroup_entities(&mut sim, &[map[&a]]), 1);
        assert!(
            sim.world().get_entity(Entity::from_bits(g)).is_err(),
            "o grupo sumiu"
        );
        assert!(
            sim.world().get::<ChildOf>(bits(&map, a)).is_none(),
            "voltou pra raiz"
        );
        assert!(sim.world().get::<RootOrder>(bits(&map, a)).is_some());
        // Os paths continuam lá.
        sync(&mut sim, &mut scene, &mut map);
        assert_eq!(scene.paths().len(), 2);

        // `a` como PAI de `b` (não um grupo): ungroup não o dissolve.
        sim.world_mut()
            .entity_mut(bits(&map, b))
            .insert(ChildOf(bits(&map, a)));
        assert_eq!(ungroup_entities(&mut sim, &[map[&b]]), 0);
        assert!(
            sim.world().get_entity(bits(&map, a)).is_ok(),
            "o path-pai sobreviveu"
        );
    }

    /// Visibilidade e trava são HERDADAS, e o flag próprio do filho nunca é tocado.
    #[test]
    fn view_state_inherits_hiding_and_locking_from_the_ancestors() {
        let (mut sim, mut scene, mut map) = setup();
        let a = scene.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
        let b = scene.push_path(rectangle([2.0, 0.0], [3.0, 1.0]));
        sync(&mut sim, &mut scene, &mut map);
        let g =
            Entity::from_bits(group_entities(&mut sim, &[map[&a], map[&b]], "G".into()).unwrap());

        assert_eq!(
            view_state(&sim, &map),
            VecViewState::default(),
            "tudo livre"
        );

        sim.world_mut()
            .entity_mut(g)
            .insert(Visibility { hidden: true });
        let v = view_state(&sim, &map);
        assert!(v.is_hidden(a) && v.is_hidden(b));
        assert!(
            sim.world().get::<Visibility>(bits(&map, a)).is_none(),
            "o flag do filho nunca foi tocado"
        );
        sim.world_mut().entity_mut(g).remove::<Visibility>();
        assert!(!view_state(&sim, &map).is_hidden(a), "reabrir devolve");

        // `GroupedChildren` no grupo trava os descendentes (predicado do gizmo).
        sim.world_mut()
            .entity_mut(g)
            .insert(ph2d_ecs::GroupedChildren);
        let v = view_state(&sim, &map);
        assert!(!v.is_pickable(a) && !v.is_pickable(b));
        assert!(!v.is_hidden(a), "travado não é escondido");
    }
}

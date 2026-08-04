//! Os gates das INSTÂNCIAS de componente (plano UI/UX W5).
//!
//! A promessa da wave é uma frase — *editar o mestre muda todas as instâncias* —, e ela só é
//! afirmável contra **geometria**: os gates do modelo (`ph2d-ecs::vec_component`) provam que a
//! lista de overrides é canônica; estes provam o que aparece na tela.

use super::*;
use ph2d_ecs::{OverrideSlot, Transform, VecComponentMain, VecInstance};
use ph2d_vec_scene::rectangle;

/// Um mestre de 20×10 na origem + `n` instâncias, cada uma deslocada `100·k` em x.
///
/// O mestre é um retângulo **e um filho** (a "peça"), porque um componente de uma peça só não
/// exercita a sub-árvore — e a sub-árvore é metade do desenho da wave.
fn master_and_instances(
    n: usize,
) -> (
    SimWorld,
    VecScene,
    VecEntityMap,
    VecPathId,
    VecPathId,
    Vec<VecPathId>,
) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let main_id = scene.push_path(rectangle([0.0, 0.0], [20.0, 10.0]));
    let piece_id = scene.push_path(rectangle([2.0, 2.0], [8.0, 8.0]));
    let insts: Vec<VecPathId> = (0..n)
        .map(|_| scene.push_path(rectangle([0.0, 0.0], [20.0, 10.0])))
        .collect();
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let main_e = Entity::from_bits(map[&main_id]);
    // ⚠️ **O mestre não está na origem E tem ESCALA — as duas metades são a fixture a conter o
    // fenómeno.** O desenho de uma instância é `pose_da_cópia ∘ pose_do_mestre⁻¹`, e a mutação que
    // troca os dois termos precisa de uma composição NÃO-COMUTATIVA para se revelar: com o mestre
    // na identidade os dois produtos são iguais, e com uma TRANSLAÇÃO pura também (translações
    // comutam). A M3 sobreviveu às duas primeiras versões desta fixture exactamente assim.
    let mut mt = Transform::default();
    mt.translation.x = 40.0;
    mt.translation.y = 7.0;
    mt.scale.x = 2.0;
    sim.world_mut()
        .entity_mut(main_e)
        .insert((mt, VecComponentMain));
    let piece_e = Entity::from_bits(map[&piece_id]);
    sim.world_mut()
        .entity_mut(piece_e)
        .insert(ph2d_ecs::ChildOf(main_e));
    for (k, id) in insts.iter().enumerate() {
        let e = Entity::from_bits(map[id]);
        let mut t = Transform::default();
        t.translation.x = 100.0 * (k as f32 + 1.0);
        sim.world_mut()
            .entity_mut(e)
            .insert((t, VecInstance::new(main_id)));
    }
    (sim, scene, map, main_id, piece_id, insts)
}

/// A caixa de mundo que uma instância DESENHA.
fn drawn(live: &LiveGeometry, id: VecPathId) -> ([f64; 2], [f64; 2]) {
    let items = live.get(&id).expect("a instância desenha");
    let mut lo = [f64::MAX; 2];
    let mut hi = [f64::MIN; 2];
    for it in items {
        for v in it.verts_all() {
            for a in 0..2 {
                lo[a] = lo[a].min(v.anchor[a]);
                hi[a] = hi[a].max(v.anchor[a]);
            }
        }
    }
    (lo, hi)
}

/// ⚠️ Os xforms são **construídos**, não `default()`. A pose de uma forma vive no `Transform` da
/// entidade dela e quem a compõe é o `vec_transform::build` — a porta que o produto chama. Com um
/// mapa vazio toda pose é a identidade, e o gate da pose ficava a medir um mundo onde a pergunta
/// dele não existe (nasceu vermelho exactamente assim: `1ª cópia em x=100: [0.0, 0.0]`).
fn cook(sim: &SimWorld, scene: &VecScene, map: &VecEntityMap) -> InstanceLive {
    let xf = crate::vec_transform::build(sim, map);
    let mut il = InstanceLive::default();
    il.recook(scene, sim, map, &xf);
    il
}

fn approx(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-6
}

/// **A promessa da wave: editar o mestre muda TODAS as instâncias.**
///
/// ⚠️ O oráculo é a geometria DESENHADA, não o documento: o que o gate tem de provar é que a
/// mudança chega à tela sem ninguém chamar um passe de propagação — se ele lesse o `VecScene` das
/// instâncias, estaria a ler o suporte, que ninguém tocou, e ficaria verde com a wave inteira
/// desligada.
#[test]
fn editing_the_main_moves_every_instance() {
    let (mut sim, mut scene, map, main_id, _piece, insts) = master_and_instances(3);
    let before: Vec<_> = {
        let live = cook(&sim, &scene, &map);
        insts.iter().map(|i| drawn(live.live(), *i)).collect()
    };
    // ⚠️ O oráculo é a largura que o MESTRE mostra, nunca um literal: a v1 deste gate cravava a
    // largura LOCAL dele (20) e ficava verde só porque a pose do mestre era cancelada. Um literal
    // aqui codifica o modelo em vez de o medir.
    let master_w = |sim: &SimWorld, scene: &VecScene| {
        let xf = crate::vec_transform::build(sim, &map);
        let (lo, hi) = scene.path_curve_bbox(main_id).expect("bbox do mestre");
        let x = ph2d_vec_scene::xform_of(&xf, main_id);
        x.apply(hi)[0] - x.apply(lo)[0]
    };
    let w0 = master_w(&sim, &scene);
    for (k, b) in before.iter().enumerate() {
        assert!(
            approx(b.1[0] - b.0[0], w0),
            "instância {k} nasce com a largura do mestre ({w0:.1}): {b:?}"
        );
    }
    // O mestre engorda para 50 de largura — uma edição de DOCUMENTO, sem tocar nas instâncias.
    *scene.path_mut(main_id).expect("mestre") = {
        let mut p = rectangle([0.0, 0.0], [50.0, 10.0]);
        p.id = main_id;
        p
    };
    let _ = &mut sim;
    let w1 = master_w(&sim, &scene);
    assert!(w1 > w0, "a fixture não engordou o mestre");
    let live = cook(&sim, &scene, &map);
    for (k, id) in insts.iter().enumerate() {
        let (lo, hi) = drawn(live.live(), *id);
        assert!(
            approx(hi[0] - lo[0], w1),
            "instância {k} não seguiu o mestre (esperado {w1:.1}): {:?}",
            (lo, hi)
        );
    }
}

/// **A sub-árvore do mestre entra no desenho** — um componente de duas peças desenha duas.
#[test]
fn the_masters_subtree_is_what_the_instance_draws() {
    let (sim, scene, map, _main, _piece, insts) = master_and_instances(1);
    let live = cook(&sim, &scene, &map);
    let items = live.live().get(&insts[0]).expect("desenha");
    assert_eq!(items.len(), 2, "a raiz e a peça: {}", items.len());
}

/// **A CÓPIA empilha como o MESTRE: a peça-filha desenha SOBRE a raiz** (Enio, 2026-08-04: *"ao
/// criar a instância, os filhos que no mestre aparecem na frente dos pais vão para trás dos
/// pais"*).
///
/// ⚠️ **A instância não tem como herdar um remendo de renderer.** Enquanto a lei *"o filho desenha
/// sobre o pai"* era imposta pela ANTECIPAÇÃO do desenho no `ph2d_vec_render::dispatch`, ela valia
/// para a cena e não para a cópia: `cook_one` produz uma LISTA que o dispatch desenha em bloco, no
/// z da instância, e nada ali sabia demover a raiz. Invertida a projeção, a ordem certa chega aqui
/// **pela mesma varredura** — `subtree_paths` percorre `scene.paths()`, que é a pilha.
///
/// O oráculo é a ORDEM da lista desenhada, não a `subtree_paths`: é a lista que o renderer
/// consome, e uma segunda travessia entre as duas é exactamente onde a ordem se perderia.
#[test]
fn the_copy_stacks_like_the_master_so_the_child_piece_draws_last() {
    let (sim, scene, map, main, piece, insts) = master_and_instances(1);
    // A premissa: na CENA o mestre vem antes da peça-filha (a pilha põe o pai ao fundo).
    let order: Vec<VecPathId> = scene.paths().iter().map(|p| p.id).collect();
    let (pm, pp) = (
        order.iter().position(|i| *i == main).unwrap(),
        order.iter().position(|i| *i == piece).unwrap(),
    );
    assert!(pm < pp, "a fixture nao contem o fenomeno: {order:?}");

    let live = cook(&sim, &scene, &map);
    let pieces = live.pieces_of(insts[0]).expect("a copia desenha");
    assert_eq!(
        pieces,
        [main, piece],
        "a copia desenhou a raiz DEPOIS da peca — ela cobre o proprio conteudo"
    );
}

/// **A instância pousa na POSE dela** — o conteúdo do mestre viaja, a disposição interna fica.
#[test]
fn the_content_lands_on_the_instances_pose() {
    let (sim, scene, map, _main, _piece, insts) = master_and_instances(2);
    let live = cook(&sim, &scene, &map);
    let (lo0, _) = drawn(live.live(), insts[0]);
    let (lo1, _) = drawn(live.live(), insts[1]);
    assert!(approx(lo0[0], 100.0), "1ª cópia em x=100: {lo0:?}");
    assert!(approx(lo1[0], 200.0), "2ª cópia em x=200: {lo1:?}");
}

/// **Um override SOBREVIVE a editar o mestre** — é a metade que separa um prefab de uma cópia.
#[test]
fn an_override_survives_an_edit_of_the_main() {
    let (mut sim, mut scene, map, main_id, piece, insts) = master_and_instances(1);
    let e = Entity::from_bits(map[&insts[0]]);
    let mut inst = VecInstance::new(main_id);
    inst.set(piece, OverrideSlot::Fill([9, 8, 7, 255]));
    sim.world_mut().entity_mut(e).insert(inst);

    // A edição do mestre reescreve a geometria da peça POR INTEIRO.
    *scene.path_mut(piece).expect("peça") = {
        let mut p = rectangle([1.0, 1.0], [3.0, 3.0]);
        p.id = piece;
        p
    };
    let live = cook(&sim, &scene, &map);
    let items = live.live().get(&insts[0]).expect("desenha");
    let tinted = items.iter().any(|it| {
        matches!(
            it.fill,
            Some(ph2d_vec_scene::Paint::Solid(c)) if (c.r, c.g, c.b) == (9, 8, 7)
        )
    });
    assert!(tinted, "o override morreu com a edição do mestre");
}

/// **`Hidden` tira a peça do DESENHO** — e não a pinta transparente.
#[test]
fn a_hidden_piece_is_not_drawn_at_all() {
    let (mut sim, scene, map, main_id, piece, insts) = master_and_instances(1);
    let e = Entity::from_bits(map[&insts[0]]);
    let mut inst = VecInstance::new(main_id);
    inst.set(piece, OverrideSlot::Hidden);
    sim.world_mut().entity_mut(e).insert(inst);
    let live = cook(&sim, &scene, &map);
    assert_eq!(
        live.live().get(&insts[0]).expect("desenha").len(),
        1,
        "a peça escondida continua na lista"
    );
}

/// **Reset devolve a instância ao mestre, ao pixel.**
#[test]
fn reset_puts_the_instance_back_on_the_main() {
    let (mut sim, scene, map, main_id, piece, insts) = master_and_instances(1);
    let clean = {
        let live = cook(&sim, &scene, &map);
        live.live().get(&insts[0]).expect("desenha").clone()
    };
    let e = Entity::from_bits(map[&insts[0]]);
    let mut inst = VecInstance::new(main_id);
    inst.set(piece, OverrideSlot::Hidden);
    inst.set(piece, OverrideSlot::Fill([1, 1, 1, 1]));
    sim.world_mut().entity_mut(e).insert(inst.clone());
    inst.reset();
    sim.world_mut().entity_mut(e).insert(inst);
    let live = cook(&sim, &scene, &map);
    assert_eq!(live.live().get(&insts[0]).expect("desenha"), &clean);
}

/// **O mestre que SUMIU não faz a instância sumir** — ela fica sem entrada no mapa (o suporte
/// desenha) e é NOMEADA como órfã.
///
/// ⚠️ O gate afirma as DUAS metades. Só a ausência do mapa seria satisfeita por um produtor que
/// simplesmente não conhece a instância; é a lista de órfãs que prova que ele a viu e recusou.
#[test]
fn an_instance_whose_main_vanished_does_not_vanish_silently() {
    let (mut sim, scene, map, main_id, _piece, insts) = master_and_instances(1);
    let main_e = Entity::from_bits(map[&main_id]);
    // O artista revoga o componente (o gesto de Detach do mestre).
    sim.world_mut()
        .entity_mut(main_e)
        .remove::<VecComponentMain>();
    let live = cook(&sim, &scene, &map);
    assert!(
        live.live().get(&insts[0]).is_none(),
        "sem mestre, o mapa fica sem entrada e o suporte desenha"
    );
    assert_eq!(live.orphans(), &insts[..1], "a órfã tem de ser NOMEADA");
}

/// **Uma instância de si mesma é recusada** — o laço não tem resposta finita.
#[test]
fn an_instance_of_itself_is_refused() {
    let (mut sim, scene, map, _main, _piece, insts) = master_and_instances(1);
    let e = Entity::from_bits(map[&insts[0]]);
    sim.world_mut()
        .entity_mut(e)
        .insert(VecInstance::new(insts[0]));
    let live = cook(&sim, &scene, &map);
    assert!(live.live().get(&insts[0]).is_none());
    assert_eq!(live.orphans(), &insts[..1]);
}

/// **Uma cena sem instâncias fica byte-intocada** — o produtor não paga nada e não escreve nada.
#[test]
fn a_scene_with_no_instances_is_untouched() {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    scene.push_path(rectangle([0.0, 0.0], [10.0, 10.0]));
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let live = cook(&sim, &scene, &map);
    assert!(live.live().is_empty() && live.orphans().is_empty());
}

/// **1000 instâncias custam O(overrides), não O(1000 × árvore) no DOCUMENTO.**
///
/// ⚠️ O oráculo é o tamanho SERIALIZADO do que o documento guarda por instância — não o custo do
/// desenho, que é derivado e por definição proporcional ao que está na tela. É a tabela do Vol. 2
/// §3 aplicada a este modelo: a alternativa (cópia + merge) guardaria a árvore inteira mil vezes.
#[test]
fn a_thousand_instances_cost_their_overrides_and_not_a_thousand_trees() {
    let mut inst = VecInstance::new(1);
    let bare = postcard::to_allocvec(&inst).expect("serializa").len();
    inst.set(10, OverrideSlot::Fill([1, 2, 3, 4]));
    let one = postcard::to_allocvec(&inst).expect("serializa").len();
    // Uma instância limpa custa o id e um comprimento; um override custa uma dezena de bytes. O
    // mestre de duas peças, serializado, é ordens de grandeza maior — e é isso que a herança
    // prototipal poupa mil vezes.
    assert!(bare <= 16, "instância limpa custa {bare} bytes");
    assert!(one - bare <= 16, "um override custa {} bytes", one - bare);
}

/// **Redimensionar o mestre redimensiona as cópias — e MOVÊ-LO não as move.**
///
/// ⚠️ O gate da promessa da wave, e ele nasceu VERMELHO: a v1 cancelava a pose do mestre INTEIRA,
/// então escalar o mestre 3× deixava as duas peças de uma instância byte a byte onde estavam. O
/// relato do Enio (*"só os filhos redimensionaram"*) é essa assimetria vista da cadeira do artista
/// — uma edição num FILHO mexe no local dele e propaga; uma na RAIZ mexe na pose dela e evaporava.
///
/// ⚠️ **As duas metades são obrigatórias.** Sem a do MOVER, a cura óbvia (parar de cancelar a pose)
/// passa — e toda cópia nasceria em cima do mestre. É a metade que separa *forma* de *lugar*.
#[test]
fn resizing_the_master_resizes_the_copies_but_moving_it_does_not_move_them() {
    let (mut sim, scene, map, main_id, _piece, insts) = master_and_instances(1);
    let (lo0, hi0) = drawn(cook(&sim, &scene, &map).live(), insts[0]);
    let main_e = Entity::from_bits(map[&main_id]);

    if let Some(mut t) = sim.world_mut().get_mut::<Transform>(main_e) {
        t.scale.x *= 3.0;
        t.scale.y *= 3.0;
    }
    let (lo1, hi1) = drawn(cook(&sim, &scene, &map).live(), insts[0]);
    for a in 0..2 {
        let (w0, w1) = (hi0[a] - lo0[a], hi1[a] - lo1[a]);
        assert!(
            (w1 - w0 * 3.0).abs() < 1e-6,
            "eixo {a}: o mestre triplicou e a cópia foi de {w0:.3} para {w1:.3}"
        );
    }

    // O CONTROLE: mover o mestre não pode arrastar a cópia atrás dele.
    let centre_before = [(lo1[0] + hi1[0]) * 0.5, (lo1[1] + hi1[1]) * 0.5];
    if let Some(mut t) = sim.world_mut().get_mut::<Transform>(main_e) {
        t.translation.x += 250.0;
        t.translation.y -= 90.0;
    }
    let (lo2, hi2) = drawn(cook(&sim, &scene, &map).live(), insts[0]);
    let centre_after = [(lo2[0] + hi2[0]) * 0.5, (lo2[1] + hi2[1]) * 0.5];
    for a in 0..2 {
        assert!(
            (centre_after[a] - centre_before[a]).abs() < 1e-6,
            "eixo {a}: mover o mestre arrastou a cópia de {:.3} para {:.3}",
            centre_before[a],
            centre_after[a]
        );
    }
}

/// **Uma instância DENTRO de um mestre desenha o suporte dela, não a arte** — o limite do nesting.
///
/// ⚠️ Não é um defeito silencioso a corrigir de passagem: `cook_one` **não desce** a um segundo nível, e fazê-lo
/// é onde moram as perguntas que o ADR-0128 pagou caro (profundidade, ciclos, e a ordem
/// em que os deltas compõem). O gate existe para o limite ficar MEDIDO em vez de descoberto — e
/// para quem o levantar ter de o levantar de propósito.
#[test]
fn an_instance_nested_in_a_master_draws_its_support_not_its_art() {
    let (mut sim, mut scene, mut map, main_id, _piece, _insts) = master_and_instances(0);
    // Um segundo mestre, e uma instância DELE pendurada dentro do primeiro.
    let other = scene.push_path(rectangle([0.0, 0.0], [4.0, 4.0]));
    let nested = scene.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    sim.world_mut()
        .entity_mut(Entity::from_bits(map[&other]))
        .insert(VecComponentMain);
    sim.world_mut()
        .entity_mut(Entity::from_bits(map[&nested]))
        .insert((
            VecInstance::new(other),
            ph2d_ecs::ChildOf(Entity::from_bits(map[&main_id])),
        ));
    let outer = scene.push_path(rectangle([0.0, 0.0], [20.0, 10.0]));
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    sim.world_mut()
        .entity_mut(Entity::from_bits(map[&outer]))
        .insert(VecInstance::new(main_id));

    let live = cook(&sim, &scene, &map);
    let items = live
        .live()
        .get(&outer)
        .expect("a instância externa desenha");
    // Três peças: a raiz do mestre, o filho, e a instância aninhada — esta última pelo SUPORTE
    // (1×1), não pelos 4×4 do mestre dela.
    let widths: Vec<f64> = items
        .iter()
        .map(|p| {
            let (mut lo, mut hi) = (f64::MAX, f64::MIN);
            for v in p.verts_all() {
                lo = lo.min(v.anchor[0]);
                hi = hi.max(v.anchor[0]);
            }
            hi - lo
        })
        .collect();
    // A pose do mestre tem escala 2 em x, então o suporte 1×1 aparece com largura 2.
    assert!(
        widths.iter().any(|w| (w - 2.0).abs() < 1e-6),
        "a instância aninhada deixou de desenhar o suporte dela: {widths:?}"
    );
    assert!(
        !widths.iter().any(|w| (w - 8.0).abs() < 1e-6),
        "a instância aninhada passou a desenhar a arte do mestre dela — o nesting mudou de \
         comportamento e este gate é o sítio onde isso se decide: {widths:?}"
    );
}

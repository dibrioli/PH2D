//! Gates da costura da shell com o diagrama.
//!
//! ⚠️ O que estes gates defendem é que o diagrama e a ARTE contam a mesma história: a vista sai do
//! registo que o motor de facto usou, e não de uma segunda triagem que divergiria dele.

use super::*;
use ph2d_ecs::VecBoolEdges;
use ph2d_vec_render::LiveGeometry;
use ph2d_vec_scene::{VecScene, VecXforms, rectangle};

/// Três retângulos sobrepostos, agrupados, com a operação `op`. Devolve `(sim, scene, map, ids, g)`.
fn scene3(op: u8) -> (SimWorld, VecScene, VecEntityMap, Vec<VecPathId>, Entity) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let a = scene.push_path(rectangle([0.0, 0.0], [2.0, 2.0]));
    let b = scene.push_path(rectangle([1.0, 0.0], [3.0, 2.0]));
    let c = scene.push_path(rectangle([2.0, 0.0], [4.0, 2.0]));
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let g = Entity::from_bits(
        crate::vec_entities::group_entities(&mut sim, &[map[&a], map[&b], map[&c]], "Bool".into())
            .unwrap(),
    );
    sim.world_mut().entity_mut(g).insert(VecBoolGroup { op });
    (sim, scene, map, vec![a, b, c], g)
}

/// Roda o produtor uma vez — é ele que publica o registo que a vista consome.
fn cook(sim: &SimWorld, scene: &VecScene, map: &VecEntityMap) -> BoolLive {
    let mut bl = BoolLive::default();
    let mut live = LiveGeometry::new();
    bl.recook(scene, sim, map, &VecXforms::default(), &mut live);
    bl
}

/// **A VISTA SAI DO REGISTO DO MOTOR** — os mesmos ids, na mesma ordem de z.
#[test]
fn a_vista_mostra_exatamente_as_formas_que_o_motor_considerou() {
    let (sim, scene, map, ids, g) = scene3(0);
    let bl = cook(&sim, &scene, &map);
    let v = view_of(&sim, &map, &bl, g);
    assert_eq!(v.nodes.iter().map(|n| n.id).collect::<Vec<_>>(), ids);
}

/// **UM GRUPO QUE NÃO COZINHOU NÃO TEM DIAGRAMA** — e não um diagrama vazio com círculos
/// fantasma.
#[test]
fn sem_registo_a_vista_e_vazia() {
    let (sim, scene, map, _ids, g) = scene3(0);
    // Sem `recook`, não há registo.
    let bl = BoolLive::default();
    let v = view_of(&sim, &map, &bl, g);
    assert!(v.nodes.is_empty() && v.links.is_empty());
    let _ = (scene, map);
}

/// **O RÓTULO É O NOME DA HIERARQUIA**, que é como o artista distingue os círculos.
#[test]
fn o_rotulo_e_o_nome_que_o_artista_ve() {
    let (mut sim, scene, map, ids, g) = scene3(0);
    let e = Entity::from_bits(map[&ids[1]]);
    sim.world_mut().entity_mut(e).insert(Name("Estrela".into()));
    let bl = cook(&sim, &scene, &map);
    let v = view_of(&sim, &map, &bl, g);
    assert_eq!(v.nodes[1].label, "Estrela");
}

/// **A ESTRELA MATERIALIZA-SE UMA VEZ SÓ.** Um segundo `open` não pode re-semear: uma lista vazia é
/// um grafo deliberado (o artista cortou tudo), e re-semeá-la faria as formas voltarem a fundir-se
/// sozinhas — o oposto exato do que ele acabou de fazer.
#[test]
fn a_estrela_materializa_uma_vez_e_nao_re_semeia() {
    let (mut sim, scene, map, ids, g) = scene3(0);
    let bl = cook(&sim, &scene, &map);
    assert!(materialize_star(&mut sim, &bl, g), "a 1ª vez escreve");
    let n = sim.world().get::<VecBoolEdges>(g).unwrap().edges.len();
    assert_eq!(n, 2, "a estrela liga os dois não-base à base");

    // O artista corta tudo.
    sim.world_mut()
        .entity_mut(g)
        .insert(VecBoolEdges::default());
    assert!(
        !materialize_star(&mut sim, &bl, g),
        "re-semeou por cima de um grafo VAZIO deliberado"
    );
    assert!(sim.world().get::<VecBoolEdges>(g).unwrap().edges.is_empty());
    let _ = ids;
}

/// **UMA RECEITA DE PILHA NÃO TEM ESTRELA.** `Trim`/`Crop`/`Merge`/`MinusBack` são afirmações sobre
/// a pilha inteira; traduzi-las em pares mudaria o desenho no instante da abertura.
#[test]
fn uma_receita_de_pilha_nao_ganha_estrela() {
    for op in [4u8, 5, 6, 7] {
        let (mut sim, scene, map, _ids, g) = scene3(op);
        let bl = cook(&sim, &scene, &map);
        assert!(
            !materialize_star(&mut sim, &bl, g),
            "op {op} inventou estrela"
        );
        assert!(sim.world().get::<VecBoolEdges>(g).is_none());
    }
}

/// **LIGAR O MESMO PAR OUTRA VEZ SUBSTITUI** em vez de empilhar.
#[test]
fn ligar_o_mesmo_par_substitui() {
    let (mut sim, scene, map, ids, g) = scene3(0);
    let _ = cook(&sim, &scene, &map);
    assert!(
        apply_intents(
            &mut sim,
            g,
            &[BoolGraphIntent::Link {
                from: ids[1],
                to: ids[0],
                op: 0
            }]
        )
        .changed
    );
    assert!(
        apply_intents(
            &mut sim,
            g,
            &[BoolGraphIntent::Link {
                from: ids[1],
                to: ids[0],
                op: 1
            }]
        )
        .changed
    );
    let e = sim.world().get::<VecBoolEdges>(g).unwrap();
    assert_eq!(e.edges.len(), 1, "empilhou uma segunda ligação invisível");
    assert_eq!(e.get(ids[1], ids[0]), Some(1));
}

/// **UMA INTENÇÃO QUE NÃO MUDA NADA NÃO SUJA O UNDO.**
///
/// ⚠️ O passo de undo é registado por diff; escrever o componente com o mesmo conteúdo criaria um
/// passo que não mudou um pixel, e desfazer pareceria não fazer nada.
#[test]
fn uma_intencao_sem_efeito_nao_escreve() {
    let (mut sim, scene, map, ids, g) = scene3(0);
    let _ = cook(&sim, &scene, &map);
    let liga = [BoolGraphIntent::Link {
        from: ids[1],
        to: ids[0],
        op: 0,
    }];
    assert!(apply_intents(&mut sim, g, &liga).changed);
    assert!(
        !apply_intents(&mut sim, g, &liga).changed,
        "reescreveu o mesmo estado"
    );
    // E cortar uma ligação que não existe também não escreve.
    assert!(
        !apply_intents(
            &mut sim,
            g,
            &[BoolGraphIntent::Unlink {
                from: ids[2],
                to: ids[0]
            }]
        )
        .changed
    );
}

/// **O CICLO É VISTO PELA MESMA CAMINHADA DO MOTOR** — e a vista di-lo.
#[test]
fn a_vista_ve_o_ciclo() {
    let (mut sim, scene, map, ids, g) = scene3(0);
    let bl = cook(&sim, &scene, &map);
    assert!(!view_of(&sim, &map, &bl, g).cycle, "não havia ciclo");
    apply_intents(
        &mut sim,
        g,
        &[
            BoolGraphIntent::Link {
                from: ids[0],
                to: ids[1],
                op: 0,
            },
            BoolGraphIntent::Link {
                from: ids[1],
                to: ids[0],
                op: 0,
            },
        ],
    );
    assert!(
        view_of(&sim, &map, &bl, g).cycle,
        "o ciclo passou despercebido"
    );
}

/// **CONSUMIDO = TEM LIGAÇÃO DE SAÍDA** — o mesmo predicado do resolvedor, e ele continua correto
/// na RECUSA (onde não há plano nenhum a consultar).
#[test]
fn consumido_e_quem_opera_sobre_alguem() {
    let (mut sim, scene, map, ids, g) = scene3(0);
    let bl = cook(&sim, &scene, &map);
    apply_intents(
        &mut sim,
        g,
        &[BoolGraphIntent::Link {
            from: ids[1],
            to: ids[0],
            op: 0,
        }],
    );
    let v = view_of(&sim, &map, &bl, g);
    assert!(!v.nodes[0].consumed, "o receptor não é consumido");
    assert!(v.nodes[1].consumed, "quem opera É consumido");
    assert!(!v.nodes[2].consumed, "o solto não é consumido");
}

/// **O CICLO DOS QUATRO VERBOS FECHA, E NUNCA SAI DELES.**
///
/// ⚠️ A segunda metade é a lei: um código inválido cai em `Union` em vez de avançar, senão um
/// clique deixaria o artista a rodar entre códigos que não desenham nada.
#[test]
fn o_ciclo_de_operacoes_fecha_nos_quatro() {
    assert_eq!([0, 1, 2, 3].map(next_op), [1, 2, 3, 0]);
    for invalido in [4u8, 7, 200] {
        assert_eq!(
            next_op(invalido),
            0,
            "o código {invalido} continuou inválido"
        );
    }
}

/// **A OPERAÇÃO UNIFORME É `None` QUANDO AS LIGAÇÕES DISCORDAM** — e também sem ligação nenhuma.
#[test]
fn a_operacao_uniforme_diz_misto_quando_ha_desacordo() {
    let (mut sim, scene, map, ids, g) = scene3(0);
    let _ = cook(&sim, &scene, &map);
    assert_eq!(uniform_op(&sim, g), None, "sem componente não há operação");
    apply_intents(
        &mut sim,
        g,
        &[
            BoolGraphIntent::Link {
                from: ids[1],
                to: ids[0],
                op: 2,
            },
            BoolGraphIntent::Link {
                from: ids[2],
                to: ids[0],
                op: 2,
            },
        ],
    );
    assert_eq!(uniform_op(&sim, g), Some(2));
    apply_intents(
        &mut sim,
        g,
        &[BoolGraphIntent::Link {
            from: ids[2],
            to: ids[0],
            op: 1,
        }],
    );
    assert_eq!(
        uniform_op(&sim, g),
        None,
        "duas operações e mesmo assim uniforme"
    );
}

/// **OS OITO BOTÕES NÃO PODEM FICAR MORTOS SOBRE UM DIAGRAMA.**
///
/// ⚠️ É o defeito *"parâmetro que não muda NADA"* na sua forma mais pura: com um grafo presente
/// quem manda é a operação de cada LIGAÇÃO, então mexer só no `VecBoolGroup` deixaria o artista a
/// clicar *Subtract* e a ver a arte não mudar. Uma das quatro de conjunto reescreve TODAS.
#[test]
fn um_verbo_de_conjunto_reescreve_todas_as_ligacoes() {
    let (mut sim, scene, map, ids, g) = scene3(0);
    let bl = cook(&sim, &scene, &map);
    materialize_star(&mut sim, &bl, g);
    assert_eq!(uniform_op(&sim, g), Some(0), "a estrela nasce uniforme");

    assert!(retarget_graph(&mut sim, g, 1), "Subtract não escreveu");
    assert_eq!(uniform_op(&sim, g), Some(1));
    // E re-clicar o MESMO verbo não escreve — o undo é por diff, e um passo que não muda nada faz
    // o Ctrl+Z parecer avariado.
    assert!(!retarget_graph(&mut sim, g, 1));
    let _ = ids;
}

/// **UMA RECEITA DE PILHA REMOVE O DIAGRAMA** — e não é ignorada em silêncio.
///
/// ⚠️ `Trim`/`Crop`/`Merge`/`MinusBack` são afirmações sobre a pilha inteira e não têm tradução em
/// pares. Ignorar o clique deixaria um botão que não faz nada; removê-lo é destrutivo para o
/// diagrama e é a leitura honesta das duas.
#[test]
fn uma_receita_de_pilha_remove_o_diagrama() {
    for op in [4u8, 5, 6, 7] {
        let (mut sim, scene, map, _ids, g) = scene3(0);
        let bl = cook(&sim, &scene, &map);
        materialize_star(&mut sim, &bl, g);
        assert!(sim.world().get::<VecBoolEdges>(g).is_some());
        assert!(retarget_graph(&mut sim, g, op), "op {op} não agiu");
        assert!(
            sim.world().get::<VecBoolEdges>(g).is_none(),
            "op {op} deixou o grafo a sobrepor-se à receita"
        );
    }
}

/// **SEM DIAGRAMA, RE-MIRAR NÃO INVENTA UM.** O caminho de sempre fica byte-intocado.
#[test]
fn re_mirar_um_grupo_sem_diagrama_nao_cria_um() {
    let (mut sim, scene, map, _ids, g) = scene3(0);
    let _ = cook(&sim, &scene, &map);
    assert!(!retarget_graph(&mut sim, g, 2));
    assert!(sim.world().get::<VecBoolEdges>(g).is_none());
}

/// **APAGAR UMA FORMA LEVA AS LIGAÇÕES DELA** — e a varredura só escreve quando de facto apaga.
///
/// ⚠️ A segunda metade é a que protege o undo: ele é registado por diff de bytes, e uma varredura
/// que reescrevesse o componente todo frame criaria um passo por frame — desfazer pareceria não
/// fazer nada.
#[test]
fn a_varredura_corta_as_orfas_e_so_escreve_quando_corta() {
    let (mut sim, mut scene, map, ids, g) = scene3(0);
    let bl = cook(&sim, &scene, &map);
    materialize_star(&mut sim, &bl, g);
    assert_eq!(sim.world().get::<VecBoolEdges>(g).unwrap().edges.len(), 2);

    // Nada morreu ainda: a varredura não pode escrever.
    assert!(
        !prune_dead_edges(&mut sim, &scene),
        "escreveu sem nada a apagar"
    );

    scene.remove_path(ids[1]);
    assert!(prune_dead_edges(&mut sim, &scene), "não cortou a órfã");
    let e = sim.world().get::<VecBoolEdges>(g).unwrap();
    assert_eq!(e.edges.len(), 1, "sobrou a ligação da forma apagada");
    assert!(e.edges.iter().all(|l| l.from != ids[1] && l.to != ids[1]));
    // E uma segunda varredura já não escreve.
    assert!(!prune_dead_edges(&mut sim, &scene));
}

/// **A VISTA LÊ A POSIÇÃO GUARDADA** — é o arrasto do artista a sobreviver.
#[test]
fn a_vista_le_a_posicao_guardada() {
    let (mut sim, scene, map, ids, g) = scene3(0);
    let bl = cook(&sim, &scene, &map);
    assert!(
        view_of(&sim, &map, &bl, g)
            .nodes
            .iter()
            .all(|n| n.at.is_none()),
        "uma forma nunca arrastada tem de cair no anel default"
    );
    let mut pos = ph2d_ecs::VecBoolGraphPos::default();
    pos.set(ids[1], [123.0, 45.0]);
    sim.world_mut().entity_mut(g).insert(pos);
    let v = view_of(&sim, &map, &bl, g);
    assert_eq!(v.nodes[1].at, Some([123.0, 45.0]));
    assert_eq!(v.nodes[0].at, None, "as outras continuam no anel");
}

/// **MOVER ESCREVE A POSIÇÃO — E SÓ QUANDO ELA MUDA.**
///
/// ⚠️ A segunda metade protege o undo: ele é por diff de bytes, e reescrever a mesma posição
/// criaria um passo que não mexeu em nada.
#[test]
fn mover_escreve_a_posicao_e_so_quando_ela_muda() {
    let (mut sim, scene, map, ids, g) = scene3(0);
    let _ = cook(&sim, &scene, &map);
    let mover = [BoolGraphIntent::Move {
        id: ids[0],
        at: [80.0, 90.0],
    }];
    assert!(apply_intents(&mut sim, g, &mover).changed);
    assert_eq!(
        sim.world()
            .get::<ph2d_ecs::VecBoolGraphPos>(g)
            .unwrap()
            .get(ids[0]),
        Some([80.0, 90.0])
    );
    assert!(
        !apply_intents(&mut sim, g, &mover).changed,
        "reescreveu a mesma posição"
    );
}

/// **SELECIONAR NÃO MUDA O DOCUMENTO.**
///
/// ⚠️ Se contasse como mudança, cada clique num círculo viraria um passo de undo que não mexeu em
/// nada — e desfazer um gesto de verdade exigiria passar por todos eles.
#[test]
fn selecionar_nao_muda_o_documento() {
    let (mut sim, scene, map, ids, g) = scene3(0);
    let _ = cook(&sim, &scene, &map);
    let out = apply_intents(&mut sim, g, &[BoolGraphIntent::Select { id: ids[2] }]);
    assert!(!out.changed, "selecionar sujou o undo");
    assert_eq!(out.select, Some(ids[2]), "a seleção não chegou à shell");
}

/// **APAGAR UMA FORMA LEVA A POSIÇÃO DELA** — a outra metade da varredura.
#[test]
fn a_varredura_corta_tambem_as_posicoes_orfas() {
    let (mut sim, mut scene, map, ids, g) = scene3(0);
    let _ = cook(&sim, &scene, &map);
    apply_intents(
        &mut sim,
        g,
        &[
            BoolGraphIntent::Move {
                id: ids[0],
                at: [10.0, 10.0],
            },
            BoolGraphIntent::Move {
                id: ids[1],
                at: [20.0, 20.0],
            },
        ],
    );
    assert!(
        !prune_dead_edges(&mut sim, &scene),
        "escreveu sem nada a apagar"
    );

    scene.remove_path(ids[1]);
    assert!(
        prune_dead_edges(&mut sim, &scene),
        "não cortou a posição órfã"
    );
    let p = sim.world().get::<ph2d_ecs::VecBoolGraphPos>(g).unwrap();
    assert_eq!(p.get(ids[1]), None, "sobrou a posição da forma apagada");
    assert_eq!(p.get(ids[0]), Some([10.0, 10.0]), "levou a errada junto");
}

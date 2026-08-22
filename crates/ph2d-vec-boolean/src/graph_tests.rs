//! Gates do **grafo** da booleana viva.
//!
//! ⚠️ O gate que sustenta a feature inteira é o primeiro: `a_estrela_derivada_desenha_o_que_o_grupo_de_hoje_desenha`.
//! Ele compara a geometria **inteira** com `assert_eq!` (e não a área) de propósito — a promessa
//! não é *"dá o mesmo tamanho"*, é *"não move um pixel"*, e área é a medida que deixa passar uma
//! forma trocada de sítio.

use super::{BoolEdge, GraphRefusal, derive_star, resolve_graph};
use crate::{PathfinderOp, area, pathfinder};
use kurbo::Shape;
use ph2d_vec_scene::{FillRule, Paint, Rgba8, VecPath, VecVertex};

/// Um quadrado `[x, x+s] × [y, y+s]`, com `fill` distinto para o estilo ser rastreável.
fn square(x: f64, y: f64, s: f64, fill: u8) -> VecPath {
    VecPath {
        verts: [[x, y], [x + s, y], [x + s, y + s], [x, y + s]]
            .into_iter()
            .map(VecVertex::corner)
            .collect(),
        closed: true,
        fill: Some(Paint::solid(Rgba8::new(fill, fill, fill, 255))),
        ..VecPath::default()
    }
}

/// Três quadrados de lado 4 em ESCADA: cada um cobre metade do anterior; o 1º e o 3º não se tocam.
fn staircase() -> Vec<(u64, Vec<VecPath>)> {
    vec![
        (1, vec![square(0.0, 0.0, 4.0, 10)]),
        (2, vec![square(2.0, 0.0, 4.0, 20)]),
        (3, vec![square(4.0, 0.0, 4.0, 30)]),
    ]
}

fn ids(nodes: &[(u64, Vec<VecPath>)]) -> Vec<u64> {
    nodes.iter().map(|(id, _)| *id).collect()
}

/// O que o nó `id` desenhou.
fn drawn(out: &[(u64, Vec<VecPath>)], id: u64) -> &[VecPath] {
    &out.iter()
        .find(|(n, _)| *n == id)
        .expect("o nó está na saída")
        .1
}

/// **A ESTRELA DERIVADA DESENHA O QUE O GRUPO DE HOJE DESENHA.**
///
/// É a promessa que torna a etapa 2 possível: materializar o grafo sobre um grupo existente e
/// resolver por `resolve_graph` tem de dar, byte a byte, o que o `pathfinder` N-ário dava. Sem
/// isto, abrir a janela do diagrama moveria a arte no instante em que o artista olhasse para ela.
///
/// ⚠️ Vale para as **quatro operações de conjunto**, e o laço passa pelas quatro de propósito:
/// `Union`/`Intersect`/`Exclude` são associativas e perdoariam um fold escrito ao contrário — só o
/// `Subtract` reprova, e um gate que testasse uma delas seria verde por acidente.
#[test]
fn a_estrela_derivada_desenha_o_que_o_grupo_de_hoje_desenha() {
    let nodes = staircase();
    let flat: Vec<&VecPath> = nodes.iter().map(|(_, v)| &v[0]).collect();
    for op in [
        PathfinderOp::Union,
        PathfinderOp::Subtract,
        PathfinderOp::Intersect,
        PathfinderOp::Exclude,
    ] {
        let hoje = pathfinder(&flat, op).expect("o motor de hoje aceita a escada");
        let edges = derive_star(&ids(&nodes), op);
        let out = resolve_graph(&nodes, &edges).expect("o grafo aceita a escada");
        // A base (o mais ao fundo) carrega o resultado; os demais desenham nada.
        assert_eq!(
            drawn(&out, 1),
            hoje.as_slice(),
            "{op:?}: a estrela divergiu do grupo"
        );
        assert!(
            drawn(&out, 2).is_empty(),
            "{op:?}: o operando 2 devia desenhar nada"
        );
        assert!(
            drawn(&out, 3).is_empty(),
            "{op:?}: o operando 3 devia desenhar nada"
        );
    }
}

/// **A ESTRELA É A MESMA COM A LISTA EMBARALHADA.** A ordem em que as ligações estão guardadas é
/// cosmética: quem manda é o z de quem opera.
///
/// ⚠️ Sem esta lei, a lista de ligações no disco vira estado com significado — e reordená-la (um
/// merge, uma edição do painel, um undo) mudaria o desenho sem que nada na tela o explicasse.
#[test]
fn a_ordem_guardada_das_ligacoes_nao_muda_o_desenho() {
    let nodes = staircase();
    let mut edges = derive_star(&ids(&nodes), PathfinderOp::Subtract);
    let reto = resolve_graph(&nodes, &edges).expect("motor ok");
    edges.reverse();
    let avesso = resolve_graph(&nodes, &edges).expect("motor ok");
    assert_eq!(reto, avesso, "a ordem guardada mudou o desenho");
}

/// **A SETA MANDA, E ELA NÃO É SIMÉTRICA.** `A−B` e `B−A` são desenhos diferentes, e é a direção
/// que os separa — a razão de existir da seta.
///
/// Na escada, A=`[0,4]` e B=`[2,6]` sobrepõem-se em `2×4 = 8`. `A−B` guarda `[0,2]` (área 8, à
/// ESQUERDA); `B−A` guarda `[4,6]` (área 8 também, à DIREITA). ⚠️ As áreas são IGUAIS de
/// propósito: um gate que medisse só área passaria com a seta invertida.
#[test]
fn a_seta_decide_quem_sobra_no_subtract() {
    let nodes = vec![
        (1, vec![square(0.0, 0.0, 4.0, 10)]),
        (2, vec![square(2.0, 0.0, 4.0, 20)]),
    ];
    let a_menos_b = resolve_graph(
        &nodes,
        &[BoolEdge {
            from: 2,
            to: 1,
            op: PathfinderOp::Subtract,
        }],
    )
    .expect("motor ok");
    let b_menos_a = resolve_graph(
        &nodes,
        &[BoolEdge {
            from: 1,
            to: 2,
            op: PathfinderOp::Subtract,
        }],
    )
    .expect("motor ok");

    let esq = crate::to_bez(&drawn(&a_menos_b, 1)[0]).bounding_box();
    assert!(
        (esq.min_x() - 0.0).abs() < 1e-9 && (esq.max_x() - 2.0).abs() < 1e-9,
        "A−B devia guardar x ∈ [0,2], guardou [{}, {}]",
        esq.min_x(),
        esq.max_x()
    );
    let dir = crate::to_bez(&drawn(&b_menos_a, 2)[0]).bounding_box();
    assert!(
        (dir.min_x() - 4.0).abs() < 1e-9 && (dir.max_x() - 6.0).abs() < 1e-9,
        "B−A devia guardar x ∈ [4,6], guardou [{}, {}]",
        dir.min_x(),
        dir.max_x()
    );
    // E o RECEPTOR troca com a seta: quem recebe é quem desenha.
    assert!(drawn(&a_menos_b, 2).is_empty(), "A−B: o 2 foi consumido");
    assert!(drawn(&b_menos_a, 1).is_empty(), "B−A: o 1 foi consumido");
}

/// **A MESMA FORMA SOMA COM UMA E SUBTRAI DE OUTRA** — o pedido do Enio, na sua forma mínima.
///
/// `2` opera em duas direções ao mesmo tempo: soma-se a `1` e é subtraída de `3`. Isso é
/// inexprimível numa operação-de-grupo, e é o que o grafo acrescenta.
///
/// Geometria: quadrados de lado 4 em `x = 0 / 2 / 4`.
/// - `1 ∪ 2` = `[0,6] × [0,4]` ⇒ área 24.
/// - `3 − 2` = `[6,8] × [0,4]` ⇒ área 8.
#[test]
fn uma_forma_soma_com_uma_vizinha_e_subtrai_de_outra() {
    let nodes = staircase();
    let out = resolve_graph(
        &nodes,
        &[
            BoolEdge {
                from: 2,
                to: 1,
                op: PathfinderOp::Union,
            },
            BoolEdge {
                from: 2,
                to: 3,
                op: PathfinderOp::Subtract,
            },
        ],
    )
    .expect("motor ok");

    let uniao = drawn(&out, 1);
    assert_eq!(uniao.len(), 1, "a união é uma peça");
    let a = area(&uniao[0]);
    assert!((a - 24.0).abs() < 1e-6, "área da união {a}, esperada 24");

    let resto = drawn(&out, 3);
    assert_eq!(resto.len(), 1, "o resto é uma peça");
    let b = area(&resto[0]);
    assert!((b - 8.0).abs() < 1e-6, "área do resto {b}, esperada 8");

    assert!(
        drawn(&out, 2).is_empty(),
        "o 2 foi consumido pelas duas ligações"
    );
}

/// **UMA CADEIA COMPÕE**: `3` opera sobre `2`, e o `2` JÁ OPERADO opera sobre `1`.
///
/// ⚠️ É o gate que distingue *"o `from` entra resolvido"* de *"o `from` entra cru"* — os dois
/// desenham a mesma coisa em muitas cenas, e aqui não: `1 ∪ (2 ∪ 3)` cobre `[0,8]` (área 32),
/// enquanto `1 ∪ 2` cru daria `[0,6]` (24).
#[test]
fn uma_cadeia_entra_ja_resolvida_no_proximo_no() {
    let nodes = staircase();
    let out = resolve_graph(
        &nodes,
        &[
            BoolEdge {
                from: 3,
                to: 2,
                op: PathfinderOp::Union,
            },
            BoolEdge {
                from: 2,
                to: 1,
                op: PathfinderOp::Union,
            },
        ],
    )
    .expect("motor ok");
    let v = drawn(&out, 1);
    assert_eq!(v.len(), 1, "uma peça");
    let a = area(&v[0]);
    assert!(
        (a - 32.0).abs() < 1e-6,
        "área {a}, esperada 32 -- o 2 entrou CRU"
    );
}

/// **DUAS LIGAÇÕES A CHEGAR DOBRAM NA ORDEM DE Z DE QUEM OPERA**, não na ordem em que foram
/// escritas — e o oráculo é o ESTILO, que o último dobrado doa.
///
/// A escada com `2` e `3` a subtraírem de `1`: o último a dobrar é o de z mais alto (`3`, fill 30),
/// e é a roupa dele que o resultado veste. É a lei do `apply_many`, e é o que faz a estrela
/// derivada vestir o mesmo que o grupo de hoje.
#[test]
fn o_ultimo_dobrado_por_z_doa_o_estilo() {
    let nodes = staircase();
    // Escritas ao contrário do z de propósito.
    let out = resolve_graph(
        &nodes,
        &[
            BoolEdge {
                from: 3,
                to: 1,
                op: PathfinderOp::Subtract,
            },
            BoolEdge {
                from: 2,
                to: 1,
                op: PathfinderOp::Subtract,
            },
        ],
    )
    .expect("motor ok");
    let v = drawn(&out, 1);
    assert_eq!(v.len(), 1, "uma peça");
    let esperado = Some(Paint::solid(Rgba8::new(30, 30, 30, 255)));
    assert_eq!(v[0].fill, esperado, "o estilo não veio do operando do TOPO");
}

/// **CICLO É RECUSA INTEIRA.** E o laço de um nó consigo mesmo também.
#[test]
fn um_ciclo_recusa_o_grafo_todo() {
    let nodes = staircase();
    let ciclo = resolve_graph(
        &nodes,
        &[
            BoolEdge {
                from: 1,
                to: 2,
                op: PathfinderOp::Union,
            },
            BoolEdge {
                from: 2,
                to: 3,
                op: PathfinderOp::Union,
            },
            BoolEdge {
                from: 3,
                to: 1,
                op: PathfinderOp::Union,
            },
        ],
    );
    assert_eq!(ciclo, Err(GraphRefusal::Cycle), "o ciclo de 3 passou");
    let laco = resolve_graph(
        &nodes,
        &[BoolEdge {
            from: 1,
            to: 1,
            op: PathfinderOp::Union,
        }],
    );
    assert_eq!(laco, Err(GraphRefusal::Cycle), "o laço de 1 passou");
}

/// **UM CICLO NUM CANTO RECUSA O GRAFO INTEIRO** — e não só o canto.
///
/// ⚠️ Este é o gate que a contagem do Kahn poderia deixar passar se a recusa fosse local: `1` não
/// está no ciclo e resolveria sozinho, e desenhar só ele mostraria uma arte que nenhuma leitura do
/// diagrama explica. A resposta do grafo é uma só.
#[test]
fn um_ciclo_num_canto_recusa_o_grafo_inteiro() {
    let mut nodes = staircase();
    nodes.push((4, vec![square(20.0, 0.0, 4.0, 40)]));
    let out = resolve_graph(
        &nodes,
        &[
            BoolEdge {
                from: 3,
                to: 4,
                op: PathfinderOp::Union,
            },
            BoolEdge {
                from: 4,
                to: 3,
                op: PathfinderOp::Union,
            },
        ],
    );
    assert_eq!(out, Err(GraphRefusal::Cycle));
}

/// **AS QUATRO RECEITAS NÃO CABEM NUMA LIGAÇÃO** — e a recusa nomeia qual.
///
/// `Trim` é *"cada forma menos a união do que está ACIMA dela"*: uma afirmação sobre a pilha
/// inteira, não uma relação entre dois. Deixá-la passar seria prometer, numa seta, o que o modelo
/// não entrega.
#[test]
fn uma_receita_de_pilha_nao_e_uma_ligacao() {
    let nodes = staircase();
    for op in [
        PathfinderOp::MinusBack,
        PathfinderOp::Trim,
        PathfinderOp::Crop,
        PathfinderOp::Merge,
    ] {
        let out = resolve_graph(&nodes, &[BoolEdge { from: 2, to: 1, op }]);
        assert_eq!(
            out,
            Err(GraphRefusal::NotBinary(op)),
            "{op:?} passou como ligação"
        );
    }
}

/// **UMA LIGAÇÃO ÓRFÃ RECUSA** em vez de responder com um operando a menos.
#[test]
fn uma_ligacao_que_nomeia_forma_ausente_recusa() {
    let nodes = staircase();
    let out = resolve_graph(
        &nodes,
        &[BoolEdge {
            from: 99,
            to: 1,
            op: PathfinderOp::Union,
        }],
    );
    assert_eq!(out, Err(GraphRefusal::UnknownNode(99)));
}

/// **SEM LIGAÇÕES, CADA FORMA DESENHA-SE A SI PRÓPRIA — VERBATIM.**
///
/// ⚠️ Verbatim é a palavra que importa: passar pelo motor um caminho que ninguém operou trocaria a
/// geometria autorada por uma varredura dela (contornos reorientados, quinas assadas). Um grafo
/// vazio tem de ser um NO-OP exato, senão criar o grupo já mexeria na arte.
#[test]
fn sem_ligacoes_a_forma_desenha_se_a_si_propria_verbatim() {
    let nodes = staircase();
    let out = resolve_graph(&nodes, &[]).expect("motor ok");
    for (id, paths) in &nodes {
        assert_eq!(
            drawn(&out, *id),
            paths.as_slice(),
            "o nó {id} não saiu verbatim"
        );
    }
}

/// **AS GEOMETRIAS DE UM NÓ SÃO UMA FORMA SÓ** — a divergência declarada no topo do módulo.
///
/// Um nó com duas peças disjuntas (o que um pattern ou um offset vivo produzem), a subtrair de
/// outro. Se as peças se dobrassem entre si com a operação da LIGAÇÃO — como o `apply_many` faz —,
/// a segunda peça seria subtraída da primeira antes de a ligação acontecer, e o operando chegaria
/// mutilado.
///
/// Geometria: o nó `2` são dois quadrados de lado 2 disjuntos, em `[2,4]` e `[6,8]` (área 4 cada);
/// o nó `1` é `[0,10] × [0,4]` (área 40). `1 − 2` = `40 − 8 = 32`.
///
/// ⚠️ A resposta ERRADA é **36**, e é ela que dá o gate: com as peças dobradas entre si por
/// `Subtract`, a segunda apagaria a primeira — só que são disjuntas, então `p1 − p2 = p1`, e o
/// operando chegaria com metade da área. Um oráculo escolhido ao acaso não separaria os dois
/// números.
#[test]
fn as_geometrias_de_um_no_sao_uma_forma_so() {
    let nodes = vec![
        (
            1,
            vec![VecPath {
                verts: [[0.0, 0.0], [10.0, 0.0], [10.0, 4.0], [0.0, 4.0]]
                    .into_iter()
                    .map(VecVertex::corner)
                    .collect(),
                closed: true,
                fill: Some(Paint::solid(Rgba8::new(10, 10, 10, 255))),
                ..VecPath::default()
            }],
        ),
        (
            2,
            vec![square(2.0, 0.0, 2.0, 20), square(6.0, 0.0, 2.0, 20)],
        ),
    ];
    let out = resolve_graph(
        &nodes,
        &[BoolEdge {
            from: 2,
            to: 1,
            op: PathfinderOp::Subtract,
        }],
    )
    .expect("motor ok");
    let total: f64 = drawn(&out, 1).iter().map(area).sum();
    assert!(
        (total - 32.0).abs() < 1e-6,
        "área {total}, esperada 32 (36 = as peças do nó 2 dobraram-se entre si)"
    );
}

/// **UM COMPOUND SOBREVIVE A UMA LIGAÇÃO** — a regra `EvenOdd` do operando é lida na entrada.
///
/// Uma rosquinha (`[0,10]` com buraco `[3,7]`, área `100 − 16 = 84`) unida a um quadradinho que
/// tapa parte do buraco (`[3,5] × [3,5]`, área 4) ⇒ `84 + 4 = 88`. Se a regra do compound se
/// perdesse, o buraco fecharia e a área saltaria para 100.
#[test]
fn um_compound_entra_com_a_regra_dele() {
    let anel = VecPath {
        verts: [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]]
            .into_iter()
            .map(VecVertex::corner)
            .collect(),
        closed: true,
        subpaths: vec![ph2d_vec_scene::Contour::new_closed(
            [[3.0, 3.0], [7.0, 3.0], [7.0, 7.0], [3.0, 7.0]]
                .into_iter()
                .map(VecVertex::corner)
                .collect(),
        )],
        fill_rule: FillRule::EvenOdd,
        fill: Some(Paint::solid(Rgba8::new(10, 10, 10, 255))),
        ..VecPath::default()
    };
    let nodes = vec![(1, vec![anel]), (2, vec![square(3.0, 3.0, 2.0, 20)])];
    let out = resolve_graph(
        &nodes,
        &[BoolEdge {
            from: 2,
            to: 1,
            op: PathfinderOp::Union,
        }],
    )
    .expect("motor ok");
    let total: f64 = drawn(&out, 1).iter().map(area).sum();
    assert!(
        (total - 88.0).abs() < 1e-6,
        "área {total}, esperada 88 -- a regra do compound perdeu-se na entrada"
    );
}

/// **DOIS SUMIDOUROS DESENHAM CADA UM NO SEU ID.** É a forma que a booleana de hoje não tem: um
/// grupo produz UM resultado, e o grafo produz um por sumidouro.
#[test]
fn cada_sumidouro_desenha_no_proprio_id() {
    let mut nodes = staircase();
    nodes.push((4, vec![square(20.0, 0.0, 4.0, 40)]));
    nodes.push((5, vec![square(22.0, 0.0, 4.0, 50)]));
    let out = resolve_graph(
        &nodes,
        &[
            BoolEdge {
                from: 2,
                to: 1,
                op: PathfinderOp::Union,
            },
            BoolEdge {
                from: 5,
                to: 4,
                op: PathfinderOp::Union,
            },
        ],
    )
    .expect("motor ok");
    assert_eq!(drawn(&out, 1).len(), 1, "o sumidouro 1 desenha");
    assert_eq!(drawn(&out, 4).len(), 1, "o sumidouro 4 desenha");
    assert_eq!(
        drawn(&out, 3).len(),
        1,
        "o 3, solto, desenha-se a si próprio"
    );
    assert!(drawn(&out, 2).is_empty());
    assert!(drawn(&out, 5).is_empty());
}

/// **A DETEÇÃO SEM COZINHAR CONCORDA COM A RECUSA DO RESOLVEDOR** — para os dois vereditos.
///
/// ⚠️ É o gate que impede as duas respostas de divergirem. Elas são a MESMA caminhada de propósito,
/// e este teste é o que torna essa afirmação verificável em vez de um comentário: uma segunda
/// deteção escrita à parte concordaria em quase todo grafo e discordaria nos casos raros, que é
/// exatamente onde ninguém olha.
#[test]
fn a_detecao_de_ciclo_concorda_com_a_recusa_do_resolvedor() {
    let nodes = staircase();
    let ids = ids(&nodes);
    let casos: Vec<Vec<BoolEdge>> = vec![
        // Aciclico: a estrela.
        derive_star(&ids, PathfinderOp::Union),
        // Cadeia.
        vec![
            BoolEdge {
                from: 3,
                to: 2,
                op: PathfinderOp::Union,
            },
            BoolEdge {
                from: 2,
                to: 1,
                op: PathfinderOp::Union,
            },
        ],
        // Ciclo de 3.
        vec![
            BoolEdge {
                from: 1,
                to: 2,
                op: PathfinderOp::Union,
            },
            BoolEdge {
                from: 2,
                to: 3,
                op: PathfinderOp::Union,
            },
            BoolEdge {
                from: 3,
                to: 1,
                op: PathfinderOp::Union,
            },
        ],
        // Laço de 1.
        vec![BoolEdge {
            from: 2,
            to: 2,
            op: PathfinderOp::Union,
        }],
        // Ciclo num canto, com um nó solto.
        vec![
            BoolEdge {
                from: 2,
                to: 3,
                op: PathfinderOp::Union,
            },
            BoolEdge {
                from: 3,
                to: 2,
                op: PathfinderOp::Union,
            },
        ],
    ];
    for (k, edges) in casos.iter().enumerate() {
        let sem_cozinhar = super::has_cycle(&ids, edges);
        let recusou = resolve_graph(&nodes, edges) == Err(GraphRefusal::Cycle);
        assert_eq!(
            sem_cozinhar, recusou,
            "caso {k}: sem-cozinhar disse {sem_cozinhar} e o resolvedor disse {recusou}"
        );
    }
}

/// **UMA LIGAÇÃO ÓRFÃ NÃO É UM CICLO.** Ela é um problema de limpeza, e chamá-la de ciclo poria na
/// tela um aviso que não descreve nada que o artista possa consertar no diagrama.
#[test]
fn uma_ligacao_orfa_nao_e_um_ciclo() {
    let ids = vec![1u64, 2, 3];
    assert!(!super::has_cycle(
        &ids,
        &[BoolEdge {
            from: 99,
            to: 1,
            op: PathfinderOp::Union
        }]
    ));
}

//! Os gates do documento.
//!
//! ⚠️ Nenhum deles pergunta "compila?". Cada um afirma uma propriedade que, quebrada, produz uma
//! **forma errada em silêncio** — que é o modo de falha desta crate, não o erro de compilação.

use super::*;

fn leaf(p: Primitive) -> Node {
    Node {
        xform: Xform::IDENTITY,
        kind: NodeKind::Leaf(p),
    }
}

fn cube(half: f32, round: f32) -> Node {
    leaf(Primitive::Box {
        half: [half; 3],
        round,
    })
}

/// Um documento mínimo válido: duas primitivas unidas com filete.
fn two_boxes() -> FieldDoc {
    FieldDoc::new(
        vec![
            cube(0.4, 0.0),
            cube(0.4, 0.0),
            Node {
                xform: Xform::IDENTITY,
                kind: NodeKind::Combine {
                    op: Op::Union(Blend::Exact { radius: 0.05 }),
                    children: vec![NodeId(0), NodeId(1)],
                },
            },
        ],
        NodeId(2),
    )
    .expect("documento mínimo é válido")
}

#[test]
fn a_minimal_document_is_accepted() {
    let doc = two_boxes();
    assert_eq!(doc.nodes().len(), 3);
    assert_eq!(doc.root(), NodeId(2));
    assert_eq!(doc.version, FIELD_DOC_VERSION);
}

/// A invariante topológica (doc da crate): filho SEMPRE antes do pai.
///
/// É ela que torna ciclo uma **impossibilidade** em vez de um erro a detectar. Sem este gate, a
/// invariante é um comentário — e um comentário não morde.
#[test]
fn a_forward_reference_is_refused_because_it_is_how_a_cycle_would_enter() {
    let err = FieldDoc::new(
        vec![
            Node {
                xform: Xform::IDENTITY,
                kind: NodeKind::Combine {
                    op: Op::Union(Blend::Sharp),
                    // Aponta para a frente: é exatamente por aqui que um ciclo entraria.
                    children: vec![NodeId(1)],
                },
            },
            cube(0.4, 0.0),
        ],
        NodeId(0),
    )
    .unwrap_err();
    assert_eq!(
        err,
        FieldError::ForwardReference {
            parent: 0,
            child: 1
        }
    );
}

/// ⚠️ O caso que motiva o limite: arredondar uma caixa encolhe a fonte em `round` nos TRÊS eixos.
/// Com `round >= a menor meia-extensão`, aquele eixo deixa de existir e o que sai **não é uma
/// distância** — é uma forma que parece plausível e mede errado.
#[test]
fn a_round_that_does_not_fit_in_the_smallest_half_is_refused() {
    // Caixa achatada: 0,5 × 0,5 × 0,06. Um raio de 0,08 cabe em dois eixos e não no terceiro.
    let err = FieldDoc::new(
        vec![leaf(Primitive::Box {
            half: [0.5, 0.5, 0.06],
            round: 0.08,
        })],
        NodeId(0),
    )
    .unwrap_err();
    assert!(
        matches!(err, FieldError::RoundTooLarge { limit, .. } if (limit - 0.06).abs() < 1e-6),
        "o limite tem de ser a MENOR meia-extensão, não a maior: {err:?}"
    );
}

#[test]
fn a_round_that_fits_exactly_under_the_limit_is_accepted() {
    FieldDoc::new(
        vec![leaf(Primitive::Box {
            half: [0.5, 0.5, 0.06],
            round: 0.059,
        })],
        NodeId(0),
    )
    .expect("cabe por 0,001");
}

/// ⛔ ADR-0161 §6: escala não-uniforme destrói ‖∇f‖ = 1. Aqui ela nem é representável — este gate
/// guarda a outra metade, que é a escala **inválida**.
#[test]
fn a_non_positive_or_non_finite_scale_is_refused() {
    for bad in [0.0_f32, -1.0, f32::NAN, f32::INFINITY] {
        let node = Node {
            xform: Xform {
                scale: bad,
                ..Xform::IDENTITY
            },
            kind: NodeKind::Leaf(Primitive::Sphere { radius: 0.5 }),
        };
        assert_eq!(
            FieldDoc::new(vec![node], NodeId(0)).unwrap_err(),
            FieldError::BadScale { node: 0 },
            "escala {bad} tinha de ser recusada"
        );
    }
}

#[test]
fn an_empty_combine_is_refused() {
    let err = FieldDoc::new(
        vec![Node {
            xform: Xform::IDENTITY,
            kind: NodeKind::Combine {
                op: Op::Union(Blend::Sharp),
                children: vec![],
            },
        }],
        NodeId(0),
    )
    .unwrap_err();
    assert_eq!(err, FieldError::EmptyCombine { node: 0 });
}

#[test]
fn a_root_outside_the_arena_is_refused() {
    assert_eq!(
        FieldDoc::new(vec![cube(0.4, 0.0)], NodeId(7)).unwrap_err(),
        FieldError::BadRoot
    );
    assert_eq!(
        FieldDoc::new(vec![], NodeId(0)).unwrap_err(),
        FieldError::BadRoot
    );
}

#[test]
fn a_non_positive_dimension_is_refused() {
    assert_eq!(
        FieldDoc::new(vec![leaf(Primitive::Sphere { radius: 0.0 })], NodeId(0)).unwrap_err(),
        FieldError::NonPositive {
            node: 0,
            what: "radius"
        }
    );
}

/// **HR-14** — a forma do documento salvo é PINADA.
///
/// ⚠️ Este gate não prova que a serialização "funciona": ele prova que ela **não mudou sem que
/// alguém decidisse**. Um campo novo no meio de um `enum` muda os bytes de todo documento já salvo
/// pelo utilizador, e o sintoma aparece no dia em que ele abre um arquivo antigo — não aqui.
/// Quebrou? A cura é **subir o [`FIELD_DOC_VERSION`] e escrever a migração**, nunca re-pinar o
/// número e seguir.
#[test]
fn the_shape_of_a_saved_field_is_pinned() {
    let doc = two_boxes();
    let bytes = postcard::to_allocvec(&doc).expect("serializa");
    assert_eq!(
        bytes.len(),
        // ⚠️ MEDIDO na criação do gate (2026-08-19), não adivinhado. Pinar um número na primeira
        // escrita é medição; re-pinar depois de ele quebrar é apagar a prova.
        145,
        "a forma serializada mudou — suba FIELD_DOC_VERSION e escreva a migração, \
         não re-pine este número"
    );

    let back: FieldDoc = postcard::from_bytes(&bytes).expect("desserializa");
    assert_eq!(back, doc, "ida e volta tem de devolver o mesmo documento");
}

/// A serialização é **determinística**: os mesmos dados dão os mesmos bytes.
/// É o que o undo por snapshot exige — ele compara BYTES, e um serializador instável faria todo
/// quadro virar um passo espúrio de undo (o bug que o `canonicalize()` do shell já pagou).
#[test]
fn serialising_twice_gives_identical_bytes() {
    let doc = two_boxes();
    let a = postcard::to_allocvec(&doc).expect("serializa");
    let b = postcard::to_allocvec(&doc).expect("serializa");
    assert_eq!(a, b);
}

/// Unir documentos preserva a invariante topológica **sem ordenar nada** — o deslocamento de
/// índice basta, e este gate é o que impede alguém de "otimizar" a concatenação para uma ordem
/// que a quebraria em silêncio (a arena aceitaria, e o campo sairia errado três waves depois).
#[test]
fn union_all_keeps_every_child_before_its_parent() {
    let a = two_boxes();
    let b = FieldDoc::new(vec![cube(0.2, 0.05)], NodeId(0)).expect("cubo");
    let merged = FieldDoc::union_all(&[a.clone(), b.clone()], Blend::Exact { radius: 0.03 })
        .expect("dois documentos")
        .expect("a união de documentos válidos é válida");

    assert_eq!(merged.nodes().len(), a.nodes().len() + b.nodes().len() + 1);
    assert_eq!(merged.root(), NodeId(merged.nodes().len() as u32 - 1));
    for (i, node) in merged.nodes().iter().enumerate() {
        if let NodeKind::Combine { children, .. } = &node.kind {
            for c in children {
                assert!(
                    (c.0 as usize) < i,
                    "nó {i} aponta para {} — a invariante caiu na união",
                    c.0
                );
            }
        }
    }
}

/// Lista vazia devolve `None`, e um documento só volta **ele mesmo**.
/// ⚠️ A segunda metade não é trivialidade: embrulhar um documento único numa união de um filho
/// acrescentaria um nó a cada salvamento, e um arquivo que engorda ao ser reaberto é o tipo de
/// bug que só aparece no décimo `abrir` do utilizador.
#[test]
fn union_all_is_identity_for_one_and_nothing_for_none() {
    assert!(FieldDoc::union_all(&[], Blend::Sharp).is_none());
    let one = two_boxes();
    let same = FieldDoc::union_all(std::slice::from_ref(&one), Blend::Sharp)
        .expect("um documento")
        .expect("válido");
    assert_eq!(same, one);
}

// ─────────────────────────────────────────────────────────────────────────────────────────────────
// W3 — o perfil
// ─────────────────────────────────────────────────────────────────────────────────────────────────

fn unit_square() -> Vec<[f32; 2]> {
    vec![[-1.0, -1.0], [1.0, -1.0], [1.0, 1.0], [-1.0, 1.0]]
}

fn a_profile() -> Profile {
    Profile::new(vec![unit_square()], FillRule::NonZero, 1e-3).expect("quadrado")
}

/// ⚠️ **O ponto de fecho repetido é REMOVIDO.**
///
/// Quem constrói uma polilinha fechada à mão repete o primeiro ponto no fim — é o hábito de todo
/// formato de desenho. Aqui isso seria uma aresta de comprimento **zero**, e a distância
/// ponto-segmento divide pelo comprimento ao quadrado: `0/0`, e o campo inteiro vira `NaN` a partir
/// dali. O construtor limpa em vez de recusar, porque a entrada é legítima — só a representação é
/// que não.
#[test]
fn a_repeated_closing_point_is_removed_not_kept() {
    let mut with_closing = unit_square();
    with_closing.push(with_closing[0]);
    let p = Profile::new(vec![with_closing], FillRule::NonZero, 1e-3).expect("quadrado fechado");
    assert_eq!(
        p.segment_count(),
        4,
        "o ponto de fecho repetido tem de sair: {:?}",
        p.contours()[0]
    );
}

/// Pontos consecutivos iguais no meio do contorno também saem — mesma razão.
#[test]
fn consecutive_duplicate_points_are_removed() {
    let mut dupes = vec![[-1.0_f32, -1.0], [-1.0, -1.0], [1.0, -1.0]];
    dupes.extend([[1.0_f32, 1.0], [1.0, 1.0], [-1.0, 1.0]]);
    let p = Profile::new(vec![dupes], FillRule::NonZero, 1e-3).expect("quadrado com repetidos");
    assert_eq!(p.segment_count(), 4);
}

/// Um contorno que colapsou numa reta não delimita área — e é recusado por uma extensão nula da
/// caixa, **não** pela área.
///
/// ⚠️ A distinção importa: uma figura em **oito** tem área líquida zero e é um perfil legítimo sob
/// `EvenOdd`. Recusar por área mataria o caso válido e deixaria passar este.
#[test]
fn a_contour_collapsed_to_a_line_is_refused() {
    let line = vec![[0.0_f32, 0.0], [1.0, 0.0], [2.0, 0.0]];
    match Profile::new(vec![line], FillRule::NonZero, 1e-3) {
        Err(ProfileError::Collapsed {
            contour, height, ..
        }) => {
            assert_eq!(contour, 0);
            assert_eq!(height, 0.0);
        }
        other => panic!("uma reta não é perfil: {other:?}"),
    }
}

#[test]
fn a_profile_needs_three_points_a_finite_coordinate_and_a_tolerance() {
    assert_eq!(
        Profile::new(vec![], FillRule::NonZero, 1e-3),
        Err(ProfileError::Empty)
    );
    assert_eq!(
        Profile::new(vec![vec![[0.0, 0.0], [1.0, 1.0]]], FillRule::NonZero, 1e-3),
        Err(ProfileError::TooFewPoints {
            contour: 0,
            points: 2
        })
    );
    assert_eq!(
        Profile::new(
            vec![vec![[0.0, 0.0], [1.0, f32::NAN], [0.0, 1.0]]],
            FillRule::NonZero,
            1e-3
        ),
        Err(ProfileError::NonFinite { contour: 0 })
    );
    assert_eq!(
        Profile::new(vec![unit_square()], FillRule::NonZero, 0.0),
        Err(ProfileError::BadTolerance { tolerance: 0.0 })
    );
}

/// ⭐ **Um perfil que cruza o eixo é RECUSADO na revolução.**
///
/// A superfície de revolução de um contorno que atravessa o eixo auto-intersecta, e o campo que sai
/// disso deixa de ser uma distância — a marcha de raios atravessa a peça e o raio de um filete
/// deixa de ser o raio. Recusar é a única resposta honesta: aceitar produziria uma forma errada sem
/// um erro em lado nenhum.
#[test]
fn a_revolve_whose_profile_crosses_the_axis_is_refused() {
    let crossing = Profile::new(
        vec![vec![[-0.5, 0.0], [0.5, 0.0], [0.0, 1.0]]],
        FillRule::NonZero,
        1e-3,
    )
    .expect("triângulo é um perfil válido — é a REVOLUÇÃO dele que não é");
    match FieldDoc::new(
        vec![leaf(Primitive::Revolve { profile: crossing })],
        NodeId(0),
    ) {
        Err(FieldError::ProfileCrossesAxis { node, min_x }) => {
            assert_eq!(node, 0);
            assert_eq!(min_x, -0.5);
        }
        other => panic!("cruzar o eixo tem de ser recusado: {other:?}"),
    }
    // Tocar o eixo (x = 0) é legítimo — é como um sólido de revolução se fecha em cima.
    let touching = Profile::new(
        vec![vec![[0.0, 0.0], [0.5, 0.0], [0.0, 1.0]]],
        FillRule::NonZero,
        1e-3,
    )
    .expect("perfil");
    assert!(
        FieldDoc::new(
            vec![leaf(Primitive::Revolve { profile: touching })],
            NodeId(0)
        )
        .is_ok(),
        "um perfil que TOCA o eixo é como um sólido de revolução se fecha; recusá-lo proibiria a \
         esfera"
    );
}

/// ⚠️ Na extrusão o limite do `round` é a **meia-altura**, e só ela.
///
/// Um `round` maior que a meia-largura do perfil **não** é erro: a receita é uma abertura
/// morfológica e o pescoço fino desaparece, que é o que arredondar com esse raio significa. Na
/// altura é diferente — o termo axial inverte de sinal e o sólido deixa de existir.
#[test]
fn an_extrusion_bounds_the_round_by_the_height_and_not_by_the_profile() {
    // O perfil tem meia-largura 1,0; o `round` abaixo é 1,5 — **maior que ela** — e mesmo assim
    // passa, porque quem limita é a meia-altura (2,0).
    let wide_round = FieldDoc::new(
        vec![leaf(Primitive::Extrude {
            profile: a_profile(),
            half_height: 2.0,
            round: 1.5,
        })],
        NodeId(0),
    );
    assert!(
        wide_round.is_ok(),
        "um round maior que a meia-largura do perfil é uma ABERTURA, não um erro"
    );

    match FieldDoc::new(
        vec![leaf(Primitive::Extrude {
            profile: a_profile(),
            half_height: 0.3,
            round: 0.3,
        })],
        NodeId(0),
    ) {
        Err(FieldError::RoundTooLarge { round, limit, .. }) => {
            assert_eq!((round, limit), (0.3, 0.3));
        }
        other => panic!("round ≥ meia-altura tem de ser recusado: {other:?}"),
    }
}

/// **HR-14 outra vez, agora para a forma que a v2 acrescentou.**
///
/// ⚠️ O gate irmão (`the_shape_of_a_saved_field_is_pinned`) continua a medir **145** bytes, e isso
/// é um resultado, não um acaso: variantes novas no FIM de um `enum` não mexem nos bytes das
/// antigas, então **todo documento já salvo continua a ler**. O `FIELD_DOC_VERSION` subiu para 2
/// porque um leitor da v1 não sabe o que é um `Extrude` — não porque os arquivos antigos partiram.
#[test]
fn the_shape_of_a_saved_profile_is_pinned() {
    let doc = FieldDoc::new(
        vec![leaf(Primitive::Extrude {
            profile: a_profile(),
            half_height: 0.4,
            round: 0.05,
        })],
        NodeId(0),
    )
    .expect("extrusão");
    let bytes = postcard::to_allocvec(&doc).expect("serializa");
    assert_eq!(
        bytes.len(),
        // ⚠️ MEDIDO na criação do gate (2026-08-19): 4 pontos × 2 × f32 + o cabeçalho da árvore.
        84,
        "a forma serializada do perfil mudou — suba FIELD_DOC_VERSION e escreva a migração, \
         não re-pine este número"
    );
    let back: FieldDoc = postcard::from_bytes(&bytes).expect("desserializa");
    assert_eq!(back, doc);
}

// ─────────────────────────────────────────────────────────────────────────────────────────────────
// W4 — o raio EDITÁVEL: a promessa central do módulo, virada em API
// ─────────────────────────────────────────────────────────────────────────────────────────────────

/// ⭐ **O raio de uma união muda depois de a peça existir** — e é isto que nem o Blender nem o MoI
/// dão: lá o filete é geometria assada, e mudá-lo é refazer a operação.
#[test]
fn the_blend_radius_is_editable_after_the_fact() {
    let mut doc = two_boxes();
    assert_eq!(doc.radius_of(NodeId(2)), Some(0.05));
    doc.set_radius(NodeId(2), 0.11).expect("raio válido");
    assert_eq!(doc.radius_of(NodeId(2)), Some(0.11));
    // E o caráter continua a ser o da operação — mexer no número não troca união por subtração.
    match &doc.nodes()[2].kind {
        NodeKind::Combine { op, .. } => assert!(matches!(op, Op::Union(Blend::Exact { .. }))),
        other => panic!("o nó 2 é a combinação: {other:?}"),
    }
}

/// **Raio zero é a aresta VIVA**, e não um modo à parte — ida e volta pela mesma porta.
#[test]
fn zero_radius_is_the_sharp_edge_and_it_comes_back() {
    let mut doc = two_boxes();
    doc.set_radius(NodeId(2), 0.0).expect("zero é válido");
    assert!(matches!(
        &doc.nodes()[2].kind,
        NodeKind::Combine {
            op: Op::Union(Blend::Sharp),
            ..
        }
    ));
    doc.set_radius(NodeId(2), 0.07).expect("acorda");
    assert_eq!(doc.radius_of(NodeId(2)), Some(0.07));
}

/// ⚠️ **O caráter ORGÂNICO sobrevive a mexer no número.**
///
/// Ele é uma escolha de produto (uma transição derretida, que entrega 3/4 do número — ver
/// [`Blend::Organic`]). Trocá-lo por `Exact` porque alguém arrastou um controle seria decidir pelo
/// utilizador, e o sintoma seria a forma mudar de caráter sem ninguém ter pedido.
#[test]
fn editing_the_number_does_not_change_an_organic_blend_into_an_exact_one() {
    let mut doc = FieldDoc::new(
        vec![
            cube(0.4, 0.0),
            cube(0.4, 0.0),
            Node {
                xform: Xform::IDENTITY,
                kind: NodeKind::Combine {
                    op: Op::Union(Blend::Organic { k: 0.05 }),
                    children: vec![NodeId(0), NodeId(1)],
                },
            },
        ],
        NodeId(2),
    )
    .expect("orgânico");
    doc.set_radius(NodeId(2), 0.09).expect("raio válido");
    assert!(matches!(
        &doc.nodes()[2].kind,
        NodeKind::Combine {
            op: Op::Union(Blend::Organic { k: 0.09 }),
            ..
        }
    ));
}

/// ⭐ **Uma edição recusada deixa o documento COMO ESTAVA.**
///
/// Um `set` que validasse depois de escrever, e devolvesse o erro sem desfazer, deixaria um
/// documento meio-mudado — que é pior do que a recusa, porque a invariante da crate (*um documento
/// que existe está válido*) passaria a ser falsa em silêncio.
#[test]
fn a_refused_edit_leaves_the_document_untouched() {
    let mut doc = FieldDoc::new(vec![cube(0.4, 0.05)], NodeId(0)).expect("cubo");
    let before = doc.clone();
    // O teto de uma caixa de meia-extensão 0,4 é 0,4 — 0,9 não cabe.
    let err = doc.set_radius(NodeId(0), 0.9).unwrap_err();
    assert!(matches!(err, FieldError::RoundTooLarge { .. }), "{err:?}");
    assert_eq!(
        doc, before,
        "o documento tem de ficar exatamente como estava"
    );
}

/// ⭐ **O teto que o controle mostra é o MESMO que a validação aplica.**
///
/// Se fossem duas contas, o controle ofereceria valores que o documento recusa — e o utilizador
/// veria o número parar sem explicação. Este gate varre a faixa e exige que os dois lados
/// concordem em **todo** valor.
#[test]
fn the_bound_the_panel_shows_is_the_bound_the_document_enforces() {
    for p in [
        Primitive::Box {
            half: [0.4, 0.3, 0.5],
            round: 0.0,
        },
        Primitive::Cylinder {
            radius: 0.25,
            half_height: 0.7,
            round: 0.0,
        },
    ] {
        let limit = round_limit(&p).expect("estas primitivas têm round");
        let mut doc = FieldDoc::new(vec![leaf(p)], NodeId(0)).expect("documento");
        assert_eq!(doc.radius_bound(NodeId(0)), Some(Bound::Hard(limit)));
        for i in 1u16..40 {
            let r = limit * f32::from(i) / 20.0;
            let accepted = doc.set_radius(NodeId(0), r).is_ok();
            assert_eq!(
                accepted,
                r < limit,
                "raio {r} contra teto {limit}: o documento diz {accepted} e a fronteira diz {}",
                r < limit
            );
        }
    }
}

/// O limite de uma MISTURA é de escala, não de validade — e o tipo diz isso.
///
/// ⚠️ Um raio enorme numa união continua a ser um documento **válido**: o campo não deixa de ser uma
/// distância, a forma é que vira uma bolha. Tratá-lo como parede proibiria o que é legítimo.
#[test]
fn a_blend_bound_is_scale_not_validity() {
    let mut doc = two_boxes();
    let bound = doc.radius_bound(NodeId(2)).expect("a combinação tem raio");
    assert!(
        matches!(bound, Bound::Soft(_)),
        "o raio de mistura não tem parede: {bound:?}"
    );
    // A escala vem da menor peça sob o nó: duas caixas de meia-extensão 0,4.
    assert!(
        (bound.value() - 0.4).abs() < 1e-6,
        "escala: {}",
        bound.value()
    );
    // E exceder a sugestão é ACEITE — é uma sugestão.
    doc.set_radius(NodeId(2), bound.value() * 3.0)
        .expect("um raio grande é feio, não é inválido");
}

/// Uma esfera não tem aresta, logo não tem raio para editar — e a porta diz isso em vez de fingir.
#[test]
fn a_shape_without_an_edge_has_no_radius_to_edit() {
    let mut doc =
        FieldDoc::new(vec![leaf(Primitive::Sphere { radius: 0.5 })], NodeId(0)).expect("esfera");
    assert_eq!(doc.radius_of(NodeId(0)), None);
    assert_eq!(doc.radius_bound(NodeId(0)), None);
    assert!(doc.set_radius(NodeId(0), 0.1).is_err());
}

/// ⭐ **As dimensões que uma forma mostra são as que ela tem** — e as meias-extensões não aparecem.
///
/// ⚠️ O documento guarda **meias**-extensões (é a forma que a distância assinada quer). Ninguém diz
/// que uma caixa tem «meia-largura 5»: a lista devolve a largura inteira e a escrita volta a
/// dividir. Sem a conversão num sítio, cada painel que a mostrasse teria a sua — e uma delas ficaria
/// para trás.
#[test]
fn a_box_reports_full_extents_not_half_ones() {
    let mut b = Primitive::Box {
        half: [0.5, 0.25, 0.1],
        round: 0.05,
    };
    let d = crate::dims(&b);
    assert_eq!(d.len(), 4);
    assert!((d[0].value - 1.0).abs() < 1e-6, "largura inteira");
    assert!((d[1].value - 0.5).abs() < 1e-6);
    assert!((d[2].value - 0.2).abs() < 1e-6);
    assert!((d[3].value - 0.05).abs() < 1e-6, "o filete é ele próprio");

    crate::set_dim(&mut b, 0, 0, 3.0).expect("largura válida");
    let Primitive::Box { half, .. } = b else {
        panic!("continua caixa")
    };
    assert!((half[0] - 1.5).abs() < 1e-6, "3 de largura é 1,5 de meia");
}

/// ⭐ **Encolher uma forma ENCOLHE o filete dela, e não é recusado.**
///
/// ⚠️ Um filete que deixa de caber é a situação normal, não um erro: o artista pediu o tamanho, e o
/// filete é o que **decorre** dele. Recusar obrigaria a desfazer o filete primeiro — dois gestos
/// onde há um. E o nó tem de ficar **válido**: a invariante do módulo é *um nó que existe está
/// válido*.
#[test]
fn shrinking_a_shape_shrinks_its_fillet_instead_of_refusing() {
    let mut b = Primitive::Box {
        half: [0.5; 3],
        round: 0.4,
    };
    // Encolhe a caixa muito abaixo do filete de 0,4.
    crate::set_dim(&mut b, 0, 0, 0.2).expect("encolher é legítimo");
    assert!(crate::clamp_round(&mut b), "o filete tinha de ser limitado");

    let Primitive::Box { half, round } = b else {
        panic!("continua caixa")
    };
    assert!((half[0] - 0.1).abs() < 1e-6);
    assert!(
        round < crate::round_limit(&b).expect("a caixa tem parede"),
        "o filete {round} tem de ficar ABAIXO da parede — «igual» já é inválido"
    );
    // E o documento aceita-a: é essa a prova de que o limite foi ao sítio certo.
    FieldDoc::new(
        vec![Node {
            xform: Xform::IDENTITY,
            kind: NodeKind::Leaf(b),
        }],
        NodeId(0),
    )
    .expect("a caixa limitada é válida");
}

/// ⚠️ **Um valor não-positivo é recusado, e o nó fica como estava.** Uma caixa de largura zero não é
/// uma caixa fina — é uma forma que deixou de existir num eixo, e o campo que sai dela não é uma
/// distância.
#[test]
fn writing_a_non_positive_dimension_is_refused_and_leaves_the_shape_alone() {
    let mut c = Primitive::Cylinder {
        radius: 0.3,
        half_height: 0.5,
        round: 0.05,
    };
    for bad in [0.0f32, -1.0, f32::NAN] {
        assert!(crate::set_dim(&mut c, 0, 0, bad).is_err(), "{bad} passou");
    }
    let Primitive::Cylinder { radius, .. } = c else {
        panic!("continua cilindro")
    };
    assert!((radius - 0.3).abs() < 1e-6, "o valor antigo tinha de ficar");
    // E um índice que não é desta forma também.
    assert!(crate::set_dim(&mut c, 0, 9, 1.0).is_err());
}

/// **Um torno não tem dimensões próprias** — a forma dele é o contorno desenhado.
///
/// ⚠️ Um número aqui prometeria mexer numa coisa que só o editor vetorial autora. E o gate mede as
/// duas metades: a lista é vazia, **e** uma escrita nela é recusada.
#[test]
fn a_revolve_has_no_dimensions_of_its_own() {
    let mut r = Primitive::Revolve {
        profile: a_profile(),
    };
    assert!(crate::dims(&r).is_empty());
    assert!(crate::set_dim(&mut r, 0, 0, 1.0).is_err());
}

/// ⭐ **Toda forma que tem filete diz onde ele está e qual é a parede dele.**
///
/// ⚠️ O gate percorre as primitivas todas: uma nova que ganhe `round` e se esqueça da entrada na
/// lista ficaria com um filete invisível ao painel — presente no documento, inalcançável pelo
/// artista.
#[test]
fn every_shape_with_a_fillet_exposes_it_with_its_wall() {
    let shapes = [
        Primitive::Box {
            half: [0.4; 3],
            round: 0.05,
        },
        Primitive::Sphere { radius: 0.3 },
        Primitive::Cylinder {
            radius: 0.3,
            half_height: 0.5,
            round: 0.05,
        },
        Primitive::Torus {
            major: 0.4,
            minor: 0.1,
        },
    ];
    for p in shapes {
        let has_wall = crate::round_limit(&p).is_some();
        let exposed = crate::dims(&p).iter().any(|d| d.key == "field.dim.round");
        assert_eq!(
            has_wall, exposed,
            "{p:?}: o documento diz que tem filete = {has_wall}, e a lista diz {exposed}"
        );
        // E a parede que a lista mostra é a MESMA que a validação aplica.
        if let Some(d) = crate::dims(&p).iter().find(|d| d.key == "field.dim.round") {
            assert_eq!(
                d.span,
                crate::Span::Wall(crate::round_limit(&p).expect("tem parede"))
            );
        }
        // ⚠️ Nenhuma dimensão que NÃO é o filete tem parede: inventar um teto para a largura de uma
        // caixa seria escrever um limite que a física não pede.
        for d in crate::dims(&p)
            .iter()
            .filter(|d| d.key != "field.dim.round")
        {
            assert_eq!(
                d.span,
                crate::Span::Positive,
                "{p:?}: `{}` ganhou um teto inventado",
                d.key
            );
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// A ROTAÇÃO COMO TRÊS ÂNGULOS — o nome canónico de um quaternion.
// ─────────────────────────────────────────────────────────────────────────────

/// A distância entre duas rotações, em componentes.
///
/// ⚠️ **`q` e `−q` são a MESMA rotação.** Comparar componente a componente sem isto reprovaria
/// metade das ida-e-voltas **corretas** — e o gate a passar a acusar o código certo é a forma cara
/// de um gate errado.
fn rotation_gap(a: [f32; 4], b: [f32; 4]) -> f32 {
    let same = (0..4).map(|i| (a[i] - b[i]).abs()).fold(0.0f32, f32::max);
    let flipped = (0..4).map(|i| (a[i] + b[i]).abs()).fold(0.0f32, f32::max);
    same.min(flipped)
}

/// A tolerância de uma ida-e-volta de orientação. Ver `EULER_LOCK_EPS`: entrar no ramo da trava a
/// `|cos β| = 1e-4` custa um desvio da mesma ordem, e é ESSE o número que a barra tem de admitir —
/// não o `f32::EPSILON`, que reprovaria o caso que a lei trata de propósito.
const ROT_EPS: f32 = 2.0e-4;

/// Uma amostra de orientações que inclui o caso difícil.
fn orientation_sample() -> Vec<[f32; 4]> {
    use crate::xform::{quat_axis_angle, quat_from_euler, quat_mul};
    let d = f32::to_radians;
    let mut out = vec![
        [0.0, 0.0, 0.0, 1.0],
        quat_axis_angle([1.0, 0.0, 0.0], d(90.0)),
        quat_axis_angle([0.0, 1.0, 0.0], d(-90.0)),
        quat_axis_angle([0.0, 0.0, 1.0], d(179.0)),
        quat_axis_angle([1.0, 2.0, -3.0], d(137.0)),
        quat_mul(
            quat_axis_angle([0.0, 1.0, 0.0], d(45.0)),
            quat_axis_angle([1.0, 0.0, 0.0], d(60.0)),
        ),
    ];
    // ⭐ **A trava de cardan, pelos dois polos** — o caso que uma amostra "genérica" nunca sorteia,
    // e o único em que o trio deixa de ser um nome de três partes.
    for beta in [90.0f32, -90.0] {
        for (a, c) in [(0.0f32, 0.0f32), (30.0, 40.0), (-120.0, 75.0)] {
            out.push(quat_from_euler([d(a), d(beta), d(c)]));
        }
    }
    out
}

/// ⭐ **O que fecha o ciclo é a ORIENTAÇÃO, nunca o trio.**
///
/// ⚠️ É a lei inteira desta representação, e é o que torna legítimo o painel **não** guardar os três
/// ângulos: eles são um *nome* do quaternion, e o quaternion é o facto. Um trio somado de uma volta
/// nomeia o mesmo sítio; nos polos, uma família inteira o nomeia. Um gate que exigisse
/// `to_euler(from_euler(e)) == e` para todo `e` estaria a exigir do código uma coisa que não é
/// verdade — e a cura seria guardar o trio, que é a segunda verdade que este módulo recusa.
#[test]
fn the_orientation_round_trips_even_though_the_triple_need_not() {
    use crate::xform::{quat_from_euler, quat_to_euler};
    for q in orientation_sample() {
        let back = quat_from_euler(quat_to_euler(q));
        let gap = rotation_gap(q, back);
        assert!(
            gap <= ROT_EPS,
            "a orientação não voltou: {q:?} -> {:?} -> {back:?} (folga {gap:e})",
            quat_to_euler(q).map(f32::to_degrees)
        );
    }
}

/// ⭐ **Nos polos a extração é FINITA e determinística** — e o eixo Z fica exatamente em zero.
///
/// ⚠️ Sem o ramo da trava, `atan2(0, 0)` daria zero por acidente num sítio e o ruído de `f32`
/// escolheria um ângulo qualquer noutro: o painel mostraria três números a tremer numa peça parada.
#[test]
fn at_the_pole_the_extraction_is_finite_and_the_third_angle_is_pinned() {
    use crate::xform::{quat_from_euler, quat_to_euler};
    let d = f32::to_radians;
    for beta in [90.0f32, -90.0] {
        for (a, c) in [(0.0f32, 0.0f32), (30.0, 40.0), (-120.0, 75.0)] {
            let q = quat_from_euler([d(a), d(beta), d(c)]);
            let e = quat_to_euler(q);
            assert!(e.iter().all(|v| v.is_finite()), "{beta} {a} {c}: {e:?}");
            assert_eq!(e[2], 0.0, "no polo o Z é fixado em zero, e não sorteado");
            assert!(
                (e[1].to_degrees() - beta).abs() < 0.05,
                "o eixo do meio tinha de ficar no polo: {}",
                e[1].to_degrees()
            );
        }
    }
}

/// ⭐ **A ordem é a do «XYZ Euler» do Blender: `R = Rz · Ry · Rx`.**
///
/// ⚠️ O oráculo é uma **matriz montada à mão** no teste, e não a composição de quaternions que a
/// função usa: um gate que compusesse `quat_mul` na mesma ordem provaria que o código faz o que o
/// código faz. Trocar a ordem (ou a mão) muda para onde a peça aponta, e as duas versões são
/// igualmente "verdes" contra si próprias.
#[test]
fn the_order_is_the_blender_xyz_euler() {
    use crate::xform::{quat_from_euler, quat_rotate};
    type M = [[f32; 3]; 3];
    let mul = |a: M, b: M| -> M {
        let mut o = [[0.0f32; 3]; 3];
        for (i, row) in o.iter_mut().enumerate() {
            for (j, cell) in row.iter_mut().enumerate() {
                *cell = (0..3).map(|k| a[i][k] * b[k][j]).sum();
            }
        }
        o
    };
    let apply =
        |m: M, v: [f32; 3]| -> [f32; 3] { [0, 1, 2].map(|i| (0..3).map(|k| m[i][k] * v[k]).sum()) };
    let d = f32::to_radians;
    let (ax, ay, az) = (d(37.0), d(-52.0), d(114.0));
    let (sx, cx) = ax.sin_cos();
    let (sy, cy) = ay.sin_cos();
    let (sz, cz) = az.sin_cos();
    let rx: M = [[1.0, 0.0, 0.0], [0.0, cx, -sx], [0.0, sx, cx]];
    let ry: M = [[cy, 0.0, sy], [0.0, 1.0, 0.0], [-sy, 0.0, cy]];
    let rz: M = [[cz, -sz, 0.0], [sz, cz, 0.0], [0.0, 0.0, 1.0]];
    let m = mul(mul(rz, ry), rx);

    let q = quat_from_euler([ax, ay, az]);
    for v in [
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        [0.3, -0.7, 0.5],
    ] {
        let by_matrix = apply(m, v);
        let by_quat = quat_rotate(q, v);
        for k in 0..3 {
            assert!(
                (by_matrix[k] - by_quat[k]).abs() < 1e-5,
                "a ordem (ou a mão) diverge em {v:?}: matriz {by_matrix:?} vs quaternion {by_quat:?}"
            );
        }
    }
}

/// ⭐ **Escrever um ângulo deixa os outros dois onde estavam.**
///
/// ⚠️ Fora dos polos, e é a diferença entre um painel com três campos e um com um só: se pôr o X
/// mexesse no Z, o artista teria de repor os três a cada correção.
#[test]
fn writing_one_angle_leaves_the_other_two_where_they_were() {
    use crate::xform::{rotation_degrees, set_rotation_degree};
    let mut pose = Xform::IDENTITY;
    set_rotation_degree(&mut pose, 0, 20.0);
    set_rotation_degree(&mut pose, 1, -35.0);
    set_rotation_degree(&mut pose, 2, 110.0);
    let before = rotation_degrees(pose);
    set_rotation_degree(&mut pose, 1, 12.0);
    let after = rotation_degrees(pose);
    assert!((after[1] - 12.0).abs() < 0.01, "o eixo pedido: {after:?}");
    assert!(
        (after[0] - before[0]).abs() < 0.01 && (after[2] - before[2]).abs() < 0.01,
        "os outros dois mexeram-se: {before:?} -> {after:?}"
    );
}

/// ⭐ **Um ângulo além de meia volta não é RECUSADO — ele é RENOMEADO, e à vista.**
///
/// ⚠️ O gate mede as duas metades, porque só juntas elas são a lei: a peça vai **exatamente** para
/// onde 200° a põem (a orientação é a mesma), e o número que o painel passa a mostrar é −160°, que
/// é o nome canónico do mesmo sítio. Um `clamp` a 180° passaria na segunda metade e reprovaria na
/// primeira — e seria uma UI a recusar uma orientação legítima.
#[test]
fn an_angle_past_half_a_turn_is_renamed_not_refused() {
    use crate::xform::{quat_from_euler, rotation_degrees, set_rotation_degree};
    let mut pose = Xform::IDENTITY;
    set_rotation_degree(&mut pose, 2, 200.0);
    let gap = rotation_gap(
        pose.rotation,
        quat_from_euler([0.0, 0.0, 200.0f32.to_radians()]),
    );
    assert!(
        gap <= ROT_EPS,
        "a peça não foi para onde 200° a põem: {gap:e}"
    );
    let shown = rotation_degrees(pose);
    assert!(
        (shown[2] + 160.0).abs() < 0.01,
        "o painel tinha de passar a chamar-lhe −160°: {shown:?}"
    );
}

/// **Um eixo que não existe, ou um número que não é número, deixam a pose como estava.**
///
/// ⚠️ Um no-op silencioso e não um erro: não há quarto ângulo a escrever, e um `NaN` que entrasse na
/// pose envenenaria o traçado inteiro — a marcha de raios devolve `NaN` para **todos** os pixels.
#[test]
fn a_bad_axis_or_a_bad_number_leaves_the_pose_alone() {
    use crate::xform::set_rotation_degree;
    let mut pose = Xform::IDENTITY;
    set_rotation_degree(&mut pose, 0, 33.0);
    let kept = pose.rotation;
    for (axis, value) in [(3u8, 10.0f32), (9, 10.0), (0, f32::NAN), (1, f32::INFINITY)] {
        set_rotation_degree(&mut pose, axis, value);
        assert_eq!(
            pose.rotation, kept,
            "eixo {axis} valor {value} mexeu a pose"
        );
    }
}

/// ⭐ **O eixo do meio PRENDE no quarto de volta; os de fora ENROLAM na meia volta.**
///
/// ⚠️ São duas leis diferentes e o gate mede as duas, porque trocá-las é o defeito: enrolar o do
/// meio dá um sítio que a leitura seguinte renomeia (o ciclo de dois que o Enio viu), e prender os de
/// fora recusaria uma orientação que existe.
///
/// ⚠️ **Prender não perde orientação nenhuma** — toda orientação tem um trio canónico com
/// `|β| ≤ 90°`. Perde-se o nome, e a segunda metade do gate prova-o: o mesmo sítio que «Y = 120»
/// nomeia é alcançável por `X = 180 · Y = 60 · Z = 180`.
#[test]
fn the_middle_angle_is_pinned_at_a_quarter_turn_and_the_outer_two_wrap() {
    use crate::xform::{quat_from_euler, rotation_degrees, set_rotation_degree};
    let mut pose = Xform::IDENTITY;
    set_rotation_degree(&mut pose, 1, 120.0);
    let shown = rotation_degrees(pose);
    assert!(
        (shown[1] - 90.0).abs() < 0.01,
        "o eixo do meio tinha de PRENDER em 90: {shown:?}"
    );

    // Os de fora enrolam, e o sítio é o mesmo.
    let mut z = Xform::IDENTITY;
    set_rotation_degree(&mut z, 2, 200.0);
    assert!(
        rotation_gap(
            z.rotation,
            quat_from_euler([0.0, 0.0, 200.0f32.to_radians()])
        ) <= ROT_EPS,
        "200° no Z tinha de ir para onde 200° põem"
    );

    // ⭐ E o sítio que «Y = 120» nomeia continua alcançável — pelo nome canónico dele.
    let d = f32::to_radians;
    let wanted = quat_from_euler([0.0, d(120.0), 0.0]);
    let mut by_name = Xform::IDENTITY;
    set_rotation_degree(&mut by_name, 0, 180.0);
    set_rotation_degree(&mut by_name, 1, 60.0);
    set_rotation_degree(&mut by_name, 2, 180.0);
    assert!(
        rotation_gap(by_name.rotation, wanted) <= ROT_EPS,
        "o mesmo sítio tinha de ser alcançável pelo trio canónico: {:?} vs {:?}",
        rotation_degrees(by_name),
        [0.0, 120.0, 0.0]
    );
}

/// ⭐ **Na trava de cardan o terceiro ângulo é RECUSADO, e não aplicado aos poucos.**
///
/// ⚠️ Em `β = ±90°` o X e o Z são o mesmo eixo, e a forma canónica dá tudo ao X. Aplicar um Z ali
/// faria o X escorregar mais um tanto a **cada** escrita — e um arrasto escreve o mesmo alvo quadro
/// após quadro, então a peça giraria sozinha com o dedo parado. O gate escreve **três vezes** de
/// propósito: uma escrita só não distingue «recusado» de «aplicado uma vez».
#[test]
fn at_the_pole_the_third_angle_is_refused_instead_of_creeping() {
    use crate::xform::{rotation_degrees, set_rotation_degree};
    let mut pose = Xform::IDENTITY;
    set_rotation_degree(&mut pose, 0, 30.0);
    set_rotation_degree(&mut pose, 1, 90.0);
    let pinned = pose.rotation;
    for _ in 0..3 {
        set_rotation_degree(&mut pose, 2, 45.0);
        assert!(
            rotation_gap(pose.rotation, pinned) <= ROT_EPS,
            "o Z na trava mexeu a peça: {:?}",
            rotation_degrees(pose)
        );
    }
    // ⭐ E o Y continua a responder — é por ele que se sai da trava.
    set_rotation_degree(&mut pose, 1, 60.0);
    assert!(
        (rotation_degrees(pose)[1] - 60.0).abs() < 0.01,
        "sair da trava tem de ser possível: {:?}",
        rotation_degrees(pose)
    );
}

/// ⭐ **O eixo que o painel marca como inerte é EXATAMENTE o que a escrita recusa.**
///
/// ⚠️ São duas portas sobre a mesma pergunta — «este eixo responde?» — e um par destas só falha
/// quando **discorda**: um controle vivo sobre uma escrita recusada dá o número a saltar e a voltar,
/// e um controle morto sobre uma escrita que funcionava esconde uma feature. O gate percorre a faixa
/// inteira do eixo do meio, **incluindo os dois polos**.
#[test]
fn the_inert_axis_is_exactly_the_one_the_write_refuses() {
    use crate::xform::{rotation_axis_is_free, set_rotation_degree};
    for middle in [-90.0f32, -89.0, -45.0, 0.0, 45.0, 89.0, 90.0] {
        let mut pose = Xform::IDENTITY;
        set_rotation_degree(&mut pose, 1, middle);
        for axis in 0..3u8 {
            let free = rotation_axis_is_free(pose, axis);
            let before = pose.rotation;
            let mut probe = pose;
            // Um alvo que MUDA a peça em qualquer estado — senão um no-op passaria por recusa.
            set_rotation_degree(&mut probe, axis, if axis == 1 { 30.0 } else { 40.0 });
            let moved = rotation_gap(before, probe.rotation) > ROT_EPS;
            assert_eq!(
                free,
                moved,
                "Y={middle}, eixo {axis}: o painel diz livre={free} e a escrita {}",
                if moved { "aplicou" } else { "recusou" }
            );
        }
    }
}

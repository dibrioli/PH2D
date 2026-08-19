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

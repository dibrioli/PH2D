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

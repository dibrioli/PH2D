//! ⭐ **Os gates do REGRESSO** (W23): um projeto que reabre traz de volta a escultura que o
//! documento nomeia — e o que não voltar tem de ser dito.
//!
//! ⚠️ **A metade cara é a COSTURA**, e é ela que dita a forma destes gates. `missing_keys` é pura e
//! `field_from_file` tem os gates do campo; o que falha em silêncio é ninguém as CHAMAR no cozimento
//! — o nó existe, a Hierarquia mostra-o, o registo não conhece o nome, e a peça é invisível sem uma
//! palavra. Por isso o gate-mãe dirige `sync_scene`, que é o caminho de produção inteiro.

use super::*;
use ph2d_ecs::SimWorld;
use ph2d_field::{Node, NodeId, Xform};

/// Escreve um OBJ **de verdade** num temp e devolve o caminho.
///
/// ⚠️ **Um arquivo e não um duplo**: o que esta wave costura é *o caminho volta a ser campo*, e um
/// `Sampled` fabricado em memória saltaria por cima do leitor, do `merge`, do `recenter` e do
/// voxelizador — que são precisamente os passos que o regresso tem de repetir na mesma ordem.
fn a_sculpture_file(stem: &str) -> std::path::PathBuf {
    let mesh = ph2d_mesh::shapes::uv_sphere(12, 24, 1.0);
    let bytes = ph2d_mesh::write_obj(&[ph2d_mesh::ExportPiece {
        name: Some("blob"),
        mesh: &mesh,
        pose: ph2d_mesh::Pose::IDENTITY,
    }]);
    let path = std::env::temp_dir().join(format!("ph2d_w23_{stem}.obj"));
    std::fs::write(&path, bytes).expect("escreve o fixture");
    path
}

/// Uma peça de um nó só: a escultura, pelo nome.
fn a_part_naming(key: &str) -> ph2d_field::FieldDoc {
    ph2d_field::FieldDoc::new(
        vec![Node::new(
            Xform::IDENTITY,
            NodeKind::Sampled { key: key.into() },
        )],
        NodeId(0),
    )
    .expect("a peça de um nó")
}

/// ⭐ **O gate-mãe: reabrir o projeto traz a escultura de volta.**
///
/// ⚠️ **O registo começa VAZIO de propósito** — é exactamente o estado de um app que acabou de
/// arrancar: o mundo veio do arquivo (com o nó, a pose e o nome), e a grade de 128³ morreu com o
/// processo que a construiu. Sem a reconciliação no cozimento, o `sampled_count` é **0** e a peça é
/// vazia; o artista vê o trabalho dele desaparecido e nada explica porquê.
#[test]
fn a_project_that_reopens_gets_its_sculpture_back() {
    forget_tried();
    let path = a_sculpture_file("volta");
    let key = path.to_string_lossy().to_string();
    assert!(
        !crate::field3d_smoke::sampled_registry().contains_key(&key),
        "o gate só prova alguma coisa com o registo VAZIO — é o estado de um app recém-aberto"
    );

    let mut sim = SimWorld::new();
    let cooked = crate::field3d_scene::sync_scene(&mut sim, Some(&a_part_naming(&key)), 0.0)
        .expect("a peça cozinha");

    let reg = crate::field3d_smoke::sampled_registry();
    assert!(
        reg.contains_key(&key),
        "o cozimento tem de regenerar a escultura do arquivo que a nomeia"
    );
    let h = ph2d_field_eval::hybrid::Hybrid::new(&cooked, &reg);
    assert_eq!(
        h.sampled_count(),
        1,
        "e o avaliador tem de a resolver — senão o nó existe e a peça é invisível"
    );

    // E ela é a MALHA, não uma casca vazia: o centro da esfera está dentro.
    let mut h = h;
    let d = h.eval(&[0.0], &[0.0], &[0.0]).expect("avalia")[0];
    assert!(
        d < 0.0,
        "o centro da esfera tem de ler NEGATIVO (dentro); leu {d} — o campo que voltou não é o do \
         arquivo"
    );
}

/// **`missing_keys` nomeia exactamente o que o registo não sabe responder** — nem mais, nem menos.
///
/// ⚠️ O «nem mais» é a metade que custa: uma escultura já registada re-lida a cada quadro pagaria
/// uma leitura de disco e um voxelizador **por quadro**, sobre um arquivo que já está em memória.
#[test]
fn missing_keys_names_exactly_what_the_registry_cannot_answer() {
    let known = "/tmp/ph2d-w23-conhecida.obj";
    crate::field3d_smoke::register_sampled(
        known,
        std::sync::Arc::new(
            ph2d_field_mesh::SampledField::from_mesh(&ph2d_mesh::shapes::uv_sphere(8, 16, 1.0), 16)
                .expect("esfera"),
        ),
    );
    let doc = ph2d_field::FieldDoc::new(
        vec![
            Node::new(Xform::IDENTITY, NodeKind::Sampled { key: known.into() }),
            Node::new(
                Xform::IDENTITY,
                NodeKind::Sampled {
                    key: "/tmp/ph2d-w23-sumida.obj".into(),
                },
            ),
            // A MESMA que falta, uma segunda vez — é o que um `duplicate` da subárvore produz.
            Node::new(
                Xform::IDENTITY,
                NodeKind::Sampled {
                    key: "/tmp/ph2d-w23-sumida.obj".into(),
                },
            ),
            Node::new(
                Xform::IDENTITY,
                NodeKind::Leaf(ph2d_field::Primitive::Sphere { radius: 0.3 }),
            ),
            Node::new(
                Xform::IDENTITY,
                NodeKind::Combine {
                    op: ph2d_field::Op::Union(ph2d_field::Blend::Sharp),
                    children: vec![NodeId(0), NodeId(1), NodeId(2), NodeId(3)],
                },
            ),
        ],
        NodeId(4),
    )
    .expect("a peça");

    let reg = crate::field3d_smoke::sampled_registry();
    assert_eq!(
        missing_keys(&doc, &reg),
        vec!["/tmp/ph2d-w23-sumida.obj".to_string()],
        "a que já está registada não pode ser re-lida, e a repetida conta UMA vez"
    );
}

/// ⭐ **Um arquivo que sumiu FALA — e fala UMA vez.**
///
/// ⚠️ As duas metades são o mesmo defeito visto dos dois lados. Sem a voz, o artista abre o projeto,
/// vê uma peça errada (ou peça nenhuma, se a escultura estava numa interseção) e conclui que o app
/// perdeu o trabalho dele. Sem o «uma vez», a reconciliação corre a cada quadro sobre um arquivo que
/// **não volta sozinho**: seriam 60 leituras falhadas e 60 avisos por segundo, e a tela ficaria
/// ilegível exactamente no caso em que ele precisa de a ler.
#[test]
fn a_sculpture_whose_file_vanished_speaks_once() {
    forget_tried();
    let key = "/tmp/ph2d-w23-esta-nao-existe.obj";
    assert!(
        !std::path::Path::new(key).exists(),
        "o fixture depende de o arquivo NÃO existir"
    );
    let doc = a_part_naming(key);

    let first = resolve_missing(&doc);
    assert_eq!(first.len(), 1, "a falta tem de virar exactamente um aviso");
    assert!(
        first[0].contains("ph2d-w23-esta-nao-existe.obj"),
        "e o aviso tem de NOMEAR o arquivo — o artista precisa de saber qual procurar; disse {:?}",
        first[0]
    );

    assert!(
        resolve_missing(&doc).is_empty(),
        "o segundo quadro não pode repetir o aviso nem a leitura — o arquivo não volta sozinho"
    );
}

/// ⭐ **A voz chega ao app pela PONTE** — a segunda metade da costura.
///
/// ⚠️ **Ela precisa de gate próprio porque o irmão acima não a alcança**: `a_sculpture_whose_file…`
/// chama `resolve_missing` de frente e mede o que ela devolve. Apagar a linha que **entrega** esse
/// resultado ao canal do app deixaria aquele gate verde e o artista mudo — o mesmo modo de falha da
/// wave inteira, uma emenda mais à frente.
#[test]
fn the_missing_file_reaches_the_artist_through_the_bridge() {
    forget_tried();
    crate::field3d_notice::clear();
    let _ = crate::field3d_notice::drain();
    let key = "/tmp/ph2d-w23-nunca-existiu.obj";
    let mut sim = SimWorld::new();
    let _ = crate::field3d_scene::sync_scene(&mut sim, Some(&a_part_naming(key)), 0.0);

    let report = crate::field3d_notice::drain();
    assert!(
        report.iter().any(|m| m.contains("nunca-existiu.obj")),
        "o cozimento tem de ENTREGAR o aviso ao app; entregou {report:?}"
    );
}

/// **O CUSTO das duas metades** — o que o regresso paga uma vez, e o que ele paga por quadro.
///
/// ⚠️ Não é gate: é a tabela que o doc 06 §24 cita. Corre com
/// `--ignored --nocapture`, e ⚠️ **só vale com a máquina calma** (CLAUDE.md §5.0).
#[test]
#[ignore = "medição, não gate — corre com --ignored --nocapture"]
fn measure_the_cost_of_coming_back() {
    println!("--- regenerar uma escultura do arquivo (o custo de ABRIR o projeto)");
    println!("triângulos | KB do .obj | ms (ler+merge+voxelizar)");
    for (rings, segs) in [(12, 24), (32, 64), (64, 128), (128, 256)] {
        let mesh = ph2d_mesh::shapes::uv_sphere(rings, segs, 1.0);
        let bytes = ph2d_mesh::write_obj(&[ph2d_mesh::ExportPiece {
            name: Some("blob"),
            mesh: &mesh,
            pose: ph2d_mesh::Pose::IDENTITY,
        }]);
        let path = std::env::temp_dir().join(format!("ph2d_w23_cost_{rings}.obj"));
        std::fs::write(&path, &bytes).expect("escreve");
        let t0 = std::time::Instant::now();
        let loaded = crate::field3d_import::field_from_file(&path).expect("lê");
        println!(
            "{:10} | {:10.0} | {:6.1}",
            loaded.tris,
            bytes.len() as f64 / 1024.0,
            t0.elapsed().as_secs_f64() * 1000.0
        );
        let _ = std::fs::remove_file(&path);
    }

    println!("\n--- a varredura POR QUADRO (o caso normal: nada em falta)");
    let reg = crate::field3d_smoke::sampled_registry();
    for n in [1usize, 8, 64, 512] {
        let nodes: Vec<Node> = (0..n)
            .map(|i| {
                Node::new(
                    Xform::IDENTITY,
                    NodeKind::Leaf(ph2d_field::Primitive::Sphere {
                        radius: 0.1 + i as f32 * 1e-4,
                    }),
                )
            })
            .chain(std::iter::once(Node::new(
                Xform::IDENTITY,
                NodeKind::Combine {
                    op: ph2d_field::Op::Union(ph2d_field::Blend::Sharp),
                    children: (0..n).map(|i| NodeId(i as u32)).collect(),
                },
            )))
            .collect();
        let doc = ph2d_field::FieldDoc::new(nodes, NodeId(n as u32)).expect("a peça");
        let t0 = std::time::Instant::now();
        const REPS: usize = 10_000;
        let mut acc = 0usize;
        for _ in 0..REPS {
            acc += missing_keys(&doc, &reg).len();
        }
        println!(
            "{n:4} nós | {:8.3} µs por varredura ({acc})",
            t0.elapsed().as_secs_f64() * 1e6 / REPS as f64
        );
    }
}

/// ⭐ **Um CAMINHO vira campo num sítio só** — e é isso que faz o que volta ser o que entrou.
///
/// ⚠️ **A divergência que este gate mata é silenciosa e permanente**: um segundo leitor — com outra
/// resolução, ou sem o `recenter` — daria uma peça que **muda de forma ao reabrir o projeto**, sem
/// uma linha de erro em lado nenhum. O documento guarda o caminho e não a grade, então *a função que
/// lê o arquivo é parte do formato*.
///
/// ⚠️ **A agulha é o LEITOR (`read_pieces`), não o voxelizador**, e a diferença apareceu na primeira
/// corrida deste gate: a cena 6 do smoke também chama `SampledField::from_mesh`, sobre uma malha que
/// ela **fabrica**. Essa não pode divergir do arquivo — não há arquivo. O que não pode existir duas
/// vezes é *ler um arquivo de malha e chamar-lhe escultura*.
///
/// Ele lê CÓDIGO: comentários ficam de fora (a lição do irmão
/// `every_field3d_test_file_is_declared_by_a_module`, que passava por se encontrar no próprio
/// doc-comment).
#[test]
fn only_one_place_turns_a_path_into_a_sculpture() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut hits: Vec<String> = Vec::new();
    for entry in std::fs::read_dir(&dir).expect("lê o diretório").flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with("field3d_") || !name.ends_with(".rs") || name.ends_with("_tests.rs") {
            continue;
        }
        let Ok(src) = std::fs::read_to_string(entry.path()) else {
            continue;
        };
        for line in src.lines().filter(|l| !l.trim_start().starts_with("//")) {
            if line.contains("read_pieces") {
                hits.push(name.clone());
            }
        }
    }
    assert_eq!(
        hits,
        vec!["field3d_import.rs".to_string()],
        "um arquivo de malha vira escultura num sítio só; estes o leem: {hits:?}"
    );
}

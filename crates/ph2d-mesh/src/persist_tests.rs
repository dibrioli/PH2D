//! Gates do documento.
//!
//! O oráculo de toda ida-e-volta é **o que o artista autorou** — nunca uma
//! segunda escrita da mesma serialização, que concordaria com ela exatamente
//! onde ela erra. E o do lado derivado é a **igualdade com uma malha construída
//! do zero**: se o load reconstrói, o resultado tem de ser indistinguível de uma
//! malha que nunca viajou.

use super::*;
use crate::{Multires, shapes};

/// Uma pilha de dois níveis com trabalho FINO no de cima — a fixture que
/// distingue *guardar a pilha* de *guardar a malha viva*.
fn stack_with_fine_work() -> Multires {
    let mut stack = Multires::new(shapes::octahedron(1.0));
    assert!(stack.add_level(), "a fixture precisa do 2º nível");
    // Uma protuberância que só existe no nível de cima.
    let mesh = stack.mesh_mut();
    mesh.positions_mut()[0][1] += 0.25;
    stack
}

/// A metade autorada atravessa o arquivo **intacta**, e a derivada é
/// reconstruída — o resultado é indistinguível de uma malha que nunca viajou.
#[test]
fn the_round_trip_keeps_what_was_authored_and_rebuilds_the_rest() {
    let mut src = shapes::uv_sphere(8, 12, 1.0);
    src.masks_mut()[3] = 0.75;
    src.colors_mut()[5] = [0.1, 0.2, 0.3];

    let back = Mesh::from_data(src.to_data()).expect("documento válido");

    assert_eq!(back.positions(), src.positions(), "posições");
    assert_eq!(back.faces(), src.faces(), "faces");
    assert_eq!(back.masks(), src.masks(), "máscara");
    assert_eq!(back.colors(), src.colors(), "cor");
    // E o derivado: igual ao de uma malha construída AGORA das mesmas partes.
    let fresh = Mesh::from_parts(src.positions().to_vec(), src.faces().to_vec())
        .expect("as partes são as mesmas");
    assert_eq!(back.normals(), fresh.normals(), "normais derivadas");
    assert_eq!(back.face_normals(), fresh.face_normals(), "normais de face");
    assert_eq!(back.bounds().min, fresh.bounds().min, "caixa");
    assert_eq!(back.bounds().max, fresh.bounds().max, "caixa");
}

/// **Uma malha que ninguém pintou volta sem o plano.** Materializá-lo no load
/// alocaria por-vértice um plano que o artista não pediu — e, pior, faria toda
/// malha importada nascer "pintada de branco" para quem pergunta ao `colors()`.
#[test]
fn an_unpainted_mesh_comes_back_without_the_plane() {
    let src = shapes::octahedron(1.0);
    assert!(
        src.colors().is_none() && src.masks().is_none(),
        "a premissa"
    );
    let back = Mesh::from_data(src.to_data()).expect("documento válido");
    assert!(back.colors().is_none(), "cor não pode ser materializada");
    assert!(back.masks().is_none(), "máscara não pode ser materializada");
}

/// **A PILHA viaja inteira** — e o oráculo é o trabalho fino, não a contagem de
/// níveis: um documento que guardasse só a malha viva devolveria uma pilha do
/// tamanho certo e com o detalhe APAGADO.
#[test]
fn the_stack_travels_whole_with_the_fine_work() {
    let src = stack_with_fine_work();
    let peak = src.mesh().positions()[0][1];

    let back = Multires::from_data(src.to_data()).expect("documento válido");

    assert_eq!(back.level_count(), src.level_count(), "níveis");
    assert_eq!(back.level(), src.level(), "o nível em mãos");
    assert!(
        (back.mesh().positions()[0][1] - peak).abs() < 1e-6,
        "o trabalho fino do nível de cima: {} contra {peak}",
        back.mesh().positions()[0][1]
    );
    // E o nível de BAIXO continua lá, com a forma dele.
    let low = back.level_mesh(0).expect("o nível 0 existe");
    assert_eq!(
        low.positions().len(),
        src.level_mesh(0).unwrap().positions().len(),
        "a base"
    );
}

/// **Salvei trabalhando na BASE; reabri e subi — o trabalho fino está lá.**
///
/// ⚠️ **Este gate nasceu VAZIO e foi a mutação que o pegou, DUAS vezes.** Ele
/// tem de ser lido junto com o mecanismo, que a própria crate documenta: o
/// `lower` **reescreve** `details[i]` a partir do nível vivo, então uma pilha
/// salva **no topo** reconstrói o detalhe sozinha na primeira descida — e
/// apagar o `details` na escrita passava nos sete gates. A janela em que o
/// `details` é a ÚNICA cópia do trabalho é esta: `sel == 0` com um nível acima,
/// onde `levels[1]` está velho e é `details[1]` que carrega a diferença. É por
/// isso que a fixture DESCE antes de salvar.
///
/// (As duas versões anteriores mediam o comprimento da base — um número que a
/// operação não pode mudar — e o pico do topo, que viaja em `levels`.)
#[test]
fn the_detail_survives_so_a_descent_still_reconstructs() {
    let mut src = stack_with_fine_work();
    let peak = src.mesh().positions()[0][1];
    src.lower().expect("dá para descer");
    assert_eq!(src.level(), 0, "a fixture salva TRABALHANDO na base");

    let mut back = Multires::from_data(src.to_data()).expect("documento válido");
    assert_eq!(back.level(), 0, "e reabre onde a mão estava");
    assert!(back.higher(), "sobe de volta");

    // ⚠️ **A barra não é bit-exatidão, e o número é MEDIDO.** Descer é a base
    // ABSORVER a parte baixa do trabalho; subir a devolve pela síntese, e a
    // ida-e-volta custa 1,8% (0,2455 contra os 0,25 autorados). Com o `details`
    // apagado a síntese devolve a predição LISA — 0,15625, ou 37,5% abaixo —,
    // então o fosso entre honesto e quebrado é de **20×** a barra.
    let back_peak = back.mesh().positions()[0][1];
    assert!(
        (back_peak - peak).abs() < 0.05 * peak,
        "o trabalho fino tem de sobreviver ao arquivo: {back_peak} contra o \
         autorado {peak} (a predição lisa, que é o que sobra sem o detalhe, dá 0,15625)"
    );
}

/// **A escala da pose é RE-CLAMPADA na leitura.** Um documento é entrada de
/// terceiro: escrever o número cru no campo privado deixaria o `point_to_local`
/// dividir por zero e todo pick devolver infinito.
#[test]
fn a_degenerate_scale_in_a_document_is_floored() {
    for bad in [0.0, -1.0, f32::NAN] {
        let pose = Pose::from_data(PoseData {
            translation: [1.0, 2.0, 3.0],
            scale: bad,
        });
        assert!(pose.scale() > 0.0, "escala {bad} virou {}", pose.scale());
        let q = pose.point_to_local([1.0, 1.0, 1.0]);
        assert!(q.iter().all(|c| c.is_finite()), "local não-finito: {q:?}");
    }
}

/// Todo documento malformado é **recusado**, e cada recusa nomeia o defeito —
/// nenhuma delas passa como geometria plausível.
#[test]
fn a_malformed_document_is_refused_with_its_reason() {
    let good = shapes::octahedron(1.0).to_data();

    // Plano de máscara com o tamanho errado.
    let mut bad = good.clone();
    bad.masks = Some(vec![0.0; 3]);
    assert!(
        matches!(
            Mesh::from_data(bad),
            Err(DocError::PlaneLen {
                plane: "máscara",
                ..
            })
        ),
        "plano de máscara curto tem de ser recusado"
    );

    // Plano de cor com o tamanho errado.
    let mut bad = good.clone();
    bad.colors = Some(vec![[0.0; 3]; 2]);
    assert!(
        matches!(
            Mesh::from_data(bad),
            Err(DocError::PlaneLen { plane: "cor", .. })
        ),
        "plano de cor curto tem de ser recusado"
    );

    // Índice de face fora de alcance — a MESMA validação que um OBJ de terceiro
    // atravessa.
    let mut bad = good.clone();
    bad.faces[0] = Face::tri(0, 1, 9_999);
    assert!(
        matches!(Mesh::from_data(bad), Err(DocError::Mesh(_))),
        "face fora de alcance tem de ser recusada"
    );

    // Pilha vazia.
    assert!(matches!(
        Multires::from_data(StackData {
            levels: vec![],
            details: vec![],
            sel: 0
        }),
        Err(DocError::EmptyStack)
    ));

    // Detalhes que não casam com os níveis.
    let stack = stack_with_fine_work().to_data();
    let mut bad = stack.clone();
    bad.details.pop();
    assert!(
        matches!(Multires::from_data(bad), Err(DocError::StackShape { .. })),
        "contagem de detalhes tem de casar com a de níveis"
    );

    // Nível selecionado fora de alcance.
    let mut bad = stack;
    bad.sel = 42;
    assert!(matches!(
        Multires::from_data(bad),
        Err(DocError::LevelOutOfRange { .. })
    ));
}

/// FNV-1a — hash local do gate, para ele ver **reordenação de campos**, que um
/// comprimento sozinho não vê (trocar dois `f32` de lugar não move um byte de
/// tamanho). Não é uma segunda resposta a nada: nenhum código de produto hasheia
/// documentos.
fn fnv1a(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// **A FORMA do que um documento guarda é PINADA.**
///
/// ⚠️ Este gate falhar não é um defeito — é o lembrete de que
/// **`SCULPT_DOC_VERSION` (no shell) tem de bumpar**. O postcard é POSICIONAL:
/// um campo novo lido por um binário velho não falha, devolve lixo bem-formado,
/// então a única defesa é a versão dentro do blob.
///
/// A fixture carrega valores DISTINGUÍVEIS de propósito: com tudo em zero,
/// trocar dois campos de lugar não moveria byte nenhum e o gate ficaria verde
/// sobre uma mudança de forma real.
#[test]
fn the_shape_of_a_saved_mesh_is_pinned() {
    fn pin<T: serde::Serialize>(label: &str, value: &T, expect: (usize, u64)) {
        let bytes = postcard::to_allocvec(value).expect("serializa");
        assert_eq!(
            (bytes.len(), fnv1a(&bytes)),
            expect,
            "a forma de {label} mudou -- bumpe o SCULPT_DOC_VERSION no shell"
        );
    }

    let mut mesh = shapes::octahedron(1.0);
    for (i, m) in mesh.masks_mut().iter_mut().enumerate() {
        *m = i as f32 * 0.125;
    }
    for (i, c) in mesh.colors_mut().iter_mut().enumerate() {
        *c = [i as f32 * 0.25, 0.5, 0.75];
    }
    pin(
        "MeshData",
        &mesh.to_data(),
        (238, 4_561_513_205_432_683_963),
    );

    let mut stack = Multires::new(shapes::cube(1.0));
    assert!(stack.add_level(), "a fixture precisa do 2º nível");
    stack.mesh_mut().positions_mut()[0][2] += 0.5;
    pin(
        "StackData/DetailData",
        &stack.to_data(),
        (857, 10_134_020_629_471_034_820),
    );

    pin(
        "PoseData",
        &Pose::new([1.0, 2.0, 3.0], 4.0).to_data(),
        (16, 10_352_097_795_870_096_280),
    );
}

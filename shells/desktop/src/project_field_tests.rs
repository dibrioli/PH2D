//! **O que um load faz com a PEÇA DE MODELAGEM 3D** (ADR-0161) — filho (`#[path]`) de
//! [`super`] (`project::tests`) pelo teto de LOC do HR-18, e FILHO e não irmão pela razão exata
//! do `sculpt`: as fixtures desta suíte (`headless_app`, `write_project_full`, `tmp_path`) são as
//! portas dele, e copiá-las seria um segundo escritor de arquivo de projeto.
//!
//! # ⚠️ O que já estava construído, e o que NÃO estava
//!
//! A peça em si **atravessa o arquivo sem uma linha de código de persistência** — ela é uma árvore
//! de entidades ECS, e o `ProjectState` é o mundo inteiro. O gate que o prova vive no módulo do
//! campo (`field3d_snapshot_tests::the_part_crosses_the_project_file_…`), incluindo a metade que
//! quase não se sustentava sozinha: a limpeza do `restore` consulta `With<Transform>`, e só a
//! **raiz** da peça o carrega.
//!
//! O que **não** atravessava é o que este arquivo mede: a memória de *"já tentei ler este arquivo
//! de escultura"* é do **PROCESSO**, e um Ctrl+O começa um **documento** novo.

use super::*;

/// Um documento que nomeia uma escultura que **não existe no disco**.
fn a_doc_naming(key: &str) -> ph2d_field::FieldDoc {
    ph2d_field::FieldDoc::new(
        vec![ph2d_field::Node {
            xform: ph2d_field::Xform::IDENTITY,
            kind: ph2d_field::NodeKind::Sampled {
                key: key.to_string(),
            },
            mods: Vec::new(),
        }],
        ph2d_field::NodeId(0),
    )
    .expect("um documento de uma escultura só")
}

/// ⭐ **UM CTRL+O RECOMEÇA AS TENTATIVAS DE LER AS ESCULTURAS.**
///
/// # O defeito, e ele é silencioso duas vezes
///
/// Quando o documento nomeia uma escultura que o registo não conhece, o módulo **lê o arquivo** e
/// diz o que não voltou ([`crate::field3d_reload::resolve_missing`], W23). Para não repetir o aviso
/// em todo quadro, ele guarda o que já tentou — e guardava-o **pelo processo**.
///
/// Consequência: o artista abre um projeto, a escultura falha (arquivo movido), ele **conserta o
/// arquivo no disco** e abre o projeto outra vez. A segunda leitura **nunca acontece**: o nome já
/// está no conjunto de tentados. A peça abre sem a escultura, e desta vez **sem uma palavra** —
/// porque o aviso também só sai na primeira tentativa.
///
/// ⚠️ *O limite daquela memória é o DOCUMENTO, não o processo.* Um Ctrl+O é o começo de um
/// documento novo, e é ele que a repõe.
#[test]
fn a_load_starts_the_sculpture_reads_over() {
    let mut app = headless_app();
    crate::field3d_reload::forget_tried();

    // 1. A primeira tentativa FALHA e é reportada — o comportamento da W23, intacto.
    let doc = a_doc_naming("/tmp/ph2d_gate_nao_existe_nunca.obj");
    let first = crate::field3d_reload::resolve_missing(&doc);
    assert_eq!(
        first.len(),
        1,
        "a primeira tentativa tem de falhar E dizer — é a W23"
    );

    // 2. A segunda é MUDA de propósito: é o que impede o aviso de repetir em todo quadro.
    assert!(
        crate::field3d_reload::resolve_missing(&doc).is_empty(),
        "dentro do mesmo documento, a segunda tentativa não repete o aviso"
    );

    // 3. ⭐ **O CTRL+O** — um documento novo. O arquivo é válido e vazio: o que se mede aqui é o
    //    efeito do load sobre a memória de tentativas, não o conteúdo dele.
    let path = tmp_path("field_load_forgets");
    write_project_full(&path, PROJECT_SCHEMA, Vec::new(), Vec::new());
    app.project_load_from(&path.to_string_lossy());
    let _ = std::fs::remove_file(&path);

    // 4. A tentativa recomeça — e volta a dizer o que não voltou.
    assert_eq!(
        crate::field3d_reload::resolve_missing(&doc).len(),
        1,
        "depois de um Ctrl+O a leitura das esculturas tem de RECOMEÇAR — senão um arquivo \
         consertado no disco nunca é relido, e o silêncio é o mesmo de quando estava certo"
    );
}

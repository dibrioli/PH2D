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
            verb: None,
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

// ─── W45: um projeto que traz uma PEÇA abre o painel dela ───

/// Uma peça de verdade: uma união de duas esferas.
fn a_part_doc() -> ph2d_field::FieldDoc {
    use ph2d_field::{Blend, Node, NodeId, NodeKind, Op, Primitive, Xform};
    let ball = |x: f32| Node {
        xform: Xform::at(x, 0.0, 0.0),
        kind: NodeKind::Leaf(Primitive::Sphere { radius: 0.2 }),
        mods: Vec::new(),
        verb: None,
    };
    ph2d_field::FieldDoc::new(
        vec![
            ball(0.0),
            ball(0.6),
            Node {
                xform: Xform::IDENTITY,
                kind: NodeKind::Combine {
                    op: Op::Union(Blend::Sharp),
                    children: vec![NodeId(0), NodeId(1)],
                },
                mods: Vec::new(),
                verb: None,
            },
        ],
        NodeId(2),
    )
    .expect("a união")
}

/// ⭐ **O FATO que o quadro pergunta: «há alguma coisa PARA VER?»**
///
/// ⚠️ **O terceiro caso é o que dá sentido à função, e ele veio de uma MUTAÇÃO SOBREVIVENTE.** As
/// duas primeiras versões perguntavam *"há uma raiz"* e *"há um nó"* — e o `spawn_doc` dá `FieldNode`
/// à raiz **sempre**, então as duas são a mesma pergunta. Com só os dois primeiros casos, trocar uma
/// pela outra passava. *Uma fixtura que concorda não prova nada; foi a mutação que exigiu a peça
/// ESVAZIADA, e foi ela que corrigiu a pergunta.*
#[test]
fn a_world_has_a_part_only_when_there_is_something_to_see() {
    let mut empty = ph2d_ecs::SimWorld::new();
    assert!(
        !crate::field3d_scene::world_has_a_part(empty.world_mut()),
        "um mundo sem peça nenhuma não pode abrir o painel de modelagem"
    );

    let mut sim = ph2d_ecs::SimWorld::new();
    let root = ph2d_field_ecs::spawn_doc(sim.world_mut(), &a_part_doc(), "peça");
    assert!(
        crate::field3d_scene::world_has_a_part(sim.world_mut()),
        "um mundo com uma peça tem de responder que sim"
    );

    // ⭐ **A peça ESVAZIADA** — apagar os filhos é um gesto normal, e deixa a raiz de pé. Salvar
    // nesse estado e reabrir não pode ocupar o encaixe da direita para não mostrar nada.
    let leaves: Vec<bevy_ecs::entity::Entity> = ph2d_field_ecs::walk(sim.world(), root)
        .into_iter()
        .map(|(e, _)| e)
        .filter(|e| *e != root)
        .collect();
    assert_eq!(leaves.len(), 2, "o controle: a fixtura tem duas folhas");
    for leaf in leaves {
        assert!(
            ph2d_field_ecs::remove(sim.world_mut(), leaf),
            "a folha some"
        );
    }
    assert!(
        !crate::field3d_scene::world_has_a_part(sim.world_mut()),
        "com a raiz vazia não há geometria nenhuma — e a raiz continua de pé, que é o caso em que \
         «há raiz» e «há nó» respondem SIM e a tela fica vazia"
    );
}

/// ⭐⭐ **A PORTA ESTAVA TRANCADA POR DENTRO.**
///
/// # O defeito
///
/// A peça atravessa o arquivo sozinha (W35) — mas nada a mostrava: reabrir o projeto trazia a obra
/// para o mundo e a tela ficava **vazia**, com o painel fechado. E o pedido de abrir o painel só era
/// aceite **com o módulo já armado**, enquanto o único caminho que o arma é a visibilidade do
/// painel. *Para pedir a abertura era preciso já estar aberto.*
///
/// ⚠️ Este gate mede as **duas** metades: que o load deixa a pergunta, e que o pedido explícito
/// atravessa a guarda do armado — que é a linha que estava a faltar.
#[test]
fn a_loaded_project_asks_to_open_the_panel_even_with_the_module_disarmed() {
    let mut app = headless_app();
    crate::field3d_smoke::forget_open_panel_request();
    crate::field3d_smoke::set_armed_by_panel(false);
    let _ = crate::field3d_smoke::with_smoke(|_| ());

    let path = tmp_path("field_load_opens");
    write_project_full(&path, PROJECT_SCHEMA, Vec::new(), Vec::new());
    app.project_load_from(&path.to_string_lossy());
    let _ = std::fs::remove_file(&path);

    assert!(
        crate::field3d_smoke::take_open_if_part_request(),
        "um load tem de deixar a PERGUNTA — o mundo vive no `gfx` e este caminho corre sem janela"
    );

    // O quadro responde-a: há peça ⇒ pede a abertura.
    let mut sim = ph2d_ecs::SimWorld::new();
    ph2d_field_ecs::spawn_doc(sim.world_mut(), &a_part_doc(), "peça");
    if crate::field3d_scene::world_has_a_part(sim.world_mut()) {
        crate::field3d_smoke::ask_open_panel();
    }
    assert!(
        crate::field3d_smoke::take_open_panel_request(),
        "⭐ o pedido EXPLÍCITO tem de atravessar a guarda do armado — senão a única porta que abre \
         o painel exige que ele já esteja aberto, e a peça do arquivo nunca aparece"
    );
    crate::field3d_smoke::forget_open_panel_request();
}

/// ⚠️ **E o controle: sem peça, ninguém abre nada.** Um painel que se abrisse em todo Ctrl+O
/// ocuparia o encaixe da direita para não mostrar coisa nenhuma — a razão pela qual a guarda do
/// armado existia, e que esta wave tem de preservar.
#[test]
fn a_project_without_a_part_opens_nothing() {
    crate::field3d_smoke::forget_open_panel_request();
    crate::field3d_smoke::set_armed_by_panel(false);
    let _ = crate::field3d_smoke::with_smoke(|_| ());

    let mut empty = ph2d_ecs::SimWorld::new();
    crate::field3d_smoke::ask_open_panel_if_part();
    if crate::field3d_smoke::take_open_if_part_request()
        && crate::field3d_scene::world_has_a_part(empty.world_mut())
    {
        crate::field3d_smoke::ask_open_panel();
    }
    assert!(
        !crate::field3d_smoke::take_open_panel_request(),
        "sem peça no mundo, um load não pode abrir o painel de modelagem"
    );
    crate::field3d_smoke::forget_open_panel_request();
}

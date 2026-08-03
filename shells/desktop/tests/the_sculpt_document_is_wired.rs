//! **Arch-gates do DOCUMENTO e da PORTA DE ENTRADA** (ADR-0150 W8.3 e W8.4) —
//! como uma escultura ENTRA na sessão, venha ela de um projeto ou de um arquivo
//! soltado.
//!
//! Os gates de unidade de `sculpt3d::doc`, `sculpt3d::import` e `project::tests`
//! cobrem o que é dirigível **sem janela**: o par escrita↔leitura, as três
//! recusas, o que o load estaciona e onde cada peça de um arquivo pousa. O que
//! sobra aqui é o que nenhum deles alcança — **ordem** de chamadas e
//! **presença** de uma linha de fiação, que num `App` headless nem roda porque
//! `gfx` é `None`.
//!
//! É o mesmo padrão (e o mesmo motivo) do
//! `the_load_installs_the_world_settings_after_the_rebuild`: quando o fato é a
//! ordem do código do produto, o gate lê o código do produto.

mod sculpt_source;
use sculpt_source::{function_body, sculpt_src, source};

/// **A RECUSA vem antes de qualquer mutação da sessão.**
///
/// ⚠️ O gate de unidade mede a CONSEQUÊNCIA (relógio e histórico intactos), e
/// ele é verdadeiro por acidente enquanto a mutação seguinte não observar nada
/// que ele meça. A ordem é a causa, e é ela que fica pinada aqui: mover o parse
/// para depois do `forget_live_producers` faz um arquivo recusado **derrubar a
/// sessão mesmo assim** — a obra do artista morre por um documento que o app
/// nem chegou a aceitar.
#[test]
fn the_refusal_precedes_every_mutation_of_the_session() {
    let body = function_body(&source("project_load.rs"), "project_load_from");
    let parse = body
        .find("decode_doc(")
        .expect("o load precisa DECODIFICAR a escultura antes de aceitar o arquivo");
    let mutate = body
        .find("forget_live_producers()")
        .expect("o load precisa esquecer o documento anterior");
    assert!(
        parse < mutate,
        "o parse da escultura aparece DEPOIS do `forget_live_producers` — um \
         arquivo recusado passaria a derrubar a sessao que continua aberta"
    );
}

/// **O que o load estacionou, o FRAME instala.**
///
/// ⚠️ Sem esta chamada o produto fica exatamente na forma que mais engana:
/// o load aceita o arquivo, guarda a escultura, **nada aparece na tela**, e o
/// próximo Ctrl+S grava a cena viva (vazia) por cima. Todos os gates de unidade
/// ficam verdes, porque a pendência de fato foi preenchida.
#[test]
fn the_frame_installs_what_the_load_parked() {
    let src = source("render_loop/mod.rs");
    assert!(
        src.contains("self.sculpt3d_install_pending();"),
        "o frame precisa CHAMAR o instalador — a cena 3D nao nasce sem device, \
         entao o load so consegue estacionar"
    );
}

/// **Uma cena instalada não herda a fila de desfazer do documento anterior.**
///
/// ⚠️ Toda entrada nomeia a peça por `ObjectId`, e as peças do arquivo são
/// outras: desfazer através de um load aplicaria o inverso de um traço a barro
/// que nunca o recebeu. É a mesma lei que o load de projeto já aplica ao undo
/// global — e ela vive numa linha só, que some sem quebrar nada visível.
#[test]
fn installing_a_document_forgets_the_undo_queue() {
    let body = function_body(&sculpt_src(), "install_doc");
    assert!(
        body.contains("forget_history()"),
        "instalar um documento tem de matar a fila de desfazer da sessao anterior"
    );
    assert!(
        body.contains("mint_id()"),
        "…e re-cunhar os ids: um id reciclado e o defeito que o `ObjectId` existe para nao ter"
    );
}

/// **O escritor passa pela porta única.**
///
/// ⚠️ `encode` foi extraída para que o round-trip escrita↔leitura seja dirigível
/// sem GPU. Se o método voltar a montar o `SculptDoc` por conta própria, a porta
/// continua lá, o gate de unidade continua verde — e passa a provar um caminho
/// que o produto não usa, que é a armadilha do ADR-0120 nesta linha.
#[test]
fn the_writer_goes_through_the_one_encoder() {
    let body = function_body(&sculpt_src(), "to_doc_bytes");
    assert!(
        body.contains("encode("),
        "o escritor tem de passar pelo `encode` — e nao montar o documento de novo"
    );
    assert!(
        !body.contains("SculptDoc {"),
        "…e nao montar o `SculptDoc` por conta propria: o gate do round-trip \
         passaria a provar um caminho que o produto nao percorre"
    );
}

/// **O save prefere a cena VIVA e cai nos bytes quando não há uma.**
///
/// ⚠️ As duas metades importam e falham diferente: sem a primeira, um Ctrl+S
/// depois de esculpir grava o documento com que o projeto foi ABERTO (o trabalho
/// da sessão some); sem a segunda, um build sem o módulo — ou um projeto aberto
/// antes de a GPU aparecer — grava **vazio** por cima da escultura, e aí ele não
/// é passa-adiante, é triturador.
#[test]
fn the_save_prefers_the_live_scene_and_falls_back_to_the_bytes() {
    let body = function_body(&source("project.rs"), "sculpt_bytes_for_save");
    let live = body
        .find("to_doc_bytes()")
        .expect("o save le a cena VIVA quando ela existe");
    let stashed = body
        .find("self.sculpt_doc")
        .expect("…e devolve os bytes do arquivo quando nao existe");
    assert!(
        live < stashed,
        "a cena viva tem de ser consultada PRIMEIRO: o fallback nao pode vencer \
         o trabalho desta sessao"
    );
}

/// **Uma malha soltada sai da fila ANTES do filtro de imagem.**
///
/// ⚠️ A ordem é a asserção inteira, e o modo de falha dela é o pior possível: o
/// filtro emite um toast *"Skipped non-image"* por arquivo que não reconhece,
/// então um `.obj` que chegasse lá produziria **um aviso de que foi ignorado** —
/// a resposta errada, com a confiança da resposta certa. Nenhum teste de unidade
/// alcança isto: `handle_dropped_files` sai no `gfx.is_none()` de um `App` sem
/// janela, que é o único que um gate headless tem.
#[test]
fn a_dropped_mesh_leaves_the_queue_before_the_image_filter() {
    let body = function_body(&source("input_drop.rs"), "handle_dropped_files");
    let claim = body
        .find("is_mesh_file")
        .expect("o drop precisa reconhecer um arquivo de malha");
    let skip = body
        .find("Skipped non-image")
        .expect("o filtro de imagem continua avisando o que ele pula");
    assert!(
        claim < skip,
        "as malhas saem DEPOIS do filtro de imagem: soltar um .obj avisaria \
         que ele foi ignorado, e depois o importaria"
    );
    assert!(
        body.contains("sculpt3d_import_files"),
        "…e o desvio tem de CHAMAR o import, nao so' reconhecer a extensao"
    );
}

/// **Soltar uma malha ARMA o módulo** — a mesma lei do load de projeto (W8.3).
///
/// ⚠️ Sem isto o artista solta um modelo e não acontece nada, com o app sabendo
/// ler o arquivo: a cena 3D só existiria sob a variável do smoke, que é uma
/// porta de desenvolvimento, não um gesto.
#[test]
fn dropping_a_mesh_arms_the_module() {
    let body = function_body(&sculpt_src(), "sculpt3d_import_files");
    assert!(
        body.contains("Sculpt3dScene::new(") && body.contains("gfx.sculpt3d = Some(scene)"),
        "sem cena, o import tem de CRIAR uma"
    );
}

/// **A colocação roda sobre a lista inteira, ANTES de qualquer peça entrar.**
///
/// ⚠️ O caso que isto protege é o mais comum de todos — um arquivo, uma peça,
/// nenhuma cena aberta —, e ali a peça que abre a cena é a única que passaria
/// sem centrar: o defeito que a wave paga, sobrevivendo justamente onde ninguém
/// olharia. Um gate de unidade não o vê porque `place` estaria correta.
#[test]
fn the_placement_runs_before_any_piece_enters_the_scene() {
    let body = function_body(&sculpt_src(), "sculpt3d_import_files");
    let placed = body.find("place(&mut loaded").expect("a colocação roda");
    let opened = body
        .find("Sculpt3dScene::new(")
        .expect("a cena pode nascer aqui");
    assert!(
        placed < opened,
        "a peça que abre a cena entraria SEM ser centrada — e o plano do \
         espelho dela ficaria fora do modelo"
    );
    assert!(
        body.contains("scene.set_pose(0,"),
        "…e a peça que abriu a cena tem de receber a pose que a colocação deu"
    );
}

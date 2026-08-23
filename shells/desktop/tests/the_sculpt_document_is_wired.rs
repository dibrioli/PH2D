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
use sculpt_source::{function_body, project_family_fn, sculpt_src, source};

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
    let body = project_family_fn("sculpt_bytes_for_save");
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
    // ⚠️ O literal MUDOU em 2026-08-23 e a claim NÃO: a mensagem dizia «Skipped non-image», e
    // isso virou mentira quando o `.ase` — que não é uma imagem — passou a ser importável
    // (`crate::import_router`). O que este gate afirma é a ORDEM, não a redacção.
    let skip = body
        .find("Skipped {name}")
        .expect("o roteador continua avisando o que ele pula");
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

/// **O gesto de diálogo termina na MESMA porta do drop.**
///
/// ⚠️ Duas maneiras de PEDIR, uma de FAZER. Se este método voltar a ler o
/// arquivo por conta própria, as duas passam a responder *onde a peça pousa* — e
/// a que diverge é sempre a que gate nenhum dirige, porque um diálogo nativo não
/// é dirigível sem alguém clicando nele.
#[test]
fn the_picker_goes_through_the_same_import_door_as_the_drop() {
    let body = function_body(&sculpt_src(), "sculpt3d_pick_and_import");
    assert!(
        body.contains("sculpt3d_import_files("),
        "o seletor tem de ENTREGAR ao import, e nao importar por conta propria"
    );
    assert!(
        !body.contains("import_obj"),
        "…e nao pode reabrir o arquivo: seria a segunda resposta a como uma malha entra"
    );
    assert!(
        body.contains("MESH_EXTS"),
        "o filtro do dialogo le a LISTA — um literal divergiria do roteador do drop \
         no dia em que o STL entrar"
    );
    assert!(
        function_body(&sculpt_src(), "is_mesh_file").contains("MESH_EXTS"),
        "…e o roteador do drop le a mesma lista, que e' o que os mantem de acordo"
    );
}

/// **O braço com `shift` vem ANTES do braço genérico.**
///
/// ⚠️ Este é o gate que de fato morde, e o modo de falha é silencioso: um `match`
/// escolhe o PRIMEIRO padrão que casa, e um `KeyCode::KeyO` sem guarda posto
/// acima deixa o braço com `if shift` **inalcançável** — sem que o compilador
/// diga nada, porque ele não consegue provar cobertura através de uma guarda.
/// O sintoma seria Ctrl+Shift+O carregando projeto: a resposta errada, com a
/// confiança da resposta certa.
///
/// ⚠️ E ele afirma as duas metades. Sem a guarda de `shift` o import comeria o
/// Ctrl+O de projeto inteiro, que é pior do que não ter o atalho.
#[test]
fn the_mesh_import_chord_is_reachable_and_does_not_eat_the_project_open() {
    // ⚠️ O bloco mudou de ARQUIVO quando o `keyboard.rs` cruzou o cap de
    // LOC, e o gate segue o FATO em vez do endereço: a propriedade que ele
    // afirma é sobre a ordem dos braços, não sobre em que arquivo eles moram.
    let body = function_body(&source("input_dispatch/keyboard_files.rs"), "file_chords");
    let guarded = body
        .find("KeyCode::KeyO if self.modifiers.shift_key()")
        .expect("o import de malha e' Ctrl+SHIFT+O — a guarda faz parte do atalho");
    let plain = body
        .find("self.project_load()")
        .expect("Ctrl+O continua carregando projeto");
    assert!(
        guarded < plain,
        "o braço com `shift` esta' DEPOIS do generico: ele nasce inalcancavel, e \
         Ctrl+Shift+O carregaria projeto em silencio"
    );
    assert!(
        body.contains("sculpt3d_pick_and_import()"),
        "…e o acorde tem de CHAMAR o seletor"
    );
}

/// **O que sai é o nível VIVO, e a POSE entra junto.**
///
/// ⚠️ As duas metades falham diferente e as duas são invisíveis num arquivo que
/// "abriu": sem `pose`, todas as peças saem **empilhadas na origem** — o defeito
/// espelho exato do que o import curou; sem o nível vivo, o artista que desceu
/// para trabalhar grosso recebe de volta os milhões de triângulos do topo.
#[test]
fn the_export_writes_the_live_level_through_the_pose() {
    let body = function_body(&sculpt_src(), "export_pieces");
    // ⚠️ **A asserção é sobre o CAMPO, não sobre o corpo**, e a diferença foi
    // medida: com `body.contains("o.stack.mesh()")` a mutação
    // `level_mesh(0).unwrap_or_else(|| o.stack.mesh())` **sobrevive** — ela
    // mantém a string como fallback e exporta a base. Presença de um texto no
    // corpo não é o mesmo que ele ser a FONTE do valor.
    assert!(
        body.contains("mesh: o.stack.mesh(),"),
        "o export tem de escrever o nível VIVO da pilha"
    );
    assert!(
        body.contains("pose: o.pose,"),
        "…e levar a pose junto, senão as peças saem empilhadas na origem"
    );
}

/// **A extensão desconhecida é RECUSADA, nunca dobrada num default.**
///
/// ⚠️ Um default calado escreveria um OBJ com o nome `.fbx`, e o primeiro
/// programa a abri-lo diria que **o arquivo** está corrompido — apontando para o
/// lugar errado, que é a forma de erro mais cara que existe.
#[test]
fn an_unknown_export_extension_is_refused_not_silently_defaulted() {
    let body = function_body(&sculpt_src(), "sculpt3d_export");
    assert!(
        body.contains("from_extension"),
        "a extensão é quem decide o formato"
    );
    assert!(
        !body.contains("unwrap_or(MeshFormat::") && !body.contains("unwrap_or_default()"),
        "uma extensão desconhecida não pode virar um formato por default"
    );
    assert!(
        body.contains("Unknown extension"),
        "…ela tem de ser NOMEADA ao artista"
    );
}

/// **O aviso do que se perde sai da MESMA tabela que o escritor consulta.**
///
/// ⚠️ Uma segunda lista aqui diria *"cor preservada"* sobre um STL no dia em que
/// alguém trocasse o escritor — e um aviso errado é pior que aviso nenhum,
/// porque o artista confia nele e só descobre no outro programa.
#[test]
fn the_loss_warning_reads_the_same_table_the_writer_does() {
    let body = function_body(&sculpt_src(), "lost_by");
    assert!(
        body.contains("keeps_colour()") && body.contains("keeps_pieces()"),
        "o aviso tem de PERGUNTAR ao formato, não repetir a tabela"
    );
    assert!(
        body.contains("\"mask\""),
        "a máscara não sobrevive a nenhum dos três, então é dita SEMPRE"
    );
}

/// **Cada peça importada grava a PRÓPRIA entrada de desfazer.**
///
/// ⚠️ **O doc desta função afirmava que o `push_object` grava undo, e era FALSO
/// — o Enio pegou no smoke:** ele só empurra na lista; quem grava são o
/// `add_primitive` e o `duplicate_active`, cada um chamando `record` por conta.
/// Um import trazido por engano ficava na cena para sempre.
///
/// ⚠️ E é `record_for`, não `record`: o `push_object` **não** torna a peça
/// ativa, então o `record` — que carimba o ATIVO — nomearia a peça errada, e o
/// desfazer removeria uma que o import não trouxe.
#[test]
fn every_imported_piece_records_its_own_undo_entry() {
    let body = function_body(&sculpt_src(), "push_placed");
    assert!(
        body.contains("record_for(") && body.contains("AddedObject"),
        "cada peça importada tem de gravar a entrada dela"
    );
    assert!(
        !body.contains("self.record("),
        "…e por `record_for`: o `record` carimba o ATIVO, que o import não move"
    );
}

/// **O diálogo oferece UM FILTRO POR FORMATO.**
///
/// ⚠️ Report do Enio: *"ao salvar em outro formato que não .obj a app coloca
/// .obj no final"*. Com um filtro único listando as três extensões, o diálogo
/// nativo COMPLETA o nome com a primeira delas — `volta.ply` virava
/// `volta.ply.obj`. Não é uma segunda porta para *"que formato é este?"*: quem
/// decide continua sendo a extensão do caminho final.
#[test]
fn the_save_dialog_offers_one_filter_per_format() {
    let body = function_body(&sculpt_src(), "sculpt3d_export");
    assert!(
        body.contains("for f in MeshFormat::ALL") && body.contains("add_filter("),
        "um filtro POR formato, senão o diálogo completa com a primeira extensão"
    );
    assert!(
        !body.contains("MeshFormat::ALL.map(MeshFormat::extension)"),
        "…e nunca um filtro único com as três extensões juntas"
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

/// **O APP ABRE NO MATERIAL QUE O RENDERIZADOR DECLARA** — e o número não é
/// escrito duas vezes.
///
/// ⚠️ **Ele nasce de um defeito de TRÊS respostas para uma pergunta** (report do
/// Enio, 2026-08-12): a shell abria em `matcap: None` — o rig do documento —,
/// o `Shade::default()` do renderizador dizia `Some(0)` (o `Skin Haz 2`, com
/// gate a pinar o nome) e o `sculpt3d_announce` anunciava, na tela, *"o app abre
/// no 'Skin Haz 2'"*. Quem ganhava era a shell, porque é ela que passa o campo
/// ao `Shade` — ou seja **a resposta que ninguém tinha escrito de propósito**, e
/// o anúncio sobreviveu ao fato.
///
/// ⚠️ **O gate lê o CÓDIGO e não o valor, e é a única coisa que ele pode ler:**
/// construir uma cena exige um `wgpu::Device`, e o defeito é a shell ter um
/// literal PRÓPRIO. Um teste que comparasse dois números não distinguiria
/// *"delega"* de *"copiou o mesmo número"* — e é a cópia que apodrece no dia em
/// que o default do renderizador mudar.
///
/// ⚠️ **E ele lê o CLUSTER inteiro, não `sculpt3d_birth.rs`** — a prosa do
/// [`sculpt_src`] já mede que nomear o arquivo de uma função vira vermelho no
/// próximo split, sobre produto correto. A propriedade é *a fiação delega*, e
/// ela sobrevive ao arquivo mudar de nome.
#[test]
fn the_scene_opens_on_the_material_the_renderer_declares() {
    assert!(
        sculpt_src().contains("matcap: ph2d_mesh_render::DEFAULT_MATCAP"),
        "o nascimento da cena tem de DELEGAR o material de abertura ao \
         renderizador; um literal aqui e' a segunda resposta que diverge"
    );
}

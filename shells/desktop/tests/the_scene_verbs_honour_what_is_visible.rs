//! **Arch-gates da FUSÃO e do ISOLAMENTO** (ADR-0150, W8.7 — o que faltava).
//!
//! Irmão dos outros três gates de escultura, e o corte é o mesmo: *o que a mão
//! pede* (`_gesture_`), *o que o documento faz com a malha* (`_mesh_edits_`), *o
//! que o arquivo guarda* (`_document_`) e — aqui — **quem está à vista**.
//!
//! ⚠️ Estes gates leem a FONTE porque uma [`Sculpt3dScene`] não nasce sem um
//! `wgpu::Device`, e o que eles afirmam não é geometria: é *quem pergunta a
//! quem*. A metade que É geometria mora nos gates de unidade — a fusão em
//! `ph2d-mesh` (`merge_tests.rs`) e a tabela de slots em
//! `sculpt3d_slots_tests.rs` —, e as duas metades são necessárias: a lei pode
//! estar certa e não ser consultada.

mod sculpt_source;
use sculpt_source::{function_body, sculpt_src};

/// **O PINCEL NÃO ALCANÇA O QUE NÃO SE VÊ.**
///
/// ⚠️ As duas metades, e nenhuma basta: o [`pick`] escolhe a peça no pen-down (e
/// sem ele o artista isolaria uma peça e o clique seguinte pegaria outra, atrás
/// dela), e o [`pick_active`] responde a cada dab (e sem ele um Ctrl+Z que leve
/// o ativo a uma peça escondida deixaria a mão esculpindo algo invisível).
#[test]
fn the_brush_cannot_reach_a_piece_that_is_not_on_screen() {
    let src = sculpt_src();

    let pick = function_body(&src, "pick(&self");
    assert!(
        pick.contains("visible_pieces()"),
        "o pen-down escolhe entre as peças À VISTA, não entre todas"
    );
    assert!(
        !pick.contains("self.objects.iter().enumerate()"),
        "varrer a lista inteira faz o raio alcançar o que o isolamento escondeu"
    );

    let active = function_body(&src, "pick_active");
    assert!(
        active.contains("isolated_index()"),
        "um dab na peça ativa tem de recusar quando ela está escondida"
    );
}

/// **A CÂMERA ENQUADRA O QUE ESTÁ NA TELA.**
///
/// ⚠️ O `world_bounds` tem dois consumidores e os dois querem a mesma coisa:
/// enquadrar (`frame_all`) e dimensionar a peça nova (`add_primitive`, que a usa
/// como régua). Com uma peça isolada, uma caixa que incluísse as escondidas
/// afastaria a câmera para caber o que ninguém vê — e a peça nova nasceria do
/// tamanho de uma cena que não está lá.
#[test]
fn the_camera_frames_what_is_on_screen() {
    let bounds = function_body(&sculpt_src(), "world_bounds");
    assert!(
        bounds.contains("visible_pieces()"),
        "a caixa é a do que se vê"
    );
}

/// **O SYNC PERGUNTA À TESTEMUNHA, e escreve nela.**
///
/// ⚠️ É o gate do bug que esta wave achou: enquanto a pergunta era *"eu já
/// mandei esta malha?"*, um slot que trocou de dono (um delete que desloca, um
/// isolamento que compacta) ficava com a geometria da peça anterior — desenhada
/// na pose da nova, sem erro nenhum. As duas metades são necessárias: quem só
/// PERGUNTA nunca vê a tabela mudar, e quem só ESCREVE responde com o que
/// escreveu.
#[test]
fn the_sync_asks_the_slot_who_lives_there_and_writes_it_back() {
    let sync = function_body(&sculpt_src(), "sync_mesh");
    assert!(
        sync.contains("slot_plan()"),
        "a decisão é do plano, e é ela que compara o dono do slot"
    );
    assert!(
        sync.contains("self.slots.push(id)") || sync.contains("self.slots[k] = id"),
        "o slot tem de passar a dizer de quem ele é, ou a próxima pergunta mente"
    );
    assert!(
        sync.contains("truncate_objects"),
        "e o device não pode continuar desenhando peça que saiu da vista"
    );
}

/// **ISOLAR NÃO ENTRA NA HISTÓRIA.**
///
/// ⚠️ Isolar não move um vértice: é estado de VISTA, o mesmo lugar em que o
/// onion da timeline mora. Uma entrada de undo aqui gastaria um Ctrl+Z para
/// devolver pixels e nenhum trabalho — e, pior, empurraria para longe o passo
/// que o artista de fato quer desfazer.
#[test]
fn isolating_is_a_view_and_never_an_undo_step() {
    let body = function_body(&sculpt_src(), "toggle_isolate");
    assert!(
        !body.contains("record"),
        "isolar não é uma edição: não há trabalho a devolver"
    );
    assert!(
        body.contains("self.isolated"),
        "e o estado é o id da peça, não uma bandeira por peça"
    );
}

/// **A FUSÃO GRAVA O UNDO COM O ID DA PEÇA QUE ELA CRIOU.**
///
/// ⚠️ O `record` normal carimba **o ativo**, e aqui isso bastaria por acidente
/// (a fusão torna a peça nova ativa). O `record_for` explícito é o que mantém a
/// entrada correta no dia em que a ordem dos passos mudar — e é a mesma porta
/// que o delete usa pelo motivo espelho: ele grava o id da peça que SAIU.
#[test]
fn the_merge_records_against_the_piece_it_created() {
    let body = function_body(&sculpt_src(), "merge_visible");
    assert!(
        body.contains("record_for(id, StrokeUndo::Merged"),
        "a entrada nomeia a peça fundida, e é ela que o Ctrl+Z remove"
    );
    // As duas recusas são NOMEADAS, não colapsadas num `None`: o conselho de uma
    // (ponha mais peças) é o oposto do da outra (reverta os níveis).
    assert!(
        body.contains("Merge::Nothing") && body.contains("Merge::Stack"),
        "as duas recusas continuam distinguíveis para o log"
    );
}

/// **A FUSÃO AGE SOBRE O QUE ESTÁ À VISTA** — e o refazer usa a MESMA porta.
///
/// ⚠️ A segunda metade é o que impede o redo de divergir: se ele fundisse *todas
/// as peças* enquanto o gesto funde *as visíveis*, um Ctrl+Shift+Z sobre uma
/// cena isolada produziria uma peça que o gesto original nunca teria produzido.
///
/// ⚠️ **A primeira versão deste gate ancorava em `StrokeUndo::Unmerged =>` e
/// lia o lugar errado**: aquele texto aparece DUAS vezes no `apply_entry` — a
/// primeira no `match` que escolhe o nível, num braço vazio (`=> {}`) que agrupa
/// os quatro verbos de lista. O `braced_block` acha a primeira, e um gate que lê
/// um bloco vazio afirma o que quiser. A cura não foi uma âncora mais esperta:
/// foi a porta única (`fuse_visible`), que dá ao gate um nome que só existe uma
/// vez — e que o produto passou a ter porque as duas rotas de fato faziam a
/// mesma pergunta duas vezes.
#[test]
fn the_merge_and_its_redo_ask_the_same_question() {
    let src = sculpt_src();
    let door = function_body(&src, "fuse_visible");
    assert!(
        door.contains("visible_pieces()"),
        "a porta funde o que se vê"
    );
    let apply = function_body(&src, "apply_entry");
    assert!(
        apply.contains("self.fuse_visible(object)"),
        "refazer passa pela MESMA porta, com o id que a entrada nomeia — \
         um id novo deixaria a inversa órfã"
    );
    let merge = function_body(&src, "merge_visible");
    assert!(
        merge.contains("fuse_visible(id)"),
        "e o gesto também: duas cópias da pergunta divergiriam numa cena isolada"
    );
}

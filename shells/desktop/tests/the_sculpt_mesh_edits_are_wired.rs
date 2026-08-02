//! **Arch-gates do que o gesto FAZ COM A MALHA** (ADR-0150 W2/W6).
//!
//! Irmão do `the_sculpt_gesture_is_wired`, e o corte é de responsabilidade: lá
//! *o que a mão pede* (ponteiro, grips, teclas, câmera), aqui *o que o documento
//! faz com o pedido* (a janela que sobe à GPU, as entradas de undo, as operações
//! de máscara, a mudança de topologia). Os dois leem a fonte pelo mesmo
//! [`sculpt_source`], porque duas cópias dos helpers divergiriam.

mod sculpt_source;
use sculpt_source::{braced_block, function_body, sculpt_src};

#[test]
fn the_gpu_is_handed_the_window_that_answers_for_every_channel() {
    // ⚠️ Este gate existe porque o defeito É este, e um gate de GPU o pegou: o
    // conjunto que muda de NORMAL é maior que o que se MOVE — um vizinho parado
    // ao lado de uma face que girou tem a normal mudada. Subir só os movidos
    // deixa a malha iluminada por normais velhas numa faixa de um anel de
    // largura, bem na BORDA do pincel.
    //
    // ⚠️ **E a pergunta certa mudou de nome na W4.2, sobre o mesmo defeito.**
    // `last_refreshed` responde *de quem eu recomputei a normal* — que é VAZIO
    // num traço de máscara, porque máscara não move geometria. A janela que o
    // upload quer é *o que a GPU precisa RE-LER, em qualquer canal*, e ela tem
    // porta própria. Perguntar a antiga deixaria a máscara invisível no device
    // com todos os gates de CPU verdes.
    let body = function_body(&sculpt_src(), "sculpt_at");
    assert!(
        body.contains("last_gpu_dirty()"),
        "a janela do upload é a que responde por TODOS os canais"
    );
    assert!(
        !body.contains("last_moved()"),
        "`last_moved` é a janela do UNDO, não a da GPU"
    );
    assert!(
        !body.contains("last_refreshed()"),
        "`last_refreshed` é só o canal das NORMAIS: ele é vazio num traço de máscara"
    );
}

#[test]
fn releasing_the_button_turns_the_stroke_into_an_undo_entry() {
    let src = sculpt_src();
    let up = function_body(&src, "sculpt3d_pointer_up");
    assert!(
        up.contains("Drag::Sculpt") && up.contains("close_stroke()"),
        "soltar um traço tem de fechá-lo; sem isso o Ctrl+Z não tem o que desfazer"
    );
    let close = function_body(&src, "close_stroke");
    // O undo NÃO é um segundo sistema: a lei do traço já congela o `pre` por
    // vértice tocado, e a lista de tocados É a janela.
    assert!(
        close.contains("stroke.touched()") && close.contains("stroke.base_positions()"),
        "a entrada de undo sai do traço, não de uma segunda captura"
    );
}

#[test]
fn undoing_rebuilds_the_index_instead_of_refitting_the_way_back() {
    // Um refit sobre a VOLTA só cresce as caixas frouxas (ele nunca as encolhe
    // abaixo do que já viu), então cada Ctrl+Z deixaria a árvore um pouco mais
    // gorda e a consulta um pouco mais lenta, para sempre. Um undo é
    // user-paced — é o lugar certo para pagar a resposta exata.
    let body = function_body(&sculpt_src(), "undo_stroke");
    assert!(
        body.contains("self.mesh_mut().rebuild()"),
        "desfazer tem de reconstruir o índice"
    );
    assert!(
        !body.contains("refresh_region"),
        "o caminho incremental não serve para a volta"
    );
}

#[test]
fn the_four_mask_operations_have_a_gesture_and_an_undo() {
    // ⚠️ Uma operação construída que o artista não alcança é a mesma coisa que
    // ela não existir — e é literalmente o defeito que a W4.2 fecha (a máscara
    // existia desde a W2 e era invisível).
    let src = sculpt_src();
    let key = function_body(&src, "sculpt3d_key");
    for (code, op) in [
        ("KeyC", "MaskOp::Clear"),
        ("KeyI", "MaskOp::Invert"),
        ("KeyB", "MaskOp::Blur"),
        ("KeyN", "MaskOp::Sharpen"),
    ] {
        assert!(
            key.contains(&format!("K::{code} => Some({op})")),
            "`{op}` precisa de uma tecla"
        );
    }
    assert!(
        key.contains("scene.mask_op(op)"),
        "e a tecla tem de chegar à porta"
    );

    // ⚠️ **As quatro NÃO são verbos.** Um verbo é uma ferramenta que fica na mão;
    // estas executam e acabam. Enfiá-las na lista de números faria "escolher a
    // máscara" e "borrar a máscara" parecerem o mesmo tipo de gesto.
    // ⚠️ O `braced_block` conta CHAVES e a lista de verbos é um ARRAY, então
    // apontá-lo aqui lê o bloco seguinte — foi o que esta asserção fez na
    // primeira versão, e ela falhou descrevendo outra coisa.
    let at = src.find("const BY_NUMBER").expect("a lista de verbos");
    let by_number = &src[at..at + src[at..].find("];").expect("a lista fecha")];
    assert!(
        !by_number.contains("MaskOp"),
        "as operações de máscara não são verbos: elas executam e acabam"
    );
    assert_eq!(
        by_number.matches("Verb::").count(),
        10,
        "a lista de números é dos DEZ verbos"
    );

    let undo = function_body(&src, "undo_stroke");
    // ⚠️ **A âncora é o CASO, não a bandeira.** Isto dizia `entry.whole_mask` e
    // ficou vermelho sobre produto correto no dia em que a terceira forma de
    // entrada (a topologia) trocou o bool por um enum — o sexto proxy a expirar
    // nesta linha. O que o gate quer dizer é *desfazer distingue a janela de um
    // traço do plano INTEIRO*, e isso é o braço.
    assert!(
        undo.contains("StrokeUndo::Mask {") && undo.contains("StrokeUndo::Stroke"),
        "desfazer tem de distinguir a janela de um traço da máscara INTEIRA"
    );
    // ⚠️ E `None` quer dizer *não havia máscara*, que se desfaz REMOVENDO o
    // plano. Zerá-lo deixaria a malha pagando 4 B/vértice por um estado que ela
    // não tinha — e desfazer um `Invert` sobre malha virgem deixaria tudo
    // protegido para sempre.
    assert!(
        undo.contains("take_masks()"),
        "o caso `None` tem de REMOVER o plano, não zerá-lo"
    );
}

/// ⚠️ **Com a PILHA, o desfazer de uma subdivisão deixou de guardar a malha.**
///
/// A entrada anterior era `Topology(Box<Mesh>)` — uma cópia do documento — e a
/// ORDEM importava (guardar antes de trocar; guardar depois teria salvo a malha
/// NOVA, um undo que não desfaz nada com todos os gates de contagem verdes).
/// Com a pilha, o nível de baixo **nunca foi tocado**: ele continua ali, e
/// desfazer é descartar o topo. Não há o que guardar, então não há ordem a
/// errar — *a representação apagou o caso especial*.
#[test]
fn undoing_a_subdivision_drops_the_top_level_instead_of_restoring_a_mesh() {
    let src = sculpt_src();
    let body = function_body(&src, "subdivide(&mut self)");
    assert!(
        body.contains("self.stack.add_level()"),
        "subdividir é acrescentar um nível à pilha"
    );
    assert!(
        body.contains("StrokeUndo::AddedLevel"),
        "a entrada é o FATO de um nível ter entrado, não uma cópia da malha"
    );
    assert!(
        !body.contains("clone()"),
        "guardar a malha inteira é o que a pilha existe para não fazer"
    );
    // ⚠️ O traço em voo MORRE: ele carrega índices e um `pre` congelado de uma
    // topologia que deixou de existir.
    assert!(
        body.contains("self.stroke = SculptStroke::default()"),
        "subdividir tem de encerrar o traço em voo"
    );
    assert!(
        body.contains("self.mesh_rebuilt()"),
        "os buffers do device mudaram de TAMANHO: o upload incremental não serve"
    );

    // O outro lado: desfazer descarta o topo e mata o traço junto.
    //
    // ⚠️ O `undo_stroke` tem DOIS `match entry` — o primeiro escolhe o nível
    // (e `AddedLevel` não participa dele, com um braço vazio que existe para o
    // enum continuar exaustivo), o segundo AGE. Sem recortar o segundo, o
    // `braced_block` acha o braço vazio e a asserção mede o lugar errado.
    let undo = function_body(&src, "undo_stroke");
    let action = &undo[undo.rfind("match entry {").expect("o match que AGE")..];
    let arm = braced_block(action, "StrokeUndo::AddedLevel =>");
    assert!(
        arm.contains("self.stack.drop_top()") && arm.contains("SculptStroke::default()"),
        "desfazer um nível o descarta E encerra o traço"
    );

    // ⚠️ **E a entrada de EDIÇÃO carrega o NÍVEL**, senão desfazer um traço do
    // nível 0 de pé no 2 escreveria posições certas nos vértices errados — em
    // silêncio, porque os índices existem nos dois.
    assert!(
        undo.contains("self.stack.select(level)"),
        "desfazer volta ao nível em que a edição aconteceu"
    );

    // E as teclas existem, com o log que diz onde o artista está.
    let key = function_body(&src, "sculpt3d_key");
    assert!(
        key.contains("K::KeyK") && key.contains("scene.subdivide()"),
        "a tecla K chega à porta"
    );
    assert!(
        key.contains("K::Comma") && key.contains("scene.change_level(up)"),
        "as teclas de nível chegam à porta"
    );
    assert!(
        key.contains("stack.level()"),
        "o log tem de dizer o NÍVEL: a malha de baixo se parece com a de cima alisada"
    );
}

/// ⚠️ **Desfazer uma subdivisão de pé NOUTRO nível** — o gate que a mutação
/// pediu.
///
/// `drop_top` só age no topo (descartar do meio deixaria detalhes descrevendo um
/// nível que não existe mais), então sem subir primeiro o Ctrl+Z era consumido e
/// **não fazia nada** — a forma exata de *"o undo parou de funcionar"*.
#[test]
fn undoing_an_added_level_climbs_back_to_the_top_first() {
    let undo = function_body(&sculpt_src(), "undo_stroke");
    // O PRIMEIRO `match entry` é o que escolhe o nível; o segundo AGE.
    let at = undo.find("match entry {").expect("o match que SELECIONA");
    let arm = braced_block(&undo[at..], "StrokeUndo::AddedLevel =>");
    assert!(
        arm.contains("level_count()") && arm.contains("self.stack.select(top)"),
        "desfazer um nível tem de SUBIR ao topo antes de descartá-lo"
    );
}

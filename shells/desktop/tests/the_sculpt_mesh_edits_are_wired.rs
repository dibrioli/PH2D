//! **Arch-gates do que o gesto FAZ COM A MALHA** (ADR-0150 W2/W6).
//!
//! Irmão do `the_sculpt_gesture_is_wired`, e o corte é de responsabilidade: lá
//! *o que a mão pede* (ponteiro, grips, teclas, câmera), aqui *o que o documento
//! faz com o pedido* (a janela que sobe à GPU, as entradas de undo, as operações
//! de máscara, a mudança de topologia). Os dois leem a fonte pelo mesmo
//! [`sculpt_source`], porque duas cópias dos helpers divergiriam.

mod sculpt_source;
use sculpt_source::{braced_block, function_body, sculpt_src};

/// ⚠️ **A ORDEM é a feature: guardar a malha ANTES de trocá-la.**
///
/// Um `push` depois da substituição guardaria a malha NOVA, e o Ctrl+Z seguinte
/// devolveria exatamente o que já está na tela — um undo que não desfaz nada,
/// com todos os gates de contagem verdes. É uma relação posicional dentro de uma
/// função (a mesma classe do *o load instala depois do rebuild* da física), não
/// uma distância em bytes.

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
        body.contains("self.mesh.rebuild()"),
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
        undo.contains("StrokeUndo::Mask(") && undo.contains("StrokeUndo::Stroke"),
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

#[test]
fn subdividing_stores_the_mesh_before_it_replaces_it() {
    let src = sculpt_src();
    let body = function_body(&src, "subdivide(&mut self)");
    let push = body
        .find("StrokeUndo::Topology")
        .expect("a entrada de undo é de TOPOLOGIA: a contagem de vértices muda");
    let swap = body
        .find("self.mesh = ph2d_mesh::subdivide")
        .expect("a malha nova sai da porta do kernel");
    assert!(
        push < swap,
        "o estado anterior tem de ser guardado ANTES de a malha ser trocada"
    );
    // ⚠️ E o traço em voo MORRE: ele carrega índices e um `pre` congelado de uma
    // topologia que deixou de existir.
    assert!(
        body.contains("self.stroke = SculptStroke::default()"),
        "subdividir tem de encerrar o traço em voo"
    );
    assert!(
        body.contains("self.mesh_rebuilt()"),
        "os buffers do device mudaram de TAMANHO: o upload incremental não serve"
    );

    // O outro lado: desfazer devolve a malha inteira e mata o traço junto.
    let undo = function_body(&src, "undo_stroke");
    let arm = braced_block(&undo, "StrokeUndo::Topology(mesh) =>");
    assert!(
        arm.contains("self.mesh = *mesh") && arm.contains("SculptStroke::default()"),
        "desfazer uma topologia devolve a malha E encerra o traço"
    );

    // E a tecla existe, com o log que diz o preço.
    let key = function_body(&src, "sculpt3d_key");
    assert!(
        key.contains("K::KeyK") && key.contains("scene.subdivide()"),
        "a tecla K chega à porta"
    );
    assert!(
        key.contains("triangle_count()"),
        "o log tem de dizer a contagem NOVA: esta tecla quadruplica a malha"
    );
}

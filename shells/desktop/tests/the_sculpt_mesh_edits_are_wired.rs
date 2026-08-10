//! **Arch-gates do que o gesto FAZ COM A MALHA** (ADR-0150 W2/W6).
//!
//! Irmão do `the_sculpt_gesture_is_wired`, e o corte é de responsabilidade: lá
//! *o que a mão pede* (ponteiro, grips, teclas, câmera), aqui *o que o documento
//! faz com o pedido* (a janela que sobe à GPU, as entradas de undo, as operações
//! de máscara, a mudança de topologia). Os dois leem a fonte pelo mesmo
//! [`sculpt_source`], porque duas cópias dos helpers divergiriam.

mod sculpt_source;
use sculpt_source::{arm_with, braced_block, call_args, function_body, sculpt_src, source};

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
    let body = function_body(&sculpt_src(), "apply_entry");
    assert!(
        body.contains("mesh_mut().rebuild()"),
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

    let undo = function_body(&src, "apply_entry");
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
        body.contains(".stack.add_level()"),
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
    // ⚠️ O `apply_entry` tem DOIS `match entry` — o primeiro vai ao nível
    // (e a topologia participa dele por outro braço), o segundo AGE. Sem
    // recortar o segundo, o `braced_block` acha o braço da seleção e a asserção
    // mede o lugar errado.
    let undo = function_body(&src, "apply_entry");
    let action = &undo[undo.rfind("match entry {").expect("o match que AGE")..];
    let arm = braced_block(action, "StrokeUndo::AddedLevel =>");
    assert!(
        arm.contains("self.stack.drop_top()") && arm.contains("SculptStroke::default()"),
        "desfazer um nível o descarta E encerra o traço"
    );
    // ⚠️ **E o que sai vira a INVERSA, inteiro.** Refazer por recomputação
    // (`add_level` de novo) diverge assim que o artista tiver descido uma vez —
    // medido em `recomputing_the_subdivision_is_not_the_level_that_was_dropped`,
    // **0,236 numa esfera de raio 1**.
    assert!(
        arm.contains("StrokeUndo::DroppedLevel"),
        "o nível destacado tem de virar a entrada de refazer"
    );
    let back = braced_block(action, "StrokeUndo::DroppedLevel(level) =>");
    assert!(
        back.contains("self.stack.push_level(*level)") && !back.contains("add_level"),
        "refazer RECOLOCA o nível que saiu; recomputá-lo devolve outra malha"
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
    let undo = function_body(&sculpt_src(), "apply_entry");
    // O PRIMEIRO `match entry` é o que escolhe o nível; o segundo AGE.
    let at = undo.find("match entry {").expect("o match que SELECIONA");
    // ⚠️ **As DUAS metades da topologia sobem**, e o braço é um só de propósito:
    // `push_level` recusa fora do topo exatamente como o `drop_top`, então um
    // refazer de pé no nível de baixo seria o mesmo no-op silencioso — com a
    // agravante de **consumir** o nível que ele deveria devolver.
    let arm = braced_block(
        &undo[at..],
        "StrokeUndo::AddedLevel | StrokeUndo::DroppedLevel(_) =>",
    );
    assert!(
        arm.contains("level_count()") && arm.contains("self.stack.select(top)"),
        "mexer na pilha tem de SUBIR ao topo antes"
    );
}

/// ⚠️ **O ATALHO DE REFAZER DESFAZIA MAIS UM PASSO** — o defeito reportado, e
/// ele é pior que um botão inerte: `Ctrl+Shift+Z` caía no mesmo braço do
/// `Ctrl+Z` porque o `shift` **nunca chegava** à cena.
///
/// ⚠️ E o gate lê as DUAS metades. A metade da cena sozinha fica verde com o
/// `shift` preso em `false` no chamador — uma capacidade sem porta passa em todo
/// gate que só olha o lado de dentro.
#[test]
fn the_redo_shortcut_redoes_instead_of_undoing_one_more() {
    let key = function_body(&sculpt_src(), "sculpt3d_key");
    assert!(
        key.contains("scene.redo_stroke()") && key.contains("scene.undo_stroke()"),
        "as duas direções têm de existir na tecla"
    );
    assert!(
        key.contains("if shift"),
        "e o que as separa é o SHIFT, não a ordem"
    );

    // A outra metade: quem chama entrega o modificador.
    let keyboard = source("input_dispatch/keyboard.rs");
    let call = call_args(&keyboard, "self.sculpt3d_key");
    assert!(
        call.contains("shift_key()"),
        "o shift tem de CHEGAR: sem ele o redo é inalcançável com a cena verde"
    );
}

/// ⚠️ **Desfazer e refazer são a MESMA porta**, e é isso que impede um segundo
/// motor de divergir do primeiro no dia em que um deles ganhar um caso especial.
#[test]
fn undoing_and_redoing_are_the_same_door() {
    let src = sculpt_src();
    for (name, arg) in [("undo_stroke", "true"), ("redo_stroke", "false")] {
        let body = function_body(&src, &format!("{name}(&mut self)"));
        assert!(
            body.contains(&format!("self.step({arg})")),
            "`{name}` tem de delegar à porta comum"
        );
    }
    assert_eq!(
        src.matches("fn apply_entry").count(),
        1,
        "há UMA função que aplica uma entrada"
    );
    let step = function_body(&src, "step(&mut self, undoing");
    assert!(
        step.contains("self.apply_entry(object, entry.undo)"),
        "e as duas direções passam por ela"
    );
    // O que sai de uma fila entra na outra — sem isso o desfazer consome a
    // entrada e o refazer nunca tem o que fazer.
    assert!(
        step.contains("self.redo.push(inverse)") && step.contains("self.undo.push(inverse)"),
        "aplicar devolve a inversa, e ela vai para a fila oposta"
    );
}

/// ⚠️ **Uma edição nova torna o futuro guardado inalcançável**, e a lei mora na
/// porta que grava — não numa lista dos sítios que editam.
///
/// Enumerar os sítios (hoje três: o traço, a máscara, a subdivisão) é a lista
/// que nasce incompleta no dia em que aparece o quarto: ele gravaria por fora,
/// o refazer sobreviveria a uma edição que o tornou impossível, e um Ctrl+Shift+Z
/// instalaria um estado que **nunca existiu**.
#[test]
fn a_new_edit_goes_through_the_door_that_clears_the_redo() {
    let src = sculpt_src();
    let record = function_body(&src, "record_for(&mut self, object");
    assert!(
        record.contains("self.undo.push(Entry { object, undo })")
            && record.contains("self.redo.clear()"),
        "gravar é empurrar E limpar o futuro"
    );
    // ⚠️ **UMA resposta a *quando uma EDIÇÃO NOVA torna o futuro inalcançável*.**
    // A W8.2 quase abriu a segunda: o delete precisa gravar com a peça que SAIU
    // (e não com a ativa) e a primeira versão escreveu um `push` paralelo em
    // `sculpt3d_objects.rs`. Este gate o pegou; a cura foi `record_for`, com o
    // `record` delegando.
    //
    // ⚠️ **O `forget_history` da W8.3 é excluído, e não é isenção de
    // conveniência:** ele responde a OUTRA pergunta — *a sessão trocou de
    // documento* — e por isso limpa as DUAS filas, não só o futuro. Um load é o
    // fim da história inteira; uma edição é a poda de um ramo dela. Contá-lo
    // aqui faria o gate exigir que um load fosse uma edição.
    let edits_only = src.replace(&function_body(&src, "forget_history"), "");
    assert_eq!(
        edits_only.matches("self.redo.clear()").count(),
        1,
        "e essa é a única resposta a *quando uma edição nova mata o refazer*"
    );
    assert!(
        function_body(&src, "delete_active(&mut self").contains("self.record_for("),
        "o delete grava pela porta, com a peça que saiu"
    );
    for edit in [
        "close_stroke",
        "subdivide(&mut self)",
        "mask_op",
        "change_level",
        "reverse_level",
        "close_holes",
    ] {
        let body = function_body(&src, edit);
        assert!(
            body.contains("self.record("),
            "`{edit}` tem de gravar pela porta"
        );
    }
    // ⚠️ Contagem, e é o dente do gate: um quarto sítio que empurre direto na
    // fila tem de passar por aqui para justificar-se. Os dois legítimos são a
    // porta que grava e o `step`, que devolve a inversa.
    assert_eq!(
        src.matches("self.undo.push(").count(),
        2,
        "só a porta que grava e o `step` empurram no desfazer"
    );
}

/// ⚠️ **TROCAR DE NÍVEL É UMA EDIÇÃO, e a versão anterior deste gate afirmava o
/// contrário** — ele exigia que `change_level` **não** gravasse, sob o lema
/// *"olhar não é editar"*.
///
/// A frase é bonita e era falsa: **descer ESCREVE na malha de baixo** (o carimbo
/// do que foi esculpido em cima). Uma mutação fora da história é uma mutação sem
/// inverso, e o Enio reportou o sintoma — *artefatos na malha* — como a
/// consequência de o undo não guardar cada etapa.
#[test]
fn walking_the_levels_is_recorded_because_descending_writes() {
    let body = function_body(&sculpt_src(), "change_level");
    assert!(
        body.contains("self.record("),
        "trocar de nível grava: a descida CARIMBA a base"
    );
    // ⚠️ E as duas direções são entradas DISTINTAS: descer carrega o carimbo a
    // devolver, subir não carrega nada (ele só escreve o que a base e o detalhe
    // já determinam). Um caso só para as duas teria de mentir num dos lados.
    assert!(
        body.contains("StrokeUndo::Ascended") && body.contains("StrokeUndo::Descended"),
        "subir e descer não são a mesma entrada"
    );
    assert!(
        body.contains(".stack.lower()") && body.contains(".stack.higher()"),
        "e cada uma sai da porta da pilha que lhe corresponde"
    );

    // O outro lado: aplicá-las devolve uma a inversa da outra.
    let undo = function_body(&sculpt_src(), "apply_entry");
    let action = &undo[undo.rfind("match entry {").expect("o match que AGE")..];
    let down = braced_block(action, "StrokeUndo::Descended { from, stamped } =>");
    assert!(
        down.contains("self.stack.undo_descent(&stamped)") && down.contains("Ascended"),
        "desfazer uma descida devolve o carimbo e vira a subida inversa"
    );
    let up = braced_block(action, "StrokeUndo::Ascended { from } =>");
    assert!(
        up.contains("self.stack.lower()") && up.contains("Descended"),
        "desfazer uma subida é descer, e o carimbo que sair vira a inversa"
    );
}

/// ⚠️ **A REVERSÃO é o par do `K`, e o gate afirma os dois lados.** Ela
/// reconstrói um nível ABAIXO da base, o que renumera a malha inteira — então
/// ela tem de gravar (o irmão acima já cobre isso) e tem de DIZER o que fez: o
/// artista não vê nada mudar, por construção, e um gesto sem resposta na tela
/// nem no log é indistinguível de um gesto quebrado.
///
/// ⚠️ **A pergunta é feita do CORPO para fora** (`branch_containing`), e a razão
/// é que a forma anterior reprovou produto correto no dia em que o `Shift+J`
/// (fundir) nasceu: as duas famílias de verbo dividem a letra de propósito, e a
/// âncora `if code == K::KeyJ` passou a achar a da outra família.
#[test]
fn reversing_is_offered_on_its_own_key_and_it_says_what_it_did() {
    let branch = sculpt_source::branch_containing(&sculpt_src(), "scene.reverse_level()");
    assert!(
        branch.contains("K::KeyJ"),
        "des-subdividir tem tecla própria, vizinha do K que subdivide"
    );
    // As DUAS metades: o que aconteceu e o que NÃO aconteceu. Sem a segunda, uma
    // malha que não é subdivisão devolveria silêncio — que é como o artista
    // conclui que a tecla não existe.
    assert!(
        branch.contains("revertida:") && branch.contains("nao' reverte"),
        "o log fala nos dois desfechos"
    );
}

/// ⚠️ **Desfazer uma reversão é TIRAR o nível, e refazer é reconstruí-lo — não
/// há estado guardado para o segundo.** É o oposto exato do `DroppedLevel`, que
/// TEM de carregar o nível porque recomputá-lo dependeria de uma base que o
/// carimbo já mudou; aqui reverter é função pura da malha. Trocar um pelo outro
/// não falha: produz uma pilha PARECIDA, que é a pior forma de errado.
#[test]
fn undoing_a_reversal_removes_the_level_and_redoing_it_rebuilds_it() {
    let src = sculpt_src();
    let apply = function_body(&src, "apply_entry");
    let action = &apply[apply.rfind("match entry {").expect("o match que AGE")..];

    let undo = braced_block(action, "StrokeUndo::ReversedLevel(rev) =>");
    assert!(
        undo.contains("self.stack.unreverse(&rev)") && undo.contains("UnreversedLevel"),
        "desfazer despermuta a pilha e vira a inversa"
    );
    assert!(
        undo.contains("StrokeUndo::ReversedLevel(rev)"),
        "e uma recusa devolve a MESMA entrada, em vez de consumir a única que desfaz"
    );

    let redo = braced_block(action, "StrokeUndo::UnreversedLevel =>");
    assert!(
        redo.contains("self.stack.reverse()") && redo.contains("ReversedLevel"),
        "refazer é chamar a reversão de novo — ela é determinística"
    );

    // E o `seek`: os dois lados pousam em níveis DIFERENTES, e aplicar uma
    // entrada do nível errado é o no-op silencioso que o gate do `AddedLevel` já
    // pagou uma vez.
    let seek = &apply[..apply.rfind("match entry {").expect("o match que AGE")];
    assert!(
        arm_with(seek, "StrokeUndo::ReversedLevel(_)").contains("select(1)"),
        "desfazer uma reversão acontece do nível 1, onde ela deixou o artista"
    );
    assert!(
        arm_with(seek, "StrokeUndo::UnreversedLevel").contains("select(0)"),
        "e refazê-la, do nível 0"
    );
}

/// ⚠️ **A cena `=3` só significa alguma coisa se a malha dela REVERTER**, e isso
/// é um fato sobre geometria que nenhum gate de fonte enxerga. Este aqui pina a
/// metade que a fonte pode dizer — que ela é construída SUBDIVIDINDO, que é a
/// condição necessária —, e a metade geométrica vive no
/// `ph2d-mesh::reversion_tests`, sobre exatamente esta forma.
#[test]
fn the_reversion_scene_opens_with_a_mesh_that_is_a_subdivision() {
    let src = sculpt_src();
    let body = function_body(&src, "smoke_mesh");
    assert!(
        body.contains("reversion_scene()"),
        "a cena escolhe a malha dela"
    );
    let arm = braced_block(&body, "if reversion_scene()");
    assert_eq!(
        arm.matches("subdivide(").count(),
        2,
        "duas subdivisões: uma só daria um nível a reconstruir, e a cena mostra dois"
    );
}

/// ⚠️ **TAPAR BURACO tem TRÊS desfechos, e o log fala nos três.** *"Tapei N"*,
/// *"não havia nenhum"* e *"não posso, a pilha está montada"* são respostas
/// diferentes, e colapsar as duas últimas em silêncio é como o artista conclui
/// que a tecla não funciona.
#[test]
fn closing_holes_is_offered_on_its_own_key_and_it_says_which_of_the_three_happened() {
    let src = sculpt_src();
    let key = function_body(&src, "sculpt3d_key");
    assert!(key.contains("K::KeyO"), "tapar buraco tem tecla própria");
    let block = braced_block(&key, "if code == K::KeyO");
    assert!(
        block.contains("scene.close_holes()"),
        "e ela sai pela porta que grava"
    );
    for said in ["tapados", "nenhum buraco", "pilha montada"] {
        assert!(
            block.contains(said),
            "o log tem de falar no desfecho `{said}`"
        );
    }
}

/// ⚠️ **Desfazer um preenchimento é TRUNCAR, e isso não é um atalho: é uma
/// propriedade do algoritmo.** Um remendo é geometria nova colada na beira, então
/// nenhum vértice antigo é tocado e o estado anterior são duas contagens. Guardar
/// a malha aqui seria a cópia do documento que a pilha de níveis existe para não
/// fazer — e o gate do `ph2d-mesh` que afirma o *só acrescenta* é quem sangra no
/// dia em que isso deixar de valer.
#[test]
fn undoing_a_hole_fill_truncates_instead_of_restoring_a_mesh() {
    let src = sculpt_src();
    let body = function_body(&src, "close_holes");
    assert!(
        body.contains("ph2d_mesh::fill_holes"),
        "tapar sai da porta da malha"
    );
    assert!(
        body.contains("self.stack.level_count() != 1"),
        "com a pilha montada ela RECUSA: tapar muda a topologia que os níveis de cima subdividem"
    );
    assert!(
        !body.contains("clone()"),
        "guardar a malha é o que o truncar existe para não fazer"
    );

    let apply = function_body(&src, "apply_entry");
    let action = &apply[apply.rfind("match entry {").expect("o match que AGE")..];
    let undo = braced_block(action, "StrokeUndo::FilledHoles { verts, faces } =>");
    assert!(
        undo.contains("truncate(verts, faces)") && undo.contains("UnfilledHoles"),
        "desfazer trunca e vira a inversa"
    );
    assert!(
        undo.contains("StrokeUndo::FilledHoles { verts, faces }"),
        "e uma recusa devolve a MESMA entrada em vez de consumi-la"
    );
    let redo = braced_block(action, "StrokeUndo::UnfilledHoles =>");
    assert!(
        redo.contains("ph2d_mesh::fill_holes") && redo.contains("FilledHoles"),
        "refazer é tapar de novo — é determinístico"
    );
}

/// ⚠️ **A cena `=4` DECLARA o furo que montou.** Um smoke de fechar buraco sobre
/// uma malha fechada é indistinguível da feature quebrada, e a única coisa que
/// separa os dois é o número da beira aparecer ANTES de o artista apertar nada.
#[test]
fn the_holes_scene_says_how_big_the_hole_it_built_is() {
    let src = sculpt_src();
    let mesh = function_body(&src, "smoke_mesh");
    assert!(
        mesh.contains("holes_scene()") && mesh.contains("punctured_sphere()"),
        "a cena escolhe a malha dela"
    );
    let punctured = function_body(&src, "punctured_sphere");
    assert!(
        punctured.contains(".filter(") && punctured.contains("faces()"),
        "o furo é feito ARRANCANDO faces, não desenhando uma beira"
    );
    // ⚠️ **Ele pergunta ao CLUSTER, e não a uma função — porque já expirou DUAS vezes.**
    // A declaração morava no `sculpt3d_smoke`, mudou para o `announce` quando aquele arquivo
    // bateu o teto de LOC, e mudou de novo para o `scripts::for_scene` quando o teto pegou o
    // arquivo das CENAS. Nas duas o produto estava certo e o gate ficou vermelho apontando para
    // um endereço. A propriedade nunca mudou — *a cena `=4` imprime quantas beiras ela abriu* —,
    // e ela não depende de em que função a linha mora. O `sculpt_src` já junta o cluster inteiro,
    // então o bloco é achado onde quer que ele esteja, e o próximo split não derruba nada.
    let arm = braced_block(&src, "if crate::sculpt3d::holes_scene()");
    assert!(
        arm.contains("valence(") && arm.contains("arestas de BEIRA"),
        "e a cena imprime quantas arestas de beira ela abriu"
    );
}

#[test]
fn the_remesh_has_a_key_and_it_says_both_counts() {
    // ⚠️ Este botão **não muda a forma** — ele muda a MALHA. Sem os dois números
    // no log, o artista aperta, vê a mesma escultura e conclui que a tecla está
    // morta; é a mesma razão pela qual o `K` imprime a contagem nova.
    let src = sculpt_src();
    let key = function_body(&src, "sculpt3d_key");
    let block = braced_block(&key, "code == K::KeyV");
    assert!(
        block.contains("scene.remesh("),
        "o V tem de chamar a porta de remesh"
    );
    assert!(
        block.contains("r.verts.0") && block.contains("r.verts.1"),
        "o log tem de trazer o ANTES e o DEPOIS: um deles sozinho não diz que a malha mudou"
    );
    assert!(
        block.contains("r.cells"),
        "e o número de células, que é o que explica o tempo"
    );
}

#[test]
fn the_remesh_refuses_with_the_stack_built_instead_of_flattening_it() {
    // ⚠️ A alternativa não é neutra: achatar a pilha em silêncio destrói níveis
    // que o artista autorou. A recusa é a MESMA lei do `close_holes` — tapar e
    // remesh trocam a topologia da base, e todo nível acima é `subdivide` dela.
    let body = function_body(&sculpt_src(), "remesh");
    assert!(
        body.contains("level_count() != 1") && body.contains("RemeshRefusal::MultiresStack"),
        "o remesh tem de recusar com a pilha montada, e NOMEAR a recusa que o log lê"
    );
    // E a recusa precisa CHEGAR ao artista, ou ele conclui que a tecla quebrou.
    let key = function_body(&sculpt_src(), "sculpt3d_key");
    let block = braced_block(&key, "code == K::KeyV");
    assert!(
        block.contains("RemeshRefusal::MultiresStack"),
        "o braço da recusa tem de existir e dizer por quê"
    );
    // ⚠️ **E cada causa precisa do braço DELA.** Este gate ancorava em `return
    // None` e num braço `None =>`, o que era a GRAFIA de uma recusa, não a
    // propriedade — e por isso ele ficava verde sobre o defeito real: três
    // causas (pilha montada · cena vazia · o campo sem interior) entravam num
    // `Option` só, e o chamador elegia UMA mensagem para as três. Um campo que
    // vazava mandava o artista reverter níveis que ele não tem.
    for cause in ["MultiresStack", "EmptyScene", "Engine"] {
        assert!(
            block.contains(cause),
            "a recusa `{cause}` sumiu do despacho: o artista não é informado dela"
        );
    }
    // ⚠️ **E a presença do NOME não basta** — a primeira versão deste laço
    // afirmava só isso, e a mutação que colapsa duas causas num braço só
    // (`MultiresStack | EmptyScene =>`) passou por ele: o nome continua no
    // texto, dentro do braço da outra. O que o artista recebe é uma MENSAGEM, e
    // a propriedade é que haja uma por causa — então o gate conta BRAÇOS.
    let arms = block.matches("Err(RemeshRefusal::").count();
    assert_eq!(
        arms, 3,
        "são {arms} braços de recusa para três causas: alguma partilha a mensagem de outra, \
         que é o defeito de origem — um campo vazado mandando reverter níveis inexistentes"
    );
}

#[test]
fn undoing_a_remesh_swaps_the_whole_mesh_because_nothing_is_shared() {
    // ⚠️ Toda outra entrada de undo deste módulo compartilha estrutura com o
    // estado anterior — a janela do traço, o `truncate` de tapar buraco, o nível
    // intocado embaixo de uma subdivisão. Um remesh não compartilha NADA: nem a
    // contagem de vértices, nem a de faces, nem a correspondência entre elas. Se
    // alguém trocar isto por uma janela ou por dois `usize`, o Ctrl+Z devolve
    // uma malha que nunca existiu.
    let src = sculpt_src();
    assert!(
        src.contains("Remeshed(Box<ph2d_mesh::Mesh>)"),
        "a entrada tem de carregar a malha inteira de antes"
    );
    let apply = function_body(&src, "apply_entry");
    let arm = braced_block(&apply, "StrokeUndo::Remeshed(previous)");
    assert!(
        arm.contains("mem::replace") && arm.contains("StrokeUndo::Remeshed(Box::new(now))"),
        "aplicar é TROCAR e devolver o que estava lá — a entrada é a própria inversa"
    );
    // E o gesto tem de gravar a entrada, senão não há o que desfazer.
    let body = function_body(&src, "remesh");
    assert!(
        body.contains("record(StrokeUndo::Remeshed"),
        "o remesh tem de entrar na história"
    );
    assert!(
        body.contains("mesh_rebuilt()"),
        "a malha é OUTRA: o device precisa de tudo, não de uma janela"
    );
}

/// **UM CTRL+Z VOLTA AO OBJETO DA EDIÇÃO** — a lei que a cena-lista inventou.
///
/// ⚠️ Sem ela o defeito é mudo e destrutivo: esculpir a peça A, esculpir a peça
/// B, e o Ctrl+Z aplicar a janela de A **na malha de B** — índices certos, malha
/// errada, sem erro nenhum. É a mesma lei do NÍVEL um degrau acima, e por isso
/// ela vem ANTES: escolher o nível dentro do objeto errado não conserta nada.
///
/// A asserção é de ORDEM, e a ordem é o que carrega o peso: `self.active` tem de
/// ser escrito **antes** de `apply_entry` ser chamada.
#[test]
fn undoing_returns_to_the_object_the_edit_was_made_on() {
    let src = sculpt_src();
    let step = function_body(&src, "step(&mut self, undoing");
    let at_active = step
        .find("self.index_of(entry.object)")
        .expect("o `step` tem de resolver a peça da entrada");
    let at_apply = step.find("self.apply_entry(").expect("e depois aplicar");
    assert!(
        at_active < at_apply,
        "voltar à peça vem ANTES de aplicar — depois já é tarde"
    );

    // ⚠️ E a resolução é pela IDENTIDADE, não pelo índice: apagar a peça 1 de
    // três faz a antiga 2 virar 1, e um índice guardado passaria a nomear outra
    // peça em silêncio.
    assert!(
        src.contains("pub(super) object: ObjectId,"),
        "a entrada aponta pelo `ObjectId`, nunca por posição"
    );

    // E quem GRAVA carimba a peça: uma entrada sem dono é uma entrada que o
    // desfazer aplica em quem estiver na mão.
    let record = function_body(&src, "record(&mut self, undo");
    assert!(
        record.contains("self.obj().map(|o| o.id)") && record.contains("self.record_for(id, undo)"),
        "gravar carimba a peça ativa e delega à porta única"
    );
}

/// **O `sync_mesh` percorre TODAS as peças à VISTA, não só a ativa.**
///
/// ⚠️ Um `sync` que olhasse só o ativo deixaria toda peça que a mão não está
/// trabalhando **sem geometria no device** — a cena mostraria menos objetos do
/// que tem, e nenhum gate de CPU veria isso (a malha está lá, correta, na RAM).
///
/// ⚠️ **A afirmação MUDOU com o isolamento, e é a metade sobre a qual ele fala
/// que decide.** Antes o laço era literalmente `0..self.objects.len()`, e o gate
/// citava esse texto; hoje a lista é a das VISÍVEIS (a `k`-ésima delas mora no
/// slot `k`), o que é uma resposta diferente à mesma pergunta. Citar a forma do
/// laço teria feito o gate reprovar produto correto — ele afirma agora *quem é
/// perguntado* (o plano, que só conhece as visíveis) e *que a pose vai junto*.
#[test]
fn every_object_reaches_the_device_not_only_the_active_one() {
    let src = sculpt_src();

    let plan = function_body(&src, "slot_plan(&self");
    assert!(
        plan.contains(".visible_pieces()"),
        "o plano tem uma linha por peça À VISTA — nunca só a ativa"
    );

    let sync = function_body(&src, "sync_mesh(&mut self, device");
    assert!(
        sync.contains("self.slot_plan()") && sync.contains("for (k, line) in plan"),
        "e o sync percorre esse plano inteiro"
    );
    assert!(
        sync.contains("set_pose(k, self.objects[i].pose)"),
        "a pose de cada uma vai junto — sem ela todas desenhariam na origem"
    );
}

/// **O pick compara em MUNDO, e o pincel desce a escala.**
///
/// ⚠️ Duas metades da mesma pergunta, e as duas são invisíveis num teste de
/// forma: comparar acertos pelo `t` faz a peça de trás ganhar o clique quando as
/// escalas diferem, e não dividir o raio pela escala encolhe a pegada num objeto
/// grande. Nenhuma das duas levanta erro; as duas aparecem só como *"o pincel
/// está estranho"*.
#[test]
fn the_pick_compares_in_world_and_the_brush_crosses_the_scale() {
    let src = sculpt_src();
    let pick = function_body(&src, "pick(&self, x: f32, y: f32)");
    assert!(
        pick.contains("o.pose.ray_to_local(&world)"),
        "o RAIO desce ao espaço de cada malha"
    );
    assert!(
        pick.contains("o.pose.point_to_world(hit.point)"),
        "e a comparação sobe de volta ao mundo — um `t` mediria em réguas diferentes"
    );

    let brush = function_body(&src, "armed_brush(&self, local_at");
    assert!(
        brush.contains("pose.point_to_world(local_at)"),
        "a câmera só responde sobre o mundo"
    );
    assert!(
        brush.contains("/ pose.scale()"),
        "e o raio volta para a régua da malha"
    );
}

/// **UM TRAÇO PERTENCE A UMA PEÇA**, escolhida no pen-down.
///
/// ⚠️ Isto não é preferência de UX, é a diferença entre desenhar e **panicar**:
/// o `SculptStroke::begin` dimensiona os planos por-vértice na malha em que o
/// traço COMEÇOU (`slot`, `stamp`), e um dab noutra peça indexa esses planos com
/// os índices dela. Enquanto a peça nova for menor, o defeito é mudo; assim que
/// for maior, o primeiro dab estoura. Medido: tocar o cubo (8 vértices) e depois
/// a esfera (6050) panicava.
///
/// As duas metades, e as duas são necessárias:
/// 1. o pen-down **MIRA antes de começar** — a ordem é o conserto;
/// 2. os dois gestos consultam a peça **ATIVA**, nunca a lista, então nada troca
///    de objeto no meio de uma pincelada.
#[test]
fn a_stroke_belongs_to_the_piece_it_started_on() {
    let src = sculpt_src();
    let down = function_body(&src, "sculpt3d_pointer_down(&mut self");
    let at_aim = down.find("scene.aim(").expect("o pen-down tem de MIRAR");
    let at_begin = down
        .find("scene.stroke.begin(")
        .expect("e depois começar o traço");
    assert!(
        at_aim < at_begin,
        "mirar vem ANTES de começar — depois já é a malha errada"
    );

    // ⚠️ **A pergunta certa é quem consulta a lista E PODE AGIR sobre ela.**
    //
    // Esta asserção já expirou DUAS vezes, sempre por contar em vez de afirmar:
    // a v1 contava `self.active =` (e os verbos da lista — acrescentar,
    // duplicar, apagar — escrevem nele por definição), e a v2 contava
    // `self.pick(x, y)` == 1, que a W12 derrubou ao dar um CURSOR à cena. O anel
    // do pincel pica para saber onde desenhar e **não pode** trocar de peça: ele
    // é `&self`, e isso é garantido pelo compilador, não por texto.
    //
    // A lei fica: **o `aim` é o único que MOVE o ativo a partir de um pick**, e
    // todo outro consumidor é somente-leitura.
    let aim = function_body(&src, "aim(&mut self");
    assert!(
        aim.contains("self.pick(x, y)") && aim.contains("self.active ="),
        "o `aim` é quem escolhe a peça a partir de um pick — se ele não faz mais \
         isso, quem faz?"
    );
    let mark = function_body(&src, "cursor_mark(&self");
    assert!(
        mark.contains("self.pick(x, y)") && !mark.contains("self.active"),
        "o cursor pica para DESENHAR e não pode mexer no ativo — `&self` já o \
         proíbe, e esta linha é o que impede alguém de torná-lo `&mut self`"
    );
    assert_eq!(
        src.matches("self.pick(x, y)").count(),
        2,
        "apareceu um TERCEIRO consumidor da lista: se ele for `&mut self`, ele \
         pode trocar de peça no meio de um gesto — nomeie-o aqui e prove que é \
         somente-leitura"
    );
    for gesture in ["sculpt_at(&mut self", "take_hold(&mut self"] {
        let body = function_body(&src, gesture);
        assert!(
            body.contains("self.pick_active(x, y)"),
            "`{gesture}` consulta a peça ATIVA"
        );
        assert!(
            !body.contains("self.pick(x, y)"),
            "`{gesture}` NÃO pode repicar a lista no meio de um gesto"
        );
    }
}

/// **OS VERBOS DA LISTA são desfazíveis**, e a peça apagada volta INTEIRA.
///
/// ⚠️ A alternativa que a W8.1 anotou — *apagar limpa a fila* — trocaria um
/// trabalho perdido por outro: o artista recuperaria a peça e perderia a
/// história de todas as outras. Carregar a peça na entrada custa um `Box` e
/// devolve a pilha de níveis, a máscara e a pose junto.
#[test]
fn the_list_verbs_are_undoable_and_the_last_piece_is_deletable_too() {
    let src = sculpt_src();

    for verb in ["add_primitive(&mut self", "duplicate_active(&mut self"] {
        let body = function_body(&src, verb);
        assert!(
            body.contains("self.record(StrokeUndo::AddedObject)"),
            "`{verb}` grava, senão a peça nova é irremovível por Ctrl+Z"
        );
        assert!(
            body.contains("self.active = self.objects.len() - 1"),
            "`{verb}` torna a peça nova ativa — escolher uma coisa USA ela"
        );
    }

    // ⚠️ **E o desfazer alcança a cena VAZIA**, que é o que torna *"apaguei
    // tudo"* reversível: o `RemovedObject` recoloca a peça onde não há nenhuma.
    // Sem esta isenção, o guard do `apply_entry` recusaria justamente a entrada
    // que devolve a última peça — e o artista ficaria com uma cena vazia e um
    // Ctrl+Z que não faz nada. Achado por mutação: nenhum outro gate a via.
    let apply = function_body(&src, "apply_entry");
    assert!(
        apply.contains("let list_verb = matches!"),
        "o guard tem de distinguir os verbos de LISTA dos de peça"
    );
    assert!(
        apply.contains("!list_verb && self.obj().is_none()"),
        "…e ISENTAR os de lista: sem isso, desfazer o delete da última peça \
         seria recusado pelo próprio guard"
    );

    let del = function_body(&src, "delete_active(&mut self");
    // ⚠️ **A ÚLTIMA peça É apagável, e este gate afirmava o CONTRÁRIO.** O Enio
    // derrubou a cerca no smoke (*"não consigo deletar todos os objetos"*): ela
    // defendia uma invariante nossa — a lista nunca-vazia que tornava o `obj()`
    // total —, não um interesse do artista. Hoje o `obj()` devolve `Option` e a
    // cena pode esvaziar; o que recusa é a cena que JÁ está vazia.
    assert!(
        !del.contains("self.objects.len() <= 1"),
        "a última peça voltou a ser inapagável — a cerca foi derrubada de propósito"
    );
    assert!(
        del.contains("let Some(object) = self.obj()") && del.contains("return false"),
        "…e o que recusa é não HAVER peça, não ela ser a última"
    );
    assert!(
        del.contains("StrokeUndo::RemovedObject(Box::new(gone))"),
        "a peça sai INTEIRA para dentro da fila"
    );
    // ⚠️ E com o id da peça que SAIU, não com o da que ficou ativa: é ela que o
    // `RemovedObject` vai recolocar.
    let at_id = del
        .find("let Some(object) = self.obj().map(|o| o.id)")
        .expect("o id da peça que sai");
    let at_remove = del
        .find("self.objects.remove(")
        .expect("e depois a remoção");
    assert!(
        at_id < at_remove,
        "o id é lido ANTES de a peça sair da lista"
    );

    // A volta: recolocar torna a peça ativa, senão o artista desfaz e continua
    // trabalhando outra.
    let arm = arm_with(&src, "StrokeUndo::RemovedObject(piece)");
    assert!(
        arm.contains("self.objects.push(*piece)") && arm.contains("self.active ="),
        "desfazer um delete recoloca a peça E volta para ela"
    );
}

/// **As quatro primitivas cabem na MESMA esfera unitária.**
///
/// ⚠️ Sem essa normalização a escala da pose teria de depender da forma, e o
/// artista veria o cubo nascer de outro tamanho que a esfera pelo mesmo gesto —
/// com o mesmo número na pose. É gate de fonte porque a cena precisa de device;
/// o oráculo geométrico das primitivas em si vive na `ph2d-mesh`.
#[test]
fn every_primitive_is_born_in_the_same_unit_ball() {
    let src = sculpt_src();
    let body = function_body(&src, "mesh(self) -> Mesh");
    for kind in ["Sphere", "Cube", "Cylinder", "Torus"] {
        assert!(
            body.contains(&format!("Self::{kind} =>")),
            "a primitiva {kind} tem de ter malha"
        );
    }
    // A aresta do cubo é 2/√3 justamente para a DIAGONAL medir 2 — a mesma
    // esfera envolvente da bola de raio 1. Um `cube(2.0)` aqui nasceria com a
    // diagonal em 3,46 e o gesto pareceria dar tamanhos diferentes.
    assert!(
        body.contains("shapes::cube(2.0 / 3.0_f32.sqrt())"),
        "o cubo é normalizado pela DIAGONAL, não pela aresta"
    );
}

/// **A resolução AUTORADA chega ao motor — nas DUAS portas.**
///
/// ⚠️ Este é o gate da quarta condição (*a sequência leva a algum lugar*): a row
/// pode estar registrada, viva sob o mouse e despachando, e o número ainda
/// morrer na fronteira do shell. Até esta wave os dois chamadores cravavam
/// `ph2d_sdf::DEFAULT_RESOLUTION`, então o slider seria um controle morto.
///
/// E ele afirma **as duas** portas de propósito: o botão do painel e a tecla
/// `V`. Duas portas para um número divergem no dia em que só uma aprende o
/// slider, e o artista fica com dois remeshes diferentes para o mesmo gesto.
#[test]
fn the_authored_resolution_reaches_both_remesh_doors() {
    let src = sculpt_src();
    for (porta, corpo) in [
        (
            "o botao do painel",
            function_body(&src, "apply_panel_intent"),
        ),
        ("a tecla V", function_body(&src, "sculpt3d_key")),
    ] {
        assert!(
            corpo.contains("remesh(self.remesh_res)") || corpo.contains("remesh(scene.remesh_res)"),
            "{porta}: o remesh nao le' a resolucao autorada -- o slider seria um controle morto"
        );
        assert!(
            !corpo.contains("remesh(ph2d_sdf::DEFAULT_RESOLUTION)"),
            "{porta}: ainda crava a const, entao o slider nao a alcanca"
        );
    }
}

/// **A PORTA da história poda, e é ela que faz o teto existir.**
///
/// ⚠️ Um gate de unidade é CEGO a isto: a poda é conferível sem device
/// (`trim_to_budget` é função solta, e há quatro gates sobre ela), mas *o único
/// ponto de crescimento de fato a chamar* só se afirma sobre a fonte — uma cena
/// precisa de um `wgpu::Device`, e nenhum teste headless a constrói.
///
/// O defeito que ele previne está medido: cada remesh a 512 empilha uma malha
/// inteira (**146 MB** de residência, `ph2d-sdf/tests/probe_repeat_remesh.rs`)
/// numa fila que não tinha teto nenhum — *fazer remesh algumas vezes* era uma
/// escada até o fim da memória, num app cujo orçamento declarado é 3500 MB.
#[test]
fn the_one_door_that_grows_the_history_is_the_one_that_trims_it() {
    let body = function_body(&sculpt_src(), "record_for");
    assert!(
        body.contains("self.undo.push("),
        "controle: `record_for` continua sendo o ponto de crescimento"
    );
    assert!(
        body.contains("self.trim_history()"),
        "a porta que empurra tem de podar -- sem isso a fila cresce sem teto"
    );
}

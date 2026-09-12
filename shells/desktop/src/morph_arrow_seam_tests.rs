//! **A COSTURA das setas do Morph — a metade que mede a SHELL** (plano 32 W4).
//!
//! ⛔⛔ **Este gate ficou na shell quando a lei saiu para [`ph2d_app_vec::morph_edit`]** (W2 Fase C),
//! e a razão é a §2.6/§2.9 do HOWTO: *uma agulha segue o SUJEITO, nunca o ficheiro*. O que ele
//! afirma é **que a shell chama a lei** — os cinco `include_str!` apontam para `render_loop/mod.rs`,
//! `morph_machine_drive.rs` e os dois do teclado, que **não se mudaram**. Levá-lo para a crate
//! obrigaria as agulhas a nomear `crate::morph_edit::`, e a shell escreve `crate::vec_morph_edit::`
//! (o alias de `main.rs`) ⇒ o gate compilaria e reprovaria a correr, com a lei intacta.
//!
//! ⚠️ **Foi exactamente o que a reescrita automática desta fatia fez**, e só a leitura o apanhou:
//! ela trocou o nome DENTRO da string, porque uma regex não distingue *«o endereço que este
//! ficheiro usa»* de *«o endereço que o ficheiro medido usa»*. É a 2.ª vez que esta linha paga a
//! forma — a 1.ª foi a agulha do marquee, na 2.ª volta.

/// **A COSTURA: o clique da seta atravessa o barramento e chega ao mundo.**
///
/// ⚠️ Um gate de TEXTO, e a razão é a mesma do gesto: o caminho real precisa de um `AppGfx`. A
/// **lei** está gateada acima; o que sobra é provar que alguém a chama — e isso só se pode ler.
///
/// ⛔ Sem esta metade, os gates acima ficariam verdes sobre controlos que **pintam, acendem sob o
/// rato e cujo clique morre no painel** — que é o defeito que a décima lista do modo custou uma
/// wave atrás.
#[test]
fn the_arrow_click_reaches_the_world() {
    let shell = include_str!("render_loop/mod.rs");
    for (needle, what) in [
        (
            "crate::vec_morph_edit::morph_cmd_for_id(*id)",
            "RESOLVER o id do clique",
        ),
        (
            "crate::vec_morph_edit::apply(sim, &self.vec_entities, e, cmd, &actions)",
            "APLICAR ao mundo",
        ),
        (
            "morph_of_selection(sim, &self.vec_entities, &sel)",
            "RESOLVER a seleccao pelo MAPA, e nunca como bits de entidade",
        ),
        (
            "set_morph_states_state(crate::vec_morph_edit::publish(",
            "PUBLICAR a projeccao",
        ),
    ] {
        assert!(
            shell.contains(needle),
            "a shell perdeu o `{needle}` -- {what}. A lei continua gateada e INALCANCAVEL."
        );
    }
    // ⭐ E o CONJUNTO sai por outra porta, porque ele cria a entidade que o `apply` exigiria já
    // existente. Sem esta linha o botão pinta, acende e o clique morre no `else if` do irmão.
    for (needle, what) in [
        (
            "pending_morph_arrow == Some(crate::vec_morph_edit::MorphCmd::MakeSet)",
            "reconhecer o pedido de FAZER o conjunto",
        ),
        (
            "ph2d_vec_entities::morph_set::create(",
            "CRIAR o conjunto (o path novo + o pendente)",
        ),
        (
            "ph2d_vec_entities::morph_set::upkeep(",
            "DRENAR o pendente: pendurar a maquina, reparentar e esconder os membros",
        ),
        (
            "self.vec_pen.select_many(&[p.path]);",
            "SELECCIONAR o conjunto novo -- senao a seleccao fica nos MEMBROS, que acabaram de \
             ficar ocultos e com dono, e a seccao oferece um SEGUNDO conjunto sobre eles",
        ),
    ] {
        assert!(
            shell.contains(needle),
            "a shell perdeu o `{needle}` -- {what}. O botao pinta e nao faz nada."
        );
    }
    // ⭐⭐ **O MODO DE PRÉ-VISUALIZAÇÃO** (W9): o interruptor, o que ele dirige, e — a metade que
    // dá sentido ao modo — o teclado que ele TOMA. ⛔ Sem a guarda, a tecla morfa a forma **e** faz
    // o que ela faz no editor: é o report do Enio (*"as setas do teclado movendo as formas"*).
    for (needle, what) in [
        (
            "*id == ph2d_editor_core::ids::VECTOR_MORPH_PREVIEW",
            "RECONHECER o clique no interruptor",
        ),
        (
            "self.morph_preview = !self.morph_preview",
            "LIGAR e DESLIGAR o modo",
        ),
        // ⭐⭐⭐ **E o modo passa pelo `drives`** (W11e): ele é o MODO **menos** o que o sistema de
        // States está a fazer. ⛔ Sem o segundo termo, a máquina de teclas repõe a forma dela por
        // cima do `Default` no repouso e na chegada — é o 2.º report do Enio (*"Default não
        // segurou wide e está em tall"*).
        (
            "crate::morph_machine_drive::drives(self.morph_preview, self.ui_state_live),\n                self.fixed_step.fixed_dt(),",
            "DIRIGIR a maquina pelo MODO, e nao pelo playhead -- e LARGAR enquanto os States agem",
        ),
        // ⭐⭐⭐ **E a RECONCILIAÇÃO corre FORA do modo** (W11g): a lista de estados é derivada dos
        // filhos, mas o par que a cena desenha é guardado — e o ⊘ corre com a pré-visualização
        // desligada. ⛔ Sem esta chamada o conjunto continua a cozer a forma que saiu, e o painel
        // continua a nomeá-la: é o 3.º report do Enio.
        (
            "crate::morph_machine_drive::reconcile(",
            // ⚠️ A agulha e' o NOME da porta e nao a chamada inteira: o `cargo fmt` quebra a
            // chamada em cinco linhas assim que ela passa da largura, e uma agulha multi-linha
            // fica refem da formatacao em vez de medir a fiacao.
            "RECONCILIAR o par desenhado com a lista de estados, todo quadro",
        ),
    ] {
        assert!(
            shell.contains(needle),
            "a shell perdeu o `{needle}` -- {what}."
        );
    }
    // ⛔ **E o playhead NÃO pode voltar a ser a porta:** ele não tranca o teclado do editor, que é
    // exactamente o conflito que este modo existe para curar.
    assert!(
        !shell.contains("self.playhead.is_playing(),\n                self.fixed_step.fixed_dt(),"),
        "o playhead voltou a dirigir a maquina -- o conflito de atalhos volta com ele"
    );
    // ⭐⭐ **A DERIVAÇÃO chega ao MOTOR** (W11) — e ela vive no `morph_machine_drive`, não aqui.
    // Sem esta linha, arrastar uma forma para dentro do conjunto na Hierarquia não a faria
    // participar: o motor continuaria a percorrer uma lista que ninguém actualiza.
    assert!(
        // ⚠️ **Esta agulha sozinha segue o MOTOR, e ele mudou de crate** (W2 Fase D): as outras
        // deste ficheiro medem o que a SHELL escreve (`render_loop/mod.rs`, os dois do teclado) e
        // ficam. ⭐ *O ficheiro partiu-se em dois sujeitos, e cada agulha segue o seu* — é a 4.ª vez
        // que esta linha aplica a regra, e a 1.ª em que ela dá duas respostas DENTRO do mesmo gate.
        include_str!("../../../crates/ph2d-app-vec/src/morph_machine_drive.rs").contains(
            "ph2d_vec_entities::morph_set::graph_of(sim, map_paths, Entity::from_bits(b))"
        ),
        "o motor deixou de DERIVAR o grafo dos filhos -- arrastar para dentro deixa de entrar"
    );
    let modal = include_str!("input_dispatch/keyboard_modal.rs");
    for (needle, what) in [
        (
            "if !self.morph_preview || self.modifiers.control_key()",
            "TOMAR o teclado enquanto o modo corre (e deixar passar os acordes)",
        ),
        ("self.morph_preview_leave = true", "o Esc PEDIR a saida"),
    ] {
        assert!(
            modal.contains(needle),
            "a porta modal perdeu o `{needle}` -- {what}. A tecla faz DUAS coisas."
        );
    }
    // ⚠️⚠️ **A ORDEM na cadeia é metade do desenho, e este é o gate dela.**
    //
    // A porta tem de correr **DEPOIS** do retrato dos dispositivos (`input.apply_event`) — barrar
    // antes mataria a própria acção que a máquina lê, e o modo ficaria **inerte com o teclado
    // tomado**, que é o pior dos dois mundos — e **ANTES** do primeiro consumidor do editor.
    let kb = include_str!("input_dispatch/keyboard.rs");
    let feed = kb
        .find("self.input.apply_event")
        .expect("o retrato dos dispositivos sumiu");
    let gate = kb
        .find("self.modal_owns_the_keyboard(")
        .expect("a porta modal deixou de ser chamada -- a tecla volta a fazer duas coisas");
    let editor = kb
        .find("ph2d_app_flip::peek::key_transition")
        .expect("o primeiro consumidor do editor sumiu");
    assert!(
        feed < gate,
        "a porta modal corre ANTES do retrato dos dispositivos: a maquina fica MUDA e o teclado \
         fica tomado ao mesmo tempo"
    );
    assert!(
        gate < editor,
        "a porta modal corre DEPOIS de um consumidor do editor: a tecla morfa a forma E faz o que \
         ela faz no editor -- e' o report do Enio de volta"
    );

    // E o painel tem de FORWARDAR os cliques, senão eles morrem antes de chegar aqui.
    let panel = include_str!("../../../crates/ph2d-panel-vector/src/event_clicks.rs");
    for (needle, what) in [
        (
            "ids::VECTOR_MORPH_STATES_MAKE",
            "o botao que faz o conjunto",
        ),
        (
            "ids::VECTOR_MORPH_PREVIEW",
            "o interruptor da pre-visualizacao",
        ),
        ("ids::VECTOR_MORPH_DISSOLVE", "o botao de desfazer tudo"),
        ("ids::morph_shape_play_id(r)", "o Play de cada forma"),
        (
            "ids::morph_shape_disconnect_id(r)",
            "o Desconectar de cada forma",
        ),
        (
            "morph_shape_key_option_id(r, a)",
            "a opcao do menu da condicao",
        ),
    ] {
        assert!(
            panel.contains(needle),
            "o painel deixou de encaminhar {what}: ele acende sob o rato e morre ali"
        );
    }
}

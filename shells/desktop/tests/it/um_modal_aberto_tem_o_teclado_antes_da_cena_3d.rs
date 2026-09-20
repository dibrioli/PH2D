//! ⭐⭐⭐ **ENQUANTO UM MODAL ESTÁ NO ECRÃ, NADA POR BAIXO DELE TEM TECLADO.**
//!
//! # ⛔⛔ O report que isto fecha (Enio, 2026-09-20)
//!
//! *«O modal não captura o que escrevo. O painel lateral captura os atalhos.»*
//!
//! A paleta de comandos é um modal de ecrã cheio com uma caixa de busca, e o doc do
//! `input_dispatch::keyboard_palette` promete por escrito que ela *«vem PRIMEIRO (antes dos
//! atalhos de painel/ferramenta) para uma letra digitada nunca vazar num atalho»*.
//!
//! ⚠️ **Ela estava DUAS LINHAS ABAIXO do `ramo_teclas_3d`**, que com a escultura na mão devolve
//! `true` em `1`–`0`, `G`, `H`, `T`, `S`, `A` e `M` — logo escrever `clay` na busca **trocava o
//! pincel por baixo do modal** e não punha uma letra na caixa. *Uma promessa escrita num ficheiro
//! e violada por duas linhas noutro.*
//!
//! # ⚠️⚠️ É a MESMA CLASSE que aquele ramo já pagou uma vez, com outra pergunta
//!
//! O doc do `ramo_teclas_3d` conta: uma porta que perguntava *«a cena existe?»* passou a comer
//! *«os dez dígitos e ~26 letras de todo painel do app, para sempre»* no dia em que o pill fez a
//! cena sobreviver a sair do modo. A cura de então foi perguntar pelo **PONTEIRO**
//! (`sculpt3d_keys_live`) — e um **MODAL é outra pergunta**: ele não depende de onde o ponteiro
//! está.
//!
//! # ⚠️ Porque este gate lê TEXTO e não dirige uma tecla
//!
//! O `key_input` pede um `winit::KeyEvent`, que **não se constrói num teste** (é a mesma razão
//! pela qual o `player_input` desta shell é uma política pura). O que se pode afirmar sem ele é a
//! **ORDEM DOS RAMOS**, que é exactamente a grandeza que o defeito tinha errada.
//!
//! ⛔ **E o controlo está dentro:** o gate exige que as três âncoras EXISTAM. Sem isso, renomear
//! um ramo deixaria as três buscas a devolver `None` e o teste **verde a afirmar nada** — que é a
//! forma de falha de todo censo textual desta casa.

const KEYBOARD: &str = include_str!("../../src/input_dispatch/keyboard.rs");

/// A posição da primeira ocorrência de uma chamada, ou reprova a dizer que a âncora sumiu.
fn onde(agulha: &str) -> usize {
    KEYBOARD.find(agulha).unwrap_or_else(|| {
        panic!(
            "a âncora `{agulha}` não existe mais no `keyboard.rs` — este gate mede a ORDEM de três \
             ramos, e sem a âncora ele ficaria verde a afirmar nada. Reaponte-o."
        )
    })
}

/// ⭐⭐⭐ **A paleta vê a tecla ANTES da cena 3D.**
///
/// *Mutação que sangra:* voltar a pôr a chamada da paleta depois do `ramo_teclas_3d`.
#[test]
fn a_paleta_ve_a_tecla_antes_da_cena_3d() {
    let paleta = onde("self.command_palette_keys(");
    let cena_3d = onde("self.ramo_teclas_3d(");
    assert!(
        paleta < cena_3d,
        "o `command_palette_keys` está em {paleta} e o `ramo_teclas_3d` em {cena_3d} — com a \
         escultura na mão aquele ramo devolve `true` em `1`-`0`, `G`, `H`, `T`, `S`, `A` e `M`, e \
         uma letra escrita na busca do modal troca o PINCEL em vez de filtrar a lista."
    );
}

/// ⭐⭐ **E antes do `handler.on_key`, que é quem alimenta o dedo do jogador e a fita de input.**
///
/// ⚠️ Pela mesma lei: com um modal aberto, aquilo que o artista escreve não é entrada de jogo.
///
/// *Mutação que sangra:* mover a chamada da paleta para depois do `handler.on_key`.
#[test]
fn a_paleta_ve_a_tecla_antes_do_dedo_do_jogador() {
    let paleta = onde("self.command_palette_keys(");
    let handler = onde("self.handler.on_key(");
    assert!(
        paleta < handler,
        "o `command_palette_keys` está em {paleta} e o `handler.on_key` em {handler} — com um \
         modal aberto, a tecla que o artista escreve não pode chegar ao dedo do jogador."
    );
}

/// ⛔⛔ **A ÚNICA coisa acima dela é o capturador de atalhos, e isso é DECLARADO.**
///
/// O doc dele diz que ele *«É O PRIMEIRO RAMO DESTA FUNÇÃO, E TEM DE SER»* — ele está a ESCUTAR
/// uma tecla para a gravar num mapa de entrada, e os dois nunca estão abertos ao mesmo tempo (o
/// press-to-bind vive na janela do *Input Map*).
///
/// ⚠️ Este gate existe para a excepção ser **consultável** em vez de um acidente de ordem: no dia
/// em que alguém puser um terceiro ramo entre os dois, ele reprova.
#[test]
fn so_o_capturador_de_atalhos_ve_a_tecla_antes_do_modal() {
    let captura = onde("self.capture_binding_if_listening(");
    let paleta = onde("self.command_palette_keys(");
    assert!(
        captura < paleta,
        "o capturador do Input Map tem de continuar a ser o primeiro ramo — ele está a ESCUTAR \
         uma tecla para a gravar."
    );
    // ⚠️ **E nada entre os dois**: a fatia entre eles não pode conter outra chamada `self.<algo>(`
    //    que devolva cedo. A régua é o `return` — é ele que engole a tecla.
    let entre = &KEYBOARD[captura..paleta];
    let returns = entre.matches("return;").count();
    assert_eq!(
        returns, 1,
        "há {returns} `return;` entre o capturador de atalhos e o modal (esperado 1, o do próprio \
         capturador) — um ramo novo no meio volta a roubar o teclado ao modal:\n{entre}"
    );
}

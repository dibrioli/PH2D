//! **O gesto do lápis é INTEIRO dele: press, move, release e a fuga.**
//!
//! As metades vivem no `input_dispatch`, que precisa de janela + GPU — nenhum teste de
//! unidade as alcança, então a asserção é sobre o FONTE. É a 4ª condição da política de UI que a
//! `line/physics` escreveu: *todo edit pode ter gate e o gesto ainda não levar a lugar nenhum*.
//!
//! # O que cada asserção protege
//!
//! 1. **A ordem do press.** O roteador de Down é uma cadeia de modos que dão `return`; se o braço
//!    do lápis vier DEPOIS do da caneta, um arrasto de mão livre cai no `PenTool` e planta uma
//!    âncora — o modo existiria, seria alcançável pelo painel, e desenharia outra coisa.
//! 2. **O move é DESPACHADO.** Sem a chamada no handler de movimento a curva nunca cresce: o
//!    press põe um vértice na cena e o gesto inteiro vira um ponto.
//! 3. **O release COMITA — e é ALCANÇÁVEL.** Sem o commit o traço fica como path vivo sem passo
//!    de undo, e o `post_frame_undo` registraria um passo espúrio pelo diff. A segunda metade é a
//!    asserção que faltava, e que custou um defeito reportado: as outras afirmam que as chamadas
//!    EXISTEM, e existiam — dentro de um ramo que o modo Pencil nunca visita. **Presença não é
//!    alcançabilidade.**
//! 4. **O direito ABORTA.** É a tecla de fuga que a caneta, a forma e o conector já têm; sem ela
//!    um traço começado por acidente não tem como ser descartado.
//!
//! ⚠️ **A asserção é sobre a RELAÇÃO entre as CHAMADAS, nunca sobre distância em bytes.** Dois
//! arch-gates desta linha morreram na integração de 2026-07-23 por afirmarem *"a menos de 400
//! bytes"* / *"a menos de 1200"* — janelas que uma feature vizinha legítima estoura
//! ([[feedback_a_gate_anchored_on_a_byte_distance_is_a_proxy_that_expires]]).

const SRC: &str = include_str!("../src/input_dispatch.rs");

/// A posição da 1ª ocorrência, com uma mensagem que nomeia o que se perdeu.
fn at(needle: &str) -> usize {
    SRC.find(needle)
        .unwrap_or_else(|| panic!("o `input_dispatch` nao contem `{needle}`"))
}

/// A janela do fonte que começa em `from` e termina no próximo `until` (ou no fim do arquivo).
///
/// Afirmar dentro de uma janela é o que permite falar de ORDEM sem falar de distância: o `rustfmt`
/// pode refluir tudo lá dentro e a relação continua a mesma.
fn window(from: usize, until: &str) -> &'static str {
    let rest = &SRC[from..];
    &rest[..rest.find(until).unwrap_or(rest.len())]
}

/// **Controle positivo:** os âncoras existem. Um scanner que não acha nada passaria em silêncio
/// por todas as outras asserções, e este arquivo inteiro seria decoração.
#[test]
fn the_scanner_finds_what_it_scans_for() {
    for needle in [
        "DrawMode::Pencil",
        "self.vec_pencil.on_press(",
        "self.vec_pen.on_press(",
        "self.vec_shape.on_press(",
        "self.vec_pencil.on_release(",
        "if shape_kind_for_mode(&self.vec_draw_config).is_none() {",
    ] {
        assert!(
            SRC.contains(needle),
            "controle positivo falhou: `{needle}` sumiu do dispatch — as assercoes de ORDEM \
             abaixo passariam sem examinar nada"
        );
    }
}

/// **O press do lápis precede o da caneta E o da forma.**
#[test]
fn the_pencil_press_runs_before_the_pen_and_the_shape() {
    let pencil = at("self.vec_pencil.on_press(");
    let pen = at("self.vec_pen.on_press(");
    let shape = at("self.vec_shape.on_press(");
    assert!(
        pencil < pen && pencil < shape,
        "o braco do lapis (byte {pencil}) roda DEPOIS da caneta ({pen}) ou da forma ({shape}) — \
         um arrasto de mao livre cairia no PenTool e plantaria uma ancora"
    );
}

/// **O move do lápis é despachado** (a chamada, não só a função).
///
/// A `fn vec_pencil_drag_move` e a chamada `self.vec_pencil_drag_move(` são strings diferentes de
/// propósito: definir o método e nunca o chamar é exatamente o modo de falha (a função ficaria
/// coberta pelos gates da crate e o produto não cresceria a curva).
#[test]
fn the_pencil_move_is_dispatched() {
    assert!(
        SRC.contains("fn vec_pencil_drag_move"),
        "o metodo de move do lapis nao existe"
    );
    assert!(
        SRC.contains("self.vec_pencil_drag_move("),
        "o move do lapis NUNCA e' chamado — o press poe um vertice na cena e o gesto inteiro \
         vira um ponto"
    );
}

/// **O release comita o passo de undo**, e as duas metades (commit / cancel) existem.
#[test]
fn the_pencil_release_commits_one_undo_step() {
    let release = at("self.vec_pencil.on_release(");
    assert!(
        SRC[release..].contains("commit_if_changed"),
        "o release do lapis nao comita passo de undo nenhum"
    );
    assert!(
        SRC[release..].contains("self.vec_history.cancel()"),
        "o release do lapis nao cancela o passo pendente num clique perdido"
    );
}

/// **O Up do lápis é um braço PRÓPRIO, antes da cadeia de modo** — e é isto que o torna
/// ALCANÇÁVEL.
///
/// ⚠️ **Este gate nasceu vermelho sobre um defeito reportado pelo Enio, com os outros cinco deste
/// arquivo VERDES.** O braço do release vivia no `else` de
/// `shape_kind_for_mode(&self.vec_draw_config).is_none()`, que é **verdadeiro em modo Pencil** (o
/// lápis não é um `ShapeKind`) ⇒ a primeira metade ganhava sempre e o `else if` era **código morto
/// no único modo capaz de o alcançar**. Os cinco afirmavam que as *strings* existem no arquivo, e
/// existiam: **presença não é alcançabilidade** — a mesma família do *registrado ≠ despachado* que
/// os botões Undo/Redo da barra pagaram.
///
/// Os dois defeitos que o ramo morto produzia, e que o Enio viu como um só: o `active` do lápis
/// nunca era limpo, então ele **continuava a desenhar depois de soltar** (todo move seguinte
/// entrava no traço) e o press seguinte **apagava o traço anterior** (o `on_press` remove o path
/// que encontra vivo).
///
/// A asserção é uma RELAÇÃO de posição, nunca uma distância em bytes: se o release corre ANTES da
/// cadeia de modo, ele não pode estar dentro de nenhum ramo dela.
#[test]
fn the_pencil_release_is_its_own_arm_before_the_mode_chain() {
    let release = at("self.vec_pencil.on_release(");
    let mode_chain = at("if shape_kind_for_mode(&self.vec_draw_config).is_none() {");
    assert!(
        release < mode_chain,
        "o release do lapis (byte {release}) corre DEPOIS da cadeia de modo ({mode_chain}) — se \
         ele esta' num ramo dela, o modo Pencil nunca o alcanca: `shape_kind_for_mode` e' None no \
         Pencil, entao a metade `is_none()` ganha sempre e o lapis NUNCA solta (continua a \
         desenhar com o botao em cima, e o proximo traco apaga o anterior)"
    );
}

/// **O botão direito aborta o traço vivo.**
#[test]
fn the_secondary_button_cancels_a_live_pencil_stroke() {
    assert!(
        SRC.contains("self.vec_pencil.cancel("),
        "o lapis nao tem tecla de fuga — um traco comecado por acidente nao pode ser descartado"
    );
}

/// **O estabilizador corre ANTES da conversão para mundo, e o press SEMEIA a mão.**
///
/// Duas asserções sobre a MESMA costura, e cada uma protege um defeito diferente:
///
/// 1. **Filtrar depois do `screen_to_world`** deixaria o número do slider a significar coisas
///    diferentes em zooms diferentes no dia em que o filtro ganhasse qualquer termo absoluto — e o
///    tremor é um fato da mão sobre a mesa, medido em px de tela.
/// 2. **Sem a semente no press** o 1º move mistura a partir de onde o gesto ANTERIOR acabou: o
///    traço nasce com um salto vindo do outro lado da tela, e o modo de falha é pior no 2º traço,
///    ou seja depois de o artista já ter aprovado o 1º.
///
/// ⚠️ As duas correm dentro de uma JANELA (a função de move · o braço do press), nunca sobre uma
/// distância em bytes nem sobre um recorte de linha que o `rustfmt` reflui.
#[test]
fn the_stabiliser_filters_screen_px_and_the_press_seeds_the_hand() {
    // ── a função de MOVE: o filtro antes da conversão ──
    let mv = window(at("fn vec_pencil_drag_move"), "\n    fn ");
    let filter = mv.find("vec_pencil_hand").expect(
        "o move do lapis nao passa pela mao filtrada — o estabilizador nao chega ao produto",
    );
    let to_world = mv
        .find("screen_to_world")
        .expect("o move do lapis nao converte para mundo");
    assert!(
        filter < to_world,
        "o estabilizador corre DEPOIS do `screen_to_world` — o filtro tem de ver px de TELA, que \
         e' onde o tremor da mao tem tamanho"
    );

    // ── o braço do PRESS: a semente mora nele ──
    let press = window(
        at("DrawMode::Pencil {"),
        "\n                    // Modo Connect",
    );
    assert!(
        press.contains("self.vec_pencil.on_press("),
        "controle positivo: a janela do press nao contem o proprio press"
    );
    assert!(
        press.contains("self.vec_pencil_hand.begin("),
        "o braco do press NAO semeia a mao — o 1o move mistura a partir do fim do gesto ANTERIOR e \
         o traco nasce com um salto vindo do outro lado da tela"
    );
}

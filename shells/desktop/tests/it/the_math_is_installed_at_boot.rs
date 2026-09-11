//! **Arch-gate: a capacidade de fórmula é INSTALADA, e antes de haver janela** (W4c.3).
//!
//! # Por que isto é um arch-gate e não um teste de unidade
//!
//! O host da math mora num `thread_local` da `ph2d-tokens`, e **toda suíte que precisa dele o
//! instala ela própria** — a da `ph2d-token-math` porque é o assunto dela, as de painel porque
//! precisam de ver o botão. Apague a linha do produto e a workspace inteira fica **VERDE** com o
//! `f(x)` a nunca aparecer no app: `math_available()` devolve falso, o painel não oferece o botão,
//! e nada erra.
//!
//! ⚠️ É o modo de falha CERTO (um controlo que não existe em vez de um que não faz nada — o padrão
//! do `set_ml_available`), e é exactamente por ser silencioso que ele precisa deste gate.
//!
//! ⚠️ E a POSIÇÃO é metade da afirmação: a instalação corre na thread do laço de eventos, que é a
//! que pinta. Instalá-la numa thread de trabalho deixaria o `thread_local` da thread da UI vazio, e
//! o sintoma seria idêntico ao de não a instalar de todo.

use std::fs;

const MAIN: &str = "src/main.rs";

fn read(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

#[test]
fn the_shell_installs_the_math_before_it_builds_the_app() {
    let file = read(MAIN);

    // ⚠️ A varredura é do CORPO do `fn main`, e não do arquivo. A 1ª versão deste gate procurava
    // no arquivo inteiro e falhou por casar com **o próprio comentário que esta wave escreveu**
    // ("antes do `App::new()`") — *um oráculo que casa com a documentação de si mesmo não está a
    // olhar para o produto*, a cicatriz que a linha do Painter já pagou.
    let start = file
        .find("fn main() {")
        .expect("o shell nao tem `fn main` — re-ancore este gate");
    let src = &file[start..];

    let install = src.find("ph2d_token_math::install()").expect(
        "o shell nao instala a math dos tokens — o painel nunca oferece o botao `f(x)` e a \
         workspace inteira fica verde, porque toda suite que precisa do host o instala ela propria",
    );

    // ⚠️ A âncora é a construção do `App`, e não uma distância em bytes: um gate ancorado numa
    // janela de bytes expira quando alguém acrescenta uma linha no meio (a cicatriz que os dois
    // arch-gates desta linha já pagaram em 23/07).
    // ⚠️ E a âncora é a LIGAÇÃO (`let mut app = App::new()`), que só o código tem — a chamada nua
    // aparece também em prosa, e foi assim que a 1ª versão se enganou.
    let app_new = src
        .find("let mut app = App::new()")
        .expect("o `fn main` mudou de forma — re-ancore este gate no sitio que CONSTROI o app");

    assert!(
        install < app_new,
        "a math e' instalada depois de o app existir: qualquer coisa que o `App::new` pinte ou \
         pergunte veria a capacidade ausente"
    );
}

/// **Controle positivo.** Sem ele, um `MAIN` que deixasse de existir faria o `read` explodir — mas
/// um `MAIN` que existisse e estivesse VAZIO faria o gate acima falhar pelo motivo certo, e este
/// prova que a varredura de facto lê o produto em vez de um arquivo vazio.
#[test]
fn the_gate_is_reading_the_product() {
    let src = read(MAIN);
    assert!(
        src.contains("fn main()"),
        "o arquivo lido nao e' o `fn main` do shell"
    );
}

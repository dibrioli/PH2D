//! ⭐ Os arch-gates da **TABELA SINAL → PAPEL** (item 4 do estudo dos contêineres).
//!
//! # Por que sobre o FONTE, e não por comportamento
//!
//! A composição *nome → alvos → pedido* tem gate próprio no `ui_state_bridge`, e ele passa com a
//! feature **inteiramente desligada**: o consumidor mora dentro do `run_render_frame`, que precisa
//! de janela e de GPU, e nenhum teste de unidade o alcança. É literalmente a lição que esta casa
//! pagou com vinte gates verdes sobre um `draw` cravado em `true` — *um gate de unidade é cego à
//! fiação da shell*.
//!
//! O que se afirma aqui é a **PROPRIEDADE**, nunca um endereço: que a leitura existe, que ela
//! atravessa a tabela, e que a AÇÃO é gateada na preview enquanto o CURSOR não é.

fn frame_src() -> String {
    std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/render_loop/mod.rs"
    ))
    .expect("o laço de frame existe")
}

/// **O consumidor existe e passa pela tabela.**
#[test]
fn the_frame_reads_the_outbox_and_asks_the_table_who_listens() {
    let s = frame_src();
    let read = s
        .find("read(&mut self.ui_signal_reader)")
        .expect("o frame lê a saída de sinais com o cursor da UI");
    let targets = s[read..]
        .find("ui_states.targets(")
        .expect("e pergunta à tabela quem escuta aquele nome");
    let request = s[read + targets..]
        .find("ui_state_bridge::request")
        .expect("e o alvo vira um PEDIDO, nunca uma escrita de pose direta");
    assert!(
        request > 0,
        "a ordem é ler → perguntar → pedir; ela é o caminho inteiro"
    );
}

/// ⭐ **O CURSOR ANDA SEMPRE E A AÇÃO SÓ CORRE NA PREVIEW** — as duas metades, e elas são
/// afirmações diferentes.
///
/// ⚠️ **A metade do cursor é a que um gate ingênuo esqueceria.** Se a leitura estivesse DENTRO de
/// um `if preview`, o leitor acumularia `missed` enquanto o modo está desligado e entrar na
/// preview entregaria de uma vez os dois quadros que a janela do outbox ainda guarda — a cena
/// saltaria de pose por um sinal anterior ao gesto do artista. O `filter` sobre o iterador é o
/// que separa *ler* de *agir*.
#[test]
fn the_cursor_always_advances_and_only_the_action_is_gated_on_the_preview() {
    let s = frame_src();
    let read = s
        .find("read(&mut self.ui_signal_reader)")
        .expect("a leitura existe");
    // A janela é o bloco do consumidor — generosa, e o que importa é o que está DENTRO dela.
    let win = &s[read.saturating_sub(400)..(read + 400).min(s.len())];
    assert!(
        win.contains("self.ui_preview.is_on()"),
        "a ação não é gateada na preview: um sinal moveria o desenho do artista enquanto ele \
         edita, e fora da preview nada o restaura"
    );
    let gate = win
        .find("self.ui_preview.is_on()")
        .expect("a guarda existe");
    let at = win
        .find("read(&mut self.ui_signal_reader)")
        .expect("a leitura");
    assert!(
        gate < at,
        "a guarda tem de ser um VALOR calculado antes da leitura (o `filter`), e não um `if` à \
         volta dela — dentro de um `if` o cursor pararia de andar"
    );
}

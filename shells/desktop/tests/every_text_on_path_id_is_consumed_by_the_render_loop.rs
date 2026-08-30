//! **Arch-gate: os ids do Text on Path são CONSUMIDOS pela `render_loop`.**
//!
//! O gate de seam do painel (`ph2d-panel-vector/tests/seam.rs`) prova que o clique chega ao
//! **barramento**. Isso é metade: um id pode chegar ao barramento e **morrer lá**, porque
//! ninguém do outro lado o lê — o botão pinta, arma, despacha, e não acontece nada.
//!
//! É exatamente a classe de bug que o repo já pagou com os botões Undo/Redo da barra (*"o Redo
//! não despachava coisa alguma — pintado, clicável, órfão, com um gate ao lado afirmando que
//! ele estava no store: **registrado ≠ despachado**"*).
//!
//! Nenhum teste de unidade alcança a `render_loop` (ela precisa de janela e GPU), então a
//! prova é sobre o FONTE — o mesmo recurso que a linha de física usou para pinar que o Join
//! não faz fan-out, e que o load da física instala as settings depois do `rebuild`.
//!
//! ⚠️ **Contador de controle positivo:** se o scanner deixar de encontrar os ids por uma
//! mudança de forma (um `match` no lugar do `else if`, um `use` que encurta o caminho), ele
//! passa a guardar nada — e um gate que não vê nada passa sempre. Por isso ele exige encontrar
//! os CINCO, e falha nomeando qual faltou.
//!
//! ⛔ **E havia um SEXTO modo de falha, que o controle acima não cobria: a agulha casava como
//! SUBCADEIA.** `VECTOR_TEXTPATH_FLIP` é prefixo de `VECTOR_TEXTPATH_FLIP_OFF`, e os dois ramos
//! são vizinhos no mesmo `else if` — apagar o ramo do `VECTOR_TEXTPATH_FLIP` deixava o botão
//! *«Other side»* **pintado, clicável e órfão** (a classe que o doc acima nomeia) com este gate
//! VERDE, porque o irmão `_OFF` continuava a satisfazer o `contains`. A agulha passa agora pela
//! [`consumes`], que exige fronteira de identificador dos dois lados.
//!
//! ⚠️ **A cura é da FAMÍLIA, não do caso:** as CINCO agulhas passam pela mesma porta. Duas delas
//! são ambíguas por prefixo hoje — `VECTOR_TEXTPATH_FLIP` (mascarada por
//! `VECTOR_TEXTPATH_FLIP_OFF`, que já vive neste arquivo) e `VECTOR_TEXTPATH_OFFSET` (o irmão
//! `VECTOR_TEXTPATH_OFFSET_NUM` existe em `ids/chrome/vector_textpath.rs` e ainda não chegou
//! aqui — no dia em que chegar, a agulha do slider ficaria cega sem uma linha de aviso).

use std::fs;

/// Os ids que o painel manda para o barramento, e que a `render_loop` tem de ler.
///
/// O Offset entra: ele é um `ValueChanged`, não um `Click`, e o modo de falha é o mesmo (o
/// slider anda na tela e o documento não muda).
const CONSUMED: &[&str] = &[
    "VECTOR_TEXTPATH_LINK",
    "VECTOR_TEXTPATH_DETACH",
    "VECTOR_TEXTPATH_FLIP",
    "VECTOR_TEXTPATH_FLIP_OFF",
    "VECTOR_TEXTPATH_OFFSET",
];

/// **O nome casa como PALAVRA** — fronteira de identificador (`[A-Za-z0-9_]`) dos dois lados.
///
/// ⚠️ Um `contains` cru responde *«esta fonte tem estas letras?»*; a pergunta do gate é *«esta
/// fonte NOMEIA este id?»*, e as duas divergem exatamente quando um id é prefixo de outro — que é
/// a forma que a família `_FLIP` / `_FLIP_OFF` e `_OFFSET` / `_OFFSET_NUM` têm por construção
/// (o par exclusivo escreve-se acrescentando um sufixo ao nome do irmão).
fn consumes(src: &str, id: &str) -> bool {
    fn is_word(c: char) -> bool {
        c.is_alphanumeric() || c == '_'
    }
    // ⚠️ A cerca é do PRÓPRIO helper: ele só sabe responder sobre um NOME. Apontá-lo a uma agulha
    // com pontuação (`vec_text_ride::link(`) exigiria que o caractere seguinte ao `(` fosse
    // não-identificador, e ele quase nunca é — o gate ficaria vermelho sobre produto correcto.
    assert!(
        id.chars().all(is_word),
        "`consumes` responde sobre um NOME, e `{id}` não é um"
    );
    src.match_indices(id).any(|(at, _)| {
        src[..at].chars().next_back().is_none_or(|c| !is_word(c))
            && src[at + id.len()..]
                .chars()
                .next()
                .is_none_or(|c| !is_word(c))
    })
}

#[test]
fn every_text_on_path_id_is_read_by_the_render_loop() {
    let src = fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/render_loop/mod.rs"
    ))
    .expect("render_loop/mod.rs");

    let missing: Vec<&str> = CONSUMED
        .iter()
        .copied()
        .filter(|id| !consumes(&src, id))
        .collect();
    assert!(
        missing.is_empty(),
        "estes ids do Text on Path chegam ao barramento e MORREM lá — a `render_loop` nunca os \
         lê, então o controle pinta, arma, despacha e não faz nada: {missing:?}"
    );

    // …e o outro lado: cada um tem de acabar numa PORTA do `vec_text_ride`, não numa ação
    // inventada no meio do laço. As quatro portas são a superfície inteira da feature.
    for door in [
        "vec_text_ride::link(",
        "vec_text_ride::detach(",
        "vec_text_ride::edit(",
    ] {
        assert!(
            src.contains(door),
            "a `render_loop` não chama `{door}` — o comando chegou e não tem quem o execute"
        );
    }
}

/// **O controle da agulha** — os dois sentidos, sobre o texto REAL do ramo que a mutação apaga.
///
/// ⚠️ Sem ele o [`consumes`] podia ser um `|_, _| true` e o gate acima voltaria a ser verde por
/// vácuo — que é a doença que ele foi curar.
#[test]
fn the_id_does_not_match_inside_a_longer_id() {
    let alive = "} else if *id == ph2d_editor::ids::VECTOR_TEXTPATH_FLIP {\n\
                 } else if *id == ph2d_editor::ids::VECTOR_TEXTPATH_FLIP_OFF {";
    assert!(consumes(alive, "VECTOR_TEXTPATH_FLIP"));
    assert!(consumes(alive, "VECTOR_TEXTPATH_FLIP_OFF"));

    // A mutação: o ramo do `FLIP` apagado, o do `_OFF` intacto. O `contains` cru lia VERDE.
    let orphaned = "} else if *id == ph2d_editor::ids::VECTOR_TEXTPATH_FLIP_OFF {";
    assert!(
        !consumes(orphaned, "VECTOR_TEXTPATH_FLIP"),
        "o botao «Other side» orfao tem de ler-se como AUSENTE"
    );
    assert!(consumes(orphaned, "VECTOR_TEXTPATH_FLIP_OFF"));

    // O gêmeo latente: o slider e o campo numérico dele.
    let only_num = "*id == ph2d_editor::ids::VECTOR_TEXTPATH_OFFSET_NUM";
    assert!(!consumes(only_num, "VECTOR_TEXTPATH_OFFSET"));
}

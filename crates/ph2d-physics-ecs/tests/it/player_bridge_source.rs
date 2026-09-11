//! **O TEXTO da ponte do player** — a porta única que os arch-gates deste
//! assunto leem.
//!
//! ⚠️ **Ela existe porque o assunto é uma FAMÍLIA, não um arquivo.** O laço de
//! `drive_players` é cortado por responsabilidade sempre que bate o teto de LOC
//! (`player_leg` · `player_probes` · `player_marks` · `player_push` ·
//! `player_kinmove`), e um gate ancorado em `src/bridge/player.rs` **reprova
//! sobre produto correto** no dia do corte seguinte — que foi exatamente o que
//! aconteceu quando o `kinematic_settle` se mudou para o irmão. *Afirme a
//! PROPRIEDADE, nunca o endereço.*
//!
//! ⚠️ **Incluída por `#[path]` nos dois gates, e não copiada**: cada arquivo em
//! `tests/` é um crate próprio, e duas cópias de *"que texto é a ponte do
//! player?"* divergiriam no primeiro corte que uma delas não visse.
//!
//! ⚠️ **O `allow(dead_code)` é do formato**, pela razão que o `platform_scene`
//! já escreve: cada consumidor usa o subconjunto de que precisa.

#![allow(dead_code)]

use std::fs;

/// Um arquivo do crate, pelo caminho relativo ao `Cargo.toml`.
pub fn read(rel: &str) -> String {
    let path = format!("{}/{rel}", env!("CARGO_MANIFEST_DIR"));
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// **O arquivo do LAÇO** — o assunto de quem afirma ORDEM dentro de uma função.
///
/// ⚠️ Ordem entre dois pontos só significa alguma coisa dentro de um arquivo:
/// afirmá-la sobre a família seria afirmar a ordem alfabética dos irmãos.
pub fn player_loop() -> String {
    read("src/bridge/player.rs")
}

/// **A FAMÍLIA inteira**, em ordem de nome — o assunto de quem CONTA (quantos
/// leitores existem, onde uma chamada mora).
pub fn player_family() -> String {
    let dir = format!("{}/src/bridge", env!("CARGO_MANIFEST_DIR"));
    let mut files: Vec<_> = fs::read_dir(&dir)
        .expect("o diretorio da ponte tem de existir")
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n == "player.rs" || (n.starts_with("player_") && n.ends_with(".rs")))
        .collect();
    files.sort();
    assert!(
        files.len() > 1,
        "a familia tem o pai e os irmaos; achei {files:?}"
    );
    files
        .iter()
        .map(|n| fs::read_to_string(format!("{dir}/{n}")).expect("o irmao tem de ser legivel"))
        .collect::<Vec<_>>()
        .join("\n")
}

//! ⛔⛔⛔ **CENSO: «que textura usa esta peça?» tem UMA porta** — e ela chama-se `texture_of`.
//!
//! # A doença, medida quatro vezes
//!
//! Uma peça carrega a textura de **duas** formas, e elas não são meio-a-meio:
//!
//! - **átlas** (`Sprite::source == Atlas`) — o caminho de **toda** imagem importada e de toda tela
//!   nova, ou seja a esmagadora maioria;
//! - **`SpritePixels`** — o carimbo, a minoria.
//!
//! ⇒ um sítio que pergunta só por `SpritePixels` **compila, corre, e responde vazio sobre o
//! caminho normal**. Não há erro, não há aviso, e o gate que usar uma fixtura de `SpritePixels`
//! fica verde. Foi assim quatro vezes, sempre com um report do Enio a fechar o ciclo:
//!
//! | # | o que ficou mudo | como se leu |
//! |---|---|---|
//! | 1 | a cor dominante de um prefab | *«não funcionou»* — cartão cinzento |
//! | 2 | a peça-cara (`largest_piece_texture`) | idem |
//! | 3 | os utilizadores de uma imagem (`users_of`) | *«Selected 1 object(s)»* e nada acendia |
//! | 4 | as **dependências** de um prefab | *«não conseguiu listar os ítens»* |
//!
//! ⚠️ **A 4.ª aconteceu a QUINZE LINHAS da porta**, num ficheiro onde ela já era chamada pelo
//! vizinho, com um doc-comment a prometer que uma terceira forma entraria ali sem partir nada.
//! *Uma porta que o vizinho não chama ainda não é uma porta — é uma função.* Este censo é o que a
//! torna uma porta.

use std::fs;
use std::path::{Path, PathBuf};

fn shell_src() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// As linhas de CÓDIGO de `path` (a prosa é descascada: estas notas NOMEIAM o padrão proibido de
/// propósito, e apagar a explicação para calar uma varredura é trocar a coisa certa pela medível).
fn code_lines(path: &Path) -> Vec<(usize, String)> {
    let Ok(text) = fs::read_to_string(path) else {
        return Vec::new();
    };
    text.lines()
        .enumerate()
        .filter(|(_, l)| !l.trim_start().starts_with("//"))
        .map(|(i, l)| (i + 1, l.to_string()))
        .collect()
}

/// ⛔ **Ninguém nestes ficheiros DECIDE pela forma da textura — todos entregam à porta.**
///
/// ⚠️⚠️ **A régua olha as LEITURAS, e não a palavra.** A 1.ª versão procurava `SpritePixels` solto
/// e acusou três inocentes: a linha do `use`, e uma `query::<(…, Option<&SpritePixels>,
/// Option<&ph2d_render::Sprite>)>` que pede **as duas formas** para as dar ao `texture_of` — que é
/// exactamente o padrão certo. *Um gate que parseia o fonte tem de saber todas as formas do que
/// parseia, senão acusa o inocente e cega-se ao culpado.*
///
/// ⇒ a acusação é uma **leitura por entidade** (`get::<SpritePixels>`) ou uma **consulta** que peça
/// `SpritePixels` **sem** pedir também o `Sprite`; e as duas são perdoadas quando o `texture_of`
/// aparece na janela à volta.
#[test]
fn the_index_asks_the_texture_door() {
    // Os ficheiros que respondem «que textura é esta peça?» — o construtor do índice, o que
    // desenha o retrato, e os verbos do cartão.
    const WATCHED: [&str; 3] = [
        "asset_index_build.rs",
        "asset_card_art.rs",
        "asset_card_verbs.rs",
    ];
    /// Quantas linhas à volta contam como «a chamada está ali» — o `fmt` parte uma chamada de
    /// quatro argumentos em cinco linhas.
    const WINDOW: usize = 6;

    let mut guilty: Vec<String> = Vec::new();
    for name in WATCHED {
        let path = shell_src().join(name);
        let lines = code_lines(&path);
        assert!(
            !lines.is_empty(),
            "{name} nao foi lido — o censo mediria nada"
        );
        // A definição da porta é a única leitura legítima sem chamada ao lado.
        let mut in_door = false;
        for (n, l) in &lines {
            if l.contains("fn texture_of(") {
                in_door = true;
            }
            if in_door && l.starts_with('}') {
                in_door = false;
            }
            if in_door {
                continue;
            }
            let reads_one_by_one = l.contains("get::<SpritePixels>");
            // ⚠️ Uma consulta que peça `SpritePixels` E o `Sprite` está a alimentar a porta; uma
            // que peça só a primeira é a doença, em forma de consulta.
            let queries_only_pixels = l.contains("query")
                && l.contains("SpritePixels")
                && !l.contains("ph2d_render::Sprite");
            if !(reads_one_by_one || queries_only_pixels) {
                continue;
            }
            let fed_to_the_door = lines
                .iter()
                .filter(|(m, _)| m.abs_diff(*n) <= WINDOW)
                .any(|(_, w)| w.contains("texture_of("));
            if !fed_to_the_door {
                guilty.push(format!("{name}:{n}: {}", l.trim()));
            }
        }
    }
    assert!(
        guilty.is_empty(),
        "leituras de `SpritePixels` que NAO entregam a` porta `texture_of` — cada uma fica MUDA \
         sobre toda sprite de atlas, que e' o caminho normal de toda imagem importada:\n  {}",
        guilty.join("\n  ")
    );
}

//! **Duas cenas de smoke não podem reclamar o mesmo número.**
//!
//! O roteador é uma lista de `if level == N` e o **primeiro vence**. Uma colisão não é um erro de
//! compilação, não é um warning, e não é visível em lado nenhum: a segunda cena fica
//! **inalcançável em silêncio**, e quem digita o número dela vê a outra rodar.
//!
//! ⚠️ **Isto aconteceu** (2026-08-02): a wave do AUTO LAYOUT tomou o `=50`, que já era da cena dos
//! TOKENS. `PH2D_BUILD_SMOKE=50` passou a correr o layout, e a cena dos tokens deixou de existir
//! para o artista — sem nada na tela a dizer porquê. Foi achado escrevendo o handoff de
//! integração, ao conferir *"as cenas que eu afirmo existir existem mesmo?"*, e não por um gate.
//!
//! O modo de falha que este arquivo fecha é exactamente esse: a **próxima** colisão falha alto.

use std::collections::BTreeMap;
use std::fs;

/// O roteador é a fonte: um `if level == N` por cena.
#[test]
fn no_two_smoke_scenes_claim_the_same_level() {
    let src = fs::read_to_string("src/build_smoke_router.rs").expect("build_smoke_router.rs");

    // `level == N` → a linha em que ele aparece, para o erro apontar as DUAS.
    let mut claims: BTreeMap<u32, Vec<usize>> = BTreeMap::new();
    for (i, line) in src.lines().enumerate() {
        let Some(rest) = line.split("level == ").nth(1) else {
            continue;
        };
        let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
        if let Ok(n) = digits.parse::<u32>() {
            claims.entry(n).or_default().push(i + 1);
        }
    }

    assert!(
        claims.len() >= 10,
        "a varredura achou {} cenas — o roteador mudou de forma e o gate esta' a medir nada",
        claims.len()
    );

    let dupes: Vec<String> = claims
        .iter()
        .filter(|(_, lines)| lines.len() > 1)
        .map(|(n, lines)| format!("=({n}) nas linhas {lines:?}"))
        .collect();
    assert!(
        dupes.is_empty(),
        "duas cenas de smoke reclamam o MESMO numero, e o primeiro `if` vence — a segunda fica \
         inalcancavel em silencio: {}",
        dupes.join(" · ")
    );
}

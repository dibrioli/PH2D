//! **Duas cenas do smoke de OBJETO não podem reclamar o mesmo modo.** O irmão do
//! `no_two_smoke_scenes_claim_the_same_level`, sobre o outro roteador.
//!
//! ⚠️ **Ele não estava coberto, e o mecanismo é o mesmo — pior.** Aquele gate lê o
//! `build_smoke_router.rs` (`if level == N`); este roteador é um `match mode` com guardas
//! (`9 if f == 3 => …`), e **dois braços com o mesmo par `(modo, frame)` não são um erro
//! nem um warning**: o primeiro vence e a segunda cena fica inalcançável em silêncio.
//! `match` com guarda não é exaustivo, então o compilador não pode dizer nada.
//!
//! A família já pagou este defeito uma vez (2026-08-02, a wave do auto layout tomou o
//! `=50` da cena dos tokens, achado a escrever um handoff e não por um gate). Este
//! arquivo fecha a mesma porta no roteador que faltava — aberto quando o `=9` (o estilo
//! do sink, doc 89 folha 17) entrou.

use std::collections::BTreeMap;
use std::fs;

#[test]
fn no_two_object_smoke_modes_claim_the_same_number() {
    let src = fs::read_to_string("src/motion_object_smoke.rs").expect("motion_object_smoke.rs");

    // `<modo> if f == <frame> =>` → as linhas em que o par aparece.
    let mut claims: BTreeMap<(u32, u32), Vec<usize>> = BTreeMap::new();
    for (i, line) in src.lines().enumerate() {
        let t = line.trim();
        let Some((head, tail)) = t.split_once(" if f == ") else {
            continue;
        };
        let Ok(mode) = head.parse::<u32>() else {
            continue;
        };
        let frame: String = tail.chars().take_while(char::is_ascii_digit).collect();
        if let Ok(frame) = frame.parse::<u32>() {
            claims.entry((mode, frame)).or_default().push(i + 1);
        }
    }

    // Controle positivo: uma varredura vazia passaria por vácuo.
    assert!(
        claims.len() >= 8,
        "a varredura achou {} bracos — o roteador mudou de forma e o gate mede nada",
        claims.len()
    );

    let dupes: Vec<String> = claims
        .iter()
        .filter(|(_, lines)| lines.len() > 1)
        .map(|((m, f), lines)| format!("=({m}) no frame {f}, linhas {lines:?}"))
        .collect();
    assert!(
        dupes.is_empty(),
        "dois bracos reclamam o MESMO (modo, frame) e o primeiro vence — a segunda cena \
         fica inalcancavel em silencio: {}",
        dupes.join(" · ")
    );
}

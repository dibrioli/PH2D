//! **Duas cenas do módulo 3D não podem reclamar o mesmo número.**
//!
//! ⚠️ **O irmão `no_two_smoke_scenes_claim_the_same_level` NÃO cobre isto**, e a diferença é o
//! mecanismo: lá o roteador é uma lista de `if level == N` num arquivo só; aqui cada cena é uma
//! função que compara o **próprio literal** contra `PH2D_SCULPT3D_SMOKE`, e elas moram em dois
//! arquivos. Um gate que varre o roteador do vetor passa VERDE sobre uma colisão aqui.
//!
//! O modo de falha é o mesmo, e é mudo: `scene_objects` pergunta às cenas **em ordem** e a primeira
//! que disser sim vence, então a segunda fica inalcançável em silêncio — quem digita o número dela
//! vê a outra rodar, sem nada na tela a dizer porquê.
//!
//! ⚠️ **Isto já aconteceu no repo, do lado do vetor** (2026-08-02: a wave do AUTO LAYOUT tomou o
//! `=50`, que era dos TOKENS), e foi achado a escrever um handoff — não por um gate. Este arquivo
//! fecha a **próxima**, do lado 3D, antes de ela custar um smoke.

use std::collections::BTreeMap;
use std::fs;

/// Todo arquivo do shell que pode declarar uma cena do módulo 3D.
///
/// ⚠️ **Uma FAMÍLIA e não dois nomes**, pelo motivo que esta linha já pagou duas vezes: as cenas de
/// sombreamento saíram do pai para um irmão quando ele cruzou o teto de LOC, e um gate que nomeasse
/// os arquivos ficaria cego ao terceiro corte. O `src/` inteiro é varrido, e o controle positivo
/// abaixo é o que impede a varredura de medir nada.
const ROOT: &str = "src";

#[test]
fn no_two_sculpt3d_scenes_claim_the_same_level() {
    let mut claims: BTreeMap<u32, Vec<String>> = BTreeMap::new();
    let mut walked = 0usize;
    let mut stack = vec![std::path::PathBuf::from(ROOT)];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).expect("o src/ do shell existe") {
            let path = entry.expect("entrada legível").path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().is_none_or(|e| e != "rs") {
                continue;
            }
            walked += 1;
            let src = fs::read_to_string(&path).expect("arquivo legível");
            let name = path.display().to_string();
            for line in src.lines() {
                // A forma canônica de uma cena: comparar o env var contra o próprio literal.
                let Some(rest) = line.split("PH2D_SCULPT3D_SMOKE").nth(1) else {
                    continue;
                };
                let Some(after) = rest.split("Some(\"").nth(1) else {
                    continue;
                };
                let digits: String = after.chars().take_while(char::is_ascii_digit).collect();
                if let Ok(n) = digits.parse::<u32>() {
                    claims.entry(n).or_default().push(name.clone());
                }
            }
        }
    }

    // **Controle positivo**, nas duas pontas: uma varredura que não achasse arquivo nenhum — ou que
    // achasse arquivos e nenhuma cena — passaria por vácuo, que é a falha que este gate existe para
    // não ter.
    assert!(walked > 20, "a varredura leu {walked} arquivos — o `src/` mudou de lugar");
    assert!(
        claims.len() >= 15,
        "achei {} cenas do modulo 3D — a forma de declarar uma mudou e o gate esta' a medir nada",
        claims.len()
    );

    let dupes: Vec<String> = claims
        .iter()
        .filter(|(_, files)| files.len() > 1)
        .map(|(n, files)| format!("=({n}) em {files:?}"))
        .collect();
    assert!(
        dupes.is_empty(),
        "duas cenas reclamam o mesmo numero, e a segunda esta' INALCANCAVEL: {}",
        dupes.join(" · ")
    );
}

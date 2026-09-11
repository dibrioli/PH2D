//! **Uma cena do módulo 3D que não é ANUNCIADA nasce muda.**
//!
//! Toda cena desta linha carrega um roteiro (`announce`) que diz ao artista o que julgar, e os
//! números dele saem do motor de propósito — é o roteiro que torna o smoke *julgável* em vez de
//! *olhado*. Mas o roteiro só chega à tela se alguém o **chamar**, e o chamador vive noutro arquivo
//! (`sculpt3d_scripts.rs`): escrever a cena e esquecer a linha compila, passa em toda a suíte, e
//! entrega ao Enio uma janela sem instruções.
//!
//! ⚠️ **É o irmão do `no_two_sculpt3d_scenes_claim_the_same_level`, e o modo de falha é o mesmo
//! tipo de silêncio** — lá a cena existe e é inalcançável, aqui ela é alcançável e não se
//! apresenta. Aquele gate nasceu de uma colisão achada a escrever um handoff; este fecha a metade
//! que ele não vê.
//!
//! ⚠️ **A varredura é da FAMÍLIA `src/` inteira**, e não de uma lista de nomes: as cenas já se
//! mudaram de arquivo duas vezes por teto de LOC, e um gate que enumerasse os arquivos ficaria cego
//! ao próximo corte.

use std::collections::BTreeSet;
use std::fs;

const ROOT: &str = "src";

/// Os módulos que DEFINEM um roteiro, e o texto inteiro do shell onde as chamadas vivem.
fn scan() -> (BTreeSet<String>, String, usize) {
    let mut defs = BTreeSet::new();
    let mut all = String::new();
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
            // A forma canônica de um roteiro: `pub(crate) fn announce()` num arquivo de cena.
            if src.contains("pub(crate) fn announce()")
                && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
                && let Some(tail) = stem.strip_prefix("sculpt3d_scenes_")
            {
                defs.insert(tail.to_string());
            }
            all.push_str(&src);
        }
    }
    (defs, all, walked)
}

#[test]
fn every_sculpt3d_scene_script_is_announced() {
    let (defs, all, walked) = scan();

    // **Controle positivo, nas duas pontas.** Uma varredura que lesse nada — ou que achasse
    // arquivos e nenhum roteiro — passaria por vácuo, que é exactamente a falha que este gate
    // existe para não ter.
    assert!(
        walked > 20,
        "a varredura leu {walked} arquivos — o `src/` mudou de lugar"
    );
    assert!(
        defs.len() >= 2,
        "achei {} roteiros de cena — a forma de declarar um mudou e o gate esta' a medir nada: {defs:?}",
        defs.len()
    );

    let mute: Vec<&String> = defs
        .iter()
        .filter(|m| !all.contains(&format!("scenes::{m}::announce()")))
        .collect();
    assert!(
        mute.is_empty(),
        "estes roteiros existem e NINGUEM os chama — a cena abre sem instrucoes: {mute:?}"
    );
}

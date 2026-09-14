//! ⭐⭐ **TODA CENA QUE ESCREVE UM ROTEIRO TEM QUEM O IMPRIMA** — o censo que
//! teria apanhado a `=42`.
//!
//! # O defeito, medido (2026-09-14)
//!
//! A cena do pincel de contorno nasceu com o roteiro de seis passos escrito e
//! **nenhum chamador**: o despacho em [`crate::scripts`] enumera as cenas **à
//! mão**, e acrescentar um ficheiro novo não acrescenta uma linha lá. O artista
//! abriria a cena e veria a peça sem uma palavra sobre onde clicar — que é a
//! metade do smoke em que ele **aprende** a ferramenta (`CLAUDE.md` §0.8).
//!
//! ⚠️⚠️ **O que o apanhou desta vez foi um `warning: function is never used`, e
//! isso é sorte que não se repete:** a função é `pub(crate)`, logo o `dead_code`
//! ainda a alcança — bastava ela ser `pub`, ou ser citada por um teste, e o
//! aviso desaparecia com o roteiro na mesma mudo. *Um aviso do compilador não é
//! um gate: ele mede visibilidade, não a lei.*
//!
//! # A régua
//!
//! Todo ficheiro `scenes_*.rs` desta crate que declara `fn announce(` sem
//! argumentos tem de ser citado por uma linha `crate::scenes::<nome>::announce()`
//! no despacho. ⚠️ **O piso de população é obrigatório** — um censo que varra
//! zero ficheiros devolve `vazio.is_empty() == true` e lê-se como aprovado
//! (`CLAUDE.md` §5.0, a espécie MUDA de gate partido).
//!
//! ⚠️ **O que ela NÃO vê, e é declarado:** um `announce` que recebe argumentos
//! (o [`crate::announce`] da malha é outro assunto e vive noutro ficheiro), e um
//! roteiro escrito **dentro** de outra função em vez de num `announce`. E ela
//! não sabe se o `eprintln!` de facto sai — só que alguém o chama.

use std::fs;
use std::path::{Path, PathBuf};

fn raiz_da_crate() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// Os ficheiros `scenes_*.rs` que declaram um `announce()` sem argumentos.
fn quem_escreve_roteiro() -> Vec<String> {
    let mut saida = Vec::new();
    for entrada in fs::read_dir(raiz_da_crate()).expect("a pasta src desta crate") {
        let caminho = entrada.expect("uma entrada legível").path();
        let Some(nome) = caminho.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let Some(resto) = nome.strip_prefix("scenes_") else {
            continue;
        };
        let Some(modulo) = resto.strip_suffix(".rs") else {
            continue;
        };
        if modulo.ends_with("_tests") {
            continue;
        }
        let texto = fs::read_to_string(&caminho).expect("o ficheiro da cena");
        if texto.contains("fn announce()") {
            saida.push(modulo.to_string());
        }
    }
    saida.sort();
    saida
}

#[test]
fn toda_cena_com_roteiro_e_anunciada() {
    let despacho =
        fs::read_to_string(raiz_da_crate().join("scripts.rs")).expect("o despacho dos roteiros");
    let escritores = quem_escreve_roteiro();

    // ⚠️ O PISO: sem ele, uma renomeação do prefixo faria este gate varrer zero
    // ficheiros e ficar verde a medir nada.
    assert!(
        escritores.len() >= 14,
        "o censo varreu só {} cenas com roteiro — a varredura partiu-se, não a \
         família encolheu: {escritores:?}",
        escritores.len()
    );

    let mudas: Vec<&String> = escritores
        .iter()
        .filter(|m| !despacho.contains(&format!("crate::scenes::{m}::announce()")))
        .collect();
    assert!(
        mudas.is_empty(),
        "estas cenas escrevem um roteiro que ninguém imprime: {mudas:?} — o \
         artista abre a cena e não tem o passo a passo, que é onde ele APRENDE \
         a ferramenta"
    );
}

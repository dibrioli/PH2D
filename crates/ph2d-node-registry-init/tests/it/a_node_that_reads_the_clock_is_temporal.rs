//! ⭐⭐⭐ **QUEM LÊ O RELÓGIO DECLARA-SE `Temporal`** — o censo que apanha, de uma vez, a
//! família inteira do defeito de 2026-08-28.
//!
//! # O mecanismo, e por que ele é silencioso
//!
//! A impressão digital do memo (`cook_fingerprint.rs`) inclui **os bits do playhead só se o nó
//! for `Temporal`**. Um nó que chama `ctx.playhead()` e se declara `Effect::Pure` cozinha
//! **uma vez** e devolve o mesmo stream para sempre — não há erro, não há aviso, não há nada
//! vermelho. Da cadeira: *"não há nenhuma animação ou movimento"*.
//!
//! ⚠️ **O `motion.sub_uv` shipou assim desde que existe.** Ninguém viu porque **nenhuma cena
//! tinha ligado o relógio dele**: a única que o usava (`PH2D_MOTION_OBJ_SMOKE=9`) deixa o
//! `speed` no default, que é `0`. *Um defeito só é visível onde há uma cena que o contenha.*
//!
//! ⚠️ **E os gates não o viam por uma razão própria:** eles constroem um `Cook::new()` por
//! instante, e *um memo que nasce vazio nunca devolve nada de velho*. Quem reusa o cozinhador é
//! o app.
//!
//! # Por que este censo lê o FONTE
//!
//! A alternativa — cozinhar cada nó duas vezes e ver se mexe — não é aplicável: a maioria dos
//! nós precisa de entradas, de externos ou de um `pre`, e um que não mexesse por não ter nada
//! para mexer daria um falso positivo. A propriedade que interessa é **sintáctica e local**:
//! *este ficheiro chama `ctx.playhead()`?* e *este manifesto diz `Pure`?* Duas perguntas que o
//! texto responde exactamente.
//!
//! ⚠️ **Os `*_tests.rs` ficam de fora, e a exclusão é load-bearing:** o `motion.trail` chama
//! `ctx.playhead()` numa FONTE DE FIXTURA (`lib_tests.rs`) e o produto dele nunca o lê — sem a
//! exclusão, este censo acusaria um nó correcto e treinaria quem o lê a ignorá-lo.

use std::path::{Path, PathBuf};

/// A pasta `crates/`, a partir deste crate.
fn crates_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("o crate vive dentro de crates/")
        .to_path_buf()
}

/// Os ficheiros de PRODUTO de um crate de nó — tudo em `src/` menos os irmãos de teste.
fn product_sources(crate_dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![crate_dir.join("src")];
    while let Some(d) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else {
            continue;
        };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().is_some_and(|x| x == "rs")
                && !p
                    .file_name()
                    .and_then(|f| f.to_str())
                    .is_some_and(|f| f.ends_with("_tests.rs"))
            {
                out.push(p);
            }
        }
    }
    out.sort();
    out
}

/// `(lê o relógio, declara Pure, declara Temporal)` — só sobre o produto.
fn survey(crate_dir: &Path) -> (bool, bool, bool) {
    let (mut clock, mut pure, mut temporal) = (false, false, false);
    for f in product_sources(crate_dir) {
        let Ok(src) = std::fs::read_to_string(&f) else {
            continue;
        };
        clock |= src.contains("ctx.playhead()");
        pure |= src.contains("effect: Effect::Pure");
        temporal |= src.contains("effect: Effect::Temporal");
    }
    (clock, pure, temporal)
}

/// Todos os crates `ph2d-node-*`.
fn node_crates() -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = std::fs::read_dir(crates_dir())
        .expect("crates/ existe")
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.is_dir()
                && p.file_name()
                    .and_then(|f| f.to_str())
                    .is_some_and(|f| f.starts_with("ph2d-node-"))
        })
        .collect();
    out.sort();
    out
}

/// **Nenhum nó lê o relógio e se diz sem tempo.**
///
/// ⚠️ **O CONTROLE é a contagem de quem LÊ**: um censo que varresse zero ficheiros passaria
/// mudo, e *um zero de «não medido» e um de «tudo certo» são o mesmo byte*. Aqui exige-se que
/// a varredura encontre uma dúzia de leitores do relógio — que é o que ela de facto vê — antes
/// de a ausência de violações querer dizer alguma coisa.
#[test]
fn no_node_reads_the_clock_while_declaring_itself_timeless() {
    let mut readers = 0usize;
    let mut offenders = Vec::new();
    for c in node_crates() {
        let (clock, pure, temporal) = survey(&c);
        if !clock {
            continue;
        }
        readers += 1;
        if pure && !temporal {
            offenders.push(c.file_name().unwrap().to_string_lossy().into_owned());
        }
    }
    assert!(
        readers >= 12,
        "a varredura so' achou {readers} nos a ler o relogio — ela esta' partida, e um censo \
         partido responde SEMPRE que esta' tudo bem"
    );
    assert!(
        offenders.is_empty(),
        "estes nos leem `ctx.playhead()` e declaram-se `Effect::Pure`: {offenders:?}\n\
         A impressao digital do memo so' inclui o relogio para um no' `Temporal`, entao eles \
         cozinham UMA vez e devolvem o mesmo stream para sempre — congelados, sem nada \
         vermelho em lado nenhum. Foi o defeito do `motion.sub_uv` (2026-08-28)."
    );
}

/// ⚠️ **A exclusão dos `*_tests.rs` é load-bearing, e este gate afirma-o.**
///
/// O `motion.trail` chama `ctx.playhead()` numa fonte de FIXTURA e o produto dele nunca o lê.
/// Sem a exclusão o censo acima acusaria um nó correcto — e um censo que acusa inocentes é um
/// censo que se aprende a ignorar. Se um dia o produto dele passar a ler o relógio, este gate
/// cai e a pessoa vai ao censo em vez de ao ficheiro errado.
#[test]
fn the_test_fixture_exclusion_is_what_keeps_a_correct_node_out_of_the_list() {
    let trail = crates_dir().join("ph2d-node-motion-trail");
    assert!(trail.is_dir(), "o motion.trail existe");
    let (clock_in_product, _, _) = survey(&trail);
    assert!(
        !clock_in_product,
        "o produto do motion.trail passou a ler o relogio — reveja o efeito dele, e depois esta \
         nota"
    );
    // E o CONTROLE: sem a exclusão, ele APARECERIA — é isso que torna a exclusão necessária.
    let any_test_file_reads = std::fs::read_dir(trail.join("src"))
        .expect("src/")
        .flatten()
        .any(|e| {
            e.path()
                .file_name()
                .and_then(|f| f.to_str())
                .is_some_and(|f| f.ends_with("_tests.rs"))
                && std::fs::read_to_string(e.path()).is_ok_and(|s| s.contains("ctx.playhead()"))
        });
    assert!(
        any_test_file_reads,
        "a fixtura do motion.trail deixou de ler o relogio — a exclusao pode ter deixado de ser \
         necessaria, e esta nota tem de ser reconferida antes de alguem a apagar"
    );
}

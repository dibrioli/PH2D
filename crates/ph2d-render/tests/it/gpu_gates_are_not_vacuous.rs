//! **O SALTO TEM DE SE DIZER.** Um gate de GPU que não encontrou adaptador
//! devolve cedo e o corredor lê **verde** — indistinguível de um que correu e
//! provou alguma coisa. O `CLAUDE.md` §5.0 já tem a lei escrita (*«Gates de GPU
//! são `#[ignore]` e precisam de adapter — skip gracioso não é verde»*); o que
//! faltava era um **instrumento** que a fizesse valer.
//!
//! ## O que foi MEDIDO (2026-08-30, `chore/stack-upgrade-2026-08`)
//!
//! Censo desta crate, conferido contra `cargo nextest list` (52 = 52, zero
//! divergências):
//!
//! | sítios `try_headless_gpu()` em `ph2d-render` | 201 |
//! |---|---:|
//! | dentro de um `#[test]` **com** `#[ignore]` (fora do CI) | 149 |
//! | dentro de um `#[test]` **sem** `#[ignore]` (**o CI corre-os**) | **52** |
//!
//! Repartição desses 52 por NATUREZA — *a cura de uma sonda que salta não é a
//! de um gate que salta*, então a repartição decidiu o desenho:
//!
//! | forma | quantos |
//! |---|---:|
//! | **gate** (`assert*!` / `panic!` / `expect`) | 48 |
//! | **gate por validação** (o device recusa o pipeline e o wgpu entra em panic) | 3 |
//! | **sonda** (só imprime medições, não afirma nada) | 1 |
//!
//! ⇒ **51 dos 52 são gates.** Uma cura só serve quase todos, e é por isso que
//! este ficheiro é UM teste e não 52 edições.
//!
//! ## A prova de que o verde era vazio
//!
//! Mutação em `tests/individual_readback.rs` (`try_headless_gpu` a devolver
//! `None`, restaurada a seguir): os 6 testes daquele binário passaram em
//! **0,29 s** cada, contra **3,1–3,4 s** com device real. Verdes, sem terem
//! tocado numa GPU.
//!
//! ⚠️ **E o CI não tem adaptador** — não é suposição: `ph2d-gpu/src/context.rs`
//! diz-o no `#[ignore]` do irmão dele (*«requires a GPU adapter (no GPU on
//! CI)»*), e nenhum workflow instala lavapipe/WARP. ⇒ hoje, no CI, os **52
//! passam sem executar uma linha**.
//!
//! ## Por que UM sentinela, e não as outras duas saídas
//!
//! - ⛔ **Marcar os 52 `#[ignore]`** (como as 149 irmãs) é honesto na
//!   contabilidade e **apaga o sinal**: numa máquina COM GPU — esta — os 52
//!   passam a sério em 8,07 s e apanhariam uma regressão de device. Trocar um
//!   verde falso por *nenhuma* leitura não é um upgrade.
//! - ⛔ **Fazer cada um falhar alto** dá **52 vermelhos idênticos** para um
//!   facto só, põe a válvula de escape em 52 sítios, e o `nextest` cancela na
//!   primeira falha — um vermelho que esconde a suíte custa outra corrida.
//! - ✅ **Um sentinela** transforma «52 verdes mudos» em **um vermelho
//!   legível**, e deixa o retorno cedo dos outros 52 ser o que ele deve ser:
//!   barato e já reportado por outrem.
//!
//! ⚠️ **E há um motivo MEDIDO para a cura não ser por-sítio:** **28 dos 52** já
//! tentam anunciar-se com `eprintln!` (os outros 24 devolvem em silêncio) — e
//! **ninguém os ouve**. O `nextest` (e o `cargo test`) capturam a saída de um
//! teste que PASSA e deitam-na fora. Um aviso no caminho do sucesso é teatro;
//! só o veredito se ouve. ⇒ *metade destes gates já tentou curar-se sozinha, e
//! a tentativa foi inaudível por construção.* É por isso que o anúncio tem de
//! vir de um teste que **falha**.
//!
//! ⚠️ **E a «GPU lane» não existe.** 43 das mensagens de `#[ignore]` desta
//! crate mandam *«run with --ignored on the GPU lane»*, e não há lane nenhuma
//! em `.github/workflows/` — nem GPU, nem passo que corra `--ignored`. Essas
//! 149 irmãs não são «adiadas para outro corredor»: elas **não correm em lado
//! nenhum** a não ser à mão, nesta máquina.
//!
//! ## A válvula de escape
//!
//! `PH2D_ALLOW_NO_GPU=1` faz este sentinela render-se. Ela existe para quem
//! corre a suíte sem GPU **de propósito** (um runner de lint, um container).
//! ⚠️ Ela é, ela própria, um salto silencioso — mas é **um**, explícito e
//! escrito, em vez de 52 implícitos. *Uma renúncia que se digita é uma decisão;
//! uma que acontece sozinha é um defeito.*

use ph2d_gpu::GpuContext;
use std::path::{Path, PathBuf};

/// A única porta para desligar este sentinela.
const WAIVER: &str = "PH2D_ALLOW_NO_GPU";

/// ⛔ **Este número NÃO se escreve, conta-se.** Um `52` cravado aqui estaria
/// errado no dia em que alguém acrescentasse o 53.º — e um número errado numa
/// mensagem de falha ensina o leitor a não confiar nela. A contagem é derivada
/// do FONTE, com a mesma regra que o censo usou (e que bateu com o
/// `cargo nextest list` em 52 de 52).
fn count_unignored_gpu_gates() -> usize {
    let mut files = Vec::new();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    collect_rs(&root.join("src"), &mut files);
    collect_rs(&root.join("tests"), &mut files);

    let mut n = 0;
    for path in files {
        // Este ficheiro NOMEIA a função em prosa; contá-lo seria contar-se a si
        // próprio.
        if path
            .file_name()
            .is_some_and(|f| f == "gpu_gates_are_not_vacuous.rs")
        {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let lines: Vec<&str> = text.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            // Só CHAMADAS: nem a definição, nem uma menção num comentário.
            if !line.contains("try_headless_gpu()")
                || line.contains("fn try_headless_gpu")
                || trimmed.starts_with("//")
                || trimmed.starts_with("*")
                || trimmed.starts_with("//!")
            {
                continue;
            }
            if enclosing_test_runs_in_ci(&lines, i) {
                n += 1;
            }
        }
    }
    n
}

/// Sobe da linha `i` até à assinatura da função que a contém, depois lê o bloco
/// de atributos por cima dela. Conta só o que o CI de facto corre: tem `#[test]`
/// e **não** tem `#[ignore]`.
fn enclosing_test_runs_in_ci(lines: &[&str], i: usize) -> bool {
    let mut fn_line = None;
    for k in (0..=i).rev() {
        if lines[k].contains("fn ") {
            fn_line = Some(k);
            break;
        }
    }
    let Some(fn_line) = fn_line else {
        return false;
    };

    let mut has_test = false;
    let mut has_ignore = false;
    for k in (0..fn_line).rev() {
        let t = lines[k].trim();
        // O bloco contíguo de atributos / doc-comments imediatamente acima.
        let is_attr_ish = t.starts_with("#[")
            || t.starts_with("///")
            || t.starts_with("//")
            || t.starts_with(']')
            || t.ends_with(',')
            || t.ends_with(")]");
        if !is_attr_ish {
            break;
        }
        if t.starts_with("#[test]") {
            has_test = true;
        }
        if t.starts_with("#[ignore") {
            has_ignore = true;
        }
    }
    has_test && !has_ignore
}

fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rs(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// **OU CORRERAM, OU ESTA CORRIDA NÃO É VERDE.**
///
/// Sem adaptador, os gates de GPU desta crate devolvem cedo e passam. Este
/// teste é o único sítio onde essa ausência vira **veredito** em vez de
/// silêncio. Ele pergunta exactamente o que o `try_headless_gpu` de cada um
/// pergunta — `GpuContext::new` — para que a resposta não possa divergir.
#[test]
fn the_gpu_gates_have_a_device_or_this_run_is_not_green() {
    if GpuContext::new(GpuContext::default_instance(), None).is_ok() {
        return; // Há device: os 52 correram de verdade.
    }

    let n = count_unignored_gpu_gates();

    if std::env::var_os(WAIVER).is_some() {
        // Renúncia explícita. Continua a ser um salto — mas é UM, e foi digitado.
        eprintln!(
            "[gpu-gates] {WAIVER} activa: {n} gate(s) de GPU nesta crate NAO correram. \
             Esta corrida nao prova nada sobre o device."
        );
        return;
    }

    panic!(
        "SEM ADAPTADOR DE GPU: {n} gate(s) desta crate devolveram cedo e contariam \
         como APROVADOS sem terem executado nada. Skip gracioso nao e' verde \
         (CLAUDE.md 5.0).\n\
         \n\
         Duas saidas, e as duas sao decisoes que alguem toma:\n\
           1. dar um adaptador ao corredor (lavapipe/WARP no CI, ou uma GPU real) \
             -- e' o que faz os {n} passarem a valer alguma coisa;\n\
           2. correr com {WAIVER}=1 -- aceita a perda, e deixa-a escrita.\n\
         \n\
         Este teste falha para que a perda tenha UM vermelho legivel em vez de \
         {n} verdes mudos."
    );
}

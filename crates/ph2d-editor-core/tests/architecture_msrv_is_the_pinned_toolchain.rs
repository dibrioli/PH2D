//! **A MSRV declarada é a toolchain PINADA** — e este gate substitui um job de CI que
//! passava há meses sem nunca ter medido o que o nome dele prometia.
//!
//! ## O que ele substitui, e por quê
//!
//! O `spike.yml` tinha um job `MSRV (rust 1.92) check` que instalava 1.92 e rodava
//! `cargo check --workspace --locked`. Ele **nunca testou 1.92**: a action põe 1.92 como
//! `rustup default`, mas na precedência do rustup o **`rust-toolchain.toml` do repo vence o
//! default** — `rustup show active-toolchain` responde, literalmente,
//! `1.95 (overridden by <repo>/rust-toolchain.toml)`. O job media o PIN, ou seja o mesmo que
//! o job `test` do ubuntu já mede, e ficava verde por isso.
//!
//! Medido honestamente em 2026-08-18 (`RUSTUP_TOOLCHAIN=<v> cargo check --workspace --locked`),
//! a workspace **não compila** abaixo do pin, por dois mecanismos independentes:
//!
//! | toolchain | reprova por |
//! |---|---|
//! | **1.92** | as DEPS — cranelift/pulley/wasmtime (`0.134.3`/`47.0.3`) exigem 1.94 |
//! | **1.94** | o NOSSO código — `error[E0658]: 'if let' guards are experimental` (estável em 1.95) |
//!
//! ⇒ **o piso real É o pin**, e um job de MSRV com piso == pin é um duplicado por construção,
//! mesmo depois de consertada a precedência. O que sobra de valor é o invariante que este
//! arquivo afirma, e ele custa microssegundos em vez de um runner — e, ao contrário do job,
//! **não pode travar em rede** (o job morria no `apt-get update`, três vezes numa noite,
//! derrubando a matriz inteira que dependia dele).
//!
//! ## O que ele defende de verdade
//!
//! A DERIVA entre os dois números: alguém bumpa o pin e esquece a MSRV, ou o contrário. Hoje
//! eles têm de ser iguais — e no dia em que o projeto quiser de facto suportar um compilador
//! mais velho, é **este** gate que vira vermelho e obriga a decisão a ser tomada de propósito,
//! em vez de um job verde a afirmar uma coisa que ninguém mediu.

use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

/// Lê o valor de uma chave `chave = "valor"` do TOML, sem dep de parser.
fn toml_string(text: &str, key: &str) -> Option<String> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.starts_with('#'))
        .find_map(|l| {
            let rest = l.strip_prefix(key)?.trim_start();
            let rest = rest.strip_prefix('=')?.trim();
            rest.strip_prefix('"')?.split('"').next().map(str::to_owned)
        })
}

#[test]
fn the_declared_msrv_is_the_pinned_toolchain() {
    let r = root();
    let manifest = std::fs::read_to_string(r.join("Cargo.toml")).expect("Cargo.toml da raiz");
    let toolchain =
        std::fs::read_to_string(r.join("rust-toolchain.toml")).expect("rust-toolchain.toml");

    let msrv = toml_string(&manifest, "rust-version")
        .expect("`rust-version` sumiu do [workspace.package] — o gate mede outra coisa");
    let pin = toml_string(&toolchain, "channel")
        .expect("`channel` sumiu do rust-toolchain.toml — o gate mede outra coisa");

    assert_eq!(
        msrv, pin,
        "a MSRV declarada ({msrv}) e a toolchain pinada ({pin}) divergiram.\n\
         \n\
         Se foi o PIN que subiu: ou a workspace ainda compila na MSRV — e aí MEÇA, com\n\
         `RUSTUP_TOOLCHAIN=<msrv> cargo check --workspace --locked`, nunca com `rustup default`,\n\
         que o rust-toolchain.toml VENCE — ou a MSRV sobe junto.\n\
         \n\
         Se foi a MSRV que baixou de propósito (o projeto passou a suportar um compilador mais\n\
         velho), este gate é o lugar de dizer isso: relaxe-o com o número MEDIDO ao lado, e\n\
         devolva ao CI um job que de facto rode aquele toolchain (com `RUSTUP_TOOLCHAIN`, ou\n\
         removendo o rust-toolchain.toml naquele job)."
    );
}

/// **Controle: as duas fontes existem e não estão vazias.**
///
/// Sem ele, um `rust-toolchain.toml` renomeado deixaria o `expect` acima explodir — o que é
/// falha alta, correto — mas um valor VAZIO nos dois lados passaria como "iguais".
#[test]
fn both_sources_carry_a_real_version() {
    let r = root();
    let manifest = std::fs::read_to_string(r.join("Cargo.toml")).expect("Cargo.toml da raiz");
    let toolchain =
        std::fs::read_to_string(r.join("rust-toolchain.toml")).expect("rust-toolchain.toml");
    for (what, v) in [
        ("rust-version", toml_string(&manifest, "rust-version")),
        ("channel", toml_string(&toolchain, "channel")),
    ] {
        let v = v.unwrap_or_default();
        assert!(
            v.chars().next().is_some_and(|c| c.is_ascii_digit()),
            "`{what}` não parece uma versão: {v:?}"
        );
    }
}

//! ⭐⭐ **NENHUMA PALAVRA DESTE PAINEL É ESCRITA NO FONTE** — o HR-15, com a régua que vê a crate
//! inteira (os knobs do solver de tinta molhada).
//!
//! ⚠️ **Esta crate NUNCA teve régua** até 2026-09-17: ela era uma das OITO crates de UI sem o
//! `every_word_this_panel_shows_comes_from_the_string_table` — *um censo que não corre sobre uma
//! crate não afirma nada sobre ela*, e é ali que o próximo literal nasce calado. A régua é a folha
//! partilhada ([`ph2d_label_census`]); só a lista, o prefixo e as excepções são desta crate.
//!
//! ⛔⛔ **A metade das ÓRFÃS teria mandado apagar DEZANOVE rótulos VIVOS.** As
//! `panel.wet_tuning.knob.*` não aparecem como literal em sítio nenhum: elas são montadas em
//! runtime (`format!("panel.wet_tuning.knob.{}", d.key)` no [`rows.rs`]), e um censo que só vê
//! literais lê-as como declaradas-e-não-usadas. ⇒ *uma régua que mede literais prescreve a cura
//! errada para uma chave que é ASSEMBLADA* — e a cura errada aqui era apagar os rótulos de
//! dezanove knobs do painel.
use ph2d_label_census::gate::{self, Excecao};

const PREFIX: &str = "panel.wet_tuning.";
const TABLES: &[&str] = &["crates/ph2d-i18n/src/lib.rs"];

/// ⭐ As excepções, **com o mecanismo** — nunca uma lista aberta.
const NOT_LANGUAGE: &[Excecao] = &[];

#[test]
fn every_word_this_panel_shows_comes_from_the_string_table() {
    let (src, _) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let intrusos = gate::intrusos(&src, NOT_LANGUAGE);
    assert!(
        intrusos.is_empty(),
        "estes textos com cara de língua estão escritos no fonte e nunca chegam à tabela de \
         strings (HR-15):\n  {}\n\nA cura é uma chave `{PREFIX}<nome>` numa das {TABLES:?} e um \
         `tr(\"…\")` no sítio (uma frase com peças do código: `tr_with`). ⚠️ Se o texto NÃO é \
         língua, a cura é uma linha em `NOT_LANGUAGE` **com o mecanismo**.",
        intrusos.join("\n  ")
    );
}

/// ⭐ A metade justa — e o controlo de vacuidade: uma régua cega não acha as excepções.
#[test]
fn every_named_exception_still_shelters_a_real_literal() {
    let (src, _) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let mortas = gate::excecoes_mortas(&src, NOT_LANGUAGE);
    assert!(
        mortas.is_empty(),
        "excepções sem mecanismo ou sem literal:\n  {}",
        mortas.join("\n  ")
    );
}

/// ⭐⭐⭐ **UMA CHAVE COM ERRO DE ESCRITA PINTA O IDENTIFICADOR CRU NA TELA** — e vaza uma string por
/// quadro (`leak_key`). O censo é dos DOIS lados: usada-e-não-declarada pinta o identificador;
/// declarada-e-não-usada é uma órfã, que é onde alguém escreve, um dia, uma frase sobre um
/// controlo que já não existe.
#[test]
fn every_key_of_this_panel_exists_on_both_sides() {
    let (_, repo) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let c = gate::chaves(&repo, PREFIX, TABLES);
    // ⛔ Controlo de vacuidade: um caminho errado dá dois conjuntos vazios, que concordam sempre.
    // ⚠️⚠️ **OS DOIS PISOS SÃO DIFERENTES, e a diferença é o ACHADO:** a tabela declara ~52 chaves
    //    e o fonte usa ~10 como LITERAL, porque as `knob.*` são montadas em runtime. *Um piso
    //    simétrico aqui reprovaria sobre produto correcto* — e foi o que a 1.ª redacção fez.
    assert!(
        c.declaradas >= 40 && c.usadas >= 8,
        "o censo achou {} declaradas e {} usadas — os pisos são 40 e 8, e dois conjuntos vazios \
         concordam sempre",
        c.declaradas,
        c.usadas
    );
    assert!(
        c.sem_traducao.is_empty(),
        "estas chaves são usadas e NÃO existem em {TABLES:?} — o `tr` faz `leak_key` e pinta o \
         identificador cru:\n  {}",
        c.sem_traducao.join("\n  ")
    );

    // ⛔⛔ **A família MONTADA sai das órfãs — e a isenção só vale enquanto o sítio que a monta
    //    existir.** Sem esta leitura do fonte, apagar o `format!` deixaria dezanove chaves
    //    genuinamente mortas isentas para sempre: *uma isenção que nada pode apagar é a catraca
    //    que vira licença* (`CLAUDE.md` §5.0).
    let (src, _) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let rows = std::fs::read_to_string(src.join("rows.rs")).expect("o `rows.rs` do painel existe");
    assert!(
        rows.contains("format!(\"panel.wet_tuning.knob.{}\""),
        "a isenção da família montada em runtime já não descreve o código: o `format!` do \
         `rows.rs` saiu. Ou as chaves passaram a ser literais (apague a isenção) ou os rótulos \
         morreram (apague as chaves)."
    );
    let montadas = c
        .orfas
        .iter()
        .filter(|k| k.starts_with(MONTADA_EM_RUNTIME))
        .count();
    assert!(
        montadas >= 15,
        "a família montada em runtime tem {montadas} chaves e em 2026-09-17 tinha 19 — ou os knobs \
         encolheram (baixe o piso com o número novo) ou o censo deixou de as ver"
    );
    let orfas: Vec<&String> = c
        .orfas
        .iter()
        .filter(|k| !k.starts_with(MONTADA_EM_RUNTIME))
        .collect();
    assert!(
        orfas.is_empty(),
        "estas chaves estão na tabela e ninguém as usa — apague-as:\n  {orfas:?}"
    );
}

/// ⛔⛔ O prefixo da família cujas chaves o painel MONTA em runtime — ver o cabeçalho.
const MONTADA_EM_RUNTIME: &str = "panel.wet_tuning.knob.";

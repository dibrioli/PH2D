//! ⭐⭐ **NENHUMA PALAVRA DESTE PAINEL É ESCRITA NO FONTE** — o HR-15, com a régua que vê a crate
//! inteira (a BANCADA onde o redesenho se estuda).
//!
//! ⚠️ **Esta crate NUNCA teve régua** até 2026-09-17: ela era uma das OITO crates de UI sem o
//! `every_word_this_panel_shows_comes_from_the_string_table` — *um censo que não corre sobre uma
//! crate não afirma nada sobre ela*, e é ali que o próximo literal nasce calado. A régua é a folha
//! partilhada ([`ph2d_label_census`]); só a lista, o prefixo e as excepções são desta crate.
//!
//! ⛔ **O `study.rs` é uma BANCADA e está ISENTO, com o mesmo estatuto da galeria de widgets da
//! `ph2d-editor-core`** (que o doc-comment dela já declara, e que nomeia esta crate ao lado):
//! os textos dele são CABEÇALHOS DE ESTUDO (*"2 · WIDTH RULER — the chosen design, squeezed"*) e
//! LEGENDAS COM AS CONTAS (*"label 70 + gap 6 + track + gap 6 + box 72 = 154 px"*). Traduzir a
//! legenda de uma medição não é i18n, é ruído. ⚠️ A isenção é do FICHEIRO e por isso traz **piso de
//! população**: se a bancada sair, ela deixa de abrigar o que nomeia e o gate manda apagá-la.
use ph2d_label_census::gate::{self, Excecao, Isento};

const PREFIX: &str = "panel.widget_lab.";
const TABLES: &[&str] = &["crates/ph2d-i18n/src/lib.rs"];

/// ⭐ As excepções, **com o mecanismo** — nunca uma lista aberta.
const NOT_LANGUAGE: &[Excecao] = &[];

/// ⛔ A BANCADA, isenta como FICHEIRO — ver o cabeçalho.
const ISENTOS: &[Isento] = &[(
    "study.rs",
    "CONFIRMADO PELO DONO em 2026-09-17, com a FOTO na mão (ele viu, listou, ouviu o argumento e \
     respondeu «nenhum — deixe como está»). A bancada do redesenho: cabeçalhos de estudo numerados e legendas que são a CONTA de uma \
     medição (`label 70 + gap 6 + track + gap 6 + box 72 = 154 px`). Traduzir a legenda de uma \
     medição não é i18n, é ruído — o mesmo estatuto da galeria de widgets da `ph2d-editor-core`, \
     cujo doc-comment nomeia esta crate ao lado",
)];

#[test]
fn every_word_this_panel_shows_comes_from_the_string_table() {
    let (src, _) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let intrusos = gate::intrusos_fora_de(&src, NOT_LANGUAGE, ISENTOS, &[]);
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
    let mut mortas = gate::excecoes_mortas(&src, NOT_LANGUAGE);
    mortas.extend(gate::isentos_mortos(&src, ISENTOS));
    assert!(
        mortas.is_empty(),
        "excepções sem mecanismo ou sem literal:\n  {}",
        mortas.join("\n  ")
    );
    // ⭐ **O PISO DA BANCADA**: a isenção é de um FICHEIRO INTEIRO, logo ela tem de continuar a
    //   abrigar uma bancada — se o estudo sair, ela deixa de descrever alguma coisa.
    let bancada = ph2d_label_census::language_literals(&src)
        .into_iter()
        .filter(|l| l.rel == "study.rs")
        .count();
    assert!(
        bancada >= 10,
        "a bancada abriga {bancada} textos e em 2026-09-17 abrigava 15 — ou o estudo saiu (apague \
         a isenção) ou a régua ficou cega"
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
    assert!(
        c.declaradas >= 1 && c.usadas >= 1,
        "o censo achou {} declaradas e {} usadas — o piso é 1 e dois conjuntos vazios \
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
    assert!(
        c.orfas.is_empty(),
        "estas chaves estão na tabela e ninguém as usa — apague-as:\n  {:?}",
        c.orfas
    );
}

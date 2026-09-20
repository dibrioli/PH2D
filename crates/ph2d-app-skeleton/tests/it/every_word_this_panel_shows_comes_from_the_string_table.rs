//! ⭐⭐ **NENHUMA PALAVRA DESTE PAINEL É ESCRITA NO FONTE** — o HR-15, com a régua que vê a crate
//! inteira (a família do esqueleto).
//!
//! ⚠️ **Esta crate NUNCA teve régua** até 2026-09-17: ela era uma das OITO crates de UI sem o
//! `every_word_this_panel_shows_comes_from_the_string_table` — *um censo que não corre sobre uma
//! crate não afirma nada sobre ela*, e é ali que o próximo literal nasce calado. A régua é a folha
//! partilhada ([`ph2d_label_census`]); só a lista, o prefixo e as excepções são desta crate.
//!
//! ⚠️ **Sem censo de CHAVES:** esta crate não declara prefixo próprio (`app.skeleton.*` não existe),
//! e um censo sobre um prefixo vazio é dois conjuntos vazios a concordarem.
use ph2d_label_census::gate::{self, Excecao};

/// ⭐⭐ **O mecanismo das dezasseis excepções do [`verbos.rs`], escrito UMA vez.**
///
/// Elas são as AGULHAS de um censo textual: o `Verbo::rastos_na_shell` devolve, por verbo, os
/// fragmentos de CÓDIGO que a fase do quadro tem de conter — e o gate daquela crate varre o fonte
/// da shell à procura deles. ⛔ Traduzi-las apagaria o censo: o rasto é um caminho de Rust
/// (`skeleton_live::release(`), uma variante (`Keep::Deformed`) ou uma escrita (`smart_pick =
/// Some(`), e nenhum deles é uma palavra que alguém lê no ecrã.
///
/// ⚠️ **Porque a régua lexical as lê como língua, e ela não está errada:** um `(` ou um espaço
/// quebram o teste do `TokenNu`, e `Keep::Deformed` é Capitalizado — *a cegueira que as salvaria
/// seria «tudo o que parece código», que branquearia centenas de rótulos verdadeiros*.
///
/// ⭐ **E este ficheiro fica sob censo:** `verbos.rs` hoje não tem uma única palavra de ecrã, logo
/// um rótulo escrito lá amanhã REPROVA. Uma isenção de FICHEIRO INTEIRO
/// ([`gate::intrusos_fora_de`]) calaria essa metade para sempre.
const AGULHA_DO_CENSO_DA_SHELL: &str = "\
é uma AGULHA do censo textual de `Verbo::rastos_na_shell`: um fragmento de CÓDIGO que a fase do \
quadro tem de conter para o verbo ter rasto. Traduzi-la apagaria o censo — ver a nota acima";

/// ⭐ As excepções, **com o mecanismo** — nunca uma lista aberta.
const NOT_LANGUAGE: &[Excecao] = &[
    ("verbos.rs", "espelho::espelha(", AGULHA_DO_CENSO_DA_SHELL),
    ("verbos.rs", "goal::add_look_at(", AGULHA_DO_CENSO_DA_SHELL),
    (
        "verbos.rs",
        "skeleton_live::bind(",
        AGULHA_DO_CENSO_DA_SHELL,
    ),
    (
        "verbos.rs",
        "skeleton_live::release(",
        AGULHA_DO_CENSO_DA_SHELL,
    ),
    ("verbos.rs", "Keep::Deformed", AGULHA_DO_CENSO_DA_SHELL),
    ("verbos.rs", "Keep::Source", AGULHA_DO_CENSO_DA_SHELL),
    (
        "verbos.rs",
        "pose_de_repouso::aplica(",
        AGULHA_DO_CENSO_DA_SHELL,
    ),
    ("verbos.rs", "Verbo::Repor", AGULHA_DO_CENSO_DA_SHELL),
    ("verbos.rs", "Verbo::Guardar", AGULHA_DO_CENSO_DA_SHELL),
    ("verbos.rs", "skeleton_goal::add(", AGULHA_DO_CENSO_DA_SHELL),
    (
        "verbos.rs",
        "skeleton_goal::remove(",
        AGULHA_DO_CENSO_DA_SHELL,
    ),
    (
        "verbos.rs",
        "bone_limit::add_limit(",
        AGULHA_DO_CENSO_DA_SHELL,
    ),
    (
        "verbos.rs",
        "bone_limit::remove_limit(",
        AGULHA_DO_CENSO_DA_SHELL,
    ),
    ("verbos.rs", "smart::add(", AGULHA_DO_CENSO_DA_SHELL),
    (
        "verbos.rs",
        "skeleton_smart::remove(",
        AGULHA_DO_CENSO_DA_SHELL,
    ),
    ("verbos.rs", "smart_pick = Some(", AGULHA_DO_CENSO_DA_SHELL),
    (
        "goal.rs",
        "Shoulder",
        "é o NOME de uma entidade da fixtura de braço (`braco()`, atrás de `cfg(test, \
         feature = \"test-support\")`): conteúdo de um documento de amostra, como o nome de uma \
         camada — traduzir o nome de um objecto do artista seria errado",
    ),
    (
        "goal.rs",
        "Elbow",
        "o segundo osso da mesma fixtura de amostra — ver a nota do `Shoulder` acima",
    ),
];

#[test]
fn every_word_this_panel_shows_comes_from_the_string_table() {
    let (src, _) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let intrusos = gate::intrusos(&src, NOT_LANGUAGE);
    assert!(
        intrusos.is_empty(),
        "estes textos com cara de língua estão escritos no fonte e nunca chegam à tabela de \
         strings (HR-15):\n  {}\n\nA cura é uma chave na tabela do assunto e um \
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

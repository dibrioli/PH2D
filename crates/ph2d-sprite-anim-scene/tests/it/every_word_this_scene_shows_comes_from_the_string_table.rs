//! ⭐ **NENHUMA PALAVRA DA CENA 5 É ESCRITA NO FONTE** — o HR-15 na folha que recebeu o executor da cena
//! `PH2D_GAME_SMOKE=5` da shell (10/11). Sem este censo as isenções que saíram de
//! `every_word_the_shell_shows_comes_from_the_string_table` deixavam a folha muda (CLAUDE.md §5.0, mover parte gates).

use ph2d_label_census::gate::{self, Excecao, Isento};

/// A crate É uma cena de smoke: o guião e as fotos são dado da cena, a regra `gate::CENAS` (por nome de ficheiro)
/// não os casa porque a crate inteira é a cena.
const FORA: &[Isento] = &[
    (
        "lib.rs",
        "o guião da cena 5: os NOMES dos objectos que ela cria (conteúdo do documento, como o nome de uma camada), \
         a linha `[game-smoke]` do terminal e os rótulos `[animado-prova]` de cada relato — nunca pintados pelo app",
    ),
    (
        "fotos.rs",
        "os passos das fotos do editor (`PH2D_GAME_SMOKE_FOTO`): só os rótulos `[animado-prova]` do relato no terminal",
    ),
];

/// O executor (`mesa.rs`) fica sob a régua literal a literal: é onde um texto de UI novo poderia entrar.
const NOT_LANGUAGE: &[Excecao] = &[
    (
        "mesa.rs",
        "o botão Pivot",
        "o nome do alvo da prova no terminal (`[animado-prova] pivô: … não está na tela`) — diagnóstico de quem \
         corre a prova, nunca pintado (veio da shell com o executor, 10/11)",
    ),
    (
        "mesa.rs",
        "o ponto do pivô",
        "o nome do alvo da prova no terminal, como o `o botão Pivot` acima",
    ),
    (
        "mesa.rs",
        "selecionado={sel} origem={t:?} folha_e_pivo={pivo:?} desenho={c:?}",
        "o relato `chave=valor` da prova do Pivot no terminal (`relato_do_pivo`) — o diagnóstico de quem segue a \
         prova, nunca pintado",
    ),
];

#[test]
fn every_word_this_scene_shows_comes_from_the_string_table() {
    let (src, _) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let intrusos = gate::intrusos_fora_de(&src, NOT_LANGUAGE, FORA, &[]);
    assert!(
        intrusos.is_empty(),
        "texto com cara de língua escrito no fonte da cena 5 (HR-15):\n  {}\n\n\
         Se o artista o lê: uma chave em `ph2d-i18n` e um `tr(\"…\")`. Se é diagnóstico de terminal: \
         uma linha em `NOT_LANGUAGE` com o mecanismo.",
        intrusos.join("\n  ")
    );
}

/// A metade justa e o controlo de vacuidade: uma régua cega não acha as excepções.
#[test]
fn every_named_exemption_still_shelters_what_it_names() {
    let (src, _) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let mortas: Vec<String> = gate::excecoes_mortas(&src, NOT_LANGUAGE)
        .into_iter()
        .chain(gate::isentos_mortos(&src, FORA))
        .collect();
    assert!(
        mortas.is_empty(),
        "isenções mortas:\n  {}",
        mortas.join("\n  ")
    );
}

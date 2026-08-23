//! **Os gates do ficheiro de projeto** ([`super`]).
//!
//! ⚠️ **O que está aqui é a POLÍTICA**, que é a parte onde um erro custa trabalho perdido: *o que
//! pergunta, o que não pergunta, e com que nome*. O seletor nativo em si (`rfd`) não é alcançável
//! de um teste — ele abre uma janela modal do sistema —, e o resíduo por gatear é isso mais as
//! três linhas que o chamam. Foi por isso que a decisão saiu para funções puras.

use super::*;

/// **O PRIMEIRO save pergunta; os seguintes não.** É a lei inteira do `Ctrl+S`.
///
/// ⛔ Gravar em silêncio num nome inventado é como o trabalho acaba num ficheiro que ninguém
/// encontra depois — foi literalmente o que este app fez até 2026-08-23 (`ph2d_project.postcard`
/// no diretório corrente, sempre o mesmo).
///
/// **Mutação que deve sangrar:** devolver `Path(UNTITLED)` quando não há sessão.
#[test]
fn the_first_save_asks_and_the_next_ones_do_not() {
    assert_eq!(
        save_target(None, false),
        IoTarget::Ask {
            suggested: UNTITLED.to_owned()
        },
        "sem ficheiro, o primeiro save tem de perguntar"
    );
    assert_eq!(
        save_target(Some("/a/hero.ph2dproj"), false),
        IoTarget::Path("/a/hero.ph2dproj".to_owned()),
        "com ficheiro, gravar e' silencioso"
    );
    // Um caminho VAZIO é o mesmo que não haver: ele viria de uma env var mal preenchida.
    assert!(matches!(save_target(Some(""), false), IoTarget::Ask { .. }));
}

/// **`Save As…` pergunta SEMPRE, e sugere o nome que o ficheiro já tem** — quem faz «guardar como»
/// quer partir do que está aberto, não de uma folha em branco.
///
/// **Mutação que deve sangrar:** ignorar o `force_ask`.
#[test]
fn save_as_always_asks_and_suggests_the_current_name() {
    assert_eq!(
        save_target(Some("/a/hero.ph2dproj"), true),
        IoTarget::Ask {
            suggested: "hero.ph2dproj".to_owned()
        },
        "sugere o NOME, sem a pasta"
    );
    assert_eq!(
        save_target(None, true),
        IoTarget::Ask {
            suggested: UNTITLED.to_owned()
        }
    );
}

/// **ABRIR PERGUNTA SEMPRE**, com ou sem ficheiro na sessão.
///
/// ⛔ Um `Ctrl+O` que reabrisse o caminho da sessão em silêncio deitaria fora o trabalho não
/// gravado com **uma tecla e nenhuma pergunta**. *O gesto que destrói pergunta; o que grava é que
/// pode ser silencioso.*
///
/// **Mutação que deve sangrar:** devolver `Path(session)` quando a sessão tem um.
#[test]
fn opening_always_asks_because_it_throws_work_away() {
    assert!(matches!(open_target(None), IoTarget::Ask { .. }));
    assert!(matches!(
        open_target(Some("/a/hero.ph2dproj")),
        IoTarget::Ask { .. }
    ));
}

/// **Um nome sem extensão ganha a do projeto** — e uma que já é de projeto é respeitada.
///
/// ⚠️ Um seletor nativo devolve o que foi escrito, e no Linux com o portal XDG isso pode vir **sem
/// sufixo nenhum**: um ficheiro `hero` não volta a aparecer no próprio diálogo que o gravou,
/// porque o filtro não o reconhece.
///
/// **Mutação que deve sangrar:** acrescentar a extensão sempre (daria `hero.postcard.ph2dproj`).
#[test]
fn a_chosen_name_always_ends_up_with_a_project_extension() {
    assert_eq!(with_project_extension("/a/hero"), "/a/hero.ph2dproj");
    assert_eq!(
        with_project_extension("/a/hero.ph2dproj"),
        "/a/hero.ph2dproj",
        "a extensao certa nao se duplica"
    );
    assert_eq!(
        with_project_extension("/a/hero.postcard"),
        "/a/hero.postcard",
        "o formato antigo continua a ser um projeto"
    );
    assert_eq!(
        with_project_extension("/a/hero.PH2DPROJ"),
        "/a/hero.PH2DPROJ",
        "e as maiusculas do sistema de ficheiros contam como a mesma"
    );
    // Uma extensão de OUTRA coisa não é um projeto: ela ganha a nossa por cima, e o ficheiro
    // continua a dizer o que era.
    assert_eq!(
        with_project_extension("/a/hero.png"),
        "/a/hero.png.ph2dproj"
    );
}

/// **⚠️ `.ph2d` NÃO é o projeto — ela já é uma IMAGEM neste app.**
///
/// ⛔ Oferecer a mesma extensão para duas coisas diferentes é a ambiguidade que o
/// [`crate::import_router`] curou noutro sítio no mesmo dia. Este gate liga as duas listas para
/// que a colisão não possa nascer por distracção.
#[test]
fn the_project_extension_does_not_collide_with_an_image_one() {
    for e in PROJECT_EXTENSIONS {
        assert!(
            !ph2d_asset::SUPPORTED_IMAGE_EXTENSIONS.contains(e),
            ".{e} e' projeto E imagem — o dialogo passaria a oferecer a mesma coisa para duas"
        );
    }
    assert!(ph2d_asset::SUPPORTED_IMAGE_EXTENSIONS.contains(&"ph2d"));
    assert!(!PROJECT_EXTENSIONS.contains(&"ph2d"));
}

/// **A barra de título diz que projeto está aberto** — e dizia «M5+M6+M7+M11+M12 demo», que é
/// verdade sobre o binário e sobre nada que o artista tenha aberto.
#[test]
fn the_title_carries_the_file_name() {
    assert_eq!(title_name(None), "untitled");
    assert_eq!(
        title_name(Some("/home/enio/jogo/hero.ph2dproj")),
        "hero.ph2dproj"
    );
    assert_eq!(
        title_name(Some("")),
        "untitled",
        "um caminho vazio nao tem nome"
    );
}

/// **A env continua a mandar quando existe** — é o que faz os smokes e os scripts que já dependem
/// dela funcionarem exactamente como antes.
///
/// ⚠️ O gate é sobre a REGRA, e não sobre a leitura: `std::env::set_var` é `unsafe` na edição 2024
/// e esta crate é `#![forbid(unsafe_code)]`. O que fica por gatear é **uma linha** — a leitura —, e
/// está declarado no doc da [`seed_from_env`].
#[test]
fn the_environment_still_seeds_the_session_path() {
    assert_eq!(
        seed_from_env(Some("/tmp/seeded.ph2dproj".to_owned())).as_deref(),
        Some("/tmp/seeded.ph2dproj")
    );
    assert_eq!(
        seed_from_env(Some(String::new())),
        None,
        "uma env VAZIA e' o mesmo que nao haver — senao o primeiro Ctrl+S grava num caminho vazio \
         em vez de perguntar"
    );
    assert_eq!(seed_from_env(None), None);
}

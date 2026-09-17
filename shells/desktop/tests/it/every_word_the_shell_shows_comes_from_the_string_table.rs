//! ⭐⭐⭐ **NENHUMA PALAVRA QUE A SHELL MOSTRA É ESCRITA NO FONTE** — o HR-15 na raiz de composição.
//!
//! > `CLAUDE.md` §0.3: *«UI canônica: … **zero string hardcoded** — tudo via tokens / i18n (HR-15).»*
//!
//! Medido em 2026-09-16: a shell escrevia **496** textos fora das cenas de smoke — quase todos
//! **toasts**, que são a superfície que o artista lê quando um verbo recusa. Migrados por script
//! para `ph2d-i18n/src/shell.rs` e `shell_media.rs`, com as frases montadas a passarem a `tr_with`
//! com marcadores NOMEADOS.
//!
//! # ⚠️ O que fica de fora, e porquê
//!
//! - **As cenas de SMOKE** (`*smoke*.rs`, `*probe*.rs`): elas escrevem NOMES DE OBJECTOS de uma cena
//!   de demonstração (dado da cena, como o nome de uma camada) e INSTRUÇÕES DE CONSOLA para quem as
//!   corre. ⛔ A régua é o nome do ficheiro, que é uma enumeração — por isso vem com **piso de
//!   população**: se ela deixar de casar, o gate acusa em vez de emudecer.
//! - **Formatos de ficheiro, diagnóstico de consola e razões de `#[allow]`**: a lista abaixo, cada
//!   uma com o mecanismo.
//! - **Nomes-identificador de entidade** (`Entity_{:x}`, `piece_{}`): dado, não vocabulário.

use ph2d_label_census::gate::{self, Excecao, Isento};

const TABLES: &[&str] = &[
    "crates/ph2d-i18n/src/shell.rs",
    "crates/ph2d-i18n/src/shell_media.rs",
];

/// Um ficheiro isento inteiro, com o mecanismo.
const FORA: &[Isento] = &[
    (
        "app_state_gfx.rs",
        "razões de `#[allow(..., reason = …)]` — texto que o COMPILADOR lê",
    ),
    (
        "layout_persist.rs",
        "o FORMATO do ficheiro de arrumação (`# PH2D layout`, `dock_w_left=…`)",
    ),
    (
        "palette_persist.rs",
        "o FORMATO do ficheiro de paletas do utilizador",
    ),
    (
        "prefs.rs",
        "o FORMATO do ficheiro de preferências (`chave=valor`)",
    ),
    (
        "project_texture_pattern.rs",
        "razões técnicas de recusa de um blob, escritas para o log",
    ),
    (
        "atlas_loader.rs",
        "erros de arranque do átlas de demonstração — vão para o terminal",
    ),
    (
        "integration.rs",
        "a demo de integração (script de exemplo e erros de arranque, no terminal)",
    ),
    (
        "envelope_gesture.rs",
        "o diagnóstico `vec_overlay_diag::refused`, que só sai com a env ligada",
    ),
    (
        "legacy_chrome.rs",
        "o `eprintln!` que diz se o chrome legado está visível (bissecção)",
    ),
    (
        "input_dispatch/keyboard.rs",
        "o diagnóstico do acorde de undo, impresso no terminal",
    ),
    (
        "input_dispatch/keyboard_tail.rs",
        "o mesmo diagnóstico do acorde de undo, no fim da cadeia",
    ),
    (
        "input_dispatch/despacho_metodos_modos_e_alcas.rs",
        "as razões de teclas mortas, impressas no terminal por quem caça um report",
    ),
    (
        "render_loop/fase_path_shape_and_paint.rs",
        "o `log_shape` do padrão de textura — diagnóstico de consola",
    ),
    (
        "render_loop/fase_selection_highlight.rs",
        "o `warp_overlay::diag` — diagnóstico de consola",
    ),
    (
        "render_loop/fase_game_camera.rs",
        "o relatório da câmera de jogo, impresso no terminal",
    ),
    (
        "render_loop/fase_sculpt3d_bake.rs",
        "as linhas `[sculpt3d]` do padrão de alpha, impressas no terminal",
    ),
    (
        "render_loop/fase_selection_mirror_bone_focus.rs",
        "o retrato de foco de osso, impresso no terminal",
    ),
    (
        "render_loop/cooked_texture_bridge.rs",
        "o aviso de textura cozida em falta, impresso uma vez por id",
    ),
    (
        "render_loop/present_title.rs",
        "o título da JANELA com os contadores de diagnóstico (sprites, átlas)",
    ),
];

/// Um literal isento, com o mecanismo — nomes que são DADO.
const NOT_LANGUAGE: &[Excecao] = &[
    (
        "hero_bridge.rs",
        "Entity_{:x}",
        "o nome de recurso de uma entidade sem `Name` — um IDENTIFICADOR (`Entity_1f`), não vocabulário",
    ),
    (
        "render_loop/snapshots_inspector.rs",
        "Entity_{bits:x}",
        "o mesmo identificador de recurso do `hero_bridge`, no retrato do Inspector",
    ),
    (
        "hero_intents/image_edit/bgremoval.rs",
        "sprite_{entity_bits:x}",
        "o nome-base derivado dos bits da entidade, para as ilhas separadas",
    ),
    (
        "hero_intents/image_edit/bgremoval.rs",
        "{base_name}_island{n}",
        "o nome de uma ilha separada — derivado do nome-base, um identificador",
    ),
    (
        "sheet_bake.rs",
        "piece_{}",
        "o nome de recurso de uma peça de folha sem `Name` — identificador",
    ),
    (
        "sheet_frame.rs",
        "piece_{b:x}",
        "o mesmo identificador de peça, no retrato da folha",
    ),
];

#[test]
fn every_word_the_shell_shows_comes_from_the_string_table() {
    let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let intrusos = gate::intrusos_fora_de(&src, NOT_LANGUAGE, FORA);
    assert!(
        intrusos.is_empty(),
        "texto com cara de língua escrito no fonte da shell (HR-15):\n  {}\n\n\
         A cura é uma chave `shell.<ficheiro>.<frase>` em `{TABLES:?}` e um `tr(\"…\")` no sítio \
         (uma frase com peças do código: `tr_with`, com marcadores NOMEADOS). ⚠️ Se o texto NÃO é \
         língua (um identificador, um formato de ficheiro, uma linha de terminal), a cura é uma \
         linha em `FORA` ou em `NOT_LANGUAGE` **com o mecanismo**.",
        intrusos.join("\n  ")
    );
}

/// ⭐ **A metade justa das isenções** — cada uma ainda abriga o que nomeia, e o piso das cenas de
/// smoke é o controlo de vacuidade da régua por NOME DE FICHEIRO.
#[test]
fn every_named_exemption_still_shelters_what_it_names() {
    let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mortas: Vec<String> = gate::excecoes_mortas(&src, NOT_LANGUAGE)
        .into_iter()
        .chain(gate::isentos_mortos(&src, FORA))
        .collect();
    assert!(
        mortas.is_empty(),
        "isenções mortas:\n  {}",
        mortas.join("\n  ")
    );
    let em_smoke = gate::literais_de_cena(&src);
    assert!(
        em_smoke >= 200,
        "as cenas de smoke abrigam {em_smoke} textos — em 2026-09-16 eram 421. Ou elas saíram da \
         shell (apague a regra), ou a régua por NOME deixou de casar e este gate passou a medir o \
         nada (a armadilha do censo por prefixo, HOWTO §2.7)"
    );
}

/// ⭐⭐ **As chaves existem dos dois lados** — uma chave sem braço pinta o identificador cru e vaza.
#[test]
fn every_shell_key_exists_on_both_sides() {
    let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("shells/desktop tem dois pais")
        .to_path_buf();
    let c = gate::chaves(&repo, "shell.", TABLES);
    assert!(
        c.declaradas >= 300 && c.usadas >= 300,
        "o censo achou {} declaradas e {} usadas — está a ler o sítio errado",
        c.declaradas,
        c.usadas
    );
    assert!(
        c.sem_traducao.is_empty(),
        "usadas e NÃO declaradas — o `tr` pinta o identificador cru:\n  {}",
        c.sem_traducao.join("\n  ")
    );
    assert!(
        c.orfas.is_empty(),
        "declaradas e ninguém as usa — apague-as:\n  {:?}",
        c.orfas
    );
}

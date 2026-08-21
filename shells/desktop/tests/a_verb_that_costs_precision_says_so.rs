//! **QUEM LÊ PIXELS E OS ESCREVE DE VOLTA EM 8 BITS TEM DE O DIZER.**
//!
//! Plano [`docs/Sprite_projeto/18`](../../docs/Sprite_projeto/18_precisao_de_16_bits_nas_sprites.md)
//! W7 · auditoria [`docs/Sprite_projeto/19`](../../docs/Sprite_projeto/19_auditoria_precisao_por_ferramenta.md) §5.
//!
//! # O buraco que este gate fecha, e como ele passou por uma auditoria inteira
//!
//! A auditoria de 2026-08-20 respondeu à ordem *"auditoria completa com cada tool"* e varreu as
//! **ferramentas** — as nove entradas da fila de Image Tools. Ela estava certa e ficou incompleta,
//! porque a pergunta que ela fez foi *por NOME DE MENU*.
//!
//! ⚠️ **Há quatro verbos que consomem pixels pela mesma porta e não estão naquele menu:**
//!
//! | verbo | onde | o que custava, em silêncio |
//! |---|---|---|
//! | Pack / Bake Sheet | `sheet_bake.rs` | uma folha é **uma** textura, e é de 8 bits |
//! | Merge Sprites | `sprite_merge.rs` | o acumulador é de 8 bits — **e despawna as fontes** |
//! | `Strategy → Atlas` | `inspector_strategy.rs` | o atlas partilhado é de 8 bits |
//! | doar a forma ao 3D | `sculpt3d_bake.rs` | o `base × luz` é de 8 bits |
//!
//! Os quatro rebaixavam 16 bits sem uma palavra, e o único vestígio era a linha `Format` do
//! Inspector mudar sozinha — que é **literalmente** a queixa que abriu esta wave (Enio, 2026-08-20:
//! *"após aplicar algumas das tools a sprite volta para RGBA8 no inspector"*). *Uma auditoria por
//! nome de menu deixa de fora tudo o que não está naquele menu; a pergunta certa é «quem lê pixels
//! e os escreve de volta».*
//!
//! # Como o gate classifica, e por que há um marcador em vez de uma lista
//!
//! Todo ficheiro que chama `read_sprite_source` tem de nomear **uma** destas coisas:
//!
//! - `commit_geometric_edit` / `commit_edited_texture` — as portas que já avisam por dentro;
//! - `warn_precision_loss` / `holds_sixteen_bit` — a lei nomeada no próprio sítio;
//! - `PRECISION-READONLY:` — a declaração de que aquele sítio **não escreve pixels de volta**
//!   (prévias, medições, hit-tests), com o motivo ao lado.
//!
//! ⚠️ **O marcador viaja com o código, e uma allowlist não.** Já está registado neste projeto que
//! uma lista central envelhece longe do que a justifica e a primeira pessoa apressada acrescenta-lhe
//! uma linha em vez de encaminhar o sítio — é a mesma razão do `PRECISION-BYPASS:` no gate irmão
//! [`reading_pixels_goes_through_the_precision_door`].

use std::path::{Path, PathBuf};

/// A porta por onde os pixels de um sprite são lidos.
const READS_PIXELS: &str = "read_sprite_source(";

/// **CHAMADAS** que satisfazem o gate — procuradas no corpo, nunca num comentário.
///
/// ⚠️ **A distinção custou um mutante sobrevivente.** A 1ª versão procurava todos os marcadores no
/// ficheiro inteiro, e o `inspector_strategy.rs` passava por ter a frase *"converte para Individual
/// (`commit_edited_texture`)"* num **doc-comment** — apagar a chamada de aviso real deixava o gate
/// **verde**. Um ficheiro que *fala* sobre a porta não passa por ela.
const CALLS: [&str; 4] = [
    // As duas portas de commit, que avisam por dentro.
    "commit_geometric_edit(",
    "commit_edited_texture(",
    // A lei nomeada directamente (verbos que não passam por um commit de ferramenta).
    "warn_precision_loss(",
    "holds_sixteen_bit(",
];

/// «Este sítio não escreve pixels de volta», com o motivo ao lado. Vive **num comentário**, por
/// isso é o único procurado no ficheiro inteiro.
const READONLY_MARKER: &str = "PRECISION-READONLY:";

fn shell_src() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = std::fs::read_dir(dir).unwrap_or_else(|e| panic!("read_dir {dir:?}: {e}"));
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// O corpo sem comentários de linha — vários ficheiros **citam** `read_sprite_source` num
/// doc-comment a explicar o desenho, e um comentário não lê pixel nenhum.
///
/// ⚠️ O marcador `PRECISION-READONLY:` vive **num comentário**, por isso ele é procurado no ficheiro
/// inteiro e não neste corpo. Os outros quatro são chamadas, e são procurados aqui.
fn code_without_comments(src: &str) -> String {
    src.lines()
        .map(|l| l.split("//").next().unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn every_pixel_consumer_declares_what_it_costs() {
    let mut files = Vec::new();
    rust_files(&shell_src(), &mut files);
    assert!(
        files.len() > 50,
        "so' {} ficheiros varridos — a varredura partiu-se e este gate mede o vazio",
        files.len()
    );

    let mut offenders = Vec::new();
    for path in &files {
        let src = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {path:?}: {e}"));
        let body = code_without_comments(&src);
        if !body.contains(READS_PIXELS) {
            continue;
        }
        let declared = CALLS.iter().any(|c| body.contains(c)) || src.contains(READONLY_MARKER);
        if !declared {
            offenders.push(format!("  {}", path.display()));
        }
    }

    assert!(
        offenders.is_empty(),
        "estes ficheiros leem pixels de um sprite sem declarar o que fazem a' precisao dele:\n{}\
         \n\n\
         `read_sprite_source` devolve SEMPRE 8 bits (o `SpriteImage` e' `Vec<u8>`). Se o sitio \
         escreve os pixels de volta, uma sprite de 16 bits e' rebaixada -- e sem uma palavra o \
         unico vestigio e' a linha `Format` do Inspector mudar sozinha, que e' a queixa que abriu \
         esta wave.\n\n\
         Escolha UMA:\n\
         - passa por `commit_geometric_edit` / `commit_edited_texture` (avisam por dentro);\n\
         - chama `texture_edit::warn_precision_loss(..)` dizendo DE QUE recurso e' o limite;\n\
         - se o sitio nao escreve pixels de volta (previa, medicao, hit-test), escreva \
         `PRECISION-READONLY: <motivo>` num comentario ao lado.\n\n\
         ⛔ Nao acrescente o ficheiro a uma lista dentro deste teste: nao ha' lista, de proposito.",
        offenders.join("\n")
    );
}

/// ⚠️ **Controle positivo: os QUATRO verbos que a varredura de 2026-08-21 encontrou leem mesmo
/// pixels, e agora declaram.**
///
/// Sem isto, renomear `read_sprite_source` faria o gate acima ficar verde por não encontrar nada —
/// e o dia em que ele parasse de medir seria o dia em que ninguém repararia.
#[test]
fn the_four_verbs_the_sweep_found_are_real_and_now_declare() {
    for (rel, what) in [
        ("sheet_bake.rs", "uma folha e' UMA textura, e e' de 8 bits"),
        (
            "hero_intents/sprite_merge.rs",
            "o acumulador e' de 8 bits, e a fusao despawna as fontes",
        ),
        (
            "render_loop/inspector_strategy.rs",
            "o atlas partilhado e' de 8 bits",
        ),
        ("sculpt3d_bake.rs", "o `base x luz` e' de 8 bits"),
    ] {
        let path = shell_src().join(rel);
        let src = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path:?}: {e}"));
        let body = code_without_comments(&src);
        assert!(
            body.contains(READS_PIXELS),
            "{rel} deixou de chamar `{READS_PIXELS}` — se isso e' verdade, APAGUE esta entrada em \
             vez de a silenciar"
        );
        assert!(
            body.contains("warn_precision_loss(") || body.contains("holds_sixteen_bit("),
            "{rel} voltou a rebaixar 16 bits em silencio ({what})"
        );
    }
}

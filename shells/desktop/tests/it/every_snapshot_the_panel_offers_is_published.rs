//! ⭐⭐⭐ **Todo instantâneo que o painel do Inspector OFERECE é PUBLICADO pela shell.**
//!
//! # O defeito que isto cerca
//!
//! O painel é chrome: ele não vê o mundo, e cada secção viva só existe porque a shell lhe **entrega**
//! um instantâneo por quadro (`set_current_inspector_*`). Um publicador escrito e nunca chamado é
//! **mudo**: a secção compila, o `populate` regista, o pintor sabe desenhá-la — e ela nunca aparece,
//! porque o snapshot dela fica para sempre em `None`. ⛔ *Isso lê-se como «a feature não foi feita»,
//! e nenhum gate deste repo o via.*
//!
//! ⚠️ **E o inverso também é mudo:** uma auditoria de 2026-09-13 (Tags W3b) mediu que **nenhum** dos
//! quinze publicadores tinha gate nenhum. Este censo nasceu com a 16.ª porta e cobre as dezasseis.
//!
//! # ⭐⭐ Nem toda porta `set_current_*` é um INSTANTÂNEO, e a isenção DERIVA-SE
//!
//! Há duas famílias, e a primeira corrida deste censo mostrou-a: o `set_current_display_angle` é
//! **contexto de pintura** (o painel escreve-o a si próprio no `paint`, a partir de um argumento que
//! recebe), não um retrato da cena. ⛔ **A cura NÃO é uma allowlist** — é o discriminador certo:
//! *quem o painel já chama não precisa da shell; quem ninguém chama é que é o defeito*. Assim uma
//! porta nova de contexto entra isenta sozinha, e uma porta de instantâneo esquecida continua a
//! reprovar. ⚠️ *Uma lista escrita à mão aqui envelheceria na próxima porta de contexto, e a
//! primeira pessoa a acrescentá-la iria acrescentar-se à lista em vez de perguntar porquê.*
//!
//! # ⚠️ As duas metades obrigatórias (HOWTO §2.7 e CLAUDE.md §5.0)
//!
//! - **Piso de população**: um censo que passa a encontrar zero publicadores fica verde a medir
//!   nada. Ele exige ler a crate do painel e achar lá pelo menos `DEZ` `pub fn set_current_*`.
//! - **Obsolescência**: não há allowlist. Uma porta que a shell deixe de chamar reprova, e a cura é
//!   apagar a porta ou ligá-la — nunca uma entrada numa lista tolerada.
//!
//! ⚠️ **A varredura tira comentários e textos dos DOIS lados.** Este próprio ficheiro nomeia
//! `set_current_inspector_*` em prosa, e a shell tem doc-comments que citam os publicadores pelo
//! nome — contá-los faria o censo aprovar-se a si mesmo.

use std::path::{Path, PathBuf};

/// Quantos publicadores a crate do painel tem de ter, no mínimo, para a varredura valer.
const PISO: usize = 10;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("a raiz da workspace")
        .to_path_buf()
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        let name = e.file_name();
        let name = name.to_string_lossy();
        if p.is_dir() {
            if name == "target" || name.starts_with('.') {
                continue;
            }
            rust_files(&p, out);
        } else if name.ends_with(".rs") {
            out.push(p);
        }
    }
}

/// Os publicadores que a crate do painel EXPORTA — `pub fn set_current_…`.
fn publicadores(root: &Path) -> Vec<String> {
    let src = std::fs::read_to_string(root.join("crates/ph2d-panel-inspector/src/state.rs"))
        .expect("o state do painel existe onde o censo o procura");
    let limpo = crate::rust_src::strip_comments_and_strings(&src);
    let mut out: Vec<String> = limpo
        .match_indices("pub fn set_current_")
        .filter_map(|(i, _)| {
            let resto = &limpo[i + "pub fn ".len()..];
            let fim = resto.find('(')?;
            Some(resto[..fim].trim().to_string())
        })
        .collect();
    out.sort_unstable();
    out.dedup();
    out
}

/// Todo o `.rs` de `dir`, sem comentários nem textos, com piso de população.
fn fonte_de(dir: &Path, piso: usize) -> String {
    fonte_de_excepto(dir, piso, "")
}

/// Idem, saltando o ficheiro chamado `excepto` (vazio = nenhum).
fn fonte_de_excepto(dir: &Path, piso: usize, excepto: &str) -> String {
    let mut ficheiros = Vec::new();
    rust_files(dir, &mut ficheiros);
    assert!(
        ficheiros.len() > piso,
        "o censo so' leu {} ficheiros em {dir:?} — a varredura partiu (piso {piso})",
        ficheiros.len()
    );
    ficheiros
        .iter()
        .filter(|f| excepto.is_empty() || f.file_name().is_none_or(|n| n != excepto))
        .filter_map(|f| std::fs::read_to_string(f).ok())
        .map(|s| crate::rust_src::strip_comments_and_strings(&s))
        .collect::<Vec<_>>()
        .join("\n")
}

/// ⭐⭐⭐ **Cada uma delas tem CHAMADOR** — a shell, se for um instantâneo; o painel, se for
/// contexto de pintura.
///
/// **Mutação que deve sangrar:** apagar uma linha `ph2d_panel_inspector::set_current_…(…)` do
/// `render_loop/snapshots_inspector.rs`.
#[test]
fn every_snapshot_the_panel_offers_is_published() {
    let root = workspace_root();
    let portas = publicadores(&root);
    assert!(
        portas.len() >= PISO,
        "o censo so' achou {} publicadores no painel — a varredura partiu (piso {PISO})",
        portas.len()
    );

    let shell = fonte_de(&root.join("shells/desktop/src"), 100);
    // ⭐ O painel SEM o `state.rs`: é ali que as portas são DECLARADAS, e contá-lo faria toda porta
    // parecer ter chamador (a declaração casa com `nome(`).
    let painel = fonte_de_excepto(
        &root.join("crates/ph2d-panel-inspector/src"),
        20,
        "state.rs",
    );

    // ⚠️ Procura-se a CHAMADA (`nome(`), não o nome solto: um `use` ou uma re-exportação não é
    // uma publicação, e contá-los deixaria uma porta morta a passar por viva.
    let chamada = |p: &String| format!("{p}(");
    let mudas: Vec<&String> = portas
        .iter()
        .filter(|p| !shell.contains(&chamada(p)) && !painel.contains(&chamada(p)))
        .collect();
    assert!(
        mudas.is_empty(),
        "o painel OFERECE estas portas e NINGUEM as chama: {mudas:?}\n\
         Um publicador sem chamador deixa a seccao dele para sempre em `None` — ela compila, \
         regista e pinta, e nunca aparece. Ligue-a no `render_loop/snapshots_inspector.rs` (se for \
         um instantaneo da cena) ou no `paint` do painel (se for contexto de pintura), ou apague a \
         porta."
    );
}

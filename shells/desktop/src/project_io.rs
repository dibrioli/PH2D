//! **O ficheiro do projeto tem NOME** — `Save`, `Save As…` e `Open Project…` com diálogo.
//!
//! # A sessão tem um caminho, e ele é a diferença entre Save e Save As
//!
//! `App::recovery.project_path` é o ficheiro **desta** sessão. Ele nasce da env quando ela existe
//! (é o que mantém os smokes e os scripts a funcionar como sempre), do ficheiro que um `Open`
//! abriu, ou de um `Save As`. A política é uma função pura:
//!
//! | gesto | com caminho na sessão | sem caminho |
//! |---|---|---|
//! | `Ctrl+S` / **Save** | grava, sem perguntar | **pergunta** (é o primeiro save) |
//! | `Ctrl+Shift+S` / **Save As…** | pergunta sempre | pergunta sempre |
//! | `Ctrl+O` / **Open Project…** | pergunta sempre | pergunta sempre |
//!
//! ⚠️ **Abrir pergunta SEMPRE:** o gesto que DESTRÓI (o trabalho não gravado) pergunta; o que
//! grava é que pode ser silencioso.
//!
//! # ⚠️ A extensão do projeto NÃO é `.ph2d`
//!
//! `ph2d` é uma **imagem** (`ph2d_asset::SUPPORTED_IMAGE_EXTENSIONS`). O projeto é **`.ph2dproj`**;
//! o `.postcard` continua a abrir o que já foi gravado.

use ph2d_i18n::{tr, tr_with};
use std::path::Path;

/// As extensões do projeto, para o diálogo: a primeira é a que um `Save As` sugere; as outras
/// abrem o que já está gravado.
pub(crate) const PROJECT_EXTENSIONS: &[&str] = &["ph2dproj", "postcard"];

/// O nome sugerido quando a sessão ainda não tem ficheiro.
pub(crate) const UNTITLED: &str = "untitled.ph2dproj";

/// O que um pedido de I/O precisa antes de acontecer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum IoTarget {
    /// Faz-se neste caminho, sem perguntar.
    Path(String),
    /// Pergunta-se ao artista, sugerindo este nome de ficheiro.
    Ask { suggested: String },
}

/// **Onde um `Save` grava.** `force_ask` é o `Save As…`.
///
/// ⚠️ O primeiro save de uma sessão sem ficheiro **pergunta** — gravar em silêncio num nome
/// inventado é como o trabalho acaba num ficheiro que ninguém encontra depois.
#[must_use]
pub(crate) fn save_target(session: Option<&str>, force_ask: bool) -> IoTarget {
    match session {
        Some(p) if !force_ask && !p.is_empty() => IoTarget::Path(p.to_owned()),
        other => IoTarget::Ask {
            suggested: other.map_or_else(
                || UNTITLED.to_owned(),
                |p| file_name(p).unwrap_or_else(|| UNTITLED.to_owned()),
            ),
        },
    }
}

/// **Abrir pergunta sempre.** Ver o doc do módulo: o gesto que destrói o trabalho não gravado não
/// pode acontecer com uma tecla e sem uma pergunta.
#[must_use]
pub(crate) fn open_target(session: Option<&str>) -> IoTarget {
    IoTarget::Ask {
        suggested: session
            .and_then(file_name)
            .unwrap_or_else(|| UNTITLED.to_owned()),
    }
}

/// O nome do ficheiro de um caminho, sem a pasta.
fn file_name(path: &str) -> Option<String> {
    Path::new(path)
        .file_name()
        .and_then(|s| s.to_str())
        .map(ToOwned::to_owned)
}

/// **O caminho que o artista escolheu, com uma extensão de projeto garantida.**
///
/// ⚠️ Um seletor nativo devolve o que foi escrito, e num Linux com o portal XDG isso pode vir
/// **sem extensão nenhuma** — um ficheiro `hero` sem sufixo não volta a aparecer no próprio
/// diálogo que o gravou, porque o filtro não o reconhece. Uma extensão que já é de projeto é
/// respeitada (o artista pode querer `.postcard`).
#[must_use]
pub(crate) fn with_project_extension(chosen: &str) -> String {
    let has = Path::new(chosen)
        .extension()
        .and_then(|s| s.to_str())
        .is_some_and(|e| PROJECT_EXTENSIONS.iter().any(|x| e.eq_ignore_ascii_case(x)));
    if has {
        chosen.to_owned()
    } else {
        format!("{chosen}.{}", PROJECT_EXTENSIONS[0])
    }
}

/// **O que a barra de título mostra** — o nome do ficheiro, ou `untitled` enquanto não houver um.
#[must_use]
pub(crate) fn title_name(session: Option<&str>) -> String {
    session
        .and_then(file_name)
        .unwrap_or_else(|| "untitled".to_owned())
}

/// **A regra do que a env semeia**, separada da leitura dela.
///
/// ⚠️ Pura porque `std::env::set_var` é `unsafe` na edição 2024 e esta crate é
/// `#![forbid(unsafe_code)]` — um gate que mexesse na env do processo não compila aqui, e um que
/// mexesse noutra crate mediria outra coisa. O que fica por gatear é **uma linha**: a leitura.
///
/// Uma env **vazia** é o mesmo que não haver: senão a sessão nasce com um ficheiro sem nome, e o
/// primeiro `Ctrl+S` grava num caminho vazio em vez de perguntar.
#[must_use]
pub(crate) fn seed_from_env(var: Option<String>) -> Option<String> {
    var.filter(|s| !s.is_empty())
}

impl crate::App {
    /// **O caminho com que a sessão nasce.** A env continua a mandar quando existe — é o que faz
    /// os smokes e os scripts que já dependem dela funcionarem exactamente como antes. Sem ela, a
    /// sessão nasce **sem ficheiro**, e o primeiro `Ctrl+S` pergunta.
    #[must_use]
    pub(crate) fn initial_project_path() -> Option<String> {
        seed_from_env(std::env::var("PH2D_PROJECT_PATH").ok())
    }

    /// Abre o seletor nativo para GRAVAR e devolve o caminho escolhido, já com extensão.
    fn ask_where_to_save(suggested: &str) -> Option<String> {
        let mut dialog = rfd::FileDialog::new().set_file_name(suggested);
        for ext in PROJECT_EXTENSIONS {
            dialog = dialog.add_filter(
                tr_with("shell.project_io.ph2d_project_2", &[("ext", &ext)]),
                &[*ext],
            );
        }
        dialog
            .save_file()
            .and_then(|p| p.to_str().map(with_project_extension))
    }

    /// Abre o seletor nativo para ABRIR.
    fn ask_what_to_open() -> Option<String> {
        rfd::FileDialog::new()
            .add_filter(tr("shell.project_io.ph2d_project"), PROJECT_EXTENSIONS)
            .pick_file()
            .and_then(|p| p.to_str().map(ToOwned::to_owned))
    }

    /// **Grava.** `force_ask` = `Save As…`.
    pub(crate) fn project_save_gesture(&mut self, force_ask: bool) {
        let target = save_target(self.recovery.project_path.as_deref(), force_ask);
        let path = match target {
            IoTarget::Path(p) => p,
            IoTarget::Ask { suggested } => match Self::ask_where_to_save(&suggested) {
                Some(p) => p,
                None => return, // o artista desistiu — e desistir não é um erro
            },
        };
        self.recovery.project_path = Some(path.clone());
        self.project_save_to(&path);
        self.title_dirty = true;
    }

    /// **Abre.** Pergunta sempre (ver o doc do módulo).
    pub(crate) fn project_open_gesture(&mut self) {
        let IoTarget::Ask { .. } = open_target(self.recovery.project_path.as_deref()) else {
            return;
        };
        let Some(path) = Self::ask_what_to_open() else {
            return;
        };
        if self.project_load_from(&path) {
            (self.recovery.project_path, self.title_dirty) = (Some(path), true);
            self.recovery_opened();
        }
    }

    /// **Drena os itens do menu Ficheiro**, uma vez por quadro, com `self` livre do borrow do
    /// render loop — o mesmo sítio e a mesma razão do `post_frame_undo`.
    ///
    /// ⚠️ **O menu e o teclado chamam as MESMAS funções.** Duas portas para o mesmo gesto é como o
    /// `.ase` ficou invisível no diálogo de import no mesmo dia (`crate::import_router`), e aqui a
    /// divergência seria pior: um `Save` do menu que gravasse noutro sítio que o `Ctrl+S`.
    pub(crate) fn drain_project_io(&mut self) {
        let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) else {
            return;
        };
        let asked = std::mem::take(&mut hero.file_menu);
        let (save, save_as, open) = (asked.save, asked.save_as, asked.open);
        if save || save_as {
            self.project_save_gesture(save_as);
        }
        if open {
            self.project_open_gesture();
        }
        // ⭐ **Exportar SVG** (plano 40) — a mesma porta das outras três: o menu levanta a bandeira,
        // o shell é que tem o disco e o selector de ficheiros.
        if asked.export_svg {
            self.export_svg_gesture();
        }
    }
}

#[cfg(test)]
#[path = "project_io_tests.rs"]
mod tests;

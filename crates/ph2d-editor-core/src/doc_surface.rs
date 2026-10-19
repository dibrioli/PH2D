//! ⭐ **Documentos de FAMÍLIA em aba** (CodeEditor Onda 1, E3 — `docs/CodeEditor/02_onda1.md`).
//!
//! Os quadros moram na fundação; um documento de família (o editor de scripts) mora na crate DELA e
//! liga-se aqui por um trait — o molde do [`crate::screens::hero::mode_drive::ModeFamily`]. A shell
//! instala a família uma vez ([`crate::documents::Documents::install_surface`]); o hero pinta as abas,
//! dá a área central à aba activa e entrega-lhe rato, roda, teclado, texto e composição pelas MESMAS
//! portas por onde o quadro os recebe. Dados da cena (scripts, recarga) a família troca com o mundo
//! pela crate dela, fora daqui.

use crate::ime::ComposeKey;
use crate::zones::Rect;
use ph2d_host::{Modifiers, PointerButton, PointerKind};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;
use std::any::Any;

/// Uma aba aberta por uma família: `id` é dela (estável enquanto a aba vive).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocTab {
    pub id: u64,
    pub title: String,
    /// Há alterações por salvar (a aba mostra-o).
    pub dirty: bool,
}

/// Uma tecla, já sem o desenho físico do teclado (o mesmo vocabulário do quadro).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Enter,
    Escape,
    Backspace,
    Delete,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    Space,
    Tab,
    PageUp,
    PageDown,
    Other,
    /// Uma tecla de função (`F(12)` = F12).
    F(u8),
}

/// O que uma tecla fez.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KeyOutcome {
    /// O documento tomou a tecla: nenhum atalho da cena a vê.
    pub consumed: bool,
    /// Texto para a área de transferência do sistema (copiar/cortar).
    pub copy: Option<String>,
}

/// A composição de um carácter (IME ou tecla morta), entregue ao documento activo.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Compose<'a> {
    /// Pré-edição: mostra-se no cursor, não entra no texto. Vazia = acabou.
    Preedit(&'a str, Option<(usize, usize)>),
    /// Texto confirmado (IME `Commit` ou um carácter do caminho do texto).
    Commit(char),
    /// Tecla vista pela composição sem IME (tecla morta).
    DeadKey(ComposeKey),
}

/// Uma tecla para a aba: premida ou solta, o que ela escreve (`text`) e, no Ctrl+V, a área de
/// transferência (`paste`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyInput<'a> {
    pub key: Key,
    pub mods: Modifiers,
    pub pressed: bool,
    pub text: Option<&'a str>,
    pub paste: Option<String>,
}

/// O menu da barra onde um comando de documento entra.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DocMenu {
    File,
    Edit,
}

/// Um comando de uma aba de família (ver [`DocumentSurface::commands`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocCommand {
    pub id: ph2d_a11y::NodeId,
    /// O rótulo já traduzido, com o atalho («Go to Line… · Cmd+G»).
    pub label: String,
    pub menu: DocMenu,
}

/// O tipo de um símbolo do documento (o grupo dele no `@` da paleta, I14).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DocSymbolKind {
    Function,
    Property,
    Variable,
}

/// Um símbolo do documento activo, para o `@` da paleta (CodeEditor I14). Linha e coluna (em
/// caracteres) 0-based, no INÍCIO do nome (onde o VS Code põe o cursor).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocSymbol {
    pub name: String,
    /// O contentor (`Bob` em `Bob:update`), em cinzento ao lado.
    pub container: Option<String>,
    pub kind: DocSymbolKind,
    pub line: usize,
    pub col: usize,
}

/// Os sítios do documento activo a que a paleta leva: `@` símbolos, `:` linhas.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DocPlaces {
    /// Pela ordem do documento.
    pub symbols: Vec<DocSymbol>,
    pub lines: usize,
}

/// O ponteiro que uma aba quer sob o rato (a shell traduz para o do sistema).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PointerLook {
    /// A seta.
    #[default]
    Arrow,
    /// Sobre texto editável: o I.
    Text,
    /// Sobre o que um clique leva a algum sítio: a mão.
    Hand,
}

/// O que um evento do rato traz além da posição.
pub struct PointerInput<'a> {
    pub text: &'a mut TextSystem,
    pub mods: Modifiers,
    pub now_ns: u128,
}

/// ⭐ **Uma família com documentos em aba.** Cada chamada leva o `id` da aba a que se dirige.
pub trait DocumentSurface: Any {
    /// As abas abertas, pela ordem em que se mostram.
    fn tabs(&self) -> Vec<DocTab>;
    /// Pinta o documento em `area` (o fundo também: a cena está por baixo).
    fn paint(
        &mut self,
        id: u64,
        area: Rect,
        scene: &mut VectorScene,
        text: &mut TextSystem,
        theme: &Theme,
    );
    /// Carregar/largar dentro da área. `true` = tomado.
    fn pointer(
        &mut self,
        id: u64,
        input: PointerInput<'_>,
        kind: PointerKind,
        button: PointerButton,
        x: f32,
        y: f32,
    ) -> bool;
    /// Movimento (com ou sem botão). `true` = tomado.
    fn pointer_move(&mut self, id: u64, input: PointerInput<'_>, x: f32, y: f32) -> bool;
    /// Roda sobre a área (`shift` = o Shift premido: rolar na horizontal). `true` = tomada.
    fn wheel(&mut self, id: u64, x: f32, y: f32, dy: f32, shift: bool) -> bool;
    /// Uma tecla premida ou solta ([`KeyInput`]).
    fn key(&mut self, id: u64, text_system: &mut TextSystem, input: KeyInput<'_>) -> KeyOutcome;
    /// Composição (IME / tecla morta). `true` = tomada.
    fn compose(&mut self, id: u64, c: Compose<'_>) -> bool;
    /// O artista fechou a aba. A família decide (pode guardar o que estava por salvar).
    fn close(&mut self, id: u64);
    /// Desfazer/refazer pedido pelo menu Edit com esta aba activa. `true` = havia o que fazer.
    fn undo(&mut self, id: u64, redo: bool) -> bool;
    /// Os nós de acessibilidade da aba (o `Document` primeiro, depois um `TextRun` por linha visível).
    fn a11y(&self, id: u64) -> Vec<(ph2d_a11y::NodeId, ph2d_a11y::Node)> {
        let _ = id;
        Vec::new()
    }
    /// O caret em pixels lógicos (a janela de candidatos do IME).
    fn caret_area(&self, id: u64) -> Option<Rect> {
        let _ = id;
        None
    }
    /// A aba que a família quer que fique activa (abriu um documento). Lida e limpa pelo hero.
    fn take_focus_request(&mut self) -> Option<u64> {
        None
    }
    /// Os comandos da aba: linhas nos menus da barra (com o atalho no rótulo) e na paleta `Ctrl+K`,
    /// só enquanto a aba está activa.
    fn commands(&self, id: u64) -> Vec<DocCommand> {
        let _ = id;
        Vec::new()
    }
    /// Corre o comando `cmd` (um id de [`Self::commands`]). `true` = era dela.
    fn command(&mut self, id: u64, cmd: ph2d_a11y::NodeId) -> bool {
        let _ = (id, cmd);
        false
    }
    /// Os sítios da aba para o `@` e o `:` da paleta (I14); `None` = ela não tem nenhum.
    fn places(&mut self, id: u64) -> Option<DocPlaces> {
        let _ = id;
        None
    }
    /// Leva o cursor da aba à linha e coluna (0-based, em caracteres; a coluna presa à linha).
    fn go_to(&mut self, id: u64, line: usize, col: usize) -> bool {
        let _ = (id, line, col);
        false
    }
    /// O ponteiro que a aba quer em `(x, y)`; `None` = fora dela (decide a shell). Perguntado a
    /// cada quadro: muda sem o rato mexer (uma tecla, uma resposta que chegou).
    fn pointer_look(&self, id: u64, x: f32, y: f32) -> Option<PointerLook> {
        let _ = (id, x, y);
        None
    }
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Uma aba de família: (índice da família instalada, id da aba).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SurfaceTab {
    pub family: u8,
    pub id: u64,
}

/// As famílias instaladas (por ordem de instalação: a ordem das abas).
#[derive(Default)]
pub struct Surfaces {
    pub(crate) families: Vec<Box<dyn DocumentSurface>>,
}

impl std::fmt::Debug for Surfaces {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(self.families.iter().map(|_| ()))
            .finish()
    }
}

impl Surfaces {
    pub fn get(&mut self, family: u8) -> Option<&mut (dyn DocumentSurface + 'static)> {
        self.families.get_mut(usize::from(family)).map(|b| &mut **b)
    }

    /// A família do tipo `T` (a crate dela fala com o estado dela por aqui).
    pub fn find_mut<T: DocumentSurface>(&mut self) -> Option<&mut T> {
        self.families
            .iter_mut()
            .find_map(|f| f.as_any_mut().downcast_mut::<T>())
    }

    pub fn find<T: DocumentSurface>(&self) -> Option<&T> {
        self.families
            .iter()
            .find_map(|f| f.as_any().downcast_ref::<T>())
    }

    /// Todas as abas de todas as famílias, pela ordem das famílias.
    pub fn tabs(&self) -> Vec<(SurfaceTab, DocTab)> {
        let mut out = Vec::new();
        for (i, f) in self.families.iter().enumerate() {
            let family = u8::try_from(i).unwrap_or(u8::MAX);
            out.extend(
                f.tabs()
                    .into_iter()
                    .map(|t| (SurfaceTab { family, id: t.id }, t)),
            );
        }
        out
    }
}

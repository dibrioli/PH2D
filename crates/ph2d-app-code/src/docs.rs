//! ⭐ **`CodeDocs` — a família instalada no hero**: os documentos abertos (um por script, o id da aba
//! é o `ScriptId`), os que se fecharam com alterações por salvar, e o console de cada script.
//!
//! # A aba fechada com alterações por salvar
//!
//! ⛔ **Nunca se descarta texto em silêncio.** Fechar uma aba com ● guarda o documento INTEIRO (o
//! texto, o cursor, o histórico de desfazer) em [`CodeDocs::parked`]; o «Edit script» seguinte desse
//! script reabre-o tal como estava, ainda com ●; o File ▸ Save grava-o. Abrir outro projeto esquece
//! tudo o que é por `ScriptId` e guarda os textos por gravar em ficheiros de recuperação
//! ([`crate::recovery`]) — nunca em silêncio.

use crate::console::Store;
use crate::doc::CodeDoc;
use ph2d_code_intel::GrammarWorker;
use ph2d_editor_core::ToastQueue;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_code::{Badge, FindHit, ScriptListView, count_text};
use ph2d_script::{LibraryEpoch, ScriptId};
use std::collections::BTreeMap;

/// O que um Ctrl+S pediu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SaveKind {
    Save,
    SaveAs,
}

/// O que um documento estacionado segura (a sonda dos testes).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParkedFootprint {
    pub text_bytes: usize,
    pub undo_steps: usize,
    /// Layouts de linha na cache da vista.
    pub layouts: usize,
    /// Segura a árvore da gramática.
    pub tree: bool,
}

/// O id do nó `Document` da aba `id` (os `TextRun` vêm a seguir, um por linha).
pub(crate) const A11Y_SALT: u64 = 0xC0DE_0000_0000_0000;

pub struct CodeDocs {
    pub(crate) docs: Vec<CodeDoc>,
    pub(crate) parked: BTreeMap<ScriptId, Parked>,
    pub(crate) consoles: BTreeMap<ScriptId, Store>,
    pub(crate) focus: Option<u64>,
    /// A área da aba no último quadro (o rato pergunta por ela).
    pub(crate) area: Rect,
    /// E11: a lista de scripts do projeto, à esquerda.
    pub(crate) list: crate::list::ListState,
    /// O relógio do chrome (ns) e o «movimento reduzido» — o cursor pisca por eles.
    pub(crate) clock: (u128, bool),
    /// A biblioteca que as abas seguem (outra = outro projecto).
    epoch: Option<LibraryEpoch>,
    /// A aba activa no último quadro (perdê-la larga a composição e o arrasto).
    pub(crate) active: Option<u64>,
    /// Um carregar foi tomado e ainda não largou.
    pub(crate) pressed: bool,
    /// `None` = a pasta do sistema ([`crate::recovery::default_dir`]).
    recovery_dir: Option<std::path::PathBuf>,
    /// As explicações que os balões abriram (I8, `crate::note`).
    pub(crate) seen: crate::note::Seen,
    /// O reparse fora da tecla (Onda 2, I1); a thread nasce com o 1.º trabalho.
    grammar: Option<GrammarWorker<ScriptId>>,
    /// O assistente de código (Onda 2, I2); arranca no 1.º documento aberto.
    pub(crate) intel: crate::intel::Intel,
    /// Os passos ligados à fila global do undo (I10, [`crate::link`]).
    pub(crate) links: crate::link::Links,
    /// O formatador ao salvar (I13, [`crate::format`]).
    pub(crate) format: crate::format::Format,
    /// As páginas de ajuda abertas ([`crate::help`]) e o histórico da família ([`crate::nav`]).
    pub(crate) help: crate::help::HelpTabs,
    pub(crate) nav: crate::nav::Nav,
    /// O texto que um clique copiou (o «Copy» de um exemplo), à espera do quadro.
    pub(crate) copy_out: Option<String>,
    /// Os nomes da cena lidos no último quadro (item 8); as abas partilham-nos.
    pub(crate) scene: crate::scene_names::Shared,
    /// Scripts cujo texto da biblioteca mudou fora do quadro (o renomear de um objecto): a cena
    /// recarrega-os no quadro seguinte, que tem a VM.
    pub(crate) reload: Vec<ScriptId>,
}

/// O que a gramática de uma aba fez (a sonda do gate da tecla).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GrammarProbe {
    /// Reparses feitos NA hora (o Enter, o recuo do `end`, uma dobra) — nunca por uma letra.
    pub sync_reparses: u64,
    /// A árvore ainda não tem as últimas edições.
    pub stale: bool,
}

/// O meio-período do piscar do cursor: o token `Slow` (o do piscar do F4, 1,25 Hz — abaixo dos 3 Hz
/// da WCAG 2.3.1). Aceso um período, apagado outro; uma tecla recomeça-o aceso.
pub(crate) fn blink_on(now_ns: u128, last_input_ns: u128, steady: bool) -> bool {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let half = (f64::from(ph2d_tokens::Duration::Slow.ms()) * 1e6) as u128;
    steady || (now_ns.saturating_sub(last_input_ns) / half.max(1)).is_multiple_of(2)
}

impl Default for CodeDocs {
    fn default() -> Self {
        Self {
            docs: Vec::new(),
            parked: BTreeMap::new(),
            consoles: BTreeMap::new(),
            focus: None,
            area: Rect::new(0.0, 0.0, 0.0, 0.0),
            list: crate::list::ListState::default(),
            clock: (0, false),
            epoch: None,
            active: None,
            pressed: false,
            recovery_dir: None,
            grammar: None,
            intel: crate::intel::Intel::default(),
            links: crate::link::Links::default(),
            format: crate::format::Format::default(),
            seen: crate::note::Seen::default(),
            help: crate::help::HelpTabs::default(),
            nav: crate::nav::Nav::default(),
            copy_out: None,
            scene: crate::scene_names::Shared::default(),
            reload: Vec::new(),
        }
    }
}

/// ⭐ **Um documento fechado com alterações por salvar** — só o texto (com a seleção e o histórico de
/// desfazer, no `Buffer`) e o que a biblioteca tinha; a árvore da gramática, a cache de layouts, a
/// busca e a vista largam-se ao fechar e refazem-se ao reabrir ([`CodeDoc::unpark`]).
pub(crate) struct Parked {
    pub serial: u64,
    pub title: String,
    pub buffer: ph2d_code_buffer::Buffer,
    pub synced: String,
}

impl CodeDocs {
    /// Os textos abertos e os estacionados: `(script, título, buffer)`.
    pub(crate) fn buffers(
        &self,
    ) -> impl Iterator<Item = (ScriptId, &str, &ph2d_code_buffer::Buffer)> {
        let open = self
            .docs
            .iter()
            .map(|d| (d.script, d.title.as_str(), &d.buffer));
        open.chain(
            self.parked
                .iter()
                .map(|(id, p)| (*id, p.title.as_str(), &p.buffer)),
        )
    }

    pub(crate) fn doc(&mut self, id: u64) -> Option<&mut CodeDoc> {
        self.docs.iter_mut().find(|d| d.script.0 == id)
    }

    /// A aba `id` com uma tecla, um clique ou um comando agora: o cursor fica aceso.
    pub(crate) fn touched(&mut self, id: u64) -> Option<&mut CodeDoc> {
        let now = self.clock.0;
        let d = self.doc(id)?;
        d.last_input_ns = now;
        Some(d)
    }

    /// Que assistente de código a família liga (por omissão o `luau-lsp` fixado). Antes do 1.º
    /// documento aberto — depois disso a sessão já arrancou.
    pub fn set_intel_server(&mut self, server: crate::IntelServer) {
        self.intel = crate::intel::Intel::new(server);
    }

    /// Envia ao assistente o que mudou nos scripts desde o último quadro (e arranca-o no 1.º
    /// documento aberto).
    pub(crate) fn sync_intel(&mut self, lib: &ph2d_script::ScriptLibrary) {
        self.intel
            .sync(&mut self.docs, &self.parked, lib, self.clock.0);
    }

    /// A pasta dos ficheiros de recuperação (os textos por gravar de um projecto que se trocou).
    pub fn set_recovery_dir(&mut self, dir: std::path::PathBuf) {
        self.recovery_dir = Some(dir);
    }

    /// A aba activa deste quadro (`None` = outra família, um quadro ou a Cena): nenhuma outra aba
    /// segura uma composição ou um arrasto (a sessão ao vivo pode chamar para um quadro a meio).
    pub(crate) fn follow_active(&mut self, active: Option<u64>) {
        for d in self.docs.iter_mut().filter(|d| Some(d.script.0) != active) {
            d.preedit = None;
            d.drag = None;
        }
        if self.active != active {
            self.pressed = false;
            self.active = active;
        }
    }

    /// ⛔ **Outra biblioteca = outro projecto** (os `ScriptId` recomeçam em 1): esquece as abas, os
    /// estacionados, os consoles, os selos e a lista; os textos por gravar viram ficheiros de
    /// recuperação ([`crate::recovery`]) e o aviso vai para `toasts`. `true` = trocou.
    pub(crate) fn follow_epoch(&mut self, epoch: LibraryEpoch, toasts: &mut ToastQueue) -> bool {
        if self.epoch.replace(epoch).is_none_or(|e| e == epoch) {
            return false;
        }
        let unsaved: Vec<(String, String)> = self
            .buffers()
            .filter(|d| d.2.is_dirty())
            .map(|d| (d.1.to_owned(), d.2.text().to_string()))
            .collect();
        self.docs.clear();
        self.parked.clear();
        self.consoles.clear();
        self.nav = crate::nav::Nav::default();
        self.focus = None;
        self.list.forget_project();
        if !unsaved.is_empty() {
            let dir = self
                .recovery_dir
                .clone()
                .or_else(crate::recovery::default_dir);
            toasts.push(crate::recovery::keep(dir.as_deref(), &unsaved));
        }
        true
    }

    /// O relógio do chrome (`hero.store.clock_ns()`) e o «movimento reduzido» (cursor fixo).
    pub fn set_clock(&mut self, now_ns: u128, steady: bool) {
        self.clock = (now_ns, steady);
    }

    /// O cursor da aba `id` está aceso agora?
    #[must_use]
    pub fn caret_lit(&self, id: ScriptId) -> bool {
        self.docs
            .iter()
            .find(|d| d.script == id)
            .is_some_and(|d| blink_on(self.clock.0, d.last_input_ns, self.clock.1))
    }

    /// As áreas da lista e do documento dentro da aba.
    pub(crate) fn split_area(&self, area: Rect) -> (Rect, Rect) {
        let w = ScriptListView::width(self.list.open).min(area.w);
        (
            Rect::new(area.x, area.y, w, area.h),
            Rect::new(area.x + w, area.y, (area.w - w).max(0.0), area.h),
        )
    }

    /// Abre (ou activa) o script `id`: um documento aberto ganha o foco; um estacionado volta como
    /// estava; senão nasce do texto da biblioteca.
    pub fn open(&mut self, id: ScriptId, name: &str, text: &str) {
        if !self.docs.iter().any(|d| d.script == id) {
            let doc = match self.parked.remove(&id) {
                Some(p) => CodeDoc::unpark(id, p),
                None => CodeDoc::new(id, name, text),
            };
            self.docs.push(doc);
        }
        self.focus = Some(id.0);
    }

    /// Os scripts com aba aberta.
    #[must_use]
    pub fn open_scripts(&self) -> Vec<ScriptId> {
        self.docs.iter().map(|d| d.script).collect()
    }

    /// O texto da aba do script `id` (aberta).
    #[must_use]
    pub fn text_of(&self, id: ScriptId) -> Option<String> {
        self.docs
            .iter()
            .find(|d| d.script == id)
            .map(|d| d.buffer.text().to_string())
    }

    /// O cursor primário do script `id` (aberto): `(linha, byte)`, 0-based.
    #[must_use]
    pub fn cursor_of(&self, id: ScriptId) -> Option<(usize, usize)> {
        let d = self.docs.iter().find(|d| d.script == id)?;
        let h = d.buffer.selection().primary().head;
        Some((d.buffer.text().line_of_byte(h), h))
    }

    /// As dobras fechadas do script `id`: `(cabeçalho, última escondida)`, linhas 0-based.
    #[must_use]
    pub fn closed_folds(&self, id: ScriptId) -> Vec<(usize, usize)> {
        self.docs
            .iter()
            .find(|d| d.script == id)
            .map(CodeDoc::closed_folds)
            .unwrap_or_default()
    }

    /// As linhas que a vista do script `id` mostrou no último quadro.
    #[must_use]
    pub fn visible_lines(&self, id: ScriptId) -> Option<std::ops::Range<usize>> {
        self.docs
            .iter()
            .find(|d| d.script == id)
            .map(|d| d.view.visible_lines())
    }

    /// A rolagem `(x, y)` da vista do script `id`, em píxeis.
    #[must_use]
    pub fn scroll_of(&self, id: ScriptId) -> Option<(f32, f32)> {
        self.docs
            .iter()
            .find(|d| d.script == id)
            .map(|d| d.view.scroll())
    }

    /// A composição em curso na aba do script `id` (a pré-edição do IME ou a tecla morta).
    #[must_use]
    pub fn composing(&self, id: ScriptId) -> Option<String> {
        self.docs
            .iter()
            .find(|d| d.script == id)
            .and_then(|d| d.preedit.as_ref().map(|p| p.0.clone()))
    }

    /// ⚠️ SONDA: o que o documento estacionado de `id` segura.
    #[must_use]
    pub fn parked_footprint(&self, id: ScriptId) -> Option<ParkedFootprint> {
        self.parked.get(&id).map(|p| ParkedFootprint {
            text_bytes: p.buffer.text().len(),
            undo_steps: p.buffer.history().undo_len(),
            layouts: 0,
            tree: false,
        })
    }

    /// O documento estacionado de `id` (fechado com ●), se o há.
    #[must_use]
    pub fn is_parked(&self, id: ScriptId) -> bool {
        self.parked.contains_key(&id)
    }

    /// O selo da barra de estado do script `id` (aberto).
    #[must_use]
    pub fn badge_of(&self, id: ScriptId) -> Option<Badge> {
        self.docs
            .iter()
            .find(|d| d.script == id)
            .and_then(|d| d.badge.clone())
    }

    /// O texto do selo que a barra de estado do script `id` (aberto) pinta.
    #[must_use]
    pub fn badge_text_of(&self, id: ScriptId) -> Option<String> {
        let d = self.docs.iter().find(|d| d.script == id)?;
        ph2d_panel_code::badge_line(d.badge.as_ref(), d.project_unsaved).map(|(t, _)| t)
    }

    /// Há alterações por salvar na aba (aberta ou estacionada) do script `id`?
    #[must_use]
    pub fn is_dirty(&self, id: ScriptId) -> bool {
        self.buffers().any(|d| d.0 == id && d.2.is_dirty())
    }

    /// Onde a barra de busca pintou `hit` no último quadro (os testes clicam aí).
    #[must_use]
    pub fn find_target(&self, id: ScriptId, hit: FindHit) -> Option<Rect> {
        self.docs
            .iter()
            .find(|d| d.script == id)
            .and_then(|d| d.find_view.target_rect(hit))
    }

    /// O texto da contagem da busca que a barra mostra («3 of 12» / «No results»).
    #[must_use]
    pub fn find_count_text(&mut self, id: ScriptId) -> Option<String> {
        let d = self.doc(id.0).filter(|d| d.find.open)?;
        d.find.refresh(&d.buffer);
        count_text(d.find.count())
    }

    /// Envia ao worker o reparse de cada aba cuja árvore velha ainda não está toda num trabalho (o
    /// novo substitui o que estava em voo).
    pub(crate) fn submit(&mut self) {
        for d in &mut self.docs {
            if let Some(job) = d.syntax.job(d.buffer.text()) {
                self.grammar
                    .get_or_insert_with(GrammarWorker::spawn)
                    .submit(d.script, job);
            }
        }
    }

    /// Traz as árvores que o worker acabou (nunca espera) e envia as que ainda faltam. Corre em todo
    /// quadro da família e antes de pintar: o realce alcança a tecla no quadro seguinte.
    pub fn pump(&mut self) {
        if let Some(g) = &mut self.grammar {
            let mut retired = Vec::new();
            for (id, r) in g.drain() {
                // Uma aba fechada (ou recriada) deixa a resposta sem dono: larga-se no worker.
                match self.docs.iter_mut().find(|d| d.script == id) {
                    Some(d) => {
                        d.syntax.adopt_into(r, &mut retired);
                    }
                    None => drop(r),
                }
            }
            g.retire(retired);
        }
        self.submit();
    }

    /// ⚠️ SONDA: espera que o worker acabe o que tem em mãos, SEM o trazer (o próximo quadro traz).
    pub fn wait_grammar(&mut self) {
        if let Some(g) = &mut self.grammar {
            g.wait_all();
        }
    }

    pub(crate) fn split(&mut self, id: u64) -> Option<(&mut CodeDoc, &mut Store)> {
        let doc = self.docs.iter_mut().find(|d| d.script.0 == id)?;
        let store = self.consoles.entry(doc.script).or_default();
        Some((doc, store))
    }
}

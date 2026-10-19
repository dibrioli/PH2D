//! ⭐ **E11 — a lista de scripts do projeto** (a pintura é a `ph2d_panel_code::ScriptListView`). O
//! rato e o teclado da lista viram PEDIDOS ([`Ask`]); o [`crate::frame`] aplica-os à `ScriptLibrary`
//! no quadro seguinte — é ele que vê o mundo (e por isso é ele que recusa apagar um script em uso).

use crate::docs::CodeDocs;
use crate::edit::{FieldKey, field_key};
use ph2d_code_buffer::{Buffer, Range, Selection};
use ph2d_ecs::World;
use ph2d_editor_core::doc_surface::KeyOutcome;
use ph2d_editor_core::interaction::DOUBLE_CLICK_WINDOW_NS;
use ph2d_i18n::tr_with;
use ph2d_panel_code::{ListField, ListHit, ListRow, ScriptList, ScriptListView};
use ph2d_script::{LuauScript, ProjectScripts, ScriptId, ScriptSource};

/// O texto de um script novo: os dois ganchos que todo script do projeto tem. Cada corpo leva uma
/// linha: o StyLua fixado dobra uma função VAZIA numa linha só ao salvar (I13, `formatar.rs`).
pub const NEW_SCRIPT_TEXT: &str = "function init(self)\n  -- runs once, when the object starts\nend\n\nfunction update(self, dt)\n  -- runs every frame; dt = seconds since the last one\nend\n";
/// O nome pedido para um script novo (a biblioteca torna-o único: «script 2.luau»…).
pub const NEW_SCRIPT_NAME: &str = "script.luau";

/// Um pedido da lista, aplicado pelo `frame`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ask {
    Open(ScriptId),
    New,
    Rename(ScriptId, String),
    Delete(ScriptId),
}

pub(crate) struct ListState {
    pub open: bool,
    pub filter: Buffer,
    pub focus: Option<ListField>,
    pub renaming: Option<(ScriptId, Buffer)>,
    pub notice: Option<String>,
    /// Os scripts do projeto, pela ordem da biblioteca (o `frame` renova-os).
    pub entries: Vec<(ScriptId, String)>,
    pub asks: Vec<Ask>,
    pub view: ScriptListView,
    last_click: Option<(u128, u64)>,
}

impl Default for ListState {
    fn default() -> Self {
        Self {
            open: true,
            filter: Buffer::default(),
            focus: None,
            renaming: None,
            notice: None,
            entries: Vec::new(),
            asks: Vec::new(),
            view: ScriptListView::default(),
            last_click: None,
        }
    }
}

impl ListState {
    /// Outro projecto: os pedidos, o renomear e o clique nomeiam ids do anterior. Fica o filtro e
    /// se a lista está aberta (preferências do artista, não do projecto).
    pub(crate) fn forget_project(&mut self) {
        let (open, filter) = (self.open, std::mem::take(&mut self.filter));
        *self = Self {
            open,
            filter,
            ..Self::default()
        };
    }

    /// As linhas que passam o filtro (sem maiúsculas).
    fn visible(&self) -> Vec<(ScriptId, &str)> {
        let f = self.filter.text().to_string().to_lowercase();
        self.entries
            .iter()
            .filter(|(_, n)| f.is_empty() || n.to_lowercase().contains(&f))
            .map(|(id, n)| (*id, n.as_str()))
            .collect()
    }

    /// Pinta a lista em `area`; `current` = o script da aba activa, `dirty` = quem tem ●.
    pub fn paint(
        &mut self,
        area: ph2d_editor_core::zones::Rect,
        current: u64,
        dirty: &dyn Fn(ScriptId) -> bool,
        scene: &mut ph2d_vector::VectorScene,
        ts: &mut ph2d_text::TextSystem,
        theme: &ph2d_tokens::Theme,
    ) {
        let mut view = std::mem::take(&mut self.view);
        let rows: Vec<ListRow<'_>> = self
            .visible()
            .into_iter()
            .map(|(id, name)| ListRow {
                id: id.0,
                name,
                dirty: dirty(id),
                current: id.0 == current,
            })
            .collect();
        let list = ScriptList {
            open: self.open,
            filter: &self.filter,
            focus: self.focus,
            rows: &rows,
            renaming: self.renaming.as_ref().map(|(id, b)| (id.0, b)),
            notice: self.notice.as_deref(),
        };
        view.paint(&list, area, scene, ts, theme);
        self.view = view;
    }

    /// Um clique na lista (`true` = tomado).
    pub fn press(&mut self, h: ListHit, now_ns: u128) {
        self.notice = None;
        if !matches!(h, ListHit::Field(ListField::Rename, _)) {
            self.commit_rename();
        }
        match h {
            ListHit::Toggle => {
                self.open = !self.open;
                self.focus = None;
            }
            ListHit::New => self.asks.push(Ask::New),
            ListHit::Field(f, byte) => {
                self.focus = Some(f);
                let buf = match f {
                    ListField::Filter => &mut self.filter,
                    ListField::Rename => match self.renaming.as_mut() {
                        Some((_, b)) => b,
                        None => return,
                    },
                };
                buf.set_selection(Selection::single(Range::cursor(byte)));
            }
            ListHit::Row(id) => {
                self.focus = None;
                let double = self.last_click.is_some_and(|(t, at)| {
                    at == id && now_ns.saturating_sub(t) <= DOUBLE_CLICK_WINDOW_NS
                });
                self.last_click = Some((now_ns, id));
                if double {
                    self.start_rename(ScriptId(id));
                } else {
                    self.asks.push(Ask::Open(ScriptId(id)));
                }
            }
            ListHit::Delete(id) => self.asks.push(Ask::Delete(ScriptId(id))),
            ListHit::Body => self.focus = None,
        }
    }

    /// Começa a renomear `id`: o campo nasce com o nome inteiro seleccionado.
    pub fn start_rename(&mut self, id: ScriptId) {
        let Some((_, name)) = self.entries.iter().find(|(i, _)| *i == id) else {
            return;
        };
        let mut b = Buffer::new(name);
        b.select_all();
        self.renaming = Some((id, b));
        self.focus = Some(ListField::Rename);
        self.last_click = None;
    }

    fn commit_rename(&mut self) {
        if let Some((id, b)) = self.renaming.take() {
            let name = b.text().to_string();
            if !name.trim().is_empty() {
                self.asks.push(Ask::Rename(id, name.trim().to_owned()));
            }
        }
        if self.focus == Some(ListField::Rename) {
            self.focus = None;
        }
    }

    /// Uma tecla com um campo da lista focado.
    pub fn key(&mut self, k: FieldKey<'_>) -> KeyOutcome {
        let mut out = KeyOutcome {
            consumed: true,
            copy: None,
        };
        match self.focus {
            Some(ListField::Filter) => {
                let r = field_key(&mut self.filter, k);
                out.copy = r.copy;
                if r.escape || r.enter || r.tab {
                    self.focus = None;
                }
                if r.enter
                    && let Some((id, _)) = self.visible().first()
                {
                    self.asks.push(Ask::Open(*id));
                }
            }
            Some(ListField::Rename) => {
                let Some((_, b)) = self.renaming.as_mut() else {
                    self.focus = None;
                    return out;
                };
                let r = field_key(b, k);
                out.copy = r.copy;
                if r.escape {
                    self.renaming = None;
                    self.focus = None;
                } else if r.enter || r.tab {
                    self.commit_rename();
                }
            }
            None => out.consumed = false,
        }
        out
    }
}

/// Quantos objectos do mundo correm o script `id` (a verdade é o documento, não a VM).
#[must_use]
pub fn users_of(world: &mut World, id: ScriptId) -> usize {
    let mut q = world.query::<&LuauScript>();
    q.iter(world)
        .filter(|c| c.source == ScriptSource::Library(id))
        .count()
}

impl CodeDocs {
    /// Os pedidos da lista contra a biblioteca do mundo, e as entradas renovadas.
    pub(crate) fn apply_list(&mut self, world: &mut World) {
        let asks = std::mem::take(&mut self.list.asks);
        for ask in asks {
            match ask {
                Ask::Open(id) => {
                    if let Some(d) = world
                        .get_resource::<ProjectScripts>()
                        .and_then(|l| l.get(id))
                    {
                        let (name, text) = (d.name.clone(), d.text.clone());
                        self.open(id, &name, &text);
                    }
                }
                Ask::New => {
                    let id = world
                        .get_resource_or_init::<ProjectScripts>()
                        .insert(NEW_SCRIPT_NAME, NEW_SCRIPT_TEXT);
                    let name = world
                        .resource::<ProjectScripts>()
                        .get(id)
                        .map(|d| d.name.clone());
                    self.open(
                        id,
                        name.as_deref().unwrap_or(NEW_SCRIPT_NAME),
                        NEW_SCRIPT_TEXT,
                    );
                }
                Ask::Rename(id, name) => {
                    if let Some(mut l) = world.get_resource_mut::<ProjectScripts>() {
                        l.rename(id, &name);
                    }
                }
                Ask::Delete(id) => {
                    let users = users_of(world, id);
                    let Some(mut l) = world.get_resource_mut::<ProjectScripts>() else {
                        continue;
                    };
                    let name = l.get(id).map(|d| d.name.clone()).unwrap_or_default();
                    if users > 0 {
                        self.list.notice = Some(tr_with(
                            "code.list.in_use",
                            &[("name", &name), ("count", &users)],
                        ));
                    } else {
                        l.remove(id);
                    }
                }
            }
        }
        self.list.entries = world
            .get_resource::<ProjectScripts>()
            .map(|l| l.iter().map(|(id, d)| (id, d.name.clone())).collect())
            .unwrap_or_default();
    }

    /// O que a lista mostra — os testes perguntam por aqui.
    #[must_use]
    pub fn list_names(&self) -> Vec<String> {
        self.list
            .visible()
            .into_iter()
            .map(|(_, n)| n.to_owned())
            .collect()
    }

    /// A frase de recusa que a lista mostra, se a há.
    #[must_use]
    pub fn list_notice(&self) -> Option<&str> {
        self.list.notice.as_deref()
    }

    /// A lista está aberta?
    #[must_use]
    pub fn list_open(&self) -> bool {
        self.list.open
    }

    /// Onde a lista pintou `hit` no último quadro.
    #[must_use]
    pub fn list_target(&self, hit: ListHit) -> Option<ph2d_editor_core::zones::Rect> {
        self.list.view.target_rect(hit)
    }
}

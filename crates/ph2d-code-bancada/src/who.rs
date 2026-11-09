//! ⭐ **Quem corre o script aberto, e com que valores** (plano 13 Onda B, itens 6–8): a grelha «Who
//! has what», as marcas da margem e o «In this script» saem daqui, numa leitura por quadro.
//!
//! ⚠️ Os objectos são os MESMOS e pela MESMA ordem que o motor conta (`ph2d-script` `scene.rs`
//! `actors`: todo `LuauScript` do script, pela `StableId`): a contagem «2» da lista de scripts e as
//! linhas da grelha nunca discordam, e a cor de cada um (o índice nessa ordem, D2) sobrevive a
//! fechar e abrir o projecto.

use crate::scene::{SceneOutlines, ScenePeek};
use ph2d_code_buffer::Text;
use ph2d_code_syntax::{SymbolKind, Syntax};
use ph2d_code_viewmodel::{Cell, CellKind, Column, Entry, Holder, Who};
use ph2d_ecs::{Entity, StableId, World};
use ph2d_script::props::{PropDecl, resolve};
use ph2d_script::{LuauScript, ScriptHost, ScriptId, ScriptSource, ScriptValue};

/// O que o TEXTO diz (as propriedades, as funções, os sinais ouvidos): muda com a árvore, não com a
/// cena — lê-se uma vez por geração da árvore.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TextFacts {
    pub props: Vec<Entry>,
    pub functions: Vec<Entry>,
    pub hears: Vec<Entry>,
}

impl TextFacts {
    #[must_use]
    pub fn read(syntax: &Syntax, text: &Text) -> Self {
        let line = |b: usize| text.line_of_byte(b);
        Self {
            props: syntax
                .property_declarations(text)
                .into_iter()
                .map(|(n, r)| Entry(n, line(r.start)))
                .collect(),
            functions: syntax
                .symbols(text)
                .into_iter()
                .filter(|s| s.kind == SymbolKind::Function)
                .map(|s| Entry(s.name, line(s.byte)))
                .collect(),
            hears: syntax
                .heard_signals(text)
                .into_iter()
                .map(|(n, r)| Entry(n, line(r.start)))
                .collect(),
        }
    }
}

/// O estado da Bancada da aba activa: os factos do texto (guardados pela geração da árvore) e o
/// [`Who`] do último quadro.
#[derive(Debug, Default)]
pub struct Bancada {
    key: Option<(ScriptId, u64)>,
    facts: TextFacts,
    pub who: Who,
}

/// O script da aba activa: o id, a árvore e o texto.
pub struct Open<'a> {
    pub script: ScriptId,
    pub syntax: &'a Syntax,
    pub text: &'a Text,
}

impl Bancada {
    /// Um quadro. `map` = o mapa está ligado (pede os contornos à shell); `peek` = o objecto (índice)
    /// que o rato espia agora — a cena aparece no lugar da aba com ele contornado.
    pub fn follow(
        &mut self,
        world: &mut World,
        host: Option<&ScriptHost>,
        open: Option<Open<'_>>,
        (map, peek): (bool, Option<usize>),
    ) {
        let Some(o) = open else {
            self.who = Who::default();
            world.insert_resource(ScenePeek(None));
            world.insert_resource(SceneOutlines::default());
            return;
        };
        let key = (o.script, o.syntax.generation());
        if self
            .key
            .is_none_or(|k| k.0 != o.script || (k != key && !o.syntax.is_stale()))
        {
            self.key = Some(key);
            self.facts = TextFacts::read(o.syntax, o.text);
        }
        let decls = ph2d_script::decls_shown(world, host, o.script).map(<[PropDecl]>::to_vec);
        let mut who = read(world, o.script, decls.as_deref(), &self.facts);
        let mut outlines = world.get_resource_or_init::<SceneOutlines>();
        outlines.wanted = if map {
            who.holders.iter().map(|h| h.bits).collect()
        } else {
            Vec::new()
        };
        who.map = outlines
            .got
            .iter()
            .filter(|_| map)
            .filter_map(|(bits, rings)| {
                let h = who.holders.iter().position(|h| h.bits == *bits)?;
                Some((h, rings.clone()))
            })
            .collect();
        let peeked = peek
            .and_then(|h| who.holders.get(h))
            .map(|h| (h.bits, h.color));
        world.insert_resource(ScenePeek(peeked));
        self.who = who;
    }
}

/// O [`Who`] do script `id`: os objectos (pela ordem do motor), as colunas do texto com o tipo e o
/// valor das declarações que o painel mostra (`decls`), e as células.
#[must_use]
pub fn read(world: &mut World, id: ScriptId, decls: Option<&[PropDecl]>, facts: &TextFacts) -> Who {
    let mut rows: Vec<(u64, Entity, LuauScript)> = world
        .query::<(Entity, &LuauScript, Option<&StableId>)>()
        .iter(world)
        .filter(|(_, s, _)| s.source == ScriptSource::Library(id))
        .map(|(e, s, sid)| (sid.map_or(u64::MAX, |s| s.0), e, s.clone()))
        .collect();
    rows.sort_by_key(|(sid, e, _)| (*sid, e.to_bits()));
    let decl = |name: &str| decls.and_then(|d| d.iter().find(|d| d.name == name));
    let columns: Vec<Column> = facts
        .props
        .iter()
        .map(|Entry(name, line)| {
            let d = decl(name);
            Column {
                name: name.clone(),
                line: *line,
                kind: d.map(kind_of),
                default: d.map(|d| text_of(&d.default)).unwrap_or_default(),
                options: d.map(|d| d.hint.options.clone()).unwrap_or_default(),
            }
        })
        .collect();
    let mut cells = Vec::with_capacity(rows.len() * columns.len());
    for (_, _, s) in &rows {
        let r = resolve(decls, &s.own);
        for c in &columns {
            let v = r.values.iter().find(|v| v.name == c.name);
            let default = decl(&c.name).map(|d| &d.default);
            cells.push(match v {
                Some(v) => Cell {
                    text: text_of(&v.value),
                    changed: default.is_some_and(|d| *d != v.value),
                    swatch: swatch_of(&v.value),
                },
                None => Cell {
                    text: String::new(),
                    changed: false,
                    swatch: None,
                },
            });
        }
    }
    let holders = rows
        .iter()
        .enumerate()
        .map(|(i, (_, e, _))| Holder {
            bits: e.to_bits(),
            name: crate::names::object_name(world, *e),
            color: i,
        })
        .collect();
    Who {
        holders,
        columns,
        cells,
        functions: facts.functions.clone(),
        hears: facts.hears.clone(),
        map: Vec::new(),
    }
}

fn kind_of(d: &PropDecl) -> CellKind {
    match &d.default {
        ScriptValue::Number(_) => CellKind::Number,
        ScriptValue::Bool(_) => CellKind::Bool,
        ScriptValue::Text(_) if !d.hint.options.is_empty() => CellKind::Choice,
        ScriptValue::Text(_) => CellKind::Text,
        ScriptValue::Vec2(_) => CellKind::Vec2,
        ScriptValue::Color(_) => CellKind::Color,
        ScriptValue::Object(_) => CellKind::Object,
    }
}

/// Um número como o campo o escreve, sem os zeros do fim (`220`, `0.5`).
#[must_use]
pub fn number_text(v: f64) -> String {
    let s = ph2d_editor_core::interaction::format_number(v);
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_owned()
    } else {
        s
    }
}

/// Um valor escrito como a célula o mostra (a cor é a amostra, sem texto).
#[must_use]
pub fn text_of(v: &ScriptValue) -> String {
    match v {
        ScriptValue::Number(n) => number_text(*n),
        ScriptValue::Bool(b) => b.to_string(),
        ScriptValue::Text(t) | ScriptValue::Object(t) => t.clone(),
        ScriptValue::Vec2([x, y]) => format!("{}, {}", number_text(*x), number_text(*y)),
        ScriptValue::Color(_) => String::new(),
    }
}

#[allow(clippy::cast_possible_truncation)]
fn swatch_of(v: &ScriptValue) -> Option<[f32; 4]> {
    match v {
        ScriptValue::Color(c) => Some(c.map(|x| x as f32)),
        _ => None,
    }
}

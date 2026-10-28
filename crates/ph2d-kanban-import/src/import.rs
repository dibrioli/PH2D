//! **Trello JSON → [`Kanban`].** Lê o ficheiro de «Export as JSON» de um quadro sem nunca entrar em
//! pânico: o que não serve vai para o [`ImportReport`], nunca para um erro (erro é só «isto não é um
//! quadro»).
//!
//! Os ids locais dão-se por uma ordem FIXA (etiquetas, listas, campos e opções, cartões, checklists,
//! itens, anexos, ações da mais velha para a mais nova) e, em cada tabela, pela ordem do ficheiro: é
//! o que faz a ida-e-volta Trello → Kanban → Trello → Kanban dar o MESMO Kanban. A ordem visível
//! vem do `pos` (float) e vira [`FracKey`] seguidas por grupo, sem renumerar ninguém.

use std::collections::{BTreeMap, BTreeSet};

use ph2d_kanban_model::{
    Background, Card, CardId, CheckItem, CheckItemId, Checklist, ChecklistId, Due, FieldDef,
    FieldId, FieldKind, FieldOption, FieldValue, FileId, FracKey, Header, Hue, Kanban, KanbanId,
    KanbanOp, Label, LabelId, List, ListId, Member, Meta, Moment, OptionId, Record, apply_batch,
};
use serde_json::Value;

use crate::files::FileSource;
use crate::report::{ImportError, ImportReport, LossKind};
use crate::trello::{GRADIENTS, hue_of, id_seconds, label_color_of, parse_time};

/// O resultado de uma importação: o Kanban, os bytes dos ficheiros que ele usa (para o
/// `KanbanSet::put_file`) e o relatório.
#[derive(Clone, Debug)]
pub struct Imported {
    pub kanban: Kanban,
    pub files: Vec<Vec<u8>>,
    pub report: ImportReport,
}

pub(crate) fn s<'a>(v: &'a Value, k: &str) -> Option<&'a str> {
    v.get(k).and_then(Value::as_str)
}

pub(crate) fn arr<'a>(v: &'a Value, k: &str) -> &'a [Value] {
    v.get(k)
        .and_then(Value::as_array)
        .map_or(&[], Vec::as_slice)
}

pub(crate) fn flag(v: &Value, k: &str) -> bool {
    v.get(k).and_then(Value::as_bool).unwrap_or(false)
}

/// O `pos` do Trello: um número; `"top"`/`"bottom"` (a forma da API de escrita) vão às pontas.
pub(crate) fn pos(v: &Value) -> f64 {
    match v.get("pos") {
        Some(Value::Number(n)) => n.as_f64().unwrap_or(f64::INFINITY),
        Some(Value::String(t)) if t == "top" => f64::NEG_INFINITY,
        _ => f64::INFINITY,
    }
}

/// Chaves seguidas pela ordem do `pos` (o empate fica pela ordem do ficheiro).
pub(crate) fn rank<T>(mut v: Vec<(f64, T)>) -> Vec<(FracKey, T)> {
    v.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut last: Option<FracKey> = None;
    v.into_iter()
        .map(|(_, t)| {
            let k = FracKey::between(last.as_ref(), None);
            last = Some(k.clone());
            (k, t)
        })
        .collect()
}

fn non_empty(v: Option<&Value>) -> bool {
    match v {
        Some(Value::Object(m)) => !m.is_empty(),
        Some(Value::Array(a)) => !a.is_empty(),
        _ => false,
    }
}

/// O estado de uma importação a meio.
pub(crate) struct Ctx<'f> {
    ids: BTreeMap<String, u64>,
    next: u64,
    pub(crate) report: ImportReport,
    /// id de membro do Trello → a pessoa (o `username`, o mesmo das `@menções`).
    pub(crate) members: BTreeMap<String, Member>,
    pub(crate) cards: BTreeMap<u64, Card>,
    pub(crate) records: Vec<Record>,
    pub(crate) files: Vec<Vec<u8>>,
    pub(crate) file_ids: BTreeSet<FileId>,
    pub(crate) source: Option<&'f dyn FileSource>,
}

impl Ctx<'_> {
    /// O id local de `tid`, dado agora se ainda não tem (uma referência a algo que já não existe).
    pub(crate) fn alloc(&mut self, tid: &str) -> u64 {
        if let Some(&id) = self.ids.get(tid) {
            return id;
        }
        let id = self.anon();
        self.ids.insert(tid.to_owned(), id);
        id
    }

    /// Um id para um registo NOVO; `None` = o id já foi visto (fica o primeiro).
    pub(crate) fn fresh(&mut self, tid: &str) -> Option<u64> {
        if self.ids.contains_key(tid) {
            self.report.lose(LossKind::DuplicateId, tid);
            return None;
        }
        Some(self.alloc(tid))
    }

    pub(crate) fn anon(&mut self) -> u64 {
        self.next += 1;
        self.next
    }

    pub(crate) fn known(&self, tid: &str) -> Option<u64> {
        self.ids.get(tid).copied()
    }

    pub(crate) fn member(&mut self, tid: Option<&str>, kind: LossKind, at: &str) -> Option<Member> {
        let m = tid.and_then(|t| self.members.get(t).cloned());
        if m.is_none() {
            self.report.lose(kind, at);
        }
        m
    }
}

/// Importa um quadro do Trello como o Kanban `id`. `files` = onde procurar os bytes dos anexos
/// carregados (sem fonte, entram como links).
///
/// # Errors
/// Só quando os bytes não são JSON ou o JSON não é um quadro (nem `lists` nem `cards`).
pub fn import_trello(
    json: &[u8],
    id: KanbanId,
    files: Option<&dyn FileSource>,
) -> Result<Imported, ImportError> {
    let board: Value =
        serde_json::from_slice(json).map_err(|e| ImportError::NotJson(e.to_string()))?;
    if !board.is_object() || (board.get("lists").is_none() && board.get("cards").is_none()) {
        return Err(ImportError::NotABoard);
    }
    let mut cx = Ctx {
        ids: BTreeMap::new(),
        next: 0,
        report: ImportReport::default(),
        members: BTreeMap::new(),
        cards: BTreeMap::new(),
        records: Vec::new(),
        files: Vec::new(),
        file_ids: BTreeSet::new(),
        source: files,
    };
    let header = header(&mut cx, &board);
    members(&mut cx, &board);
    let labels = labels(&mut cx, &board);
    let lists = lists(&mut cx, &board);
    let fields = fields(&mut cx, &board);
    let card_objs = cards(&mut cx, &board, [&labels, &lists], &fields);
    checklists(&mut cx, &board);
    let links = crate::import_files::attachments(&mut cx, &card_objs);
    crate::import_files::covers(&mut cx, &card_objs, &links);
    crate::import_actions::actions(&mut cx, &board);

    let mut kanban = Kanban::new(id, &header.name);
    let mut ops = vec![KanbanOp::Header(header)];
    let cards = std::mem::take(&mut cx.cards);
    ops.extend(cards.into_values().map(|c| KanbanOp::put(Record::card(c))));
    ops.extend(cx.records.drain(..).map(KanbanOp::put));
    let _ = apply_batch(&mut kanban, ops);
    let c = &mut cx.report.counts;
    (c.lists, c.cards, c.labels) = (kanban.lists.len(), kanban.cards.len(), kanban.labels.len());
    (c.checklists, c.items) = (kanban.checklists.len(), kanban.items.len());
    (c.comments, c.attachments) = (kanban.comments.len(), kanban.attachments.len());
    (c.fields, c.activities) = (kanban.fields.len(), kanban.activities.len());
    (c.members, c.files) = (cx.members.len(), cx.files.len());
    Ok(Imported {
        kanban,
        files: cx.files,
        report: cx.report,
    })
}

fn header(cx: &mut Ctx, b: &Value) -> Header {
    let prefs = b.get("prefs").unwrap_or(&Value::Null);
    let bg = s(prefs, "background").unwrap_or("blue");
    let background = if let Some(h) = hue_of(bg) {
        Background::Color(h)
    } else if let Some(i) = GRADIENTS.iter().position(|g| *g == bg) {
        Background::Gradient(i as u16)
    } else {
        cx.report.lose(LossKind::BackgroundImage, bg);
        Background::Color(Hue::Blue)
    };
    if s(b, "desc").is_some_and(|d| !d.is_empty()) {
        cx.report
            .lose(LossKind::BoardDescription, s(b, "name").unwrap_or(""));
    }
    if non_empty(b.get("pluginData")) || non_empty(b.get("boardPlugins")) {
        cx.report
            .lose(LossKind::PowerUp, s(b, "name").unwrap_or(""));
    }
    Header {
        name: s(b, "name").unwrap_or("").to_owned(),
        starred: flag(b, "starred"),
        background,
        version: 0,
        watchers: Vec::new(),
        aging: None,
    }
}

fn members(cx: &mut Ctx, b: &Value) {
    for m in arr(b, "members") {
        let name = s(m, "username").or_else(|| s(m, "fullName"));
        match (s(m, "id"), name) {
            (Some(id), Some(n)) => {
                cx.members
                    .entry(id.to_owned())
                    .or_insert_with(|| Member(n.to_owned()));
            }
            _ => cx.report.lose(LossKind::BadValue, "member"),
        }
    }
}

fn labels(cx: &mut Ctx, b: &Value) -> BTreeSet<u64> {
    let mut got = BTreeSet::new();
    for l in arr(b, "labels") {
        let Some(id) = s(l, "id").and_then(|t| cx.fresh(t)) else {
            continue;
        };
        let name = s(l, "name").unwrap_or("").to_owned();
        let color = match l.get("color") {
            Some(Value::String(c)) => {
                let lc = label_color_of(c);
                if lc.is_none() {
                    cx.report.lose(LossKind::UnknownColor, c.clone());
                }
                lc
            }
            _ => None,
        };
        got.insert(id);
        cx.records.push(Record::Label(Label {
            id: LabelId(id),
            name,
            color,
            meta: Meta::default(),
        }));
    }
    got
}

pub(crate) fn hue_field(cx: &mut Ctx, v: &Value) -> Option<Hue> {
    match v.get("color") {
        Some(Value::String(c)) if c != "none" => {
            let base = c.trim_end_matches("_dark").trim_end_matches("_light");
            let h = hue_of(base);
            if h.is_none() {
                cx.report.lose(LossKind::UnknownColor, c.clone());
            }
            h
        }
        _ => None,
    }
}

fn lists(cx: &mut Ctx, b: &Value) -> BTreeSet<u64> {
    let mut got = Vec::new();
    for l in arr(b, "lists") {
        let Some(id) = s(l, "id").and_then(|t| cx.fresh(t)) else {
            continue;
        };
        let card_limit = match l.get("softLimit") {
            Some(Value::String(t)) => t.trim().parse().ok(),
            Some(Value::Number(n)) => n.as_u64().and_then(|n| u32::try_from(n).ok()),
            _ => None,
        };
        if flag(l, "subscribed") {
            cx.report
                .lose(LossKind::Subscription, s(l, "name").unwrap_or(""));
        }
        let color = hue_field(cx, l);
        got.push((
            pos(l),
            List {
                id: ListId(id),
                title: s(l, "name").unwrap_or("").to_owned(),
                pos: FracKey::between(None, None),
                color,
                card_limit,
                archived: flag(l, "closed"),
                meta: Meta::default(),
                watchers: Vec::new(),
            },
        ));
    }
    rank(got)
        .into_iter()
        .map(|(k, mut l)| {
            l.pos = k;
            let id = l.id.0;
            cx.records.push(Record::List(l));
            id
        })
        .collect()
}

/// Os campos: id do Trello → (o nosso id, o tipo).
type Fields = BTreeMap<String, (FieldId, FieldKind)>;

fn fields(cx: &mut Ctx, b: &Value) -> Fields {
    let mut got = Vec::new();
    let mut out = Fields::new();
    for f in arr(b, "customFields") {
        let Some(tid) = s(f, "id") else { continue };
        let Some(id) = cx.fresh(tid) else { continue };
        let kind = match s(f, "type") {
            Some("text") => FieldKind::Text,
            Some("number") => FieldKind::Number,
            Some("date") => FieldKind::Date,
            Some("checkbox") => FieldKind::Checkbox,
            Some("list") => {
                let mut opts = Vec::new();
                for o in arr(f, "options") {
                    let Some(oid) = s(o, "id").and_then(|t| cx.fresh(t)) else {
                        continue;
                    };
                    let text = o.get("value").and_then(|v| s(v, "text")).unwrap_or("");
                    let color = hue_field(cx, o);
                    opts.push((
                        pos(o),
                        FieldOption {
                            id: OptionId(oid),
                            text: text.to_owned(),
                            color,
                        },
                    ));
                }
                opts.sort_by(|a, b| a.0.total_cmp(&b.0));
                FieldKind::Dropdown(opts.into_iter().map(|(_, o)| o).collect())
            }
            other => {
                cx.report
                    .lose(LossKind::BadValue, other.unwrap_or("customField"));
                FieldKind::Text
            }
        };
        let on_front = f
            .get("display")
            .and_then(|d| d.get("cardFront"))
            .and_then(Value::as_bool)
            .unwrap_or(false);
        out.insert(tid.to_owned(), (FieldId(id), kind.clone()));
        got.push((
            pos(f),
            FieldDef {
                id: FieldId(id),
                name: s(f, "name").unwrap_or("").to_owned(),
                kind,
                on_front,
                pos: FracKey::between(None, None),
                meta: Meta::default(),
            },
        ));
    }
    for (k, mut f) in rank(got) {
        f.pos = k;
        cx.records.push(Record::Field(f));
    }
    out
}

fn field_value(cx: &Ctx, kind: &FieldKind, item: &Value) -> Option<FieldValue> {
    let v = item.get("value");
    let text = |k: &str| v.and_then(|v| s(v, k));
    match kind {
        FieldKind::Text => text("text").map(|t| FieldValue::Text(t.to_owned())),
        FieldKind::Number => text("number")
            .and_then(|t| t.trim().parse::<f64>().ok())
            .filter(|n| n.is_finite())
            .map(FieldValue::Number),
        FieldKind::Date => text("date").and_then(parse_time).map(FieldValue::Date),
        FieldKind::Checkbox => text("checked").map(|t| FieldValue::Checkbox(t == "true")),
        FieldKind::Dropdown(opts) => s(item, "idValue")
            .and_then(|t| cx.known(t))
            .filter(|o| opts.iter().any(|x| x.id.0 == *o))
            .map(|o| FieldValue::Option(OptionId(o))),
    }
}

fn names(cx: &mut Ctx, c: &Value, key: &str, kind: LossKind, at: &str) -> Vec<Member> {
    let mut out: Vec<Member> = Vec::new();
    for m in arr(c, key) {
        if let Some(m) = cx.member(m.as_str(), kind, at)
            && !out.contains(&m)
        {
            out.push(m);
        }
    }
    out
}

pub(crate) fn time_of(cx: &mut Ctx, v: &Value, k: &str, at: &str) -> Option<Moment> {
    match v.get(k) {
        Some(Value::String(t)) => {
            let m = parse_time(t);
            if m.is_none() {
                cx.report.lose(LossKind::BadValue, format!("{at}: {k}"));
            }
            m
        }
        _ => None,
    }
}

/// Os cartões aceites, pela ordem do ficheiro, com o objecto do Trello (anexos e capa leem-no).
fn cards<'b>(
    cx: &mut Ctx,
    b: &'b Value,
    [labels_known, lists]: [&BTreeSet<u64>; 2],
    fields: &Fields,
) -> Vec<(u64, &'b Value)> {
    let mut accepted = Vec::new();
    let mut by_list: BTreeMap<u64, Vec<(f64, u64)>> = BTreeMap::new();
    for c in arr(b, "cards") {
        let Some(tid) = s(c, "id") else { continue };
        let title = s(c, "name").unwrap_or("").to_owned();
        let Some(list) = s(c, "idList")
            .and_then(|l| cx.known(l))
            .filter(|l| lists.contains(l))
        else {
            cx.report.lose(LossKind::Orphan, title);
            continue;
        };
        let Some(id) = cx.fresh(tid) else { continue };
        let mut labels = Vec::new();
        for l in arr(c, "idLabels").iter().filter_map(Value::as_str) {
            match cx.known(l).filter(|l| labels_known.contains(l)) {
                Some(l) if !labels.contains(&LabelId(l)) => labels.push(LabelId(l)),
                Some(_) => {}
                None => cx
                    .report
                    .lose(LossKind::Orphan, format!("{title}: label {l}")),
            }
        }
        let members = names(cx, c, "idMembers", LossKind::UnknownMember, &title);
        let votes = names(cx, c, "idMembersVoted", LossKind::VoteWithoutMember, &title);
        let start = time_of(cx, c, "start", &title);
        let due = time_of(cx, c, "due", &title).map(|at| Due {
            at,
            done: flag(c, "dueComplete"),
            remind_before_min: c
                .get("dueReminder")
                .and_then(Value::as_i64)
                .and_then(|m| u32::try_from(m).ok()),
        });
        if due.is_none() && flag(c, "dueComplete") {
            cx.report
                .lose(LossKind::BadValue, format!("{title}: dueComplete"));
        }
        let mut values = BTreeMap::new();
        for item in arr(c, "customFieldItems") {
            let def = s(item, "idCustomField").and_then(|f| fields.get(f));
            match def.and_then(|(fid, kind)| field_value(cx, kind, item).map(|v| (*fid, v))) {
                Some((fid, v)) => {
                    values.insert(fid, v);
                }
                None => cx
                    .report
                    .lose(LossKind::BadValue, format!("{title}: customField")),
            }
        }
        if s(c, "cardRole").is_some() {
            cx.report.lose(LossKind::CardRole, title.clone());
        }
        if flag(c, "subscribed") {
            cx.report.lose(LossKind::Subscription, title.clone());
        }
        if non_empty(c.get("pluginData")) {
            cx.report.lose(LossKind::PowerUp, title.clone());
        }
        let created = match id_seconds(tid) {
            Some(secs) => Moment(secs.saturating_mul(1000)),
            None => {
                cx.report.lose(LossKind::BadValue, format!("{title}: id"));
                Moment(0)
            }
        };
        by_list.entry(list).or_default().push((pos(c), id));
        accepted.push((id, c));
        cx.cards.insert(
            id,
            Card {
                id: CardId(id),
                list: ListId(list),
                pos: FracKey::between(None, None),
                title,
                description: s(c, "desc").unwrap_or("").to_owned(),
                labels,
                members,
                start,
                due,
                cover: None,
                fields: values,
                location: s(c, "locationName").unwrap_or("").to_owned(),
                votes,
                created,
                archived: flag(c, "closed"),
                meta: Meta::default(),
                watchers: Vec::new(),
                origin: None,
                template: flag(c, "isTemplate"),
            },
        );
    }
    for group in by_list.into_values() {
        for (k, id) in rank(group) {
            if let Some(card) = cx.cards.get_mut(&id) {
                card.pos = k;
            }
        }
    }
    accepted
}

fn checklists(cx: &mut Ctx, b: &Value) {
    let mut accepted: Vec<(u64, &Value)> = Vec::new();
    let mut by_card: BTreeMap<u64, Vec<(f64, Checklist)>> = BTreeMap::new();
    for cl in arr(b, "checklists") {
        let title = s(cl, "name").unwrap_or("");
        let Some(card) = s(cl, "idCard")
            .and_then(|c| cx.known(c))
            .filter(|c| cx.cards.contains_key(c))
        else {
            cx.report.lose(LossKind::Orphan, title);
            continue;
        };
        let Some(id) = s(cl, "id").and_then(|t| cx.fresh(t)) else {
            continue;
        };
        accepted.push((id, cl));
        by_card.entry(card).or_default().push((
            pos(cl),
            Checklist {
                id: ChecklistId(id),
                card: CardId(card),
                pos: FracKey::between(None, None),
                title: title.to_owned(),
                hide_checked: false,
                meta: Meta::default(),
            },
        ));
    }
    for group in by_card.into_values() {
        for (k, mut cl) in rank(group) {
            cl.pos = k;
            cx.records.push(Record::Checklist(cl));
        }
    }
    for (cl, obj) in accepted {
        let mut got = Vec::new();
        for it in arr(obj, "checkItems") {
            let Some(id) = s(it, "id").and_then(|t| cx.fresh(t)) else {
                continue;
            };
            let text = s(it, "name").unwrap_or("").to_owned();
            let member = match s(it, "idMember") {
                Some(m) => cx.member(Some(m), LossKind::UnknownMember, &text),
                None => None,
            };
            let due = time_of(cx, it, "due", &text);
            got.push((
                pos(it),
                CheckItem {
                    id: CheckItemId(id),
                    checklist: ChecklistId(cl),
                    pos: FracKey::between(None, None),
                    text,
                    done: s(it, "state") == Some("complete"),
                    member,
                    due,
                    meta: Meta::default(),
                },
            ));
        }
        for (k, mut it) in rank(got) {
            it.pos = k;
            cx.records.push(Record::CheckItem(it));
        }
    }
}

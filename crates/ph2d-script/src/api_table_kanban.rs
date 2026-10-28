//! **`ph2d.kanban` — a API dos Kanbans do projecto** (TrelloClone W8, lote L): o irmão da
//! [`crate::api_table`] pelo tecto de LOC, lido pelas mesmas portas ([`crate::api_table::api`],
//! [`crate::api_table::api_types`]). A fiação verdadeira é do `ph2d-kanban-script`; a cena instala
//! as mesmas chaves a dizer onde correm ([`crate::kanban_stub`]).
//!
//! Os objectos que a API devolve são CLASSES ([`ApiClass`]): campos de leitura e métodos (linhas
//! [`ApiEntry`] com `table` = a classe), que saem no `ph2d.d.luau` como `export type`.

use crate::api_table::{ApiEntry, ApiGlobal, ApiType};

/// Um campo de leitura de uma classe.
pub struct ApiField {
    pub ident: &'static str,
    pub luau_type: &'static str,
    pub doc: &'static str,
}

/// Um objecto que a API devolve: os campos e (nas linhas da API com `table` = `ident`) os métodos.
pub struct ApiClass {
    pub ident: &'static str,
    pub doc: &'static str,
    pub fields: &'static [ApiField],
}

pub const KANBAN_GLOBAL: ApiGlobal = ApiGlobal {
    ident: "ph2d.kanban",
    doc: "The project's Kanbans: read lists and cards, change them, search, and react to what happens on a card.",
};

/// Os nomes dos acontecimentos de `ph2d.kanban.on` — o `ph2d-kanban-script` confere-os.
pub const KANBAN_EVENT_NAMES: &str = "\"card_created\" | \"card_moved\" | \"card_left\" | \"label_added\" | \"label_removed\" | \"member_added\" | \"member_removed\" | \"due_set\" | \"due_removed\" | \"due_completed\" | \"item_checked\" | \"checklist_completed\" | \"comment_added\" | \"card_archived\" | \"field_changed\"";

pub const KANBAN_TYPES: &[ApiType] = &[
    ApiType {
        ident: "KanbanInfo",
        definition: "{ id: number, name: string }",
        doc: "A Kanban of the project, as `ph2d.kanban.list` names it.",
    },
    ApiType {
        ident: "KanbanDate",
        definition: "string | { year: number, month: number, day: number, hour: number?, min: number? }",
        doc: "A date: ISO 8601 text (\"2026-11-03\", \"2026-11-03T14:30\", with `Z` or an offset for another zone) or a table; local time of the project.",
    },
    ApiType {
        ident: "KanbanLabel",
        definition: "{ id: number, name: string, color: string? }",
        doc: "A label of a Kanban: its name (may be empty) and its colour name.",
    },
    ApiType {
        ident: "KanbanCheckItem",
        definition: "{ text: string, done: boolean }",
        doc: "An item of a checklist.",
    },
    ApiType {
        ident: "KanbanChecklist",
        definition: "{ title: string, items: { KanbanCheckItem } }",
        doc: "A checklist of a card, its items in order.",
    },
    ApiType {
        ident: "KanbanCardSpec",
        definition: "{ title: string, description: string?, labels: { string }?, members: { string }?, due: KanbanDate?, start: KanbanDate?, top: boolean? }",
        doc: "A new card: its title and, if you want, the rest; `top = true` puts it first in the list.",
    },
    ApiType {
        ident: "KanbanCardChanges",
        definition: "{ title: string?, description: string?, due: (KanbanDate | boolean)?, start: (KanbanDate | boolean)?, done: boolean?, labels: { string }?, members: { string }?, fields: { [string]: any }? }",
        doc: "What `card:set` changes; what is missing stays, and `false` clears a date.",
    },
    ApiType {
        ident: "KanbanSortBy",
        definition: "\"newest\" | \"oldest\" | \"name\" | \"due\"",
        doc: "How `kanban:sort` orders a list.",
    },
    ApiType {
        ident: "KanbanEventName",
        definition: KANBAN_EVENT_NAMES,
        doc: "What happened to a card, for `ph2d.kanban.on`.",
    },
    ApiType {
        ident: "KanbanEvent",
        definition: "{ name: KanbanEventName, kanban: Kanban, card: KanbanCard, by: string, list: KanbanList?, from: KanbanList?, label: string?, member: string?, due: string?, done: boolean?, item: string?, checklist: string?, comment: string?, archived: boolean?, field: string? }",
        doc: "What a `ph2d.kanban.on` handler receives: the card, who did it, and what changed.",
    },
];

pub const KANBAN_CLASSES: &[ApiClass] = &[
    ApiClass {
        ident: "Kanban",
        doc: "A Kanban of the project.",
        fields: &[
            ApiField {
                ident: "id",
                luau_type: "number",
                doc: "Its number in the project.",
            },
            ApiField {
                ident: "name",
                luau_type: "string",
                doc: "Its name.",
            },
        ],
    },
    ApiClass {
        ident: "KanbanList",
        doc: "A list (column) of a Kanban.",
        fields: &[
            ApiField {
                ident: "id",
                luau_type: "number",
                doc: "Its number in the Kanban.",
            },
            ApiField {
                ident: "name",
                luau_type: "string",
                doc: "Its title.",
            },
            ApiField {
                ident: "archived",
                luau_type: "boolean",
                doc: "Whether it is archived.",
            },
            ApiField {
                ident: "kanban",
                luau_type: "Kanban",
                doc: "The Kanban it belongs to.",
            },
        ],
    },
    ApiClass {
        ident: "KanbanCard",
        doc: "A card. Its fields are read when you ask, so they show your own changes.",
        fields: &[
            ApiField {
                ident: "id",
                luau_type: "number",
                doc: "Its number in the Kanban.",
            },
            ApiField {
                ident: "title",
                luau_type: "string",
                doc: "Its title.",
            },
            ApiField {
                ident: "description",
                luau_type: "string",
                doc: "Its description, in Markdown.",
            },
            ApiField {
                ident: "labels",
                luau_type: "{ string }",
                doc: "The names of its labels.",
            },
            ApiField {
                ident: "members",
                luau_type: "{ string }",
                doc: "Its members.",
            },
            ApiField {
                ident: "due",
                luau_type: "string?",
                doc: "Its due date as local ISO 8601 text (\"2026-11-03T14:30\"); nil without one.",
            },
            ApiField {
                ident: "start",
                luau_type: "string?",
                doc: "Its start date, like `due`.",
            },
            ApiField {
                ident: "done",
                luau_type: "boolean",
                doc: "Whether its due date is marked complete.",
            },
            ApiField {
                ident: "list",
                luau_type: "KanbanList",
                doc: "The list it is in.",
            },
            ApiField {
                ident: "kanban",
                luau_type: "Kanban",
                doc: "The Kanban it is in.",
            },
            ApiField {
                ident: "fields",
                luau_type: "{ [string]: any }",
                doc: "Its custom fields by name: text, number, date text, true/false or the chosen option.",
            },
            ApiField {
                ident: "checklists",
                luau_type: "{ KanbanChecklist }",
                doc: "Its checklists, in order.",
            },
            ApiField {
                ident: "archived",
                luau_type: "boolean",
                doc: "Whether it is archived.",
            },
            ApiField {
                ident: "url",
                luau_type: "string",
                doc: "Its link inside the project (what «Copy link» gives).",
            },
        ],
    },
];

pub const KANBAN_API: &[ApiEntry] = &[
    // ── ph2d.kanban ──
    ApiEntry {
        table: "ph2d.kanban",
        ident: "list",
        luau_type: "() -> { KanbanInfo }",
        doc: "The Kanbans of the project, in order.",
    },
    ApiEntry {
        table: "ph2d.kanban",
        ident: "get",
        luau_type: "(kanban: string | number) -> Kanban?",
        doc: "A Kanban by its name or its number; nil if none.",
    },
    ApiEntry {
        table: "ph2d.kanban",
        ident: "search",
        luau_type: "(query: string) -> { KanbanCard }",
        doc: "The cards of every Kanban that match a search, the best first (same words as the search box).",
    },
    ApiEntry {
        table: "ph2d.kanban",
        ident: "on",
        luau_type: "(event: KanbanEventName, handler: (KanbanEvent) -> ()) -> ()",
        doc: "Calls `handler` each time this happens to a card, by anyone — the same events as the automation rules.",
    },
    ApiEntry {
        table: "ph2d.kanban",
        ident: "press_button",
        luau_type: "(name: string, card: KanbanCard?) -> boolean",
        doc: "Presses a board button by its name, or a card button on `card`; false if there is no such button.",
    },
    // ── Kanban ──
    ApiEntry {
        table: "Kanban",
        ident: "lists",
        luau_type: "(self: Kanban) -> { KanbanList }",
        doc: "Its open lists, left to right.",
    },
    ApiEntry {
        table: "Kanban",
        ident: "list",
        luau_type: "(self: Kanban, name: string) -> KanbanList?",
        doc: "Its first open list with this name; nil if none.",
    },
    ApiEntry {
        table: "Kanban",
        ident: "cards",
        luau_type: "(self: Kanban, filter: string?) -> { KanbanCard }",
        doc: "Its open cards in board order, or the ones a search finds in it.",
    },
    ApiEntry {
        table: "Kanban",
        ident: "labels",
        luau_type: "(self: Kanban) -> { KanbanLabel }",
        doc: "Its labels.",
    },
    ApiEntry {
        table: "Kanban",
        ident: "members",
        luau_type: "(self: Kanban) -> { string }",
        doc: "Everyone on its cards or following it, sorted.",
    },
    ApiEntry {
        table: "Kanban",
        ident: "add_list",
        luau_type: "(self: Kanban, name: string) -> KanbanList",
        doc: "Adds a list at the right end.",
    },
    ApiEntry {
        table: "Kanban",
        ident: "sort",
        luau_type: "(self: Kanban, list: KanbanList | string, by: KanbanSortBy) -> ()",
        doc: "Sorts the cards of one of its lists.",
    },
    // ── KanbanList ──
    ApiEntry {
        table: "KanbanList",
        ident: "cards",
        luau_type: "(self: KanbanList) -> { KanbanCard }",
        doc: "Its open cards, top to bottom.",
    },
    ApiEntry {
        table: "KanbanList",
        ident: "add_card",
        luau_type: "(self: KanbanList, card: string | KanbanCardSpec) -> KanbanCard",
        doc: "Adds a card at the bottom (or the top), by its title or with more.",
    },
    // ── KanbanCard ──
    ApiEntry {
        table: "KanbanCard",
        ident: "move",
        luau_type: "(self: KanbanCard, list: KanbanList | string, position: number?) -> ()",
        doc: "Moves it to a list — of any Kanban — at the bottom, or at a position (1 = top).",
    },
    ApiEntry {
        table: "KanbanCard",
        ident: "set",
        luau_type: "(self: KanbanCard, changes: KanbanCardChanges) -> ()",
        doc: "Changes several things at once: title, description, dates, labels, members, custom fields.",
    },
    ApiEntry {
        table: "KanbanCard",
        ident: "add_label",
        luau_type: "(self: KanbanCard, label: string) -> ()",
        doc: "Adds one of the Kanban's labels, by its name (or its colour if it has no name).",
    },
    ApiEntry {
        table: "KanbanCard",
        ident: "remove_label",
        luau_type: "(self: KanbanCard, label: string) -> ()",
        doc: "Removes a label, by its name (or its colour).",
    },
    ApiEntry {
        table: "KanbanCard",
        ident: "add_member",
        luau_type: "(self: KanbanCard, member: string) -> ()",
        doc: "Adds a member, by name.",
    },
    ApiEntry {
        table: "KanbanCard",
        ident: "comment",
        luau_type: "(self: KanbanCard, text: string) -> ()",
        doc: "Writes a comment (Markdown, `@name` mentions).",
    },
    ApiEntry {
        table: "KanbanCard",
        ident: "check",
        luau_type: "(self: KanbanCard, item: string, done: boolean?) -> ()",
        doc: "Ticks (or unticks) the first checklist item with this text.",
    },
    ApiEntry {
        table: "KanbanCard",
        ident: "archive",
        luau_type: "(self: KanbanCard) -> ()",
        doc: "Archives it.",
    },
];

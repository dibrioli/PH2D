//! **As páginas de ajuda de `ph2d.kanban`** — o irmão do [`crate::api_help`] pelo tecto de LOC, lido
//! pela mesma [`crate::api_help::all_help`]. O texto diz o que o `ph2d-kanban-script` faz (os testes
//! dele são a medida). Os métodos não listam o `self`.

use crate::api_help::ApiHelp;

const RUN: &str = "Everything one run of the script changes — its top, or one handler — is one step of Ctrl+Z, and shows in the card's activity as done by «script: <its name>». If the script stops with an error, nothing it changed in that run stays.";

pub(crate) const KANBAN_HELP: &[ApiHelp] = &[
    ApiHelp {
        key: "ph2d.kanban.list",
        params: &[],
        returns: "One `{ id, name }` per Kanban, in the order of the Kanbans page.",
        more: "",
        example: r#"for _, k in ph2d.kanban.list() do
  print(k.id, k.name)
end"#,
    },
    ApiHelp {
        key: "ph2d.kanban.get",
        params: &[(
            "kanban",
            "Its name (capitals and accents do not matter) or its number from `ph2d.kanban.list`.",
        )],
        returns: "The Kanban, or nil.",
        more: "",
        example: r#"local k = ph2d.kanban.get("Short film")
if k then
  print(#k:cards())
end"#,
    },
    ApiHelp {
        key: "ph2d.kanban.search",
        params: &[(
            "query",
            "Words and operators, as in the search box: `@ana`, `#urgent`, `due:week`, `due:overdue`, `is:archived`, `is:complete`, `has:checklists`, `list:Doing`, `board:\"Short film\"`, `-` before one to leave it out.",
        )],
        returns: "The cards found, the best match first.",
        more: "",
        example: r#"for _, card in ph2d.kanban.search("due:overdue -is:complete") do
  card:add_label("Late")
end"#,
    },
    ApiHelp {
        key: "ph2d.kanban.on",
        params: &[
            (
                "event",
                "What to listen for: \"card_created\", \"card_moved\" (into a list), \"card_left\" (a list), \"label_added\", \"comment_added\", \"due_completed\"… (see [KanbanEventName]).",
            ),
            ("handler", "A function that receives a [KanbanEvent]."),
        ],
        returns: "",
        more: "Call it at the top of the script. The handler runs after each change by a person, by an automation rule or by another script.\n\nA handler never runs again because of what it changed itself (directly or through rules it set off), so a handler that moves the card it was told about cannot loop; a chain of handlers and rules stops after 16 steps, and one change sets off at most 1000 of them.",
        example: r#"ph2d.kanban.on("card_moved", function(e)
  if e.list and e.list.name == "Done" then
    e.card:set({ done = true })
  end
end)"#,
    },
    ApiHelp {
        key: "ph2d.kanban.press_button",
        params: &[
            (
                "name",
                "The button's name, as on the Kanban's top bar or on the back of a card.",
            ),
            (
                "card",
                "For a card button: the card it acts on. Without it, a board button.",
            ),
        ],
        returns: "true if a button with that name was found and pressed.",
        more: "The button does what it does when you click it, in the name of the automation.",
        example: r#"ph2d.kanban.press_button("Sort by due date")"#,
    },
    ApiHelp {
        key: "Kanban.lists",
        params: &[],
        returns: "Its lists that are not archived.",
        more: "",
        example: r#"local k = ph2d.kanban.get("Short film")
if k then
  for _, list in k:lists() do
    print(list.name, #list:cards())
  end
end"#,
    },
    ApiHelp {
        key: "Kanban.list",
        params: &[(
            "name",
            "The list's title; capitals and accents do not matter.",
        )],
        returns: "The list, or nil.",
        more: "",
        example: r#"local k = ph2d.kanban.get("Short film")
local done = k and k:list("Done")
if done then
  print(#done:cards())
end"#,
    },
    ApiHelp {
        key: "Kanban.cards",
        params: &[(
            "filter",
            "A search, as in [ph2d.kanban.search], only inside this Kanban. Without it, every open card.",
        )],
        returns: "The cards, list by list and top to bottom.",
        more: "",
        example: r##"local k = ph2d.kanban.get("Short film")
if k then
  for _, card in k:cards("#urgent") do
    print(card.title, card.list.name)
  end
end"##,
    },
    ApiHelp {
        key: "Kanban.labels",
        params: &[],
        returns: "One `{ id, name, color }` per label.",
        more: "",
        example: r#"local k = ph2d.kanban.get("Short film")
if k then
  for _, l in k:labels() do
    print(l.name, l.color)
  end
end"#,
    },
    ApiHelp {
        key: "Kanban.members",
        params: &[],
        returns: "The names.",
        more: "",
        example: r#"local k = ph2d.kanban.get("Short film")
if k then
  print(table.concat(k:members(), ", "))
end"#,
    },
    ApiHelp {
        key: "Kanban.add_list",
        params: &[("name", "The new list's title (not empty).")],
        returns: "The new list.",
        more: RUN,
        example: r#"local k = ph2d.kanban.get("Short film")
if k then
  local review = k:add_list("Review")
  review:add_card("Check the sound")
end"#,
    },
    ApiHelp {
        key: "Kanban.sort",
        params: &[
            ("list", "One of its lists, or a list's title."),
            (
                "by",
                "\"newest\" or \"oldest\" (when made), \"name\", or \"due\" (the nearest first, cards without a date last).",
            ),
        ],
        returns: "",
        more: "",
        example: r#"local k = ph2d.kanban.get("Short film")
if k then
  k:sort("To do", "due")
end"#,
    },
    ApiHelp {
        key: "KanbanList.cards",
        params: &[],
        returns: "Its cards that are not archived.",
        more: "",
        example: r#"local k = ph2d.kanban.get("Short film")
local list = k and k:list("Doing")
if list then
  for i, card in list:cards() do
    print(i, card.title)
  end
end"#,
    },
    ApiHelp {
        key: "KanbanList.add_card",
        params: &[(
            "card",
            "Its title, or a [KanbanCardSpec] with the title and more.",
        )],
        returns: "The new card.",
        more: "A label is named as on the Kanban (it must exist); a member is any name of the team; dates are [KanbanDate].",
        example: r#"local k = ph2d.kanban.get("Short film")
local list = k and k:list("To do")
if list then
  list:add_card({ title = "Storyboard", labels = { "Art" }, due = "2026-11-03T18:00" })
end"#,
    },
    ApiHelp {
        key: "KanbanCard.move",
        params: &[
            (
                "list",
                "Where to: a list (of this or another Kanban) or the title of a list of this Kanban.",
            ),
            (
                "position",
                "1 = the top, 2 = under the first card…; without it, the bottom.",
            ),
        ],
        returns: "",
        more: "Moving to another Kanban moves the card with everything it has, as dragging it does; the card you hold then is the moved one.",
        example: r#"ph2d.kanban.on("label_added", function(e)
  if e.label == "Urgent" then
    e.card:move(e.card.list, 1)
  end
end)"#,
    },
    ApiHelp {
        key: "KanbanCard.set",
        params: &[(
            "changes",
            "A [KanbanCardChanges]: only what it names changes.",
        )],
        returns: "",
        more: "`labels` and `members` replace the whole set. A custom field is named as on the card; a dropdown takes the option's text.",
        example: r#"for _, card in ph2d.kanban.search("list:Done") do
  card:set({ done = true, due = false })
end"#,
    },
    ApiHelp {
        key: "KanbanCard.add_label",
        params: &[(
            "label",
            "The name of one of the Kanban's labels, or its colour (\"green\") for a label without a name.",
        )],
        returns: "",
        more: "A name the Kanban has no label for stops the script with an error naming the labels it has.",
        example: r#"for _, card in ph2d.kanban.search("due:overdue") do
  card:add_label("Late")
end"#,
    },
    ApiHelp {
        key: "KanbanCard.remove_label",
        params: &[(
            "label",
            "Its name, or its colour for a label without a name.",
        )],
        returns: "",
        more: "",
        example: r#"for _, card in ph2d.kanban.search("is:complete") do
  card:remove_label("Late")
end"#,
    },
    ApiHelp {
        key: "KanbanCard.add_member",
        params: &[("member", "The person's name in the team.")],
        returns: "",
        more: "",
        example: r#"ph2d.kanban.on("card_created", function(e)
  e.card:add_member("Ana")
end)"#,
    },
    ApiHelp {
        key: "KanbanCard.comment",
        params: &[("text", "The comment, in Markdown (not empty).")],
        returns: "",
        more: "",
        example: r#"ph2d.kanban.on("card_moved", function(e)
  e.card:comment("Moved by " .. e.by)
end)"#,
    },
    ApiHelp {
        key: "KanbanCard.check",
        params: &[
            (
                "item",
                "The item's text, as written (capitals and accents do not matter).",
            ),
            ("done", "false to untick; ticks without it."),
        ],
        returns: "",
        more: "A text no item of the card has stops the script with an error.",
        example: r#"ph2d.kanban.on("card_moved", function(e)
  if e.list and e.list.name == "Review" then
    e.card:check("Write the brief")
  end
end)"#,
    },
    ApiHelp {
        key: "KanbanCard.archive",
        params: &[],
        returns: "",
        more: "",
        example: r#"for _, card in ph2d.kanban.search("list:Done is:complete") do
  card:archive()
end"#,
    },
    ApiHelp {
        key: "type.KanbanInfo",
        params: &[],
        returns: "",
        more: "",
        example: "",
    },
    ApiHelp {
        key: "type.KanbanDate",
        params: &[],
        returns: "",
        more: "Without `Z` or an offset the time is the project's local time; a day without a time is midnight. The table form takes `hour` and `min` too.",
        example: "",
    },
    ApiHelp {
        key: "type.KanbanLabel",
        params: &[],
        returns: "",
        more: "",
        example: "",
    },
    ApiHelp {
        key: "type.KanbanCheckItem",
        params: &[],
        returns: "",
        more: "",
        example: "",
    },
    ApiHelp {
        key: "type.KanbanChecklist",
        params: &[],
        returns: "",
        more: "",
        example: "",
    },
    ApiHelp {
        key: "type.KanbanCardSpec",
        params: &[],
        returns: "",
        more: "",
        example: "",
    },
    ApiHelp {
        key: "type.KanbanCardChanges",
        params: &[],
        returns: "",
        more: "",
        example: "",
    },
    ApiHelp {
        key: "type.KanbanSortBy",
        params: &[],
        returns: "",
        more: "",
        example: "",
    },
    ApiHelp {
        key: "type.KanbanEventName",
        params: &[],
        returns: "",
        more: "\"card_moved\" is the list it came into (`list`, and `from`); \"card_left\" the list it left. \"due_completed\" fires both ways (`done`); \"card_archived\" too (`archived`).",
        example: "",
    },
    ApiHelp {
        key: "type.KanbanEvent",
        params: &[],
        returns: "",
        more: "Only the fields of that event are set: `list` and `from` for a move, `label`, `member`, `due`, `done`, `item`, `checklist`, `comment`, `archived` or `field` for the others.",
        example: "",
    },
];

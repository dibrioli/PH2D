//! **A Bancada do editor de scripts** (plano 13 Onda B): o «In this script», o balão «em quem», a
//! grelha «Who has what», o mapa, a frase do pé e o menu View.

/// A tradução de uma chave `code.bancada.*` (e do menu View), ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "code.bancada.outline.title" => "In this script",
        "code.bancada.outline.properties" => "Properties",
        "code.bancada.outline.functions" => "Functions",
        "code.bancada.outline.hears" => "Hears",
        "code.bancada.outline.line" => "ln {line}",
        "code.bancada.outline.empty" => "No properties, functions or signals yet.",
        "code.bancada.grid.title" => "Who has what",
        "code.bancada.grid.count" => {
            "{objects} {objects?object|objects} \u{00d7} {props} {props?property|properties}"
        }
        "code.bancada.grid.no_objects" => {
            "No object runs this script yet. Pick it in an object's Script card."
        }
        "code.bancada.grid.no_props" => {
            "This script has no properties yet: add one with ph2d.property(\"speed\", 220)."
        }
        "code.bancada.map.title" => "Where",
        "code.bancada.map.empty" => "Nothing to draw yet.",
        "code.bancada.pop.line" => "\u{00b7} line {line}",
        "code.bancada.pop.default_is" => "default {value}",
        "code.bancada.pop.heard_by" => "heard by",
        "code.bancada.pop.default" => "default",
        "code.bancada.pop.changed" => "changed",
        "code.bancada.pop.hears" => "runs line {line}",
        "code.bancada.pop.select" => "Select {name}",
        "code.bancada.pop.none" => "No object runs this script yet.",
        "code.bancada.say.idle" => "{script} runs on {count} {count?object|objects}: {names}.",
        "code.bancada.say.idle_none" => {
            "No object runs {script} yet. Pick it in an object's Script card to see it here."
        }
        "code.bancada.say.cell_changed" => {
            "{name} \u{00b7} {prop} is {value} here ({default} in the script). Click to edit."
        }
        "code.bancada.say.cell_default" => {
            "{name} \u{00b7} {prop} is {value}, the script's value. Click to edit."
        }
        "code.bancada.say.cell_bool" => {
            "{name} \u{00b7} {prop} is {value}. Click to switch it to {next}."
        }
        "code.bancada.say.cell_choice" => {
            "{name} \u{00b7} {prop} is {value}. Click for the next option, {next}."
        }
        "code.bancada.say.cell_color" => {
            "{name} \u{00b7} colors are edited in the Inspector. Click to select {name}."
        }
        "code.bancada.say.cell_unsaved" => "{prop} is new. Save the script to edit it here.",
        "code.bancada.say.editing" => "Type a new {prop} for {name}. Enter keeps it, Esc cancels.",
        "code.bancada.say.holder_peek" => {
            "{name} runs this script. Hold the pointer still to see it in the scene, or click to select it."
        }
        "code.bancada.say.holder" => "{name} runs this script. Click to select it.",
        "code.bancada.say.column" => "{prop} is declared on line {line}. Click to go there.",
        "code.bancada.say.mark_prop" => {
            "{prop}: {changed} of {count} {count?object|objects} changed it. Click to keep this open."
        }
        "code.bancada.say.mark_hear" => {
            "When a {signal} signal arrives, line {line} runs on {names}. Click to keep this open."
        }
        "code.bancada.say.select" => "Click to select {name} in the scene and in the Inspector.",
        "code.bancada.say.entry" => "Click to go to line {line}.",
        "code.bancada.say.peek" => "{name} in the scene. Move the pointer to bring the code back.",
        "code.bancada.say.none_yet" => "nobody yet",
        "code.menu.view" => "View",
        "code.cmd.peek" => "Peek at Scene: Off",
        "code.cmd.peek_off" => "Peek at Scene: On",
        "code.cmd.map" => "Scene Map: Off",
        "code.cmd.map_off" => "Scene Map: On",
        "code.cmd.card_sheet" => "Card Style: Sheet",
        "code.cmd.card_sheet_on" => "Card Style: Sheet \u{2713}",
        "code.cmd.card_cards" => "Card Style: Cards",
        "code.cmd.card_cards_on" => "Card Style: Cards \u{2713}",
        "code.cmd.card_strip" => "Card Style: Strip",
        "code.cmd.card_strip_on" => "Card Style: Strip \u{2713}",
        "code.cmd.text_default" => "Text rendering: Default",
        "code.cmd.text_default_on" => "Text rendering: Default \u{2713}",
        "code.cmd.text_crisp_heavy" => "Text rendering: Crisp Heavy",
        "code.cmd.text_crisp_heavy_on" => "Text rendering: Crisp Heavy \u{2713}",
        "code.cmd.text_crisp_heavy_plus" => "Text rendering: Crisp Heavy +",
        "code.cmd.text_crisp_heavy_plus_on" => "Text rendering: Crisp Heavy + \u{2713}",
        "code.cmd.text_crisp_darkened" => "Text rendering: Crisp + stem darkening",
        "code.cmd.text_crisp_darkened_on" => "Text rendering: Crisp + stem darkening \u{2713}",
        _ => return None,
    })
}

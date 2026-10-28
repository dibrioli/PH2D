//! O vocabulário do Trello que a ida e a volta partilham: cores, datas, ids e gradientes.

use ph2d_kanban_model::{Hue, LabelColor, Moment, Shade};

/// Os nomes de matiz do Trello, na ordem de [`Hue::ALL`].
const HUE_NAMES: [&str; 10] = [
    "green", "yellow", "orange", "red", "purple", "blue", "sky", "lime", "pink", "black",
];

/// Os fundos em gradiente do Trello, pela ordem do seletor — o índice é o de `Background::Gradient`.
pub(crate) const GRADIENTS: [&str; 9] = [
    "gradient-snow",
    "gradient-ocean",
    "gradient-crystal",
    "gradient-rainbow",
    "gradient-peach",
    "gradient-flower",
    "gradient-earth",
    "gradient-alien",
    "gradient-volcano",
];

pub(crate) fn hue_name(h: Hue) -> &'static str {
    HUE_NAMES[Hue::ALL.iter().position(|x| *x == h).unwrap_or(0)]
}

/// Um matiz pelo nome (`grey`/`gray` do fundo do Trello = o neutro, `Black`).
pub(crate) fn hue_of(name: &str) -> Option<Hue> {
    if matches!(name, "grey" | "gray") {
        return Some(Hue::Black);
    }
    HUE_NAMES
        .iter()
        .position(|n| *n == name)
        .map(|i| Hue::ALL[i])
}

/// `green_light` = sutil · `green` = normal · `green_dark` = forte (as 30 do seletor do Trello).
#[must_use]
pub fn label_color_of(name: &str) -> Option<LabelColor> {
    let (base, shade) = if let Some(b) = name.strip_suffix("_light") {
        (b, Shade::Subtle)
    } else if let Some(b) = name.strip_suffix("_dark") {
        (b, Shade::Bold)
    } else {
        (name, Shade::Normal)
    };
    HUE_NAMES
        .iter()
        .position(|n| *n == base)
        .map(|i| LabelColor {
            hue: Hue::ALL[i],
            shade,
        })
}

/// O nome do Trello de uma das 30 cores.
#[must_use]
pub fn label_color_name(c: LabelColor) -> String {
    let base = hue_name(c.hue);
    match c.shade {
        Shade::Subtle => format!("{base}_light"),
        Shade::Normal => base.to_owned(),
        Shade::Bold => format!("{base}_dark"),
    }
}

/// Uma data RFC 3339 do Trello (`2026-10-28T12:00:00.000Z`, ou com fuso) em milissegundos.
pub(crate) fn parse_time(s: &str) -> Option<Moment> {
    s.parse::<jiff::Timestamp>()
        .ok()
        .map(|t| Moment(t.as_millisecond()))
}

/// O formato que o Trello escreve: UTC, sempre com milissegundos.
pub(crate) fn format_time(m: Moment) -> String {
    jiff::Timestamp::from_millisecond(m.0)
        .map(|t| format!("{t:.3}"))
        .unwrap_or_default()
}

/// Um id do Trello é um ObjectId: 24 dígitos hex, os 8 primeiros são os segundos da criação.
pub(crate) fn id_seconds(id: &str) -> Option<i64> {
    if id.len() != 24 || !id.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    i64::from_str_radix(&id[..8], 16).ok()
}

/// O id que a exportação escreve: os segundos de `at` (0 sem data, ou fora de 32 bits) e o id local.
pub(crate) fn object_id(raw: u64, at: Option<Moment>) -> String {
    let secs = at
        .map(|m| m.0.div_euclid(1000))
        .and_then(|s| u32::try_from(s).ok())
        .unwrap_or(0);
    format!("{secs:08x}{raw:016x}")
}

/// Os URLs que a exportação dá aos anexos que o Trello não sabe guardar (ficheiro do projecto,
/// cartão, Kanban) — a importação reconhece-os e devolve o mesmo `Link`.
pub(crate) const FILE_SCHEME: &str = "ph2d://file/";
pub(crate) const CARD_SCHEME: &str = "ph2d://kanban/";

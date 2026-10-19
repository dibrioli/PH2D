//! ⭐ **O `.excalidraw` de ida e volta** (W8, `docs/MiroClone/06_equipa_e_saida.md` §2 X1–X4).
//!
//! O esquema é o que o oráculo (Excalidraw 0.18.1, `ferramentas/excalidraw_oracle/` §«W8») GRAVA e o
//! que o `restore` dele aceita — nunca o fonte. Ida ([`write`]): cada elemento sai com o equivalente
//! mais próximo do Excalidraw, e o nosso elemento inteiro viaja em `customData.ph2d` (o `restore` dele
//! preserva-o tal e qual). Volta ([`read`]): o que o Excalidraw edita (posição, tamanho, rotação,
//! texto, cores, traço, opacidade, rascunho, grupos, frame, travar) vem DELE — o quadro mexido lá
//! volta mexido —, o resto (o tipo de forma, as etiquetas, os votos, os carimbos, o mapa) do
//! `customData`; um ficheiro de fora entra pelo mapeamento directo. ⇒ nosso → Excalidraw → nosso =
//! IGUAL, a menos da ordem das chaves de z.

mod bind;
mod elbow;
mod embed;
mod image;
mod legacy_focus;
mod read;
mod style;
mod video;
mod write;

pub use embed::scene_from_bytes;
pub use read::{Import, read, read_with};
pub use write::write;

use ph2d_board_model::{Head, Rgba};

/// O MIME de um ficheiro `.excalidraw` (oráculo: `MIME_TYPES.excalidraw`).
pub const MIME: &str = "application/vnd.excalidraw+json";

/// A chave nossa dentro do `customData` de cada elemento.
pub(crate) const PH2D: &str = "ph2d";

/// O prefixo dos ids que escrevemos (o nosso `ElementId` em decimal).
pub(crate) const ID: &str = "ph2d-";

/// A altura de uma linha de texto em tamanhos de letra (o `lineHeight` de nascença dele, oráculo).
pub(crate) const LINE_HEIGHT: f64 = 1.25;

/// Uma cor de documento → a do Excalidraw (`transparent`, `#rrggbb`, ou `#rrggbbaa` com alfa).
pub(crate) fn hex(c: Option<Rgba>) -> String {
    match c {
        None => "transparent".to_owned(),
        Some(Rgba([r, g, b, 255])) => format!("#{r:02x}{g:02x}{b:02x}"),
        Some(Rgba([r, g, b, a])) => format!("#{r:02x}{g:02x}{b:02x}{a:02x}"),
    }
}

/// A cor do Excalidraw → a do documento: `Some(None)` = transparente; `None` = não se percebe (o
/// `restore` dele NÃO valida cores: o leitor cai no de nascença, X3).
pub(crate) fn parse_color(s: &str) -> Option<Option<Rgba>> {
    let s = s.trim();
    if s.eq_ignore_ascii_case("transparent") {
        return Some(None);
    }
    let h = s.strip_prefix('#')?;
    let digit = |i: usize, n: usize| u8::from_str_radix(h.get(i..i + n)?, 16).ok();
    let v = match h.len() {
        3 => [digit(0, 1)? * 17, digit(1, 1)? * 17, digit(2, 1)? * 17, 255],
        6 => [digit(0, 2)?, digit(2, 2)?, digit(4, 2)?, 255],
        8 => [digit(0, 2)?, digit(2, 2)?, digit(4, 2)?, digit(6, 2)?],
        _ => return None,
    };
    Some(Some(Rgba(v)))
}

/// Uma ponta nossa → a do Excalidraw (`null` = sem ponta).
pub(crate) fn head_name(h: Head) -> Option<&'static str> {
    Some(match h {
        Head::None => return None,
        Head::Arrow => "arrow",
        Head::Triangle => "triangle",
        Head::Diamond => "diamond",
        Head::DiamondOpen => "diamond_outline",
        Head::Circle => "circle",
        Head::CircleOpen => "circle_outline",
        Head::Bar => "bar",
        Head::TriangleOpen => "triangle_outline",
        // As três que o 0.18.1 já desenha vão pelo nome antigo (o next renomeia-as ao ler, XV5); as
        // outras três só existem no next (o 0.18.1 desenha-as como `arrow`, `P_tipos`).
        Head::One => "crowfoot_one",
        Head::Many => "crowfoot_many",
        Head::OneOrMany => "crowfoot_one_or_many",
        Head::ExactlyOne => "cardinality_exactly_one",
        Head::ZeroOrOne => "cardinality_zero_or_one",
        Head::ZeroOrMany => "cardinality_zero_or_many",
    })
}

/// A ponta do Excalidraw → a nossa (as duas gramáticas, XV5; uma desconhecida = sem ponta).
pub(crate) fn head_of(s: Option<&str>) -> Head {
    match s {
        Some("arrow") => Head::Arrow,
        Some("triangle") => Head::Triangle,
        Some("triangle_outline") => Head::TriangleOpen,
        Some("crowfoot_one" | "cardinality_one") => Head::One,
        Some("crowfoot_many" | "cardinality_many") => Head::Many,
        Some("crowfoot_one_or_many" | "cardinality_one_or_many") => Head::OneOrMany,
        Some("cardinality_exactly_one") => Head::ExactlyOne,
        Some("cardinality_zero_or_one") => Head::ZeroOrOne,
        Some("cardinality_zero_or_many") => Head::ZeroOrMany,
        Some("diamond") => Head::Diamond,
        Some("diamond_outline") => Head::DiamondOpen,
        Some("circle" | "dot") => Head::Circle,
        Some("circle_outline") => Head::CircleOpen,
        Some("bar") => Head::Bar,
        _ => Head::None,
    }
}

/// O `fontFamily` que escrevemos: a Virgil (1) à mão — a nossa letra à mão É a Virgil —, senão a
/// Helvetica (2), a sem serifa dele mais perto da nossa. A letra do modo segue o rascunho.
pub(crate) fn font_family(st: &ph2d_board_model::Style) -> u32 {
    let hand = match st.look.letter {
        ph2d_board_model::Letter::Auto => st.sketch,
        ph2d_board_model::Letter::Hand => true,
        ph2d_board_model::Letter::Sans => false,
    };
    if hand { 1 } else { 2 }
}

/// A letra de um `fontFamily` dele (o estudo 07 §4): as à mão (Virgil 1, Excalifont 5, Comic Shanns
/// 8) ⇒ a nossa à mão; as outras (Helvetica 2, Nunito 6, Liberation 9, e a Cascadia 3, mono, que não
/// temos) ⇒ a da interface.
pub(crate) fn letter_of(family: Option<u64>) -> ph2d_board_model::Letter {
    match family {
        Some(1 | 5 | 8) => ph2d_board_model::Letter::Hand,
        Some(_) => ph2d_board_model::Letter::Sans,
        None => ph2d_board_model::Letter::Auto,
    }
}

#[cfg(test)]
#[path = "fidelity_tests.rs"]
mod fidelity_tests;
#[cfg(test)]
#[path = "image_tests.rs"]
mod image_tests;
#[cfg(test)]
#[path = "image_w15_tests.rs"]
mod image_w15_tests;
#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
#[cfg(test)]
#[path = "video_tests.rs"]
mod video_tests;

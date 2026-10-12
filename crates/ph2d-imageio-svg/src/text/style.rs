//! O estilo de texto de um elemento: a cascata que o usvg aplica (atributo de apresentação < regras
//! do `<style>` por ordem do documento < `style=""`), o `font` expandido, e a herança.

use parley::{FontStyle, FontWeight, FontWidth};
use usvg::roxmltree::Node;

pub(crate) use simplecss::StyleSheet;

/// As propriedades que o texto lê (o resto fica para o usvg, no `<g>` que substitui o elemento).
const TEXT_PROPS: &[&str] = &[
    "font-family",
    "font-size",
    "font-weight",
    "font-style",
    "font-stretch",
    "font-variant",
    "letter-spacing",
    "word-spacing",
    "text-anchor",
    "white-space",
    "display",
    "writing-mode",
    "direction",
    "baseline-shift",
    "alignment-baseline",
    "dominant-baseline",
    "text-decoration",
];

/// O `font-size` de partida do CSS (`medium`).
const MEDIUM_PX: f32 = 16.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Anchor {
    Start,
    Middle,
    End,
}

/// O estilo calculado (herdado) de um elemento de texto.
#[derive(Clone, Debug)]
pub(crate) struct Style {
    pub family: String,
    pub size: f32,
    pub weight: FontWeight,
    pub style: FontStyle,
    pub width: FontWidth,
    pub letter: f32,
    pub word: f32,
    pub anchor: Anchor,
    pub preserve: bool,
    pub hidden: bool,
    /// As propriedades presentes que o desenho não segue (`writing-mode` vertical, …).
    pub unsupported: Vec<&'static str>,
}

impl Style {
    pub(crate) fn initial() -> Self {
        Self {
            family: String::new(),
            size: MEDIUM_PX,
            weight: FontWeight::NORMAL,
            style: FontStyle::Normal,
            width: FontWidth::NORMAL,
            letter: 0.0,
            word: 0.0,
            anchor: Anchor::Start,
            preserve: false,
            hidden: false,
            unsupported: Vec::new(),
        }
    }

    /// O estilo de `node` filho de um elemento com estilo `self`.
    pub(crate) fn child(&self, node: Node, sheet: &StyleSheet) -> Self {
        let d = declared(node, sheet);
        let get = |p: &str| {
            d.iter()
                .rev()
                .find(|(k, _)| k == p)
                .map(|(_, v)| v.as_str())
        };
        let mut s = self.clone();
        s.hidden = false;
        s.unsupported.retain(|p| INHERITED_GAPS.contains(p));
        if let Some(v) = get("font-family").filter(|v| *v != "inherit") {
            s.family = v.to_owned();
        }
        if let Some(v) = get("font-size") {
            s.size = font_size(v, self.size).unwrap_or(self.size);
        }
        if let Some(v) = get("font-weight") {
            s.weight = font_weight(v, self.weight);
        }
        if let Some(v) = get("font-style") {
            s.style = FontStyle::parse_css(v).unwrap_or(self.style);
        }
        if let Some(v) = get("font-stretch") {
            s.width = FontWidth::parse_css(v).unwrap_or(self.width);
        }
        if let Some(v) = get("letter-spacing") {
            s.letter = spacing(v, s.size).unwrap_or(self.letter);
        }
        if let Some(v) = get("word-spacing") {
            s.word = spacing(v, s.size).unwrap_or(self.word);
        }
        match get("text-anchor") {
            Some("start") => s.anchor = Anchor::Start,
            Some("middle") => s.anchor = Anchor::Middle,
            Some("end") => s.anchor = Anchor::End,
            _ => {}
        }
        let xml_space = node.attribute((usvg::roxmltree::NS_XML_URI, "space"));
        match (xml_space, get("white-space")) {
            (_, Some("pre" | "pre-wrap" | "break-spaces")) | (Some("preserve"), _) => {
                s.preserve = true;
            }
            (_, Some("normal" | "nowrap" | "pre-line")) | (Some("default"), _) => {
                s.preserve = false;
            }
            _ => {}
        }
        s.hidden = get("display") == Some("none");
        for (p, ok) in GAPS {
            match get(p) {
                Some(v) if !ok.contains(&v) => {
                    if !s.unsupported.contains(p) {
                        s.unsupported.push(p);
                    }
                }
                Some(_) => s.unsupported.retain(|q| q != p),
                None => {}
            }
        }
        s
    }
}

/// As propriedades que o desenho não segue, com os valores que equivalem a não as ter.
const GAPS: &[(&str, &[&str])] = &[
    ("writing-mode", &["lr", "lr-tb", "horizontal-tb", "inherit"]),
    ("direction", &["ltr", "inherit"]),
    ("font-variant", &["normal", "inherit"]),
    ("dominant-baseline", &["auto", "alphabetic", "inherit"]),
    ("alignment-baseline", &["auto", "baseline", "alphabetic"]),
    ("baseline-shift", &["baseline", "0", "0px"]),
    ("text-decoration", &["none"]),
];
/// As de `GAPS` que se herdam (as outras valem só no elemento).
const INHERITED_GAPS: &[&str] = &[
    "writing-mode",
    "direction",
    "font-variant",
    "dominant-baseline",
];

/// As declarações de texto de `node`, da mais fraca à mais forte.
fn declared(node: Node, sheet: &StyleSheet) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = node
        .attributes()
        .filter(|a| a.namespace().is_none() && TEXT_PROPS.contains(&a.name()))
        .map(|a| (a.name().to_owned(), a.value().trim().to_owned()))
        .collect();
    let mut put = |name: &str, value: &str| {
        if name == "font" {
            expand_font(value, &mut out);
        } else if TEXT_PROPS.contains(&name) {
            out.push((name.to_owned(), value.trim().to_owned()));
        }
    };
    for rule in &sheet.rules {
        if rule.selector.matches(&El(node)) {
            for d in &rule.declarations {
                put(d.name, d.value);
            }
        }
    }
    if let Some(v) = node.attribute("style") {
        for d in simplecss::DeclarationTokenizer::from(v) {
            put(d.name, d.value);
        }
    }
    out
}

/// As declarações das regras do `<style>` que casam com `node`, em CSS (`a:b;`) — o `<g>` que o
/// substitui leva-as no `style`, para as regras por TIPO (`text {…}`) continuarem a valer.
pub(crate) fn rule_css(node: Node, sheet: &StyleSheet) -> String {
    let mut s = String::new();
    for rule in &sheet.rules {
        if rule.selector.matches(&El(node)) {
            for d in &rule.declarations {
                s.push_str(d.name);
                s.push(':');
                s.push_str(d.value);
                s.push(';');
            }
        }
    }
    s
}

/// O `<style>` do documento (a mesma recolha que o usvg faz).
pub(crate) fn sheet<'a>(doc: &'a usvg::roxmltree::Document<'a>) -> StyleSheet<'a> {
    let mut sheet = StyleSheet::new();
    for n in doc.descendants().filter(|n| n.has_tag_name("style")) {
        if matches!(n.attribute("type"), None | Some("text/css"))
            && let Some(t) = n.text()
        {
            sheet.parse_more(t);
        }
    }
    sheet
}

fn expand_font(v: &str, out: &mut Vec<(String, String)>) {
    let Ok(f) = svgtypes::FontShorthand::from_str(v) else {
        return;
    };
    let mut put =
        |k: &str, v: Option<&str>| out.push((k.to_owned(), v.unwrap_or("normal").to_owned()));
    put("font-style", f.font_style);
    put("font-variant", f.font_variant);
    put("font-weight", f.font_weight);
    put("font-stretch", f.font_stretch);
    put("font-size", Some(f.font_size));
    put("font-family", Some(f.font_family));
}

/// Um comprimento em px; `em`/`%` contra `em`, `ex` = meio `em` (a regra do CSS sem altura-x).
pub(crate) fn length(l: svgtypes::Length, em: f32, percent_of: f32) -> f32 {
    use svgtypes::LengthUnit as U;
    let n = l.number as f32;
    match l.unit {
        U::None | U::Px => n,
        U::Em => n * em,
        U::Ex => n * em / 2.0,
        U::In => n * 96.0,
        U::Cm => n * 96.0 / 2.54,
        U::Mm => n * 96.0 / 25.4,
        U::Pt => n * 4.0 / 3.0,
        U::Pc => n * 16.0,
        U::Percent => n * percent_of / 100.0,
    }
}

/// O `font-size` (as palavras-chave pela tabela do CSS Fonts 4 §2.5).
fn font_size(v: &str, parent: f32) -> Option<f32> {
    const KEYS: &[(&str, f32)] = &[
        ("xx-small", 3.0 / 5.0),
        ("x-small", 3.0 / 4.0),
        ("small", 8.0 / 9.0),
        ("medium", 1.0),
        ("large", 6.0 / 5.0),
        ("x-large", 3.0 / 2.0),
        ("xx-large", 2.0),
        ("xxx-large", 3.0),
    ];
    if let Some((_, k)) = KEYS.iter().find(|(n, _)| *n == v) {
        return Some(MEDIUM_PX * k);
    }
    match v {
        "smaller" => return Some(parent / 1.2),
        "larger" => return Some(parent * 1.2),
        _ => {}
    }
    let l: svgtypes::Length = v.parse().ok()?;
    Some(length(l, parent, parent)).filter(|s| *s >= 0.0)
}

/// O `font-weight`, com `bolder`/`lighter` pela tabela do CSS Fonts 4 §2.2.
fn font_weight(v: &str, parent: FontWeight) -> FontWeight {
    let p = parent.value();
    match v {
        "bolder" => FontWeight::new(if p < 350.0 {
            400.0
        } else if p < 550.0 {
            700.0
        } else {
            p.max(900.0)
        }),
        "lighter" => FontWeight::new(if p < 100.0 {
            p
        } else if p < 550.0 {
            100.0
        } else if p < 750.0 {
            400.0
        } else {
            700.0
        }),
        _ => FontWeight::parse_css(v).unwrap_or(parent),
    }
}

fn spacing(v: &str, em: f32) -> Option<f32> {
    if v == "normal" {
        return Some(0.0);
    }
    let l: svgtypes::Length = v.parse().ok()?;
    Some(length(l, em, em))
}

struct El<'a, 'i>(Node<'a, 'i>);

impl simplecss::Element for El<'_, '_> {
    fn parent_element(&self) -> Option<Self> {
        self.0.parent_element().map(El)
    }
    fn prev_sibling_element(&self) -> Option<Self> {
        self.0.prev_sibling_element().map(El)
    }
    fn has_local_name(&self, name: &str) -> bool {
        self.0.tag_name().name() == name
    }
    fn attribute_matches(&self, name: &str, op: simplecss::AttributeOperator) -> bool {
        self.0.attribute(name).is_some_and(|v| op.matches(v))
    }
    fn pseudo_class_matches(&self, class: simplecss::PseudoClass) -> bool {
        matches!(class, simplecss::PseudoClass::FirstChild) && self.prev_sibling_element().is_none()
    }
}

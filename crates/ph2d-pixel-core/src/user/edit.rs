//! **A tabela `name = value` do cartão User Data** (P12 U, `P12_plano.md` §9.5): o que a mão muda no
//! mapa do artista (`""`). Quatro tipos editam-se (Bool, Int, Float, Text); os outros do `.ase` lêem-se
//! e atravessam a ida e volta intactos.

use super::{PropValue, UserData};

/// Os quatro tipos que a tela edita.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropKind {
    Bool,
    Int,
    Float,
    Text,
}

impl PropKind {
    /// O seguinte no chip (clicar roda Bool → Int → Float → Text).
    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Bool => Self::Int,
            Self::Int => Self::Float,
            Self::Float => Self::Text,
            Self::Text => Self::Bool,
        }
    }

    /// O valor de `t` neste tipo; `None` = o texto não é um valor dele.
    #[must_use]
    pub fn parse(self, t: &str) -> Option<PropValue> {
        let t = t.trim_matches(|c: char| self != Self::Text && c.is_whitespace());
        Some(match self {
            Self::Bool => PropValue::Bool(match t.to_ascii_lowercase().as_str() {
                "true" | "1" | "yes" | "on" => true,
                "false" | "0" | "no" | "off" | "" => false,
                _ => return None,
            }),
            Self::Int => {
                let n: i64 = if t.is_empty() { 0 } else { t.parse().ok()? };
                i32::try_from(n).map_or(PropValue::I64(n), PropValue::I32)
            }
            Self::Float => {
                let f: f64 = if t.is_empty() { 0.0 } else { t.parse().ok()? };
                if !f.is_finite() {
                    return None;
                }
                PropValue::F64(f)
            }
            Self::Text => PropValue::Str(t.to_owned()),
        })
    }
}

/// O tipo editável de um valor; `None` = só se lê (os outros quinze tipos do `.ase`).
#[must_use]
pub const fn kind_of(v: &PropValue) -> Option<PropKind> {
    match v {
        PropValue::Bool(_) => Some(PropKind::Bool),
        PropValue::I32(_) | PropValue::I64(_) => Some(PropKind::Int),
        PropValue::F64(_) => Some(PropKind::Float),
        PropValue::Str(_) => Some(PropKind::Text),
        _ => None,
    }
}

/// O valor escrito na célula.
#[must_use]
pub fn value_text(v: &PropValue) -> String {
    use PropValue as P;
    match v {
        P::Bool(b) => b.to_string(),
        P::I8(n) => n.to_string(),
        P::U8(n) => n.to_string(),
        P::I16(n) => n.to_string(),
        P::U16(n) => n.to_string(),
        P::I32(n) | P::Fixed(n) => n.to_string(),
        P::U32(n) => n.to_string(),
        P::I64(n) => n.to_string(),
        P::U64(n) => n.to_string(),
        P::F32(f) => f.to_string(),
        P::F64(f) => f.to_string(),
        P::Str(s) => s.clone(),
        P::Point(x, y) | P::Size(x, y) => format!("{x}, {y}"),
        P::Rect(x, y, w, h) => format!("{x}, {y}, {w}, {h}"),
        P::Vector(v) => format!("[{}]", v.len()),
        P::Map(m) => format!("{{{}}}", m.len()),
        P::Uuid(u) => u.iter().map(|b| format!("{b:02x}")).collect(),
    }
}

/// O nome do tipo no chip, como o `.ase` o chama (`docs/PixelArt/P12_oraculos.md` O17).
#[must_use]
pub const fn type_name(v: &PropValue) -> &'static str {
    use PropValue as P;
    match v {
        P::Bool(_) => "Bool",
        P::I32(_) | P::I64(_) => "Int",
        P::F64(_) => "Float",
        P::Str(_) => "Text",
        P::I8(_) => "Int8",
        P::U8(_) => "UInt8",
        P::I16(_) => "Int16",
        P::U16(_) => "UInt16",
        P::U32(_) => "UInt32",
        P::U64(_) => "UInt64",
        P::Fixed(_) => "Fixed",
        P::F32(_) => "Float32",
        P::Point(..) => "Point",
        P::Size(..) => "Size",
        P::Rect(..) => "Rect",
        P::Vector(_) => "Vector",
        P::Map(_) => "Map",
        P::Uuid(_) => "UUID",
    }
}

/// Um gesto sobre o user data de um dono.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserEdit {
    Color([u8; 4]),
    Text(String),
    /// `+`: uma linha nova `property` (`property 2`…), Text vazio.
    Add,
    /// `×`.
    Remove(String),
    Rename(String, String),
    /// O chip: o valor converte-se, ou nasce o de omissão do tipo.
    Kind(String, PropKind),
    Value(String, String),
}

/// O user data depois do gesto; `None` = nada muda (nome repetido ou vazio, valor que não é do tipo,
/// linha só de leitura).
#[must_use]
pub fn apply_edit(u: &UserData, e: &UserEdit) -> Option<UserData> {
    let mut n = u.clone();
    let m = n.props.entry(String::new()).or_default();
    match e {
        UserEdit::Color(c) => n.color = *c,
        UserEdit::Text(t) => t.clone_into(&mut n.text),
        UserEdit::Add => {
            let nome = (1..)
                .map(|i| {
                    if i == 1 {
                        "property".to_owned()
                    } else {
                        format!("property {i}")
                    }
                })
                .find(|k| !m.contains_key(k))?;
            m.insert(nome, PropValue::Str(String::new()));
        }
        UserEdit::Remove(k) => {
            m.remove(k)?;
        }
        UserEdit::Rename(de, para) => {
            if para.is_empty() || m.contains_key(para) {
                return None;
            }
            let v = m.remove(de)?;
            m.insert(para.clone(), v);
        }
        UserEdit::Kind(k, kind) => {
            let v = m.get_mut(k)?;
            kind_of(v)?;
            *v = kind.parse(&value_text(v)).or_else(|| kind.parse(""))?;
        }
        UserEdit::Value(k, t) => {
            let v = m.get_mut(k)?;
            *v = kind_of(v)?.parse(t)?;
        }
    }
    if n.props
        .get("")
        .is_some_and(std::collections::BTreeMap::is_empty)
    {
        n.props.remove("");
    }
    (n != *u).then_some(n)
}

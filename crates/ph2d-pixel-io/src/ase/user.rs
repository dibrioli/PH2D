//! O user data entre o documento e o `.ase` (P12): cor, texto e os mapas de propriedades.
//!
//! O mapa do artista é a chave `0` do ficheiro e `""` no documento; os outros vão pelo nome da
//! extensão, que o `0x2008` liga a um id (tipo 3). Um id sem entrada lê-se como `#<id>`.

use std::collections::BTreeSet;

use ph2d_aseprite::{AseExternalFile, AseProp, AsePropMap, AseUserData};
use ph2d_pixel_core::{PropMap, PropValue, Sprite, UserData};

/// O tipo de entrada do `0x2008` que dá nome às chaves dos mapas.
const KIND_EXTENSION: u8 = 3;

fn valor_de(p: &AseProp) -> PropValue {
    match p {
        AseProp::Bool(v) => PropValue::Bool(*v),
        AseProp::I8(v) => PropValue::I8(*v),
        AseProp::U8(v) => PropValue::U8(*v),
        AseProp::I16(v) => PropValue::I16(*v),
        AseProp::U16(v) => PropValue::U16(*v),
        AseProp::I32(v) => PropValue::I32(*v),
        AseProp::U32(v) => PropValue::U32(*v),
        AseProp::I64(v) => PropValue::I64(*v),
        AseProp::U64(v) => PropValue::U64(*v),
        AseProp::Fixed(v) => PropValue::Fixed(*v),
        AseProp::F32(v) => PropValue::F32(*v),
        AseProp::F64(v) => PropValue::F64(*v),
        AseProp::Str(v) => PropValue::Str(v.clone()),
        AseProp::Point(a, b) => PropValue::Point(*a, *b),
        AseProp::Size(a, b) => PropValue::Size(*a, *b),
        AseProp::Rect(a, b, c, d) => PropValue::Rect(*a, *b, *c, *d),
        AseProp::Vector(v) => PropValue::Vector(v.iter().map(valor_de).collect()),
        AseProp::Map(m) => PropValue::Map(mapa_de(m)),
        AseProp::Uuid(u) => PropValue::Uuid(*u),
    }
}

fn mapa_de(m: &[(String, AseProp)]) -> PropMap {
    m.iter().map(|(k, v)| (k.clone(), valor_de(v))).collect()
}

fn valor_saida(p: &PropValue) -> AseProp {
    match p {
        PropValue::Bool(v) => AseProp::Bool(*v),
        PropValue::I8(v) => AseProp::I8(*v),
        PropValue::U8(v) => AseProp::U8(*v),
        PropValue::I16(v) => AseProp::I16(*v),
        PropValue::U16(v) => AseProp::U16(*v),
        PropValue::I32(v) => AseProp::I32(*v),
        PropValue::U32(v) => AseProp::U32(*v),
        PropValue::I64(v) => AseProp::I64(*v),
        PropValue::U64(v) => AseProp::U64(*v),
        PropValue::Fixed(v) => AseProp::Fixed(*v),
        PropValue::F32(v) => AseProp::F32(*v),
        PropValue::F64(v) => AseProp::F64(*v),
        PropValue::Str(v) => AseProp::Str(v.clone()),
        PropValue::Point(a, b) => AseProp::Point(*a, *b),
        PropValue::Size(a, b) => AseProp::Size(*a, *b),
        PropValue::Rect(a, b, c, d) => AseProp::Rect(*a, *b, *c, *d),
        PropValue::Vector(v) => AseProp::Vector(v.iter().map(valor_saida).collect()),
        PropValue::Map(m) => AseProp::Map(mapa_saida(m)),
        PropValue::Uuid(u) => AseProp::Uuid(*u),
    }
}

fn mapa_saida(m: &PropMap) -> Vec<(String, AseProp)> {
    m.iter().map(|(k, v)| (k.clone(), valor_saida(v))).collect()
}

/// O user data do ficheiro no documento. Sem chunk = o de omissão.
pub(super) fn de(u: Option<&AseUserData>, ext: &[AseExternalFile]) -> UserData {
    let Some(u) = u else {
        return UserData::default();
    };
    let nome = |key: u32| {
        if key == 0 {
            return String::new();
        }
        ext.iter()
            .find(|e| e.id == key && e.kind == KIND_EXTENSION)
            .map_or_else(|| format!("#{key}"), |e| e.name.clone())
    };
    UserData {
        color: u.color.unwrap_or_default(),
        text: u.text.clone().unwrap_or_default(),
        props: u
            .maps
            .iter()
            .map(|m| (nome(m.key), mapa_de(&m.props)))
            .collect(),
    }
}

/// As extensões que os mapas do documento nomeiam, com ids `1..` em ordem de nome.
pub(super) fn extensoes(s: &Sprite) -> Vec<AseExternalFile> {
    let mut nomes = BTreeSet::new();
    let mut junta = |u: &UserData| {
        nomes.extend(u.props.keys().filter(|k| !k.is_empty()).cloned());
    };
    junta(s.user());
    s.layers().iter().for_each(|l| junta(&l.user));
    s.entries().for_each(|(_, c)| junta(&c.user));
    s.tags().iter().for_each(|t| junta(&t.user));
    s.tilesets().iter().for_each(|t| junta(&t.user));
    nomes
        .into_iter()
        .zip(1..)
        .map(|(name, id)| AseExternalFile {
            id,
            kind: KIND_EXTENSION,
            name,
        })
        .collect()
}

/// O user data do documento no ficheiro; `None` = o de omissão (nenhum chunk).
pub(super) fn saida(u: &UserData, ext: &[AseExternalFile]) -> Option<AseUserData> {
    if u.is_empty() {
        return None;
    }
    let key = |nome: &str| {
        if nome.is_empty() {
            0
        } else {
            ext.iter().find(|e| e.name == nome).map_or(0, |e| e.id)
        }
    };
    Some(AseUserData {
        text: (!u.text.is_empty()).then(|| u.text.clone()),
        color: (u.color != [0; 4]).then_some(u.color),
        maps: u
            .props
            .iter()
            .map(|(k, m)| AsePropMap {
                key: key(k),
                props: mapa_saida(m),
            })
            .collect(),
    })
}

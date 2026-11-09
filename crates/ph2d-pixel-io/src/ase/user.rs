//! O user data entre o documento e o `.ase` (P12): cor, texto e os mapas de propriedades.
//!
//! O mapa do artista é a chave `0` do ficheiro e `""` no documento; os outros vão pelo nome da
//! extensão, que o `0x2008` liga a um id (tipo 3). Um id sem entrada lê-se como `#<id>` e volta
//! ao ficheiro com o MESMO id e sem extensão inventada; um nome que se leria assim (`#5`, `#`,
//! `##x`, ou o nome vazio) ganha um `#` à frente. A chave do documento e o id/nome são uma bijecção.

use std::collections::BTreeSet;

use ph2d_aseprite::{AseExternalFile, AseProp, AsePropMap, AseUserData};
use ph2d_pixel_core::{PropMap, PropValue, Sprite, UserData};

/// O tipo de entrada do `0x2008` que dá nome às chaves dos mapas.
const KIND_EXTENSION: u8 = 3;

/// O que uma chave do documento diz do ficheiro.
#[derive(Debug, PartialEq, Eq)]
enum Chave<'a> {
    /// Um id sem extensão (o `0` é o mapa do artista).
    Id(u32),
    /// O nome de uma extensão.
    Nome(&'a str),
}

fn chave(k: &str) -> Chave<'_> {
    if k.is_empty() {
        return Chave::Id(0);
    }
    let Some(resto) = k.strip_prefix('#') else {
        return Chave::Nome(k);
    };
    if resto.is_empty() {
        return Chave::Nome("");
    }
    if resto.starts_with('#') {
        return Chave::Nome(resto);
    }
    match resto.parse::<u32>() {
        Ok(n) if n != 0 && n.to_string() == resto => Chave::Id(n),
        _ => Chave::Nome(k),
    }
}

/// A chave do documento de uma extensão chamada `nome` (o inverso de [`chave`]).
fn chave_do_nome(nome: &str) -> String {
    if chave(nome) == Chave::Nome(nome) {
        nome.to_owned()
    } else {
        format!("#{nome}")
    }
}

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
    let mut props = ph2d_pixel_core::Properties::new();
    for m in &u.maps {
        let bruto = || {
            if m.key == 0 {
                String::new()
            } else {
                format!("#{}", m.key)
            }
        };
        // Dois ids com o mesmo nome: o segundo fica pelo id (não se fundem).
        let k = ext
            .iter()
            .find(|e| e.id == m.key && e.kind == KIND_EXTENSION && m.key != 0)
            .map(|e| chave_do_nome(&e.name))
            .filter(|k| !props.contains_key(k))
            .unwrap_or_else(bruto);
        props.entry(k).or_default().extend(mapa_de(&m.props));
    }
    UserData {
        color: u.color.unwrap_or_default(),
        text: u.text.clone().unwrap_or_default(),
        props,
    }
}

/// As extensões que os mapas do documento nomeiam, com ids `1..` em ordem de nome, saltando os
/// ids que o documento guarda sem extensão (`#<id>`).
pub(super) fn extensoes(s: &Sprite) -> Vec<AseExternalFile> {
    let mut todos = vec![s.user()];
    todos.extend(s.layers().iter().map(|l| &l.user));
    todos.extend(s.entries().map(|(_, c)| &c.user));
    todos.extend(s.tags().iter().map(|t| &t.user));
    todos.extend(s.tilesets().iter().map(|t| &t.user));
    todos.extend(s.slices().iter().map(|c| &c.user));
    extensoes_de(todos)
}

fn extensoes_de<'a>(todos: impl IntoIterator<Item = &'a UserData>) -> Vec<AseExternalFile> {
    let mut nomes = BTreeSet::new();
    let mut brutos = BTreeSet::new();
    let mut junta = |u: &UserData| {
        for k in u.props.keys() {
            match chave(k) {
                Chave::Id(n) => {
                    brutos.insert(n);
                }
                Chave::Nome(n) => {
                    nomes.insert(n.to_owned());
                }
            }
        }
    };
    todos.into_iter().for_each(&mut junta);
    nomes
        .into_iter()
        .zip((1..).filter(|id| !brutos.contains(id)))
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
    let key = |k: &str| match chave(k) {
        Chave::Id(n) => n,
        Chave::Nome(nome) => ext.iter().find(|e| e.name == nome).map_or(0, |e| e.id),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn ext(id: u32, kind: u8, name: &str) -> AseExternalFile {
        AseExternalFile {
            id,
            kind,
            name: name.into(),
        }
    }

    fn mapa(key: u32) -> AsePropMap {
        AsePropMap {
            key,
            props: vec![(format!("v{key}"), AseProp::U32(key))],
        }
    }

    #[test]
    fn unknown_map_ids_round_trip_with_the_same_id_and_nothing_merges() {
        let file = AseUserData {
            text: None,
            color: None,
            maps: [0, 3, 4, 5, 6, 7, 9].into_iter().map(mapa).collect(),
        };
        let lidas = [
            ext(3, KIND_EXTENSION, "#7"),
            ext(4, KIND_EXTENSION, "dup"),
            ext(5, KIND_EXTENSION, "dup"),
            ext(6, KIND_EXTENSION, ""),
            ext(7, 0, "palette.aseprite"),
            ext(9, KIND_EXTENSION, "x"),
        ];
        let doc = de(Some(&file), &lidas);
        let chaves: Vec<&str> = doc.props.keys().map(String::as_str).collect();
        assert_eq!(chaves, ["", "#", "##7", "#5", "#7", "dup", "x"]);
        let novas = extensoes_de([&doc]);
        let ids: BTreeSet<u32> = novas.iter().map(|e| e.id).collect();
        assert!(!ids.contains(&5) && !ids.contains(&7), "{novas:?}");
        let nomes: Vec<&str> = novas.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(nomes, ["", "#7", "dup", "x"]);
        let saiu = saida(&doc, &novas).expect("tem mapas");
        assert_eq!(saiu.maps.len(), file.maps.len());
        let por_valor = |u: &AseUserData, v: u32| {
            u.maps
                .iter()
                .find(|m| m.props == mapa(v).props)
                .map(|m| m.key)
        };
        assert_eq!(por_valor(&saiu, 5), Some(5));
        assert_eq!(por_valor(&saiu, 7), Some(7));
        assert_eq!(por_valor(&saiu, 0), Some(0));
        assert_eq!(de(Some(&saiu), &novas), doc);
    }
}

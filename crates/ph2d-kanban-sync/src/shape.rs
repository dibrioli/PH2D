//! Os campos que são CONJUNTOS (um registo por elemento): as pessoas, as etiquetas e os campos
//! personalizados do cartão. O resto é um registo só.

use std::collections::BTreeMap;

use ph2d_kanban_model::{FieldId, FieldValue, LabelId, Member};
use serde::Serialize;
use serde::de::DeserializeOwned;

/// `(chave do elemento, valor)`: num conjunto o valor é vazio; num mapa é o valor da chave.
pub(crate) type Entry = (Vec<u8>, Vec<u8>);

pub(crate) struct Shape {
    pub split: fn(&[u8]) -> Option<Vec<Entry>>,
    pub join: fn(&[(&[u8], &[u8])]) -> Option<Vec<u8>>,
    /// A ordem do conjunto vem da escrita (a do mapa é a da chave).
    pub ordered: bool,
}

fn enc<T: Serialize>(v: &T) -> Vec<u8> {
    postcard::to_allocvec(v).expect("postcard escreve qualquer valor do modelo")
}

fn exact<T: DeserializeOwned>(b: &[u8]) -> Option<T> {
    let (v, rest) = postcard::take_from_bytes::<T>(b).ok()?;
    rest.is_empty().then_some(v)
}

fn split_set<T: Serialize + DeserializeOwned>(b: &[u8]) -> Option<Vec<Entry>> {
    let v: Vec<T> = exact(b)?;
    Some(v.iter().map(|x| (enc(x), Vec::new())).collect())
}

fn join_set<T: Serialize + DeserializeOwned>(es: &[(&[u8], &[u8])]) -> Option<Vec<u8>> {
    let v: Option<Vec<T>> = es.iter().map(|(k, _)| exact(k)).collect();
    Some(enc(&v?))
}

fn split_map<K, V>(b: &[u8]) -> Option<Vec<Entry>>
where
    K: Serialize + DeserializeOwned + Ord,
    V: Serialize + DeserializeOwned,
{
    let m: BTreeMap<K, V> = exact(b)?;
    Some(m.iter().map(|(k, v)| (enc(k), enc(v))).collect())
}

fn join_map<K, V>(es: &[(&[u8], &[u8])]) -> Option<Vec<u8>>
where
    K: Serialize + DeserializeOwned + Ord,
    V: Serialize + DeserializeOwned,
{
    let m: Option<BTreeMap<K, V>> = es
        .iter()
        .map(|(k, v)| Some((exact(k)?, exact(v)?)))
        .collect();
    Some(enc(&m?))
}

/// O feitio do campo `name` cujos bytes são `b` (`None` = um registo só). Pelo nome E pelos bytes:
/// um campo com o mesmo nome e outro tipo não se lê como conjunto.
pub(crate) fn shape_of(name: &str, b: &[u8]) -> Option<Shape> {
    let s = match name {
        "labels" => Shape {
            split: split_set::<LabelId>,
            join: join_set::<LabelId>,
            ordered: true,
        },
        "members" | "votes" | "watchers" => Shape {
            split: split_set::<Member>,
            join: join_set::<Member>,
            ordered: true,
        },
        "fields" => Shape {
            split: split_map::<FieldId, FieldValue>,
            join: join_map::<FieldId, FieldValue>,
            ordered: false,
        },
        _ => return None,
    };
    (s.split)(b).is_some().then_some(s)
}

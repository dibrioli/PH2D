//! **Os campos de um registo** — a divisão que o desfazer por campo e a réplica ao vivo (W5,
//! `ph2d-kanban-sync`) partilham: UMA divisão, lida pelo serde, para nenhum campo novo ficar de fora.
//!
//! Um registo em postcard é o índice da variante seguido dos campos, um a seguir ao outro, sem nomes.
//! Capturar cada campo à parte e voltar a juntá-los é, portanto, exacto. O `meta` sai sem a versão e
//! o desempate (o que fica é a lápide, como em [`Record::same_content`]); o `version` do cabeçalho
//! não sai.

use std::fmt;

use serde::Serialize;
use serde::ser::{self, Impossible, SerializeStruct, Serializer};

use crate::kanban::Header;
use crate::ops::Record;
use crate::records::Meta;

/// Os campos de um valor: o índice da variante (registo) e `(nome, bytes)` pela ordem da declaração.
#[derive(Default)]
struct Parts {
    variant: Option<u32>,
    fields: Vec<(&'static str, Vec<u8>)>,
}

#[derive(Debug)]
struct Nope;

impl fmt::Display for Nope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("não é um registo de campos")
    }
}

impl std::error::Error for Nope {}

impl ser::Error for Nope {
    fn custom<T: fmt::Display>(_: T) -> Self {
        Nope
    }
}

macro_rules! refuse {
    ($($f:ident($($t:ty),*)),* $(,)?) => {$(
        fn $f(self, $(_: $t),*) -> Result<(), Nope> {
            Err(Nope)
        }
    )*};
}

impl<'a> Serializer for &'a mut Parts {
    type Ok = ();
    type Error = Nope;
    type SerializeSeq = Impossible<(), Nope>;
    type SerializeTuple = Impossible<(), Nope>;
    type SerializeTupleStruct = Impossible<(), Nope>;
    type SerializeTupleVariant = Impossible<(), Nope>;
    type SerializeMap = Impossible<(), Nope>;
    type SerializeStruct = Self;
    type SerializeStructVariant = Impossible<(), Nope>;

    refuse!(
        serialize_bool(bool),
        serialize_i8(i8),
        serialize_i16(i16),
        serialize_i32(i32),
        serialize_i64(i64),
        serialize_u8(u8),
        serialize_u16(u16),
        serialize_u32(u32),
        serialize_u64(u64),
        serialize_f32(f32),
        serialize_f64(f64),
        serialize_char(char),
        serialize_str(&str),
        serialize_bytes(&[u8]),
        serialize_none(),
        serialize_unit(),
        serialize_unit_struct(&'static str),
        serialize_unit_variant(&'static str, u32, &'static str),
    );

    fn serialize_some<T: ?Sized + Serialize>(self, _: &T) -> Result<(), Nope> {
        Err(Nope)
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        _: &T,
    ) -> Result<(), Nope> {
        Err(Nope)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        index: u32,
        _: &'static str,
        value: &T,
    ) -> Result<(), Nope> {
        self.variant = Some(index);
        value.serialize(self)
    }

    fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq, Nope> {
        Err(Nope)
    }

    fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple, Nope> {
        Err(Nope)
    }

    fn serialize_tuple_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleStruct, Nope> {
        Err(Nope)
    }

    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleVariant, Nope> {
        Err(Nope)
    }

    fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap, Nope> {
        Err(Nope)
    }

    fn serialize_struct(self, _: &'static str, _: usize) -> Result<Self, Nope> {
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStructVariant, Nope> {
        Err(Nope)
    }
}

impl SerializeStruct for &mut Parts {
    type Ok = ();
    type Error = Nope;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Nope> {
        let bytes = postcard::to_allocvec(value).map_err(|_| Nope)?;
        self.fields.push((key, bytes));
        Ok(())
    }

    fn end(self) -> Result<(), Nope> {
        Ok(())
    }
}

fn parts<T: Serialize>(v: &T) -> Parts {
    let mut p = Parts::default();
    v.serialize(&mut p)
        .expect("um registo do Kanban é uma variante com uma struct de campos");
    p
}

/// Junta as partes com `changes` por cima e lê o valor. `None` = bytes que não são deste campo.
fn rebuild<T: serde::de::DeserializeOwned>(p: Parts, changes: &[(&str, &[u8])]) -> Option<T> {
    let mut bytes = match p.variant {
        Some(i) => postcard::to_allocvec(&i).ok()?,
        None => Vec::new(),
    };
    for (name, mut b) in p.fields {
        if let Some((_, new)) = changes.iter().find(|(n, _)| *n == name) {
            b = new.to_vec();
        }
        bytes.extend(b);
    }
    // Um campo com bytes a mais ou a menos desalinha os seguintes: só vale se a leitura os gasta todos.
    let (v, rest) = postcard::take_from_bytes::<T>(&bytes).ok()?;
    rest.is_empty().then_some(v)
}

const META: &str = "meta";

impl Record {
    /// ⭐ Os campos deste registo, `(nome, bytes postcard)`, pela ordem da declaração. O `meta` leva
    /// só a lápide (versão e desempate a zero).
    #[must_use]
    pub fn fields(&self) -> Vec<(&'static str, Vec<u8>)> {
        let mut p = parts(self);
        for (name, b) in &mut p.fields {
            if *name == META
                && let Ok(m) = postcard::from_bytes::<Meta>(b)
            {
                *b = postcard::to_allocvec(&Meta {
                    version: 0,
                    nonce: 0,
                    deleted: m.deleted,
                })
                .expect("postcard escreve o meta");
            }
        }
        p.fields
    }

    /// Este registo com os campos `changes` (nome, bytes de [`Self::fields`]) postos. `None` = uns
    /// bytes que não são desse campo.
    #[must_use]
    pub fn with_fields(&self, changes: &[(&str, &[u8])]) -> Option<Record> {
        rebuild(parts(self), changes)
    }
}

impl Header {
    /// Os campos do cabeçalho como [`Record::fields`], sem a `version`.
    #[must_use]
    pub fn fields(&self) -> Vec<(&'static str, Vec<u8>)> {
        let mut p = parts(self);
        p.fields.retain(|(n, _)| *n != "version");
        p.fields
    }

    /// O cabeçalho com os campos `changes` postos (a `version` fica).
    #[must_use]
    pub fn with_fields(&self, changes: &[(&str, &[u8])]) -> Option<Header> {
        rebuild(parts(self), changes)
    }
}

/// Os campos de `before` a repor sobre `now`: os que AINDA têm o valor que o gesto deixou (`was`) e
/// que o desfazer muda. Os que outro mexeu depois ficam como ele os deixou (ADR-0181 §3, por campo).
pub(crate) fn to_restore<'a>(
    now: &[(&'static str, Vec<u8>)],
    was: &[(&'static str, Vec<u8>)],
    before: &'a [(&'static str, Vec<u8>)],
) -> Vec<(&'static str, &'a [u8])> {
    before
        .iter()
        .filter_map(|(name, b)| {
            let n = now.iter().find(|(x, _)| x == name)?;
            let w = was.iter().find(|(x, _)| x == name)?;
            (n.1 == w.1 && n.1 != *b).then_some((*name, b.as_slice()))
        })
        .collect()
}

/// O campo da lápide.
pub(crate) fn meta_field<'a>(fields: &'a [(&'static str, Vec<u8>)]) -> Option<&'a [u8]> {
    fields
        .iter()
        .find(|(n, _)| *n == META)
        .map(|(_, b)| b.as_slice())
}

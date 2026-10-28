//! **Os ficheiros do Kanban por conteúdo** — imagens de capa e de fundo, ficheiros anexados.
//!
//! Os bytes vivem UMA vez por conteúdo no [`crate::KanbanSet`] (chave = sha-256 dos bytes tal como
//! chegaram), nunca no registo: um cartão que se move não arrasta uma fotografia. É a mesma chave do
//! `ph2d_board_model::FileId` (ADR-0184), sem depender do Quadro (ADR-0195 §7: o ciclo da W4).
//!
//! Um ficheiro nunca muda (a chave É o conteúdo) e não tem desfazer: tirar a capa é mudar o CARTÃO.
//! Quem grava leva só os que algum registo vivo ainda usa ([`crate::KanbanSet::files_in_use`]).

use std::fmt;
use std::sync::Arc;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sha2::{Digest, Sha256};

/// A identidade de um ficheiro: o sha-256 dos bytes. Em postcard são os mesmos 32 bytes do
/// `[u8; 32]` do formato v1.
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FileId(pub [u8; 32]);

impl FileId {
    /// A chave destes bytes.
    #[must_use]
    pub fn of(bytes: &[u8]) -> Self {
        Self(Sha256::digest(bytes).into())
    }

    /// Os 64 dígitos hexadecimais.
    #[must_use]
    pub fn hex(&self) -> String {
        self.0.iter().map(|b| format!("{b:02x}")).collect()
    }
}

impl fmt::Debug for FileId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FileId({}…)", &self.hex()[..12])
    }
}

/// Bytes partilhados sem cópia (clonar o conjunto não duplica uma fotografia). Em postcard são os
/// bytes de um `Vec<u8>`.
#[derive(Clone, PartialEq, Eq)]
pub struct Blob(pub Arc<[u8]>);

impl fmt::Debug for Blob {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Blob({} B)", self.0.len())
    }
}

impl Serialize for Blob {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_bytes(&self.0)
    }
}

impl<'de> Deserialize<'de> for Blob {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct V;
        impl serde::de::Visitor<'_> for V {
            type Value = Blob;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("bytes")
            }
            fn visit_bytes<E>(self, b: &[u8]) -> Result<Blob, E> {
                Ok(Blob(Arc::from(b)))
            }
            fn visit_byte_buf<E>(self, b: Vec<u8>) -> Result<Blob, E> {
                Ok(Blob(Arc::from(b)))
            }
        }
        d.deserialize_byte_buf(V)
    }
}

/// Um ficheiro do Kanban: os bytes como chegaram e, se é uma imagem, o tamanho NATURAL em píxeis
/// (o de quem a descodificou ao entrar — a capa reserva o espaço sem esperar pela descodificação).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KanbanFile {
    pub bytes: Blob,
    pub size: Option<[u32; 2]>,
}

impl KanbanFile {
    #[must_use]
    pub fn new(bytes: impl Into<Arc<[u8]>>, size: Option<[u32; 2]>) -> Self {
        Self {
            bytes: Blob(bytes.into()),
            size,
        }
    }

    /// A chave destes bytes (calcula o sha-256).
    #[must_use]
    pub fn id(&self) -> FileId {
        FileId::of(&self.bytes.0)
    }
}

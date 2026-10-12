//! **As imagens do quadro** (W14) — o elemento [`Image`] e os bytes que ele mostra.
//!
//! Os bytes vivem UMA vez por conteúdo no [`Board`] ([`Board::files`], chave = sha256 dos bytes tal
//! como chegaram: PNG, JPEG…), nunca no elemento: mover uma imagem reescreve o elemento inteiro (e,
//! ao vivo, a caixa), e os bytes ficavam a viajar em cada gesto. É o desenho do Excalidraw (`fileId`
//! no elemento, `files` na cena), medido no oráculo (`saidas/w14/`).
//!
//! O que é de um ficheiro de imagem não muda nunca (a chave É o conteúdo): juntar os de dois pares é
//! a união, e não há desfazer — tirar a imagem é apagar o ELEMENTO. Quem grava guarda só os que algum
//! elemento vivo ou alguma versão ainda mostra ([`Board::files_in_use`]).

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::sync::Arc;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sha2::{Digest, Sha256};

use crate::{Board, BoardDoc, ElementKind, Style};

/// A identidade de um ficheiro de imagem: o sha256 dos bytes.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FileId(pub [u8; 32]);

impl FileId {
    /// A chave destes bytes.
    #[must_use]
    pub fn of(bytes: &[u8]) -> Self {
        Self(Sha256::digest(bytes).into())
    }

    /// Os 64 dígitos hexadecimais (o `fileId` que escrevemos num `.excalidraw`).
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

/// Bytes partilhados sem cópia (um clone do quadro, do retrato ou da área de transferência não
/// duplica uma fotografia). Em `postcard` são os mesmos bytes de um `Vec<u8>`.
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
        impl<'de> serde::de::Visitor<'de> for V {
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
            fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut a: A) -> Result<Blob, A::Error> {
                let mut v = Vec::with_capacity(a.size_hint().unwrap_or(0));
                while let Some(b) = a.next_element::<u8>()? {
                    v.push(b);
                }
                Ok(Blob(Arc::from(v)))
            }
        }
        d.deserialize_byte_buf(V)
    }
}

/// Um ficheiro de imagem: os bytes como chegaram e o tamanho NATURAL em píxeis (o de quem os
/// descodificou ao entrar — o desenho e o recorte não esperam pela descodificação).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageFile {
    pub bytes: Blob,
    pub size: [u32; 2],
}

impl ImageFile {
    #[must_use]
    pub fn new(bytes: impl Into<Arc<[u8]>>, size: [u32; 2]) -> Self {
        Self {
            bytes: Blob(bytes.into()),
            size,
        }
    }

    /// A chave destes bytes (calcula o sha256).
    #[must_use]
    pub fn id(&self) -> FileId {
        FileId::of(&self.bytes.0)
    }

    /// O tipo MIME pelos bytes mágicos — o `mimeType` de um `.excalidraw` e o `data:` de um SVG.
    /// `None` = nenhum que um navegador mostre.
    ///
    /// W15: `image/svg+xml` = um ficheiro VECTORIAL (o tamanho natural é o do `<svg>`; o desenho
    /// rasteriza-o ao tamanho de cada nível).
    #[must_use]
    pub fn mime(&self) -> Option<&'static str> {
        mime_of(&self.bytes.0)
    }
}

/// O tipo MIME de uns bytes de imagem, pelos bytes mágicos.
#[must_use]
pub fn mime_of(b: &[u8]) -> Option<&'static str> {
    let at = |i: usize, m: &[u8]| b.get(i..i + m.len()) == Some(m);
    Some(if at(0, b"\x89PNG\r\n\x1a\n") {
        "image/png"
    } else if at(0, &[0xff, 0xd8, 0xff]) {
        "image/jpeg"
    } else if at(0, b"GIF87a") || at(0, b"GIF89a") {
        "image/gif"
    } else if at(0, b"RIFF") && at(8, b"WEBP") {
        "image/webp"
    } else if at(0, b"BM") {
        "image/bmp"
    } else if at(4, b"ftypavif") || at(4, b"ftypavis") {
        "image/avif"
    } else if is_svg(b) {
        "image/svg+xml"
    } else {
        return None;
    })
}

/// W15 — um SVG (texto): o 1.º elemento é `<svg`, depois do BOM, de espaços, da declaração `<?xml`,
/// de comentários e do `<!DOCTYPE`. Só se olha para o princípio (um SVG não precisa de ser lido
/// inteiro para se saber que o é).
fn is_svg(b: &[u8]) -> bool {
    let mut r = b.strip_prefix(b"\xef\xbb\xbf").unwrap_or(b);
    loop {
        r = r.trim_ascii_start();
        let skip = |r: &[u8], open: &[u8], close: &[u8]| -> Option<usize> {
            r.starts_with(open).then(|| {
                r.windows(close.len())
                    .position(|w| w == close)
                    .map(|i| i + close.len())
            })?
        };
        if let Some(n) = skip(r, b"<?", b"?>")
            .or_else(|| skip(r, b"<!--", b"-->"))
            .or_else(|| skip(r, b"<!DOCTYPE", b">"))
        {
            r = &r[n..];
            continue;
        }
        let Some(rest) = r.strip_prefix(b"<svg") else {
            return false;
        };
        return rest
            .first()
            .is_some_and(|c| c.is_ascii_whitespace() || *c == b'>' || *c == b'/');
    }
}

/// ⭐ A **porta de entrada** de uns bytes de imagem no quadro (soltar, colar, importar, um
/// `.excalidraw`): descodifica-os para saber o tamanho e devolve o ficheiro — o de um formato que um
/// navegador não mostra (TIFF, PSD…) já em PNG, para o SVG, o `.excalidraw` e o par ao vivo o
/// mostrarem. `None` = não é uma imagem que o app lê. Implementa-a quem tem o descodificador.
pub type Ingest<'a> = &'a dyn Fn(&[u8]) -> Option<ImageFile>;

/// Os ficheiros de um quadro, por conteúdo.
pub type Files = BTreeMap<FileId, ImageFile>;

/// A parte da imagem natural que se mostra, em píxeis da imagem (o `crop` do Excalidraw).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Crop {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

/// O filtro de cor de uma imagem (W15). ⚠️ Append-only: o discriminante vai para o ficheiro, e uma
/// variante nova no FIM não muda o formato.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ImageFilter {
    /// As cores do ficheiro.
    #[default]
    Original,
    /// Preto e branco: cada píxel vira a luminância relativa dele (Rec. 709 sobre a luz linear, a
    /// `LinearRgba::luminance` do `ph2d-color`), com o alfa intacto.
    Gray,
}

/// Uma imagem no quadro: estica-se à caixa `x, y, w, h` do elemento (sem manter a proporção — é o
/// `preserveAspectRatio="none"` do oráculo; quem redimensiona é que a mantém).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Image {
    pub file: FileId,
    /// A opacidade, os cantos redondos (`round`, a mesma regra de raio das formas) e a moldura (W15:
    /// `stroke`, `stroke_width`, `dash`, `sketch`, como no contorno de uma forma). O preenchimento não conta.
    pub style: Style,
    /// `None` = a imagem inteira.
    pub crop: Option<Crop>,
    /// Espelhada na horizontal e na vertical, à volta do centro da caixa (o `scale` `-1`).
    pub flip: [bool; 2],
    /// W15 (formato 13). `default`: o nosso elemento no `customData` de um `.excalidraw` da W14 não o
    /// tem (o JSON lê-se por nome; o postcard, posicional, lê-o sempre).
    #[serde(default)]
    pub filter: ImageFilter,
}

impl Image {
    /// Uma imagem inteira, sem espelho.
    #[must_use]
    pub fn new(file: FileId, style: Style) -> Self {
        Self {
            file,
            style,
            crop: None,
            flip: [false; 2],
            filter: ImageFilter::Original,
        }
    }

    /// O rectângulo da imagem natural (de tamanho `size`) que a caixa mostra: `[x, y, w, h]`.
    #[must_use]
    pub fn source(&self, size: [u32; 2]) -> [f64; 4] {
        match self.crop {
            Some(c) => [c.x, c.y, c.w, c.h],
            None => [0.0, 0.0, f64::from(size[0]), f64::from(size[1])],
        }
    }
}

impl BoardDoc {
    /// Os ficheiros que os elementos NÃO apagados mostram (os de um ramo recolhido incluídos).
    #[must_use]
    pub fn image_files(&self) -> BTreeSet<FileId> {
        self.live_with_hidden()
            .filter_map(|e| match &e.kind {
                ElementKind::Image(i) => Some(i.file),
                _ => None,
            })
            .collect()
    }
}

impl Board {
    /// Guarda o ficheiro (se ainda não está) e devolve a chave dele.
    pub fn put_file(&mut self, file: ImageFile) -> FileId {
        let id = file.id();
        self.files.entry(id).or_insert(file);
        id
    }

    /// O ficheiro com esta chave, se este quadro o tem.
    #[must_use]
    pub fn file(&self, id: FileId) -> Option<&ImageFile> {
        self.files.get(&id)
    }

    /// Os ficheiros que algum elemento refere — os APAGADOS incluídos (um desfazer, ao vivo, trá-lo de
    /// volta) — e as versões: o que o anfitrião manda a quem entra (os restos de colagens e de imagens
    /// que nunca ficaram não viajam).
    #[must_use]
    pub fn files_referenced(&self) -> BTreeSet<FileId> {
        let mut used: BTreeSet<FileId> = self
            .doc
            .elements
            .values()
            .filter_map(|e| e.image().map(|i| i.file))
            .collect();
        for v in &self.team.versions {
            used.extend(v.files.iter().copied());
        }
        used
    }

    /// Os ficheiros que alguma coisa do quadro ainda mostra: os elementos não apagados e as versões.
    #[must_use]
    pub fn files_in_use(&self) -> BTreeSet<FileId> {
        let mut used = self.doc.image_files();
        for v in &self.team.versions {
            used.extend(v.files.iter().copied());
        }
        used
    }
}

/// Os ficheiros de `files` que estão em `keep`, para gravar (o mapa do postcard precisa do
/// comprimento à cabeça — por isso a lista, e não um filtro).
pub(crate) struct Kept<'a>(pub(crate) Vec<(&'a FileId, &'a ImageFile)>);

impl<'a> Kept<'a> {
    pub(crate) fn of(files: &'a Files, keep: &BTreeSet<FileId>) -> Self {
        Self(files.iter().filter(|(k, _)| keep.contains(k)).collect())
    }
}

impl Serialize for Kept<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_map(self.0.iter().copied())
    }
}

#[cfg(test)]
#[path = "image_tests.rs"]
mod tests;

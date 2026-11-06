//! ⭐⭐ **Os SONS do projecto** (GEF 21 E2) — a biblioteca que o [`super::AudioSource2D::sound`]
//! nomeia.
//!
//! O som é um asset do projecto por referência, como a imagem (decisão do dono de 26/10): o
//! `.ph2dproj` leva os BYTES, e o componente guarda o NOME. ⚠️ **O nome e não o hash**, porque o
//! campo é lido e escrito pelo nome pelo script (`ph2d.set(obj, "sound", "moeda.ogg")`) e pelo
//! Auto Key — o mesmo idioma do alvo da câmera, que é o nome de um objecto.
//!
//! - **Um nome, um som.** Dois ficheiros com o mesmo nome e bytes diferentes recebem `moeda 2.ogg`.
//! - **O conteúdo decide a identidade:** importar os mesmos bytes outra vez devolve o nome que já lá
//!   está, e não uma cópia.
//! - ⛔ **Fora do `ProjectState`**, como os scripts: o undo não guarda bytes estáveis, e um Ctrl+Z
//!   não «desimporta».

use std::collections::BTreeMap;
use std::sync::Arc;

/// O endereço de um som pelo CONTEÚDO — o blake3 dos bytes, como o `AssetId` das imagens.
pub type SoundId = [u8; 32];

/// Um som do projecto: os bytes do ficheiro (codificados — Ogg, WAV…), nunca as amostras.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectSound {
    pub id: SoundId,
    pub bytes: Arc<[u8]>,
}

/// **A biblioteca de sons do projecto**, por nome.
#[derive(bevy_ecs::resource::Resource, Clone, Debug, Default, PartialEq, Eq)]
pub struct ProjectSounds {
    by_name: BTreeMap<String, ProjectSound>,
}

/// O blob não é uma biblioteca de sons legível.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoundsBlobError(pub String);

impl std::fmt::Display for SoundsBlobError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

const MAGIC: &[u8; 8] = b"PH2DSONS";
const VERSAO: u8 = 1;

impl ProjectSounds {
    /// O endereço de uns bytes.
    #[must_use]
    pub fn id_of(bytes: &[u8]) -> SoundId {
        *blake3::hash(bytes).as_bytes()
    }

    /// **Importa** uns bytes com o nome desejado e devolve o nome com que ficaram.
    ///
    /// Os mesmos bytes já lá ⇒ o nome que já têm. Nome ocupado por outro som ⇒ `stem N.ext`.
    pub fn add(&mut self, wanted: &str, bytes: Vec<u8>) -> String {
        let id = Self::id_of(&bytes);
        if let Some(name) = self.name_of(&id) {
            return name.to_owned();
        }
        let wanted = match wanted.trim() {
            "" => "Sound",
            w => w,
        };
        let name = self.free_name(wanted);
        self.by_name.insert(
            name.clone(),
            ProjectSound {
                id,
                bytes: bytes.into(),
            },
        );
        name
    }

    /// Este nome passa a ter estes bytes, substituindo o que lá estava: o nome é o que os objetos guardam.
    pub fn set(&mut self, name: &str, bytes: Vec<u8>) {
        self.by_name.insert(
            name.to_owned(),
            ProjectSound {
                id: Self::id_of(&bytes),
                bytes: bytes.into(),
            },
        );
    }

    /// O nome desejado, ou `stem N.ext` se ele já estiver ocupado.
    #[must_use]
    pub fn free_name(&self, wanted: &str) -> String {
        if !self.by_name.contains_key(wanted) {
            return wanted.to_owned();
        }
        let (stem, ext) = match wanted.rsplit_once('.') {
            Some((s, e)) if !s.is_empty() => (s, format!(".{e}")),
            _ => (wanted, String::new()),
        };
        (2..)
            .map(|n| format!("{stem} {n}{ext}"))
            .find(|c| !self.by_name.contains_key(c))
            .unwrap_or_else(|| wanted.to_owned())
    }

    /// O som com este nome.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&ProjectSound> {
        self.by_name.get(name.trim())
    }

    /// O nome do som com este conteúdo.
    #[must_use]
    pub fn name_of(&self, id: &SoundId) -> Option<&str> {
        self.by_name
            .iter()
            .find(|(_, s)| s.id == *id)
            .map(|(n, _)| n.as_str())
    }

    /// Os sons, por ordem de nome.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &ProjectSound)> {
        self.by_name.iter().map(|(n, s)| (n.as_str(), s))
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.by_name.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.by_name.is_empty()
    }

    /// O blob a gravar — vazio quando não há sons (o projecto sem sons é o de antes, ao byte).
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        if self.by_name.is_empty() {
            return Vec::new();
        }
        let corpo: Vec<(&str, &[u8])> = self.iter().map(|(n, s)| (n, &*s.bytes)).collect();
        let mut out = MAGIC.to_vec();
        out.push(VERSAO);
        out.extend(postcard::to_allocvec(&corpo).unwrap_or_default());
        out
    }

    /// Lê o blob de [`Self::to_bytes`]. Vazio = biblioteca vazia. ⚠️ O endereço é RECALCULADO: o
    /// ficheiro guarda os bytes, e a identidade é deles.
    ///
    /// # Errors
    /// Cabeçalho, versão ou corpo ilegíveis.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SoundsBlobError> {
        if bytes.is_empty() {
            return Ok(Self::default());
        }
        let corpo = bytes
            .strip_prefix(MAGIC.as_slice())
            .ok_or_else(|| SoundsBlobError("not a sound library".into()))?;
        let (&versao, corpo) = corpo
            .split_first()
            .ok_or_else(|| SoundsBlobError("sound library without a version".into()))?;
        if versao != VERSAO {
            return Err(SoundsBlobError(format!(
                "sound library version {versao}, this app reads {VERSAO}"
            )));
        }
        let lista: Vec<(String, Vec<u8>)> =
            postcard::from_bytes(corpo).map_err(|e| SoundsBlobError(e.to_string()))?;
        let mut out = Self::default();
        for (name, b) in lista {
            out.by_name.insert(
                name,
                ProjectSound {
                    id: Self::id_of(&b),
                    bytes: b.into(),
                },
            );
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_same_bytes_keep_one_name_and_a_taken_name_gets_a_number() {
        let mut s = ProjectSounds::default();
        assert_eq!(s.add("moeda.ogg", vec![1, 2, 3]), "moeda.ogg");
        assert_eq!(
            s.add("outra.ogg", vec![1, 2, 3]),
            "moeda.ogg",
            "o conteúdo decide"
        );
        assert_eq!(s.add("moeda.ogg", vec![9]), "moeda 2.ogg");
        assert_eq!(s.add("moeda.ogg", vec![8]), "moeda 3.ogg");
        assert_eq!(s.add("  ", vec![7]), "Sound");
        assert_eq!(s.len(), 4);
    }

    #[test]
    fn the_blob_round_trips_and_an_empty_library_writes_nothing() {
        assert!(ProjectSounds::default().to_bytes().is_empty());
        assert_eq!(
            ProjectSounds::from_bytes(&[]).unwrap(),
            ProjectSounds::default()
        );
        let mut s = ProjectSounds::default();
        s.add("a.wav", vec![1, 2]);
        s.add("b.ogg", vec![3]);
        let back = ProjectSounds::from_bytes(&s.to_bytes()).unwrap();
        assert_eq!(back, s);
        assert_eq!(back.get("a.wav").unwrap().id, ProjectSounds::id_of(&[1, 2]));
        assert!(ProjectSounds::from_bytes(b"lixo").is_err());
    }
}

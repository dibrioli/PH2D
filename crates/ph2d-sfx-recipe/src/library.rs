//! **As receitas e o material do projeto**, o campo `sfx_lab` do `ProjectFile` (blob `PH2DSFXL`, com versão
//! própria; vazio = nada a gravar). Fora do `ProjectState`, como os kanbans: o desfazer chega com a aba, em W3.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{Recipe, Take, Takes};

const MAGIC: &[u8; 8] = b"PH2DSFXL";
const VERSAO: u8 = 1;

#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "component", derive(bevy_ecs::resource::Resource))]
pub struct SfxLibrary {
    pub recipes: BTreeMap<String, Recipe>,
    pub takes: Takes,
}

/// O blob não é uma biblioteca legível.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryBlobError(pub String);

impl std::fmt::Display for LibraryBlobError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Serialize)]
struct Escrita<'a> {
    recipes: Vec<(&'a str, &'a Recipe)>,
    takes: Vec<(u32, u16, &'a [f32])>,
}

#[derive(Deserialize)]
struct Lida {
    recipes: Vec<(String, Recipe)>,
    takes: Vec<(u32, u16, Vec<f32>)>,
}

impl SfxLibrary {
    /// O blob a gravar: vazio quando não há nada, e o projeto sem receitas continua igual ao byte.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        if self.recipes.is_empty() && self.takes.is_empty() {
            return Vec::new();
        }
        let corpo = Escrita {
            recipes: self.recipes.iter().map(|(n, r)| (n.as_str(), r)).collect(),
            takes: (self.takes.iter())
                .map(|t| (t.rate, t.channels, &*t.samples))
                .collect(),
        };
        let mut out = MAGIC.to_vec();
        out.push(VERSAO);
        out.extend(postcard::to_allocvec(&corpo).unwrap_or_default());
        out
    }

    /// Lê o blob de [`Self::to_bytes`]. O endereço do material é recalculado, e toda receita passa pela lei dos valores.
    ///
    /// # Errors
    /// Cabeçalho, versão ou corpo ilegíveis.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, LibraryBlobError> {
        if bytes.is_empty() {
            return Ok(Self::default());
        }
        let corpo = bytes
            .strip_prefix(MAGIC.as_slice())
            .ok_or_else(|| LibraryBlobError("not a sound recipe library".into()))?;
        let (&versao, corpo) = corpo
            .split_first()
            .ok_or_else(|| LibraryBlobError("sound recipe library without a version".into()))?;
        if versao != VERSAO {
            return Err(LibraryBlobError(format!(
                "sound recipe library version {versao}, this app reads {VERSAO}"
            )));
        }
        let lida: Lida =
            postcard::from_bytes(corpo).map_err(|e| LibraryBlobError(e.to_string()))?;
        let mut out = Self::default();
        for (nome, mut r) in lida.recipes {
            r.sanitize();
            out.recipes.insert(nome, r);
        }
        for (rate, channels, samples) in lida.takes {
            out.takes.add(Take::new(rate, channels, samples));
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cheia() -> SfxLibrary {
        let mut lib = SfxLibrary::default();
        let id = lib.takes.add(Take::new(44_100, 2, vec![0.25, -0.25]));
        lib.recipes
            .insert("Take".into(), crate::factory::recorded(id, 0.1));
        for (n, r) in crate::factory::all() {
            lib.recipes.insert(n.into(), r);
        }
        lib
    }

    #[test]
    fn a_biblioteca_volta_igual_e_a_vazia_nao_escreve_nada() {
        assert!(SfxLibrary::default().to_bytes().is_empty());
        assert_eq!(SfxLibrary::from_bytes(&[]).unwrap(), SfxLibrary::default());
        let lib = cheia();
        assert_eq!(SfxLibrary::from_bytes(&lib.to_bytes()).unwrap(), lib);
    }

    #[test]
    fn blob_ilegivel_e_recusado_com_o_motivo() {
        assert!(SfxLibrary::from_bytes(b"lixo").is_err());
        let mut outra_versao = cheia().to_bytes();
        outra_versao[8] = 99;
        let e = SfxLibrary::from_bytes(&outra_versao).unwrap_err();
        assert!(e.0.contains("99"), "{e}");
    }

    /// O mesmo áudio entra uma vez, e o endereço muda com qualquer bit.
    #[test]
    fn o_material_e_endereçado_pelo_conteudo() {
        let mut t = Takes::default();
        let a = t.add(Take::new(48_000, 1, vec![0.5, 0.25]));
        assert_eq!(a, t.add(Take::new(48_000, 1, vec![0.5, 0.25])));
        assert_eq!(t.iter().count(), 1);
        assert_ne!(a, Take::new(44_100, 1, vec![0.5, 0.25]).id);
        assert_ne!(a, Take::new(48_000, 1, vec![0.5, 0.26]).id);
    }
}

/// A porta do projeto: a biblioteca do mundo em bytes e de volta.
#[cfg(feature = "component")]
pub mod project {
    use super::{LibraryBlobError, SfxLibrary};
    use bevy_ecs::world::World;

    /// O blob a gravar.
    #[must_use]
    pub fn save(world: &World) -> Vec<u8> {
        world
            .get_resource::<SfxLibrary>()
            .map(SfxLibrary::to_bytes)
            .unwrap_or_default()
    }

    /// # Errors
    /// O blob é ilegível: o load recusa o arquivo inteiro, como os sons.
    pub fn open(bytes: &[u8]) -> Result<SfxLibrary, LibraryBlobError> {
        SfxLibrary::from_bytes(bytes)
    }

    /// Substitui a biblioteca do documento anterior.
    pub fn install(world: &mut World, lib: SfxLibrary) {
        world.insert_resource(lib);
    }
}
